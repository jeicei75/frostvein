use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use bevy::{
    app::AppExit,
    ecs::message::MessageWriter,
    prelude::{Commands, On, PointLight, Query, Res, ResMut, Resource, Transform, Without},
    render::render_resource::TextureFormat,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
};
use client_core::Mirror;
use protocol::{EntityKind, JobState, LightKind, Tile};

use crate::{
    ingest::MirrorResource,
    project::{TerrainTile, WorldProjected},
    slice::SliceLevel,
    transform::world_to_render,
};

#[derive(Debug, Clone, Copy)]
struct DrawStats {
    level: i32,
    terrain_tiles: usize,
    /// Tiles drawn exactly AT the cut, i.e. the floor of the cut.
    cut_face_tiles: usize,
    /// Tiles the mirror says the cut face must contain. Read from the world, not from the draw
    /// set, so it is an independent oracle rather than a restatement of what was drawn.
    expected_cut_face: usize,
}

impl DrawStats {
    fn new(
        level: i32,
        terrain_tiles: usize,
        cut_face_tiles: usize,
        expected_cut_face: usize,
    ) -> Self {
        Self {
            level,
            terrain_tiles,
            cut_face_tiles,
            expected_cut_face,
        }
    }

    /// `terrain_tiles > 0` alone cannot fail for this story's own defect. World-boundary tiles are
    /// always exposed, so a HOLLOW SHELL — the cut with no floor drawn — keeps the global count
    /// comfortably positive and passes identically to a correct cut. Measured on a 9x9x9 block:
    /// 258 tiles correct against 209 hollow, both far above zero. The cut face is the feature, so
    /// the cut face is what the instrument has to count.
    fn assert_valid(self) {
        assert!(
            self.terrain_tiles > 0,
            "capture drew no terrain cubes at requested z {}",
            self.level
        );
        assert_eq!(
            self.cut_face_tiles, self.expected_cut_face,
            "capture drew a hollow cut at z {}: the mirror has {} solid tiles at that level but \
             {} were drawn — the cut face is the feature, and it is missing",
            self.level, self.expected_cut_face, self.cut_face_tiles
        );
    }
}

/// Whether a missing lantern at this cut is a defect rather than an operator choice. Asks the
/// MIRROR whether any dwarf sits at or below the cut: an empty observation means "the slice hides
/// them" AND "entity projection is broken", and keying off the observation alone let every capture
/// below the top — which is every capture this story takes — exit 0 on a total lantern regression.
fn lantern_assertions_apply(mirror: &Mirror, level: i32) -> bool {
    mirror
        .entities()
        .any(|entity| entity.kind == EntityKind::Dwarf && entity.pos[2] <= level)
}

/// The tiles the mirror says the cut face must contain: solid or ramp, exactly at the cut.
fn expected_cut_face(mirror: &Mirror, level: i32) -> usize {
    let dims = mirror.dims();
    let mut count = 0;
    for y in 0..dims.y as i32 {
        for x in 0..dims.x as i32 {
            if matches!(
                mirror.tile([x, y, level]),
                Some(Tile::Solid(_) | Tile::Ramp(_))
            ) {
                count += 1;
            }
        }
    }
    count
}

#[derive(Resource)]
pub struct CaptureState {
    path: PathBuf,
    frames: u32,
    elapsed: u32,
    requested: bool,
    expect_work: bool,
    motion: MotionStats,
    lantern: LanternStats,
}

#[derive(Default, Debug)]
pub struct MotionStats {
    ticks: BTreeSet<u64>,
    positions: BTreeMap<u32, [i32; 3]>,
    pub position_changes: usize,
    pub mid_blend_frames: usize,
    pub max_working: usize,
    pub item_count: usize,
}

