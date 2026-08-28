use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use bevy::{
    app::AppExit,
    ecs::message::MessageWriter,
    prelude::{
        Camera3d, Commands, On, PointLight, Query, Res, ResMut, Resource, Transform, Vec2, Window,
        With, Without,
    },
    render::render_resource::TextureFormat,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
};
use client_core::Mirror;
use protocol::{EntityKind, JobState, LightKind, Tile};

use crate::{
    camera::{BOOT_VERTICAL_FOV, CameraRig},
    designate::DesignateMode,
    ingest::MirrorResource,
    ingest::{ScriptedCursor, ScriptedDrag},
    pick::PickedTile,
    project::{ProjectedDesignation, ProjectedZone, TerrainTile, WorldProjected},
    slice::SliceLevel,
    transform::world_to_render,
};
use bevy::window::PrimaryWindow;

#[derive(Debug, Clone, Copy)]
pub struct DrawStats {
    level: i32,
    terrain_tiles: usize,
    /// Tiles drawn exactly AT the cut, i.e. the floor of the cut.
    cut_face_tiles: usize,
    /// Tiles the mirror says the cut face must contain. Read from the world, not from the draw
    /// set, so it is an independent oracle rather than a restatement of what was drawn.
    expected_cut_face: usize,
    designations: usize,
    zones: usize,
    /// What the MIRROR says must be projected at or below the cut. Read from the world, not from
    /// the draw set, on the same principle as `expected_cut_face` above.
    expected_designations: usize,
    expected_zones: usize,
}

impl DrawStats {
    // Six counts and a level, each a distinct measurement the instrument prints by name. Grouping
    // them into a struct solely to satisfy this lint would put a second shape between the queries
    // and the printed line, which is the shape this type already is. Same call as `reconcile`.
    #[allow(clippy::too_many_arguments)]
    fn new(
        level: i32,
        terrain_tiles: usize,
        cut_face_tiles: usize,
        expected_cut_face: usize,
        designations: usize,
        zones: usize,
        expected_designations: usize,
        expected_zones: usize,
    ) -> Self {
        Self {
            level,
            terrain_tiles,
            cut_face_tiles,
            expected_cut_face,
            designations,
            zones,
            expected_designations,
            expected_zones,
        }
    }

    /// `terrain_tiles > 0` alone cannot fail for this story's own defect. World-boundary tiles are
    /// always exposed, so a HOLLOW SHELL — the cut with no floor drawn — keeps the global count
    /// comfortably positive and passes identically to a correct cut. Measured on a 9x9x9 block:
    /// 258 tiles correct against 209 hollow, both far above zero. The cut face is the feature, so
    /// the cut face is what the instrument has to count.
    pub fn designations(&self) -> usize {
        self.designations
    }

    pub fn zones(&self) -> usize {
        self.zones
    }

    pub fn assert_valid(self, expect_work: bool) {
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
        // RULED 2026-08-21 (Wolf): marks get an INDEPENDENT ORACLE, on the `expected_cut_face`
        // precedent, rather than the bare `> 0` this shipped with. Two reasons. It is strictly
        // stronger — `> 0` cannot see a projection that drops half its marks, and this can. And
        // `> 0` could not tell "this view legitimately has no marks" from "mark rendering broke",
        // so it turned every no-mark capture in the project into a panic, 7.1's own shipped recipe
        // included.
        assert_eq!(
            self.designations, self.expected_designations,
            "capture projected {} designations at or below z {} but the mirror holds {}",
            self.designations, self.level, self.expected_designations
        );
        assert_eq!(
            self.zones, self.expected_zones,
            "capture projected {} zones at or below z {} but the mirror holds {}",
            self.zones, self.level, self.expected_zones
        );
        if expect_work {
            // AC13's "exit 0 is not a result", kept where it belongs. The oracle above proves the
            // PROJECTION is faithful; it passes a world that has no marks left to project. That is
            // the exact false pass this story was created around: the dwarves CONSUME designations,
            // an 8-tile site is dug out in ~52 ticks against a ~110-tick capture window, and the
            // naive recipe therefore photographs an empty site and exits 0. A capture that says it
            // is of a working site must find work there.
            assert!(
                self.expected_designations > 0,
                "capture of a working site found no designations at or below z {} in the mirror —                  the marks were consumed before the trigger frame, so this frame shows none of                  what it was taken to show",
                self.level
            );
            assert!(
                self.expected_zones > 0,
                "capture of a working site found no zones at or below z {} in the mirror — a                  stockpile rect on non-standable ground is silently dropped, and a zone tile sits                  one level ABOVE the rock it rests on",
                self.level
            );
        }
    }
}

