#![forbid(unsafe_code)]

mod worldgen;

use std::collections::BTreeSet;

use bevy_ecs::{
    component::Component,
    resource::Resource,
    schedule::{IntoScheduleConfigs, Schedule},
    system::ResMut,
    world::World as EcsWorld,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

const STREAM_WORLDGEN: u64 = 0x4652_4f53_5456_4549;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Stone,
    Soil,
    Ice,
    Snow,
}

/// A voxel. `Empty` is air; `Solid` is wall/floor; `Ramp` is a walkable slope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    Solid(Material),
    Ramp(Material),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Component)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Resource)]
struct Tick(pub u64);

fn advance_tick(mut tick: ResMut<Tick>) {
    tick.0 += 1;
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
    dims: Dims,
    tiles: Vec<Tile>,
    dirty: BTreeSet<Pos>,
    ecs: EcsWorld,
    schedule: Schedule,
    ids: IdAllocator,
    seed: u64,
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

        let mut ecs = EcsWorld::new();
        ecs.insert_resource(Tick(0));
        let mut schedule = Schedule::default();
        schedule.add_systems((advance_tick,).chain());

        let mut world = World {
            dims,
            tiles,
            dirty: BTreeSet::new(),
            ecs,
            schedule,
            ids: IdAllocator::default(),
            seed,
        };
        world.spawn_dwarves(&heights, &mut rng);
        world
    }

    pub fn dims(&self) -> Dims {
        self.dims
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
        &self.tiles
    }

    pub fn tile(&self, p: Pos) -> Option<Tile> {
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

    pub fn set_tile(&mut self, p: Pos, tile: Tile) -> bool {
        if self.tile(p).is_none() {
            return false;
        }

        let index = worldgen::index(self.dims, p.x as u32, p.y as u32, p.z as u32);
        self.tiles[index] = tile;
        self.dirty.insert(p);
        true
    }

    pub fn drain_dirty(&mut self) -> Vec<(Pos, Tile)> {
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

    /// Sorted ascending by `Id` — stable order is required by AD-7.
    pub fn dwarves(&self) -> Vec<(Id, Pos)> {
        let mut dwarves: Vec<_> = self
            .ecs
            .iter_entities()
            .filter(|entity| entity.contains::<Dwarf>())
            .filter_map(|entity| Some((*entity.get::<Id>()?, *entity.get::<Pos>()?)))
            .collect();
        dwarves.sort_by_key(|(id, _)| *id);
        dwarves
    }

    fn spawn_dwarves(&mut self, heights: &[u32], rng: &mut ChaCha8Rng) {
        let mut candidates = Vec::new();
        for y in 0..self.dims.y {
            for x in 0..self.dims.x {
                let height = heights[(x + y * self.dims.x) as usize];
                let is_flat = [
                    (x as i32 - 1, y as i32),
                    (x as i32 + 1, y as i32),
                    (x as i32, y as i32 - 1),
                    (x as i32, y as i32 + 1),
                ]
                .into_iter()
                .filter(|&(nx, ny)| {
                    nx >= 0 && ny >= 0 && nx < self.dims.x as i32 && ny < self.dims.y as i32
                })
                .all(|(nx, ny)| heights[(nx as u32 + ny as u32 * self.dims.x) as usize] == height);
                if is_flat
                    && matches!(
                        self.tiles[worldgen::index(self.dims, x, y, height)],
                        Tile::Solid(_)
                    )
                {
                    candidates.push(Pos {
                        x: x as i32,
                        y: y as i32,
                        z: height as i32 + 1,
                    });
                }
            }
        }

        for _ in 0..5 {
            let candidate = rng.random_range(0..candidates.len());
            let pos = candidates.swap_remove(candidate);
            let id = self.ids.allocate();
            self.ecs.spawn((Dwarf, id, pos));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Dims, World};

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
}