impl MotionStats {
    pub fn observe(
        &mut self,
        tick: u64,
        entities: impl Iterator<Item = (u32, [i32; 3], JobState)>,
        item_count: usize,
        mid_blend: bool,
    ) {
        self.ticks.insert(tick);
        let entities = entities.collect::<Vec<_>>();
        for (id, position, _) in &entities {
            if self
                .positions
                .insert(*id, *position)
                .is_some_and(|old| old != *position)
            {
                self.position_changes += 1;
            }
        }
        self.max_working = self.max_working.max(
            entities
                .iter()
                .filter(|(_, _, state)| *state == JobState::Work)
                .count(),
        );
        // A running maximum, like `max_working` above: both answer "did this happen at any point
        // in the run?", and items can be hauled away before the final observed frame.
        self.item_count = self.item_count.max(item_count);
        self.mid_blend_frames += usize::from(mid_blend);
    }

    pub fn assert_valid(&self, expect_work: bool) {
        assert!(
            self.ticks.len() >= 100,
            "capture observed only {} delivered ticks",
            self.ticks.len()
        );
        assert!(
            self.position_changes > 0,
            "capture observed no dwarf position changes"
        );
        assert!(
            self.mid_blend_frames > 0,
            "capture rendered no mid-blend entities"
        );
        if expect_work {
            // NOTE: these global counts are site counts in this scenario: it has the only work
            // and the only items, without teaching the client a dig-site rectangle.
            assert!(self.max_working >= 1, "capture observed no working dwarves");
            assert!(self.item_count >= 1, "capture observed no stone items");
        }
    }
}

#[derive(Default, Debug)]
struct LanternStats {
    positions: BTreeSet<[i32; 3]>,
    last_positions: BTreeMap<u32, [i32; 3]>,
    /// Each dwarf's first NON-EMPTY lit region. An empty region must never become a baseline: it
    /// would differ from every later observation and make `moved()` true for the rest of the run,
    /// silently. The terrain query can come back empty on a frame where the lanterns are already
    /// projected, so this is reachable and would defeat the one assertion this instrument exists
    /// for.
    first_regions: BTreeMap<u32, BTreeSet<[i32; 3]>>,
    /// Dwarves whose OWN lit region has differed from their own first at some point. Per-dwarf so
    /// one dwarf's move cannot be cancelled by another's in an aggregate union, and sticky so a
    /// dwarf that wanders back to where it started still counts as having moved — that shape would
    /// otherwise fail a working capture on the vehicle.
    moved_ids: BTreeSet<u32>,
    last_region: BTreeSet<[i32; 3]>,
    /// Distinct terrain tiles lit at any observation. NOT a running sum of region sizes: that
    /// prints a six-figure number on the vehicle that reads as a tile count, matches no threshold
    /// and is comparable with nothing.
    lit_tiles: BTreeSet<[i32; 3]>,
}

impl LanternStats {
    /// Observe each lantern dwarf's delivered position and the terrain covered by its rendered
    /// point-light range. Only a new delivered position advances the first/last comparison.
    fn observe(&mut self, dwarves: impl Iterator<Item = (u32, [i32; 3], BTreeSet<[i32; 3]>)>) {
        let observations = dwarves.collect::<Vec<_>>();
        let positions = observations
            .iter()
            .map(|(id, position, _)| (*id, *position))
            .collect::<BTreeMap<_, _>>();
        if !self.first_regions.is_empty() && positions == self.last_positions {
            return;
        }

        let mut region = BTreeSet::new();
        for (id, position, lit) in observations {
            self.positions.insert(position);
            region.extend(lit.iter().copied());
            if lit.is_empty() {
                continue;
            }
            match self.first_regions.get(&id) {
                None => {
                    self.first_regions.insert(id, lit);
                }
                Some(first) if *first != lit => {
                    self.moved_ids.insert(id);
                }
                Some(_) => {}
            }
        }
        self.lit_tiles.extend(region.iter().copied());
        self.last_region = region;
        self.last_positions = positions;
    }

    fn needs_observation(&self, positions: &BTreeMap<u32, [i32; 3]>) -> bool {
        self.first_regions.is_empty() || positions != &self.last_positions
    }

    fn moved(&self) -> bool {
        !self.moved_ids.is_empty()
    }

