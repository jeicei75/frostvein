use std::collections::BTreeSet;

use sim_core::{Dims, Material, Pos, Tile, World};

fn surface_height(world: &World, x: i32, y: i32) -> i32 {
    (0..world.dims().z as i32)
        .rev()
        .find(|&z| world.tile(Pos { x, y, z }) != Some(Tile::Empty))
        .expect("every column has terrain")
}

#[test]
fn same_seed_produces_identical_worlds() {
    let first = World::generate(42, Dims::DEFAULT);
    let second = World::generate(42, Dims::DEFAULT);

    assert_eq!(first.tiles(), second.tiles());
    assert_eq!(first.dwarves(), second.dwarves());
}

#[test]
fn different_seed_produces_different_world() {
    let first = World::generate(42, Dims::DEFAULT);
    let second = World::generate(43, Dims::DEFAULT);

    assert_ne!(first.tiles(), second.tiles());
}

#[test]
fn surface_is_icy() {
    let world = World::generate(42, Dims::DEFAULT);
    let mut has_stone = false;
    let mut has_soil = false;
    let mut has_ice = false;
    let mut has_snow = false;

    for tile in world.tiles() {
        match tile {
            Tile::Solid(Material::Stone) => has_stone = true,
            Tile::Solid(Material::Soil) => has_soil = true,
            _ => {}
        }
    }

    for y in 0..world.dims().y as i32 {
        for x in 0..world.dims().x as i32 {
            let z = surface_height(&world, x, y);
            match world.tile(Pos { x, y, z }) {
                Some(Tile::Solid(Material::Ice) | Tile::Ramp(Material::Ice)) => has_ice = true,
                Some(Tile::Solid(Material::Snow) | Tile::Ramp(Material::Snow)) => has_snow = true,
                _ => {}
            }
        }
    }

    assert!(has_stone);
    assert!(has_soil);
    assert!(has_ice);
    assert!(has_snow);
}

#[test]
fn height_varies_and_steps_are_at_most_one() {
    let world = World::generate(42, Dims::DEFAULT);
    let mut heights = BTreeSet::new();

    for y in 0..world.dims().y as i32 {
        for x in 0..world.dims().x as i32 {
            let height = surface_height(&world, x, y);
            heights.insert(height);

            if x + 1 < world.dims().x as i32 {
                let right = surface_height(&world, x + 1, y);
                assert!((height - right).abs() <= 1);
            }
            if y + 1 < world.dims().y as i32 {
                let down = surface_height(&world, x, y + 1);
                assert!((height - down).abs() <= 1);
            }
        }
    }

    assert!(heights.len() >= 3, "surface heights were {heights:?}");
}

#[test]
fn ramps_connect_every_step() {
    let world = World::generate(42, Dims::DEFAULT);

    for y in 0..world.dims().y as i32 {
        for x in 0..world.dims().x as i32 {
            let height = surface_height(&world, x, y);
            for (nx, ny) in [(x + 1, y), (x, y + 1)] {
                if nx >= world.dims().x as i32 || ny >= world.dims().y as i32 {
                    continue;
                }

                let neighbour = surface_height(&world, nx, ny);
                if (height - neighbour).abs() == 1 {
                    let (lower_x, lower_y, lower_z) = if height < neighbour {
                        (x, y, height)
                    } else {
                        (nx, ny, neighbour)
                    };
                    assert!(matches!(
                        world.tile(Pos {
                            x: lower_x,
                            y: lower_y,
                            z: lower_z,
                        }),
                        Some(Tile::Ramp(_))
                    ));
                }
            }
        }
    }
}

#[test]
fn five_dwarves_on_walkable_surface() {
    let world = World::generate(42, Dims::DEFAULT);
    let dwarves = world.dwarves();

    assert_eq!(dwarves.len(), 5);
    assert_eq!(
        dwarves
            .iter()
            .map(|(id, _)| *id)
            .collect::<BTreeSet<_>>()
            .len(),
        5
    );
    assert_eq!(
        dwarves
            .iter()
            .map(|(_, pos)| *pos)
            .collect::<BTreeSet<_>>()
            .len(),
        5
    );

    for (_, pos) in dwarves {
        assert_eq!(world.tile(pos), Some(Tile::Empty));
        assert!(matches!(
            world.tile(Pos {
                x: pos.x,
                y: pos.y,
                z: pos.z - 1,
            }),
            Some(Tile::Solid(_))
        ));
    }
}
