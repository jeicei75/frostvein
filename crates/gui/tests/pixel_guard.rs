//! The only tests in this workspace that assert what the client actually DREW.
//!
//! Every other guard here stops one level short of the picture: `bench_contract.rs` greps source
//! text, `a_mesh_drawn_tree_hides_no_terrain_face` reads emitted mesh masks, and
//! `lighting_keys_change_the_live_scene_and_its_readout` reads the values handed to the renderer.
//! Each is the right test for its own question, and none of them can see a frame. Story 10.7 shipped
//! three findings that only a picture could catch -- black quads at trunk bases that every geometry
//! count called healthy, a campfire still glowing with its light switched off, and an "after the fix"
//! artifact that was really the rejected fix -- so the gap is measured, not theoretical.
//!
//! These run the REAL binaries: a real daemon, the real client, a real Vulkan device (lavapipe in the
//! devpod), and a decoded PNG. That costs about a minute each, which is why every test here is
//! `#[ignore]`d and `scripts/gate.sh` runs them explicitly in its FULL tier only. The fast tier names
//! them in its SKIPPED list, exactly as it names `serve.rs`: a check that did not run is a coverage
//! hole, never a clean result.
//!
//! ORACLES ARE HAND-WRITTEN AND DELIBERATELY NOT DERIVED from the constants they guard, in the style
//! of `APPROVED_PEAK` and `APPROVED_DOWNWARD_FLOOR`. Each names the measurement it was set from and
//! the states it must separate.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

/// `--frames 160` is the story's own capture recipe. It is not arbitrary: the capture's motion
/// health floor panics if too few ticks are observed, and software rendering observes roughly a
/// third of them.
const FRAMES: &str = "160";
const DAEMON_TIMEOUT: Duration = Duration::from_secs(30);

/// The luminance the story's `lumstats.py` computes, integer arithmetic and all, so a figure printed
/// by this guard can be compared directly against every figure in the story record.
fn mean_luminance(pixels: &[[u8; 4]]) -> f32 {
    let total: u64 = pixels
        .iter()
        .map(|p| (p[0] as u64 * 299 + p[1] as u64 * 587 + p[2] as u64 * 114) / 1000)
        .sum();
    total as f32 / pixels.len() as f32
}

/// Sky pixels that no path from the frame border can reach through sky -- a port of
/// `10-7-signoff/enclosed.py`, and the replacement for the silhouette count this file shipped with.
///
/// THE OLD ORACLE MEASURED THE WRONG QUANTITY. It resolved a column's silhouette as that column's
/// topmost non-sky pixel, and the night sky is a GRADIENT, so no column's top pixel is ever exactly
/// `SKY` and the silhouette landed at `y <= 19` in all 1,280 columns. What it counted was OPEN SKY:
/// 11,174 px of a frame holding 18,889 sky pixels, against 1,650 genuinely enclosed. Its delta still
/// tracked holes, which is why it looked like it worked -- but a delta can say "some closed" and
/// never "none left", and AC12 asks for gone. Story 10.7 read a delta as a level and shipped 54
/// holes under a green guard, until Wolf saw them from the seat.
///
/// A hole is a TOPOLOGICAL fact -- sky with terrain drawn all the way around it -- so this resolves
/// it as a flood fill from the border. Everything the fill reaches is open sky however deep in the
/// frame it looks; everything it cannot reach is a hole. Same-build noise floor: 0 px, twice, at
/// both subdivisions, against the old oracle's 45 px spread over eight readings.
fn enclosed_sky(pixels: &[[u8; 4]], width: usize, height: usize) -> (usize, usize) {
    const SKY: [u8; 3] = [5, 12, 28];
    let sky: Vec<bool> = pixels.iter().map(|p| [p[0], p[1], p[2]] == SKY).collect();
    let mut seen = vec![false; width * height];
    let mut queue = std::collections::VecDeque::new();
    let push = |i: usize, seen: &mut Vec<bool>, queue: &mut std::collections::VecDeque<usize>| {
        if sky[i] && !seen[i] {
            seen[i] = true;
            queue.push_back(i);
        }
    };
    for x in 0..width {
        push(x, &mut seen, &mut queue);
        push((height - 1) * width + x, &mut seen, &mut queue);
    }
    for y in 0..height {
        push(y * width, &mut seen, &mut queue);
        push(y * width + width - 1, &mut seen, &mut queue);
    }
    while let Some(i) = queue.pop_front() {
        let (x, y) = (i % width, i / width);
        if x > 0 {
            push(i - 1, &mut seen, &mut queue);
        }
        if x + 1 < width {
            push(i + 1, &mut seen, &mut queue);
        }
        if y > 0 {
            push(i - width, &mut seen, &mut queue);
        }
        if y + 1 < height {
            push(i + width, &mut seen, &mut queue);
        }
    }
    // Second pass: how many SEPARATE regions, which is where the discrimination is -- 38 trunk
    // holes came to only 135 pixels between them.
    let mut total = 0;
    let mut blobs = 0;
    for start in 0..width * height {
        if !sky[start] || seen[start] {
            continue;
        }
        blobs += 1;
        seen[start] = true;
        queue.push_back(start);
        while let Some(i) = queue.pop_front() {
            total += 1;
            let (x, y) = (i % width, i / width);
            if x > 0 {
                push(i - 1, &mut seen, &mut queue);
            }
            if x + 1 < width {
                push(i + 1, &mut seen, &mut queue);
            }
            if y > 0 {
                push(i - width, &mut seen, &mut queue);
            }
            if y + 1 < height {
                push(i + width, &mut seen, &mut queue);
            }
        }
    }
    (total, blobs)
}