    fn assert_valid(&self) {
        assert!(
            !self.lit_tiles.is_empty(),
            "capture observed no terrain lit by dwarf lanterns"
        );
        assert!(
            !self.last_region.is_empty(),
            "capture's final dwarf lantern observation lit no terrain"
        );
        assert!(
            self.moved(),
            "capture observed dwarf lantern light but no dwarf's lit terrain region ever moved"
        );
    }
}

pub const WARM_RED_OVER_BLUE: u8 = 30;
/// `capture-2026-08-15T1717-boot.png` measured 17,648 warm-lit pixels at the boot framing.
/// The old 100 floor (and its ~64 emitter-face estimate) could not distinguish missing point
/// lights from their emissive source faces; 3,000 leaves framing headroom while requiring pools.
pub const WARM_PIXEL_FLOOR: usize = 3_000;

pub fn warm_lit_pixels(rgba: &[[u8; 4]]) -> usize {
    rgba.iter()
        .filter(|pixel| pixel[0].saturating_sub(pixel[2]) > WARM_RED_OVER_BLUE)
        .count()
}

/// The centre of the valley floor, as fractions of the frame. Deliberately inside the world
/// edge dissolve and below the skyline, so the sample is terrain and not sky or rim.
const GROUND_WINDOW_X: (f32, f32) = (0.25, 0.75);
const GROUND_WINDOW_Y: (f32, f32) = (0.50, 0.90);

/// AC9's value discipline made measurable. Sampled in the window above, the APPROVED ARTIFACT
/// reads a median sRGB luminance of 123; the round-4 capture read 21 — a night scene that is
/// simply black. No headless test can see this, so the instrument carries it. The floor sits
/// between the two so the dark-field failure class cannot pass while the light budget is free
/// to land anywhere near the target.
pub const GROUND_LUMINANCE_FLOOR: u8 = 70;

/// The other end of AC9's discipline, added after the boot3 capture measured 156 against the
/// artifact's 123: a field pushed toward white passes the floor as easily as a correct one.
/// "Night snow stays midtone" needs a ceiling as much as a floor.
pub const GROUND_LUMINANCE_CEILING: u8 = 180;

fn luminance(pixel: [u8; 4]) -> f32 {
    0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32
}

/// Median luminance of the valley floor. Median, not mean, so a handful of blown-out emitter
/// faces cannot carry a black field over the floor.
pub fn median_ground_luminance(pixels: &[[u8; 4]], width: u32, height: u32) -> u8 {
    let column_range =
        (width as f32 * GROUND_WINDOW_X.0) as u32..(width as f32 * GROUND_WINDOW_X.1).ceil() as u32;
    let row_range = (height as f32 * GROUND_WINDOW_Y.0) as u32
        ..(height as f32 * GROUND_WINDOW_Y.1).ceil() as u32;
    let mut samples: Vec<u8> = row_range
        .flat_map(|row| column_range.clone().map(move |column| (row, column)))
        .filter_map(|(row, column)| pixels.get((row * width + column) as usize))
        .map(|pixel| luminance(*pixel).round() as u8)
        .collect();
    if samples.is_empty() {
        return 0;
    }
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn decode_rgba8(bytes: &[u8], format: TextureFormat) -> Vec<[u8; 4]> {
    assert!(
        bytes.len().is_multiple_of(4),
        "capture pixel data must contain whole four-channel pixels"
    );
    match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => bytes
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
            .collect(),
        TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => bytes
            .chunks_exact(4)
            .map(|pixel| [pixel[2], pixel[1], pixel[0], pixel[3]])
            .collect(),
        _ => panic!("capture range check cannot decode {format:?} pixels"),
    }
}

impl CaptureState {
    pub fn new(path: PathBuf, frames: u32, expect_work: bool) -> Self {
        Self {
            path,
            frames,
            elapsed: 0,
            requested: false,
            expect_work,
            motion: MotionStats::default(),
            lantern: LanternStats::default(),
        }
    }

    /// Read path for the headless test that drives `accumulate_motion` through the production
    /// systems. Without it the extraction half of the instrument — the filter, the range read and
    /// the terrain sweep — is only reachable from a run that needs a window.
    pub fn lantern_moved(&self) -> bool {
        self.lantern.moved()
    }

