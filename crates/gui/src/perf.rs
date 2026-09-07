//! The performance log: frame cost written to a file, to be read AFTER a run.
//!
//! WHY A LOG AND NOT THE OVERLAY. `F3` already shows fps live, and that is the wrong instrument for
//! the question being asked — a number on screen has to be watched while it happens, and the run
//! where something felt wrong is over by the time you want to look at it. This writes a row per
//! frame so the run can be examined afterwards, at leisure, with the numbers that were actually
//! true rather than the ones a human caught sight of.
//!
//! WHY CONTENT COUNTERS SIT BESIDE THE TIME. A timing column alone cannot be told apart from a
//! broken instrument. This project has measured ~140 fps surviving a 39% triangle cut, because the
//! terrain was never rasterised — the timing was flat and the flatness WAS the finding, invisible
//! without something saying whether the scene had changed. `terrain`, `trees` and `dwarves` move
//! when the drawn world moves, so a flat frametime beside a changed scene is a result and a flat
//! frametime beside an unchanged one is a tautology.
//!
//! WHY `dirty_tiles` IS ITS OWN COLUMN. The expensive frame in this game is not the steady one, it
//! is the one where a dug tile forces a re-mesh. 10.6 measured everything about a still scene and
//! missed that entirely. Averaged together the edit cost disappears into the steady state, so the
//! summariser splits the two populations on this column and reports them separately.
//!
//! WHY EVERY ROW IS WRITTEN AS IT HAPPENS. Batching was tried first and it cost 60 of 300 frames on
//! the very first real run. The client's exit paths include a PANIC — the capture range-check exits
//! 101 on `main` — and a panic runs no destructors and never returns from `app.run()`, so a
//! buffered tail is simply gone, and a short file reads as a short RUN rather than as a truncated
//! one. That is the worst way for an instrument to be wrong. One `write` of ~40 bytes per frame
//! costs microseconds against a frame of milliseconds, so durability is bought for well under a
//! tenth of a percent — and there is then no tail, no batch boundary, and no flush path that can
//! silently go unreached.

//! NOTE: A FRAMETIME FROM THIS DEVPOD IS NOT A PERFORMANCE STATEMENT. There is no GPU here --
//! rendering goes through `llvmpipe`, a software rasteriser -- so the numbers this writes on the
//! devpod measure a CPU emulating a graphics card. A review run measured p50 = 574.63 ms and a
//! debug build has read over 1,100 ms; neither is a fact about the game. The real logs come from
//! the vehicle, and this file's own tests pin the ARITHMETIC against synthetic rows precisely
//! because the numbers cannot be checked where they are written.
//!
//! Task 7 asked for this caveat and it went to `HeadlessTarget`'s doc instead -- one hop from the
//! instrument that actually emits the milliseconds, which is where somebody reading a p50 will be.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bevy::prelude::{Res, ResMut, Resource};

pub const CSV_HEADER: &str = "frame,t_ms,frametime_ms,terrain,trees,dwarves,dirty_tiles,mark";

/// What produced this log, written as a single `#` preamble line (AC9).
///
/// WHY IT IS WRITTEN ON THE FIRST FRAME AND NOT AT OPEN. Three of its seven facts are content
/// counts, and nothing has been drawn when the file is created. Writing the preamble lazily keeps
/// the column header exactly where it was -- a file that exists is still always a file with a
/// schema, which is the property `the_header_is_written_when_the_log_opens...` pins -- and still
/// puts the provenance above every measured row.
///
/// WHY IT EXISTS AT ALL. A log is read on a different day than it is written, from a different
/// machine than it was measured on. Without this line it cannot say which build, which asset tree,
/// which subdivision, or whether a vsync cap meant the run measured the monitor rather than the
/// scene -- and this project has already published one fps figure that never moved because the
/// terrain was not being rasterised at all.
#[derive(Debug, Clone)]
pub struct RunProvenance {
    pub build: String,
    pub assets: String,
    pub subdiv: u32,
    pub vsync: bool,
}