impl DrawStats {
    /// AC15's non-zero range check, for the ONE case that could not reach it. The check lived
    /// only behind `--expect-work`, which the story's own `--drag` recipe does not pass and which
    /// a dig-only drag CANNOT pass — it also demands `expected_zones > 0`, and a fresh world has
    /// no zones. So a `--drag` that designated nothing evaluated `0 == 0`, printed a pass, saved
    /// the PNG and exited 0: verbatim the 7.2 empty-site false pass this AC was written to kill.
    /// A drag asserts against the count for the mode it actually dragged, so it needs neither
    /// flag nor a zone it was never going to create.
    fn assert_drag_produced_work(&self, mode: DesignateMode) {
        match mode {
            DesignateMode::Dig | DesignateMode::Channel => assert!(
                self.expected_designations > 0,
                "scripted --drag designated nothing: the mirror holds no designations at or \
                 below z {}, so this capture shows none of what it was taken to show",
                self.level
            ),
            DesignateMode::Stockpile => assert!(
                self.expected_zones > 0,
                "scripted --drag placed no stockpile: the mirror holds no zones at or below z \
                 {}. A stockpile rect on non-standable ground is silently dropped by the sim",
                self.level
            ),
            // Clear REMOVES; a correct clear legitimately ends with nothing to count.
            DesignateMode::Clear | DesignateMode::None => {}
        }
    }
}

/// The capture's projection-derived draw counts. Kept as one production system so the headless
/// instrument test drives the exact queries the capture uses, without requiring a render surface.
pub fn draw_stats(
    slice: Res<SliceLevel>,
    mirror: Res<MirrorResource>,
    terrain: Query<&TerrainTile>,
    designations: Query<&ProjectedDesignation>,
    zones: Query<&ProjectedZone>,
) -> DrawStats {
    collect_draw_stats(slice.level(), &mirror.0, &terrain, &designations, &zones)
}

fn collect_draw_stats(
    level: i32,
    mirror: &Mirror,
    terrain: &Query<&TerrainTile>,
    designations: &Query<&ProjectedDesignation>,
    zones: &Query<&ProjectedZone>,
) -> DrawStats {
    DrawStats::new(
        level,
        terrain.iter().count(),
        terrain.iter().filter(|tile| tile.0[2] == level).count(),
        expected_cut_face(mirror, level),
        designations.iter().count(),
        zones.iter().count(),
        mirror
            .designations()
            .iter()
            .filter(|designation| designation.pos[2] <= level)
            .count(),
        mirror
            .zones()
            .iter()
            .filter(|zone| zone.pos[2] <= level)
            .count(),
    )
}