    pub fn lantern_lit_tiles(&self) -> usize {
        self.lantern.lit_tiles.len()
    }

    pub fn lantern_positions(&self) -> &BTreeSet<[i32; 3]> {
        &self.lantern.positions
    }
}

pub fn accumulate_motion(
    mirror: Option<Res<MirrorResource>>,
    capture: Option<ResMut<CaptureState>>,
    projected: Query<(&WorldProjected, &Transform, Option<&PointLight>), Without<TerrainTile>>,
    terrain: Query<(&TerrainTile, &Transform)>,
) {
    let (Some(mirror), Some(mut capture)) = (mirror, capture) else {
        return;
    };
    // A frame counts as mid-blend only if some entity that ACTUALLY MOVED between the two
    // delivered ticks is ACTUALLY DRAWN somewhere other than either endpoint. Reading the clock
    // instead would count a frozen world — `apply_delta` stores a previous entry for every
    // surviving entity, moving or not — and would pass with the blend deleted entirely.
    let mid_blend = projected.iter().any(|(marker, transform, _)| {
        let Some(entity) = mirror.0.entities().find(|entity| entity.id == marker.0) else {
            return false;
        };
        let Some(previous) = mirror.0.previous_entity(marker.0) else {
            return false;
        };
        previous.pos != entity.pos
            && transform.translation != world_to_render(previous.pos)
            && transform.translation != world_to_render(entity.pos)
    });
    capture.motion.observe(
        mirror.0.tick(),
        mirror
            .0
            .entities()
            .map(|entity| (entity.id, entity.pos, entity.state)),
        mirror.0.items().count(),
        mid_blend,
    );
    let lanterns = projected
        .iter()
        .filter_map(|(marker, transform, light)| {
            let entity = mirror.0.entities().find(|entity| entity.id == marker.0)?;
            let light = light?;
            (entity.kind == EntityKind::Dwarf && entity.light == Some(LightKind::Lantern))
                .then_some((entity.id, entity.pos, transform.translation, light.range))
        })
        .collect::<Vec<_>>();
    let positions = lanterns
        .iter()
        .map(|(id, position, _, _)| (*id, *position))
        .collect::<BTreeMap<_, _>>();
    // NOTE: this samples only frames where a DELIVERED position changed, so every observation
    // lands near factor 0 and no mid-blend frame is ever sampled. The lantern line therefore
    // evidences "the pool follows the dwarf between wire positions", NOT "the pool slides rather
    // than snaps" — sampling every frame would run the terrain sweep across the whole draw set each
    // frame and corrupt the very fps reading AC13 asks for. The sliding property is carried by
    // `motion.mid_blend_frames` and by the headless blend test, not by this line.
    if capture.lantern.needs_observation(&positions) {
        let terrain = terrain
            .iter()
            .map(|(tile, transform)| (tile.0, transform.translation))
            .collect::<Vec<_>>();
        capture.lantern.observe(lanterns.into_iter().map(
            |(id, position, light_translation, range)| {
                let lit_region = terrain
                    .iter()
                    .filter(|(_, terrain_translation)| {
                        light_translation.distance(*terrain_translation) <= range
                    })
                    .map(|(position, _)| *position)
                    .collect();
                (id, position, lit_region)
            },
        ));
    }
}

