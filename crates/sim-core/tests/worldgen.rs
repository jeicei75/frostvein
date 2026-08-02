use std::collections::BTreeSet;

use sim_core::{Dims, Id, Material, Pos, Tile, World};

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

    // The tile-array check above passes if a single one of 524288 tiles differs, which
    // the per-column Snow/Ice coin flip alone satisfies. AC9 exists to catch terrain
    // that ignores the seed, so compare the height field itself.
    let mut differing_columns = 0;
    for y in 0..first.dims().y as i32 {
        for x in 0..first.dims().x as i32 {
            if surface_height(&first, x, y) != surface_height(&second, x, y) {
                differing_columns += 1;
            }
        }
    }
    let columns = (first.dims().x * first.dims().y) as i32;
    assert!(
        differing_columns > columns / 10,
        "terrain shape barely responded to the seed: {differing_columns}/{columns} columns differ"
    );

    // ...and the spawn draw must be seeded too, not a constant pick.
    let first_positions: Vec<Pos> = first.dwarves().into_iter().map(|(_, pos)| pos).collect();
    let second_positions: Vec<Pos> = second.dwarves().into_iter().map(|(_, pos)| pos).collect();
    assert_ne!(first_positions, second_positions);
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

    // AC4 also claims a 128x128x32 volume, an ordering (stone below soil below the icy
    // surface) and Air above it. Presence checks alone would pass on a world that
    // scattered stone and soil at random, so assert the ordering per column.
    assert_eq!(world.tiles().len(), 128 * 128 * 32);
    for y in 0..world.dims().y as i32 {
        for x in 0..world.dims().x as i32 {
            let top = surface_height(&world, x, y);

            let mut seen_soil = false;
            for z in 0..top {
                match world.tile(Pos { x, y, z }) {
                    Some(Tile::Solid(Material::Stone)) => assert!(
                        !seen_soil,
                        "stone above soil at ({x},{y},{z}) — layering is inverted"
                    ),
                    Some(Tile::Solid(Material::Soil)) => seen_soil = true,
                    other => panic!("unexpected sub-surface tile {other:?} at ({x},{y},{z})"),
                }
            }
            assert!(seen_soil, "column ({x},{y}) has no soil layer");

            assert!(
                matches!(
                    world.tile(Pos { x, y, z: top }),
                    Some(Tile::Solid(Material::Ice | Material::Snow))
                        | Some(Tile::Ramp(Material::Ice | Material::Snow))
                ),
                "column ({x},{y}) top is not an icy surface"
            );

            for z in top + 1..world.dims().z as i32 {
                assert_eq!(
                    world.tile(Pos { x, y, z }),
                    Some(Tile::Empty),
                    "expected Air above the surface at ({x},{y},{z})"
                );
            }
        }
    }
}

#[test]
fn height_varies_and_steps_are_at_most_one() {
    // AC5 is a property, so check it at several seeds rather than at one lucky sample.
    for seed in [0, 1, 42, 7777] {
        let world = World::generate(seed, Dims::DEFAULT);
        let mut heights = BTreeSet::new();

        for y in 0..world.dims().y as i32 {
            for x in 0..world.dims().x as i32 {
                let height = surface_height(&world, x, y);
                heights.insert(height);

                if x + 1 < world.dims().x as i32 {
                    let right = surface_height(&world, x + 1, y);
                    assert!((height - right).abs() <= 1, "seed {seed}");
                }
                if y + 1 < world.dims().y as i32 {
                    let down = surface_height(&world, x, y + 1);
                    assert!((height - down).abs() <= 1, "seed {seed}");
                }
            }
        }

        assert!(
            heights.len() >= 3,
            "seed {seed} surface heights were {heights:?}"
        );
    }
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
    // AC7 requires the ids come from the world's single monotonic allocator. Asserting
    // only distinctness would also accept random ids, so pin the exact sequence.
    assert_eq!(
        dwarves.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        vec![Id(0), Id(1), Id(2), Id(3), Id(4)]
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
