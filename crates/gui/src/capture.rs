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
use protocol::{EntityKind, JobState, LightKind};

use crate::{
    ingest::MirrorResource,
    project::{TerrainTile, WorldProjected},
    transform::world_to_render,
};

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
    first_region: Option<BTreeSet<[i32; 3]>>,
    last_region: BTreeSet<[i32; 3]>,
    lit_terrain_tiles: usize,
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
        if self.first_region.is_some() && positions == self.last_positions {
            return;
        }

        let region = observations
            .into_iter()
            .flat_map(|(_, position, region)| {
                self.positions.insert(position);
                region
            })
            .collect::<BTreeSet<_>>();
        self.lit_terrain_tiles += region.len();
        self.first_region.get_or_insert_with(|| region.clone());
        self.last_region = region;
        self.last_positions = positions;
    }

    fn moved(&self) -> bool {
        self.first_region
            .as_ref()
            .is_some_and(|first| *first != self.last_region)
    }

    fn assert_valid(&self) {
        assert!(
            self.lit_terrain_tiles > 0,
            "capture observed no terrain lit by dwarf lanterns"
        );
        assert!(
            self.moved(),
            "capture observed dwarf lantern light but its lit terrain region never moved"
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
    let terrain = terrain
        .iter()
        .map(|(tile, transform)| (tile.0, transform.translation))
        .collect::<Vec<_>>();
    capture
        .lantern
        .observe(projected.iter().filter_map(|(marker, transform, light)| {
            let entity = mirror.0.entities().find(|entity| entity.id == marker.0)?;
            let light = light?;
            (entity.kind == EntityKind::Dwarf && entity.light == Some(LightKind::Lantern)).then(
                || {
                    let lit_region = terrain
                        .iter()
                        .filter(|(_, terrain_translation)| {
                            transform.translation.distance(*terrain_translation) <= light.range
                        })
                        .map(|(position, _)| *position)
                        .collect();
                    (entity.id, entity.pos, lit_region)
                },
            )
        }));
}

/// Captures from the primary window after the real render loop has advanced N frames.
pub fn capture_after_frames(mut commands: Commands, mut capture: ResMut<CaptureState>) {
    if capture.requested {
        return;
    }
    capture.elapsed += 1;
    if capture.elapsed >= capture.frames {
        // The line comes BEFORE the assertion: a run that fails its thresholds is exactly the
        // run whose five numbers are needed to diagnose it, and a panic prints none of them.
        println!(
            "lantern: dwarf positions observed={:?} lit terrain tiles at dwarf positions={} moved={}",
            capture.lantern.positions,
            capture.lantern.lit_terrain_tiles,
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
        capture.lantern.assert_valid();
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
    }
}