struct Daemon {
    child: Child,
    port: u16,
}

impl Daemon {
    /// Binds port 0 and reports what it got, following `simd/tests/serve.rs`: reserving a port here
    /// and handing the number on loses races against sibling agents holding the fixed port.
    fn spawn() -> Self {
        let simd = PathBuf::from(env!("CARGO_BIN_EXE_gui"))
            .parent()
            .expect("the gui binary must sit in a directory")
            .join("simd");
        assert!(
            simd.exists(),
            "the daemon binary is missing at {simd:?}. `cargo test -p gui` alone does not build \
             simd; scripts/gate.sh runs the full `cargo test` first, which does. A missing daemon \
             is a COVERAGE HOLE and must fail loudly rather than skip."
        );
        let mut child = Command::new(&simd)
            .arg("0")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("simd must start");
        let stdout = child.stdout.take().expect("simd stdout must be piped");
        let (sender, receiver): (_, Receiver<String>) = channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let line = match receiver.recv_timeout(DAEMON_TIMEOUT) {
            Ok(line) => line,
            Err(error) => {
                let _ = child.kill();
                panic!("simd never printed its listening line: {error:?}");
            }
        };
        let port = line
            .trim_end()
            .strip_prefix("listening on 127.0.0.1:")
            .expect("simd must print the expected listening line")
            .parse()
            .expect("simd must print a numeric port");
        Self { child, port }
    }

