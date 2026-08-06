#![forbid(unsafe_code)]

mod save;
mod worldgen;

pub use save::{SaveState, SavedDwarf};

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
};

use bevy_ecs::{
    component::Component,
    resource::Resource,
    schedule::{IntoScheduleConfigs, Schedule},
    system::{Query, Res, ResMut},
    world::World as EcsWorld,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

const STREAM_WORLDGEN: u64 = 0x4652_4f53_5456_4549;
const STREAM_SPAWN: u64 = 0x5350_4157_4e5f_5f5f;
const STREAM_WANDER: u64 = 0x5741_4e44_4552_5f5f;
const WANDER_RADIUS: i32 = 3;
const WANDER_REST_TICKS: u32 = 10;
const MAX_DESIGNATIONS: usize = 4096;
const MAX_ASTAR_NODES: usize = 50_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Material {
    Stone,
    Soil,
    Ice,
    Snow,
}

/// A voxel. `Empty` is air; `Solid` is wall/floor; `Ramp` is a walkable slope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Solid(Material),
    Ramp(Material),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Component, Serialize, Deserialize)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesignationKind {
    Dig,
    Channel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub min: Pos,
    pub max: Pos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimCommand {
    Designate { kind: DesignationKind, rect: Rect },
    CancelDesignation { rect: Rect },
    PlaceStockpile { rect: Rect },
    RemoveStockpile { rect: Rect },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dims {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Dims {
    pub const DEFAULT: Dims = Dims {
        x: 128,
        y: 128,
        z: 32,
    };
}

/// Sim-assigned stable entity id (AD-9). One allocator for every entity kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Component)]
pub struct Id(pub u32);

#[derive(Component)]
pub struct Dwarf;

#[derive(Component)]
struct Item;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Serialize, Deserialize)]
pub enum JobState {
    Idle,
    Walk,
    Work,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JobId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind {
    Dig,
    Channel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub kind: JobKind,
    pub target: Pos,
    pub created_tick: u64,
    pub retry_after: u64,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct CurrentJob(Option<JobId>);

/// The map and target index move together through `insert` and `remove`.
#[derive(Resource, Default)]
struct Jobs {
    by_id: BTreeMap<JobId, Job>,
    targets: BTreeSet<Pos>,
    next_id: u32,
}

impl Jobs {
    fn insert(&mut self, job: Job) -> bool {
        if self.by_id.contains_key(&job.id) || self.targets.contains(&job.target) {
            return false;
        }
        self.targets.insert(job.target);
        self.by_id.insert(job.id, job);
        true
    }

    #[allow(dead_code)] // Used by completion and cancellation later in this story.
    fn remove(&mut self, id: JobId) -> Option<Job> {
        let job = self.by_id.remove(&id)?;
        self.targets.remove(&job.target);
        Some(job)
    }

    #[allow(dead_code)] // Used by retry handling later in this story.
    fn get_mut(&mut self, id: JobId) -> Option<&mut Job> {
        self.by_id.get_mut(&id)
    }

    fn iter(&self) -> impl Iterator<Item = &Job> {
        self.by_id.values()
    }
}

fn create_jobs(tick: Res<Tick>, designations: Res<Designations>, mut jobs: ResMut<Jobs>) {
    for (&target, &designation) in &designations.0 {
        if jobs.targets.contains(&target) {
            continue;
        }
        let id = JobId(jobs.next_id);
        jobs.next_id += 1;
        let kind = match designation {
            DesignationKind::Dig => JobKind::Dig,
            DesignationKind::Channel => JobKind::Channel,
        };
        let inserted = jobs.insert(Job {
            id,
            kind,
            target,
            created_tick: tick.0,
            retry_after: 0,
        });
        debug_assert!(inserted, "target and id were checked before insertion");
    }
}

/// FR5 / AD-7: fixed named FNV-1a, independent of RNG streams and process state.
fn reaction_delay(seed: u64, dwarf: Id, job: JobId) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in seed.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    for byte in dwarf.0.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    for byte in job.0.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    5 + hash % 26
}

fn claim_jobs(
    seed: Res<Seed>,
    tick: Res<Tick>,
    jobs: Res<Jobs>,
    mut dwarves: Query<(&Id, &mut CurrentJob)>,
) {
    let mut dwarves: Vec<_> = dwarves.iter_mut().collect();
    dwarves.sort_by_key(|(id, _)| **id);
    let mut claimed: BTreeSet<_> = dwarves
        .iter()
        .filter_map(|(_, current)| current.0)
        .collect();

    for job in jobs.iter() {
        if claimed.contains(&job.id) || tick.0 < job.retry_after {
            continue;
        }
        for (id, current) in &mut dwarves {
            if current.0.is_none()
                && tick.0
                    >= job
                        .created_tick
                        .saturating_add(reaction_delay(seed.0, **id, job.id))
            {
                current.0 = Some(job.id);
                claimed.insert(job.id);
                break;
            }
        }
    }
}

#[derive(Component)]
struct Wander {
    home: Pos,
    cooldown: u32,
}

#[derive(Resource)]
struct WanderRng(ChaCha8Rng);

#[derive(Resource)]
struct Tick(pub u64);

#[derive(Resource)]
struct Seed(u64);

#[derive(Resource, Default)]
struct Designations(BTreeMap<Pos, DesignationKind>);

#[derive(Resource, Default)]
struct Zones(BTreeSet<Pos>);

/// The tile grid lives in the ECS so systems can read it; `World` delegates.
#[derive(Resource)]
struct Terrain {
    dims: Dims,
    tiles: Vec<Tile>,
    dirty: BTreeSet<Pos>,
}

impl Terrain {
    fn tile(&self, p: Pos) -> Option<Tile> {
        if p.x < 0
            || p.y < 0
            || p.z < 0
            || p.x >= self.dims.x as i32
            || p.y >= self.dims.y as i32
            || p.z >= self.dims.z as i32
        {
            return None;
        }

        Some(self.tiles[worldgen::index(self.dims, p.x as u32, p.y as u32, p.z as u32)])
    }

    fn set_tile(&mut self, p: Pos, tile: Tile) -> bool {
        if self.tile(p).is_none() {
            return false;
        }

        let index = worldgen::index(self.dims, p.x as u32, p.y as u32, p.z as u32);
        self.tiles[index] = tile;
        self.dirty.insert(p);
        true
    }

    fn drain_dirty(&mut self) -> Vec<(Pos, Tile)> {
        std::mem::take(&mut self.dirty)
            .into_iter()
            .map(|pos| {
                let tile = self
                    .tile(pos)
                    .expect("dirty positions must have passed set_tile bounds checking");
                (pos, tile)
            })
            .collect()
    }

    fn is_standable(&self, p: Pos) -> bool {
        matches!(self.tile(p), Some(Tile::Empty))
            && matches!(
                self.tile(Pos { z: p.z - 1, ..p }),
                Some(Tile::Solid(_) | Tile::Ramp(_))
            )
    }
}

fn astar_neighbours(terrain: &Terrain, from: Pos) -> Vec<Pos> {
    const DIRECTIONS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let mut neighbours = Vec::with_capacity(12);
    for (dx, dy) in DIRECTIONS {
        let candidate = Pos {
            x: from.x + dx,
            y: from.y + dy,
            z: from.z,
        };
        if terrain.is_standable(candidate) {
            neighbours.push(candidate);
        }
    }
    for dz in [-1, 1] {
        for (dx, dy) in DIRECTIONS {
            let candidate = Pos {
                x: from.x + dx,
                y: from.y + dy,
                z: from.z + dz,
            };
            let lower = if candidate.z < from.z {
                candidate
            } else {
                from
            };
            if terrain.is_standable(candidate)
                && matches!(
                    terrain.tile(Pos {
                        z: lower.z - 1,
                        ..lower
                    }),
                    Some(Tile::Ramp(_))
                )
            {
                neighbours.push(candidate);
            }
        }
    }
    neighbours
}

fn astar_heuristic(from: Pos, goals: &BTreeSet<Pos>) -> u32 {
    goals
        .iter()
        .map(|goal| from.x.abs_diff(goal.x) + from.y.abs_diff(goal.y) + from.z.abs_diff(goal.z))
        .min()
        .unwrap_or(0)
}

#[allow(dead_code)] // Called by job execution in the next task group.
fn astar(terrain: &Terrain, from: Pos, goals: &BTreeSet<Pos>) -> Option<Vec<Pos>> {
    if goals.is_empty() {
        return None;
    }
    let mut open = BinaryHeap::from([Reverse((astar_heuristic(from, goals), from))]);
    let mut came_from = BTreeMap::new();
    let mut costs = BTreeMap::from([(from, 0_u32)]);
    let mut expanded = 0;

    while let Some(Reverse((queued_f, current))) = open.pop() {
        let current_cost = costs[&current];
        if queued_f != current_cost + astar_heuristic(current, goals) {
            continue;
        }
        if expanded >= MAX_ASTAR_NODES {
            return None;
        }
        expanded += 1;
        if goals.contains(&current) {
            let mut path = Vec::new();
            let mut cursor = current;
            while cursor != from {
                path.push(cursor);
                cursor = came_from[&cursor];
            }
            path.reverse();
            return Some(path);
        }

        for neighbour in astar_neighbours(terrain, current) {
            let next_cost = current_cost + 1;
            if next_cost < costs.get(&neighbour).copied().unwrap_or(u32::MAX) {
                costs.insert(neighbour, next_cost);
                came_from.insert(neighbour, current);
                open.push(Reverse((
                    next_cost + astar_heuristic(neighbour, goals),
                    neighbour,
                )));
            }
        }
    }
    None
}

fn advance_tick(mut tick: ResMut<Tick>) {
    tick.0 += 1;
}

fn wander(
    mut rng: ResMut<WanderRng>,
    terrain: Res<Terrain>,
    mut dwarves: Query<(&Id, &mut Pos, &mut Wander, &mut JobState)>,
) {
    // AD-7: query iteration is archetype order, not Id order, and all dwarves draw from
    // one stream. Draw order is a sim outcome, so sort before touching the RNG.
    let mut dwarves: Vec<_> = dwarves.iter_mut().collect();
    dwarves.sort_by_key(|(id, ..)| **id);

    for (_, mut pos, mut wander, mut state) in dwarves {
        // NOTE: a resting dwarf never re-checks the tile it is standing on, so terrain
        // mutated underneath it goes unnoticed until its cooldown expires — it will report
        // standing inside solid rock, or hovering with no floor, for up to
        // WANDER_REST_TICKS - 1 ticks. Unreachable while `set_tile` has no production
        // caller; Story 3.2's dig is the first, and owns the fix along with gravity.
        if wander.cooldown > 0 {
            wander.cooldown -= 1;
            *state = JobState::Idle;
            continue;
        }

        let here = *pos;
        // NOTE: fixed order, same z only. Ramp climbing arrives with A* in Story 3.2.
        let candidates: Vec<Pos> = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .map(|(dx, dy)| Pos {
                x: here.x + dx,
                y: here.y + dy,
                z: here.z,
            })
            // NOTE: standability only — occupancy is not checked, so two dwarves whose home
            // boxes overlap can share a tile, and `view::render` draws them in ascending Id
            // into the same cell, silently hiding the lower one. Tile claiming arrives with
            // Story 3.2's jobs, which needs a reservation model anyway.
            .filter(|p| {
                (p.x - wander.home.x).abs() <= WANDER_RADIUS
                    && (p.y - wander.home.y).abs() <= WANDER_RADIUS
                    && terrain.is_standable(*p)
            })
            .collect();
        wander.cooldown = WANDER_REST_TICKS;
        match candidates.len() {
            0 => *state = JobState::Idle,
            n => {
                *pos = candidates[rng.0.random_range(0..n)];
                *state = JobState::Walk;
            }
        }
    }
}

#[derive(Resource, Default)]
struct IdAllocator {
    next: u32,
}

impl IdAllocator {
    fn allocate(&mut self) -> Id {
        let id = Id(self.next);
        self.next += 1;
        id
    }
}

pub struct World {
    ecs: EcsWorld,
    schedule: Schedule,
}

// The one assembly site intentionally receives every deterministic state component so generate
// and load cannot diverge; wrapping them solely to satisfy the argument-count lint adds no model.
#[allow(clippy::too_many_arguments)]
fn assemble(
    seed: u64,
    dims: Dims,
    tiles: Vec<Tile>,
    tick: u64,
    wander_rng: ChaCha8Rng,
    ids: IdAllocator,
    designations: BTreeMap<Pos, DesignationKind>,
    zones: BTreeSet<Pos>,
) -> World {
    let mut ecs = EcsWorld::new();
    ecs.insert_resource(Tick(tick));
    ecs.insert_resource(Seed(seed));
    ecs.insert_resource(WanderRng(wander_rng));
    ecs.insert_resource(ids);
    ecs.insert_resource(Designations(designations));
    ecs.insert_resource(Zones(zones));
    ecs.insert_resource(Jobs::default());
    ecs.insert_resource(Terrain {
        dims,
        tiles,
        dirty: BTreeSet::new(),
    });
    let mut schedule = Schedule::default();
    schedule.add_systems((advance_tick, create_jobs, claim_jobs, wander).chain());
    World { ecs, schedule }
}

impl World {
    /// # Panics
    ///
    /// Only `Dims::DEFAULT`-scale worlds are supported. Smaller worlds panic:
    /// `dims.z <= 4` inverts the height clamp, and a footprint yielding fewer than
    /// five flat columns exhausts the spawn candidate list.
    // NOTE: worldgen supports z >= 6 and a footprint with at least 5 flat columns.
    // z == 5 collapses the height range to a single level: no variation, no ramps.
    // Small-world support arrives when a scenario test actually needs one.
    pub fn generate(seed: u64, dims: Dims) -> World {
        debug_assert!(dims.z >= 6, "worldgen needs dims.z >= 6, got {}", dims.z);
        debug_assert!(
            dims.x >= 3 && dims.y >= 3,
            "worldgen needs at least 5 flat columns, got {}x{}",
            dims.x,
            dims.y
        );
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_WORLDGEN);
        let heights = worldgen::height_field(dims, &mut rng);
        let mut tiles = worldgen::layered_terrain(dims, &heights, &mut rng);
        worldgen::place_ramps(dims, &heights, &mut tiles);
        let mut spawn_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_SPAWN);

        let mut world = assemble(
            seed,
            dims,
            tiles,
            0,
            ChaCha8Rng::seed_from_u64(seed ^ STREAM_WANDER),
            IdAllocator::default(),
            BTreeMap::new(),
            BTreeSet::new(),
        );
        world.spawn_dwarves(&heights, &mut spawn_rng);
        world
    }

    pub fn to_save(&self) -> SaveState {
        let terrain = self.ecs.resource::<Terrain>();
        let mut dwarves: Vec<_> = self
            .ecs
            .iter_entities()
            .filter(|entity| entity.contains::<Dwarf>())
            // NOTE: a `Dwarf` missing any of these components is skipped rather than reported, so
            // it would vanish from the save silently. Both construction sites (`spawn_dwarves` and
            // `from_save`) attach the whole set together, so that cannot happen today; a story
            // that spawns dwarves a third way must keep the set intact.
            .filter_map(|entity| {
                let wander = entity.get::<Wander>()?;
                Some(SavedDwarf {
                    id: entity.get::<Id>()?.0,
                    pos: *entity.get::<Pos>()?,
                    state: *entity.get::<JobState>()?,
                    home: wander.home,
                    cooldown: wander.cooldown,
                })
            })
            .collect();
        dwarves.sort_by_key(|dwarf| dwarf.id);

        SaveState {
            seed: self.seed(),
            tick: self.tick(),
            dims: terrain.dims,
            tiles: terrain.tiles.clone(),
            wander_rng: self.ecs.resource::<WanderRng>().0.clone(),
            next_id: self.ecs.resource::<IdAllocator>().next,
            dwarves,
            designations: self.designations(),
            zones: self.zones(),
        }
    }

    pub fn from_save(save: SaveState) -> World {
        let SaveState {
            seed,
            tick,
            dims,
            tiles,
            wander_rng,
            next_id,
            dwarves,
            designations,
            zones,
        } = save;
        let mut world = assemble(
            seed,
            dims,
            tiles,
            tick,
            wander_rng,
            IdAllocator { next: next_id },
            designations.into_iter().collect(),
            zones.into_iter().collect(),
        );
        for dwarf in dwarves {
            world.ecs.spawn((
                Dwarf,
                Id(dwarf.id),
                dwarf.pos,
                dwarf.state,
                Wander {
                    home: dwarf.home,
                    cooldown: dwarf.cooldown,
                },
                CurrentJob(None),
            ));
        }
        world
    }

    pub fn dims(&self) -> Dims {
        self.ecs.resource::<Terrain>().dims
    }

    pub fn seed(&self) -> u64 {
        self.ecs.resource::<Seed>().0
    }

    pub fn tick(&self) -> u64 {
        self.ecs.resource::<Tick>().0
    }

    pub fn step(&mut self) {
        self.schedule.run(&mut self.ecs);
    }

    /// Flat row-major: index = x + y*dims.x + z*dims.x*dims.y
    pub fn tiles(&self) -> &[Tile] {
        &self.ecs.resource::<Terrain>().tiles
    }

    pub fn tile(&self, p: Pos) -> Option<Tile> {
        self.ecs.resource::<Terrain>().tile(p)
    }

    pub fn set_tile(&mut self, p: Pos, tile: Tile) -> bool {
        self.ecs.resource_mut::<Terrain>().set_tile(p, tile)
    }

    pub fn drain_dirty(&mut self) -> Vec<(Pos, Tile)> {
        self.ecs.resource_mut::<Terrain>().drain_dirty()
    }

    /// AD-10: `simd` calls this at loop-iteration start, in arrival order, including while
    /// paused. Designation intake changes marks only; it is not world advancement.
    // NOTE: command ordering is explicit at the call site rather than enforced by `.chain()`.
    pub fn apply_command(&mut self, command: SimCommand) {
        let dims = self.dims();
        let rect = match command {
            SimCommand::Designate { rect, .. }
            | SimCommand::CancelDesignation { rect }
            | SimCommand::PlaceStockpile { rect }
            | SimCommand::RemoveStockpile { rect } => rect,
        };
        let min = Pos {
            x: rect.min.x.min(rect.max.x),
            y: rect.min.y.min(rect.max.y),
            z: rect.min.z.min(rect.max.z),
        };
        let max = Pos {
            x: rect.min.x.max(rect.max.x),
            y: rect.min.y.max(rect.max.y),
            z: rect.min.z.max(rect.max.z),
        };
        if max.x < 0
            || max.y < 0
            || max.z < 0
            || min.x >= dims.x as i32
            || min.y >= dims.y as i32
            || min.z >= dims.z as i32
        {
            return;
        }
        let min = Pos {
            x: min.x.max(0),
            y: min.y.max(0),
            z: min.z.max(0),
        };
        let max = Pos {
            x: max.x.min(dims.x as i32 - 1),
            y: max.y.min(dims.y as i32 - 1),
            z: max.z.min(dims.z as i32 - 1),
        };

        let positions = || {
            (min.z..=max.z).flat_map(move |z| {
                (min.y..=max.y).flat_map(move |y| (min.x..=max.x).map(move |x| Pos { x, y, z }))
            })
        };
        match command {
            SimCommand::Designate { kind, .. } => {
                let workable: Vec<_> = {
                    let terrain = self.ecs.resource::<Terrain>();
                    positions()
                        .filter(|pos| match kind {
                            DesignationKind::Dig => {
                                matches!(terrain.tile(*pos), Some(Tile::Solid(_)))
                            }
                            DesignationKind::Channel => terrain.is_standable(*pos),
                        })
                        .collect()
                };
                let mut designations = self.ecs.resource_mut::<Designations>();
                for pos in workable {
                    if designations.0.len() >= MAX_DESIGNATIONS
                        && !designations.0.contains_key(&pos)
                    {
                        continue;
                    }
                    designations.0.insert(pos, kind);
                }
            }
            SimCommand::CancelDesignation { .. } => {
                let mut designations = self.ecs.resource_mut::<Designations>();
                for pos in positions() {
                    designations.0.remove(&pos);
                }
            }
            SimCommand::PlaceStockpile { .. } => {
                let standable: Vec<_> = {
                    let terrain = self.ecs.resource::<Terrain>();
                    positions()
                        .filter(|pos| terrain.is_standable(*pos))
                        .collect()
                };
                let mut zones = self.ecs.resource_mut::<Zones>();
                zones.0.extend(standable);
            }
            SimCommand::RemoveStockpile { .. } => {
                let mut zones = self.ecs.resource_mut::<Zones>();
                for pos in positions() {
                    zones.0.remove(&pos);
                }
            }
        }
    }

    /// Sorted ascending by `Pos`.
    pub fn designations(&self) -> Vec<(Pos, DesignationKind)> {
        self.ecs
            .resource::<Designations>()
            .0
            .iter()
            .map(|(&pos, &kind)| (pos, kind))
            .collect()
    }

    /// Sorted ascending by `Pos`.
    pub fn zones(&self) -> Vec<Pos> {
        self.ecs.resource::<Zones>().0.iter().copied().collect()
    }

    /// Sorted ascending by `JobId`.
    pub fn jobs(&self) -> Vec<Job> {
        self.ecs.resource::<Jobs>().iter().copied().collect()
    }

    /// Sorted ascending by dwarf `Id`.
    pub fn claims(&self) -> Vec<(Id, Option<JobId>)> {
        let mut claims: Vec<_> = self
            .ecs
            .iter_entities()
            .filter(|entity| entity.contains::<Dwarf>())
            .filter_map(|entity| Some((*entity.get::<Id>()?, entity.get::<CurrentJob>()?.0)))
            .collect();
        claims.sort_by_key(|(id, _)| *id);
        claims
    }

    /// Sorted ascending by `Id`.
    pub fn items(&self) -> Vec<(Id, Pos)> {
        let mut items: Vec<_> = self
            .ecs
            .iter_entities()
            .filter(|entity| entity.contains::<Item>())
            .filter_map(|entity| Some((*entity.get::<Id>()?, *entity.get::<Pos>()?)))
            .collect();
        items.sort_by_key(|(id, _)| *id);
        items
    }

    /// Sorted ascending by `Id` — stable order is required by AD-7.
    // NOTE: promote this tuple to a struct at the fourth field (Story 3.2 adds carried item).
    pub fn dwarves(&self) -> Vec<(Id, Pos, JobState)> {
        let mut dwarves: Vec<_> = self
            .ecs
            .iter_entities()
            .filter(|entity| entity.contains::<Dwarf>())
            .filter_map(|entity| {
                Some((
                    *entity.get::<Id>()?,
                    *entity.get::<Pos>()?,
                    *entity.get::<JobState>()?,
                ))
            })
            .collect();
        dwarves.sort_by_key(|(id, ..)| *id);
        dwarves
    }

    fn spawn_dwarves(&mut self, heights: &[u32], rng: &mut ChaCha8Rng) {
        let mut candidates = {
            let terrain = self.ecs.resource::<Terrain>();
            let dims = terrain.dims;
            let mut candidates = Vec::new();
            for y in 0..dims.y {
                for x in 0..dims.x {
                    let height = heights[(x + y * dims.x) as usize];
                    let is_flat = [
                        (x as i32 - 1, y as i32),
                        (x as i32 + 1, y as i32),
                        (x as i32, y as i32 - 1),
                        (x as i32, y as i32 + 1),
                    ]
                    .into_iter()
                    .filter(|&(nx, ny)| {
                        nx >= 0 && ny >= 0 && nx < dims.x as i32 && ny < dims.y as i32
                    })
                    .all(|(nx, ny)| heights[(nx as u32 + ny as u32 * dims.x) as usize] == height);
                    let pos = Pos {
                        x: x as i32,
                        y: y as i32,
                        z: height as i32 + 1,
                    };
                    if is_flat && terrain.is_standable(pos) {
                        candidates.push(pos);
                    }
                }
            }
            candidates
        };

        for _ in 0..5 {
            let candidate = rng.random_range(0..candidates.len());
            let pos = candidates.swap_remove(candidate);
            let id = self.ecs.resource_mut::<IdAllocator>().allocate();
            self.ecs.spawn((
                Dwarf,
                id,
                pos,
                JobState::Idle,
                Wander {
                    home: pos,
                    // NOTE: staggers the spawn phases so the dwarves do not step in lockstep,
                    // without spending a second RNG draw. It wraps at WANDER_REST_TICKS, so an
                    // eleventh dwarf would share dwarf 0's phase — harmless at five.
                    cooldown: id.0 % WANDER_REST_TICKS,
                },
                CurrentJob(None),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{Dims, Job, JobId, JobKind, JobState, Jobs, Material, Pos, Terrain, Tile, World};

    fn flat_terrain(x: u32, y: u32) -> Terrain {
        let dims = Dims { x, y, z: 2 };
        let mut tiles = vec![Tile::Empty; (x * y * 2) as usize];
        for floor_y in 0..y {
            for floor_x in 0..x {
                tiles[super::worldgen::index(dims, floor_x, floor_y, 0)] =
                    Tile::Solid(Material::Stone);
            }
        }
        Terrain {
            dims,
            tiles,
            dirty: BTreeSet::new(),
        }
    }

    #[test]
    fn terrain_identifies_standable_tiles() {
        let terrain = Terrain {
            dims: Dims { x: 2, y: 1, z: 2 },
            tiles: vec![
                Tile::Solid(Material::Stone),
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
            ],
            dirty: BTreeSet::new(),
        };

        assert!(terrain.is_standable(Pos { x: 0, y: 0, z: 1 }));
        assert!(!terrain.is_standable(Pos { x: 1, y: 0, z: 1 }));
    }

    #[test]
    fn generated_world_starts_at_tick_zero() {
        let world = World::generate(42, Dims::DEFAULT);

        assert_eq!(world.tick(), 0);
    }

    #[test]
    fn allocator_lives_in_the_ecs() {
        let world = World::generate(42, Dims::DEFAULT);

        assert_eq!(world.ecs.resource::<super::IdAllocator>().next, 5);
    }

    #[test]
    fn jobs_keep_the_target_index_paired_with_the_map() {
        let mut jobs = Jobs::default();
        let target = Pos { x: 3, y: 4, z: 5 };
        let job = Job {
            id: JobId(7),
            kind: JobKind::Dig,
            target,
            created_tick: 9,
            retry_after: 0,
        };

        assert!(jobs.insert(job));
        assert!(jobs.targets.contains(&target));
        assert_eq!(jobs.iter().copied().collect::<Vec<_>>(), vec![job]);
        assert_eq!(jobs.remove(JobId(7)), Some(job));
        assert!(!jobs.targets.contains(&target));
    }

    #[test]
    fn generated_world_has_empty_job_and_claim_readers() {
        let world = World::generate(42, Dims::DEFAULT);

        assert!(world.jobs().is_empty());
        assert_eq!(
            world.claims(),
            vec![
                (super::Id(0), None),
                (super::Id(1), None),
                (super::Id(2), None),
                (super::Id(3), None),
                (super::Id(4), None),
            ]
        );
    }

    #[test]
    fn item_reader_filters_and_sorts_stones() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let later = Pos { x: 9, y: 8, z: 7 };
        let earlier = Pos { x: 1, y: 2, z: 3 };
        world.ecs.spawn((super::Item, super::Id(12), later));
        world.ecs.spawn((super::Item, super::Id(11), earlier));

        assert_eq!(
            world.items(),
            vec![(super::Id(11), earlier), (super::Id(12), later)]
        );
        assert_eq!(world.dwarves().len(), 5);
    }

    #[test]
    fn reaction_delay_table_is_pinned() {
        let expected = [
            [28, 19, 26],
            [17, 10, 15],
            [6, 13, 8],
            [21, 14, 23],
            [10, 17, 8],
        ];

        for dwarf in 0..=4 {
            for job in 0..=2 {
                assert_eq!(
                    super::reaction_delay(42, super::Id(dwarf), JobId(job)),
                    expected[dwarf as usize][job as usize],
                    "seed 42, dwarf {dwarf}, job {job}"
                );
            }
        }
    }

    #[test]
    fn claim_jobs_waits_for_the_reaction_delay() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let target = Pos { x: 40, y: 40, z: 8 };
        assert!(world.set_tile(target, Tile::Solid(Material::Stone)));
        world.apply_command(super::SimCommand::Designate {
            kind: super::DesignationKind::Dig,
            rect: super::Rect {
                min: target,
                max: target,
            },
        });

        for _ in 0..6 {
            world.step();
            assert!(world.claims().iter().all(|(_, job)| job.is_none()));
        }
        world.step();
        assert_eq!(world.claims()[2], (super::Id(2), Some(JobId(0))));
    }

    #[test]
    fn claim_jobs_takes_fifo_and_skips_busy_dwarves_and_claimed_jobs() {
        let mut world = World::generate(42, Dims::DEFAULT);
        {
            let mut query = world.ecs.query::<(&super::Id, &mut super::CurrentJob)>();
            for (id, mut current) in query.iter_mut(&mut world.ecs) {
                if id.0 > 0 {
                    current.0 = Some(JobId(100 + id.0));
                }
            }
        }
        {
            let mut jobs = world.ecs.resource_mut::<Jobs>();
            assert!(jobs.insert(Job {
                id: JobId(0),
                kind: JobKind::Dig,
                target: Pos { x: 1, y: 1, z: 1 },
                created_tick: 0,
                retry_after: 0,
            }));
            assert!(jobs.insert(Job {
                id: JobId(1),
                kind: JobKind::Dig,
                target: Pos { x: 2, y: 1, z: 1 },
                created_tick: 0,
                retry_after: 0,
            }));
        }
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        world.step();
        assert_eq!(world.claims()[0], (super::Id(0), Some(JobId(0))));

        let mut claimed = World::generate(42, Dims::DEFAULT);
        {
            let mut query = claimed.ecs.query::<(&super::Id, &mut super::CurrentJob)>();
            for (id, mut current) in query.iter_mut(&mut claimed.ecs) {
                current.0 = if id.0 == 1 {
                    Some(JobId(0))
                } else if id.0 > 1 {
                    Some(JobId(100 + id.0))
                } else {
                    None
                };
            }
        }
        {
            let mut jobs = claimed.ecs.resource_mut::<Jobs>();
            assert!(jobs.insert(Job {
                id: JobId(0),
                kind: JobKind::Dig,
                target: Pos { x: 1, y: 1, z: 1 },
                created_tick: 0,
                retry_after: 0,
            }));
            assert!(jobs.insert(Job {
                id: JobId(1),
                kind: JobKind::Dig,
                target: Pos { x: 2, y: 1, z: 1 },
                created_tick: 0,
                retry_after: 0,
            }));
        }
        claimed.ecs.resource_mut::<super::Tick>().0 = 100;
        claimed.step();
        assert_eq!(claimed.claims()[0], (super::Id(0), Some(JobId(1))));
    }

    #[test]
    fn astar_finds_the_literal_shortest_corridor_path_repeatably() {
        let terrain = flat_terrain(5, 1);
        let from = Pos { x: 0, y: 0, z: 1 };
        let goal = Pos { x: 4, y: 0, z: 1 };
        let goals = BTreeSet::from([goal]);
        let expected = vec![
            Pos { x: 1, y: 0, z: 1 },
            Pos { x: 2, y: 0, z: 1 },
            Pos { x: 3, y: 0, z: 1 },
            Pos { x: 4, y: 0, z: 1 },
        ];

        assert_eq!(super::astar(&terrain, from, &goals), Some(expected.clone()));
        assert_eq!(super::astar(&terrain, from, &goals), Some(expected));
    }

    #[test]
    fn astar_crosses_only_a_ramp_backed_level_change() {
        let dims = Dims { x: 2, y: 1, z: 3 };
        let mut terrain = Terrain {
            dims,
            tiles: vec![Tile::Empty; 6],
            dirty: BTreeSet::new(),
        };
        terrain.tiles[super::worldgen::index(dims, 0, 0, 0)] = Tile::Solid(Material::Stone);
        terrain.tiles[super::worldgen::index(dims, 1, 0, 1)] = Tile::Solid(Material::Stone);
        super::worldgen::place_ramps(dims, &[0, 1], &mut terrain.tiles);
        let lower = Pos { x: 0, y: 0, z: 1 };
        let higher = Pos { x: 1, y: 0, z: 2 };

        assert_eq!(
            super::astar(&terrain, lower, &BTreeSet::from([higher])),
            Some(vec![higher])
        );
        terrain.tiles[super::worldgen::index(dims, 0, 0, 0)] = Tile::Solid(Material::Stone);
        assert_eq!(
            super::astar(&terrain, lower, &BTreeSet::from([higher])),
            None
        );
    }

    #[test]
    fn astar_returns_none_for_a_walled_off_goal() {
        let mut terrain = flat_terrain(3, 1);
        let wall = Pos { x: 1, y: 0, z: 1 };
        terrain.set_tile(wall, Tile::Solid(Material::Stone));

        assert_eq!(
            super::astar(
                &terrain,
                Pos { x: 0, y: 0, z: 1 },
                &BTreeSet::from([Pos { x: 2, y: 0, z: 1 }]),
            ),
            None
        );
    }

    #[test]
    fn astar_horizontal_neighbour_order_is_pinned() {
        let terrain = flat_terrain(3, 3);
        let center = Pos { x: 1, y: 1, z: 1 };

        assert_eq!(
            super::astar_neighbours(&terrain, center),
            vec![
                Pos { x: 0, y: 1, z: 1 },
                Pos { x: 2, y: 1, z: 1 },
                Pos { x: 1, y: 0, z: 1 },
                Pos { x: 1, y: 2, z: 1 },
            ]
        );
    }

    #[test]
    fn astar_stops_at_the_node_cap() {
        let terrain = flat_terrain(224, 224);

        assert_eq!(
            super::astar(
                &terrain,
                Pos { x: 0, y: 0, z: 1 },
                &BTreeSet::from([Pos {
                    x: 223,
                    y: 223,
                    z: 1,
                }]),
            ),
            None
        );
    }

    #[test]
    fn stepping_advances_the_world_tick_once() {
        let mut world = World::generate(42, Dims::DEFAULT);

        world.step();
        assert_eq!(world.tick(), 1);

        world.step();
        assert_eq!(world.tick(), 2);
    }

    #[test]
    fn dwarves_spawn_idle_and_wander_in_staggered_id_order() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let before = world.dwarves();

        assert!(before.iter().all(|(_, _, state)| *state == JobState::Idle));
        world.step();
        let after = world.dwarves();

        assert_ne!(after[0].1, before[0].1);
        assert_eq!(after[0].2, JobState::Walk);
        for index in 1..5 {
            assert_eq!(after[index].1, before[index].1);
            assert_eq!(after[index].2, JobState::Idle);
        }
    }

    #[test]
    fn wander_rest_is_ten_ticks() {
        let mut world = World::generate(42, Dims::DEFAULT);

        world.step();
        assert_eq!(world.dwarves()[0].2, JobState::Walk);
        for _ in 0..10 {
            world.step();
            assert_eq!(world.dwarves()[0].2, JobState::Idle);
        }
        world.step();
        assert_eq!(world.dwarves()[0].2, JobState::Walk);
    }
}