/// Captures from the primary window after the real render loop has advanced N frames.
pub fn capture_after_frames(
    mut commands: Commands,
    mut capture: ResMut<CaptureState>,
    slice: Res<SliceLevel>,
    mirror: Res<MirrorResource>,
    terrain: Query<&TerrainTile>,
) {
    if capture.requested {
        return;
    }
    capture.elapsed += 1;
    if capture.elapsed >= capture.frames {
        // The line comes BEFORE the assertion: a run that fails its thresholds is exactly the
        // run whose five numbers are needed to diagnose it, and a panic prints none of them.
        let draw = DrawStats::new(
            slice.level(),
            terrain.iter().count(),
            terrain
                .iter()
                .filter(|tile| tile.0[2] == slice.level())
                .count(),
            expected_cut_face(&mirror.0, slice.level()),
        );
        // Print the actual count before every assertion. A successful process with a blank cut is
        // not a capture result; this remains truthful when a requested level changes the draw set.
        println!(
            "slice: z {} projected {} terrain cubes ({} of {} cut-face tiles at z {})",
            draw.level, draw.terrain_tiles, draw.cut_face_tiles, draw.expected_cut_face, draw.level
        );
        println!(
            "lantern: dwarf positions observed={:?} lit terrain tiles at dwarf positions={} moved={}",
            capture.lantern.positions,
            capture.lantern.lit_tiles.len(),
            capture.lantern.moved(),
        );
        println!(
            "motion: ticks observed={} dwarf position changes={} mid-blend frames={} max working dwarves={} item count={}",
            capture.motion.ticks.len(),
            capture.motion.position_changes,
            capture.motion.mid_blend_frames,
            capture.motion.max_working,
            capture.motion.item_count
        );
        // EVERY number is printed above, before ANY assertion below: a run that fails its
        // thresholds is exactly the run whose numbers are needed to diagnose it, and a panic
        // prints none of them. The draw check was briefly asserted up at its own print, which
        // silenced the lantern, motion and range numbers on precisely those failures.
        draw.assert_valid();
        // A cut below the dwarves hides every lantern, so the lantern assertions would report a
        // defect when the operator merely asked for a lower slice. Ask the MIRROR whether any
        // dwarf is at or below the cut rather than trusting the observation: `observed()` is
        // empty both when the slice legitimately hides them AND when entity projection is broken
        // entirely, so keying off it alone made every non-top capture — which is every capture
        // this story takes — exit 0 on a total lantern regression.
        if lantern_assertions_apply(&mirror.0, slice.level()) {
            capture.lantern.assert_valid();
        } else {
            println!(
                "lantern: no dwarf sits at or below z {} — the requested slice hides them all, \
                 lantern assertions skipped",
                slice.level()
            );
        }
        capture.motion.assert_valid(capture.expect_work);
        capture.requested = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(capture.path.clone()))
            .observe(validate_capture_ranges)
            .observe(exit_after_capture);
    }
}