/// Whether a missing lantern at this cut is a defect rather than an operator choice. Asks the
/// MIRROR whether any dwarf sits at or below the cut: an empty observation means "the slice hides
/// them" AND "entity projection is broken", and keying off the observation alone let every capture
/// below the top — which is every capture this story takes — exit 0 on a total lantern regression.
/// Whether this world can move at all. A mirror with no dwarves cannot report a position change
/// or a mid-blend frame, so the motion instrument has nothing to say about it.
fn motion_assertions_apply(mirror: &Mirror) -> bool {
    mirror
        .entities()
        .any(|entity| entity.kind == EntityKind::Dwarf)
}

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
    at_tick: Option<(u64, u64)>,
    requested: bool,
    failed: bool,
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
        self.assert_tick_floor(100);
        self.assert_motion(expect_work);
    }

    /// The delivered-tick floor, SEPARATED so an `--at-tick N` capture can scale it to the ticks
    /// it actually asked for. Previously the whole motion instrument was skipped for `--at-tick`,
    /// which silently dropped the two checks below that have nothing to do with tick count.
    pub fn assert_tick_floor(&self, min_ticks: usize) {
        assert!(
            self.ticks.len() >= min_ticks,
            "capture observed only {} delivered ticks, expected at least {min_ticks}",
            self.ticks.len()
        );
    }

    pub fn assert_motion(&self, expect_work: bool) {
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

/// `boot7.png`, the 5.4 Bevy frame Wolf approved, measures a 0.6651% largest region at this
/// luminance. The star shell (192.9 luma) stays below it, so the measure follows the pool.
pub const BLOWN_POOL_LUMINANCE_THRESHOLD: u8 = 200;

/// `boot7.png`, the 5.4 Bevy frame Wolf approved, measures a 0.6651% largest near-white pool.
pub const BLOWN_POOL_FRACTION_CEILING: f32 = 0.006_651_476;

fn luminance(pixel: [u8; 4]) -> f32 {
    0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32
}

/// Fraction of the frame occupied by its largest four-connected near-white region.
pub fn largest_blown_pool_fraction(
    pixels: &[[u8; 4]],
    width: u32,
    height: u32,
    threshold: u8,
) -> f32 {
    assert_eq!(
        pixels.len(),
        (width as usize) * (height as usize),
        "capture dimensions must describe every pixel"
    );
    if pixels.is_empty() {
        return 0.0;
    }

    let mut visited = vec![false; pixels.len()];
    let mut largest = 0;
    for start in 0..pixels.len() {
        if visited[start] || luminance(pixels[start]) < threshold as f32 {
            continue;
        }

        let mut region = 0;
        let mut pending = vec![start];
        visited[start] = true;
        while let Some(index) = pending.pop() {
            region += 1;
            let row = index / width as usize;
            let column = index % width as usize;
            let mut visit = |neighbour: usize| {
                if !visited[neighbour] && luminance(pixels[neighbour]) >= threshold as f32 {
                    visited[neighbour] = true;
                    pending.push(neighbour);
                }
            };
            if column > 0 {
                visit(index - 1);
            }
            if column + 1 < width as usize {
                visit(index + 1);
            }
            if row > 0 {
                visit(index - width as usize);
            }
            if row + 1 < height as usize {
                visit(index + width as usize);
            }
        }
        largest = largest.max(region);
    }

    largest as f32 / pixels.len() as f32
}

/// The 99th percentile of frame luminance, using the nearest sample in sorted pixel order.
pub fn p99_luminance(pixels: &[[u8; 4]]) -> f32 {
    if pixels.is_empty() {
        return 0.0;
    }
    let mut values = pixels.iter().copied().map(luminance).collect::<Vec<_>>();
    values.sort_by(f32::total_cmp);
    // NOTE: the index is computed in f64. At f32, 225,210 of the first 3M possible pixel
    // counts round to the neighbouring sample; no resolution this repo produces is among them
    // today, but a printed instrument that is silently off by one sample is exactly the class
    // this project keeps getting bitten by.
    values[((values.len() - 1) as f64 * 0.99).round() as usize]
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
            at_tick: None,
            requested: false,
            failed: false,
            expect_work,
            motion: MotionStats::default(),
            lantern: LanternStats::default(),
        }
    }

    pub fn at_tick(
        path: PathBuf,
        frames: u32,
        start_tick: u64,
        ticks_after_start: u64,
        expect_work: bool,
    ) -> Self {
        let mut capture = Self::new(path, frames, expect_work);
        capture.at_tick = Some((start_tick, ticks_after_start));
        capture
    }

    pub fn requested(&self) -> bool {
        self.requested
    }

    pub fn failed(&self) -> bool {
        self.failed
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
// The capture instrument is one production system so its frame observations retain a single
// ordering edge. Grouping unrelated ECS queries merely to satisfy this lint would hide that.
#[allow(clippy::too_many_arguments)]
pub fn capture_after_frames(
    mut commands: Commands,
    mut capture: ResMut<CaptureState>,
    slice: Res<SliceLevel>,
    mirror: Res<MirrorResource>,
    terrain: Query<&TerrainTile>,
    designations: Query<&ProjectedDesignation>,
    zones: Query<&ProjectedZone>,
    picked: Option<Res<PickedTile>>,
    cursor: Option<Res<ScriptedCursor>>,
    drag: Option<Res<ScriptedDrag>>,
    cameras: Query<&CameraRig, With<Camera3d>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.requested || capture.failed {
        return;
    }
    capture.elapsed += 1;
    let capture_due = match capture.at_tick {
        Some((start_tick, ticks_after_start)) => {
            let target_tick = start_tick.saturating_add(ticks_after_start);
            if mirror.0.tick() >= target_tick {
                true
            } else if capture.elapsed >= capture.frames {
                eprintln!(
                    "capture --at-tick {ticks_after_start} did not reach tick {target_tick} \
                     within {} frames; reached tick {}",
                    capture.frames,
                    mirror.0.tick()
                );
                capture.failed = true;
                exit.write(AppExit::error());
                false
            } else {
                false
            }
        }
        None => capture.elapsed >= capture.frames,
    };
    if capture_due {
        // The line comes BEFORE the assertion: a run that fails its thresholds is exactly the
        // run whose five numbers are needed to diagnose it, and a panic prints none of them.
        let draw = collect_draw_stats(slice.level(), &mirror.0, &terrain, &designations, &zones);
        if let Some(cursor) = cursor {
            let expected = cameras.single().ok().and_then(|rig| {
                windows
                    .single()
                    .ok()
                    .and_then(|window| expected_pick(cursor.0, *rig, window, &terrain))
            });
            let picked = picked.and_then(|picked| picked.tile());
            println!("{}", pick_capture_line(cursor.0, picked, expected));
            assert_eq!(
                picked, expected,
                "capture cursor ({}, {}) picked {:?} but independent projection expected {:?}",
                cursor.0.x, cursor.0.y, picked, expected
            );
            // `None == None` passed and exited 0, which AC10 does not permit: a legitimate
            // cursor over sky and a FAILURE to resolve the camera or the primary window reach
            // that branch by independent routes, and the second collapses the oracle and the
            // live pick to `None` together. A scripted cursor is aimed at terrain by whoever
            // scripted it, so picking nothing is the instrument reporting a defect.
            assert!(
                picked.is_some(),
                "capture cursor ({}, {}) picked no tile — a scripted cursor must be aimed at \
                 terrain, and a camera or primary window that failed to resolve reaches this \
                 same branch",
                cursor.0.x,
                cursor.0.y
            );
        }
        // Print the actual count before every assertion. A successful process with a blank cut is
        // not a capture result; this remains truthful when a requested level changes the draw set.
        println!(
            "slice: z {} projected {} terrain cubes ({} of {} cut-face tiles at z {})",
            draw.level, draw.terrain_tiles, draw.cut_face_tiles, draw.expected_cut_face, draw.level
        );
        println!(
            "marks: z {} designations={} of {} zones={} of {}",
            draw.level,
            draw.designations,
            draw.expected_designations,
            draw.zones,
            draw.expected_zones
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
        if let Some(drag) = drag.as_deref() {
            // A drag still mid-stage never released, so nothing it was meant to create exists.
            assert!(
                drag.completed(),
                "scripted --drag never completed: it is still mid-stage at the capture frame, so \
                 no designation was ever issued and this PNG shows the world untouched"
            );
            draw.assert_drag_produced_work(drag.mode());
        }
        draw.assert_valid(capture.expect_work);
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
        match capture.at_tick {
            // An `--at-tick N` run cannot meet the 100-tick floor, but the movement and mid-blend
            // checks are live-client health, unrelated to tick count, and were being dropped with
            // it on precisely the new path the vehicle recipe uses. Scale the floor to the ticks
            // actually requested and keep the rest.
            //
            // Applicability is asked of the MIRROR, exactly as `lantern_assertions_apply` does:
            // a world with no dwarves in it cannot produce a position change, and demanding one
            // would report a defect when the operator merely captured an empty scene. Keying off
            // the OBSERVATION instead would be the trap that rule already exists to avoid — it is
            // empty both when there is nothing to see and when the instrument is broken.
            Some((_, ticks_after_start)) => {
                if motion_assertions_apply(&mirror.0) {
                    capture.motion.assert_tick_floor(ticks_after_start as usize);
                    capture.motion.assert_motion(capture.expect_work);
                } else {
                    println!(
                        "motion: the mirror holds no dwarves — motion assertions skipped for \
                         this --at-tick capture"
                    );
                }
            }
            None => capture.motion.assert_valid(capture.expect_work),
        }
        capture.requested = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_then_validate(capture.path.clone(), *slice))
            .observe(exit_after_capture);
    }
}

fn expected_pick(
    cursor: Vec2,
    rig: CameraRig,
    window: &Window,
    terrain: &Query<&TerrainTile>,
) -> Option<[i32; 3]> {
    let viewport = window.resolution.size();
    let mut candidates = terrain
        .iter()
        .filter_map(|tile| {
            let (normalized, depth) = rig.project_world_point_with_depth(tile.0)?;
            let screen = normalized * viewport;
            let distance_squared = screen.distance_squared(cursor);
            let half_extent = tile_half_extent_px(depth, viewport.y);
            (distance_squared <= half_extent * half_extent).then_some((distance_squared, tile.0))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|(left, _), (right, _)| left.total_cmp(right));
    if candidates.len() > 1 {
        // A screen-space oracle is depth-blind BY CONSTRUCTION: where several tiles project
        // inside one tile's apparent footprint, `min_by` on screen distance can prefer the one
        // further from the camera while the pick correctly returns the nearer. Scaling the
        // window shrinks that residual; it does not remove it. It is printed rather than
        // silently resolved so a disagreement is read as the instrument's limit, not the pick's.
        println!("{}", pick_ambiguity_line(cursor, &candidates));
    }
    candidates.first().map(|(_, tile)| *tile)
}

/// Half a tile's apparent size in pixels at `depth`, from the same vertical FOV the camera
/// renders with: `0.5 * height / (2 * depth * tan(fov/2))`, i.e. `651.9 / depth` at 1080p.
///
/// The fixed 32 px this replaces was honest only in a band around depth 20-60. At the near
/// clamp (4.0) it was 0.098 world units — a tenth of a tile's half-width — so a cursor anywhere
/// off dead-centre made the oracle answer `None` against a correct `Some` and the assertion
/// below fired BEFORE `Screenshot::primary_window()`, producing a false failure with no PNG to
/// adjudicate it. At the far clamp (500.0) it was 12.3 units, admitting roughly 24 tiles.
fn tile_half_extent_px(depth: f32, viewport_height: f32) -> f32 {
    0.5 * viewport_height / (2.0 * depth * (BOOT_VERTICAL_FOV * 0.5).tan())
}

/// Names every tile inside the oracle's window when more than one is, nearest screen-distance
/// first, so the residual depth-blindness is visible in the log rather than silent.
fn pick_ambiguity_line(cursor: Vec2, candidates: &[(f32, [i32; 3])]) -> String {
    let tiles = candidates
        .iter()
        .map(|(_, tile)| tile_text(*tile))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "pick: WARNING cursor=({},{}) {} tiles inside the oracle's window, screen-nearest first: {tiles} — \
the oracle is depth-blind and asserts against the first",
        cursor.x,
        cursor.y,
        candidates.len()
    )
}

/// Formats the capture's cursor observation before its equality assertion can abort the process.
pub fn pick_capture_line(
    cursor: Vec2,
    picked: Option<[i32; 3]>,
    expected: Option<[i32; 3]>,
) -> String {
    match (picked, expected) {
        (Some(picked), Some(expected)) => format!(
            "pick: cursor=({},{}) picked={} expected={}",
            cursor.x,
            cursor.y,
            tile_text(picked),
            tile_text(expected)
        ),
        (None, None) => format!("pick: cursor=({},{}) no tile picked", cursor.x, cursor.y),
        (picked, expected) => format!(
            "pick: cursor=({},{}) picked={picked:?} expected={expected:?}",
            cursor.x, cursor.y
        ),
    }
}

fn tile_text([x, y, z]: [i32; 3]) -> String {
    format!("[{x},{y},{z}]")
}

#[cfg(test)]
mod pick_tests {
    use bevy::prelude::Vec2;

    use super::{pick_ambiguity_line, pick_capture_line, tile_half_extent_px};

    #[test]
    fn capture_pick_line_changes_with_the_cursor_and_names_no_pick() {
        let first = pick_capture_line(Vec2::new(100.0, 200.0), Some([1, 2, 3]), Some([1, 2, 3]));
        let second = pick_capture_line(Vec2::new(300.0, 200.0), Some([4, 2, 3]), Some([4, 2, 3]));
        assert_ne!(
            first, second,
            "different scripted cursors must report different picks"
        );
        assert_eq!(
            first,
            "pick: cursor=(100,200) picked=[1,2,3] expected=[1,2,3]"
        );
        assert_eq!(
            pick_capture_line(Vec2::new(0.0, 0.0), None, None),
            "pick: cursor=(0,0) no tile picked"
        );
    }

    /// The oracle's window must be a tile's own apparent size, not a constant. Expected values
    /// are hand-computed from the lens equation (`0.5 * 1080 / (2 * d * tan(pi/8))` = `651.87/d`)
    /// rather than read back out of the function under test.
    #[test]
    fn the_oracle_window_is_the_tiles_own_half_extent_at_that_depth() {
        for (depth, expected) in [
            (4.0, 162.967),
            (30.0, 21.729),
            (90.0, 7.243),
            (500.0, 1.304),
        ] {
            let measured = tile_half_extent_px(depth, 1080.0);
            assert!(
                (measured - expected).abs() < 0.01,
                "at depth {depth} the window must be {expected} px, measured {measured}"
            );
        }
        // The property the fixed 32 px got wrong at BOTH clamps, stated directly: it was five
        // times too narrow at the near clamp and twenty-four times too wide at the far one.
        assert!(tile_half_extent_px(4.0, 1080.0) > 32.0);
        assert!(tile_half_extent_px(500.0, 1080.0) < 32.0);
    }

    #[test]
    fn an_ambiguous_window_names_every_tile_in_it() {
        let line = pick_ambiguity_line(
            Vec2::new(960.0, 540.0),
            &[(4.0, [1, 2, 3]), (9.0, [4, 5, 6])],
        );
        assert!(
            line.contains("[1,2,3]") && line.contains("[4,5,6]"),
            "an ambiguous window must name every candidate, not just the one asserted against: \
             {line}"
        );
        assert!(
            line.starts_with("pick: WARNING cursor=(960,540) 2 tiles"),
            "{line}"
        );
    }
}

fn exit_after_capture(_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

/// Writes the PNG and only THEN validates it, in one observer.
///
/// These were two observers on the same entity — `save_to_disk` registered first, the range
/// checks second — and Bevy runs entity observers for one event in an unspecified order. It
/// consistently ran the checks first, so a failing range check panicked before the file was
/// ever written: the run whose frame most needed looking at was the one run that produced no
/// frame. Measured on the vehicle 2026-08-20, and visible in every passing run's log too, where
/// `capture range check:` prints above `Screenshot saved to`.
///
/// Sequencing them inside a single observer is the fix; registration order cannot express it.
fn save_then_validate(path: PathBuf, slice: SliceLevel) -> impl FnMut(On<ScreenshotCaptured>) {
    let mut save = save_to_disk(path);
    move |event: On<ScreenshotCaptured>| {
        let bytes = event
            .image
            .data
            .as_deref()
            .expect("capture screenshot must include pixel data")
            .to_vec();
        let format = event.image.texture_descriptor.format;
        let size = event.image.texture_descriptor.size;
        // The saver consumes the event, so the pixels are taken first. It is synchronous: it
        // writes the file and logs before returning, so the PNG exists by the next line.
        save_before_validate(
            || save(event),
            || {
                validate_capture_ranges(
                    &bytes,
                    format,
                    size.width,
                    size.height,
                    range_band_applies(slice),
                    slice.level(),
                )
            },
        );
    }
}

/// Writes first, judges second — the ordering itself, split out so it can be tested.
///
/// *(Mechanism is the requirement here, the same justification AC5 and AC6 carry: the live saver
/// needs a real render surface, so the only way this ordering can go red in `cargo test` is if the
/// sequence exists apart from the Bevy plumbing it sequences.)*
fn save_before_validate(save: impl FnOnce(), validate: impl FnOnce()) {
    save();
    validate();
}

/// Whether 5.4's calibrated band describes the frame about to be judged.
///
/// The floor and ceiling were measured on the APPROVED ARTIFACT at the boot framing, and their own
/// wording says what they watch: "the valley floor", "night snow stays midtone". A cut removes
/// everything above it, so the sample window stops showing sky-lit snow and starts showing the
/// interior rock the cut exposes — darker by material, not by any light regression. Measured on the
/// vehicle 2026-08-20: a z 9 capture read 67 against the 70 floor and Wolf confirmed by eye that
/// the picture was fine.
///
/// This is the same correction the 2026-08-19 review made one assertion higher up, for the lantern
/// checks, and stopped short of making here: an operator asking for a lower slice must not read as
/// a defect. Scoped strictly to cuts BELOW the top, so every full-depth capture — which is every
/// capture stories 6.1 and 6.2 take — is judged exactly as before.
fn range_band_applies(slice: SliceLevel) -> bool {
    slice.level() >= slice.top()
}

pub fn validate_capture_ranges(
    bytes: &[u8],
    format: TextureFormat,
    width: u32,
    height: u32,
    band_applies: bool,
    level: i32,
) {
    validate_capture_ranges_with_report(
        bytes,
        format,
        width,
        height,
        band_applies,
        level,
        |line| println!("{line}"),
    );
}

fn validate_capture_ranges_with_report(
    bytes: &[u8],
    format: TextureFormat,
    width: u32,
    height: u32,
    band_applies: bool,
    level: i32,
    mut report: impl FnMut(&str),
) {
    let pixels = decode_rgba8(bytes, format);
    let warm = warm_lit_pixels(&pixels);
    let ground = median_ground_luminance(&pixels, width, height);
    let blown_pool =
        largest_blown_pool_fraction(&pixels, width, height, BLOWN_POOL_LUMINANCE_THRESHOLD);
    let p99 = p99_luminance(&pixels);
    report(&format!(
        "capture range check: warm-lit pixels={warm} ground-median-luminance={ground} \
         blown-pool={:.4}% p99-luminance={p99:.1}",
        blown_pool * 100.0
    ));
    assert!(
        pixels.iter().any(|pixel| pixel[..3] != [0, 0, 0]),
        "capture is black"
    );
    assert!(
        pixels.windows(2).any(|pair| pair[0] != pair[1]),
        "capture is uniform"
    );
    // The numbers print either way. Only the calibrated band is conditional, and `capture is
    // black` / `capture is uniform` above are not — a slice capture is never left ungated.
    if !band_applies {
        println!(
            "capture range check: the cut at z {level} is below the world top, where 5.4's band \
             was measured on sky-lit snow — warm, ground, and blown-pool assertions skipped"
        );
        return;
    }
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
    assert!(
        blown_pool <= BLOWN_POOL_FRACTION_CEILING,
        "the largest near-white pool is {:.4}%, above the {:.4}% ceiling calibrated on boot7.png",
        blown_pool * 100.0,
        BLOWN_POOL_FRACTION_CEILING * 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The band must still bite at full depth, and must NOT bite at a cut. A dark frame is the
    /// discriminator: identical pixels, opposite verdicts, decided only by where the cut sits.
    ///
    /// The dark value is hand-written at 20 — comfortably under the shipped floor without being
    /// derived from it, so raising the floor cannot quietly make this test vacuous.
    #[test]
    fn the_calibrated_band_judges_the_boot_framing_and_stands_aside_at_a_cut() {
        let dims = protocol::Dims { x: 4, y: 4, z: 8 };
        let top = SliceLevel::at_world_top(dims);
        let cut = SliceLevel::pinned(dims, 3);
        assert!(range_band_applies(top), "full depth is the calibrated case");
        assert!(
            !range_band_applies(cut),
            "a cut exposes interior rock, which the band was never measured against"
        );

        // A near-black field with one lighter pixel: dark enough to fail the ground floor, varied
        // enough to clear the unconditional `black` and `uniform` gates above it.
        let mut bytes = vec![20u8; 4 * 64];
        bytes[0..4].copy_from_slice(&[30, 30, 30, 255]);

        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let at_top = std::panic::catch_unwind(|| {
            validate_capture_ranges(&bytes, TextureFormat::Rgba8Unorm, 8, 8, true, top.level());
        });
        let at_cut = std::panic::catch_unwind(|| {
            validate_capture_ranges(&bytes, TextureFormat::Rgba8Unorm, 8, 8, false, cut.level());
        });
        std::panic::set_hook(previous);

        assert!(
            at_top.is_err(),
            "a black field at the boot framing is still the failure 5.4's floor exists to catch"
        );
        assert!(
            at_cut.is_ok(),
            "the same frame at a cut must not be judged against a band measured on sky-lit snow"
        );
    }

    /// The run that fails its range checks is the run whose frame most needs looking at, and until
    /// 2026-08-20 it was the one run that produced no frame: `save_to_disk` and the range checks
    /// were two observers on one event, Bevy runs those in an unspecified order, and it picked the
    /// checks first. Measured on the vehicle — a z 9 capture panicked on the ground-luminance floor
    /// and wrote no PNG — and visible in every passing run's log too, where `capture range check:`
    /// prints above `Screenshot saved to`.
    ///
    /// The validation must still fail loudly. The point is only that the evidence survives it.
    #[test]
    fn the_capture_is_written_before_it_is_judged() {
        let saved = std::cell::Cell::new(false);
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            save_before_validate(
                || saved.set(true),
                || panic!("the valley floor reads 67, below the 70 value floor"),
            );
        }));
        std::panic::set_hook(previous);

        assert!(
            outcome.is_err(),
            "a failing range check must still panic — this is not a way to silence it"
        );
        assert!(
            saved.get(),
            "the PNG must be written before the range checks can panic, or a failing capture \
             destroys the frame that would explain it"
        );
    }

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
    fn blown_pool_uses_the_largest_four_connected_region_and_reports_p99() {
        let mut pixels = vec![[20, 20, 20, 255]; 16];
        for index in [0, 1, 4, 5, 15] {
            pixels[index] = [220, 220, 220, 255];
        }

        assert_eq!(largest_blown_pool_fraction(&pixels, 4, 4, 200), 0.25);
        assert_eq!(p99_luminance(&pixels), 220.0);
    }

    #[test]
    fn blown_pool_ceiling_judges_the_boot_framing_and_stands_aside_at_a_cut() {
        let mut bytes = vec![0; 64 * 64 * 4];
        for pixel in bytes.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[195, 150, 130, 255]);
        }
        for row in 0..20 {
            for column in 0..20 {
                let start = (row * 64 + column) * 4;
                bytes[start..start + 4].copy_from_slice(&[230, 230, 230, 255]);
            }
        }

        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let at_top = std::panic::catch_unwind(|| {
            validate_capture_ranges(&bytes, TextureFormat::Rgba8Unorm, 64, 64, true, 9);
        });
        let at_cut = std::panic::catch_unwind(|| {
            validate_capture_ranges(&bytes, TextureFormat::Rgba8Unorm, 64, 64, false, 8);
        });
        std::panic::set_hook(previous);

        assert!(
            at_top.is_err(),
            "a large near-white pool at the boot framing must fail its own ceiling"
        );
        assert!(
            at_cut.is_ok(),
            "the same frame at a cut must keep skipping the boot-vista range checks"
        );
    }

    #[test]
    fn capture_range_report_is_emitted_before_a_blown_pool_panic() {
        let mut bytes = vec![0; 64 * 64 * 4];
        for pixel in bytes.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[195, 150, 130, 255]);
        }
        for row in 0..20 {
            for column in 0..20 {
                let start = (row * 64 + column) * 4;
                bytes[start..start + 4].copy_from_slice(&[230, 230, 230, 255]);
            }
        }

        let reported = std::cell::Cell::new(false);
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            validate_capture_ranges_with_report(
                &bytes,
                TextureFormat::Rgba8Unorm,
                64,
                64,
                true,
                9,
                |line| {
                    // Latch, never assign: a second report line must not be able to clear this.
                    if line.contains("blown-pool=") && line.contains("p99-luminance=") {
                        reported.set(true);
                    }
                },
            );
        }));
        std::panic::set_hook(previous);

        assert!(
            reported.get(),
            "the metrics must be reported before the ceiling panics"
        );
        assert!(
            outcome.is_err(),
            "the ceiling must still make the observer panic"
        );
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
        let empty = DrawStats::new(4, 0, 0, 0, 0, 0, 0, 0);
        assert!(
            std::panic::catch_unwind(|| empty.assert_valid(true)).is_err(),
            "a capture must not claim success when its requested slice drew nothing"
        );
        DrawStats::new(4, 12, 5, 5, 1, 1, 1, 1).assert_valid(true);
        assert!(
            std::panic::catch_unwind(|| DrawStats::new(4, 12, 5, 5, 0, 0, 0, 0).assert_valid(true))
                .is_err(),
            "a terrain draw without marks must not claim a working-order capture"
        );
    }

    /// What the oracle buys over the `> 0` it replaced, in both directions. `> 0` is blind to a
    /// projection that drops SOME of its marks — the commonest shape of a partial regression —
    /// and it cannot let a legitimately markless view through, which is why it turned 7.1's own
    /// shipped recipe into a panic.
    #[test]
    fn mark_counts_are_checked_against_the_mirror_not_merely_against_zero() {
        assert!(
            std::panic::catch_unwind(|| DrawStats::new(4, 12, 5, 5, 3, 2, 6, 2).assert_valid(true))
                .is_err(),
            "half the designations going missing must fail, and `> 0` could not see it"
        );
        assert!(
            std::panic::catch_unwind(|| DrawStats::new(4, 12, 5, 5, 2, 1, 2, 2).assert_valid(true))
                .is_err(),
            "a zone going missing must fail, and `> 0` could not see it"
        );
        // A capture that never claimed to be of a working site, over a view the mirror agrees is
        // markless: correct, and the old instrument panicked on it.
        DrawStats::new(4, 12, 5, 5, 0, 0, 0, 0).assert_valid(false);
        // AC15's non-zero range check, for the path that could not reach it. A `--drag` that
        // designated nothing used to evaluate `0 == 0`, print a pass, save the PNG and exit 0 —
        // the 7.2 empty-site false pass, reproduced on the new instrument.
        assert!(
            std::panic::catch_unwind(|| DrawStats::new(4, 12, 5, 5, 0, 0, 0, 0)
                .assert_drag_produced_work(DesignateMode::Dig))
            .is_err(),
            "a dig drag that produced no designation must fail, not pass silently"
        );
        assert!(
            std::panic::catch_unwind(|| DrawStats::new(4, 12, 5, 5, 0, 0, 0, 0)
                .assert_drag_produced_work(DesignateMode::Stockpile))
            .is_err(),
            "a stockpile drag that produced no zone must fail: a rect on non-standable ground is \
             silently dropped by the sim"
        );
        DrawStats::new(4, 12, 5, 5, 1, 0, 1, 0).assert_drag_produced_work(DesignateMode::Dig);
        DrawStats::new(4, 12, 5, 5, 0, 1, 0, 1).assert_drag_produced_work(DesignateMode::Stockpile);
        // Clear REMOVES; ending with nothing to count is the correct outcome, not a failure.
        DrawStats::new(4, 12, 5, 5, 0, 0, 0, 0).assert_drag_produced_work(DesignateMode::Clear);
    }

    #[test]
    fn draw_count_instrument_rejects_a_hollow_cut_that_kept_its_total_up() {
        // The defect the old instrument could not see: boundary tiles keep the global count high
        // while the cut face itself is missing. These are the measured 9x9x9 figures — 258 tiles
        // correct against 209 hollow, 81 cut-face tiles against 32.
        let hollow = DrawStats::new(4, 209, 32, 81, 1, 1, 1, 1);
        assert!(
            std::panic::catch_unwind(|| hollow.assert_valid(true)).is_err(),
            "a cut drawn with no floor is not a capture result, however many tiles it drew"
        );
        DrawStats::new(4, 258, 81, 81, 1, 1, 1, 1).assert_valid(true);
    }

    #[test]
    fn a_level_with_no_solid_tiles_is_a_legitimate_empty_cut_face() {
        // Slicing into open sky above the mountain: the mirror says nothing is there, so drawing
        // nothing at the cut is correct and must not be read as a hollow shell.
        DrawStats::new(30, 53_365, 0, 0, 1, 1, 1, 1).assert_valid(true);
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
