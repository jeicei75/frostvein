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

/// Sky-coloured pixels INSIDE the terrain silhouette -- a port of `10-7-signoff/holes.py`.
///
/// A hole is sky with terrain drawn around it. Horizon sky is not a hole, which is why the
/// silhouette is resolved per COLUMN (the topmost non-sky pixel of that column) rather than by a
/// y-threshold. The threshold version was tried first, counted horizon sky whose edge shifts between
/// builds, and reported an 884-pixel REGRESSION as an improvement.
fn interior_sky(pixels: &[[u8; 4]], width: usize, height: usize) -> usize {
    const SKY: [u8; 3] = [5, 12, 28];
    let mut count = 0;
    for x in 0..width {
        let mut inside = false;
        for y in 0..height {
            let p = pixels[y * width + x];
            let is_sky = [p[0], p[1], p[2]] == SKY;
            if !inside {
                inside = !is_sky;
            } else if is_sky {
                count += 1;
            }
        }
    }
    count
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
/// The permanent mask test proves the mesher emits the specific face a mesh-drawn tree must not
/// hide. It is a genuinely discriminating test and it is still geometry: it cannot see a hole opened
/// anywhere it was not pointed. This counts sky drawn INSIDE the silhouette across the whole frame,
/// which is what a hole actually is and what Wolf saw as hard black quads at the trunk bases.
#[test]
#[ignore = "renders real frames; scripts/gate.sh runs it in the full tier"]
fn the_fine_mesher_leaves_no_sky_showing_through_the_terrain() {
    let daemon = Daemon::spawn();
    let (fine, width, height) = daemon.capture("subdiv2", &["--subdiv", "2"]);
    let holes = interior_sky(&fine, width, height);
    println!("AC12 pixel guard: subdiv 2 interior-sky px = {holes}");

    // Hand-written, NOT derived from the mesher. The states this must separate, all measured on
    // 2026-09-03 with `10-7-signoff/holes.py`, run-to-run noise 12 px:
    //   fixed (this build)          12,277 - 12,363   <- must pass
    //   before the fix              12,722            <- must fail
    //   the REJECTED first fix      13,606            <- must fail
    // 12,600 clears the worst fixed reading by ~240 px and still fails the before-state by 122.
    // The residual ~12,300 is legitimate sky between real terrain, not holes, which is why this is
    // a calibrated ceiling and not zero.
    const INTERIOR_SKY_CEILING: usize = 12_600;
    assert!(
        holes <= INTERIOR_SKY_CEILING,
        "sky is showing through the terrain at --subdiv 2: {holes} interior-sky pixels, above the \
         {INTERIOR_SKY_CEILING} ceiling. A mesh-drawn tree cell is skipped by the terrain mesher, \
         so letting it suppress a neighbouring face leaves that face drawn by NOTHING and the \
         camera sees straight through to the sky."
    );
}