fn exit_after_capture(_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

fn validate_capture_ranges(event: On<ScreenshotCaptured>) {
    let bytes = event
        .image
        .data
        .as_deref()
        .expect("capture screenshot must include pixel data");
    let pixels = decode_rgba8(bytes, event.image.texture_descriptor.format);
    assert!(
        pixels.iter().any(|pixel| pixel[..3] != [0, 0, 0]),
        "capture is black"
    );
    assert!(
        pixels.windows(2).any(|pair| pair[0] != pair[1]),
        "capture is uniform"
    );
    let warm = warm_lit_pixels(&pixels);
    let size = event.image.texture_descriptor.size;
    let ground = median_ground_luminance(&pixels, size.width, size.height);
    println!("capture range check: warm-lit pixels={warm} ground-median-luminance={ground}");
    // NOTE: confirm this source-face-derived floor on the native-Windows vehicle run.
    assert!(
        warm >= WARM_PIXEL_FLOOR,
        "capture contains fewer than {WARM_PIXEL_FLOOR} warm-lit pixels"
    );
    assert!(
        ground >= GROUND_LUMINANCE_FLOOR,
        "the valley floor reads {ground}, below the {GROUND_LUMINANCE_FLOOR} value floor — \
         the frame is a black field, not a lit night"
    );
    assert!(
        ground <= GROUND_LUMINANCE_CEILING,
        "the valley floor reads {ground}, above the {GROUND_LUMINANCE_CEILING} value ceiling — \
         night snow must stay midtone; only emissive approaches white"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-written oracle: a 4x4 frame whose centre window is exactly the four pixels at
    /// rows 2..4, columns 1..3. Values are chosen so the MEAN (112) differs from the MEDIAN
    /// (100) — the first attempt used 10/90/100/200, where both are 100 and a mean-based
    /// implementation passed the test unchanged. The sabotage caught it.
    #[test]
    fn the_ground_median_reads_the_valley_floor_and_ignores_the_sky() {
        let sky = [0u8, 0, 0, 255];
        let grey = |v: u8| [v, v, v, 255];
        // Rec.709 of a neutral grey is the grey itself.
        let mut frame = vec![sky; 16];
        frame[2 * 4 + 1] = grey(10);
        frame[2 * 4 + 2] = grey(90);
        frame[3 * 4 + 1] = grey(100);
        frame[3 * 4 + 2] = grey(250);

        assert_eq!(median_ground_luminance(&frame, 4, 4), 100);
        // The sky rows are bright here and must NOT be able to lift the reading.
        let mut bright_sky = frame.clone();
        for pixel in bright_sky.iter_mut().take(8) {
            *pixel = grey(255);
        }
        assert_eq!(median_ground_luminance(&bright_sky, 4, 4), 100);
    }

    #[test]
    fn a_black_field_fails_the_value_floor_that_a_lit_one_passes() {
        let black = vec![[12u8, 14, 20, 255]; 64];
        assert!(median_ground_luminance(&black, 8, 8) < GROUND_LUMINANCE_FLOOR);
        let lit = vec![[95u8, 112, 129, 255]; 64];
        assert!(median_ground_luminance(&lit, 8, 8) >= GROUND_LUMINANCE_FLOOR);
    }

    #[test]
    fn a_blown_out_field_fails_the_value_ceiling_that_a_midtone_one_passes() {
        let blown = vec![[205u8, 215, 230, 255]; 64];
        assert!(median_ground_luminance(&blown, 8, 8) > GROUND_LUMINANCE_CEILING);
        let midtone = vec![[95u8, 112, 129, 255]; 64];
        assert!(median_ground_luminance(&midtone, 8, 8) <= GROUND_LUMINANCE_CEILING);
    }

    fn mirror_with_dwarf_at(z: i32) -> Mirror {
        use protocol::{Dims, Entity, Snapshot, Speed};
        Mirror::from_snapshot(Snapshot {
            msg_type: protocol::MessageType::Snapshot,
            dims: Dims { x: 1, y: 1, z: 3 },
            tiles: vec![Tile::Solid(protocol::Material::Stone); 3],
            entities: vec![Entity {
                id: 1,
                kind: EntityKind::Dwarf,
                pos: [0, 0, z],
                state: JobState::Idle,
                light: Some(LightKind::Lantern),
            }],
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .expect("a hand-built snapshot must load")
    }

    #[test]
    fn lantern_assertions_follow_the_mirror_not_the_observation() {
        // The defect: the gate keyed off "did we observe a lantern?", which is empty BOTH when the
        // cut legitimately hides the dwarves AND when entity projection is broken outright — so a
        // total lantern regression exited 0 at every level below the top.
        let mirror = mirror_with_dwarf_at(2);
        assert!(
            !lantern_assertions_apply(&mirror, 1),
            "a cut below every dwarf genuinely hides the lanterns, so skipping is correct"
        );
        assert!(
            lantern_assertions_apply(&mirror, 2),
            "a dwarf sitting at the cut must still be holding a visible lantern"
        );
        assert!(
            lantern_assertions_apply(&mirror_with_dwarf_at(0), 1),
            "a dwarf BELOW the cut is visible, so a missing lantern there is a real defect"
        );
    }

    #[test]
    fn draw_count_instrument_rejects_an_empty_level_and_accepts_terrain() {
        let empty = DrawStats::new(4, 0, 0, 0);
        assert!(
            std::panic::catch_unwind(|| empty.assert_valid()).is_err(),
            "a capture must not claim success when its requested slice drew nothing"
        );
        DrawStats::new(4, 12, 5, 5).assert_valid();
    }

    #[test]
    fn draw_count_instrument_rejects_a_hollow_cut_that_kept_its_total_up() {
        // The defect the old instrument could not see: boundary tiles keep the global count high
        // while the cut face itself is missing. These are the measured 9x9x9 figures — 258 tiles
        // correct against 209 hollow, 81 cut-face tiles against 32.
        let hollow = DrawStats::new(4, 209, 32, 81);
        assert!(
            std::panic::catch_unwind(|| hollow.assert_valid()).is_err(),
            "a cut drawn with no floor is not a capture result, however many tiles it drew"
        );
        DrawStats::new(4, 258, 81, 81).assert_valid();
    }

    #[test]
    fn a_level_with_no_solid_tiles_is_a_legitimate_empty_cut_face() {
        // Slicing into open sky above the mountain: the mirror says nothing is there, so drawing
        // nothing at the cut is correct and must not be read as a hollow shell.
        DrawStats::new(30, 53_365, 0, 0).assert_valid();
    }

    #[test]
    fn bgra_capture_bytes_decode_before_warm_pixel_detection() {
        let pixels = decode_rgba8(&[10, 120, 240, 255], TextureFormat::Bgra8Unorm);

        assert_eq!(pixels, vec![[240, 120, 10, 255]]);
        assert_eq!(warm_lit_pixels(&pixels), 1);
    }

    #[test]
    fn motion_instrument_rejects_stillness_and_accepts_the_required_observation() {
        let mut still = MotionStats::default();
        for tick in 0..100 {
            still.observe(tick, [(1, [2, 3, 4], JobState::Idle)].into_iter(), 0, false);
        }
        assert!(std::panic::catch_unwind(|| still.assert_valid(false)).is_err());

        let mut moving = MotionStats::default();
        for tick in 0..100 {
            moving.observe(
                tick,
                [(1, [tick as i32, 3, 4], JobState::Work)].into_iter(),
                1,
                tick > 0,
            );
        }
        moving.assert_valid(true);

        // Items hauled away before the last observed frame still happened. `max_working` and
        // `item_count` answer the same "at any point in the run?" question and must agree.
        let mut items_then_gone = MotionStats::default();
        for tick in 0..100 {
            items_then_gone.observe(
                tick,
                [(1, [tick as i32, 3, 4], JobState::Work)].into_iter(),
                usize::from(tick < 50),
                tick > 0,
            );
        }
        assert_eq!(
            items_then_gone.item_count, 1,
            "the item count is a running maximum, not the last frame's reading"
        );
        items_then_gone.assert_valid(true);
    }

    #[test]
    fn lantern_instrument_requires_a_lit_region_to_move() {
        // Hand-built mirror observations: the dwarf wire position and the terrain tiles in its
        // rendered pool are both literals, so this does not prove itself through projection code.
        let mut still = LanternStats::default();
        let region = BTreeSet::from([[1, 2, 3], [2, 2, 3]]);
        still.observe([(7, [2, 3, 4], region.clone())].into_iter());
        still.observe([(7, [2, 3, 4], region)].into_iter());
        assert!(std::panic::catch_unwind(|| still.assert_valid()).is_err());

        let mut moving = LanternStats::default();
        moving.observe([(7, [2, 3, 4], BTreeSet::from([[1, 2, 3], [2, 2, 3]]))].into_iter());
        moving.observe([(7, [3, 3, 4], BTreeSet::from([[2, 2, 3], [3, 2, 3]]))].into_iter());
        moving.assert_valid();

        let mut vanished = LanternStats::default();
        vanished.observe([(7, [2, 3, 4], BTreeSet::from([[1, 2, 3], [2, 2, 3]]))].into_iter());
        vanished.observe(std::iter::empty());
        assert!(std::panic::catch_unwind(|| vanished.assert_valid()).is_err());
    }

    /// A first observation whose region is empty must not become that dwarf's baseline. It used to
    /// be latched, after which every later region differed from it and `moved()` stayed true for
    /// the rest of the run — so a lit region that never actually changed passed the one assertion
    /// this instrument exists for. Reachable whenever the terrain query is not yet populated on a
    /// frame the lanterns already are.
    ///
    /// The dwarf must keep CHANGING POSITION here while its region stays constant. An earlier
    /// draft held it still, which made `needs_observation`'s early return swallow observations two
    /// and three, and the sabotage table caught that the test was passing for the wrong reason.
    #[test]
    fn an_empty_first_observation_cannot_stand_in_for_movement() {
        let mut stalled = LanternStats::default();
        let region = BTreeSet::from([[1, 2, 3], [2, 2, 3]]);
        stalled.observe([(7, [2, 3, 4], BTreeSet::new())].into_iter());
        stalled.observe([(7, [3, 3, 4], region.clone())].into_iter());
        stalled.observe([(7, [4, 3, 4], region)].into_iter());
        assert!(
            std::panic::catch_unwind(|| stalled.assert_valid()).is_err(),
            "an unchanging lit region must fail even when the first observation lit nothing"
        );
    }

    /// Lanterns that go dark by the final observation must fail the capture. Once `moved()` became
    /// per-dwarf and sticky it started catching the vanished case on its own, which left the
    /// final-region assert pinned by nothing — the sabotage table caught that too.
    #[test]
    fn a_final_observation_that_lit_nothing_fails_even_after_a_dwarf_moved() {
        let mut went_dark = LanternStats::default();
        went_dark.observe([(7, [2, 3, 4], BTreeSet::from([[1, 2, 3]]))].into_iter());
        went_dark.observe([(7, [3, 3, 4], BTreeSet::from([[2, 2, 3]]))].into_iter());
        went_dark.observe([(7, [4, 3, 4], BTreeSet::new())].into_iter());
        assert!(
            std::panic::catch_unwind(|| went_dark.assert_valid()).is_err(),
            "a run whose lanterns lit nothing at the end must not report success"
        );
    }

    /// `moved()` compared only the first and last observation, so a dwarf that wandered away and
    /// came back failed a working capture. It is now per-dwarf and sticky.
    #[test]
    fn a_dwarf_that_returns_to_where_it_started_still_counts_as_having_moved() {
        let mut wandered = LanternStats::default();
        let home = BTreeSet::from([[1, 2, 3], [2, 2, 3]]);
        wandered.observe([(7, [2, 3, 4], home.clone())].into_iter());
        wandered.observe([(7, [3, 3, 4], BTreeSet::from([[2, 2, 3], [3, 2, 3]]))].into_iter());
        wandered.observe([(7, [2, 3, 4], home)].into_iter());
        wandered.assert_valid();
    }

    /// The lit regions used to be unioned across every dwarf before the comparison, so one dwarf's
    /// move could be cancelled by another's and read as a dead instrument.
    #[test]
    fn one_dwarf_moving_is_not_cancelled_by_another_dwarf_taking_its_tiles() {
        let mut offset = LanternStats::default();
        offset.observe(
            [
                (7, [2, 3, 4], BTreeSet::from([[1, 2, 3]])),
                (8, [9, 3, 4], BTreeSet::from([[2, 2, 3]])),
            ]
            .into_iter(),
        );
        // The aggregate union is unchanged between the two observations; only the per-dwarf
        // regions reveal that both dwarves moved.
        offset.observe(
            [
                (7, [3, 3, 4], BTreeSet::from([[2, 2, 3]])),
                (8, [8, 3, 4], BTreeSet::from([[1, 2, 3]])),
            ]
            .into_iter(),
        );
        offset.assert_valid();
    }

    /// The printed figure was `+= region.len()` per observation — a running sum that reads as a
    /// tile count. Ten observations of the same five-tile pool must report five, not fifty.
    #[test]
    fn the_lit_tile_figure_counts_distinct_tiles_rather_than_summing_observations() {
        let mut repeated = LanternStats::default();
        for step in 0..10 {
            repeated
                .observe([(7, [step, 3, 4], BTreeSet::from([[1, 2, 3], [2, 2, 3]]))].into_iter());
        }
        assert_eq!(
            repeated.lit_tiles.len(),
            2,
            "the instrument must report distinct lit tiles, not a running sum"
        );
    }
}
