use std::collections::BTreeSet;

use sim_core::{Dims, Id, Material, Pos, Tile, World};

fn surface_height(world: &World, x: i32, y: i32) -> i32 {
    (0..world.dims().z as i32)
        .rev()
        .find(|&z| {
            !matches!(
                world.tile(Pos { x, y, z }),
                Some(
                    Tile::Empty
                        | Tile::Solid(Material::TreeTrunk | Material::TreeFoliage)
                        | Tile::Ramp(Material::TreeTrunk | Material::TreeFoliage)
                )
            )
        })
        .expect("every column has terrain")
}

fn is_standable(world: &World, pos: Pos) -> bool {
    world.tile(pos) == Some(Tile::Empty)
        && matches!(
            world.tile(Pos {
                z: pos.z - 1,
                ..pos
            }),
            Some(Tile::Solid(_) | Tile::Ramp(_))
        )
}

fn tree_trunk_columns(world: &World) -> usize {
    (0..world.dims().y as i32)
        .flat_map(|y| (0..world.dims().x as i32).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            (0..world.dims().z as i32)
                .any(|z| world.tile(Pos { x, y, z }) == Some(Tile::Solid(Material::TreeTrunk)))
        })
        .count()
}

#[test]
fn same_seed_produces_identical_worlds() {
    let first = World::generate(42, Dims::DEFAULT);
    let second = World::generate(42, Dims::DEFAULT);

    assert_eq!(first.tiles(), second.tiles());
    assert_eq!(first.dwarves(), second.dwarves());
}

#[test]
fn default_world_has_mountainous_height_span() {
    assert_eq!(sim_core::DEFAULT_SEED, 0xF005_7E1A);
    let world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let mut heights = Vec::new();
    for y in 0..world.dims().y as i32 {
        for x in 0..world.dims().x as i32 {
            heights.push(surface_height(&world, x, y));
        }
    }
    let minimum = heights.iter().min().copied().unwrap();
    let maximum = heights.iter().max().copied().unwrap();

    assert!(minimum <= 10, "minimum surface height was {minimum}");
    assert!(maximum >= 26, "maximum surface height was {maximum}");
    assert!(
        maximum - minimum >= 16,
        "surface height span was only {} ({minimum}..={maximum})",
        maximum - minimum
    );
}

#[test]
fn generated_world_writes_no_tile_beyond_vertical_bounds() {
    // NOTE: the guard this test exists for is `place_trees`'s `crown_top >= dims.z` skip.
    // Remove it and `tiles[index(..)]` addresses past the grid, so generation panics — a
    // successful generate across seeds that genuinely reach the ceiling IS the assertion.
    // The original version asserted `chunks_exact(plane).rposition(..) < dims.z`, which
    // `chunks_exact` makes true by construction: it could not fail for any implementation.
    // The headroom counter keeps this test from going vacuous the same way if the terrain
    // ever stops reaching the ceiling.
    const MAX_TREE_HEIGHT: i32 = 6;
    let dims = Dims::DEFAULT;
    let mut columns_without_crown_headroom = 0;

    for seed in [sim_core::DEFAULT_SEED, 42, 7] {
        let world = World::generate(seed, dims);
        for y in 0..dims.y as i32 {
            for x in 0..dims.x as i32 {
                if surface_height(&world, x, y) + MAX_TREE_HEIGHT >= dims.z as i32 {
                    columns_without_crown_headroom += 1;
                }
            }
        }
    }

    assert!(
        columns_without_crown_headroom > 0,
        "no column across the sampled seeds came within a full crown of the ceiling, so \
         place_trees' crown-headroom skip was never exercised and this test proves nothing"
    );
}

#[test]
fn camp_is_the_nearest_flat_central_clearing() {
    const RADIUS: i32 = 3;

    let world = World::generate(42, Dims::DEFAULT);
    let camp = world.camp_origin();
    let centre = (world.dims().x as i32 / 2, world.dims().y as i32 / 2);
    let camp_key = (
        (camp.x - centre.0).pow(2) + (camp.y - centre.1).pow(2),
        camp.y,
        camp.x,
    );

    for y in RADIUS..world.dims().y as i32 - RADIUS {
        for x in RADIUS..world.dims().x as i32 - RADIUS {
            let height = surface_height(&world, x, y);
            let flat = (y - RADIUS..=y + RADIUS).all(|ny| {
                (x - RADIUS..=x + RADIUS).all(|nx| surface_height(&world, nx, ny) == height)
            });
            if flat {
                let candidate_key = ((x - centre.0).pow(2) + (y - centre.1).pow(2), y, x);
                assert!(camp_key <= candidate_key);
            }
        }
    }
}

