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

/// Mean Rec.601 luminance for a fixed screen rectangle, matching `11-1-signoff/creases.py`.
/// The literal windows and expected values are the hand-measured instrument contract, rather than
/// values derived from the camera's components: this test has to notice when Bevy silently skips
/// SSAO because MSAA was re-enabled.
fn rec601_mean(pixels: &[[u8; 4]], width: usize, rect: (usize, usize, usize, usize)) -> f32 {
    let (x0, y0, x1, y1) = rect;
    let mut total = 0_u64;
    for y in y0..y1 {
        for x in x0..x1 {
            let [r, g, b, _] = pixels[y * width + x];
            total += (r as u64 * 299 + g as u64 * 587 + b as u64 * 114) / 1000;
        }
    }
    total as f32 / ((x1 - x0) * (y1 - y0)) as f32
}

/// Fraction of a rectangle at or above near-white, as a percentage. `Bloom`'s composite mode is
/// what this separates: `EnergyConserving` moves energy OUT of bright cores, so it LOWERS this;
/// `Additive` adds energy and raises it. Nothing else in the frame distinguishes the two presets.
fn rec601_near_white_percent(
    pixels: &[[u8; 4]],
    width: usize,
    rect: (usize, usize, usize, usize),
) -> f32 {
    const NEAR_WHITE: u32 = 230;
    let (x0, y0, x1, y1) = rect;
    let mut hits = 0_u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let [r, g, b, _] = pixels[y * width + x];
            if (r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000 >= NEAR_WHITE {
                hits += 1;
            }
        }
    }
    100.0 * hits as f32 / ((x1 - x0) * (y1 - y0)) as f32
}