impl RunProvenance {
    fn line(&self, counts: FrameCounts) -> String {
        format!(
            "# run: build={} assets={} subdiv={} vsync={} terrain={} trees={} dwarves={}",
            self.build,
            self.assets,
            self.subdiv,
            if self.vsync { "on" } else { "off" },
            counts.terrain,
            counts.trees,
            counts.dwarves,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerfRow {
    pub frame: u64,
    pub t_ms: f64,
    pub frametime_ms: f64,
    pub terrain: usize,
    pub trees: usize,
    pub dwarves: usize,
    pub dirty_tiles: usize,
    /// Set on the frame the operator pressed the mark key. Finding "the bit that felt bad" in a
    /// ten-minute log is otherwise a search with no landmark.
    pub mark: bool,
}

impl PerfRow {
    pub fn to_csv(self) -> String {
        format!(
            "{},{:.3},{:.3},{},{},{},{},{}",
            self.frame,
            self.t_ms,
            self.frametime_ms,
            self.terrain,
            self.trees,
            self.dwarves,
            self.dirty_tiles,
            u8::from(self.mark)
        )
    }
}

/// What the drawn world contained on this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FrameCounts {
    /// Terrain DRAW entities, which are not the same thing in both modes: `--subdiv 1` draws one
    /// `TerrainTile` per exposed cell (40,148 in the shipped valley) and `--subdiv N>1` draws
    /// `TerrainChunk` partitions instead. Both are summed, because a column that silently counts
    /// nothing in one of the two modes is worse than no column.
    ///
    /// NOT named `chunks`: it was, and it read 40,148 on the first real run — a plausible-looking
    /// number under a name that made it mean something else entirely.
    pub terrain: usize,
    pub trees: usize,
    pub dwarves: usize,
    /// Tiles the daemon reported changed this frame. `0` is a steady-state frame; anything else is
    /// an EDIT frame and belongs to the other population.
    ///
    /// NOTE: this is dirty TILES, not the chunks rebuilt from them — `reconcile` does not report a
    /// chunk count and giving it one would change a signature with many callers for a refinement
    /// nothing has asked for yet. Tiles answer the question the split actually needs, which is
    /// whether this frame did re-mesh work at all.
    pub dirty_tiles: usize,
}

/// The open log for one run.
#[derive(Resource)]
pub struct PerfLog {
    path: PathBuf,
    file: Option<File>,
    /// Set by the FIRST recorded frame, not by the constructor. `t_ms` then means "since the first
    /// frame", which is both the more useful anchor and the only one a test can control — anchoring
    /// it to the open would silently fold `File::create`'s duration into every row.
    started: Option<Instant>,
    last_frame_at: Option<Instant>,
    frame: u64,
    /// Set by the mark key, consumed by the next row.
    pending_mark: bool,
    marks: u64,
    /// Reported once and then suppressed: a write that fails usually fails every frame, and sixty
    /// copies a second of one error buries the run's own output.
    reported_error: bool,
    /// Taken and written by the first recorded frame, which is the first moment the content counts
    /// in it are real.
    run: Option<RunProvenance>,
}

impl PerfLog {
    /// Open the log and write its header, so a file that exists is always a file with a schema.
    pub fn new(path: PathBuf) -> Self {
        let mut log = Self {
            path,
            file: None,
            started: None,
            last_frame_at: None,
            frame: 0,
            pending_mark: false,
            marks: 0,
            reported_error: false,
            run: None,
        };
        match File::create(&log.path) {
            Ok(mut file) => match writeln!(file, "{CSV_HEADER}") {
                Ok(()) => log.file = Some(file),
                Err(error) => eprintln!("gui perf-log: cannot write the header: {error}"),
            },
            Err(error) => eprintln!(
                "gui perf-log: cannot create {}: {error}",
                log.path.display()
            ),
        }
        log
    }

    /// Attach the run's provenance. A SETTER rather than a second constructor argument: every
    /// existing call site is a test that has no build stamp, no asset tree and no window to ask
    /// about vsync, and giving them one would be inventing the facts this line exists to record.
    pub fn with_run(mut self, run: RunProvenance) -> Self {
        self.run = Some(run);
        self
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn frames_recorded(&self) -> u64 {
        self.frame
    }

    pub fn marks(&self) -> u64 {
        self.marks
    }

    pub fn mark_next_frame(&mut self) {
        self.pending_mark = true;
    }

    /// Build the row for this frame. `now` is injected so a test drives exact intervals rather than
    /// racing the wall clock — an instrument whose own test is timing-dependent is not fixed.
    pub fn row(&mut self, now: Instant, counts: FrameCounts) -> PerfRow {
        let started = *self.started.get_or_insert(now);
        let frametime = self
            .last_frame_at
            .map_or(0.0, |last| now.duration_since(last).as_secs_f64() * 1000.0);
        self.last_frame_at = Some(now);
        let mark = std::mem::take(&mut self.pending_mark);
        if mark {
            self.marks += 1;
        }
        let row = PerfRow {
            frame: self.frame,
            t_ms: now.duration_since(started).as_secs_f64() * 1000.0,
            frametime_ms: frametime,
            terrain: counts.terrain,
            trees: counts.trees,
            dwarves: counts.dwarves,
            dirty_tiles: counts.dirty_tiles,
            mark,
        };
        self.frame += 1;
        row
    }

    /// Record and write one frame. There is no buffer, so there is no tail to lose.
    pub fn record(&mut self, now: Instant, counts: FrameCounts) {
        let row = self.row(now, counts);
        let preamble = self.run.take().map(|run| run.line(counts));
        let Some(file) = self.file.as_mut() else {
            return;
        };
        if let Some(preamble) = preamble
            && let Err(error) = writeln!(file, "{preamble}")
        {
            eprintln!("gui perf-log: cannot write the run preamble: {error}");
        }
        if let Err(error) = writeln!(file, "{}", row.to_csv())
            && !self.reported_error
        {
            self.reported_error = true;
            eprintln!("gui perf-log: write failed, the log is incomplete from here: {error}");
        }
    }
}

/// The mark key: `F4` flags the frame it was pressed on.
///
/// The flag serves a scripted run, which knows in advance that it wants numbers. This serves the
/// case a scripted run can never reproduce — a human noticing something at the seat — by leaving a
/// landmark in the log rather than by dumping it, because with per-frame writes the rows are
/// already on disk.
pub fn mark_perf_frame_on_key(
    keys: Res<bevy::prelude::ButtonInput<bevy::prelude::KeyCode>>,
    log: Option<ResMut<PerfLog>>,
) {
    if !keys.just_pressed(bevy::prelude::KeyCode::F4) {
        return;
    }
    let Some(mut log) = log else {
        eprintln!(
            "gui perf-log: F4 pressed but no --perf-log was given; nothing is being recorded"
        );
        return;
    };
    log.mark_next_frame();
    let (frame, path) = (log.frames_recorded(), log.path().display().to_string());
    eprintln!("gui perf-log: marked frame {frame} in {path}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("frostvein-perf-{}-{name}.csv", std::process::id()))
    }

    /// The row is the instrument's output, so its exact text is pinned against a hand-written
    /// expectation rather than re-derived from the formatter under test.
    #[test]
    fn a_row_carries_the_time_and_the_content_that_explains_it() {
        let row = PerfRow {
            frame: 7,
            t_ms: 116.5,
            frametime_ms: 16.667,
            terrain: 40148,
            trees: 265,
            dwarves: 5,
            dirty_tiles: 0,
            mark: false,
        };
        assert_eq!(row.to_csv(), "7,116.500,16.667,40148,265,5,0,0");
        assert_eq!(
            CSV_HEADER.split(',').count(),
            row.to_csv().split(',').count(),
            "every header column must have a value; a header that outgrows its rows is how a \
             column silently reads as another column's data"
        );
    }

    /// Frametime is a DELTA and the first frame has nothing to subtract from, so it must report
    /// 0 rather than the time since the log opened — which would look like one enormous hitch.
    #[test]
    fn frametime_is_the_gap_between_frames_and_the_first_frame_has_none() {
        let start = Instant::now();
        let path = scratch("gap");
        let mut log = PerfLog::new(path.clone());
        let rows = [
            log.row(start, FrameCounts::default()),
            log.row(start + Duration::from_millis(16), FrameCounts::default()),
            log.row(start + Duration::from_millis(48), FrameCounts::default()),
        ];
        let _ = std::fs::remove_file(&path);

        assert_eq!(
            rows[0].frametime_ms, 0.0,
            "the first frame has no predecessor"
        );
        assert!((rows[1].frametime_ms - 16.0).abs() < 0.001);
        assert!(
            (rows[2].frametime_ms - 32.0).abs() < 0.001,
            "a 32ms gap is a 32ms frame, not the 48ms since the log opened"
        );
        // t_ms is the other axis: it keeps running while frametime resets each frame. It is
        // anchored to the FIRST RECORDED FRAME rather than to the log's open — anchored to the
        // open it folded in however long `File::create` took, which made this assertion a race
        // and failed it exactly that way once.
        assert_eq!(rows[0].t_ms, 0.0, "the first frame is the origin of t_ms");
        assert!((rows[2].t_ms - 48.0).abs() < 0.001);
        assert!(rows[2].t_ms > rows[1].t_ms, "t_ms must be monotonic");
    }

    /// The mark lands on ONE frame — the next one — and not on every frame after it.
    #[test]
    fn a_mark_flags_exactly_the_next_frame() {
        let start = Instant::now();
        let path = scratch("mark");
        let mut log = PerfLog::new(path.clone());
        let before = log.row(start, FrameCounts::default());
        log.mark_next_frame();
        let marked = log.row(start + Duration::from_millis(16), FrameCounts::default());
        let after = log.row(start + Duration::from_millis(32), FrameCounts::default());
        let _ = std::fs::remove_file(&path);

        assert!(!before.mark);
        assert!(
            marked.mark,
            "the frame after the key press carries the mark"
        );
        assert!(!after.mark, "a mark is a landmark, not a mode");
        assert_eq!(log.marks(), 1);
    }

    /// Every recorded frame must reach disk, because the client's exit paths include a panic that
    /// runs no destructors. This is the regression test for the batching that lost 60 of 300.
    #[test]
    fn every_frame_is_on_disk_the_moment_it_is_recorded() {
        let path = scratch("durable");
        let start = Instant::now();
        let mut log = PerfLog::new(path.clone());
        for frame in 0..250u64 {
            log.record(
                start + Duration::from_millis(frame * 16),
                FrameCounts {
                    terrain: 40148,
                    trees: 265,
                    dwarves: 5,
                    dirty_tiles: usize::from(frame == 7),
                },
            );
        }
        // Read WITHOUT dropping or flushing the log: whatever is on disk now is what a panic would
        // have left behind.
        let text = std::fs::read_to_string(&path).expect("the log must exist");
        let lines: Vec<_> = text.lines().collect();

        assert_eq!(lines[0], CSV_HEADER);
        assert_eq!(
            lines.len(),
            251,
            "one header and 250 rows must be durable with no explicit flush"
        );
        assert!(
            lines[8].ends_with(",1,0"),
            "the edit frame's row: {:?}",
            lines[8]
        );
        assert_eq!(log.frames_recorded(), 250);
        let _ = std::fs::remove_file(&path);
    }

    /// A file that exists must always have a schema, so the summariser can refuse it by name
    /// rather than mis-parse it.
    #[test]
    fn the_header_is_written_when_the_log_opens_not_when_the_first_frame_lands() {
        let path = scratch("header");
        let log = PerfLog::new(path.clone());
        let text = std::fs::read_to_string(&path).expect("the log must exist immediately");
        drop(log);
        let _ = std::fs::remove_file(&path);
        assert_eq!(text.trim_end(), CSV_HEADER);
    }

    /// An unwritable path must not take the run down with it: the log is an instrument, not the
    /// product. It says so once and the client keeps running.
    #[test]
    fn an_unwritable_path_is_reported_and_survived() {
        let mut log = PerfLog::new(PathBuf::from("/does/not/exist/run.csv"));
        log.record(Instant::now(), FrameCounts::default());
        assert_eq!(
            log.frames_recorded(),
            1,
            "frames are still counted even when they cannot be written"
        );
    }
}
