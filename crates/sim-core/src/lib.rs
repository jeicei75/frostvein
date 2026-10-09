#![forbid(unsafe_code)]

mod save;
mod worldgen;

pub use save::{SaveState, SavedDwarf};

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
};

use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    resource::Resource,
    schedule::{IntoScheduleConfigs, Schedule},
    system::{Commands, Query, Res, ResMut},
    world::World as EcsWorld,
};
use rand::{RngExt, SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub const DEFAULT_SEED: u64 = 0xF005_7E1A;
const STREAM_WORLDGEN: u64 = 0x4652_4f53_5456_4549;
const STREAM_SPAWN: u64 = 0x5350_4157_4e5f_5f5f;
const STREAM_WANDER: u64 = 0x5741_4e44_4552_5f5f;
const STREAM_TREES: u64 = 0x5452_4545_535f_5f5f;
const STREAM_IDENTITY: u64 = 0x4944_454e_5449_5459; // "IDENTITY"
const STREAM_PROFESSION: u64 = 0x5052_4f46_4553_534e; // "PROFESSN"
const WANDER_RADIUS: i32 = 3;
const WANDER_REST_TICKS: u32 = 10;
/// Ticks a dwarf rests between steps while WORKING, so a job-walk is paced like a wander.
///
/// Without it a pathing dwarf took one cell per tick. A cell is 1.6 m and a tick is 100 ms, so
/// he crossed the valley at 16 m/s -- for a 1.2 m figure, about 23 m/s at human scale. Nothing
/// downstream could survive that: the walk cycle it drives is 0.4926 m per stride, which is 32.5
/// gait cycles a second, under two frames per cycle at 60 fps. The legs aliased into a blur and
/// the figure read as sliding.
const STEP_REST_TICKS: u32 = WANDER_REST_TICKS;
pub const MAX_DESIGNATIONS: usize = 4096;
const MAX_ASTAR_NODES: usize = 50_000;
/// Ticks of work a hauler spends picking a stone up, and again putting it down.
pub const WORK_TICKS: u32 = 5;
/// Ticks of work one dig or channel takes: 10 swings of the pick at the client's 5 ticks per swing
/// (Wolf at the 12.5 seat, 2026-10-03: "maybe it could dig longer one cell.. now it's just one
/// hit", then "10 swings"). 5 s at Normal. Hauls keep `WORK_TICKS`.
pub const DIG_WORK_TICKS: u32 = 50;
// NOTE: tuned with 12.8's clip.
pub const CUT_WORK_TICKS: u32 = 50;
const RETRY_COOLDOWN: u64 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Material {
    Stone,
    Soil,
    Ice,
    Snow,
    TreeTrunk,
    TreeFoliage,
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
    Cut,
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
    SetProfession { dwarf: Id, profession: Profession },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    PlaceStockpile { rect: Rect },
    SetProfession { dwarf: Id },
    Designate { kind: DesignationKind, rect: Rect },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ItemKind {
    Stone,
    Wood,
}

#[derive(Component)]
struct Item(ItemKind);

#[derive(Component)]
struct Emitter(LightKind);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightKind {
    Torch,
    Campfire,
    Lantern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DwarfName {
    Durin,
    Dvalin,
    Nori,
    Ori,
    Dori,
    Bifur,
    Bofur,
    Gloin,
    Nain,
    Thrain,
    Frar,
    Loni,
    Regin,
    Alf,
    Fjalar,
    Frosti,
}

impl DwarfName {
    pub const ALL: [Self; 16] = [
        Self::Durin,
        Self::Dvalin,
        Self::Nori,
        Self::Ori,
        Self::Dori,
        Self::Bifur,
        Self::Bofur,
        Self::Gloin,
        Self::Nain,
        Self::Thrain,
        Self::Frar,
        Self::Loni,
        Self::Regin,
        Self::Alf,
        Self::Fjalar,
        Self::Frosti,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DwarfColour {
    Red,
    Gold,
    Green,
    Blue,
    Purple,
}

impl DwarfColour {
    pub const ALL: [Self; 5] = [Self::Red, Self::Gold, Self::Green, Self::Blue, Self::Purple];
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub name: DwarfName,
    pub colour: DwarfColour,
}

/// A dwarf's trade. Its own component, not part of `Identity`, because 12.6 makes it mutable.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Profession {
    Miner,
    Hauler,
    Woodcutter,
}

/// Every dwarf carries the same permanent lantern.
// NOTE: this becomes saved per-dwarf state only when a future story lets a lantern be dropped.
pub const DWARF_LIGHT: LightKind = LightKind::Lantern;

#[derive(Resource)]
struct Camp(Pos);

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
    Cut,
    // NOTE: `item` is the identity. `Job.target` for a Haul is only the stone's position at
    // creation, kept so load validation can bounds-check every job the same way. Claiming and
    // execution read the stone's live `Pos` — never `target`, which is stale the moment the
    // stone is picked up.
    Haul { item: u32 },
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

/// Present on every dwarf from spawn, exactly like `CurrentJob`, and `Option` rather than an
/// optional component: `to_save`'s `filter_map` silently skips a dwarf missing any component it
/// reads, so an optional `Carrying` would drop every non-carrying dwarf from the save with
/// nothing failing. A query asking for `&Carrying` would skip them too.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct Carrying(Option<u32>);

#[derive(Component)]
struct Path(Vec<Pos>);

#[derive(Component)]
struct WorkProgress(u32);

/// The map and both uniqueness indexes move together through `insert` and `remove`.
/// Tile jobs are unique by `target`; haul jobs are unique by the stone they name, because a
/// stone may well sit on the tile a dig was designated for.
#[derive(Resource, Default)]
struct Jobs {
    by_id: BTreeMap<JobId, Job>,
    targets: BTreeSet<Pos>,
    haul_items: BTreeSet<u32>,
    next_id: u32,
}

impl Jobs {
    fn insert(&mut self, job: Job) -> bool {
        if self.by_id.contains_key(&job.id) {
            return false;
        }
        match job.kind {
            JobKind::Dig | JobKind::Channel | JobKind::Cut => {
                if self.targets.contains(&job.target) {
                    return false;
                }
                self.targets.insert(job.target);
            }
            JobKind::Haul { item } => {
                if self.haul_items.contains(&item) {
                    return false;
                }
                self.haul_items.insert(item);
            }
        }
        self.by_id.insert(job.id, job);
        true
    }

    fn remove(&mut self, id: JobId) -> Option<Job> {
        let job = self.by_id.remove(&id)?;
        match job.kind {
            JobKind::Dig | JobKind::Channel | JobKind::Cut => {
                self.targets.remove(&job.target);
            }
            JobKind::Haul { item } => {
                self.haul_items.remove(&item);
            }
        }
        Some(job)
    }

    /// The one job-id allocator (AD-9), shared by both creation systems. Saturating rather
    /// than wrapping: `insert` only rejects ids of *live* jobs, so a wrapped id could silently
    /// reuse a long-completed one.
    fn next_job_id(&mut self) -> JobId {
        let id = JobId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        id
    }

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
        let id = jobs.next_job_id();
        let kind = match designation {
            DesignationKind::Dig => JobKind::Dig,
            DesignationKind::Channel => JobKind::Channel,
            DesignationKind::Cut => JobKind::Cut,
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

/// Exclusive, because retiring a claimed job calls `release_claim`. It sits inside the chained
/// schedule, so a paused daemon takes new stockpiles and designations but derives no work from
/// them — 3.2's line, extended to stones unchanged.
fn create_haul_jobs(ecs: &mut EcsWorld) {
    let (stored, loose, any_zone) = {
        let zones = &ecs.resource::<Zones>().0;
        // AC3, as amended at 3.3's review (Wolf's call): a stone is STORED iff it is not carried,
        // stands on a stockpile tile, AND is the LOWEST-ID uncarried stone on that tile. Everything
        // else uncarried is LOOSE.
        //
        // The lowest-id clause still recognizes extra stones in an older save as loose. New
        // retry drops avoid making a stack; the pick-up leg waits for a free stockpile tile.
        // `uncarried_stones` is ascending by item id, so "first seen per tile" IS "lowest id".
        let mut occupied: BTreeSet<Pos> = BTreeSet::new();
        let mut stored: BTreeSet<u32> = BTreeSet::new();
        let mut loose = Vec::new();
        for (id, pos) in uncarried_stones(ecs) {
            if zones.contains(&pos) && occupied.insert(pos) {
                stored.insert(id);
            } else {
                loose.push((id, pos));
            }
        }
        (stored, loose, !zones.is_empty())
    };

    // Retire first. The only way a stored stone still has a job is a stockpile placed over a
    // loose stone while a dwarf walks to it — so its holder is by definition not yet carrying
    // and nothing is dropped, but it must still be released rather than left holding a ghost.
    let retired: Vec<_> = ecs
        .resource::<Jobs>()
        .iter()
        .filter(|job| matches!(job.kind, JobKind::Haul { item } if stored.contains(&item)))
        .map(|job| job.id)
        .collect();
    for job_id in retired {
        ecs.resource_mut::<Jobs>().remove(job_id);
        let holders: Vec<_> = ecs
            .iter_entities()
            .filter(|entity| {
                entity.get::<CurrentJob>().and_then(|current| current.0) == Some(job_id)
            })
            .map(|entity| entity.id())
            .collect();
        for holder in holders {
            release_claim(ecs, holder);
        }
    }

    if !any_zone {
        return;
    }
    let tick = ecs.resource::<Tick>().0;
    for (item, pos) in loose {
        if ecs.resource::<Jobs>().haul_items.contains(&item) {
            continue;
        }
        let id = ecs.resource_mut::<Jobs>().next_job_id();
        let inserted = ecs.resource_mut::<Jobs>().insert(Job {
            id,
            kind: JobKind::Haul { item },
            target: pos,
            created_tick: tick,
            retry_after: 0,
        });
        debug_assert!(inserted, "item and id were checked before insertion");
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

/// The trade a job needs. Exhaustive on purpose: a new `JobKind` must name its trade.
fn trade(kind: JobKind) -> Profession {
    match kind {
        JobKind::Dig | JobKind::Channel => Profession::Miner,
        JobKind::Cut => Profession::Woodcutter,
        JobKind::Haul { .. } => Profession::Hauler,
    }
}

// AD-12: one claiming system. It filters by trade (a dwarf is considered only for jobs whose
// `trade` is his profession), then FIFO by `JobId`, ascending dwarf `Id`, reaction delay and
// `retry_after`, and spends one node budget per dwarf (#159).
#[allow(clippy::too_many_arguments)]
fn claim_jobs(
    mut commands: Commands,
    seed: Res<Seed>,
    tick: Res<Tick>,
    terrain: Res<Terrain>,
    zones: Res<Zones>,
    mut jobs: ResMut<Jobs>,
    stones: Query<(&Id, &Pos), With<Item>>,
    emitters: Query<&Pos, With<Emitter>>,
    mut dwarves: Query<(Entity, &Id, &Pos, &mut CurrentJob, &Carrying, &Profession)>,
) {
    let mut dwarves: Vec<_> = dwarves.iter_mut().collect();
    dwarves.sort_by_key(|(_, id, _, _, _, _)| **id);
    let mut claimed: BTreeSet<_> = dwarves
        .iter()
        .filter_map(|(_, _, _, current, _, _)| current.0)
        .collect();
    let carried: BTreeSet<u32> = dwarves
        .iter()
        .filter_map(|(_, _, _, _, carrying, _)| carrying.0)
        .collect();
    let items: BTreeMap<u32, Pos> = stones
        .iter()
        .filter(|(id, _)| !carried.contains(&id.0))
        .map(|(id, pos)| (id.0, *pos))
        .collect();
    // Emitters plus every uncarried item (#162): items block claim-time reachability like fires.
    let blocked = blocked_cells(emitters.iter().chain(items.values()));
    // One budget per dwarf, indexed like the sorted `dwarves`.
    let mut budgets = vec![MAX_ASTAR_NODES; dwarves.len()];
    // Walkable components that a COMPLETED failed search has already flooded this call (#132).
    // Per call only: terrain can change between ticks, so nothing is cached across them.
    // NOTE: the per-tick bound is (idle dwarves) x `MAX_ASTAR_NODES`. A dwarf whose search runs
    // out of his budget sits out the rest of the tick. Still open on #159: one walkable area over
    // the budget with more than `RETRY_COOLDOWN` unreachable jobs ahead of a reachable one, since
    // that dwarf exhausts on one job per tick and never reaches it.
    let mut components: Vec<BTreeSet<Pos>> = Vec::new();

    let jobs_in_order: Vec<_> = jobs.iter().copied().collect();
    for job in jobs_in_order {
        if claimed.contains(&job.id) || tick.0 < job.retry_after {
            continue;
        }
        // A claimable dwarf holds no job, and by AC10 therefore carries nothing — so one goal
        // set serves every candidate.
        let goals = work_positions(&terrain, &blocked, &zones.0, &items, job, None);
        // A haul's delivery leg: the stone must also have somewhere reachable to go (#132).
        let delivery = match job.kind {
            JobKind::Haul { item } => Some(work_positions(
                &terrain,
                &blocked,
                &zones.0,
                &items,
                job,
                Some(item),
            )),
            _ => None,
        };
        let mut attempted = false;
        let mut assigned = false;
        // Some eligible dwarf had no budget left, so this job was not really tried by him.
        let mut sat_out = false;
        for (slot, (entity, id, pos, current, carrying, profession)) in
            dwarves.iter_mut().enumerate()
        {
            // NOTE: a trade with no dwarf leaves its jobs unclaimed and silent, by Wolf's ruling
            // (12.6 Task 0.2); the roster is the player's signal.
            if trade(job.kind) != **profession {
                // Before `attempted`: a dwarf of another trade must not stamp a cooldown on this job.
                continue;
            }
            debug_assert!(
                current.0.is_some() || carrying.0.is_none(),
                "a dwarf holding no job must be carrying nothing"
            );
            if current.0.is_none()
                && tick.0
                    >= job
                        .created_tick
                        .saturating_add(reaction_delay(seed.0, **id, job.id))
            {
                if budgets[slot] == 0 {
                    sat_out = true;
                    continue;
                }
                attempted = true;
                // A component belongs to its start: it proves nothing for a dwarf outside it.
                if components.iter().any(|component| {
                    component.contains(*pos)
                        && (goals.is_disjoint(component)
                            || delivery.as_ref().is_some_and(|d| d.is_disjoint(component)))
                }) {
                    continue;
                }
                if let Some(delivery) = &delivery {
                    match astar_with_budget(&terrain, &blocked, **pos, delivery, &mut budgets[slot])
                    {
                        (Some(_), false, _) => {}
                        (None, false, explored) => {
                            components.push(explored);
                            continue;
                        }
                        (None, true, _) => continue,
                        (Some(_), true, _) => {
                            unreachable!("a completed search cannot exhaust its budget")
                        }
                    }
                }
                let path = match astar_with_budget(
                    &terrain,
                    &blocked,
                    **pos,
                    &goals,
                    &mut budgets[slot],
                ) {
                    (Some(path), false, _) => path,
                    (None, false, explored) => {
                        components.push(explored);
                        continue;
                    }
                    (None, true, _) => continue,
                    (Some(_), true, _) => {
                        unreachable!("a completed search cannot exhaust its budget")
                    }
                };
                current.0 = Some(job.id);
                commands
                    .entity(*entity)
                    .insert((Path(path), WorkProgress(0)));
                claimed.insert(job.id);
                assigned = true;
                break;
            }
        }
        if attempted && !assigned && !sat_out {
            jobs.get_mut(job.id)
                .expect("iterated job still exists")
                .retry_after = tick.0.saturating_add(RETRY_COOLDOWN);
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

/// Can a dwarf stand here — the ONE rule both movement paths ask.
///
/// Standable terrain, and not a cell a fire occupies. Both halves matter and they are deliberately
/// in one function: job routing (`astar_neighbours`) and idle movement (`wander`) previously each
/// carried their own `is_standable` check, and a rule added to one of them would have been a rule
/// the other quietly ignored. A dwarf that cannot be routed through a fire but can wander into one
/// is not fixed. See issue #74.
fn is_walkable(terrain: &Terrain, blocked: &BTreeSet<Pos>, candidate: Pos) -> bool {
    terrain.is_standable(candidate) && !blocked.contains(&candidate)
}

/// The cells a dwarf must not enter: every emitter's own tile and every UNCARRIED item's tile.
///
/// Items (#162, 12.9): a stone or log lying on a tile blocks it for walking, like a fire does. A
/// carried item occupies no tile, so callers pass only `uncarried_stones`. Dwarves are NOT here:
/// they move within a system call (see `dwarf_tiles`).
///
/// `spawn_dwarves` already refuses to PLACE a dwarf on one; nothing stopped a dwarf walking through
/// afterwards, which is what Wolf saw from the seat — dwarves crossing the campfire, their shadows
/// swinging through a large angle as they passed the point light's own position.
///
/// NOTE: emitters are static after worldgen (`spawn_emitters` is the only producer), but this is
/// derived from the live query rather than cached in a resource, so an emitter added later is
/// blocked without anyone remembering to invalidate anything.
fn blocked_cells<'a>(positions: impl Iterator<Item = &'a Pos>) -> BTreeSet<Pos> {
    positions.copied().collect()
}

/// `blocked_cells` read from the live world, for the callers that hold no per-call set of their own.
fn world_blocked(ecs: &mut EcsWorld) -> BTreeSet<Pos> {
    let emitters: Vec<Pos> = ecs
        .query_filtered::<&Pos, With<Emitter>>()
        .iter(ecs)
        .copied()
        .collect();
    let stones = uncarried_stones(ecs);
    blocked_cells(emitters.iter().chain(stones.values()))
}

fn side_neighbours(p: Pos) -> [Pos; 4] {
    [(-1, 0), (1, 0), (0, -1), (0, 1)].map(|(dx, dy)| Pos {
        x: p.x + dx,
        y: p.y + dy,
        z: p.z,
    })
}

/// Every free stockpile cell and how deep it lies: the breadth-first layer from the walkable
/// non-pile cells around the zone (a cell touching one is depth 1). Deep cells fill first, so a
/// pile fills from the inside out. The layers run through FREE cells only (#184): a free cell
/// walled in by taken ones (an item lies on them) is no target, so it cannot hold the deepest
/// layer and stop every delivery to the cells a hauler can still reach.
///
/// NOTE: a pile cell this never reaches (no walkable non-pile ground around its part of the zone)
/// is not a target; such a zone takes no deliveries. Cells are joined by same-z 4-neighbours only.
fn pile_targets(
    terrain: &Terrain,
    blocked: &BTreeSet<Pos>,
    zones: &BTreeSet<Pos>,
) -> BTreeMap<Pos, u32> {
    let mut depth: BTreeMap<Pos, u32> = BTreeMap::new();
    let mut queue = VecDeque::new();
    for &cell in zones {
        if is_walkable(terrain, blocked, cell)
            && side_neighbours(cell)
                .iter()
                .any(|n| !zones.contains(n) && is_walkable(terrain, blocked, *n))
        {
            depth.insert(cell, 1);
            queue.push_back(cell);
        }
    }
    while let Some(cell) = queue.pop_front() {
        let deeper = depth[&cell] + 1;
        for n in side_neighbours(cell) {
            if zones.contains(&n) && is_walkable(terrain, blocked, n) && !depth.contains_key(&n) {
                depth.insert(n, deeper);
                queue.push_back(n);
            }
        }
    }
    depth
}

/// The ONE rule for where a delivery lands: the deepest free pile cell 4-adjacent to `from`,
/// lowest `Pos` on a tie. `work_positions` sends a hauler to the neighbours of the deepest free
/// cells and the drop reads this, so they cannot disagree. A cell `refused` takes (a dwarf stands
/// on it, or filling it walls a dwarf in; #182) is skipped, and the next deepest one is used.
fn drop_cell(
    targets: &BTreeMap<Pos, u32>,
    from: Pos,
    refused: impl Fn(Pos) -> bool,
) -> Option<Pos> {
    side_neighbours(from)
        .into_iter()
        .filter(|cell| !refused(*cell))
        .filter_map(|cell| Some((*targets.get(&cell)?, Reverse(cell))))
        .max()
        .map(|(_, Reverse(cell))| cell)
}

/// Filling `cell` walls a dwarf in (#182): closing it cuts its walkable neighbours into pieces,
/// and a dwarf (the one filling it included) stands in a piece smaller than the largest. Sealing
/// off a pocket nobody stands in is allowed.
///
/// NOTE: a piece is measured up to `MAX_ASTAR_NODES` tiles, so two pieces past that both read as
/// the world and neither walls anybody in. A dwarf standing ON `cell` (an abnormal drop) is in no
/// piece; it walks into whichever one it steps to.
fn walls_in_a_dwarf(
    terrain: &Terrain,
    blocked: &BTreeSet<Pos>,
    dwarves: &BTreeSet<Pos>,
    cell: Pos,
) -> bool {
    let mut closed = blocked.clone();
    closed.insert(cell);
    let sides = astar_neighbours(terrain, &closed, cell);
    let Some((&anchor, rest)) = sides.split_first() else {
        return false;
    };
    if rest
        .iter()
        .all(|side| route_to_nearest(terrain, &closed, *side, |tile| tile == anchor).is_some())
    {
        return false;
    }
    let mut pieces: Vec<BTreeSet<Pos>> = Vec::new();
    for side in sides {
        if !pieces.iter().any(|piece| piece.contains(&side)) {
            pieces.push(piece_of(terrain, &closed, side));
        }
    }
    let largest = pieces.iter().map(BTreeSet::len).max().unwrap_or(0);
    pieces
        .iter()
        .any(|piece| piece.len() < largest && piece.iter().any(|tile| dwarves.contains(tile)))
}

/// Every tile walkable from `from`, up to `MAX_ASTAR_NODES` of them.
fn piece_of(terrain: &Terrain, closed: &BTreeSet<Pos>, from: Pos) -> BTreeSet<Pos> {
    let mut piece = BTreeSet::from([from]);
    let mut queue = VecDeque::from([from]);
    while let Some(cell) = queue.pop_front() {
        for next in astar_neighbours(terrain, closed, cell) {
            if piece.len() >= MAX_ASTAR_NODES {
                return piece;
            }
            if piece.insert(next) {
                queue.push_back(next);
            }
        }
    }
    piece
}

fn astar_neighbours(terrain: &Terrain, blocked: &BTreeSet<Pos>, from: Pos) -> Vec<Pos> {
    const DIRECTIONS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let mut neighbours = Vec::with_capacity(12);
    for (dx, dy) in DIRECTIONS {
        let candidate = Pos {
            x: from.x + dx,
            y: from.y + dy,
            z: from.z,
        };
        if is_walkable(terrain, blocked, candidate) {
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
            if is_walkable(terrain, blocked, candidate)
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
        .map(|goal| {
            let horizontal = from.x.abs_diff(goal.x) + from.y.abs_diff(goal.y);
            horizontal.max(from.z.abs_diff(goal.z))
        })
        .min()
        .unwrap_or(0)
}

fn astar_with_budget(
    terrain: &Terrain,
    blocked: &BTreeSet<Pos>,
    from: Pos,
    goals: &BTreeSet<Pos>,
    nodes_remaining: &mut usize,
) -> (Option<Vec<Pos>>, bool, BTreeSet<Pos>) {
    if goals.is_empty() {
        return (None, false, BTreeSet::new());
    }
    let mut open = BinaryHeap::from([Reverse((astar_heuristic(from, goals), from))]);
    let mut came_from = BTreeMap::new();
    let mut costs = BTreeMap::from([(from, 0_u32)]);

    while let Some(Reverse((queued_f, current))) = open.pop() {
        let current_cost = costs[&current];
        if queued_f != current_cost + astar_heuristic(current, goals) {
            continue;
        }
        if *nodes_remaining == 0 {
            return (None, true, BTreeSet::new());
        }
        *nodes_remaining -= 1;
        if goals.contains(&current) {
            let mut path = Vec::new();
            let mut cursor = current;
            while cursor != from {
                path.push(cursor);
                cursor = came_from[&cursor];
            }
            path.reverse();
            return (Some(path), false, BTreeSet::new());
        }

        for neighbour in astar_neighbours(terrain, blocked, current) {
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
    // A completed failure has flooded `from`'s whole walkable component: hand it back.
    (None, false, costs.into_keys().collect())
}

fn astar(
    terrain: &Terrain,
    blocked: &BTreeSet<Pos>,
    from: Pos,
    goals: &BTreeSet<Pos>,
) -> Option<Vec<Pos>> {
    let mut nodes_remaining = MAX_ASTAR_NODES;
    astar_with_budget(terrain, blocked, from, goals, &mut nodes_remaining).0
}

/// The tiles live dwarves stand on. FR49: one dwarf per tile. Each writer of a dwarf's `Pos`
/// (`execute_jobs`, `settle`, `wander`) builds this once per system and keeps it current as its
/// own dwarves move, so "is this tile free of other dwarves" is one `contains`. Dwarves are NOT
/// part of `blocked_cells`/`is_walkable`: those are static terrain-and-fire rules that A* and
/// claim-time reachability both read, and a dwarf moves within a system call.
fn dwarf_tiles<'a>(positions: impl Iterator<Item = &'a Pos>) -> BTreeSet<Pos> {
    positions.cloned().collect()
}

/// Breadth-first from `from` (not itself a candidate) over the tiles `astar_neighbours` allows
/// with `blocked` closed, to the nearest tile `accept` takes. Returns the route, `from` excluded.
/// Bounded by `MAX_ASTAR_NODES` expansions, like every other search.
fn route_to_nearest(
    terrain: &Terrain,
    blocked: &BTreeSet<Pos>,
    from: Pos,
    accept: impl Fn(Pos) -> bool,
) -> Option<Vec<Pos>> {
    let mut parents = BTreeMap::new();
    let mut queue = VecDeque::from([from]);
    let mut nodes_remaining = MAX_ASTAR_NODES;
    while let Some(cell) = queue.pop_front() {
        if nodes_remaining == 0 {
            return None;
        }
        nodes_remaining -= 1;
        for next in astar_neighbours(terrain, blocked, cell) {
            if next == from || parents.contains_key(&next) {
                continue;
            }
            parents.insert(next, cell);
            if accept(next) {
                let mut route = vec![next];
                while let Some(&parent) = parents.get(route.last().expect("route is non-empty")) {
                    route.push(parent);
                }
                route.pop(); // `from`: the first cell with no parent
                route.reverse();
                return Some(route);
            }
            queue.push_back(next);
        }
    }
    None
}

/// The one tree rule: a tree is its trunk column plus the `TreeFoliage` in the 3x3 column around
/// it, from the base to one above the top trunk cell. Returns the base (the lowest contiguous
/// trunk cell) and every tile. `None` for anything that is not a tree tile.
// NOTE: relies on `place_trees` keeping trunks at least 3 apart (Chebyshev), so no other tree's
// tile lies in the 3x3 box; crowns can still touch, which is why this is a box and not a flood
// fill. The gui's `incremental_tree_cover` uses the same box. Foliage with no trunk in reach is
// no tree.
fn tree_of(terrain: &Terrain, pos: Pos) -> Option<(Pos, Vec<Pos>)> {
    let is_trunk = |p: Pos| terrain.tile(p) == Some(Tile::Solid(Material::TreeTrunk));
    let column = match terrain.tile(pos)? {
        Tile::Solid(Material::TreeTrunk) => (pos.x, pos.y),
        Tile::Solid(Material::TreeFoliage) => {
            let beside = (-1..=1)
                .flat_map(|dy| (-1..=1).map(move |dx| (pos.x + dx, pos.y + dy)))
                .find(|&(x, y)| is_trunk(Pos { x, y, z: pos.z }));
            beside.or_else(|| {
                let below = Pos {
                    z: pos.z - 1,
                    ..pos
                };
                is_trunk(below).then_some((pos.x, pos.y))
            })?
        }
        _ => return None,
    };
    let (x, y) = column;
    // The trunk cell of this column the search stands on: the one at `pos.z`, else the one under it.
    let mut anchor = pos.z;
    while !is_trunk(Pos { x, y, z: anchor }) {
        anchor -= 1;
    }
    let mut base = anchor;
    while is_trunk(Pos { x, y, z: base - 1 }) {
        base -= 1;
    }
    let mut top = anchor;
    while is_trunk(Pos { x, y, z: top + 1 }) {
        top += 1;
    }
    let mut tiles: Vec<Pos> = (base..=top).map(|z| Pos { x, y, z }).collect();
    for z in base..=top + 1 {
        for fy in y - 1..=y + 1 {
            for fx in x - 1..=x + 1 {
                let p = Pos { x: fx, y: fy, z };
                if terrain.tile(p) == Some(Tile::Solid(Material::TreeFoliage)) {
                    tiles.push(p);
                }
            }
        }
    }
    Some((Pos { x, y, z: base }, tiles))
}

/// `items` holds UNCARRIED stones only — a stone in transit occupies no tile, so a carrier
/// crossing the pile never blocks a tile for anyone else. `blocked` holds those same tiles
/// (`blocked_cells`), so an item is never a work position and never stood on (#162): a dwarf works
/// from the next tile.
fn work_positions(
    terrain: &Terrain,
    blocked: &BTreeSet<Pos>,
    zones: &BTreeSet<Pos>,
    items: &BTreeMap<u32, Pos>,
    job: Job,
    carrying: Option<u32>,
) -> BTreeSet<Pos> {
    match job.kind {
        // NOTE: a one-wide tunnel waits at each stone until it is hauled (the stone blocks the only
        // work position), and with no stockpile it stops there: never-drop (FR8) wins.
        JobKind::Dig | JobKind::Cut => side_neighbours(job.target)
            .into_iter()
            .filter(|candidate| is_walkable(terrain, blocked, *candidate))
            .collect(),
        // A channel works from the next tile too, and its stone spawns on the target. A target that
        // is not standable has no work position and its job retries.
        // NOTE: once the pile is full, a mark whose every neighbour holds an unhauled stone (its
        // neighbours' own stones) has no work position and waits: the ruled #180 / FR8 never-drop
        // shape (12.9 review, #182 mode ii). It cages no dwarf.
        JobKind::Channel => {
            if terrain.is_standable(job.target) {
                side_neighbours(job.target)
                    .into_iter()
                    .filter(|candidate| is_walkable(terrain, blocked, *candidate))
                    .collect()
            } else {
                BTreeSet::new()
            }
        }
        JobKind::Haul { item } => {
            debug_assert!(
                carrying.is_none_or(|carried| carried == item),
                "a dwarf only ever carries the stone of the haul job it holds"
            );
            // Recompute the free pile cells every tick. If two carriers converge on the last
            // free cell, the second can arrive after it fills; its delivery then finds no cell
            // and `release_claim` drops it off the pile.
            // NOTE: `claim_jobs` checks delivery reachability at claim time, so a sealed cell never
            // starts a pick-up; one sealed off mid-walk can still cost a pick-up/drop per retry (#132).
            let targets = pile_targets(terrain, blocked, zones);
            if carrying.is_some() {
                // The neighbours of the DEEPEST free cells; `drop_cell` then lands on one of them.
                let deepest = targets.values().copied().max();
                return targets
                    .iter()
                    .filter(|(_, depth)| Some(**depth) == deepest)
                    .flat_map(|(cell, _)| side_neighbours(*cell))
                    .filter(|candidate| is_walkable(terrain, blocked, *candidate))
                    .collect();
            }
            // Both legs read the same free-cell set. With nowhere to deliver the pick-up leg is
            // empty too, so the job is never claimed rather than claimed into a
            // pick-up-and-drop cycle. The stone's own tile is still standable ground, but it is
            // blocked: the hauler lifts it from a walkable 4-neighbour.
            // NOTE: a stone whose floor was dug away is on no standable tile — items never fall —
            // so its job retries forever. Retry is nearly free and never-drop wins (FR8).
            match items.get(&item) {
                Some(pos) if !targets.is_empty() && terrain.is_standable(*pos) => {
                    side_neighbours(*pos)
                        .into_iter()
                        .filter(|candidate| is_walkable(terrain, blocked, *candidate))
                        .collect()
                }
                _ => BTreeSet::new(),
            }
        }
    }
}

/// Stone positions by id, ascending, EXCLUDING stones in transit: a carried stone occupies no
/// tile. Recomputed rather than cached because a pick-up or a drop changes it mid-system.
fn uncarried_stones(ecs: &EcsWorld) -> BTreeMap<u32, Pos> {
    let carried: BTreeSet<u32> = ecs
        .iter_entities()
        .filter(|entity| entity.contains::<Dwarf>())
        .filter_map(|entity| entity.get::<Carrying>()?.0)
        .collect();
    ecs.iter_entities()
        .filter(|entity| entity.contains::<Item>())
        .filter_map(|entity| {
            let id = entity.get::<Id>()?.0;
            (!carried.contains(&id)).then_some((id, *entity.get::<Pos>()?))
        })
        .collect()
}

/// Stones are few and never despawn, so a scan beats a reverse index that has to stay in sync.
fn item_entity(ecs: &EcsWorld, item: u32) -> Option<Entity> {
    ecs.iter_entities()
        .find(|entity| entity.contains::<Item>() && entity.get::<Id>() == Some(&Id(item)))
        .map(|entity| entity.id())
}

fn release_claim(ecs: &mut EcsWorld, entity: Entity) {
    // A dwarf that stops holding a job stops carrying its stone, and drops it where it stands
    // unless that would stack stones on a stockpile cell or wall a dwarf in (#182); then on the
    // nearest tile it can walk to that does neither and holds no other dwarf. A normal delivery
    // never gets here with the stone still in hand: `execute_jobs` lands it on the pile cell
    // beside the hauler first, or lets go when every one is refused.
    // NOTE: this abnormal-exit drop is at the dwarf's own tile, so the dwarf stands in the item
    // until it walks off. That is rare (a retry, a cancel, a vanished job, a refused delivery).
    // Doing it here is what keeps every abnormal exit — a vanished job, a retry, a cancel, a
    // retire — from welding a stone to an idle dwarf.
    if let Some(item) = ecs.get::<Carrying>(entity).and_then(|carrying| carrying.0) {
        let dropped_at = ecs.get::<Pos>(entity).copied();
        if let (Some(pos), Some(stone)) = (dropped_at, item_entity(ecs, item)) {
            let zones = &ecs.resource::<Zones>().0;
            let occupied: BTreeSet<Pos> = uncarried_stones(ecs)
                .values()
                .copied()
                .filter(|cell| zones.contains(cell))
                .collect();
            let blocked = world_blocked(ecs);
            let dwarves = dwarf_tiles(ecs.query_filtered::<&Pos, With<Dwarf>>().iter(ecs));
            let terrain = ecs.resource::<Terrain>();
            let refused = |cell: Pos| {
                occupied.contains(&cell)
                    || (cell != pos && dwarves.contains(&cell))
                    || walls_in_a_dwarf(terrain, &blocked, &dwarves, cell)
            };
            let drop_pos = if refused(pos) {
                // Breadth-first over the tiles the carrier can walk to, so the drop is never
                // behind a wall or on a level it cannot reach.
                let mut seen = BTreeSet::from([pos]);
                let mut frontier = BTreeSet::from([pos]);
                let mut nearest = None;
                while !frontier.is_empty() && nearest.is_none() {
                    let mut next = BTreeSet::new();
                    for cell in frontier {
                        if !refused(cell) {
                            nearest = Some(cell);
                            break;
                        }
                        for candidate in astar_neighbours(terrain, &blocked, cell) {
                            if seen.insert(candidate) {
                                next.insert(candidate);
                            }
                        }
                    }
                    frontier = next;
                }
                // NOTE: a carrier whose whole reachable ground is refused keeps the stone on its
                // own tile rather than stop the sim.
                nearest.unwrap_or(pos)
            } else {
                pos
            };
            *ecs.get_mut::<Pos>(stone)
                .expect("every stone has a position") = drop_pos;
        }
        if let Some(mut carrying) = ecs.get_mut::<Carrying>(entity) {
            carrying.0 = None;
        }
    }
    if let Some(mut current) = ecs.get_mut::<CurrentJob>(entity) {
        current.0 = None;
    }
    if let Some(mut state) = ecs.get_mut::<JobState>(entity) {
        *state = JobState::Idle;
    }
    // The dwarf now lives where it finished. `wander` only accepts tiles within
    // WANDER_RADIUS of `home` and A* has no such limit, so without this a dwarf that walked
    // to a distant job is motionless FOREVER: from 5+ tiles out every neighbour is still
    // outside the radius, so the candidate set is empty on every future tick. Distance 4 is
    // the boundary — from there one step inward reaches 3 and it recovers on its own, which
    // is why only genuinely distant jobs strand a dwarf.
    //
    // This is the ONE place that has to do it: every path where a dwarf stops holding a job
    // funnels here — completion, a no-op completion, a vanished job, a retry, and cancel.
    // NOTE: a dwarf therefore never returns to its spawn; it settles wherever work took it.
    let released_at = ecs.get::<Pos>(entity).copied();
    if let (Some(pos), Some(mut wander)) = (released_at, ecs.get_mut::<Wander>(entity)) {
        wander.home = pos;
    }
    let mut entity = ecs.entity_mut(entity);
    entity.remove::<Path>();
    entity.remove::<WorkProgress>();
}

fn retry_claim(ecs: &mut EcsWorld, entity: Entity, job_id: JobId) {
    let retry_after = ecs.resource::<Tick>().0.saturating_add(RETRY_COOLDOWN);
    if let Some(job) = ecs.resource_mut::<Jobs>().get_mut(job_id) {
        job.retry_after = retry_after;
    }
    release_claim(ecs, entity);
}

fn clear_paths(ecs: &mut EcsWorld) {
    let entities: Vec<_> = ecs
        .iter_entities()
        .filter(|entity| entity.contains::<Path>())
        .map(|entity| entity.id())
        .collect();
    for entity in entities {
        ecs.entity_mut(entity).remove::<Path>();
    }
}

/// Moves the dwarf on `from` one tile, to `aside`. It calls `aside` home and rests a step period, so
/// `wander` neither drags it back nor moves it a second cell this tick, and any `Path` is dropped.
fn step_aside(
    ecs: &mut EcsWorld,
    occupied: &mut BTreeSet<Pos>,
    dwarf: Entity,
    from: Pos,
    aside: Pos,
) {
    *ecs.get_mut::<Pos>(dwarf)
        .expect("every dwarf has a position") = aside;
    let mut wander = ecs.get_mut::<Wander>(dwarf).expect("every dwarf wanders");
    wander.home = aside;
    wander.cooldown = STEP_REST_TICKS;
    ecs.entity_mut(dwarf).remove::<Path>();
    occupied.remove(&from);
    occupied.insert(aside);
}

/// The static blocked set (fires and items) plus every dwarf tile except `except`: what a search treats as closed when the
/// dwarves in `except` are the ones it is planning for.
fn closed_to_dwarves(
    blocked: &BTreeSet<Pos>,
    occupied: &BTreeSet<Pos>,
    except: &[Pos],
) -> BTreeSet<Pos> {
    let mut closed = blocked.clone();
    closed.extend(occupied.iter().filter(|tile| !except.contains(tile)));
    closed
}

/// The next tile of a holder's `path` holds another dwarf (FR49). Returns `true` when `path` now
/// starts on a free tile and the holder should step, `false` when it waits. May move the blocker
/// (it steps aside, or yields) or replace `path` (a re-route, or the holder yields).
///
/// 1. The blocker is idle: it steps to a free neighbour off the holder's path. With none, it gets
///    an exit `Path` to the nearest free tile off the holder's path, which starts on the holder's
///    tile, so this becomes case 3.
/// 2. The blocker holds a job and is not coming the other way: re-route around every dwarf.
/// 3. Head-on (the blocker's path starts on the holder's tile): each side's ESCAPE is the nearest
///    tile, reached without passing the other dwarf, that is off the other's remaining path. The
///    lower id yields if it has one, else the other does. The yield IS the yielder's `Path`.
///    A per-step "back off if you can" rule livelocks in a dead end (see the story); an escape
///    gets nearer as the yielder retreats and never appears for the dwarf that advances.
/// 4. Otherwise wait.
///
/// NOTE: no swap anywhere (Wolf, Task 0 Q1b). Two dwarves sealed in one pocket both wait, and a
/// ring of dwarves each waiting on the next waits too; neither is detected.
fn resolve_blocked_step(
    ecs: &mut EcsWorld,
    blocked: &BTreeSet<Pos>,
    occupied: &mut BTreeSet<Pos>,
    (holder_id, pos): (Id, Pos),
    path: &mut Vec<Pos>,
    goals: &BTreeSet<Pos>,
) -> bool {
    let next = path[0];
    let (blocker_id, blocker) = ecs
        .iter_entities()
        .filter(|entity| entity.contains::<Dwarf>())
        .find(|entity| entity.get::<Pos>() == Some(&next))
        .map(|entity| {
            (
                *entity.get::<Id>().expect("every dwarf has an id"),
                entity.id(),
            )
        })
        .expect("an occupied tile has a dwarf on it");
    let idle = ecs
        .get::<CurrentJob>(blocker)
        .is_some_and(|current| current.0.is_none());
    let mut blocker_path: Vec<Pos> = ecs
        .get::<Path>(blocker)
        .map(|path| path.0.clone())
        .unwrap_or_default();
    let holder_tiles: BTreeSet<Pos> = path.iter().copied().collect();
    let head_on = |blocker_path: &[Pos]| blocker_path.first() == Some(&pos);

    if idle && !head_on(&blocker_path) {
        let terrain = ecs.resource::<Terrain>();
        let aside = astar_neighbours(terrain, blocked, next)
            .into_iter()
            .find(|tile| !occupied.contains(tile) && !holder_tiles.contains(tile));
        if let Some(aside) = aside {
            step_aside(ecs, occupied, blocker, next, aside);
            return true;
        }
        // No way aside: leave by the nearest free tile off the holder's path. The holder's own
        // tile is passable for the search (it is the way out) but not a destination.
        let closed = closed_to_dwarves(blocked, occupied, &[pos, next]);
        let Some(route) = route_to_nearest(terrain, &closed, next, |tile| {
            tile != pos && !holder_tiles.contains(&tile)
        }) else {
            return false;
        };
        blocker_path = route.clone();
        ecs.entity_mut(blocker).insert(Path(route));
    }
    if !head_on(&blocker_path) {
        if idle {
            return false;
        }
        let closed = closed_to_dwarves(blocked, occupied, &[pos]);
        let Some(rerouted) = astar(ecs.resource::<Terrain>(), &closed, pos, goals) else {
            return false;
        };
        *path = rerouted;
        return true;
    }

    let blocker_tiles: BTreeSet<Pos> = blocker_path.iter().copied().collect();
    let (holder_escape, blocker_escape) = {
        let terrain = ecs.resource::<Terrain>();
        (
            route_to_nearest(
                terrain,
                &closed_to_dwarves(blocked, occupied, &[pos]),
                pos,
                |tile| !blocker_tiles.contains(&tile),
            ),
            route_to_nearest(
                terrain,
                &closed_to_dwarves(blocked, occupied, &[next]),
                next,
                |tile| !holder_tiles.contains(&tile),
            ),
        )
    };
    let holder_yields = match (&holder_escape, &blocker_escape) {
        (Some(_), Some(_)) => holder_id < blocker_id,
        (Some(_), None) => true,
        (None, Some(_)) => false,
        // Both wait. An idle blocker's exit `Path` is dropped (#183): it was derived when the
        // holder stood elsewhere and can lead back through the holder, so keeping it would hold
        // the idle dwarf still forever. The next blocked step derives a fresh one.
        (None, None) => {
            if idle {
                ecs.entity_mut(blocker).remove::<Path>();
            }
            return false;
        }
    };
    if holder_yields {
        *path = holder_escape.expect("checked above");
        return true;
    }
    let route = blocker_escape.expect("checked above");
    let (step, rest) = route.split_first().expect("an escape route is not empty");
    *ecs.get_mut::<Pos>(blocker)
        .expect("every dwarf has a position") = *step;
    occupied.remove(&next);
    occupied.insert(*step);
    let mut wander = ecs.get_mut::<Wander>(blocker).expect("every dwarf wanders");
    wander.cooldown = STEP_REST_TICKS;
    if rest.is_empty() && idle {
        wander.home = *step;
    }
    *ecs.get_mut::<JobState>(blocker)
        .expect("every dwarf has a job state") = JobState::Walk;
    if rest.is_empty() {
        ecs.entity_mut(blocker).remove::<Path>();
    } else {
        ecs.entity_mut(blocker).insert(Path(rest.to_vec()));
    }
    true
}

/// Exclusive so terrain mutation and stone spawning are visible in the same tick.
fn execute_jobs(ecs: &mut EcsWorld) {
    // Built once, before the loop borrows the world for terrain. Emitters are static; the items
    // are not (a pick-up, a drop or a spawn changes them mid-loop), so `blocked` is rebuilt from
    // them for every dwarf.
    let emitters: Vec<Pos> = ecs
        .query_filtered::<&Pos, With<Emitter>>()
        .iter(ecs)
        .copied()
        .collect();
    let mut occupied = dwarf_tiles(ecs.query_filtered::<&Pos, With<Dwarf>>().iter(ecs));
    let mut dwarves: Vec<_> = ecs
        .iter_entities()
        .filter(|entity| entity.contains::<Dwarf>())
        .filter_map(|entity| Some((*entity.get::<Id>()?, entity.id())))
        .collect();
    dwarves.sort_by_key(|(id, _)| *id);

    for (holder_id, entity) in dwarves {
        let Some(job_id) = ecs.get::<CurrentJob>(entity).and_then(|current| current.0) else {
            continue;
        };
        let Some(job) = ecs.resource::<Jobs>().by_id.get(&job_id).copied() else {
            release_claim(ecs, entity);
            continue;
        };
        let pos = *ecs.get::<Pos>(entity).expect("every dwarf has a position");
        if !ecs.resource::<Terrain>().is_standable(pos) {
            *ecs.get_mut::<JobState>(entity)
                .expect("every dwarf has a job state") = JobState::Walk;
            continue;
        }
        let carrying = ecs.get::<Carrying>(entity).and_then(|carrying| carrying.0);
        let stones = uncarried_stones(ecs);
        let blocked = blocked_cells(emitters.iter().chain(stones.values()));
        let work_positions = work_positions(
            ecs.resource::<Terrain>(),
            &blocked,
            &ecs.resource::<Zones>().0,
            &stones,
            job,
            carrying,
        );

        if !work_positions.contains(&pos) {
            let mut path = ecs
                .get::<Path>(entity)
                .map(|path| path.0.clone())
                .unwrap_or_default();
            // A stored path can lead onto an item that landed after it was computed (a delivery
            // changes no terrain, so `clear_paths` never ran): plan again rather than step in.
            if path.is_empty() || blocked.contains(&path[0]) {
                let terrain = ecs.resource::<Terrain>();
                let Some(computed) = astar(terrain, &blocked, pos, &work_positions) else {
                    retry_claim(ecs, entity, job.id);
                    continue;
                };
                path = computed;
            }
            // Pace the step. `wander` skips any dwarf holding a job before it touches this
            // cooldown, so the two never decrement it in the same tick and one field can pace
            // both kinds of walking.
            let resting = ecs
                .get::<Wander>(entity)
                .map(|wander| wander.cooldown)
                .unwrap_or(0);
            if resting > 0 {
                if let Some(mut wander) = ecs.get_mut::<Wander>(entity) {
                    wander.cooldown -= 1;
                }
                *ecs.get_mut::<JobState>(entity)
                    .expect("every dwarf has a job state") = JobState::Walk;
                ecs.entity_mut(entity).insert(Path(path));
                continue;
            }
            if occupied.contains(&path[0])
                && !resolve_blocked_step(
                    ecs,
                    &blocked,
                    &mut occupied,
                    (holder_id, pos),
                    &mut path,
                    &work_positions,
                )
            {
                // Wait one step period and look again, so a blocked dwarf searches once per
                // period and not every tick.
                if let Some(mut wander) = ecs.get_mut::<Wander>(entity) {
                    wander.cooldown = STEP_REST_TICKS;
                }
                *ecs.get_mut::<JobState>(entity)
                    .expect("every dwarf has a job state") = JobState::Walk;
                ecs.entity_mut(entity).insert(Path(path));
                continue;
            }
            let next = path.remove(0);
            *ecs.get_mut::<Pos>(entity)
                .expect("every dwarf has a position") = next;
            occupied.remove(&pos);
            occupied.insert(next);
            if let Some(mut wander) = ecs.get_mut::<Wander>(entity) {
                wander.cooldown = STEP_REST_TICKS;
            }
            *ecs.get_mut::<JobState>(entity)
                .expect("every dwarf has a job state") = JobState::Walk;
            ecs.entity_mut(entity).insert(Path(path));
            continue;
        }

        let progress = ecs
            .get::<WorkProgress>(entity)
            .map(|progress| progress.0)
            .unwrap_or(0);
        let needed = match job.kind {
            JobKind::Dig | JobKind::Channel => DIG_WORK_TICKS,
            JobKind::Cut => CUT_WORK_TICKS,
            JobKind::Haul { .. } => WORK_TICKS,
        };
        if progress < needed {
            *ecs.get_mut::<JobState>(entity)
                .expect("every dwarf has a job state") = JobState::Work;
            ecs.entity_mut(entity).insert(WorkProgress(progress + 1));
            continue;
        }

        // Dispatch on kind BEFORE the terrain change below: a haul mutates no tile, and above
        // all must never reach the no-op-completion arm, which removes the designation at
        // `job.target` — where a real, unrelated order may legitimately sit.
        if let JobKind::Haul { item } = job.kind {
            match carrying {
                // Pick up. The job stays claimed and the same dwarf now walks to the pile, so
                // the work counter restarts and the path to the stone is spent.
                None => {
                    ecs.get_mut::<Carrying>(entity)
                        .expect("every dwarf has a carrying slot")
                        .0 = Some(item);
                    *ecs.get_mut::<JobState>(entity)
                        .expect("every dwarf has a job state") = JobState::Walk;
                    ecs.entity_mut(entity).insert(WorkProgress(0));
                    ecs.entity_mut(entity).remove::<Path>();
                }
                // Deliver. The stone lands on the deepest free pile cell beside the hauler
                // (`drop_cell`, the rule `work_positions` sent it here by); `release_claim` then
                // clears the slot, the same funnel every abnormal exit uses.
                Some(item) => {
                    let terrain = ecs.resource::<Terrain>();
                    let targets = pile_targets(terrain, &blocked, &ecs.resource::<Zones>().0);
                    let landing = drop_cell(&targets, pos, |cell| {
                        occupied.contains(&cell)
                            || walls_in_a_dwarf(terrain, &blocked, &occupied, cell)
                    });
                    // Every pile cell beside the hauler is refused (#182): it lets go and the job
                    // retries after the cooldown, as a refused channel does. Holding deadlocked
                    // when the hauler stood on the occupant's only way out. No claim-time change
                    // (AD-12).
                    if landing.is_none()
                        && side_neighbours(pos)
                            .iter()
                            .any(|cell| targets.contains_key(cell))
                    {
                        retry_claim(ecs, entity, job.id);
                        continue;
                    }
                    // NOTE: `None` means the pile filled under the hauler after the goal set was
                    // computed; the stone then takes `release_claim`'s abnormal-exit drop.
                    if let (Some(landing), Some(stone)) = (landing, item_entity(ecs, item)) {
                        *ecs.get_mut::<Pos>(stone)
                            .expect("every stone has a position") = landing;
                        ecs.get_mut::<Carrying>(entity)
                            .expect("every dwarf has a carrying slot")
                            .0 = None;
                    }
                    ecs.resource_mut::<Jobs>().remove(job.id);
                    release_claim(ecs, entity);
                }
            }
            continue;
        }

        if job.kind == JobKind::Cut {
            let tree = tree_of(ecs.resource::<Terrain>(), job.target);
            if let Some((base, tiles)) = tree {
                let mut trunk_cells = 0;
                for tile in &tiles {
                    let terrain = ecs.resource::<Terrain>();
                    if terrain.tile(*tile) == Some(Tile::Solid(Material::TreeTrunk)) {
                        trunk_cells += 1;
                    }
                    let changed = ecs.resource_mut::<Terrain>().set_tile(*tile, Tile::Empty);
                    debug_assert!(changed, "tree tiles are in bounds");
                }
                clear_paths(ecs);
                // One log per trunk cell, all at the base (Wolf, 12.7 Task 0.2).
                for _ in 0..trunk_cells {
                    let item_id = ecs.resource_mut::<IdAllocator>().allocate();
                    ecs.spawn((Item(ItemKind::Wood), item_id, base));
                }
            }
            ecs.resource_mut::<Jobs>().remove(job.id);
            ecs.resource_mut::<Designations>().0.remove(&job.target);
            release_claim(ecs, entity);
            continue;
        }

        let change = {
            let terrain = ecs.resource::<Terrain>();
            match job.kind {
                // NOTE: designation never marks tree tiles (12.7), so a dig or channel target is
                // never a tree material by the time it completes.
                JobKind::Dig => match terrain.tile(job.target) {
                    Some(Tile::Solid(_)) => Some((job.target, Tile::Empty)),
                    _ => None,
                },
                JobKind::Channel => {
                    let below = Pos {
                        z: job.target.z - 1,
                        ..job.target
                    };
                    match terrain.tile(below) {
                        Some(Tile::Solid(material)) => Some((below, Tile::Ramp(material))),
                        _ => None,
                    }
                }
                JobKind::Cut | JobKind::Haul { .. } => {
                    unreachable!("cut and haul jobs are dispatched above")
                }
            }
        };
        let Some((changed_pos, tile)) = change else {
            ecs.resource_mut::<Jobs>().remove(job.id);
            ecs.resource_mut::<Designations>().0.remove(&job.target);
            release_claim(ecs, entity);
            continue;
        };
        // The stone spawns on the target, and a channel target is open ground a dwarf can stand on
        // (#182). That dwarf first steps aside, as an idle blocker does. With no free tile beside
        // it, or when the stone would wall a dwarf in, the worker lets go and the job retries after
        // the cooldown: holding at Work deadlocked a worker whose only way out was the occupant's.
        // NOTE: the wall-in check reads the terrain before the ramp is cut below the target.
        if occupied.contains(&job.target) {
            let aside = astar_neighbours(ecs.resource::<Terrain>(), &blocked, job.target)
                .into_iter()
                .find(|tile| !occupied.contains(tile));
            let Some(aside) = aside else {
                retry_claim(ecs, entity, job.id);
                continue;
            };
            let occupant = ecs
                .iter_entities()
                .find(|other| other.contains::<Dwarf>() && other.get::<Pos>() == Some(&job.target))
                .map(|other| other.id())
                .expect("an occupied tile has a dwarf on it");
            step_aside(ecs, &mut occupied, occupant, job.target, aside);
        }
        if walls_in_a_dwarf(ecs.resource::<Terrain>(), &blocked, &occupied, job.target) {
            retry_claim(ecs, entity, job.id);
            continue;
        }
        let changed = ecs.resource_mut::<Terrain>().set_tile(changed_pos, tile);
        debug_assert!(
            changed,
            "job targets were bounds-checked at designation time"
        );
        clear_paths(ecs);
        let item_id = ecs.resource_mut::<IdAllocator>().allocate();
        ecs.spawn((Item(ItemKind::Stone), item_id, job.target));
        ecs.resource_mut::<Jobs>().remove(job.id);
        ecs.resource_mut::<Designations>().0.remove(&job.target);
        release_claim(ecs, entity);
    }
}

fn settle(ecs: &mut EcsWorld) {
    // Items never move in this system, so one set serves the whole call.
    let blocked = world_blocked(ecs);
    let mut occupied = dwarf_tiles(ecs.query_filtered::<&Pos, With<Dwarf>>().iter(ecs));
    let mut dwarves: Vec<_> = ecs
        .iter_entities()
        .filter(|entity| entity.contains::<Dwarf>())
        .filter_map(|entity| Some((*entity.get::<Id>()?, entity.id())))
        .collect();
    dwarves.sort_by_key(|(id, _)| *id);

    for (_, entity) in dwarves {
        let pos = *ecs.get::<Pos>(entity).expect("every dwarf has a position");
        let below = Pos {
            z: pos.z - 1,
            ..pos
        };
        let should_settle = {
            let terrain = ecs.resource::<Terrain>();
            !terrain.is_standable(pos) && matches!(terrain.tile(below), Some(Tile::Empty))
        };
        if should_settle {
            // A dwarf already on `below`, or an item lying there, is not landed on: the faller
            // rests on the nearest free walkable tile instead.
            let landing = if occupied.contains(&below) || blocked.contains(&below) {
                route_to_nearest(ecs.resource::<Terrain>(), &blocked, below, |cell| {
                    !occupied.contains(&cell)
                })
                .and_then(|route| route.last().copied())
            } else {
                Some(below)
            };
            // NOTE: with no free tile the faller stays put this tick and tries again; "no dwarf
            // on air" is 12.11's.
            let Some(landing) = landing else { continue };
            *ecs.get_mut::<Pos>(entity)
                .expect("every dwarf has a position") = landing;
            occupied.remove(&pos);
            occupied.insert(landing);
            ecs.entity_mut(entity).remove::<Path>();
        }
    }
    // NOTE: gravity is deliberately limited to one level per dwarf per tick; items never fall.
}

/// Exclusive and LAST in the chain, not merely after `settle`: a carried stone sits on its
/// carrier's tile at the end of every tick no matter which system moved the carrier.
fn carry_items(ecs: &mut EcsWorld) {
    let mut carriers: Vec<_> = ecs
        .iter_entities()
        .filter(|entity| entity.contains::<Dwarf>())
        .filter_map(|entity| {
            Some((
                *entity.get::<Id>()?,
                entity.get::<Carrying>()?.0?,
                *entity.get::<Pos>()?,
            ))
        })
        .collect();
    carriers.sort_by_key(|(id, ..)| *id);

    for (_, item, pos) in carriers {
        if let Some(stone) = item_entity(ecs, item) {
            *ecs.get_mut::<Pos>(stone)
                .expect("every stone has a position") = pos;
        }
    }
}

fn advance_tick(mut tick: ResMut<Tick>) {
    tick.0 += 1;
}

// One query over the whole dwarf row; splitting it only to satisfy the lint would split a row
// that is read together.
#[allow(clippy::type_complexity)]
fn wander(
    mut commands: Commands,
    mut rng: ResMut<WanderRng>,
    terrain: Res<Terrain>,
    emitters: Query<&Pos, With<Emitter>>,
    items: Query<(&Id, &Pos), With<Item>>,
    // `Without<Emitter>` and `Without<Item>` are what make this disjoint from the two queries
    // above: this one takes `&mut Pos` and those take `&Pos`, and bevy_ecs rejects the pair
    // (B0001) unless a filter proves no entity can be in both. Nothing is ever both a dwarf and
    // a fire or a stone, so it costs nothing to say so.
    mut dwarves: Query<
        (
            Entity,
            &Id,
            &mut Pos,
            &mut Wander,
            &mut JobState,
            &CurrentJob,
            Option<&mut Path>,
            &Carrying,
        ),
        (Without<Emitter>, Without<Item>),
    >,
) {
    // AD-7: query iteration is archetype order, not Id order, and all dwarves draw from
    // one stream. Draw order is a sim outcome, so sort before touching the RNG.
    let mut dwarves: Vec<_> = dwarves.iter_mut().collect();
    dwarves.sort_by_key(|(_, id, ..)| **id);
    let mut occupied = dwarf_tiles(dwarves.iter().map(|(_, _, pos, ..)| &**pos));
    // Emitters plus every uncarried item (#162). A carried item's `Pos` is its carrier's tile from
    // before this tick's move, so it is left out; items do not move in this system.
    let carried: BTreeSet<u32> = dwarves
        .iter()
        .filter_map(|(.., carrying)| carrying.0)
        .collect();
    let blocked = blocked_cells(
        emitters.iter().chain(
            items
                .iter()
                .filter(|(id, _)| !carried.contains(&id.0))
                .map(|(_, pos)| pos),
        ),
    );

    for (entity, _, mut pos, mut wander, mut state, current_job, path, _) in dwarves {
        if current_job.0.is_some() {
            continue;
        }
        // NOTE: `settle` handles terrain mutated under a dwarf before wandering runs.
        if wander.cooldown > 0 {
            wander.cooldown -= 1;
            *state = JobState::Idle;
            continue;
        }

        let here = *pos;
        // An idle dwarf with a `Path` is leaving someone's way (`resolve_blocked_step`): it
        // follows the path at the step pace, waits while the next tile is occupied, draws no
        // random number, and on arrival calls the tile it reached home.
        if let Some(mut path) = path {
            *state = JobState::Idle;
            if let Some(&next) = path.0.first() {
                // An item landed on the route after it was planned: give the route up and stay.
                if blocked.contains(&next) {
                    wander.home = here;
                    commands.entity(entity).remove::<Path>();
                    continue;
                }
                if occupied.contains(&next) {
                    continue;
                }
                path.0.remove(0);
                occupied.remove(&here);
                occupied.insert(next);
                *pos = next;
                wander.cooldown = STEP_REST_TICKS;
                *state = JobState::Walk;
            }
            if path.0.is_empty() {
                wander.home = *pos;
                commands.entity(entity).remove::<Path>();
            }
            continue;
        }
        // NOTE: fixed order, same z only. Ramp climbing arrives with A* in Story 3.2.
        let candidates: Vec<Pos> = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .map(|(dx, dy)| Pos {
                x: here.x + dx,
                y: here.y + dy,
                z: here.z,
            })
            // FR49: one dwarf per tile, so a tile another dwarf stands on is not a candidate.
            .filter(|p| {
                (p.x - wander.home.x).abs() <= WANDER_RADIUS
                    && (p.y - wander.home.y).abs() <= WANDER_RADIUS
                    && is_walkable(&terrain, &blocked, *p)
                    && !occupied.contains(p)
            })
            .collect();
        wander.cooldown = WANDER_REST_TICKS;
        match candidates.len() {
            0 => *state = JobState::Idle,
            n => {
                occupied.remove(&here);
                *pos = candidates[rng.0.random_range(0..n)];
                occupied.insert(*pos);
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
    camp_origin: Pos,
    jobs: Jobs,
    designations: BTreeMap<Pos, DesignationKind>,
    zones: BTreeSet<Pos>,
) -> World {
    let mut ecs = EcsWorld::new();
    ecs.insert_resource(Tick(tick));
    ecs.insert_resource(Seed(seed));
    ecs.insert_resource(WanderRng(wander_rng));
    ecs.insert_resource(ids);
    ecs.insert_resource(Camp(camp_origin));
    ecs.insert_resource(Designations(designations));
    ecs.insert_resource(Zones(zones));
    ecs.insert_resource(jobs);
    ecs.insert_resource(Terrain {
        dims,
        tiles,
        dirty: BTreeSet::new(),
    });
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            advance_tick,
            create_jobs,
            create_haul_jobs,
            claim_jobs,
            execute_jobs,
            settle,
            wander,
            carry_items,
        )
            .chain(),
    );
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
        let mut heights = worldgen::height_field(dims, &mut rng);
        worldgen::apply_lake(dims, &mut heights);
        worldgen::apply_ridges(dims, &mut heights);
        let mut tiles = worldgen::layered_terrain(dims, &heights);
        worldgen::place_ramps(dims, &heights, &mut tiles);
        let camp_origin = worldgen::camp_origin(dims, &heights);
        let mut tree_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_TREES);
        worldgen::place_trees(dims, &heights, &mut tiles, camp_origin, &mut tree_rng);
        let mut spawn_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_SPAWN);

        let mut world = assemble(
            seed,
            dims,
            tiles,
            0,
            ChaCha8Rng::seed_from_u64(seed ^ STREAM_WANDER),
            IdAllocator::default(),
            camp_origin,
            Jobs::default(),
            BTreeMap::new(),
            BTreeSet::new(),
        );
        // Identity has its own stream so it never shifts a spawn position.
        let mut identity_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_IDENTITY);
        let mut names = DwarfName::ALL;
        let mut colours = DwarfColour::ALL;
        names.shuffle(&mut identity_rng);
        colours.shuffle(&mut identity_rng);
        let identities = std::array::from_fn(|i| Identity {
            name: names[i],
            colour: colours[i],
        });
        // Professions have their own stream too, so they never shift a spawn or an identity.
        let mut profession_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_PROFESSION);
        let mut professions = [
            Profession::Miner,
            Profession::Miner,
            Profession::Hauler,
            Profession::Hauler,
            Profession::Woodcutter,
        ];
        professions.shuffle(&mut profession_rng);
        world.spawn_dwarves(camp_origin, &mut spawn_rng, identities, professions);
        world.spawn_emitters(camp_origin);
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
                let current_job = entity.get::<CurrentJob>()?;
                let carrying = entity.get::<Carrying>()?;
                Some(SavedDwarf {
                    id: entity.get::<Id>()?.0,
                    pos: *entity.get::<Pos>()?,
                    state: *entity.get::<JobState>()?,
                    home: wander.home,
                    cooldown: wander.cooldown,
                    current_job: current_job.0.map(|job| job.0),
                    work_progress: entity
                        .get::<WorkProgress>()
                        .map(|progress| progress.0)
                        .unwrap_or(0),
                    carrying: carrying.0,
                    identity: *entity.get::<Identity>()?,
                    profession: *entity.get::<Profession>()?,
                    path: entity
                        .get::<Path>()
                        .map(|path| path.0.clone())
                        .unwrap_or_default(),
                })
            })
            .collect();
        dwarves.sort_by_key(|dwarf| dwarf.id);
        let jobs = self.jobs();
        let job_resource = self.ecs.resource::<Jobs>();
        let kinds: BTreeMap<Id, ItemKind> = self.item_kinds().into_iter().collect();
        let items = self
            .items()
            .into_iter()
            .map(|(id, pos)| (id.0, pos, kinds[&id]))
            .collect();
        let emitters = self
            .emitters()
            // NOTE: like dwarves, an emitter missing Id or Pos is silently skipped by this
            // filter_map. Both construction sites attach Emitter, Id and Pos together.
            .into_iter()
            .map(|(id, pos, light)| (id.0, pos, light))
            .collect();

        SaveState {
            seed: self.seed(),
            tick: self.tick(),
            dims: terrain.dims,
            tiles: terrain.tiles.clone(),
            wander_rng: self.ecs.resource::<WanderRng>().0.clone(),
            next_id: self.ecs.resource::<IdAllocator>().next,
            camp_origin: self.camp_origin(),
            dwarves,
            designations: self.designations(),
            zones: self.zones(),
            jobs,
            next_job_id: job_resource.next_id,
            items,
            emitters,
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
            camp_origin,
            dwarves,
            designations,
            zones,
            jobs,
            next_job_id,
            items,
            emitters,
        } = save;
        let mut job_resource = Jobs {
            next_id: next_job_id,
            ..Jobs::default()
        };
        for job in jobs {
            let inserted = job_resource.insert(job);
            debug_assert!(
                inserted,
                "validated saves have unique job ids, unique tile targets and unique haul items"
            );
        }
        let mut world = assemble(
            seed,
            dims,
            tiles,
            tick,
            wander_rng,
            IdAllocator { next: next_id },
            camp_origin,
            job_resource,
            designations.into_iter().collect(),
            zones.into_iter().collect(),
        );
        for dwarf in dwarves {
            let current_job = dwarf.current_job.map(JobId);
            let entity = world
                .ecs
                .spawn((
                    Dwarf,
                    Id(dwarf.id),
                    dwarf.pos,
                    dwarf.state,
                    Wander {
                        home: dwarf.home,
                        cooldown: dwarf.cooldown,
                    },
                    CurrentJob(current_job),
                    Carrying(dwarf.carrying),
                    dwarf.identity,
                    dwarf.profession,
                ))
                .id();
            if current_job.is_some() {
                world
                    .ecs
                    .entity_mut(entity)
                    .insert(WorkProgress(dwarf.work_progress));
            }
            if !dwarf.path.is_empty() {
                world.ecs.entity_mut(entity).insert(Path(dwarf.path));
            }
        }
        for (id, pos, kind) in items {
            world.ecs.spawn((Item(kind), Id(id), pos));
        }
        for (id, pos, light) in emitters {
            world.ecs.spawn((Emitter(light), Id(id), pos));
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

    pub fn camp_origin(&self) -> Pos {
        self.ecs.resource::<Camp>().0
    }

    pub fn step(&mut self) {
        self.schedule.run(&mut self.ecs);
        debug_assert!(
            {
                let mut stood_on = BTreeSet::new();
                self.ecs
                    .iter_entities()
                    .filter(|entity| entity.contains::<Dwarf>())
                    .filter_map(|entity| entity.get::<Pos>().copied())
                    .all(|pos| stood_on.insert(pos))
            },
            "two dwarves share a tile after tick {}",
            self.tick()
        );
    }

    /// Flat row-major: index = x + y*dims.x + z*dims.x*dims.y
    pub fn tiles(&self) -> &[Tile] {
        &self.ecs.resource::<Terrain>().tiles
    }

    pub fn tile(&self, p: Pos) -> Option<Tile> {
        self.ecs.resource::<Terrain>().tile(p)
    }

    pub fn set_tile(&mut self, p: Pos, tile: Tile) -> bool {
        let changed = self.ecs.resource_mut::<Terrain>().set_tile(p, tile);
        if changed {
            clear_paths(&mut self.ecs);
        }
        changed
    }

    pub fn drain_dirty(&mut self) -> Vec<(Pos, Tile)> {
        self.ecs.resource_mut::<Terrain>().drain_dirty()
    }

    /// One player's stockpile drag, which is several rects over uneven ground. It is refused only
    /// when no rect in it holds a valid cell: a drag that zones the ring around the campfire has
    /// not been refused just because the fire's own cell was left alone in one of its rects.
    ///
    /// NOTE: the refusal names the drag's bounding box, which can span levels; nothing reads the
    /// rect beyond the command it names.
    pub fn place_stockpile(&mut self, rects: &[Rect]) -> Option<Refusal> {
        let mut refused = 0;
        for rect in rects {
            if self
                .apply_command(SimCommand::PlaceStockpile { rect: *rect })
                .is_some()
            {
                refused += 1;
            }
        }
        let first = rects.first()?;
        (refused == rects.len()).then(|| Refusal::PlaceStockpile {
            rect: rects.iter().fold(*first, |bounds, rect| Rect {
                min: Pos {
                    x: bounds.min.x.min(rect.min.x.min(rect.max.x)),
                    y: bounds.min.y.min(rect.min.y.min(rect.max.y)),
                    z: bounds.min.z.min(rect.min.z.min(rect.max.z)),
                },
                max: Pos {
                    x: bounds.max.x.max(rect.min.x.max(rect.max.x)),
                    y: bounds.max.y.max(rect.min.y.max(rect.max.y)),
                    z: bounds.max.z.max(rect.min.z.max(rect.max.z)),
                },
            }),
        })
    }

    /// AD-10: `simd` calls this at loop-iteration start, in arrival order, including while
    /// paused. Designation intake changes marks only; it is not world advancement.
    // NOTE: command ordering is explicit at the call site rather than enforced by `.chain()`.
    pub fn apply_command(&mut self, command: SimCommand) -> Option<Refusal> {
        // Before the rect prelude: this command carries no rect.
        if let SimCommand::SetProfession { dwarf, profession } = command {
            return self.set_profession(dwarf, profession);
        }
        let dims = self.dims();
        let rect = match command {
            SimCommand::Designate { rect, .. }
            | SimCommand::CancelDesignation { rect }
            | SimCommand::PlaceStockpile { rect }
            | SimCommand::RemoveStockpile { rect } => rect,
            SimCommand::SetProfession { .. } => unreachable!("dispatched before the rect prelude"),
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
            return match command {
                SimCommand::PlaceStockpile { rect } => Some(Refusal::PlaceStockpile { rect }),
                _ => None,
            };
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
            SimCommand::Designate { kind, rect } => {
                let workable: Vec<_> = {
                    let terrain = self.ecs.resource::<Terrain>();
                    let is_tree = |pos: Pos| {
                        matches!(
                            terrain.tile(pos),
                            Some(Tile::Solid(Material::TreeTrunk | Material::TreeFoliage))
                        )
                    };
                    match kind {
                        // One mark per distinct tree base, in ascending order.
                        DesignationKind::Cut => positions()
                            .filter_map(|pos| tree_of(terrain, pos).map(|(base, _)| base))
                            .collect::<BTreeSet<_>>()
                            .into_iter()
                            .collect(),
                        _ => positions()
                            .filter(|pos| match kind {
                                DesignationKind::Dig => {
                                    matches!(terrain.tile(*pos), Some(Tile::Solid(_)))
                                        && !is_tree(*pos)
                                }
                                DesignationKind::Channel => {
                                    terrain.is_standable(*pos)
                                        && !is_tree(Pos {
                                            z: pos.z - 1,
                                            ..*pos
                                        })
                                }
                                DesignationKind::Cut => unreachable!("handled above"),
                            })
                            .collect(),
                    }
                };
                let mut designations = self.ecs.resource_mut::<Designations>();
                let mut applied = 0;
                for pos in workable {
                    if designations.0.len() >= MAX_DESIGNATIONS
                        && !designations.0.contains_key(&pos)
                    {
                        continue;
                    }
                    designations.0.insert(pos, kind);
                    applied += 1;
                }
                if applied == 0 {
                    return Some(Refusal::Designate { kind, rect });
                }
            }
            SimCommand::CancelDesignation { .. } => {
                let targets: BTreeSet<_> = positions().collect();
                {
                    let mut designations = self.ecs.resource_mut::<Designations>();
                    for pos in &targets {
                        designations.0.remove(pos);
                    }
                }
                // A cut mark sits at its tree's base, which a surface-following clear drag never
                // reaches: the rect also cancels a mark whose tree has any tile in it.
                let cut_bases: Vec<Pos> = {
                    let terrain = self.ecs.resource::<Terrain>();
                    self.ecs
                        .resource::<Designations>()
                        .0
                        .iter()
                        .filter(|(_, kind)| **kind == DesignationKind::Cut)
                        .filter(|(base, _)| {
                            tree_of(terrain, **base).is_some_and(|(_, tiles)| {
                                tiles.iter().any(|tile| targets.contains(tile))
                            })
                        })
                        .map(|(base, _)| *base)
                        .collect()
                };
                let mut targets = targets;
                {
                    let mut designations = self.ecs.resource_mut::<Designations>();
                    for base in cut_bases {
                        designations.0.remove(&base);
                        targets.insert(base);
                    }
                }
                // Tile jobs only. A haul job's `target` is the stone's position when the job was
                // created and is stale the moment it is picked up, so matching cancel rects
                // against it would drop a haul the player never cancelled — and haul jobs are
                // never dropped (FR8, AC6).
                let job_ids: BTreeSet<_> = self
                    .ecs
                    .resource::<Jobs>()
                    .iter()
                    .filter(|job| {
                        matches!(job.kind, JobKind::Dig | JobKind::Channel | JobKind::Cut)
                    })
                    .filter(|job| targets.contains(&job.target))
                    .map(|job| job.id)
                    .collect();
                {
                    let mut jobs = self.ecs.resource_mut::<Jobs>();
                    for job_id in &job_ids {
                        jobs.remove(*job_id);
                    }
                }
                let holders: Vec<_> = self
                    .ecs
                    .iter_entities()
                    .filter(|entity| {
                        entity
                            .get::<CurrentJob>()
                            .and_then(|current| current.0)
                            .is_some_and(|job_id| job_ids.contains(&job_id))
                    })
                    .map(|entity| entity.id())
                    .collect();
                for entity in holders {
                    release_claim(&mut self.ecs, entity);
                }
            }
            SimCommand::PlaceStockpile { .. } => {
                let blocked = blocked_cells(self.emitters().iter().map(|(_, pos, _)| pos));
                let standable: Vec<_> = {
                    let terrain = self.ecs.resource::<Terrain>();
                    positions()
                        .filter(|pos| is_walkable(terrain, &blocked, *pos))
                        .collect()
                };
                if standable.is_empty() {
                    return Some(Refusal::PlaceStockpile { rect });
                }
                let mut zones = self.ecs.resource_mut::<Zones>();
                zones.0.extend(standable);
            }
            SimCommand::RemoveStockpile { .. } => {
                let mut zones = self.ecs.resource_mut::<Zones>();
                for pos in positions() {
                    zones.0.remove(&pos);
                }
            }
            SimCommand::SetProfession { .. } => unreachable!("dispatched before the rect prelude"),
        }
        None
    }

    fn set_profession(&mut self, dwarf: Id, profession: Profession) -> Option<Refusal> {
        let Some(entity) = self
            .ecs
            .iter_entities()
            .find(|entity| entity.contains::<Dwarf>() && entity.get::<Id>() == Some(&dwarf))
            .map(|entity| entity.id())
        else {
            return Some(Refusal::SetProfession { dwarf });
        };
        if self.ecs.get::<Profession>(entity) == Some(&profession) {
            return None;
        }
        // NOTE: an emptied trade is allowed (12.6 Task 0.2); no last-of-trade refusal.
        self.ecs.entity_mut(entity).insert(profession);
        let held = self
            .ecs
            .get::<CurrentJob>(entity)
            .and_then(|current| current.0)
            .and_then(|job_id| {
                self.ecs
                    .resource::<Jobs>()
                    .iter()
                    .find(|job| job.id == job_id)
            })
            .copied();
        // A held job is always of the old trade, so a changed trade lets go of it. Not
        // `retry_claim`: the job did not fail, and a cooldown would delay the right-trade claim.
        // NOTE: the released job's `WorkProgress` is lost; the next miner starts the dig from 0.
        if held.is_some_and(|job| trade(job.kind) != profession) {
            release_claim(&mut self.ecs, entity);
        }
        None
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

    /// Sorted ascending by dwarf `Id`. A sibling reader to `claims()` and `items()`, so carried
    /// state stays out of `dwarves()` and the clients need no new arm. (`dwarves()` itself is a
    /// four-tuple since 6.2 added the uniform lantern; the split this comment describes is about
    /// what belongs in a sibling reader, not about that tuple's width.)
    pub fn carrying(&self) -> Vec<(Id, Option<u32>)> {
        let mut carrying: Vec<_> = self
            .ecs
            .iter_entities()
            .filter(|entity| entity.contains::<Dwarf>())
            .filter_map(|entity| Some((*entity.get::<Id>()?, entity.get::<Carrying>()?.0)))
            .collect();
        carrying.sort_by_key(|(id, _)| *id);
        carrying
    }

    /// Sorted ascending by dwarf `Id`. A sibling reader like `carrying()`; `dwarves()` stays as is.
    pub fn identities(&self) -> Vec<(Id, Identity)> {
        let mut identities: Vec<_> = self
            .ecs
            .iter_entities()
            .filter(|entity| entity.contains::<Dwarf>())
            .filter_map(|entity| Some((*entity.get::<Id>()?, *entity.get::<Identity>()?)))
            .collect();
        identities.sort_by_key(|(id, _)| *id);
        identities
    }

    /// Sorted ascending by dwarf `Id`. A sibling reader like `identities()`.
    pub fn professions(&self) -> Vec<(Id, Profession)> {
        let mut professions: Vec<_> = self
            .ecs
            .iter_entities()
            .filter(|entity| entity.contains::<Dwarf>())
            .filter_map(|entity| Some((*entity.get::<Id>()?, *entity.get::<Profession>()?)))
            .collect();
        professions.sort_by_key(|(id, _)| *id);
        professions
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

    /// Sorted ascending by `Id`. A sibling reader to `items()`, which keeps its signature.
    pub fn item_kinds(&self) -> Vec<(Id, ItemKind)> {
        let mut kinds: Vec<_> = self
            .ecs
            .iter_entities()
            .filter(|entity| entity.contains::<Item>())
            .filter_map(|entity| Some((*entity.get::<Id>()?, entity.get::<Item>()?.0)))
            .collect();
        kinds.sort_by_key(|(id, _)| *id);
        kinds
    }

    /// Sorted ascending by `Id`.
    pub fn emitters(&self) -> Vec<(Id, Pos, LightKind)> {
        let mut emitters: Vec<_> = self
            .ecs
            .iter_entities()
            .filter_map(|entity| {
                let emitter = entity.get::<Emitter>()?;
                Some((*entity.get::<Id>()?, *entity.get::<Pos>()?, emitter.0))
            })
            .collect();
        emitters.sort_by_key(|(id, ..)| *id);
        emitters
    }

    /// Sorted ascending by `Id` — stable order is required by AD-7.
    // NOTE: `DWARF_LIGHT` is a uniform property, not saved per-dwarf state, until lanterns can be
    // dropped. `carrying()` remains the sibling reader for the one mutable dwarf property.
    pub fn dwarves(&self) -> Vec<(Id, Pos, JobState, LightKind)> {
        let mut dwarves: Vec<_> = self
            .ecs
            .iter_entities()
            .filter(|entity| entity.contains::<Dwarf>())
            .filter_map(|entity| {
                Some((
                    *entity.get::<Id>()?,
                    *entity.get::<Pos>()?,
                    *entity.get::<JobState>()?,
                    DWARF_LIGHT,
                ))
            })
            .collect();
        dwarves.sort_by_key(|(id, ..)| *id);
        dwarves
    }

    fn spawn_dwarves(
        &mut self,
        camp: Pos,
        rng: &mut ChaCha8Rng,
        identities: [Identity; 5],
        professions: [Profession; 5],
    ) {
        let emitter_positions: BTreeSet<_> = camp_emitters(camp)
            .into_iter()
            .map(|(pos, _)| pos)
            .collect();
        let mut candidates = {
            let terrain = self.ecs.resource::<Terrain>();
            let mut candidates = Vec::new();
            let radius = worldgen::CAMP_RADIUS as i32;
            for y in camp.y - radius..=camp.y + radius {
                for x in camp.x - radius..=camp.x + radius {
                    let pos = Pos { x, y, z: camp.z };
                    if terrain.is_standable(pos) && !emitter_positions.contains(&pos) {
                        candidates.push(pos);
                    }
                }
            }
            candidates
        };

        for (identity, profession) in identities.into_iter().zip(professions) {
            let candidate = rng.random_range(0..candidates.len());
            let pos = candidates.swap_remove(candidate);
            let id = self.ecs.resource_mut::<IdAllocator>().allocate();
            self.ecs.spawn((
                Dwarf,
                id,
                pos,
                JobState::Idle,
                Wander {
                    home: camp,
                    // NOTE: staggers the spawn phases so the dwarves do not step in lockstep,
                    // without spending a second RNG draw. It wraps at WANDER_REST_TICKS, so an
                    // eleventh dwarf would share dwarf 0's phase — harmless at five.
                    cooldown: id.0 % WANDER_REST_TICKS,
                },
                CurrentJob(None),
                Carrying(None),
                identity,
                profession,
            ));
        }
    }

    fn spawn_emitters(&mut self, camp: Pos) {
        for (pos, light) in camp_emitters(camp) {
            let id = self.ecs.resource_mut::<IdAllocator>().allocate();
            self.ecs.spawn((Emitter(light), id, pos));
        }
    }
}

/// At F's lighting, moving the torch ring from ±2 to ±8 cells reduced near-white from 1.2713%
/// to 0.7235% and raised the ground median from 69 to 82; dimming lowered both instead.
const TORCH_RING_OFFSET: i32 = 8;

fn camp_emitters(camp: Pos) -> [(Pos, LightKind); 5] {
    [
        (camp, LightKind::Campfire),
        (
            Pos {
                x: camp.x - TORCH_RING_OFFSET,
                y: camp.y - TORCH_RING_OFFSET,
                ..camp
            },
            LightKind::Torch,
        ),
        (
            Pos {
                x: camp.x + TORCH_RING_OFFSET,
                y: camp.y - TORCH_RING_OFFSET,
                ..camp
            },
            LightKind::Torch,
        ),
        (
            Pos {
                x: camp.x - TORCH_RING_OFFSET,
                y: camp.y + TORCH_RING_OFFSET,
                ..camp
            },
            LightKind::Torch,
        ),
        (
            Pos {
                x: camp.x + TORCH_RING_OFFSET,
                y: camp.y + TORCH_RING_OFFSET,
                ..camp
            },
            LightKind::Torch,
        ),
    ]
}

#[cfg(test)]
mod tests {
    /// No fire on the map — the pathing tests below are about terrain, not about issue #74's rule.
    /// Named rather than an inline empty set so a test that MEANS to place a fire reads differently
    /// from one that never considered it.
    static NO_FIRE: std::sync::LazyLock<BTreeSet<Pos>> = std::sync::LazyLock::new(BTreeSet::new);

    use std::{
        collections::{BTreeMap, BTreeSet},
        time::Instant,
    };

    use super::{
        DesignationKind, Dims, Job, JobId, JobKind, JobState, Jobs, Material, Pos, SimCommand,
        Terrain, Tile, World,
    };

    /// Tests insert the trade directly: there is no public setter until 12.6 adds the command.
    fn set_profession(world: &mut World, id: u32, profession: super::Profession) {
        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| {
                entity.contains::<super::Dwarf>()
                    && entity.get::<super::Id>().is_some_and(|i| i.0 == id)
            })
            .expect("dwarf exists")
            .id();
        world.ecs.entity_mut(entity).insert(profession);
    }

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
    fn generated_world_has_sorted_camp_emitters() {
        let world = World::generate(42, Dims::DEFAULT);
        let camp = world.camp_origin();

        assert_eq!(
            world.emitters(),
            vec![
                (super::Id(5), camp, super::LightKind::Campfire),
                (
                    super::Id(6),
                    Pos {
                        x: camp.x - 8,
                        y: camp.y - 8,
                        ..camp
                    },
                    super::LightKind::Torch,
                ),
                (
                    super::Id(7),
                    Pos {
                        x: camp.x + 8,
                        y: camp.y - 8,
                        ..camp
                    },
                    super::LightKind::Torch,
                ),
                (
                    super::Id(8),
                    Pos {
                        x: camp.x - 8,
                        y: camp.y + 8,
                        ..camp
                    },
                    super::LightKind::Torch,
                ),
                (
                    super::Id(9),
                    Pos {
                        x: camp.x + 8,
                        y: camp.y + 8,
                        ..camp
                    },
                    super::LightKind::Torch,
                ),
            ]
        );
        assert_eq!(world.to_save().emitters.len(), 5);
    }

    #[test]
    fn allocator_lives_in_the_ecs() {
        let world = World::generate(42, Dims::DEFAULT);

        assert_eq!(world.ecs.resource::<super::IdAllocator>().next, 10);
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
    fn jobs_index_haul_jobs_by_item_and_never_by_target() {
        let mut jobs = Jobs::default();
        let target = Pos { x: 3, y: 4, z: 5 };
        let dig = Job {
            id: JobId(0),
            kind: JobKind::Dig,
            target,
            created_tick: 0,
            retry_after: 0,
        };
        let haul = Job {
            id: JobId(1),
            kind: JobKind::Haul { item: 7 },
            target,
            created_tick: 0,
            retry_after: 0,
        };

        assert!(jobs.insert(dig));
        // A stone can sit on the tile a dig was designated for, and vice versa. Indexing a
        // haul job by `target` would make one of the two silently refused.
        assert!(jobs.insert(haul));
        assert!(jobs.haul_items.contains(&7));
        assert!(!jobs.insert(Job {
            id: JobId(2),
            kind: JobKind::Haul { item: 7 },
            target: Pos { x: 9, y: 9, z: 9 },
            created_tick: 0,
            retry_after: 0,
        }));

        assert_eq!(jobs.remove(JobId(1)), Some(haul));
        assert!(!jobs.haul_items.contains(&7));
        assert!(
            jobs.targets.contains(&target),
            "removing a haul job must not release a tile job's target"
        );
    }

    #[test]
    fn next_job_id_counts_up_and_saturates_at_the_maximum() {
        let mut jobs = Jobs::default();

        assert_eq!(jobs.next_job_id(), JobId(0));
        assert_eq!(jobs.next_job_id(), JobId(1));

        jobs.next_id = u32::MAX;
        assert_eq!(jobs.next_job_id(), JobId(u32::MAX));
        assert_eq!(
            jobs.next_job_id(),
            JobId(u32::MAX),
            "a saturated allocator must repeat its last id, never wrap onto a reusable one"
        );
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
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), later));
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(11), earlier));

        assert_eq!(
            world.items(),
            vec![(super::Id(11), earlier), (super::Id(12), later)]
        );
        assert_eq!(world.dwarves().len(), 5);
    }

    fn place_stockpile(world: &mut World, pos: Pos) {
        world.apply_command(super::SimCommand::PlaceStockpile {
            rect: super::Rect { min: pos, max: pos },
        });
    }

    fn dwarf_entity(world: &World, id: u32) -> super::Entity {
        world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(id)))
            .expect("dwarf exists")
            .id()
    }

    #[test]
    fn create_haul_jobs_makes_one_job_per_loose_stone_in_ascending_item_order() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let pile = world.dwarves()[0].1;
        place_stockpile(&mut world, pile);
        assert_eq!(world.zones(), vec![pile]);
        // Spawned in descending id order on purpose: the job order must follow the item id,
        // not the order the ECS happens to hand the stones back.
        let loose = [
            (12, Pos { x: 20, y: 20, z: 8 }),
            (11, Pos { x: 21, y: 20, z: 8 }),
            (10, Pos { x: 22, y: 20, z: 8 }),
        ];
        for (id, pos) in loose {
            world
                .ecs
                .spawn((super::Item(super::ItemKind::Stone), super::Id(id), pos));
        }
        world.ecs.resource_mut::<super::Tick>().0 = 7;

        super::create_haul_jobs(&mut world.ecs);

        assert_eq!(
            world.jobs(),
            vec![
                Job {
                    id: JobId(0),
                    kind: JobKind::Haul { item: 10 },
                    target: Pos { x: 22, y: 20, z: 8 },
                    created_tick: 7,
                    retry_after: 0,
                },
                Job {
                    id: JobId(1),
                    kind: JobKind::Haul { item: 11 },
                    target: Pos { x: 21, y: 20, z: 8 },
                    created_tick: 7,
                    retry_after: 0,
                },
                Job {
                    id: JobId(2),
                    kind: JobKind::Haul { item: 12 },
                    target: Pos { x: 20, y: 20, z: 8 },
                    created_tick: 7,
                    retry_after: 0,
                },
            ]
        );

        super::create_haul_jobs(&mut world.ecs);

        assert_eq!(
            world.jobs().len(),
            3,
            "an unchanged world grew a second job for a stone that already has one"
        );
    }

    #[test]
    fn no_stockpile_means_no_haul_job_at_all() {
        let mut world = World::generate(42, Dims::DEFAULT);
        world.ecs.spawn((
            super::Item(super::ItemKind::Stone),
            super::Id(12),
            Pos { x: 20, y: 20, z: 8 },
        ));

        super::create_haul_jobs(&mut world.ecs);
        assert!(world.jobs().is_empty());

        for _ in 0..20 {
            world.step();
        }

        assert!(
            world.jobs().is_empty(),
            "a stone became work with nowhere to put it"
        );
        assert!(world.carrying().iter().all(|(_, item)| item.is_none()));
    }

    #[test]
    fn a_stockpile_placed_over_a_loose_stone_retires_its_job_and_idles_the_claimant() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let stone = world.dwarves()[1].1;
        let pile = world.dwarves()[0].1;
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), stone));
        place_stockpile(&mut world, pile);
        super::create_haul_jobs(&mut world.ecs);
        let job = world.jobs()[0];
        assert_eq!(job.kind, JobKind::Haul { item: 12 });
        // A dwarf holds the job but has not reached the stone, so it carries nothing — the only
        // way a stone can become stored while its job is claimed.
        let entity = dwarf_entity(&world, 3);
        world.ecs.get_mut::<super::CurrentJob>(entity).unwrap().0 = Some(job.id);
        world
            .ecs
            .entity_mut(entity)
            .insert((super::Path(Vec::new()), super::WorkProgress(0)));

        place_stockpile(&mut world, stone);
        super::create_haul_jobs(&mut world.ecs);

        assert!(world.jobs().is_empty(), "a stored stone kept its haul job");
        assert_eq!(world.claims()[3].1, None);
        assert_eq!(world.dwarves()[3].2, JobState::Idle);
        assert_eq!(world.carrying()[3], (super::Id(3), None));
        assert_eq!(world.items(), vec![(super::Id(12), stone)]);
        assert!(!world.ecs.entity(entity).contains::<super::Path>());
    }

    fn make_standable(world: &mut World, pos: Pos) {
        assert!(world.set_tile(
            Pos {
                z: pos.z - 1,
                ..pos
            },
            Tile::Solid(Material::Stone),
        ));
        assert!(world.set_tile(pos, Tile::Empty));
    }

    /// A four-cell standable run east (or west, at the map edge) of dwarf zero.
    fn corridor(world: &mut World) -> impl Fn(i32) -> Pos + 'static {
        let start = world.dwarves()[0].1;
        let dx = if start.x + 4 < world.dims().x as i32 {
            1
        } else {
            -1
        };
        let cell = move |steps: i32| Pos {
            x: start.x + dx * steps,
            ..start
        };
        for steps in 1..=4 {
            make_standable(world, cell(steps));
        }
        cell
    }

    #[test]
    fn a_haul_walks_picks_up_walks_and_drops_in_two_work_runs() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let cell = corridor(&mut world);
        let stone = cell(2);
        let pile = cell(4);
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), stone));
        place_stockpile(&mut world, pile);
        // A real, unrelated order at the stone's tile. The dig path removes the designation at
        // `job.target`; a haul must not, or it deletes an order the player gave.
        world.apply_command(super::SimCommand::Designate {
            kind: super::DesignationKind::Channel,
            rect: super::Rect {
                min: stone,
                max: stone,
            },
        });
        super::create_haul_jobs(&mut world.ecs);
        let job = world.jobs()[0];
        assert_eq!(job.kind, JobKind::Haul { item: 12 });
        let entity = dwarf_entity(&world, 0);
        world.ecs.get_mut::<super::CurrentJob>(entity).unwrap().0 = Some(job.id);
        world
            .ecs
            .entity_mut(entity)
            .insert((super::Path(Vec::new()), super::WorkProgress(0)));
        world.drain_dirty();

        let mut states = Vec::new();
        for _ in 0..80 {
            super::execute_jobs(&mut world.ecs);
            states.push(world.dwarves()[0].2);
            if world.jobs().is_empty() {
                break;
            }
        }

        use JobState::{Idle, Walk, Work};
        // Run-length encoded, because the literal sequence this used to assert was really two
        // numbers -- the length of each walk -- buried in a list that had to be rewritten by hand
        // whenever the pacing moved. Stating them as `walk_ticks(steps)` keeps the cadence pinned
        // to `STEP_REST_TICKS` instead of to a magic list, so a pacing regression still fails here
        // while a deliberate pacing CHANGE does not need the expectation rewritten.
        fn walk_ticks(steps: usize) -> usize {
            1 + (steps - 1) * (1 + super::STEP_REST_TICKS as usize)
        }
        // 12.9 Task 10 (#162): an item blocks its tile, so the hauler lifts the stone from the
        // next tile: the first walk is ONE step (cell 0 -> cell 1, beside the stone at cell 2),
        // not two. The delivery walk (cell 1 -> cell 3, beside the pile at cell 4) is unchanged
        // in length. Intended change from the old rule, which sent the hauler onto the stone.
        let mut runs: Vec<(JobState, usize)> = Vec::new();
        for state in &states {
            match runs.last_mut() {
                Some((last, count)) if last == state => *count += 1,
                _ => runs.push((*state, 1)),
            }
        }
        assert_eq!(
            runs,
            vec![
                (Walk, walk_ticks(1)),
                (Work, super::WORK_TICKS as usize),
                (Walk, walk_ticks(3)),
                (Work, super::WORK_TICKS as usize),
                (Idle, 1),
            ],
            "two walks, paced by STEP_REST_TICKS, and exactly WORK_TICKS of work in each leg"
        );
        assert_eq!(world.items(), vec![(super::Id(12), pile)]);
        assert_eq!(world.carrying()[0], (super::Id(0), None));
        assert!(world.claims().iter().all(|(_, job)| job.is_none()));
        assert!(world.jobs().is_empty());
        assert_eq!(
            world.designations(),
            vec![(stone, super::DesignationKind::Channel)],
            "a haul completion removed a designation at its stale target"
        );
        assert!(
            world.drain_dirty().is_empty(),
            "a haul completion mutated a tile"
        );
    }

    #[test]
    fn a_stone_on_unstandable_ground_leaves_its_haul_job_queued_and_retried() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let cell = corridor(&mut world);
        place_stockpile(&mut world, cell(4));
        // Items never fall, so a stone whose floor was dug away has no standable position and
        // no work position with it.
        let stranded = Pos {
            x: 20,
            y: 20,
            z: 20,
        };
        assert!(world.set_tile(stranded, Tile::Empty));
        assert!(world.set_tile(
            Pos {
                z: stranded.z - 1,
                ..stranded
            },
            Tile::Empty,
        ));
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), stranded));

        for _ in 0..60 {
            world.step();
        }

        assert_eq!(world.jobs().len(), 1, "the unreachable job was dropped");
        assert_eq!(world.jobs()[0].kind, JobKind::Haul { item: 12 });
        assert!(world.jobs()[0].retry_after > 0, "the job was never retried");
        assert!(world.claims().iter().all(|(_, job)| job.is_none()));
        assert!(world.carrying().iter().all(|(_, item)| item.is_none()));
        assert_eq!(world.items(), vec![(super::Id(12), stranded)]);
    }

    #[test]
    fn a_full_stockpile_parks_the_haul_job_until_a_free_tile_appears() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let cell = corridor(&mut world);
        let loose = cell(2);
        let pile = cell(4);
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(11), pile));
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), loose));
        place_stockpile(&mut world, pile);

        for _ in 0..60 {
            world.step();
        }

        assert_eq!(
            world.jobs().len(),
            1,
            "one job for the loose stone, none for the stored one: {:?}",
            world.jobs()
        );
        assert_eq!(world.jobs()[0].kind, JobKind::Haul { item: 12 });
        assert!(
            world.jobs()[0].retry_after > 0,
            "a job with nowhere to deliver was never retried"
        );
        assert!(
            world.claims().iter().all(|(_, job)| job.is_none()),
            "a job with nowhere to deliver was claimed into a pick-up-and-drop cycle"
        );
        assert!(world.carrying().iter().all(|(_, item)| item.is_none()));
        assert_eq!(
            world.items(),
            vec![(super::Id(11), pile), (super::Id(12), loose)]
        );

        // One stone per stockpile tile: a second tile is all it takes to revive the job.
        place_stockpile(&mut world, cell(3));
        for _ in 0..300 {
            world.step();
            if world.jobs().is_empty() {
                break;
            }
        }

        assert!(world.jobs().is_empty(), "the revived job never completed");
        assert_eq!(
            world.items(),
            vec![(super::Id(11), pile), (super::Id(12), cell(3))]
        );
        assert!(world.carrying().iter().all(|(_, item)| item.is_none()));
    }

    /// `Job.target` for a haul is only the stone's position when the job was created. Claiming
    /// and execution must read the stone's live `Pos`, so a job whose target has gone stale still
    /// sends a dwarf to the stone — and the stone never jumps to meet the dwarf instead.
    /// AC8's pick-up effect, clause by clause, asserted on the components rather than through a
    /// scenario. The path handed in is deliberately NON-empty: in production the walk always
    /// exhausts it before arrival, which is why review found this clause pinned by nothing.
    #[test]
    fn pickup_sets_carrying_resets_the_work_counter_and_spends_the_path() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let cell = corridor(&mut world);
        let stone = cell(2);
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), stone));
        place_stockpile(&mut world, cell(4));
        super::create_haul_jobs(&mut world.ecs);
        let job = world.jobs()[0];
        assert_eq!(job.kind, JobKind::Haul { item: 12 });
        let entity = dwarf_entity(&world, 0);
        // Standing beside the stone already (12.9 Task 10: a dwarf never stands on an item, it
        // lifts from the next tile), so the very next WORK_TICKS runs end in the pick-up.
        *world.ecs.get_mut::<Pos>(entity).unwrap() = cell(1);
        world.ecs.get_mut::<super::CurrentJob>(entity).unwrap().0 = Some(job.id);
        world
            .ecs
            .entity_mut(entity)
            .insert((super::Path(vec![cell(3)]), super::WorkProgress(0)));

        for _ in 0..super::WORK_TICKS {
            super::execute_jobs(&mut world.ecs);
        }
        assert_eq!(world.carrying()[0], (super::Id(0), None), "picked up early");

        super::execute_jobs(&mut world.ecs);

        assert_eq!(world.carrying()[0], (super::Id(0), Some(12)));
        assert_eq!(
            world.claims()[0].1,
            Some(job.id),
            "a pick-up must not complete the job"
        );
        assert!(world.jobs().iter().any(|queued| queued.id == job.id));
        assert_eq!(
            world.ecs.get::<super::WorkProgress>(entity).map(|p| p.0),
            Some(0),
            "the second leg's work counter must start from zero"
        );
        assert!(
            !world.ecs.entity(entity).contains::<super::Path>(),
            "the path to the stone must be spent, not carried into the delivery leg"
        );
    }

    /// The haul goal sets, called directly — the only way to see the pick-up leg's standability
    /// rule, since an unreachable stone is unclaimable with or without it.
    ///
    /// 12.9 Task 10 (#162) changed this test's rule, intentionally: an item blocks its tile, so
    /// the pick-up leg is the stone's walkable 4-neighbours (it was the stone's own tile), and the
    /// delivery leg is the walkable 4-neighbours of the free pile cell (it was the cell itself).
    /// As in production, `blocked` carries the uncarried items' tiles.
    #[test]
    fn haul_work_positions_gate_both_legs_on_a_free_standable_pile_tile() {
        let terrain = flat_terrain(5, 1);
        let pile = Pos { x: 0, y: 0, z: 1 };
        let zones = BTreeSet::from([pile]);
        let standing = Pos { x: 2, y: 0, z: 1 };
        // z == 0 is the solid floor, so a stone there is on no standable tile.
        let sunken = Pos { x: 2, y: 0, z: 0 };
        let job = Job {
            id: JobId(0),
            kind: JobKind::Haul { item: 12 },
            target: standing,
            created_tick: 0,
            retry_after: 0,
        };
        let work = |items: &BTreeMap<u32, Pos>, carrying| {
            let blocked: BTreeSet<Pos> = items.values().copied().collect();
            super::work_positions(&terrain, &blocked, &zones, items, job, carrying)
        };

        let reachable = BTreeMap::from([(12, standing)]);
        assert_eq!(
            work(&reachable, None),
            BTreeSet::from([Pos { x: 1, y: 0, z: 1 }, Pos { x: 3, y: 0, z: 1 }]),
            "a standable stone with a free pile tile is lifted from its walkable neighbours"
        );
        assert_eq!(
            work(&reachable, Some(12)),
            BTreeSet::from([Pos { x: 1, y: 0, z: 1 }]),
            "a carrying dwarf is sent to the walkable neighbour of the free pile tile"
        );

        let unstandable = BTreeMap::from([(12, sunken)]);
        assert!(
            work(&unstandable, None).is_empty(),
            "a stone on unstandable ground has no work position"
        );

        // 12.9: the stone's NEIGHBOURS are now the work positions, so a stone whose own floor is
        // gone while its neighbours' floors stand is the case only the pick-up leg's own standable
        // gate can refuse (the `sunken` stone above has no walkable neighbour either way).
        let mut holed = flat_terrain(5, 1);
        assert!(holed.set_tile(
            Pos {
                z: standing.z - 1,
                ..standing
            },
            Tile::Empty
        ));
        let holed_blocked: BTreeSet<Pos> = reachable.values().copied().collect();
        assert!(
            super::work_positions(&holed, &holed_blocked, &zones, &reachable, job, None).is_empty(),
            "a stone whose floor is gone has no work position even beside standable ground"
        );

        // The pile itself holding a stored stone leaves both legs empty.
        let occupied = BTreeMap::from([(12, standing), (13, pile)]);
        assert!(
            work(&occupied, None).is_empty(),
            "the pick-up leg must be gated on a free tile existing"
        );
        assert!(
            work(&occupied, Some(12)).is_empty(),
            "a full pile is no delivery target"
        );
    }

    #[test]
    fn an_old_save_zone_on_an_emitter_is_no_haul_goal() {
        let terrain = flat_terrain(5, 1);
        let fire = Pos { x: 0, y: 0, z: 1 };
        let stone = Pos { x: 2, y: 0, z: 1 };
        let blocked = BTreeSet::from([fire]);
        let zones = BTreeSet::from([fire]);
        let items = BTreeMap::from([(12, stone)]);
        let job = Job {
            id: JobId(0),
            kind: JobKind::Haul { item: 12 },
            target: stone,
            created_tick: 0,
            retry_after: 0,
        };
        assert!(super::work_positions(&terrain, &blocked, &zones, &items, job, None).is_empty());
        assert!(
            super::work_positions(&terrain, &blocked, &zones, &items, job, Some(12)).is_empty()
        );
    }

    #[test]
    fn a_drag_is_refused_only_when_none_of_its_rects_has_a_valid_cell() {
        let mut world = World::generate(super::DEFAULT_SEED, Dims::DEFAULT);
        let fire = world.camp_origin();
        let cell = |dy: i32| super::Rect {
            min: Pos {
                y: fire.y + dy,
                ..fire
            },
            max: Pos {
                y: fire.y + dy,
                ..fire
            },
        };
        // The gui's column drag through the fire: one rect per cell, the fire's alone.
        let column: Vec<_> = (-2..=2).map(cell).collect();
        assert_eq!(world.place_stockpile(&column), None);
        assert_eq!(world.zones().len(), 4);

        let rock = super::Rect {
            min: Pos {
                z: fire.z - 1,
                ..fire
            },
            max: Pos {
                z: fire.z - 1,
                ..fire
            },
        };
        assert_eq!(
            world.place_stockpile(&[cell(0), rock]),
            Some(super::Refusal::PlaceStockpile {
                rect: super::Rect {
                    min: rock.min,
                    max: fire,
                },
            })
        );
        assert_eq!(world.zones().len(), 4);
    }

    #[test]
    fn stockpile_refuses_only_when_every_cell_is_invalid() {
        let mut world = World::generate(super::DEFAULT_SEED, Dims::DEFAULT);
        let fire = world.camp_origin();
        let fire_rect = super::Rect {
            min: fire,
            max: fire,
        };
        assert_eq!(
            world.apply_command(SimCommand::PlaceStockpile { rect: fire_rect }),
            Some(super::Refusal::PlaceStockpile { rect: fire_rect })
        );
        assert!(world.zones().is_empty());

        let rock = Pos {
            z: fire.z - 1,
            ..fire
        };
        let rock_rect = super::Rect {
            min: rock,
            max: rock,
        };
        assert_eq!(
            world.apply_command(SimCommand::PlaceStockpile { rect: rock_rect }),
            Some(super::Refusal::PlaceStockpile { rect: rock_rect })
        );
        let off = Pos {
            x: -1,
            y: -1,
            z: -1,
        };
        let off_rect = super::Rect { min: off, max: off };
        assert_eq!(
            world.apply_command(SimCommand::PlaceStockpile { rect: off_rect }),
            Some(super::Refusal::PlaceStockpile { rect: off_rect })
        );
        assert!(world.zones().is_empty());

        let around = super::Rect {
            min: Pos {
                x: fire.x - 1,
                y: fire.y - 1,
                ..fire
            },
            max: Pos {
                x: fire.x + 1,
                y: fire.y + 1,
                ..fire
            },
        };
        assert_eq!(
            world.apply_command(SimCommand::PlaceStockpile { rect: around }),
            None
        );
        assert_eq!(world.zones().len(), 8);
        assert!(!world.zones().contains(&fire));

        assert_eq!(
            world.apply_command(SimCommand::Designate {
                kind: DesignationKind::Dig,
                rect: rock_rect
            }),
            None
        );
        assert_eq!(
            world.apply_command(SimCommand::CancelDesignation { rect: rock_rect }),
            None
        );
        assert_eq!(
            world.apply_command(SimCommand::RemoveStockpile { rect: around }),
            None
        );
    }

    #[test]
    fn haul_execution_reads_the_stones_live_position_not_the_jobs_target() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let cell = corridor(&mut world);
        let stone = cell(1);
        // 12.9: the work positions are the stone's NEIGHBOURS, so a stale target two tiles from the
        // stone (cell 3) still shared a neighbour with it (cell 2) and a pick-up from the target's
        // neighbours looked right. Cell 4 shares none: a hauler sent there lifts the stone from
        // two or more tiles away and the stone jumps.
        let pile = cell(4);
        let stale = pile;
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), stone));
        place_stockpile(&mut world, pile);
        assert!(world.ecs.resource_mut::<Jobs>().insert(Job {
            id: JobId(0),
            kind: JobKind::Haul { item: 12 },
            target: stale,
            created_tick: 0,
            retry_after: 0,
        }));

        for _ in 0..200 {
            let before = world.items();
            world.step();
            for (id, pos) in world.items() {
                let was = before
                    .iter()
                    .find(|(old, _)| *old == id)
                    .expect("stones are never despawned")
                    .1;
                let step = (pos.x - was.x)
                    .abs()
                    .max((pos.y - was.y).abs())
                    .max((pos.z - was.z).abs());
                assert!(step <= 1, "stone {id:?} jumped from {was:?} to {pos:?}");
            }
            if world.jobs().is_empty() {
                break;
            }
        }

        assert!(world.jobs().is_empty(), "the haul never completed");
        assert_eq!(world.items(), vec![(super::Id(12), pile)]);
        assert!(world.carrying().iter().all(|(_, item)| item.is_none()));
    }

    #[test]
    fn a_stockpile_tile_whose_floor_is_gone_is_never_a_delivery_target() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let cell = corridor(&mut world);
        let loose = cell(2);
        let pile = cell(4);
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), loose));
        place_stockpile(&mut world, pile);
        // Zone tiles are validated standable when the command lands and never re-checked, so a
        // pile can lose its floor to a later dig.
        assert!(world.set_tile(
            Pos {
                z: pile.z - 1,
                ..pile
            },
            Tile::Empty,
        ));

        for _ in 0..80 {
            world.step();
            // AC6's shape: with no free tile the goal set is empty at BOTH legs, so the job is
            // never claimed — not claimed and then abandoned halfway to a pile nobody can
            // stand on.
            assert!(
                world.claims().iter().all(|(_, job)| job.is_none()),
                "a job whose only pile tile lost its floor was claimed: {:?}",
                world.claims()
            );
            assert!(
                world.carrying().iter().all(|(_, item)| item.is_none()),
                "a stone was picked up for a pile tile nobody can stand on"
            );
        }

        assert_eq!(world.jobs().len(), 1);
        assert_eq!(world.jobs()[0].kind, JobKind::Haul { item: 12 });
        assert_eq!(world.items(), vec![(super::Id(12), loose)]);
        assert!(world.zones().contains(&pile));
    }

    #[test]
    fn carrying_reader_lists_every_dwarf_ascending_by_id() {
        let mut world = World::generate(42, Dims::DEFAULT);

        assert_eq!(
            world.carrying(),
            vec![
                (super::Id(0), None),
                (super::Id(1), None),
                (super::Id(2), None),
                (super::Id(3), None),
                (super::Id(4), None),
            ],
            "every dwarf carries nothing at spawn, and none is missing from the reader"
        );

        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(3)))
            .expect("dwarf three exists")
            .id();
        world.ecs.get_mut::<super::Carrying>(entity).unwrap().0 = Some(12);

        assert_eq!(world.carrying()[3], (super::Id(3), Some(12)));
    }

    #[test]
    fn a_carried_stone_tracks_its_carrier_every_tick_including_a_settle_fall() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let start = world.dwarves()[0].1;
        let stone = Pos { x: 0, y: 0, z: 1 };
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), stone));
        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("dwarf zero exists")
            .id();
        world.ecs.get_mut::<super::Carrying>(entity).unwrap().0 = Some(12);

        // Wandering moves the carrier; the stone must be under it at the end of every tick.
        for _ in 0..12 {
            world.step();
            assert_eq!(
                world.items(),
                vec![(super::Id(12), world.dwarves()[0].1)],
                "a carried stone lagged behind its carrier"
            );
        }

        // Now the ground under the carrier is dug away and `settle` — not `wander` — moves it.
        let standing = world.dwarves()[0].1;
        let below = Pos {
            z: standing.z - 1,
            ..standing
        };
        assert!(world.set_tile(below, Tile::Empty));
        assert!(world.set_tile(
            Pos {
                z: standing.z - 2,
                ..standing
            },
            Tile::Solid(Material::Stone),
        ));

        world.step();

        assert_eq!(world.dwarves()[0].1, below, "the carrier did not fall");
        assert_eq!(
            world.items(),
            vec![(super::Id(12), below)],
            "the stone stayed on the level its carrier fell from"
        );
        assert_ne!(start, below);
    }

    #[test]
    fn release_claim_drops_the_carried_stone_at_the_dwarfs_tile() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let dwarf_pos = world.dwarves()[0].1;
        let far_away = Pos { x: 0, y: 0, z: 1 };
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), far_away));
        let job = Job {
            id: JobId(0),
            kind: JobKind::Haul { item: 12 },
            target: far_away,
            created_tick: 0,
            retry_after: 0,
        };
        assert!(world.ecs.resource_mut::<Jobs>().insert(job));
        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("dwarf zero exists")
            .id();
        world.ecs.get_mut::<super::CurrentJob>(entity).unwrap().0 = Some(job.id);
        world.ecs.get_mut::<super::Carrying>(entity).unwrap().0 = Some(12);

        super::release_claim(&mut world.ecs, entity);

        assert_eq!(
            world.items(),
            vec![(super::Id(12), dwarf_pos)],
            "an abnormal exit must leave a loose stone where the dwarf stood"
        );
        assert_eq!(world.carrying()[0], (super::Id(0), None));
        assert_eq!(world.claims()[0].1, None);
        assert_eq!(world.dwarves()[0].2, JobState::Idle);
    }

    #[test]
    fn release_claim_avoids_an_occupied_stockpile_cell() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let dwarf_pos = world.dwarves()[0].1;
        let far_away = Pos { x: 0, y: 0, z: 1 };
        world.ecs.resource_mut::<super::Zones>().0.insert(dwarf_pos);
        world.ecs.spawn((
            super::Item(super::ItemKind::Stone),
            super::Id(11),
            dwarf_pos,
        ));
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(12), far_away));
        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("dwarf zero exists")
            .id();
        world.ecs.get_mut::<super::Carrying>(entity).unwrap().0 = Some(12);

        super::release_claim(&mut world.ecs, entity);

        let items = world.items();
        assert!(items.contains(&(super::Id(11), dwarf_pos)));
        let dropped = items.iter().find(|(id, _)| *id == super::Id(12)).unwrap().1;
        assert_ne!(dropped, dwarf_pos);
        assert!(!world.zones().contains(&dropped));
        assert_eq!(
            dwarf_pos.x.abs_diff(dropped.x)
                + dwarf_pos.y.abs_diff(dropped.y)
                + dwarf_pos.z.abs_diff(dropped.z),
            1,
            "the nearest open tile is a neighbour"
        );
        assert_eq!(world.carrying()[0], (super::Id(0), None));
    }

    #[test]
    fn release_claim_drops_where_the_carrier_can_walk() {
        // A walled pocket three cells long: the carrier on a full pile cell, then free cells. By
        // grid distance the top of a wall is as near as the free cell and sorts first, but no
        // dwarf can reach it without a ramp.
        //
        // 12.9 Task 10 (#162), intended change: items block, so a second full pile cell now
        // WALLS the pocket (the old test searched through it). That left pocket[1] the only
        // answer, which a search through rock finds too, so pocket[1] now holds a loose stone and
        // the carrier is sealed in; the intent -- the drop is where the carrier can walk -- is unchanged.
        let mut world = World::generate(42, Dims::DEFAULT);
        let p = world.dwarves()[0].1;
        let pocket = [p, Pos { y: p.y + 1, ..p }, Pos { y: p.y + 2, ..p }];
        for x in p.x - 1..=p.x + 1 {
            for y in p.y - 1..=p.y + 3 {
                let cell = Pos { x, y, ..p };
                let below = Pos { z: p.z - 1, ..cell };
                if pocket.contains(&cell) {
                    world.set_tile(cell, Tile::Empty);
                    world.set_tile(below, Tile::Solid(Material::Stone));
                } else {
                    world.set_tile(cell, Tile::Solid(Material::Stone));
                }
            }
        }
        world
            .ecs
            .resource_mut::<super::Zones>()
            .0
            .extend([pocket[0]]);
        world.ecs.spawn((
            super::Item(super::ItemKind::Stone),
            super::Id(12),
            Pos { x: 0, y: 0, z: 1 },
        ));
        world.ecs.spawn((
            super::Item(super::ItemKind::Stone),
            super::Id(13),
            pocket[0],
        ));
        // 12.9: a loose stone on the pocket's second cell walls the carrier in (items block), so
        // nothing is reachable. A search that walks through rock would still find the wall tops
        // and the cells behind the rock; the right answer is to keep the stack on its own tile.
        world.ecs.spawn((
            super::Item(super::ItemKind::Stone),
            super::Id(14),
            pocket[1],
        ));
        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("dwarf zero exists")
            .id();
        world.ecs.get_mut::<super::Carrying>(entity).unwrap().0 = Some(12);

        super::release_claim(&mut world.ecs, entity);

        let dropped = world
            .items()
            .into_iter()
            .find(|(id, _)| *id == super::Id(12))
            .unwrap()
            .1;
        assert_eq!(
            dropped, pocket[0],
            "walled in, the stack stays on the carrier's own tile and never leaves the pocket"
        );
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
        set_profession(&mut world, 2, super::Profession::Miner);
        let worker = world.dwarves()[2].1;
        let target = Pos {
            x: if worker.x + 1 < world.dims().x as i32 {
                worker.x + 1
            } else {
                worker.x - 1
            },
            ..worker
        };
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
        // Read his cell on the tick BEFORE the claim rather than reusing the one captured at
        // spawn: he is jobless until here, so he has been wandering, and how far a wander has
        // carried him is a function of the movement pacing rather than of anything this test is
        // about. What it is about is that the claim tick claims and does NOT also step.
        let before_claim = world.dwarves()[2].1;
        world.step();
        assert_eq!(world.claims()[2], (super::Id(2), Some(JobId(0))));
        assert_eq!(world.dwarves()[2].1, before_claim);
        assert_eq!(world.dwarves()[2].2, JobState::Walk);
        let worker = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(2)))
            .expect("dwarf two exists");
        assert!(worker.contains::<super::Path>());
        assert!(worker.contains::<super::WorkProgress>());
    }

    #[test]
    fn claim_jobs_takes_fifo_and_skips_busy_dwarves_and_claimed_jobs() {
        let mut world = World::generate(42, Dims::DEFAULT);
        set_profession(&mut world, 0, super::Profession::Miner);
        let worker = world.dwarves()[0].1;
        let first_target = Pos {
            x: worker.x + 1,
            ..worker
        };
        let second_target = Pos {
            y: worker.y + 1,
            ..worker
        };
        assert!(world.set_tile(first_target, Tile::Solid(Material::Stone)));
        assert!(world.set_tile(second_target, Tile::Solid(Material::Stone)));
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
                target: first_target,
                created_tick: 0,
                retry_after: 0,
            }));
            assert!(jobs.insert(Job {
                id: JobId(1),
                kind: JobKind::Dig,
                target: second_target,
                created_tick: 0,
                retry_after: 0,
            }));
        }
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        world.step();
        assert_eq!(world.claims()[0], (super::Id(0), Some(JobId(0))));

        let mut claimed = World::generate(42, Dims::DEFAULT);
        set_profession(&mut claimed, 0, super::Profession::Miner);
        let claimed_worker = claimed.dwarves()[0].1;
        let claimed_first_target = Pos {
            x: claimed_worker.x + 1,
            ..claimed_worker
        };
        let claimed_second_target = Pos {
            y: claimed_worker.y + 1,
            ..claimed_worker
        };
        assert!(claimed.set_tile(claimed_first_target, Tile::Solid(Material::Stone)));
        assert!(claimed.set_tile(claimed_second_target, Tile::Solid(Material::Stone)));
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
                target: claimed_first_target,
                created_tick: 0,
                retry_after: 0,
            }));
            assert!(jobs.insert(Job {
                id: JobId(1),
                kind: JobKind::Dig,
                target: claimed_second_target,
                created_tick: 0,
                retry_after: 0,
            }));
        }
        claimed.ecs.resource_mut::<super::Tick>().0 = 100;
        claimed.step();
        assert_eq!(claimed.claims()[0], (super::Id(0), Some(JobId(1))));
    }

    #[test]
    fn claim_jobs_prefers_the_lowest_free_dwarf_id() {
        let mut world = World::generate(42, Dims::DEFAULT);
        for id in 0..5 {
            set_profession(&mut world, id, super::Profession::Miner);
        }
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        let worker = world.dwarves()[0].1;
        let target = Pos {
            x: worker.x + 1,
            ..worker
        };
        assert!(world.set_tile(target, Tile::Solid(Material::Stone)));
        assert!(world.ecs.resource_mut::<Jobs>().insert(Job {
            id: JobId(0),
            kind: JobKind::Dig,
            target,
            created_tick: 0,
            retry_after: 0,
        }));
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(super::claim_jobs);

        schedule.run(&mut world.ecs);

        assert_eq!(world.claims()[0], (super::Id(0), Some(JobId(0))));
        assert!(
            world.claims()[1..]
                .iter()
                .all(|(_, current)| current.is_none())
        );
    }

    #[test]
    fn an_unreachable_lower_id_does_not_starve_a_reachable_dwarf() {
        let mut world = World::generate(42, Dims::DEFAULT);
        set_profession(&mut world, 0, super::Profession::Miner);
        set_profession(&mut world, 1, super::Profession::Miner);
        let unreachable = Pos { x: 10, y: 10, z: 1 };
        let reachable = Pos { x: 20, y: 20, z: 1 };
        let target = Pos { x: 21, y: 20, z: 1 };
        for (pos, tile) in [
            (
                Pos {
                    z: 0,
                    ..unreachable
                },
                Tile::Solid(Material::Stone),
            ),
            (unreachable, Tile::Empty),
            (Pos { z: 0, ..reachable }, Tile::Solid(Material::Stone)),
            (reachable, Tile::Empty),
            (Pos { z: 0, ..target }, Tile::Solid(Material::Stone)),
            (target, Tile::Solid(Material::Stone)),
        ] {
            assert!(world.set_tile(pos, tile));
        }
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            assert!(world.set_tile(
                Pos {
                    x: unreachable.x + dx,
                    y: unreachable.y + dy,
                    z: unreachable.z,
                },
                Tile::Solid(Material::Stone),
            ));
        }
        let entities: Vec<_> = world
            .ecs
            .iter_entities()
            .filter_map(|entity| Some((*entity.get::<super::Id>()?, entity.id())))
            .collect();
        for (id, entity) in entities {
            match id.0 {
                0 => *world.ecs.get_mut::<Pos>(entity).unwrap() = unreachable,
                1 => *world.ecs.get_mut::<Pos>(entity).unwrap() = reachable,
                _ => {
                    world.ecs.despawn(entity);
                }
            }
        }
        assert!(world.ecs.resource_mut::<Jobs>().insert(Job {
            id: JobId(0),
            kind: JobKind::Dig,
            target,
            created_tick: 0,
            retry_after: 0,
        }));
        world
            .ecs
            .resource_mut::<super::Designations>()
            .0
            .insert(target, super::DesignationKind::Dig);
        world.ecs.resource_mut::<super::Tick>().0 = 100;

        for _ in 0..100 {
            world.step();
            if !world.items().is_empty() {
                break;
            }
        }

        assert_eq!(world.items(), vec![(super::Id(10), target)]);
        assert_eq!(
            world.item_kinds(),
            vec![(super::Id(10), super::ItemKind::Stone)]
        );
        assert!(world.jobs().is_empty());
    }

    /// A standable cell next to a dwarf's spawn cell, off every emitter and every listed cell.
    fn free_cell_beside(world: &World, near: Pos, taken: &[Pos]) -> Pos {
        let emitters: Vec<Pos> = world
            .emitters()
            .into_iter()
            .map(|(_, pos, _)| pos)
            .collect();
        let terrain = world.ecs.resource::<Terrain>();
        [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (2, 0),
            (-2, 0),
            (0, 2),
            (0, -2),
        ]
        .into_iter()
        .map(|(dx, dy)| Pos {
            x: near.x + dx,
            y: near.y + dy,
            ..near
        })
        .find(|cell| {
            terrain.is_standable(*cell) && !emitters.contains(cell) && !taken.contains(cell)
        })
        .expect("the camp has a free standable cell beside every dwarf")
    }

    fn dwarf_of(world: &World, profession: super::Profession, nth: usize) -> (super::Id, Pos) {
        let id = world
            .professions()
            .into_iter()
            .filter(|(_, p)| *p == profession)
            .nth(nth)
            .expect("seed 42 has this many of the trade")
            .0;
        let pos = world
            .dwarves()
            .into_iter()
            .find(|(i, ..)| *i == id)
            .unwrap()
            .1;
        (id, pos)
    }

    #[test]
    fn claim_jobs_takes_fifo_within_a_trade() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let (miner, miner_pos) = dwarf_of(&world, super::Profession::Miner, 0);
        let (hauler, hauler_pos) = dwarf_of(&world, super::Profession::Hauler, 0);
        let dig_target = Pos {
            x: miner_pos.x + 1,
            ..miner_pos
        };
        let stone = free_cell_beside(&world, hauler_pos, &[dig_target, miner_pos]);
        let pile = free_cell_beside(&world, stone, &[dig_target, miner_pos, stone]);
        assert!(world.set_tile(dig_target, Tile::Solid(Material::Stone)));
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(50), stone));
        world.ecs.resource_mut::<super::Zones>().0.insert(pile);
        {
            let mut jobs = world.ecs.resource_mut::<Jobs>();
            // The haul is queued FIRST, so a trade-blind FIFO would hand it to the lowest free id.
            assert!(jobs.insert(Job {
                id: JobId(0),
                kind: JobKind::Haul { item: 50 },
                target: stone,
                created_tick: 0,
                retry_after: 0,
            }));
            assert!(jobs.insert(Job {
                id: JobId(1),
                kind: JobKind::Dig,
                target: dig_target,
                created_tick: 0,
                retry_after: 0,
            }));
        }
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(super::claim_jobs);

        schedule.run(&mut world.ecs);

        for (id, job) in world.claims() {
            let expected = if id == miner {
                Some(JobId(1))
            } else if id == hauler {
                Some(JobId(0))
            } else {
                None
            };
            assert_eq!(job, expected, "dwarf {id:?}");
        }
    }

    #[test]
    fn a_job_with_no_free_dwarf_of_its_trade_gets_no_retry_stamp() {
        let mut world = World::generate(42, Dims::DEFAULT);
        for id in 0..5 {
            set_profession(&mut world, id, super::Profession::Miner);
        }
        let (_, near) = dwarf_of(&world, super::Profession::Miner, 0);
        let stone = free_cell_beside(&world, near, &[]);
        let pile = free_cell_beside(&world, stone, &[stone]);
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(50), stone));
        world.ecs.resource_mut::<super::Zones>().0.insert(pile);
        assert!(world.ecs.resource_mut::<Jobs>().insert(Job {
            id: JobId(0),
            kind: JobKind::Haul { item: 50 },
            target: stone,
            created_tick: 0,
            retry_after: 0,
        }));
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(super::claim_jobs);

        schedule.run(&mut world.ecs);

        assert_eq!(world.jobs()[0].retry_after, 0, "no miner may stamp a haul");
        assert!(world.claims().iter().all(|(_, job)| job.is_none()));

        // Control: with one hauler free, the same job is claimed rather than ignored.
        set_profession(&mut world, 3, super::Profession::Hauler);
        schedule.run(&mut world.ecs);
        assert_eq!(world.claims()[3], (super::Id(3), Some(JobId(0))));
    }

    /// All five dwarves are miners, one per entry of `spots`, in ascending id order.
    fn stand_miners_at(world: &mut World, spots: [Pos; 5]) {
        for (id, spot) in spots.into_iter().enumerate() {
            set_profession(world, id as u32, super::Profession::Miner);
            let entity = world
                .ecs
                .iter_entities()
                .find(|entity| {
                    entity.contains::<super::Dwarf>()
                        && entity.get::<super::Id>().is_some_and(|i| i.0 == id as u32)
                })
                .unwrap()
                .id();
            *world.ecs.get_mut::<Pos>(entity).unwrap() = spot;
        }
    }

    fn insert_dig(world: &mut World, id: u32, target: Pos) {
        assert!(world.ecs.resource_mut::<Jobs>().insert(Job {
            id: JobId(id),
            kind: JobKind::Dig,
            target,
            created_tick: 0,
            retry_after: 0,
        }));
    }

    // #159 (12.4, AC11). Each dwarf stands on his own 11,000-cell plate (a separate component), so
    // five floods cost 55,000 > MAX_ASTAR_NODES in sum while each is under it. 25 unreachable digs
    // (more than RETRY_COOLDOWN, so stamp-and-stop cannot pass) are queued ahead of one reachable
    // dig on plate 4. Replaces `claim_jobs_bounds_aggregate_astar_expansions_per_tick`, whose first
    // assert pinned this very starvation.
    #[test]
    fn a_reachable_job_behind_unreachable_ones_is_claimed_when_areas_sum_past_the_budget() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let dims = world.dims();
        let mut tiles = vec![Tile::Solid(Material::Stone); world.tiles().len()];
        // Plate k is the floor at z = 1 + 2k, stone above and below, so no plate touches another.
        for plate in 0..5_u32 {
            for y in 0..100 {
                for x in 0..110 {
                    tiles[super::worldgen::index(dims, x, y, 1 + 2 * plate)] = Tile::Empty;
                }
            }
        }
        for job in 0..25_u32 {
            let work = Pos {
                x: 3 + 2 * job as i32,
                y: 2,
                z: 20,
            };
            tiles[super::worldgen::index(dims, work.x as u32, work.y as u32, work.z as u32)] =
                Tile::Empty;
        }
        let reachable = Pos { x: 50, y: 50, z: 9 };
        tiles[super::worldgen::index(dims, 50, 50, 9)] = Tile::Solid(Material::Stone);
        world.ecs.resource_mut::<Terrain>().tiles = tiles;
        stand_miners_at(
            &mut world,
            std::array::from_fn(|plate| Pos {
                x: 0,
                y: 0,
                z: 1 + 2 * plate as i32,
            }),
        );
        for job in 0..25_u32 {
            insert_dig(
                &mut world,
                job,
                Pos {
                    x: 2 + 2 * job as i32,
                    y: 2,
                    z: 20,
                },
            );
        }
        insert_dig(&mut world, 25, reachable);
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(super::claim_jobs);

        schedule.run(&mut world.ecs);

        assert_eq!(
            world.claims()[4],
            (super::Id(4), Some(JobId(25))),
            "the plate-4 miner must claim the reachable dig on the first claim tick"
        );
    }

    /// Solid stone except one 55,003-cell walkable area (five plates joined by ramp staircases),
    /// from (0, 0, 1). It floods past `MAX_ASTAR_NODES`, so a search for a goal outside it exhausts.
    fn joined_plates(world: &World) -> Vec<Tile> {
        let dims = world.dims();
        let idx =
            |x: i32, y: i32, z: i32| super::worldgen::index(dims, x as u32, y as u32, z as u32);
        let mut tiles = vec![Tile::Solid(Material::Stone); world.tiles().len()];
        for plate in 0..5_i32 {
            for y in 0..100 {
                for x in 0..110 {
                    tiles[idx(x, y, 1 + 2 * plate)] = Tile::Empty;
                }
            }
        }
        // A ramp staircase at row y = 99 joins plate z to plate z + 2. A move up needs a Ramp under
        // the LOWER cell.
        for k in 0..4_i32 {
            let z = 1 + 2 * k;
            tiles[idx(108, 99, z - 1)] = Tile::Ramp(Material::Stone);
            tiles[idx(109, 99, z + 1)] = Tile::Empty;
            tiles[idx(109, 99, z)] = Tile::Ramp(Material::Stone);
            tiles[idx(110, 99, z + 2)] = Tile::Empty;
        }
        tiles
    }

    // #159 (12.4, AC12). One miner in a 55,003-cell area (five plates joined by ramp staircases),
    // four in sealed one-cell pockets, 10 unreachable digs, then one reachable dig in his area.
    // He exhausts his own budget on one dig per tick, so he reaches the reachable one at tick 110;
    // a job is stamped only when no dwarf of its trade sat it out for budget.
    #[test]
    fn a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let dims = world.dims();
        let idx =
            |x: i32, y: i32, z: i32| super::worldgen::index(dims, x as u32, y as u32, z as u32);
        let mut tiles = joined_plates(&world);
        let pockets = [120, 122, 124, 126].map(|x| Pos { x, y: 120, z: 20 });
        for pocket in pockets {
            tiles[idx(pocket.x, pocket.y, pocket.z)] = Tile::Empty;
        }
        for job in 0..10_i32 {
            tiles[idx(3 + 2 * job, 2, 20)] = Tile::Empty;
        }
        tiles[idx(5, 5, 1)] = Tile::Solid(Material::Stone);
        world.ecs.resource_mut::<Terrain>().tiles = tiles;
        stand_miners_at(
            &mut world,
            [
                Pos { x: 0, y: 0, z: 1 },
                pockets[0],
                pockets[1],
                pockets[2],
                pockets[3],
            ],
        );
        for job in 0..10_u32 {
            insert_dig(
                &mut world,
                job,
                Pos {
                    x: 2 + 2 * job as i32,
                    y: 2,
                    z: 20,
                },
            );
        }
        insert_dig(&mut world, 10, Pos { x: 5, y: 5, z: 1 });
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(super::claim_jobs);

        world.ecs.resource_mut::<super::Tick>().0 = 100;
        schedule.run(&mut world.ecs);
        let stamps: Vec<u64> = world.jobs().iter().map(|job| job.retry_after).collect();
        let mut expected = vec![0; 11];
        expected[0] = 120;
        assert_eq!(
            stamps, expected,
            "only the job he exhausted on is stamped; the jobs he sat out stay unstamped"
        );
        assert!(world.claims()[0].1.is_none());

        for tick in 101..=110_u64 {
            world.ecs.resource_mut::<super::Tick>().0 = tick;
            schedule.run(&mut world.ecs);
            let claimed = world.claims()[0].1;
            if tick < 110 {
                assert_eq!(claimed, None, "tick {tick}: one exhausted dig per tick");
            } else {
                assert_eq!(
                    claimed,
                    Some(JobId(10)),
                    "tick 110 reaches the reachable dig"
                );
            }
        }
    }

    // #159 (12.4, AC12 review). Miner 0 in the joined plates exhausts on dig 0, which sits beside
    // miner 1's sealed pocket: miner 1 must claim it on the same tick (no stamp-and-stop, no leaving
    // the dwarf loop). Haul 1 follows, unreachable for hauler 2 in his own pocket: miner 0's empty
    // budget must not mark it sat out, because the trade filter comes before the budget check.
    // Dig 2 comes after both, beside miner 3's pocket: miner 0 sits it out with no budget left, and
    // miner 3 must still claim it.
    #[test]
    fn a_job_one_dwarf_exhausted_on_goes_to_the_next_of_his_trade_and_others_still_stamp() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let dims = world.dims();
        let idx = |p: Pos| super::worldgen::index(dims, p.x as u32, p.y as u32, p.z as u32);
        let mut tiles = joined_plates(&world);
        let pocket = |x, y| Pos { x, y, z: 20 };
        let (stone, pile) = (pocket(124, 124), pocket(126, 124));
        let spots = [
            Pos { x: 0, y: 0, z: 1 },
            pocket(120, 120),
            pocket(124, 120),
            pocket(126, 120),
            pocket(120, 124),
        ];
        for cell in spots[1..].iter().chain([&stone, &pile]) {
            tiles[idx(*cell)] = Tile::Empty;
        }
        world.ecs.resource_mut::<Terrain>().tiles = tiles;
        stand_miners_at(&mut world, spots);
        set_profession(&mut world, 2, super::Profession::Hauler);
        // Dig 0's only work position is miner 1's pocket.
        insert_dig(&mut world, 0, pocket(121, 120));
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(50), stone));
        world.ecs.resource_mut::<super::Zones>().0.insert(pile);
        assert!(world.ecs.resource_mut::<Jobs>().insert(Job {
            id: JobId(1),
            kind: JobKind::Haul { item: 50 },
            target: stone,
            created_tick: 0,
            retry_after: 0,
        }));
        // Dig 2's only work position is miner 3's pocket.
        insert_dig(&mut world, 2, pocket(127, 120));
        // Precondition: miner 0's search for dig 0 exhausts a fresh budget. Without it no dwarf
        // exhausts, and every assertion below passes vacuously.
        let blocked = super::blocked_cells(world.emitters().iter().map(|(_, pos, _)| pos));
        let mut budget = super::MAX_ASTAR_NODES;
        let (_, exhausted, _) = super::astar_with_budget(
            world.ecs.resource::<Terrain>(),
            &blocked,
            spots[0],
            &BTreeSet::from([spots[1]]),
            &mut budget,
        );
        assert!(exhausted, "miner 0 must exhaust his budget on dig 0");
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(super::claim_jobs);

        schedule.run(&mut world.ecs);

        assert_eq!(
            world.claims()[1],
            (super::Id(1), Some(JobId(0))),
            "miner 1 claims the dig miner 0 exhausted on, on the same tick"
        );
        assert!(world.claims()[0].1.is_none());
        assert_eq!(
            world.claims()[3],
            (super::Id(3), Some(JobId(2))),
            "miner 3 claims the dig miner 0 sat out with no budget left, on the same tick"
        );
        let stamps: Vec<u64> = world.jobs().iter().map(|job| job.retry_after).collect();
        assert_eq!(
            stamps,
            [0, 120, 0],
            "the claimed dig is unstamped; the unreachable haul is stamped despite miner 0's empty budget"
        );
    }

    #[test]
    fn retry_claim_keeps_the_job_and_sets_twenty_tick_cooldown() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let job = Job {
            id: JobId(0),
            kind: JobKind::Dig,
            target: Pos { x: 20, y: 20, z: 8 },
            created_tick: 0,
            retry_after: 0,
        };
        assert!(world.ecs.resource_mut::<Jobs>().insert(job));
        world.ecs.resource_mut::<super::Tick>().0 = 7;
        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("dwarf zero exists")
            .id();
        world.ecs.get_mut::<super::CurrentJob>(entity).unwrap().0 = Some(job.id);
        world
            .ecs
            .entity_mut(entity)
            .insert((super::Path(Vec::new()), super::WorkProgress(2)));

        super::retry_claim(&mut world.ecs, entity, job.id);

        assert_eq!(world.jobs()[0].retry_after, 27);
        assert_eq!(world.claims()[0].1, None);
        assert!(!world.ecs.entity(entity).contains::<super::Path>());
        assert!(!world.ecs.entity(entity).contains::<super::WorkProgress>());
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

        assert_eq!(
            super::astar(&terrain, &NO_FIRE, from, &goals),
            Some(expected.clone())
        );
        assert_eq!(
            super::astar(&terrain, &NO_FIRE, from, &goals),
            Some(expected)
        );
    }

    /// Manual resolution instrument for story 10.6.  It uses the existing private A* directly,
    /// without making pathfinding a public benchmark API or changing its behaviour.
    #[test]
    #[ignore = "manual resolution-bench measurement; run with --ignored --nocapture"]
    fn resolution_bench_times_existing_astar_on_subdivided_flat_grids() {
        for (k, edge, expected_found) in [(1, 128, true), (2, 256, false), (4, 512, false)] {
            let terrain = flat_terrain(edge, edge);
            let from = Pos { x: 0, y: 0, z: 1 };
            let goal = Pos {
                x: edge as i32 - 1,
                y: edge as i32 - 1,
                z: 1,
            };
            let started = Instant::now();
            let path = super::astar(&terrain, &NO_FIRE, from, &BTreeSet::from([goal]));
            println!(
                "resolution-astar sim_k={k} edge={edge} path_found={} elapsed_seconds={:.6}",
                path.is_some(),
                started.elapsed().as_secs_f64(),
            );
            // The simulated dwarf's existing aggregate node budget stops this diagonal query at
            // k=2 and k=4; that is the cost finding, not a reason to weaken the benchmark grid.
            assert_eq!(
                path.is_some(),
                expected_found,
                "sim_k={k} node-budget result drifted"
            );
        }
    }

    #[test]
    fn astar_routes_a_dwarf_around_a_tree_trunk() {
        let mut terrain = flat_terrain(5, 3);
        let trunk = Pos { x: 2, y: 1, z: 1 };
        terrain.tiles[super::worldgen::index(terrain.dims, 2, 1, 1)] =
            Tile::Solid(Material::TreeTrunk);
        let from = Pos { x: 0, y: 1, z: 1 };
        let goal = Pos { x: 4, y: 1, z: 1 };

        let path = super::astar(&terrain, &NO_FIRE, from, &BTreeSet::from([goal]))
            .expect("dwarf can walk around a tree");

        assert_eq!(path.last(), Some(&goal));
        assert!(!path.contains(&trunk));
        assert_eq!(path.len(), 6, "tree must force a two-step detour");
    }

    #[test]
    fn astar_ties_break_on_position_not_insertion_order() {
        let terrain = flat_terrain(3, 3);
        let from = Pos { x: 0, y: 0, z: 1 };
        let goal = Pos { x: 2, y: 2, z: 1 };

        assert_eq!(
            super::astar(&terrain, &NO_FIRE, from, &BTreeSet::from([goal])),
            Some(vec![
                Pos { x: 0, y: 1, z: 1 },
                Pos { x: 0, y: 2, z: 1 },
                Pos { x: 1, y: 2, z: 1 },
                Pos { x: 2, y: 2, z: 1 },
            ])
        );
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
            super::astar(&terrain, &NO_FIRE, lower, &BTreeSet::from([higher])),
            Some(vec![higher])
        );
        terrain.tiles[super::worldgen::index(dims, 0, 0, 0)] = Tile::Solid(Material::Stone);
        assert_eq!(
            super::astar(&terrain, &NO_FIRE, lower, &BTreeSet::from([higher])),
            None
        );
    }

    #[test]
    fn astar_prefers_the_shorter_ramp_route_over_a_flat_detour() {
        let dims = Dims { x: 4, y: 3, z: 5 };
        let mut terrain = Terrain {
            dims,
            tiles: vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize],
            dirty: BTreeSet::new(),
        };
        let heights = [
            ((0, 0), 3),
            ((0, 1), 3),
            ((0, 2), 3),
            ((1, 0), 3),
            ((1, 1), 1),
            ((1, 2), 2),
            ((2, 0), 3),
            ((2, 1), 2),
            ((2, 2), 1),
            ((3, 0), 2),
            ((3, 1), 1),
            ((3, 2), 3),
        ];
        let ramps = BTreeSet::from([
            (0, 0),
            (0, 2),
            (1, 0),
            (1, 2),
            (2, 0),
            (2, 1),
            (2, 2),
            (3, 0),
            (3, 1),
            (3, 2),
        ]);
        for ((x, y), z) in heights {
            terrain.tiles[super::worldgen::index(dims, x, y, z - 1)] = if ramps.contains(&(x, y)) {
                Tile::Ramp(Material::Stone)
            } else {
                Tile::Solid(Material::Stone)
            };
        }
        let goal = Pos { x: 2, y: 0, z: 3 };

        assert_eq!(
            super::astar(
                &terrain,
                &NO_FIRE,
                Pos { x: 1, y: 2, z: 2 },
                &BTreeSet::from([goal]),
            ),
            Some(vec![
                Pos { x: 2, y: 2, z: 1 },
                Pos { x: 2, y: 1, z: 2 },
                goal,
            ])
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
                &NO_FIRE,
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
            super::astar_neighbours(&terrain, &NO_FIRE, center),
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
        // 224x224 = 50,176 standable positions against the 50,000 cap. The margin is TIGHT ON
        // PURPOSE and must stay that way: it has to sit BETWEEN the real cap and the smallest
        // widening the mutation set probes (`MAX_ASTAR_NODES is widened`, 50_000 -> 60_000). At
        // 50,176 a widened cap swallows the whole grid, the search succeeds, and this assertion
        // fails — which is how that mutation is killed. Widening the grid to "make the margin
        // safer" (tried at 3.2's review, 320x320) exhausts the budget under BOTH the real and the
        // widened cap, so the test passes either way and the mutation SURVIVES. Downward movement
        // of the constant is pinned by `astar_finds_a_path_well_inside_the_node_cap` below and by
        // `claim_jobs_bounds_aggregate_astar_expansions_per_tick`, not by this test.
        let terrain = flat_terrain(224, 224);

        assert_eq!(
            super::astar(
                &terrain,
                &NO_FIRE,
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
    fn astar_finds_a_path_well_inside_the_node_cap() {
        // The other direction, which the cap test alone cannot give: a search that SHOULD
        // succeed still does. Without this, lowering MAX_ASTAR_NODES to 1 leaves the cap test
        // green — it only ever asserts `None`.
        let terrain = flat_terrain(40, 40);

        let path = super::astar(
            &terrain,
            &NO_FIRE,
            Pos { x: 0, y: 0, z: 1 },
            &BTreeSet::from([Pos { x: 5, y: 5, z: 1 }]),
        )
        .expect("a 10-step goal on open ground is far inside the 50,000-node budget");
        assert_eq!(path.len(), 10, "shortest path is |dx| + |dy|");
    }

    #[test]
    fn execute_jobs_walks_then_digs_for_exactly_dig_work_ticks() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let start = world.dwarves()[0].1;
        let dx = if start.x + 2 < world.dims().x as i32 {
            1
        } else {
            -1
        };
        let work = Pos {
            x: start.x + dx,
            ..start
        };
        let target = Pos {
            x: start.x + 2 * dx,
            ..start
        };
        assert!(world.set_tile(
            Pos {
                z: work.z - 1,
                ..work
            },
            Tile::Solid(Material::Stone),
        ));
        assert!(world.set_tile(work, Tile::Empty));
        assert!(world.set_tile(target, Tile::Solid(Material::Stone)));
        world.drain_dirty();
        let job = Job {
            id: JobId(0),
            kind: JobKind::Dig,
            target,
            created_tick: 0,
            retry_after: 0,
        };
        assert!(world.ecs.resource_mut::<Jobs>().insert(job));
        world
            .ecs
            .resource_mut::<super::Designations>()
            .0
            .insert(target, super::DesignationKind::Dig);
        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("dwarf zero exists")
            .id();
        world.ecs.get_mut::<super::CurrentJob>(entity).unwrap().0 = Some(job.id);
        world
            .ecs
            .entity_mut(entity)
            .insert((super::Path(Vec::new()), super::WorkProgress(0)));

        super::execute_jobs(&mut world.ecs);
        assert_eq!(world.dwarves()[0].1, work);
        assert_eq!(world.dwarves()[0].2, JobState::Walk);
        // A literal, not `DIG_WORK_TICKS`: Wolf ruled ten swings at the client's five ticks each
        // (12.5 seat), and a loop bound by the constant would follow any change to it.
        for _ in 0..50 {
            super::execute_jobs(&mut world.ecs);
            assert_eq!(world.dwarves()[0].2, JobState::Work);
            assert_eq!(world.claims()[0].1, Some(JobId(0)));
        }
        super::execute_jobs(&mut world.ecs);

        assert_eq!(world.dwarves()[0].2, JobState::Idle);
        assert_eq!(world.claims()[0].1, None);
        assert!(world.jobs().is_empty());
        assert!(world.designations().is_empty());
        assert_eq!(world.tile(target), Some(Tile::Empty));
        assert_eq!(world.items(), vec![(super::Id(10), target)]);
        assert_eq!(world.drain_dirty(), vec![(target, Tile::Empty)]);
    }

    #[test]
    fn a_dig_mark_never_lands_on_a_tree_tile() {
        for material in [Material::TreeTrunk, Material::TreeFoliage] {
            let mut world = World::generate(42, Dims::DEFAULT);
            let work = world.dwarves()[0].1;
            let target = Pos {
                x: work.x + 1,
                ..work
            };
            assert!(world.set_tile(target, Tile::Solid(material)));
            let rect = super::Rect {
                min: target,
                max: target,
            };
            let kind = super::DesignationKind::Dig;
            assert_eq!(
                world.apply_command(super::SimCommand::Designate { kind, rect }),
                Some(super::Refusal::Designate { kind, rect }),
                "dig over {material:?}"
            );
            assert!(world.designations().is_empty(), "dig over {material:?}");
            // Stone beside it still takes the mark.
            assert!(world.set_tile(target, Tile::Solid(Material::Stone)));
            assert_eq!(
                world.apply_command(super::SimCommand::Designate { kind, rect }),
                None
            );
            assert_eq!(world.designations(), vec![(target, kind)]);
        }
    }

    #[test]
    fn a_channel_mark_never_lands_on_a_cell_standing_on_a_tree_tile() {
        for material in [Material::TreeTrunk, Material::TreeFoliage] {
            let mut world = World::generate(42, Dims::DEFAULT);
            let target = world.dwarves()[0].1;
            let below = Pos {
                z: target.z - 1,
                ..target
            };
            assert!(world.set_tile(below, Tile::Solid(material)));
            assert!(world.set_tile(target, Tile::Empty));
            let rect = super::Rect {
                min: target,
                max: target,
            };
            let kind = super::DesignationKind::Channel;
            assert_eq!(
                world.apply_command(super::SimCommand::Designate { kind, rect }),
                Some(super::Refusal::Designate { kind, rect }),
                "channel over {material:?}"
            );
            assert!(world.designations().is_empty(), "channel over {material:?}");
            assert!(world.set_tile(below, Tile::Solid(Material::Stone)));
            assert_eq!(
                world.apply_command(super::SimCommand::Designate { kind, rect }),
                None
            );
            assert_eq!(world.designations(), vec![(target, kind)]);
        }
    }

    #[test]
    fn execute_jobs_channels_a_material_preserving_ramp_and_spawns_stone() {
        let mut world = World::generate(42, Dims::DEFAULT);
        // 12.9 AC13: the miner works from the next tile, so the target is the cell beside him.
        let stand = world.dwarves()[0].1;
        let target = Pos {
            x: stand.x + 1,
            ..stand
        };
        let below = Pos {
            z: target.z - 1,
            ..target
        };
        assert!(world.set_tile(below, Tile::Solid(Material::Soil)));
        assert!(world.set_tile(target, Tile::Empty));
        world.drain_dirty();
        let job = Job {
            id: JobId(0),
            kind: JobKind::Channel,
            target,
            created_tick: 0,
            retry_after: 0,
        };
        assert!(world.ecs.resource_mut::<Jobs>().insert(job));
        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("dwarf zero exists")
            .id();
        world.ecs.get_mut::<super::CurrentJob>(entity).unwrap().0 = Some(job.id);
        world
            .ecs
            .entity_mut(entity)
            .insert((super::Path(Vec::new()), super::WorkProgress(0)));

        for _ in 0..super::DIG_WORK_TICKS {
            super::execute_jobs(&mut world.ecs);
            assert_eq!(world.dwarves()[0].2, JobState::Work);
            assert_eq!(world.claims()[0].1, Some(JobId(0)));
        }
        super::execute_jobs(&mut world.ecs);

        assert_eq!(world.tile(below), Some(Tile::Ramp(Material::Soil)));
        assert_eq!(world.items(), vec![(super::Id(10), target)]);
        assert_eq!(
            world.drain_dirty(),
            vec![(below, Tile::Ramp(Material::Soil))]
        );
    }

    #[test]
    fn execute_jobs_removes_a_channel_job_when_the_support_is_already_a_ramp() {
        let mut world = World::generate(42, Dims::DEFAULT);
        // 12.9 AC13: the miner works from the next tile, so the target is the cell beside him.
        let stand = world.dwarves()[0].1;
        let target = Pos {
            x: stand.x + 1,
            ..stand
        };
        let below = Pos {
            z: target.z - 1,
            ..target
        };
        assert!(world.set_tile(below, Tile::Ramp(Material::Soil)));
        assert!(world.set_tile(target, Tile::Empty));
        world.drain_dirty();
        let job = Job {
            id: JobId(0),
            kind: JobKind::Channel,
            target,
            created_tick: 0,
            retry_after: 0,
        };
        assert!(world.ecs.resource_mut::<Jobs>().insert(job));
        world
            .ecs
            .resource_mut::<super::Designations>()
            .0
            .insert(target, super::DesignationKind::Channel);
        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("dwarf zero exists")
            .id();
        world.ecs.get_mut::<super::CurrentJob>(entity).unwrap().0 = Some(job.id);
        world.ecs.entity_mut(entity).insert((
            super::Path(Vec::new()),
            super::WorkProgress(super::DIG_WORK_TICKS),
        ));

        super::execute_jobs(&mut world.ecs);

        assert!(world.jobs().is_empty());
        assert!(world.designations().is_empty());
        assert_eq!(world.claims()[0].1, None);
        assert_eq!(world.dwarves()[0].2, JobState::Idle);
        assert!(world.items().is_empty());
        assert_eq!(world.tile(below), Some(Tile::Ramp(Material::Soil)));
    }

    #[test]
    fn settle_moves_one_level_down_and_discards_the_path() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let start = world.dwarves()[0].1;
        let below = Pos {
            z: start.z - 1,
            ..start
        };
        assert!(world.set_tile(start, Tile::Empty));
        assert!(world.set_tile(below, Tile::Empty));
        assert!(world.set_tile(
            Pos {
                z: start.z - 2,
                ..start
            },
            Tile::Solid(Material::Stone),
        ));
        let entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("dwarf zero exists")
            .id();
        world
            .ecs
            .entity_mut(entity)
            .insert(super::Path(vec![start]));

        super::settle(&mut world.ecs);

        assert_eq!(world.dwarves()[0].1, below);
        assert!(!world.ecs.entity(entity).contains::<super::Path>());
    }

    #[test]
    fn settle_descends_one_level_per_tick_through_a_deep_empty_shaft() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let start = world.dwarves()[0].1;
        let first = Pos {
            z: start.z - 1,
            ..start
        };
        let second = Pos {
            z: start.z - 2,
            ..start
        };
        let floor = Pos {
            z: start.z - 3,
            ..start
        };
        for pos in [start, first, second] {
            assert!(world.set_tile(pos, Tile::Empty));
        }
        assert!(world.set_tile(floor, Tile::Solid(Material::Stone)));

        super::settle(&mut world.ecs);
        assert_eq!(world.dwarves()[0].1, first);
        super::settle(&mut world.ecs);
        assert_eq!(world.dwarves()[0].1, second);
        super::settle(&mut world.ecs);
        assert_eq!(world.dwarves()[0].1, second);
    }

    #[test]
    fn claimed_dwarf_settles_before_moving_from_newly_unsupported_ground() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let worker = Pos { x: 10, y: 10, z: 1 };
        let dug_floor = Pos { x: 11, y: 10, z: 1 };
        let victim = Pos { x: 11, y: 10, z: 2 };
        let escape = Pos { x: 11, y: 11, z: 2 };
        let second_target = Pos { x: 12, y: 10, z: 2 };
        // 12.9 Task 10 (#162): the dig's stone spawns in `dug_floor` and an item is not landed on,
        // so the faller rests on the nearest free tile instead. This one is beside `dug_floor`.
        let landing = Pos { x: 11, y: 9, z: 1 };
        for (pos, tile) in [
            (Pos { z: 0, ..landing }, Tile::Solid(Material::Stone)),
            (landing, Tile::Empty),
            (Pos { z: 0, ..worker }, Tile::Solid(Material::Stone)),
            (worker, Tile::Empty),
            (Pos { z: 0, ..dug_floor }, Tile::Solid(Material::Stone)),
            (dug_floor, Tile::Solid(Material::Stone)),
            (victim, Tile::Empty),
            (Pos { z: 1, ..escape }, Tile::Solid(Material::Stone)),
            (escape, Tile::Empty),
            (second_target, Tile::Solid(Material::Stone)),
        ] {
            assert!(world.set_tile(pos, tile));
        }
        let worker_entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("worker exists")
            .id();
        let victim_entity = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(1)))
            .expect("victim exists")
            .id();
        *world.ecs.get_mut::<Pos>(worker_entity).unwrap() = worker;
        *world.ecs.get_mut::<Pos>(victim_entity).unwrap() = victim;
        let first_job = Job {
            id: JobId(0),
            kind: JobKind::Dig,
            target: dug_floor,
            created_tick: 0,
            retry_after: 0,
        };
        let second_job = Job {
            id: JobId(1),
            kind: JobKind::Dig,
            target: second_target,
            created_tick: 0,
            retry_after: 0,
        };
        assert!(world.ecs.resource_mut::<Jobs>().insert(first_job));
        assert!(world.ecs.resource_mut::<Jobs>().insert(second_job));
        world
            .ecs
            .get_mut::<super::CurrentJob>(worker_entity)
            .unwrap()
            .0 = Some(first_job.id);
        world
            .ecs
            .get_mut::<super::CurrentJob>(victim_entity)
            .unwrap()
            .0 = Some(second_job.id);
        world.ecs.entity_mut(worker_entity).insert((
            super::Path(Vec::new()),
            super::WorkProgress(super::DIG_WORK_TICKS),
        ));
        world
            .ecs
            .entity_mut(victim_entity)
            .insert((super::Path(vec![escape]), super::WorkProgress(0)));

        super::execute_jobs(&mut world.ecs);
        super::settle(&mut world.ecs);

        assert_eq!(*world.ecs.get::<Pos>(victim_entity).unwrap(), landing);
        assert_eq!(
            *world.ecs.get::<JobState>(victim_entity).unwrap(),
            JobState::Walk,
            "a dwarf holding a job is never reported idle while falling"
        );
        assert!(!world.ecs.entity(victim_entity).contains::<super::Path>());
        assert_eq!(world.claims()[1].1, Some(JobId(1)));
    }

    #[test]
    fn a_holder_whose_every_work_position_is_walled_off_mid_walk_lets_go_in_one_step() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let target = Pos { x: 15, y: 10, z: 1 };
        let only_work_position = Pos { x: 14, y: 10, z: 1 };
        for x in 10..=14 {
            for (pos, tile) in [
                (Pos { x, y: 10, z: 0 }, Tile::Solid(Material::Stone)),
                (Pos { x, y: 10, z: 1 }, Tile::Empty),
            ] {
                assert!(world.set_tile(pos, tile));
            }
        }
        assert!(world.set_tile(Pos { z: 0, ..target }, Tile::Solid(Material::Stone)));
        assert!(world.set_tile(target, Tile::Solid(Material::Stone)));
        for (dx, dy) in [(1, 0), (0, -1), (0, 1)] {
            assert!(world.set_tile(
                Pos {
                    x: target.x + dx,
                    y: target.y + dy,
                    z: target.z,
                },
                Tile::Solid(Material::Stone),
            ));
        }
        let holder = world
            .ecs
            .iter_entities()
            .find(|entity| entity.get::<super::Id>() == Some(&super::Id(0)))
            .expect("holder exists")
            .id();
        *world.ecs.get_mut::<Pos>(holder).unwrap() = Pos { x: 11, y: 10, z: 1 };
        let job = Job {
            id: JobId(0),
            kind: JobKind::Dig,
            target,
            created_tick: 0,
            retry_after: 0,
        };
        assert!(world.ecs.resource_mut::<Jobs>().insert(job));
        world
            .ecs
            .resource_mut::<super::Designations>()
            .0
            .insert(target, super::DesignationKind::Dig);
        world.ecs.get_mut::<super::CurrentJob>(holder).unwrap().0 = Some(job.id);
        world.ecs.entity_mut(holder).insert((
            super::Path(vec![
                Pos { x: 12, y: 10, z: 1 },
                Pos { x: 13, y: 10, z: 1 },
                only_work_position,
            ]),
            super::WorkProgress(0),
        ));

        // The terrain change and the path wipe `execute_jobs` does after any sim dig.
        assert!(
            world
                .ecs
                .resource_mut::<Terrain>()
                .set_tile(only_work_position, Tile::Solid(Material::Stone))
        );
        super::clear_paths(&mut world.ecs);
        world.step();

        assert_eq!(world.claims()[0].1, None);
        let jobs = world.jobs();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].retry_after, world.tick() + 20);
        assert_eq!(
            world.designations(),
            vec![(target, super::DesignationKind::Dig)]
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

        assert!(
            before
                .iter()
                .all(|(_, _, state, _)| *state == JobState::Idle)
        );
        world.step();
        let after = world.dwarves();

        assert_ne!(after[0].1, before[0].1);
        assert_eq!(after[0].2, JobState::Walk);
        for index in 1..5 {
            assert_eq!(after[index].1, before[index].1);
            assert_eq!(after[index].2, JobState::Idle);
        }
    }

    /// Issue #74. A dwarf must never stand in a fire — the OBSERVABLE outcome, checked against the
    /// running sim rather than against either movement path's internals.
    ///
    /// WHY IT RUNS THE WORLD AND NOT `astar_neighbours`. There are two writers of a dwarf's
    /// position — job routing and `wander` — and a unit test of either would pass while the other
    /// walked dwarves through the campfire. That is the shape this project shipped before, where a
    /// draw offset applied at the spawn was rewritten by the blend on the very next frame. Only
    /// ticking the world exercises both.
    ///
    /// It also pins the SPAWN, because `spawn_dwarves` excluding emitters was already true and is
    /// the half that would silently carry the assertion if movement regressed on tick one.
    #[test]
    fn a_dwarf_never_stands_in_a_fire() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let fires: BTreeSet<Pos> = world
            .emitters()
            .into_iter()
            .map(|(_, pos, _)| pos)
            .collect();
        assert!(
            fires.len() >= 5,
            "the camp must actually have emitters, or this test asserts nothing: {fires:?}"
        );

        for (_, pos, _, _) in world.dwarves() {
            assert!(
                !fires.contains(&pos),
                "a dwarf SPAWNED in a fire at {pos:?}"
            );
        }

        for tick in 0..200 {
            world.step();
            for (id, pos, _, _) in world.dwarves() {
                assert!(
                    !fires.contains(&pos),
                    "dwarf {id:?} walked into a fire at {pos:?} on tick {tick}"
                );
            }
        }
    }

    /// WANDER_RADIUS had NO guard at all. The test that pinned it was deleted at some point and
    /// nothing replaced it, so widening the radius would have changed how the fortress behaves
    /// with every suite still green. Found 2026-08-22 by the mutation-table audit: the row
    /// "wander radius widens from three to six" named a test that no longer existed, and
    /// `mutate.sh` reports a missing test as SURVIVED rather than as an error — so the hole read
    /// as a weak test rather than as an absent one.
    #[test]
    fn dwarves_stay_standable_and_near_home() {
        let mut world = World::generate(42, Dims::DEFAULT);
        // Every dwarf's `home` is the CAMP ORIGIN, not its own spawn tile -- `spawn_dwarves`
        // scatters them over the standable tiles around the camp and gives them all the same
        // home. A dwarf spawned outside the radius is simply stuck: the move filter offers it no
        // candidates, so it idles where it is. The invariant is therefore about MOVEMENT -- a
        // dwarf never steps to a tile outside the radius -- not about where it starts.
        let camp = world.camp_origin();
        let spawned: Vec<Pos> = world
            .dwarves()
            .into_iter()
            .map(|(_, pos, _, _)| pos)
            .collect();

        for _ in 0..200 {
            world.step();
            for ((_, pos, _, _), start) in world.dwarves().into_iter().zip(&spawned) {
                assert!(
                    pos == *start
                        || ((pos.x - camp.x).abs() <= super::WANDER_RADIUS
                            && (pos.y - camp.y).abs() <= super::WANDER_RADIUS),
                    "dwarf moved from {start:?} to {pos:?}, outside WANDER_RADIUS {} of camp \
                     {camp:?}",
                    super::WANDER_RADIUS
                );
                assert_eq!(
                    world.tile(pos),
                    Some(Tile::Empty),
                    "a dwarf must stand in an empty tile, not inside terrain"
                );
                let below = Pos {
                    z: pos.z - 1,
                    ..pos
                };
                assert!(
                    matches!(world.tile(below), Some(Tile::Solid(_))),
                    "a dwarf at {pos:?} must have solid ground beneath it, found {:?}",
                    world.tile(below)
                );
            }
        }
    }

    // ---- 12.9 fixtures: a solid world with a room, a one-wide tunnel and a cave carved into it ----

    /// The walking level of the 12.9 fixtures. The floor at `FIX_Z - 1` is solid everywhere.
    const FIX_Z: i32 = 5;
    /// The tunnel row.
    const FIX_Y: i32 = 20;

    fn at(x: i32, y: i32, z: i32) -> Pos {
        Pos { x, y, z }
    }

    /// All stone, nothing carved: the caller digs the fixture out of it.
    fn solid_world() -> World {
        let mut world = World::generate(42, Dims::DEFAULT);
        let len = world.tiles().len();
        world.ecs.resource_mut::<Terrain>().tiles = vec![Tile::Solid(Material::Stone); len];
        world
    }

    fn carve(world: &mut World, cells: impl IntoIterator<Item = Pos>) {
        for cell in cells {
            assert!(
                world.set_tile(cell, Tile::Empty),
                "{cell:?} was already empty"
            );
        }
    }

    /// Dwarves 0 and 1 stand on `a` and `b`; dwarves 2..5 are parked in sealed one-cell closets
    /// far from home, so they never wander (outside WANDER_RADIUS) and never reach a job. Only
    /// `miners` are miners; everyone else is a woodcutter, so they claim nothing here.
    fn crew_at(world: &mut World, a: Pos, b: Pos, miners: &[u32]) {
        let closets = [at(40, 40, FIX_Z), at(42, 40, FIX_Z), at(44, 40, FIX_Z)];
        carve(world, closets);
        stand_miners_at(world, [a, b, closets[0], closets[1], closets[2]]);
        for id in 0..5 {
            if !miners.contains(&id) {
                set_profession(world, id, super::Profession::Woodcutter);
            }
        }
    }

    /// Room x 6..=10, y 17..=23; tunnel x 11..=16 on the tunnel row, one wide; rock at x 17.
    fn room_and_tunnel() -> World {
        let mut world = solid_world();
        let room = (6..=10).flat_map(|x| (FIX_Y - 3..=FIX_Y + 3).map(move |y| at(x, y, FIX_Z)));
        carve(&mut world, room);
        carve(&mut world, (11..=16).map(|x| at(x, FIX_Y, FIX_Z)));
        world
    }

    /// A dig whose only work position is the tunnel's last cell.
    const EAST_END: Pos = Pos {
        x: 17,
        y: FIX_Y,
        z: FIX_Z,
    };
    /// A dig whose only work position is in the room, (8, 23).
    const ROOM_WALL: Pos = Pos {
        x: 8,
        y: FIX_Y + 4,
        z: FIX_Z,
    };

    /// Steps until `done` or `limit` ticks, panicking on the tick two dwarves share a tile or
    /// swap tiles. Calls `each_tick` after every step. Returns every tick's positions by id.
    fn watch(
        world: &mut World,
        limit: u64,
        mut each_tick: impl FnMut(&World),
        done: impl Fn(&World) -> bool,
    ) -> Vec<Vec<Pos>> {
        let positions = |world: &World| -> Vec<Pos> {
            world.dwarves().iter().map(|(_, pos, ..)| *pos).collect()
        };
        let mut trace = vec![positions(world)];
        while !done(world) {
            assert!(world.tick() < limit, "not done by tick {limit}");
            world.step();
            let now = positions(world);
            let before = trace.last().unwrap();
            let tick = world.tick();
            assert_eq!(
                now.iter().collect::<BTreeSet<_>>().len(),
                now.len(),
                "two dwarves share a tile at tick {tick}: {now:?}"
            );
            for a in 0..now.len() {
                for b in 0..now.len() {
                    assert!(
                        a == b || !(now[a] == before[b] && now[b] == before[a] && now[a] != now[b]),
                        "dwarves {a} and {b} swapped tiles at tick {tick}: {before:?} -> {now:?}"
                    );
                }
            }
            each_tick(world);
            trace.push(now);
        }
        trace
    }

    /// Both jobs stay claimed by their own miner until they complete, and neither is ever stamped
    /// with a retry: nobody was released.
    fn assert_nobody_released(world: &World) {
        for job in world.jobs() {
            assert_eq!(job.retry_after, 0, "job {:?} was released", job.id);
            let holder = world.claims()[job.id.0 as usize].1;
            assert_eq!(holder, Some(job.id), "dwarf {} lost its job", job.id.0);
        }
    }

    /// Two miners meet head-on in the tunnel: the first (id 0) to `first_target`, the second
    /// (id 1) to `second_target`, each starting on its own cell.
    fn head_on(first: (Pos, Pos), second: (Pos, Pos)) -> (World, Vec<Vec<Pos>>) {
        let mut world = room_and_tunnel();
        crew_at(&mut world, first.0, second.0, &[0, 1]);
        insert_dig(&mut world, 0, first.1);
        insert_dig(&mut world, 1, second.1);
        // Past every reaction delay, so both claim on the first step.
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        let trace = watch(&mut world, 1_000, assert_nobody_released, |world| {
            world.jobs().is_empty()
        });
        (world, trace)
    }

    // 12.9 AC3(a): the LOWER id is nearer the open end. It has an escape (the room), so it
    // yields; the other walks out of the tunnel's way, then the lower id goes back in.
    #[test]
    fn head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_open_end() {
        let (world, trace) = head_on(
            (at(12, FIX_Y, FIX_Z), EAST_END),
            (at(14, FIX_Y, FIX_Z), ROOM_WALL),
        );
        assert_eq!(world.tile(EAST_END), Some(Tile::Empty));
        assert_eq!(world.tile(ROOM_WALL), Some(Tile::Empty));
        // Positive: the lower id really did back off (it started at x = 12 and its work lies east).
        assert!(
            trace.iter().any(|positions| positions[0].x < 12),
            "the lower id never backed out of the tunnel"
        );
    }

    // 12.9 AC3(b): the LOWER id is nearer the dead end and the higher id's work position is the
    // tunnel's last cell. The lower has no escape, so the higher must back off. A per-step flip
    // rule livelocks here (the story's hand trace); this must resolve.
    #[test]
    fn head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_dead_end() {
        let (world, trace) = head_on(
            (at(14, FIX_Y, FIX_Z), ROOM_WALL),
            (at(12, FIX_Y, FIX_Z), EAST_END),
        );
        assert_eq!(world.tile(EAST_END), Some(Tile::Empty));
        assert_eq!(world.tile(ROOM_WALL), Some(Tile::Empty));
        // Positive: the higher id (its work is east) backed out west of where it started.
        assert!(
            trace.iter().any(|positions| positions[1].x < 12),
            "the higher id never backed out of the tunnel"
        );
    }

    // 12.9 review #183: a head-on where neither side has an escape. The idle blocker's exit `Path`
    // still starts on the holder's tile, so it would keep the head-on forever. Both wait, and the
    // blocker's stale `Path` is dropped so the next blocked step derives a fresh one.
    // NOTE: the review's five-dwarf chain fixture still livelocks with this fix (filed apart).
    #[test]
    fn a_head_on_with_no_escape_drops_the_idle_blockers_stale_path() {
        let mut world = room_and_tunnel();
        let spots = [16, 15, 14, 13, 12].map(|x| at(x, FIX_Y, FIX_Z));
        stand_miners_at(&mut world, spots);
        set_profession(&mut world, 1, super::Profession::Woodcutter);
        let (holder, blocker) = (spots[0], spots[1]);
        let entity = dwarf_entity(&world, 1);
        assert_eq!(world.ecs.get::<super::CurrentJob>(entity).unwrap().0, None);
        world
            .ecs
            .entity_mut(entity)
            .insert(super::Path(vec![holder]));
        let mut occupied: BTreeSet<Pos> = spots.into_iter().collect();
        let mut path = vec![blocker];

        let step = super::resolve_blocked_step(
            &mut world.ecs,
            &BTreeSet::new(),
            &mut occupied,
            (super::Id(0), holder),
            &mut path,
            &BTreeSet::new(),
        );

        // Both wait: nobody moved and the holder kept its path.
        assert!(!step);
        assert_eq!(path, vec![blocker]);
        assert_eq!(occupied, spots.into_iter().collect());
        assert_eq!(*world.ecs.get::<Pos>(entity).unwrap(), blocker);
        assert!(
            world.ecs.get::<super::Path>(entity).is_none(),
            "the idle blocker kept its stale exit path"
        );
    }

    // 12.9 AC4: a dwarf falls onto a tile another dwarf stands on (an overhang dug over a cave).
    // It comes to rest on a free standable tile within 3 ticks of the support's removal.
    #[test]
    fn a_dwarf_that_falls_onto_another_comes_to_rest_on_a_free_tile() {
        let mut world = solid_world();
        let (cave, support, ledge) = (at(10, 10, 2), at(10, 10, 3), at(10, 10, 4));
        carve(&mut world, [at(9, 10, 2), cave, at(11, 10, 2), ledge]);
        crew_at(&mut world, ledge, cave, &[]);
        assert!(world.ecs.resource::<Terrain>().is_standable(ledge));
        assert!(world.set_tile(support, Tile::Empty));
        let removed_at = world.tick();

        let trace = watch(
            &mut world,
            removed_at + 3,
            |_| {},
            |world| world.dwarves()[0].1.z == 2,
        );
        let landed = world.dwarves()[0].1;
        assert!(world.tick() <= removed_at + 3);
        assert_ne!(landed, cave, "the faller landed on the dwarf under it");
        assert!(world.ecs.resource::<Terrain>().is_standable(landed));
        assert_eq!(
            world.dwarves()[1].1,
            cave,
            "the dwarf in the cave was moved"
        );
        // Positive: it did fall, through the hole, rather than being left hanging.
        assert!(trace.iter().any(|positions| positions[0] == support));
    }

    // 12.9 AC5 (amended, Task 0 Q1b): an idle dwarf stands on the only work position of a dig at
    // the end of the tunnel; the miner is in the tunnel between it and the open end. NO swap: the
    // miner backs out, the idle dwarf follows it out on an exit path, and the dig completes.
    #[test]
    fn an_idle_dwarf_on_the_dig_face_follows_the_miner_out_and_the_dig_completes() {
        let mut world = room_and_tunnel();
        let (miner_start, idle_start) = (at(13, FIX_Y, FIX_Z), at(16, FIX_Y, FIX_Z));
        crew_at(&mut world, miner_start, idle_start, &[0]);
        insert_dig(&mut world, 0, EAST_END);
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        let idle = dwarf_entity(&world, 1);

        let miner = dwarf_entity(&world, 0);
        let mut had_path = false;
        let mut last_home = at(-1, -1, -1);
        // Each time the idle dwarf is given a new home: where it stood, whether it still had a
        // `Path`, and the tiles the miner still had to walk through.
        let mut relocations = Vec::new();
        let trace = watch(
            &mut world,
            1_200,
            |world| {
                had_path |= world.ecs.get::<super::Path>(idle).is_some();
                let home = world.ecs.get::<super::Wander>(idle).unwrap().home;
                if home != last_home {
                    last_home = home;
                    let mut route: Vec<Pos> = world
                        .ecs
                        .get::<super::Path>(miner)
                        .map(|path| path.0.clone())
                        .unwrap_or_default();
                    route.push(world.dwarves()[0].1);
                    let still_walking = world.ecs.get::<super::Path>(idle).is_some();
                    relocations.push((world.dwarves()[1].1, home, still_walking, route));
                }
                assert_nobody_released(world);
            },
            |world| world.jobs().is_empty(),
        );
        assert_eq!(world.tile(EAST_END), Some(Tile::Empty));
        // Positive: the idle dwarf was given an exit path and walked it.
        assert!(had_path, "the idle dwarf never got an exit path");
        // Its last move left it on a tile it calls home, with no path, outside the tunnel and off
        // the miner's remaining route. (It wanders again afterwards, within WANDER_RADIUS of home.)
        let (tile, home, still_walking, route) = relocations.last().cloned().unwrap();
        assert_eq!(tile, home);
        assert!(!still_walking, "the exit path was not removed on arrival");
        assert!(
            tile.x <= 10,
            "the idle dwarf stopped inside the tunnel at {tile:?}"
        );
        assert!(
            !route.contains(&tile),
            "{tile:?} is on the miner's route {route:?}"
        );
        // Positive: the miner backed out west of where it started.
        assert!(trace.iter().any(|positions| positions[0].x < miner_start.x));
    }

    // 12.9 AC6: occupancy is a step-time rule, never a claim-time one. An idle dwarf stands in the
    // only passage (the one-wide tunnel) to a reachable dig. The miner still claims it once its
    // reaction delay has passed, the job is never stamped with a retry, and it completes.
    #[test]
    fn a_dwarf_in_the_only_passage_does_not_make_a_reachable_job_unreachable_at_claim_time() {
        let mut world = room_and_tunnel();
        let (miner_start, idle_start) = (at(9, FIX_Y, FIX_Z), at(13, FIX_Y, FIX_Z));
        crew_at(&mut world, miner_start, idle_start, &[0]);
        insert_dig(&mut world, 0, EAST_END);
        // Starts at tick 0, so the claim waits out the reaction delay, and the idle dwarf's wander
        // rest outlasts it: it is still in the passage when the claim is made.
        let idle = dwarf_entity(&world, 1);
        world.ecs.get_mut::<super::Wander>(idle).unwrap().cooldown = 40;

        let mut claimed_at = None;
        let mut stamped = false;
        watch(
            &mut world,
            1_000,
            |world| {
                stamped |= world.jobs().iter().any(|job| job.retry_after != 0);
                if claimed_at.is_none() && world.claims()[0].1.is_some() {
                    claimed_at = Some((world.tick(), world.dwarves()[1].1));
                }
            },
            |world| world.jobs().is_empty(),
        );
        assert!(!stamped, "the job was stamped with a retry");
        let (tick, blocker) = claimed_at.expect("the miner never claimed the dig");
        assert!(
            tick > 1,
            "claimed at tick {tick}, before any reaction delay"
        );
        assert!(
            (11..=16).contains(&blocker.x) && blocker.y == FIX_Y,
            "the idle dwarf left the passage before the claim: {blocker:?}"
        );
        assert_eq!(world.tile(EAST_END), Some(Tile::Empty));
    }

    // ---- 12.9 Task 10 (#162): every uncarried item blocks ----

    fn spawn_stone(world: &mut World, id: u32, pos: Pos) {
        world
            .ecs
            .spawn((super::Item(super::ItemKind::Stone), super::Id(id), pos));
    }

    /// Uncarried item tiles as the wire shows them: `items()` minus whatever `carrying()` holds.
    fn loose_item_tiles(world: &World) -> BTreeMap<u32, Pos> {
        let carried: BTreeSet<u32> = world
            .carrying()
            .into_iter()
            .filter_map(|(_, c)| c)
            .collect();
        world
            .items()
            .into_iter()
            .filter(|(id, _)| !carried.contains(&id.0))
            .map(|(id, pos)| (id.0, pos))
            .collect()
    }

    // 12.9 AC10 (corridor): one stone in a one-wide corridor that has a way round (a parallel
    // row joined at both ends). The miner's goal lies past the stone: it takes the way round and
    // never enters the stone's cell.
    #[test]
    fn a_dwarf_with_a_goal_past_a_stone_in_a_corridor_goes_round_it() {
        let mut world = solid_world();
        let corridor = (11..=17).map(|x| at(x, FIX_Y, FIX_Z));
        let bypass = (11..=17).map(|x| at(x, FIX_Y - 2, FIX_Z));
        carve(&mut world, corridor.chain(bypass));
        carve(
            &mut world,
            [at(11, FIX_Y - 1, FIX_Z), at(17, FIX_Y - 1, FIX_Z)],
        );
        let (start, stone, face) = (
            at(11, FIX_Y, FIX_Z),
            at(14, FIX_Y, FIX_Z),
            at(18, FIX_Y, FIX_Z),
        );
        let parked = at(30, 30, FIX_Z);
        carve(&mut world, [parked]);
        crew_at(&mut world, start, parked, &[0]);
        spawn_stone(&mut world, 100, stone);
        insert_dig(&mut world, 0, face);
        world.ecs.resource_mut::<super::Tick>().0 = 100;

        let mut entered = Vec::new();
        let mut used_bypass = false;
        let trace = watch(
            &mut world,
            1_000,
            |world| {
                let at_now = world.dwarves()[0].1;
                if at_now == stone {
                    entered.push(world.tick());
                }
                used_bypass |= at_now.y == FIX_Y - 2;
            },
            |world| world.jobs().is_empty(),
        );
        assert_eq!(world.tile(face), Some(Tile::Empty), "the dig completed");
        assert!(
            entered.is_empty(),
            "the miner stood in the stone's cell at ticks {entered:?}"
        );
        assert!(trace.iter().all(|positions| positions[0] != stone));
        // Positive: the miner really went the long way round.
        assert!(used_bypass, "the miner never used the bypass row");
    }

    /// A flat room (x 6..=24, y 14..=26, z `FIX_Z`) for the haul fixtures: dwarves 0 and 1 are
    /// haulers, the rest are parked in closets.
    fn haul_room(hauler: Pos) -> World {
        let mut world = solid_world();
        let room = (6..=24).flat_map(|x| (14..=26).map(move |y| at(x, y, FIX_Z)));
        carve(&mut world, room);
        crew_at(&mut world, hauler, at(6, 14, FIX_Z), &[]);
        set_profession(&mut world, 0, super::Profession::Hauler);
        set_profession(&mut world, 1, super::Profession::Hauler);
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        world
    }

    fn manhattan_one_same_z(a: Pos, b: Pos) -> bool {
        a.z == b.z && a.x.abs_diff(b.x) + a.y.abs_diff(b.y) == 1
    }

    // 12.9 AC11: the hauler picks up from a 4-neighbour of the stone and delivers from a
    // 4-neighbour of the cell the stone lands on; it is never on either cell.
    #[test]
    fn a_hauler_picks_up_and_drops_from_the_next_tile() {
        let mut world = haul_room(at(8, 20, FIX_Z));
        let (stone, landing) = (at(10, 20, FIX_Z), at(18, 20, FIX_Z));
        spawn_stone(&mut world, 100, stone);
        world.ecs.resource_mut::<super::Zones>().0.insert(landing);

        let mut picked_up = None;
        let mut dropped = None;
        let mut was_carrying = [false; 5];
        watch(
            &mut world,
            2_000,
            |world| {
                for (i, (_, held)) in world.carrying().into_iter().enumerate() {
                    let hauler = world.dwarves()[i].1;
                    if held.is_some() && !was_carrying[i] {
                        picked_up = Some((world.tick(), hauler));
                    }
                    if held.is_none() && was_carrying[i] {
                        dropped = Some((world.tick(), hauler, loose_item_tiles(world)[&100]));
                    }
                    was_carrying[i] = held.is_some();
                }
            },
            // The haul job is made on the first step, so wait for it to exist and then finish.
            |world| world.tick() > 100 && world.jobs().is_empty(),
        );
        let (tick, hauler) = picked_up.expect("the stone was never picked up");
        assert!(
            manhattan_one_same_z(hauler, stone),
            "tick {tick}: the hauler picked up from {hauler:?}, not a 4-neighbour of {stone:?}"
        );
        let (tick, hauler, landed) = dropped.expect("the stone was never delivered");
        assert_eq!(landed, landing, "the stone must land on the pile cell");
        assert!(
            manhattan_one_same_z(hauler, landing),
            "tick {tick}: the hauler dropped from {hauler:?}, not a 4-neighbour of {landing:?}"
        );
    }

    // 12.9 AC12: a 3x3 pile with more than nine reachable loose stones fills all nine cells, and
    // the centre is filled before every edge cell (the pile fills from the inside out).
    #[test]
    fn a_3x3_pile_fills_from_the_inside_out() {
        // Measured: all nine cells full by tick 1,257 (the clock starts at 100). The bound is 1.6x.
        const FILL_BOUND: u64 = 2_000;
        let mut world = haul_room(at(8, 20, FIX_Z));
        let pile: BTreeSet<Pos> = (17..=19)
            .flat_map(|x| (19..=21).map(move |y| at(x, y, FIX_Z)))
            .collect();
        world
            .ecs
            .resource_mut::<super::Zones>()
            .0
            .extend(pile.iter().copied());
        for (k, y) in (15..=25).enumerate() {
            spawn_stone(&mut world, 100 + k as u32, at(8 + (k as i32 % 2), y, FIX_Z));
        }
        // Eleven stones for nine cells; the hauler at (8, 20) stands beside one of them.

        let mut filled_at: BTreeMap<Pos, u64> = BTreeMap::new();
        watch(
            &mut world,
            FILL_BOUND,
            |world| {
                for pos in loose_item_tiles(world).into_values() {
                    if pile.contains(&pos) {
                        filled_at.entry(pos).or_insert(world.tick());
                    }
                }
            },
            |world| {
                loose_item_tiles(world)
                    .into_values()
                    .filter(|pos| pile.contains(pos))
                    .collect::<BTreeSet<_>>()
                    .len()
                    == 9
            },
        );
        assert_eq!(filled_at.len(), 9, "all nine cells filled: {filled_at:?}");
        let centre = at(18, 20, FIX_Z);
        for (cell, tick) in &filled_at {
            assert!(
                *cell == centre || filled_at[&centre] < *tick,
                "centre filled at {} but edge {cell:?} at {tick}",
                filled_at[&centre]
            );
        }
    }

    // 12.9 review #184: stones already lie on the four edge-middles of a 3x3 pile (a pile placed
    // over loose stones, or `release_claim`'s abnormal drop), so the free centre is walled in and
    // only the four corners can be reached. The pile still takes the next stone, on a corner.
    #[test]
    fn a_walled_in_free_cell_does_not_stop_the_pile() {
        // Measured: the corner delivery lands at tick 312 (the clock starts at 100).
        const DELIVERY_BOUND: u64 = 1_000;
        let mut world = haul_room(at(8, 20, FIX_Z));
        let pile: BTreeSet<Pos> = (17..=19)
            .flat_map(|x| (19..=21).map(move |y| at(x, y, FIX_Z)))
            .collect();
        world
            .ecs
            .resource_mut::<super::Zones>()
            .0
            .extend(pile.iter().copied());
        let edges = [
            at(17, 20, FIX_Z),
            at(19, 20, FIX_Z),
            at(18, 19, FIX_Z),
            at(18, 21, FIX_Z),
        ];
        for (k, edge) in edges.into_iter().enumerate() {
            spawn_stone(&mut world, 200 + k as u32, edge);
        }
        spawn_stone(&mut world, 100, at(10, 20, FIX_Z));

        watch(
            &mut world,
            DELIVERY_BOUND,
            |_| {},
            |world| {
                loose_item_tiles(world)
                    .get(&100)
                    .is_some_and(|pos| pile.contains(pos))
            },
        );
        let landed = loose_item_tiles(&world)[&100];
        let corners = [
            at(17, 19, FIX_Z),
            at(19, 19, FIX_Z),
            at(17, 21, FIX_Z),
            at(19, 21, FIX_Z),
        ];
        assert!(
            corners.contains(&landed),
            "the stone landed on {landed:?}, not a reachable corner"
        );
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
