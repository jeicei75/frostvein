#![forbid(unsafe_code)]

mod save;
mod worldgen;

pub use save::{SaveState, SavedDwarf};

use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Serialize, Deserialize)]
pub enum JobState {
    Idle,
    Walk,
    Work,
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

#[derive(Default)]
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
    ids: IdAllocator,
    seed: u64,
}

fn assemble(
    seed: u64,
    dims: Dims,
    tiles: Vec<Tile>,
    tick: u64,
    wander_rng: ChaCha8Rng,
    ids: IdAllocator,
) -> World {
    let mut ecs = EcsWorld::new();
    ecs.insert_resource(Tick(tick));
    ecs.insert_resource(WanderRng(wander_rng));
    ecs.insert_resource(Designations::default());
    ecs.insert_resource(Zones::default());
    ecs.insert_resource(Terrain {
        dims,
        tiles,
        dirty: BTreeSet::new(),
    });
    let mut schedule = Schedule::default();
    schedule.add_systems((advance_tick, wander).chain());
    World {
        ecs,
        schedule,
        ids,
        seed,
    }
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
            seed: self.seed,
            tick: self.tick(),
            dims: terrain.dims,
            tiles: terrain.tiles.clone(),
            wander_rng: self.ecs.resource::<WanderRng>().0.clone(),
            next_id: self.ids.next,
            dwarves,
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
        } = save;
        let mut world = assemble(
            seed,
            dims,
            tiles,
            tick,
            wander_rng,
            IdAllocator { next: next_id },
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
            ));
        }
        world
    }

    pub fn dims(&self) -> Dims {
        self.ecs.resource::<Terrain>().dims
    }

    pub fn seed(&self) -> u64 {
        self.seed
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
                let mut designations = self.ecs.resource_mut::<Designations>();
                // NOTE: Story 3.2 owns diggability; every in-bounds tile is marked here.
                for pos in positions() {
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
            let id = self.ids.allocate();
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
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{Dims, JobState, Material, Pos, Terrain, Tile, World};

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