fn rec601_median(pixels: &[[u8; 4]], width: usize, rect: (usize, usize, usize, usize)) -> u8 {
    let (x0, y0, x1, y1) = rect;
    let mut values = Vec::with_capacity((x1 - x0) * (y1 - y0));
    for y in y0..y1 {
        for x in x0..x1 {
            let [r, g, b, _] = pixels[y * width + x];
            values.push(((r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000) as u8);
        }
    }
    values.sort_unstable();
    values[values.len() / 2]
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

    /// One real client run, decoded. The exit status is deliberately NOT asserted: the all-off
    /// capture always exits 101 because its intentional darkness trips `WARM_PIXEL_FLOOR`
    /// ("capture contains fewer than 3000 warm-lit pixels"), not the near-white ceiling.
    /// `write_png_before_validate` leaves its PNG on disk first. Asserting success here would make
    /// this guard fail for a reason that has nothing to do with the pixels it came to read -- and
    /// "raise the ceiling so my run goes green" is exactly what 10.7's AC7 forbids.
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

/// AC2 and AC3, on the rendered frame rather than on the camera components.
///
/// A DELTA between an AO-on and an AO-off capture, not a level against a fixed ceiling. The level
/// version shipped first and the code review measured what was wrong with it: bloom brightens this
/// same window by +1.77 Rec.601 while AO darkens it by only -0.62, so the AO-off case cleared the
/// 69.75 ceiling by 0.38 only because bloom was holding it up. With `--fx-off ao,bloom` the terrace
/// reads 68.403 -- under the ceiling with SSAO ABSENT. The guard survived that state only through
/// its open-snow clause, which the story describes as an unrelated control; any later change that
/// darkens this window by more than 0.38 would have taken the stated mechanism out entirely, and
/// 11.2 (fog) and 11.3 (day/night) are the next two stories. A delta cannot be propped up by an
/// effect it does not measure.
///
/// Measured on `6140ca3`, fresh daemon per capture, Rec.601 integer luma:
///   AO on   terrace mean 69.499 / 69.511   (same-build floor 0.012)
///   AO off  terrace mean 70.142 / 70.125   (same-build floor 0.017)
///   delta   0.62-0.64 against a floor of 0.017 -- about 36x
/// The 0.30 floor below sits roughly halfway, ~18x the noise and ~half the signal.
///
/// Re-enabling MSAA makes Bevy's SSAO extractor `return` out of its whole loop and skip EVERY
/// camera while the frame still renders and the process still exits 0. That restores the AO-off
/// reading on the AO-ON capture, collapsing the delta; this oracle is what notices.
#[test]
#[ignore = "renders two real frames; scripts/gate.sh runs it in the full tier"]
fn ambient_occlusion_darkens_terrace_creases_and_msaa_cannot_silently_disable_it() {
    const TERRACE: (usize, usize, usize, usize) = (860, 190, 1060, 290);
    const OPEN_SNOW_LL: (usize, usize, usize, usize) = (180, 620, 380, 700);
    const OPEN_SNOW_LR: (usize, usize, usize, usize) = (950, 590, 1150, 670);
    /// Minimum Rec.601 darkening SSAO must produce in the terrace window.
    const SSAO_TERRACE_DARKENING_FLOOR: f32 = 0.30;
    const CONTROL_OPEN_SNOW_MEDIAN: u8 = 116;

    // ONE DAEMON PER CAPTURE, and this is load-bearing for a delta. `--static-world` freezes the
    // world at whatever tick it has reached when the client connects, so a second capture against
    // a daemon that has been running through the first freezes a LATER world -- different dwarf
    // positions, measured as if they were the effect. A freshly spawned daemon lands on the same
    // tick every time (measured: tick 40, four runs in a row).
    let (on, width, _height) = Daemon::spawn().capture(
        "ambient-occlusion-on",
        &["--static-world", "--lights-steady", "--subdiv", "4"],
    );
    let (off, _width, _height) = Daemon::spawn().capture(
        "ambient-occlusion-off",
        &[
            "--static-world",
            "--lights-steady",
            "--subdiv",
            "4",
            "--fx-off",
            "ao",
        ],
    );

    let on_terrace = rec601_mean(&on, width, TERRACE);
    let off_terrace = rec601_mean(&off, width, TERRACE);
    let darkening = off_terrace - on_terrace;
    let open_snow_ll_median = rec601_median(&on, width, OPEN_SNOW_LL);
    let open_snow_lr_median = rec601_median(&on, width, OPEN_SNOW_LR);
    println!(
        "AC2/AC3 pixel guard (Rec.601): terrace mean AO-on={on_terrace:.3} AO-off={off_terrace:.3} \
         darkening={darkening:.3}; open-snow LL/LR median={open_snow_ll_median}/{open_snow_lr_median}"
    );

    assert!(
        darkening >= SSAO_TERRACE_DARKENING_FLOOR,
        "SSAO must darken the terrace creases: AO-on mean={on_terrace:.3}, AO-off mean={off_terrace:.3}, \
         darkening={darkening:.3}, below the {SSAO_TERRACE_DARKENING_FLOOR:.2} floor. Either SSAO is \
         not reaching the camera, or MSAA is silently disabling Bevy's SSAO extractor."
    );
    // NOT an AO-independent control, and the review measured that: AO moves these medians 117 -> 116
    // on its own, by the same ~0.6 Rec.601 it moves the terrace. What they pin is the POST-STACK
    // state of a window with no emitter in it, so a change that brightens flat snow is caught. The
    // "AO acts on creases, not globally" reading this clause once carried is NOT supported --
    // issue #106 carries that AC-bar defect.
    assert_eq!(
        open_snow_ll_median, CONTROL_OPEN_SNOW_MEDIAN,
        "the all-effects-on open-snow-LL median must hold at its post-stack control"
    );
    assert_eq!(
        open_snow_lr_median, CONTROL_OPEN_SNOW_MEDIAN,
        "the all-effects-on open-snow-LR median must hold at its post-stack control"
    );
}

/// AC4, on the rendered frame. The guard the Project Structure table promised and the story never
/// wrote: before this, bloom's only regression net was a component-presence assertion, so a preset
/// change or a silent post-process skip was caught by no pixel anywhere.
///
/// `--fx-off bloom` is the control, and it is a control for BLOOM'S MARGINAL contribution only:
/// removing the `Bloom` component does not remove the `Hdr` it `#[require]`s, so `Hdr` is on in
/// both halves. That is deliberate and is why `--fx-off bloom` uses a plain `remove` while
/// `--fx-off ao` also drops its prepasses.
///
/// Measured on `6140ca3`, fresh daemon per capture, camp window (500,400)-(760,620), Rec.601:
///   bloom on   median 95 / 95     mean 120.334 / 120.288   (floors 0 and 0.046)
///   bloom off  median 82 / 82     mean 113.427 / 113.435   (floors 0 and 0.008)
///   delta      median +13, mean +6.88
/// The floors below are ~half the signal and orders above the noise.
///
/// The HALO, not the bright tail. `Bloom::default()` is `NATURAL`/`EnergyConserving`, which
/// REDISTRIBUTES energy out of bright cores rather than adding any, so p90 actually FALLS by a
/// level or two. An AC asking the bright tail to rise was unsatisfiable by construction; this
/// measures the signature the chosen composite mode actually has.
#[test]
#[ignore = "renders two real frames; scripts/gate.sh runs it in the full tier"]
fn bloom_lifts_the_camp_halo_without_brightening_open_snow() {
    const CAMP: (usize, usize, usize, usize) = (500, 400, 760, 620);
    const OPEN_SNOW_LL: (usize, usize, usize, usize) = (180, 620, 380, 700);
    const OPEN_SNOW_LR: (usize, usize, usize, usize) = (950, 590, 1150, 670);
    /// Minimum Rec.601 rise bloom must produce in the camp halo.
    const BLOOM_HALO_MEDIAN_FLOOR: i32 = 6;
    const BLOOM_HALO_MEAN_FLOOR: f32 = 3.0;

    // One daemon per capture: the camp window is where the lantern-carrying dwarves walk, so a
    // shared daemon's later freeze tick lands them somewhere else and the delta measures that
    // instead of bloom. This is issue #105's mechanism, one level down.
    let (on, width, _height) = Daemon::spawn().capture(
        "bloom-on",
        &["--static-world", "--lights-steady", "--subdiv", "4"],
    );
    let (off, _width, _height) = Daemon::spawn().capture(
        "bloom-off",
        &[
            "--static-world",
            "--lights-steady",
            "--subdiv",
            "4",
            "--fx-off",
            "bloom",
        ],
    );

    let on_median = i32::from(rec601_median(&on, width, CAMP));
    let off_median = i32::from(rec601_median(&off, width, CAMP));
    let on_mean = rec601_mean(&on, width, CAMP);
    let off_mean = rec601_mean(&off, width, CAMP);
    let median_rise = on_median - off_median;
    let mean_rise = on_mean - off_mean;
    println!(
        "AC4 pixel guard (Rec.601): camp median bloom-on={on_median} bloom-off={off_median} \
         rise={median_rise}; camp mean bloom-on={on_mean:.3} bloom-off={off_mean:.3} rise={mean_rise:.3}"
    );

    assert!(
        median_rise >= BLOOM_HALO_MEDIAN_FLOOR,
        "bloom must lift the camp halo: median rose {median_rise} (on={on_median}, off={off_median}), \
         below the {BLOOM_HALO_MEDIAN_FLOOR} floor. Bloom is not reaching the camera, or the post-process \
         stack is being skipped."
    );
    assert!(
        mean_rise >= BLOOM_HALO_MEAN_FLOOR,
        "bloom must lift the camp halo: mean rose {mean_rise:.3} (on={on_mean:.3}, off={off_mean:.3}), \
         below the {BLOOM_HALO_MEAN_FLOOR:.1} floor."
    );
    // THE PRESET, not just the presence. A swap to `OLD_SCHOOL` leaves the component in place and
    // still lifts the halo -- it is `Additive`, so it ADDS energy rather than redistributing it --
    // and it SURVIVED this guard's first version, which is how this assertion came to exist. The
    // near-white fraction is what separates the two composite modes: `EnergyConserving` moves
    // energy out of the bright cores and LOWERS it, `Additive` raises it.
    // Measured on `6140ca3` WITH `--lights-steady`: bloom-on 5.1573 / 5.1766 %, bloom-off
    // 5.4930 / 5.4738 % -- a fall of about 0.32 pp against a same-build floor of 0.019.
    // THE FLAG IS REQUIRED FOR THIS CLAUSE and the guard's first version omitted it. The camp is
    // the emitter window: its near-white swings about 1.44 pp between same-build captures while
    // the flicker runs free, which is FOUR TIMES this signal. Run unpinned once, this assertion
    // read bloom-on 5.5839 % against bloom-off 4.5577 % -- bloom apparently RAISING near-white by
    // 1.03 pp -- and failed. It was measuring the flicker phase, not the composite mode.
    let on_near_white = rec601_near_white_percent(&on, width, CAMP);
    let off_near_white = rec601_near_white_percent(&off, width, CAMP);
    println!(
        "AC4 pixel guard (Rec.601): camp near-white bloom-on={on_near_white:.4}% \
         bloom-off={off_near_white:.4}%"
    );
    assert!(
        off_near_white - on_near_white >= 0.10,
        "bloom must REDISTRIBUTE energy out of the bright cores, not add it: camp near-white went \
         {off_near_white:.4}% -> {on_near_white:.4}%. A rise means the composite mode is no longer \
         EnergyConserving -- check the preset."
    );

    // Only emitters and their halo may BRIGHTEN. These windows hold no emitter; bloom darkens them
    // by a level, which is the EnergyConserving signature, and the bar is that they do not rise.
    for (name, rect) in [("LL", OPEN_SNOW_LL), ("LR", OPEN_SNOW_LR)] {
        let on_snow = i32::from(rec601_median(&on, width, rect));
        let off_snow = i32::from(rec601_median(&off, width, rect));
        assert!(
            on_snow <= off_snow,
            "bloom must not brighten emitter-free open snow: {name} median went {off_snow} -> {on_snow}"
        );
    }
}

impl Daemon {
    /// One real client run, returning its `gui dwarves:` line. Stderr is captured rather than
    /// discarded, which is where the startup instrument prints.
    ///
    /// THE EXIT STATUS IS ASSERTED, NOT DISCARDED. It was discarded, and that hid a client which
    /// panicked on every run of both callers below: they passed `--frames 60` while the capture's
    /// own floor demands 100 delivered ticks, so the process died moments after printing the line
    /// these tests grep for. Every assertion passed on the output of a crashed process, which is
    /// manufactured evidence rather than a missing test -- the worse of the two.
    fn dwarf_report(&self, extra: &[&str]) -> String {
        // The scratch name is derived from `extra`, which carries an ABSOLUTE PATH under
        // `--assets`. Joining that raw put `/` inside the file NAME, so `PathBuf::join` addressed
        // directories that do not exist and the screenshot was silently never written.
        let slug = extra
            .join("_")
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>();
        let out = std::env::temp_dir().join(format!(
            "frostvein-dwarf-report-{}-{slug}.png",
            std::process::id(),
        ));
        let result = Command::new(env!("CARGO_BIN_EXE_gui"))
            .arg(self.port.to_string())
            .args([
                "--headless",
                "--capture",
                out.to_str().expect("a utf-8 path"),
            ])
            .args(extra)
            .stdout(Stdio::null())
            .output()
            .expect("the client must run");
        let _ = std::fs::remove_file(&out);
        let stderr = String::from_utf8_lossy(&result.stderr).into_owned();
        assert!(
            result.status.success(),
            "the client must exit cleanly for its startup line to be evidence of anything; it \
             exited {:?} with args {extra:?}\n{stderr}",
            result.status.code()
        );
        stderr
            .lines()
            .find(|line| line.starts_with("gui dwarves:"))
            .unwrap_or("<no dwarf line printed>")
            .to_string()
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// AC10: the dwarf's startup line, driven through the real binary and proved to MOVE.
///
/// A well-formedness assertion would pass against a hardcoded string. The state this varies is the
/// slice: the dwarves stand at z 9, so a run cut at z 5 draws none of them. Same binary, same
/// daemon, one flag apart.
///
/// The zero case is the one worth having. Reporting `meshes=0` is the instrument admitting it drew
/// nothing; going SILENT there is what it did before the tree and dwarf reports were given separate
/// readiness flags, and a line that only appears when things worked is not an instrument.
///
/// It costs a 600-frame run: below the cut there are no dwarves, so the report waits out
/// `TREE_REPORT_DEADLINE_FRAMES` rather than firing on success.
#[test]
#[ignore = "drives the real binary; scripts/gate.sh runs it in the full tier"]
fn the_dwarf_startup_line_reports_what_was_actually_drawn() {
    let daemon = Daemon::spawn();

    let above = daemon.dwarf_report(&["--subdiv", "1", "--z", "9", "--frames", FRAMES]);
    // Was 700 frames. Below the cut there is no dwarf to report on, so this run cannot fire on
    // success and must wait out the report deadline -- which used to be counted only in FRAMES and
    // so cost this software renderer over three minutes (0.313 s/frame, measured 2026-09-08). The
    // deadline now also trips on wall clock, so FRAMES is past it with room to spare.
    let below = daemon.dwarf_report(&["--subdiv", "1", "--z", "5", "--frames", FRAMES]);
    println!("AC10 dwarf line: above the cut {above:?} / below {below:?}");

    assert_eq!(
        above, "gui dwarves: meshes=5 scenes_loaded=true source=embedded",
        "with the slice at the dwarves' own level the line must name all five and say the \
         authored scene loaded"
    );
    assert_eq!(
        below, "gui dwarves: meshes=0 scenes_loaded=true source=embedded",
        "cut below them the line must still appear and report ZERO. Silence here would mean the \
         instrument only speaks when it has good news."
    );
    assert_ne!(
        above, below,
        "the line must change with the state it claims to report"
    );
}

/// Issue #77: a capture cut below every dwarf must not demand motion its own slice cannot draw.
#[test]
#[ignore = "drives the real binary; scripts/gate.sh runs it in the full tier"]
fn a_capture_below_the_dwarves_skips_motion_but_still_writes_a_png() {
    let daemon = Daemon::spawn();
    let out = std::env::temp_dir().join(format!(
        "frostvein-below-dwarf-capture-{}.png",
        std::process::id()
    ));
    let result = Command::new(env!("CARGO_BIN_EXE_gui"))
        .arg(daemon.port.to_string())
        .args(["--headless", "--capture"])
        .arg(out.to_str().expect("a utf-8 scratch path"))
        // FRAMES, not 700: the report deadline now trips on wall clock too. See the note in
        // `the_dwarf_startup_line_reports_what_was_actually_drawn`.
        .args(["--subdiv", "1", "--z", "5", "--frames", FRAMES])
        .output()
        .expect("the client must run");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        result.status.success(),
        "a capture below every dwarf must exit cleanly; it exited {:?}\n{stderr}",
        result.status.code()
    );
    assert!(
        out.exists(),
        "the below-dwarf capture must leave its requested PNG on disk"
    );
    std::fs::remove_file(out).expect("the temporary capture must be removable");
}

/// AC6: the startup line reports the RESOLVED asset source, and it MOVES.
///
/// `source=embedded` used to be a hardcoded word. It printed `embedded` with `--assets` pointed
/// anywhere, which is the same shape as 10.1's constant guard that stayed green while the bench
/// camera was rolled 110 degrees: text the mechanism cannot move.
///
/// THE ASSERTION THAT MATTERS IS `scenes_loaded=true` ON THE DISK RUN, not the changed label. A
/// label can be made to move by printing a different string; `scenes_loaded=true` can only be true
/// if the client actually resolved and decoded a `.glb` through the disk path. The directory is
/// this repo's own `assets/`, so the BYTES are identical to the embedded ones and the only thing
/// under test is where they were read from -- AC2 is the separate measurement that different bytes
/// produce a different frame.
#[test]
#[ignore = "drives the real binary; scripts/gate.sh runs it in the full tier"]
fn the_startup_line_reports_the_resolved_asset_source() {
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .canonicalize()
        .expect("the repo's own assets/ directory must exist");
    let daemon = Daemon::spawn();

    let embedded = daemon.dwarf_report(&["--subdiv", "1", "--z", "9", "--frames", FRAMES]);
    let from_disk = daemon.dwarf_report(&[
        "--subdiv",
        "1",
        "--z",
        "9",
        "--frames",
        FRAMES,
        "--assets",
        assets.to_str().expect("a utf-8 path"),
    ]);
    println!("AC6 source line: embedded {embedded:?} / disk {from_disk:?}");

    assert_eq!(
        embedded, "gui dwarves: meshes=5 scenes_loaded=true source=embedded",
        "with no flag the client must read the blobs compiled into it, and say so"
    );
    assert_eq!(
        from_disk,
        format!(
            "gui dwarves: meshes=5 scenes_loaded=true source=disk:{}",
            assets.display()
        ),
        "under --assets the line must name the directory actually read -- and `scenes_loaded=true` \
         is the half that cannot be faked by printing a different string"
    );
    assert_ne!(
        embedded, from_disk,
        "the line must change when the source changes; it used to print `embedded` either way"
    );
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

// AC12's frame-level guard is RETIRED. Issue #108, Wolf's ruling 2026-09-19.
//
// `the_fine_mesher_leaves_no_sky_showing_through_the_terrain` resolved sky with an EXACT RGB match
// (`const SKY: [u8; 3] = [5, 12, 28]`). `Hdr` moved the rendered sky to `[7, 15, 31]`, so by 11.1b
// ZERO pixels in a frame classified as sky where 8,434 had pre-`Hdr`. The flood fill then had
// nothing to fill and the guard returned 0 holes / 0 blobs -- not because the terrain was sound,
// but because it could no longer SEE sky. It would have passed with the terrain entirely absent.
// That is worse than the vacuous ceiling it was first filed as: a green line asserting nothing.
//
// It is retired rather than repaired because repairing it is a story, not a re-baseline. The old
// oracle worked only because the night sky was a single exact colour over a large flat region, and
// the post-stack destroyed that property: dark shadowed terrain and night sky now OVERLAP in
// colour space. Measured attempts, both rejected -- a self-calibrated exact match fragments the
// fill's connectivity (the sky is dithered across many near-values), and a tolerant rule
// calibrated from the frame's own top rows reports ~21,000 false holes in ~790 blobs.
//
// WHAT IS STILL COVERED: the permanent mask tests, which prove the mesher emits the faces a
// mesh-drawn tree must not hide and that the ground under a trunk reaches the mesher at all. They
// are geometry, they are genuinely discriminating, and the whole-world face oracle still reports
// zero missing faces. WHAT IS NOT: a hole opened anywhere nothing was pointed at. That is the gap
// #108 carries, and nothing here should be read as covering it.

/// AC9's header clause, on the real binary.
///
/// This is also the ONLY thing pinning `--perf-log`'s wiring. The `PerfLog` resource was inserted
/// in `run()`, outside the extracted builder, so deleting the two lines left the entire suite
/// green while the flag parsed, validated and reached nothing — the project's own named
/// antipattern, ruled closed at the root after five Milestone 2 instances, and this was a sixth.
///
/// The preamble is asserted FIELD BY FIELD rather than as a whole line. A log is read on a
/// different day and a different machine than it was written on, and each of these is a fact the
/// numbers below it are meaningless without: which build, which asset tree, how much geometry per
/// cell, and whether a vsync cap meant the run measured the monitor rather than the scene.
#[test]
#[ignore = "drives the real binary; scripts/gate.sh runs it in the full tier"]
fn the_perf_log_names_the_run_that_produced_it() {
    let daemon = Daemon::spawn();
    let log = std::env::temp_dir().join(format!("frostvein-perf-run-{}.csv", std::process::id()));
    let out = std::env::temp_dir().join(format!("frostvein-perf-run-{}.png", std::process::id()));
    let _ = std::fs::remove_file(&log);

    let result = Command::new(env!("CARGO_BIN_EXE_gui"))
        .arg(daemon.port.to_string())
        .args([
            "--headless",
            "--capture",
            out.to_str().expect("a utf-8 path"),
            "--subdiv",
            "1",
            "--z",
            "9",
            "--frames",
            FRAMES,
            "--perf-log",
            log.to_str().expect("a utf-8 path"),
        ])
        .stdout(Stdio::null())
        .output()
        .expect("the client must run");
    let stderr = String::from_utf8_lossy(&result.stderr).into_owned();
    let _ = std::fs::remove_file(&out);
    assert!(
        result.status.success(),
        "the client must exit cleanly; it exited {:?}\n{stderr}",
        result.status.code()
    );

    let written = std::fs::read_to_string(&log).expect("--perf-log must have written a file");
    let _ = std::fs::remove_file(&log);
    println!(
        "AC9 preamble: {:?}",
        written.lines().find(|line| line.starts_with("# run:"))
    );

    assert_eq!(
        written.lines().next(),
        Some(gui::perf::CSV_HEADER),
        "a file that exists must always be a file with a schema, on its very first line"
    );
    let preamble = written
        .lines()
        .find(|line| line.starts_with("# run:"))
        .expect("the log must name the run that produced it (AC9)");
    for field in [
        "build=",
        "assets=embedded",
        "subdiv=1",
        "vsync=off",
        "terrain=",
        "trees=",
        "dwarves=",
    ] {
        assert!(
            preamble.contains(field),
            "the run preamble must carry {field:?}; got {preamble:?}"
        );
    }
    let rows = written
        .lines()
        .filter(|line| !line.starts_with('#') && !line.starts_with("frame"))
        .count();
    assert!(
        rows > 1,
        "a {FRAMES}-frame run must leave more than one measured row; got {rows}"
    );
}

/// AC11's live half, and the seam the review found untested: the framing a capture failure names
/// must come off the RIG THAT TOOK IT.
///
/// `capture_after_frames` resolves it with
/// `cameras.iter().next().map_or_else(|| "camera: unavailable", camera_readout_line)`, and nothing
/// exercised that expression: `tests/capture.rs` hand-writes `NEAR_FRAMING` and the `src/capture.rs`
/// tests hand-write `TEST_FRAMING`, so replacing the whole expression with its literal fallback
/// left every capture reporting a framing it was not taken at, with the suite green. A broken
/// observability instrument is patched here regardless of severity — it manufactures false
/// evidence rather than merely missing true evidence.
///
/// Driven through the real binary because that is the only place the live query exists. The
/// framing is asserted in the rig's OWN formatting (`pitch=1`, not the `1` that was typed), which
/// is what makes this evidence that the line was built from the camera rather than echoed back
/// from argv.
///
/// The band that trips is the WARM-PIXEL FLOOR, not the near-white ceiling AC11 names: pointed up
/// and away from the camp, this framing has no warm light in it at all (`warm-lit pixels=0`).
/// That is the review's other finding in the same place — the ceiling asserts LAST, and for most
/// `--camera` framings one of the earlier bands fires first, so all five name the framing now.
#[test]
#[ignore = "drives the real binary; scripts/gate.sh runs it in the full tier"]
fn a_capture_failure_names_the_framing_the_live_rig_was_actually_at() {
    const FRAMING: &str = "--camera 2.5,1,40,20,100,12";
    let daemon = Daemon::spawn();
    let out = std::env::temp_dir().join(format!("frostvein-framing-{}.png", std::process::id()));
    let result = Command::new(env!("CARGO_BIN_EXE_gui"))
        .arg(daemon.port.to_string())
        .args([
            "--headless",
            "--static-world",
            "--subdiv",
            "4",
            "--frames",
            FRAMES,
            "--camera",
            "2.5,1,40,20,100,12",
            "--capture",
            out.to_str().expect("a utf-8 path"),
        ])
        .stdout(Stdio::null())
        .output()
        .expect("the client must run");
    let stderr = String::from_utf8_lossy(&result.stderr).into_owned();
    assert!(
        out.exists(),
        "every capture in this project saves before it validates, so a missing PNG is a real \
         failure rather than the expected 101\n{stderr}"
    );
    let _ = std::fs::remove_file(&out);
    assert_eq!(
        result.status.code(),
        Some(101),
        "this framing has no warm light in it; the run must die on a band, or there is no failure \
         message to read\n{stderr}"
    );
    assert!(
        stderr.contains("fewer than"),
        "the warm-pixel floor must be the band that tripped; got\n{stderr}"
    );
    assert!(
        stderr.contains(FRAMING),
        "the failure must name the framing it was taken at, pasteable as {FRAMING}; got\n{stderr}"
    );
    // NOT the boot rig, and not the fallback: both would still be a string, and both would be a
    // lie about which view produced the frame.
    assert!(
        !stderr.contains("--camera 0.7,0.45,90,64,64,9") && !stderr.contains("camera: unavailable"),
        "the framing must be the live rig's, not the boot default or the unavailable fallback; \
         got\n{stderr}"
    );
}