    /// One real client run, decoded. The exit status is deliberately NOT asserted: the capture's
    /// near-white range check has been breached on `main` since before story 10.7 and exits 101,
    /// and `save_before_validate` writes the PNG before validating it. Asserting success here would
    /// make this guard fail for a reason that has nothing to do with the pixels it came to read --
    /// and "raise the ceiling so my run goes green" is exactly what 10.7's AC7 forbids.
    fn capture(&self, label: &str, extra: &[&str]) -> (Vec<[u8; 4]>, usize, usize) {
        let out = std::env::temp_dir().join(format!(
            "frostvein-pixel-guard-{}-{label}.png",
            std::process::id()
        ));
        let mut command = Command::new(env!("CARGO_BIN_EXE_gui"));
        command
            .arg(self.port.to_string())
            .args(["--headless", "--frames", FRAMES])
            .args(["--capture", out.to_str().expect("a utf-8 scratch path")])
            .args(extra)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let status = command.status().expect("the client must run");
        assert!(
            out.exists(),
            "the client wrote no PNG for {label} (exit {status:?}); every capture in this project \
             saves before it validates, so a missing file is a real failure, not the expected 101"
        );
        let image = image::open(&out)
            .unwrap_or_else(|error| panic!("the {label} capture must decode: {error}"))
            .to_rgba8();
        let (width, height) = (image.width() as usize, image.height() as usize);
        let pixels = image.pixels().map(|pixel| pixel.0).collect::<Vec<_>>();
        let _ = std::fs::remove_file(&out);
        (pixels, width, height)
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// AC11, on the frame rather than on the flag.
///
/// The permanent component test proves the toggles reach the values the renderer READS. It cannot
/// prove the renderer then draws differently, and it could not have caught what Wolf found from the
/// seat: with every source switched off the campfire still glowed, because a light-bearing entity
/// owns a `PointLight` AND a baked emissive face and only the light was being switched. `warm_lit`
/// is the instrument that sees it -- the emissive faces are `srgb_u8(255, 173, 92)`, unmistakably
/// warm -- and on the shipped build with everything off it must read exactly ZERO.
#[test]
#[ignore = "renders real frames; scripts/gate.sh runs it in the full tier"]
fn switching_every_light_off_darkens_the_frame_and_leaves_no_emitter_glowing() {
    let daemon = Daemon::spawn();
    let (lit, _, _) = daemon.capture("all-on", &["--subdiv", "1"]);
    let (dark, _, _) = daemon.capture(
        "all-off",
        &[
            "--subdiv",
            "1",
            "--lights-off",
            "sun,campfire,torches,lanterns,ambient",
        ],
    );

    let lit_mean = mean_luminance(&lit);
    let dark_mean = mean_luminance(&dark);
    let warm = gui::capture::warm_lit_pixels(&dark);
    println!(
        "AC11 pixel guard: all-on mean={lit_mean:.3} all-off mean={dark_mean:.3} \
         drop={:.3} warm-lit-when-dark={warm}",
        lit_mean - dark_mean
    );

    // Hand-written, NOT derived from any lighting constant. Measured 2026-09-03 on this build:
    // all-on 101.1, all-off 13.2, a drop of ~87.9, against a same-build noise floor of 0.16.
    // 40.0 sits far above the noise and far below the signal, so it separates "the lights do
    // work" from "the toggles are inert" without pinning today's exposure.
    const ALL_OFF_DROP_FLOOR: f32 = 40.0;
    assert!(
        lit_mean - dark_mean > ALL_OFF_DROP_FLOOR,
        "switching every source off must visibly darken the frame: {lit_mean:.3} -> {dark_mean:.3} \
         is a drop of {:.3}, at or under the {ALL_OFF_DROP_FLOOR} floor. A toggle that moves the \
         renderer's inputs but not the picture is the inert mechanism this guard exists for.",
        lit_mean - dark_mean
    );
    assert_eq!(
        warm, 0,
        "with every light off NOTHING may still glow. Wolf found this from the seat: \"if I turn \
         all lights off there is still light emitter in the campfire's place\". A source owns a \
         point light AND a baked emissive face; switching only the light leaves the face lit, and \
         {warm} warm pixels is that defect returning."
    );
}

/// AC12, on the pixels rather than on the mesh masks.
///
/// The permanent mask tests prove the mesher emits the specific faces a mesh-drawn tree must not
/// hide, and that the ground under a trunk reaches the mesher at all. They are genuinely
/// discriminating and they are still geometry: they cannot see a hole opened anywhere they were not
/// pointed. This one asks the frame.
#[test]
#[ignore = "renders real frames; scripts/gate.sh runs it in the full tier"]
fn the_fine_mesher_leaves_no_sky_showing_through_the_terrain() {
    let daemon = Daemon::spawn();
    let (fine, width, height) = daemon.capture("subdiv2", &["--subdiv", "2"]);
    let (holes, blobs) = enclosed_sky(&fine, width, height);
    println!("AC12 pixel guard: subdiv 2 enclosed-sky px = {holes} in {blobs} blobs");

    // Hand-written, NOT derived from the mesher. Every state this must separate, measured with
    // `10-7-signoff/enclosed.py`, same-build noise floor 0 px:
    //   this build                       2,042 px /  16 blobs   <- must pass
    //   the draw-set hole (10.7's first  2,177 px /  54 blobs   <- must fail
    //     fix, which Wolf's eye caught)
    //   before any 10.7 fix              2,571 px /  82 blobs   <- must fail
    //   the REJECTED first fix           3,449 px /  67 blobs   <- must fail
    //
    // THE BLOB COUNT IS THE PRIMARY BAR, because it is where the separation actually is: the
    // trunk-base family was 38 separate holes but only 135 pixels, so a pixel ceiling alone
    // discriminates it by a margin barely above nothing. The pixel ceiling is the second net,
    // for a regression that grows a hole rather than adding one.
    //
    // NEITHER CEILING IS ZERO, AND THAT IS CORRECT -- it is not a debt. The 2,042 px residual is
    // NOT holes: it is the sky BEYOND THE WORLD'S EDGE, where the terrain stops at its outer
    // boundary and pines carry on standing past the last terrain cell, so the canopy closes over
    // pockets of open sky and a border flood fill cannot reach them. The whole-world face oracle
    // reports zero missing faces after this fix, which is what rules out the alternative. Wolf
    // ruled at the 2026-09-04 sitting, having looked at `--subdiv` 1, 2 and 4: the holes are gone
    // and 16 is fine. So these ceilings are a property of this framing, not a bug waiting to be
    // fixed, and raising them to clear a failing run would still be forbidden.
    const BLOB_CEILING: usize = 20;
    const ENCLOSED_SKY_CEILING: usize = 2_300;
    assert!(
        blobs <= BLOB_CEILING,
        "sky is showing through the terrain at --subdiv 2: {blobs} separate enclosed-sky regions, \
         above the {BLOB_CEILING} ceiling. Every hole beyond the four of issue #65 is a terrain \
         face that nothing drew -- either a mesh-drawn tree cell suppressing a neighbour's face, \
         or the ground under a trunk never reaching the mesher at all."
    );
    assert!(
        holes <= ENCLOSED_SKY_CEILING,
        "sky is showing through the terrain at --subdiv 2: {holes} enclosed-sky pixels, above the \
         {ENCLOSED_SKY_CEILING} ceiling."
    );
}
