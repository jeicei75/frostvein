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
    entity::Entity,
    query::With,
    resource::Resource,
    schedule::{IntoScheduleConfigs, Schedule},
    system::{Commands, Query, Res, ResMut},
    world::World as EcsWorld,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

const STREAM_WORLDGEN: u64 = 0x4652_4f53_5456_4549;
const STREAM_SPAWN: u64 = 0x5350_4157_4e5f_5f5f;
const STREAM_WANDER: u64 = 0x5741_4e44_4552_5f5f;
const STREAM_TREES: u64 = 0x5452_4545_535f_5f5f;
const WANDER_RADIUS: i32 = 3;
const WANDER_REST_TICKS: u32 = 10;
pub const MAX_DESIGNATIONS: usize = 4096;
const MAX_ASTAR_NODES: usize = 50_000;
pub const WORK_TICKS: u32 = 5;
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
            JobKind::Dig | JobKind::Channel => {
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
            JobKind::Dig | JobKind::Channel => {
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
        // The lowest-id clause is the repair mechanism for a real race review found: two carriers
        // can both be walking to the last free tile, the first delivers, and the second — now
        // standing on a tile that is no longer in its goal set — is retried, and `release_claim`
        // drops its stone where it stands. Two stones on one tile. Under the old rule both counted
        // as stored, so both jobs were retired and the stack was permanent and invisible. Now the
        // extra one stays loose, keeps (or regains) a haul job, and re-hauls itself to a genuinely
        // free tile. It cannot thrash: the pick-up leg is gated on a free tile existing, and a
        // delivery only ever targets a free tile, so the stone it is standing on is never a goal.
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

// AD-12: claiming LOGIC is unchanged — FIFO by `JobId`, ascending dwarf `Id`, reaction delay,
// `retry_after` and one shared node budget. Only the goal set it asks for learned `Haul`, which
// is why the stockpile and the stones have to reach this system.
#[allow(clippy::too_many_arguments)]
fn claim_jobs(
    mut commands: Commands,
    seed: Res<Seed>,
    tick: Res<Tick>,
    terrain: Res<Terrain>,
    zones: Res<Zones>,
    mut jobs: ResMut<Jobs>,
    stones: Query<(&Id, &Pos), With<Item>>,
    mut dwarves: Query<(Entity, &Id, &Pos, &mut CurrentJob, &Carrying)>,
) {
    let mut dwarves: Vec<_> = dwarves.iter_mut().collect();
    dwarves.sort_by_key(|(_, id, _, _, _)| **id);
    let mut claimed: BTreeSet<_> = dwarves
        .iter()
        .filter_map(|(_, _, _, current, _)| current.0)
        .collect();
    let carried: BTreeSet<u32> = dwarves
        .iter()
        .filter_map(|(_, _, _, _, carrying)| carrying.0)
        .collect();
    let items: BTreeMap<u32, Pos> = stones
        .iter()
        .filter(|(id, _)| !carried.contains(&id.0))
        .map(|(id, pos)| (id.0, *pos))
        .collect();
    let mut astar_nodes_remaining = MAX_ASTAR_NODES;

    let jobs_in_order: Vec<_> = jobs.iter().copied().collect();
    'jobs: for job in jobs_in_order {
        if astar_nodes_remaining == 0 {
            break;
        }
        if claimed.contains(&job.id) || tick.0 < job.retry_after {
            continue;
        }
        // A claimable dwarf holds no job, and by AC10 therefore carries nothing — so one goal
        // set serves every candidate.
        let goals = work_positions(&terrain, &zones.0, &items, job, None);
        let mut attempted = false;
        let mut assigned = false;
        for (entity, id, pos, current, carrying) in &mut dwarves {
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
                attempted = true;
                let path =
                    match astar_with_budget(&terrain, **pos, &goals, &mut astar_nodes_remaining) {
                        (Some(path), false) => path,
                        (None, false) => continue,
                        (None, true) => break 'jobs,
                        (Some(_), true) => {
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
        if attempted && !assigned {
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
        .map(|goal| {
            let horizontal = from.x.abs_diff(goal.x) + from.y.abs_diff(goal.y);
            horizontal.max(from.z.abs_diff(goal.z))
        })
        .min()
        .unwrap_or(0)
}

fn astar_with_budget(
    terrain: &Terrain,
    from: Pos,
    goals: &BTreeSet<Pos>,
    nodes_remaining: &mut usize,
) -> (Option<Vec<Pos>>, bool) {
    if goals.is_empty() {
        return (None, false);
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
            return (None, true);
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
            return (Some(path), false);
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
    (None, false)
}

fn astar(terrain: &Terrain, from: Pos, goals: &BTreeSet<Pos>) -> Option<Vec<Pos>> {
    let mut nodes_remaining = MAX_ASTAR_NODES;
    astar_with_budget(terrain, from, goals, &mut nodes_remaining).0
}

/// `items` holds UNCARRIED stones only — a stone in transit occupies no tile, so a carrier
/// crossing the pile never blocks a tile for anyone else.
fn work_positions(
    terrain: &Terrain,
    zones: &BTreeSet<Pos>,
    items: &BTreeMap<u32, Pos>,
    job: Job,
    carrying: Option<u32>,
) -> BTreeSet<Pos> {
    match job.kind {
        JobKind::Dig => [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .map(|(dx, dy)| Pos {
                x: job.target.x + dx,
                y: job.target.y + dy,
                z: job.target.z,
            })
            .filter(|candidate| terrain.is_standable(*candidate))
            .collect(),
        JobKind::Channel => {
            if terrain.is_standable(job.target) {
                BTreeSet::from([job.target])
            } else {
                BTreeSet::new()
            }
        }
        JobKind::Haul { item } => {
            debug_assert!(
                carrying.is_none_or(|carried| carried == item),
                "a dwarf only ever carries the stone of the haul job it holds"
            );
            // One stone per stockpile tile (Wolf, 2026-08-07). Recomputed every tick and never
            // cached: that is what makes two carriers converging on one free tile self-healing —
            // the moment the first drops, the tile leaves the second's goal set and it repaths.
            let stored: BTreeSet<Pos> = items
                .values()
                .copied()
                .filter(|pos| zones.contains(pos))
                .collect();
            let free: BTreeSet<Pos> = zones
                .iter()
                .copied()
                // Zone tiles are validated standable at command time and never re-checked.
                .filter(|pos| terrain.is_standable(*pos) && !stored.contains(pos))
                .collect();
            if carrying.is_some() {
                return free;
            }
            // Both legs read the same free-tile set. With nowhere to deliver the pick-up leg is
            // empty too, so the job is never claimed rather than claimed into a
            // pick-up-and-drop cycle.
            // NOTE: a stone whose floor was dug away is on no standable tile — items never fall —
            // so its job retries forever. Retry is nearly free and never-drop wins (FR8).
            match items.get(&item) {
                Some(pos) if !free.is_empty() && terrain.is_standable(*pos) => {
                    BTreeSet::from([*pos])
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
    // A dwarf that stops holding a job stops carrying its stone, and drops it where it stands.
    // Doing it here is what keeps every abnormal exit — a vanished job, a retry, a cancel, a
    // retire — from welding a stone to an idle dwarf.
    if let Some(item) = ecs.get::<Carrying>(entity).and_then(|carrying| carrying.0) {
        let dropped_at = ecs.get::<Pos>(entity).copied();
        if let (Some(pos), Some(stone)) = (dropped_at, item_entity(ecs, item)) {
            *ecs.get_mut::<Pos>(stone)
                .expect("every stone has a position") = pos;
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

/// Exclusive so terrain mutation and stone spawning are visible in the same tick.
fn execute_jobs(ecs: &mut EcsWorld) {
    let mut dwarves: Vec<_> = ecs
        .iter_entities()
        .filter(|entity| entity.contains::<Dwarf>())
        .filter_map(|entity| Some((*entity.get::<Id>()?, entity.id())))
        .collect();
    dwarves.sort_by_key(|(id, _)| *id);

    for (_, entity) in dwarves {
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
        let work_positions = {
            let stones = uncarried_stones(ecs);
            work_positions(
                ecs.resource::<Terrain>(),
                &ecs.resource::<Zones>().0,
                &stones,
                job,
                carrying,
            )
        };

        if !work_positions.contains(&pos) {
            let mut path = ecs
                .get::<Path>(entity)
                .map(|path| path.0.clone())
                .unwrap_or_default();
            if path.is_empty() {
                let terrain = ecs.resource::<Terrain>();
                let Some(computed) = astar(terrain, pos, &work_positions) else {
                    retry_claim(ecs, entity, job.id);
                    continue;
                };
                path = computed;
            }
            let next = path.remove(0);
            *ecs.get_mut::<Pos>(entity)
                .expect("every dwarf has a position") = next;
            *ecs.get_mut::<JobState>(entity)
                .expect("every dwarf has a job state") = JobState::Walk;
            ecs.entity_mut(entity).insert(Path(path));
            continue;
        }

        let progress = ecs
            .get::<WorkProgress>(entity)
            .map(|progress| progress.0)
            .unwrap_or(0);
        if progress < WORK_TICKS {
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
                // Deliver. `release_claim` is what puts the stone on the tile the dwarf stands
                // on and clears the slot — deliberately the same funnel every abnormal exit
                // uses, so there is one place a stone can be dropped and not two.
                Some(_) => {
                    ecs.resource_mut::<Jobs>().remove(job.id);
                    release_claim(ecs, entity);
                }
            }
            continue;
        }

        let change = {
            let terrain = ecs.resource::<Terrain>();
            match job.kind {
                JobKind::Dig => match terrain.tile(job.target) {
                    Some(Tile::Solid(material)) => Some((
                        job.target,
                        Tile::Empty,
                        !matches!(material, Material::TreeTrunk | Material::TreeFoliage),
                    )),
                    _ => None,
                },
                JobKind::Channel => {
                    let below = Pos {
                        z: job.target.z - 1,
                        ..job.target
                    };
                    match terrain.tile(below) {
                        Some(Tile::Solid(material)) => Some((
                            below,
                            Tile::Ramp(material),
                            !matches!(material, Material::TreeTrunk | Material::TreeFoliage),
                        )),
                        _ => None,
                    }
                }
                JobKind::Haul { .. } => unreachable!("haul jobs are dispatched above"),
            }
        };
        let Some((changed_pos, tile, yields_stone)) = change else {
            ecs.resource_mut::<Jobs>().remove(job.id);
            ecs.resource_mut::<Designations>().0.remove(&job.target);
            release_claim(ecs, entity);
            continue;
        };
        let changed = ecs.resource_mut::<Terrain>().set_tile(changed_pos, tile);
        debug_assert!(
            changed,
            "job targets were bounds-checked at designation time"
        );
        clear_paths(ecs);
        if yields_stone {
            let item_id = ecs.resource_mut::<IdAllocator>().allocate();
            ecs.spawn((Item, item_id, job.target));
        }
        ecs.resource_mut::<Jobs>().remove(job.id);
        ecs.resource_mut::<Designations>().0.remove(&job.target);
        release_claim(ecs, entity);
    }
}

fn settle(ecs: &mut EcsWorld) {
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
            *ecs.get_mut::<Pos>(entity)
                .expect("every dwarf has a position") = below;
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

fn wander(
    mut rng: ResMut<WanderRng>,
    terrain: Res<Terrain>,
    mut dwarves: Query<(&Id, &mut Pos, &mut Wander, &mut JobState, &CurrentJob)>,
) {
    // AD-7: query iteration is archetype order, not Id order, and all dwarves draw from
    // one stream. Draw order is a sim outcome, so sort before touching the RNG.
    let mut dwarves: Vec<_> = dwarves.iter_mut().collect();
    dwarves.sort_by_key(|(id, ..)| **id);

    for (_, mut pos, mut wander, mut state, current_job) in dwarves {
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
        // NOTE: fixed order, same z only. Ramp climbing arrives with A* in Story 3.2.
        let candidates: Vec<Pos> = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .map(|(dx, dy)| Pos {
                x: here.x + dx,
                y: here.y + dy,
                z: here.z,
            })
            // NOTE: standability only — occupancy is deliberately not a movement rule. The
            // renderer uses a crowd glyph when dwarves share a tile.
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
        let heights = worldgen::height_field(dims, &mut rng);
        let mut tiles = worldgen::layered_terrain(dims, &heights, &mut rng);
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
        world.spawn_dwarves(camp_origin, &mut spawn_rng);
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
                })
            })
            .collect();
        dwarves.sort_by_key(|dwarf| dwarf.id);
        let jobs = self.jobs();
        let job_resource = self.ecs.resource::<Jobs>();
        let items = self
            .items()
            .into_iter()
            .map(|(id, pos)| (id.0, pos))
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
                ))
                .id();
            if current_job.is_some() {
                world
                    .ecs
                    .entity_mut(entity)
                    .insert(WorkProgress(dwarf.work_progress));
            }
        }
        for (id, pos) in items {
            world.ecs.spawn((Item, Id(id), pos));
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
                let targets: BTreeSet<_> = positions().collect();
                {
                    let mut designations = self.ecs.resource_mut::<Designations>();
                    for pos in &targets {
                        designations.0.remove(pos);
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
                    .filter(|job| matches!(job.kind, JobKind::Dig | JobKind::Channel))
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

    /// Sorted ascending by dwarf `Id`. A sibling reader to `claims()` and `items()`, which is
    /// why `dwarves()` keeps its three-tuple shape and the clients need no new arm.
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
    // NOTE: the carried stone deliberately did NOT become a fourth field here. `carrying()` is a
    // sibling reader instead, which leaves this tuple — and therefore `simd`'s bridge — untouched.
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

    fn spawn_dwarves(&mut self, camp: Pos, rng: &mut ChaCha8Rng) {
        let mut candidates = {
            let terrain = self.ecs.resource::<Terrain>();
            let mut candidates = Vec::new();
            let radius = worldgen::CAMP_RADIUS as i32;
            for y in camp.y - radius..=camp.y + radius {
                for x in camp.x - radius..=camp.x + radius {
                    let pos = Pos { x, y, z: camp.z };
                    if terrain.is_standable(pos) {
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
                Carrying(None),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

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
        world.ecs.spawn((super::Item, super::Id(12), later));
        world.ecs.spawn((super::Item, super::Id(11), earlier));

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
            world.ecs.spawn((super::Item, super::Id(id), pos));
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
        world
            .ecs
            .spawn((super::Item, super::Id(12), Pos { x: 20, y: 20, z: 8 }));

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
        world.ecs.spawn((super::Item, super::Id(12), stone));
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
        world.ecs.spawn((super::Item, super::Id(12), stone));
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
        for _ in 0..24 {
            super::execute_jobs(&mut world.ecs);
            states.push(world.dwarves()[0].2);
            if world.jobs().is_empty() {
                break;
            }
        }

        use JobState::{Idle, Walk, Work};
        assert_eq!(
            states,
            vec![
                Walk, Walk, Work, Work, Work, Work, Work, Walk, Walk, Walk, Work, Work, Work, Work,
                Work, Idle,
            ],
            "two walks and exactly WORK_TICKS of work in each of the two legs"
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
        world.ecs.spawn((super::Item, super::Id(12), stranded));

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
        world.ecs.spawn((super::Item, super::Id(11), pile));
        world.ecs.spawn((super::Item, super::Id(12), loose));
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
        world.ecs.spawn((super::Item, super::Id(12), stone));
        place_stockpile(&mut world, cell(4));
        super::create_haul_jobs(&mut world.ecs);
        let job = world.jobs()[0];
        assert_eq!(job.kind, JobKind::Haul { item: 12 });
        let entity = dwarf_entity(&world, 0);
        // Standing on the stone already, so the very next WORK_TICKS runs end in the pick-up.
        *world.ecs.get_mut::<Pos>(entity).unwrap() = stone;
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

        let reachable = BTreeMap::from([(12, standing)]);
        assert_eq!(
            super::work_positions(&terrain, &zones, &reachable, job, None),
            BTreeSet::from([standing]),
            "a standable stone with a free pile tile is its own work position"
        );
        assert_eq!(
            super::work_positions(&terrain, &zones, &reachable, job, Some(12)),
            BTreeSet::from([pile]),
            "a carrying dwarf is sent to the free pile tile"
        );

        let unstandable = BTreeMap::from([(12, sunken)]);
        assert!(
            super::work_positions(&terrain, &zones, &unstandable, job, None).is_empty(),
            "a stone on unstandable ground has no work position"
        );

        // The pile itself holding a stored stone leaves both legs empty.
        let occupied = BTreeMap::from([(12, standing), (13, pile)]);
        assert!(
            super::work_positions(&terrain, &zones, &occupied, job, None).is_empty(),
            "the pick-up leg must be gated on a free tile existing"
        );
        assert!(
            super::work_positions(&terrain, &zones, &occupied, job, Some(12)).is_empty(),
            "a full pile is no delivery target"
        );
    }

    #[test]
    fn haul_execution_reads_the_stones_live_position_not_the_jobs_target() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let cell = corridor(&mut world);
        let stone = cell(1);
        let stale = cell(3);
        let pile = cell(4);
        world.ecs.spawn((super::Item, super::Id(12), stone));
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
        world.ecs.spawn((super::Item, super::Id(12), loose));
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
        world.ecs.spawn((super::Item, super::Id(12), stone));
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
        world.ecs.spawn((super::Item, super::Id(12), far_away));
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
        world.step();
        assert_eq!(world.claims()[2], (super::Id(2), Some(JobId(0))));
        assert_eq!(world.dwarves()[2].1, worker);
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

        assert_eq!(world.items(), vec![(super::Id(5), target)]);
        assert!(world.jobs().is_empty());
    }

    #[test]
    fn claim_jobs_bounds_aggregate_astar_expansions_per_tick() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let dims = world.dims();
        let mut tiles = vec![Tile::Solid(Material::Stone); world.tiles().len()];
        for y in 0..40 {
            for x in 0..50 {
                tiles[super::worldgen::index(dims, x, y, 1)] = Tile::Empty;
            }
        }
        for job in 0..10_u32 {
            let target = Pos {
                x: 2 + 2 * job as i32,
                y: 2,
                z: 10,
            };
            tiles[super::worldgen::index(
                dims,
                (target.x + 1) as u32,
                target.y as u32,
                target.z as u32,
            )] = Tile::Empty;
        }
        world.ecs.resource_mut::<Terrain>().tiles = tiles;

        let dwarves: Vec<_> = world
            .ecs
            .iter_entities()
            .filter_map(|entity| Some((*entity.get::<super::Id>()?, entity.id())))
            .collect();
        for (id, entity) in dwarves {
            *world.ecs.get_mut::<Pos>(entity).unwrap() = Pos {
                x: id.0 as i32,
                y: 0,
                z: 1,
            };
        }
        for job in 0..10_u32 {
            assert!(world.ecs.resource_mut::<Jobs>().insert(Job {
                id: JobId(job),
                kind: JobKind::Dig,
                target: Pos {
                    x: 2 + 2 * job as i32,
                    y: 2,
                    z: 10,
                },
                created_tick: 0,
                retry_after: 0,
            }));
        }
        world.ecs.resource_mut::<super::Tick>().0 = 100;
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(super::claim_jobs);

        schedule.run(&mut world.ecs);

        let retried: Vec<_> = world
            .jobs()
            .into_iter()
            .filter(|job| job.retry_after == 120)
            .map(|job| job.id)
            .collect();
        assert_eq!(
            retried,
            vec![JobId(0), JobId(1), JobId(2), JobId(3), JobId(4)],
            "one tick may expand at most MAX_ASTAR_NODES across all failed claim searches"
        );
        assert!(world.jobs()[5..].iter().all(|job| job.retry_after == 0));
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

        assert_eq!(super::astar(&terrain, from, &goals), Some(expected.clone()));
        assert_eq!(super::astar(&terrain, from, &goals), Some(expected));
    }

    #[test]
    fn astar_routes_a_dwarf_around_a_tree_trunk() {
        let mut terrain = flat_terrain(5, 3);
        let trunk = Pos { x: 2, y: 1, z: 1 };
        terrain.tiles[super::worldgen::index(terrain.dims, 2, 1, 1)] =
            Tile::Solid(Material::TreeTrunk);
        let from = Pos { x: 0, y: 1, z: 1 };
        let goal = Pos { x: 4, y: 1, z: 1 };

        let path = super::astar(&terrain, from, &BTreeSet::from([goal]))
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
            super::astar(&terrain, from, &BTreeSet::from([goal])),
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
            super::astar(&terrain, Pos { x: 1, y: 2, z: 2 }, &BTreeSet::from([goal]),),
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
            Pos { x: 0, y: 0, z: 1 },
            &BTreeSet::from([Pos { x: 5, y: 5, z: 1 }]),
        )
        .expect("a 10-step goal on open ground is far inside the 50,000-node budget");
        assert_eq!(path.len(), 10, "shortest path is |dx| + |dy|");
    }

    #[test]
    fn execute_jobs_walks_then_digs_for_exactly_five_work_ticks() {
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
        for _ in 0..5 {
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
        assert_eq!(world.items(), vec![(super::Id(5), target)]);
        assert_eq!(world.drain_dirty(), vec![(target, Tile::Empty)]);
    }

    #[test]
    fn execute_jobs_digs_tree_materials_without_spawning_items() {
        for material in [Material::TreeTrunk, Material::TreeFoliage] {
            let mut world = World::generate(42, Dims::DEFAULT);
            let work = world.dwarves()[0].1;
            let target = Pos {
                x: work.x + 1,
                ..work
            };
            assert!(world.set_tile(target, Tile::Solid(material)));
            world.drain_dirty();
            let items_before = world.items().len();
            let job = Job {
                id: JobId(0),
                kind: JobKind::Dig,
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
                .insert(super::WorkProgress(super::WORK_TICKS));

            super::execute_jobs(&mut world.ecs);

            assert_eq!(world.tile(target), Some(Tile::Empty));
            assert_eq!(world.drain_dirty(), vec![(target, Tile::Empty)]);
            assert_eq!(world.items().len(), items_before, "dug {material:?}");
        }
    }

    #[test]
    fn execute_jobs_channels_tree_materials_without_spawning_items() {
        for material in [Material::TreeTrunk, Material::TreeFoliage] {
            let mut world = World::generate(42, Dims::DEFAULT);
            let target = world.dwarves()[0].1;
            let below = Pos {
                z: target.z - 1,
                ..target
            };
            assert!(world.set_tile(below, Tile::Solid(material)));
            assert!(world.set_tile(target, Tile::Empty));
            world.drain_dirty();
            let items_before = world.items().len();
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
                .insert(super::WorkProgress(super::WORK_TICKS));

            super::execute_jobs(&mut world.ecs);

            assert_eq!(world.tile(below), Some(Tile::Ramp(material)));
            assert_eq!(world.drain_dirty(), vec![(below, Tile::Ramp(material))]);
            assert_eq!(world.items().len(), items_before, "channelled {material:?}");
        }
    }

    #[test]
    fn execute_jobs_channels_a_material_preserving_ramp_and_spawns_stone() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let target = world.dwarves()[0].1;
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

        for _ in 0..5 {
            super::execute_jobs(&mut world.ecs);
            assert_eq!(world.dwarves()[0].2, JobState::Work);
            assert_eq!(world.claims()[0].1, Some(JobId(0)));
        }
        super::execute_jobs(&mut world.ecs);

        assert_eq!(world.tile(below), Some(Tile::Ramp(Material::Soil)));
        assert_eq!(world.items(), vec![(super::Id(5), target)]);
        assert_eq!(
            world.drain_dirty(),
            vec![(below, Tile::Ramp(Material::Soil))]
        );
    }

    #[test]
    fn execute_jobs_removes_a_channel_job_when_the_support_is_already_a_ramp() {
        let mut world = World::generate(42, Dims::DEFAULT);
        let target = world.dwarves()[0].1;
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
        world
            .ecs
            .entity_mut(entity)
            .insert((super::Path(Vec::new()), super::WorkProgress(5)));

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
        for (pos, tile) in [
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
        world
            .ecs
            .entity_mut(worker_entity)
            .insert((super::Path(Vec::new()), super::WorkProgress(5)));
        world
            .ecs
            .entity_mut(victim_entity)
            .insert((super::Path(vec![escape]), super::WorkProgress(0)));

        super::execute_jobs(&mut world.ecs);
        super::settle(&mut world.ecs);

        assert_eq!(*world.ecs.get::<Pos>(victim_entity).unwrap(), dug_floor);
        assert_eq!(
            *world.ecs.get::<JobState>(victim_entity).unwrap(),
            JobState::Walk,
            "a dwarf holding a job is never reported idle while falling"
        );
        assert!(!world.ecs.entity(victim_entity).contains::<super::Path>());
        assert_eq!(world.claims()[1].1, Some(JobId(1)));
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
