#![forbid(unsafe_code)]

mod worldgen;

use bevy_ecs::{component::Component, world::World as EcsWorld};
use rand::SeedableRng;
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

#[derive(Default)]
struct IdAllocator {
    next: u32,
}

pub struct World {
    dims: Dims,
    tiles: Vec<Tile>,
    ecs: EcsWorld,
    ids: IdAllocator,
    seed: u64,
}

impl World {
    pub fn generate(seed: u64, dims: Dims) -> World {
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_WORLDGEN);
        let heights = worldgen::height_field(dims, &mut rng);
        let mut tiles = worldgen::layered_terrain(dims, &heights, &mut rng);
        worldgen::place_ramps(dims, &heights, &mut tiles);

        World {
            dims,
            tiles,
            ecs: EcsWorld::new(),
            ids: IdAllocator::default(),
            seed,
        }
    }

    pub fn dims(&self) -> Dims {
        self.dims
    }

    pub fn seed(&self) -> u64 {
        self.seed
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
}