#[test]
fn all_dwarves_spawn_inside_the_camp_with_room_to_move() {
    const RADIUS: i32 = 3;

    let world = World::generate(42, Dims::DEFAULT);
    let camp = world.camp_origin();

    for (_, pos, _, _) in world.dwarves() {
        assert!((pos.x - camp.x).abs() <= RADIUS);
        assert!((pos.y - camp.y).abs() <= RADIUS);
        assert_eq!(pos.z, camp.z);
        assert_eq!(world.tile(pos), Some(Tile::Empty));
        assert!(
            [
                (pos.x - 1, pos.y),
                (pos.x + 1, pos.y),
                (pos.x, pos.y - 1),
                (pos.x, pos.y + 1)
            ]
            .into_iter()
            .any(|(x, y)| is_standable(&world, Pos { x, y, z: pos.z }))
        );
    }
}

#[test]
fn pines_use_both_tree_materials_and_leave_the_camp_clear() {
    let world = World::generate(42, Dims::DEFAULT);
    let camp = world.camp_origin();
    let mut trunks = 0;
    let mut foliage = 0;

    for tile in world.tiles() {
        match tile {
            Tile::Solid(Material::TreeTrunk) => trunks += 1,
            Tile::Solid(Material::TreeFoliage) => foliage += 1,
            _ => {}
        }
    }
    assert!(trunks > 0, "world contains no tree trunks");
    assert!(foliage > 0, "world contains no tree foliage");

    for y in camp.y - 3..=camp.y + 3 {
        for x in camp.x - 3..=camp.x + 3 {
            for z in 0..world.dims().z as i32 {
                assert!(!matches!(
                    world.tile(Pos { x, y, z }),
                    Some(Tile::Solid(Material::TreeTrunk | Material::TreeFoliage))
                ));
            }
        }
    }
}

#[test]
fn default_world_tree_column_count_is_measured_before_density_change() {
    let count = tree_trunk_columns(&World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT));

    assert_eq!(count, 704, "count distinct trunk columns, not trunk cells");
}

#[test]
fn spawn_positions_for_seed_42_are_pinned() {
    let world = World::generate(42, Dims::DEFAULT);
    let positions: Vec<_> = world
        .dwarves()
        .into_iter()
        .map(|(_, pos, _, _)| pos)
        .collect();
    assert_eq!(
        positions,
        vec![
            Pos {
                x: 64,
                y: 65,
                z: 25
            },
            Pos {
                x: 64,
                y: 66,
                z: 25
            },
            Pos {
                x: 65,
                y: 61,
                z: 25
            },
            Pos {
                x: 67,
                y: 66,
                z: 25
            },
            Pos {
                x: 62,
                y: 67,
                z: 25
            },
        ]
    );

    let terrain_fingerprint = world
        .tiles()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, tile| {
            let code = match tile {
                Tile::Empty => 0,
                Tile::Solid(Material::Stone) => 1,
                Tile::Solid(Material::Soil) => 2,
                Tile::Solid(Material::Ice) => 3,
                Tile::Solid(Material::Snow) => 4,
                Tile::Ramp(Material::Stone) => 5,
                Tile::Ramp(Material::Soil) => 6,
                Tile::Ramp(Material::Ice) => 7,
                Tile::Ramp(Material::Snow) => 8,
                Tile::Solid(Material::TreeTrunk) => 9,
                Tile::Solid(Material::TreeFoliage) => 10,
                Tile::Ramp(Material::TreeTrunk) => 11,
                Tile::Ramp(Material::TreeFoliage) => 12,
            };
            (hash ^ code).wrapping_mul(0x0000_0100_0000_01b3)
        });
    assert_eq!(terrain_fingerprint, 0xbd48_ac6b_7250_d2e9);
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
    let first_positions: Vec<Pos> = first
        .dwarves()
        .into_iter()
        .map(|(_, pos, _, _)| pos)
        .collect();
    let second_positions: Vec<Pos> = second
        .dwarves()
        .into_iter()
        .map(|(_, pos, _, _)| pos)
        .collect();
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
                assert!(
                    matches!(
                        world.tile(Pos { x, y, z }),
                        Some(
                            Tile::Empty | Tile::Solid(Material::TreeTrunk | Material::TreeFoliage)
                        )
                    ),
                    "expected air or a tree above the surface at ({x},{y},{z})"
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
                    assert!(
                        (height - right).abs() <= 1,
                        "seed {seed} step ({x},{y})={height} -> ({},{y})={right}",
                        x + 1
                    );
                }
                if y + 1 < world.dims().y as i32 {
                    let down = surface_height(&world, x, y + 1);
                    assert!(
                        (height - down).abs() <= 1,
                        "seed {seed} step ({x},{y})={height} -> ({x},{})={down}",
                        y + 1
                    );
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
        dwarves.iter().map(|(id, _, _, _)| *id).collect::<Vec<_>>(),
        vec![Id(0), Id(1), Id(2), Id(3), Id(4)]
    );
    assert_eq!(
        dwarves
            .iter()
            .map(|(_, pos, _, _)| *pos)
            .collect::<BTreeSet<_>>()
            .len(),
        5
    );

    for (_, pos, _, _) in dwarves {
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
