use std::collections::BTreeSet;

use sim_core::{DesignationKind, Dims, JobState, Material, Pos, Rect, SimCommand, Tile, World};

fn rect(min: Pos, max: Pos) -> Rect {
    Rect { min, max }
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

#[test]
fn dwarves_stay_standable_and_near_home() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let homes = world.dwarves();

    for _ in 0..200 {
        world.step();
        for (id, pos, _) in world.dwarves() {
            let home = homes
                .iter()
                .find(|(home_id, _, _)| *home_id == id)
                .expect("every dwarf keeps its spawn home")
                .1;
            assert_eq!(world.tile(pos), Some(Tile::Empty));
            assert!(matches!(
                world.tile(Pos {
                    z: pos.z - 1,
                    ..pos
                }),
                Some(Tile::Solid(_) | Tile::Ramp(_))
            ));
            assert_eq!(pos.z, home.z);
            assert!((pos.x - home.x).abs() <= 3, "dwarf {id:?} escaped in x");
            assert!((pos.y - home.y).abs() <= 3, "dwarf {id:?} escaped in y");
        }
    }
}

#[test]
fn same_seed_wanders_identically() {
    let mut first = World::generate(42, Dims::DEFAULT);
    let mut second = World::generate(42, Dims::DEFAULT);

    for _ in 0..200 {
        first.step();
        second.step();
        assert_eq!(first.dwarves(), second.dwarves());
    }
}

#[test]
fn wander_directions_are_not_constant() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let mut previous = world.dwarves()[0].1;
    let mut directions = BTreeSet::new();

    for _ in 0..200 {
        world.step();
        let current = world.dwarves()[0].1;
        if current != previous {
            directions.insert((
                current.x - previous.x,
                current.y - previous.y,
                current.z - previous.z,
            ));
            previous = current;
        }
    }

    assert!(
        directions.len() >= 2,
        "one repeated step vector is not random wandering: {directions:?}"
    );
    assert!(
        directions.iter().any(|(_, dy, _)| *dy != 0),
        "constant candidate zero only bounced on the x axis: {directions:?}"
    );
}

#[test]
fn a_walled_in_dwarf_stays_idle() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let (id, home, _) = world.dwarves()[0];
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        assert!(world.set_tile(
            Pos {
                x: home.x + dx,
                y: home.y + dy,
                z: home.z,
            },
            Tile::Solid(Material::Stone),
        ));
    }

    for _ in 0..25 {
        world.step();
        let dwarf = world
            .dwarves()
            .into_iter()
            .find(|(dwarf_id, _, _)| *dwarf_id == id)
            .expect("walled dwarf remains present");
        assert_eq!(dwarf.1, home);
        assert_eq!(dwarf.2, JobState::Idle);
    }
}

#[test]
fn set_tile_shows_up_once_in_the_dirty_set() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let pos = Pos { x: 3, y: 4, z: 5 };
    let tile = Tile::Solid(Material::Ice);

    assert!(world.set_tile(pos, tile));
    world.step();
    assert_eq!(world.drain_dirty(), vec![(pos, tile)]);

    world.step();
    // NOTE: this proves `drain_dirty` empties the set. It does NOT prove `step()` clears
    // it — nothing here could tell those apart. `stepping_does_not_clear_the_dirty_set`
    // pins which of the two actually holds.
    assert!(world.drain_dirty().is_empty());
}

/// Per-drain, not per-tick (Wolf's ruling, 2026-08-03, resolving AC2's wording against
/// the code): only `drain_dirty` empties the set. Invisible in production because the
/// daemon drains every iteration, but 2.2 and 3.2 are built on top of it.
#[test]
fn stepping_does_not_clear_the_dirty_set() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let pos = Pos { x: 3, y: 4, z: 5 };
    let tile = Tile::Solid(Material::Ice);

    assert!(world.set_tile(pos, tile));
    world.step();
    world.step();
    world.step();

    assert_eq!(
        world.drain_dirty(),
        vec![(pos, tile)],
        "stepping without draining must leave the change pending, never swallow it"
    );
}

#[test]
fn dirty_tiles_are_sorted_and_out_of_bounds_writes_do_nothing() {
    let mut world = World::generate(42, Dims::DEFAULT);
    // Chosen to differ in x, y AND z so the ordering is genuinely pinned to `Pos`'s
    // (x, y, z) lexicographic `Ord`. Positions differing only in x would still pass if
    // the tie-break silently became (x, z, y).
    let first = Pos { x: 1, y: 5, z: 2 };
    let second = Pos { x: 1, y: 5, z: 9 };
    let third = Pos { x: 1, y: 7, z: 0 };
    let fourth = Pos { x: 4, y: 0, z: 0 };
    let out_of_bounds = Pos {
        x: world.dims().x as i32,
        y: 0,
        z: 0,
    };

    // Inserted out of order; the drain must sort them.
    assert!(world.set_tile(third, Tile::Empty));
    assert!(world.set_tile(fourth, Tile::Solid(Material::Stone)));
    assert!(world.set_tile(first, Tile::Ramp(Material::Snow)));
    assert!(world.set_tile(second, Tile::Solid(Material::Ice)));
    assert!(!world.set_tile(out_of_bounds, Tile::Solid(Material::Stone)));
    assert_eq!(world.tile(out_of_bounds), None);
    assert_eq!(
        world.drain_dirty(),
        vec![
            (first, Tile::Ramp(Material::Snow)),
            (second, Tile::Solid(Material::Ice)),
            (third, Tile::Empty),
            (fourth, Tile::Solid(Material::Stone)),
        ]
    );
    assert!(world.drain_dirty().is_empty());
}

#[test]
fn reversed_rect_designates_the_normalized_inclusive_tiles() {
    let mut reversed = World::generate(42, Dims::DEFAULT);
    for y in 2..=3 {
        for x in 1..=2 {
            assert!(reversed.set_tile(Pos { x, y, z: 4 }, Tile::Solid(Material::Stone)));
        }
    }
    reversed.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(Pos { x: 2, y: 3, z: 4 }, Pos { x: 1, y: 2, z: 4 }),
    });

    let expected = vec![
        (Pos { x: 1, y: 2, z: 4 }, DesignationKind::Dig),
        (Pos { x: 1, y: 3, z: 4 }, DesignationKind::Dig),
        (Pos { x: 2, y: 2, z: 4 }, DesignationKind::Dig),
        (Pos { x: 2, y: 3, z: 4 }, DesignationKind::Dig),
    ];
    assert_eq!(reversed.designations(), expected);

    let mut normalized = World::generate(42, Dims::DEFAULT);
    for y in 2..=3 {
        for x in 1..=2 {
            assert!(normalized.set_tile(Pos { x, y, z: 4 }, Tile::Solid(Material::Stone)));
        }
    }
    normalized.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(Pos { x: 1, y: 2, z: 4 }, Pos { x: 2, y: 3, z: 4 }),
    });
    assert_eq!(reversed.designations(), normalized.designations());
}

#[test]
fn designation_rect_clips_to_world_bounds() {
    let mut world = World::generate(42, Dims::DEFAULT);
    make_standable(&mut world, Pos { x: 0, y: 0, z: 1 });
    make_standable(&mut world, Pos { x: 1, y: 0, z: 1 });
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(Pos { x: -1, y: -1, z: 1 }, Pos { x: 1, y: 0, z: 1 }),
    });

    assert_eq!(
        world.designations(),
        vec![
            (Pos { x: 0, y: 0, z: 1 }, DesignationKind::Channel),
            (Pos { x: 1, y: 0, z: 1 }, DesignationKind::Channel),
        ]
    );
}

#[test]
fn fully_out_of_bounds_rect_is_a_no_op() {
    let mut world = World::generate(42, Dims::DEFAULT);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(
            Pos {
                x: -3,
                y: -3,
                z: -3,
            },
            Pos {
                x: -1,
                y: -1,
                z: -1,
            },
        ),
    });
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(
            Pos {
                x: 128,
                y: 128,
                z: 32,
            },
            Pos {
                x: 130,
                y: 130,
                z: 34,
            },
        ),
    });

    assert!(world.designations().is_empty());
    assert!(world.zones().is_empty());
}

#[test]
fn designate_overwrites_the_existing_kind() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let pos = Pos { x: 7, y: 8, z: 9 };
    assert!(world.set_tile(pos, Tile::Solid(Material::Stone)));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(pos, pos),
    });
    make_standable(&mut world, pos);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(pos, pos),
    });

    assert_eq!(world.designations(), vec![(pos, DesignationKind::Channel)]);
}

#[test]
fn each_eraser_leaves_the_other_mark_kind_untouched() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let pos = Pos {
        x: 10,
        y: 10,
        z: 10,
    };
    assert!(world.set_tile(pos, Tile::Solid(Material::Stone)));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(pos, pos),
    });
    make_standable(&mut world, pos);
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(pos, pos),
    });

    world.apply_command(SimCommand::CancelDesignation {
        rect: rect(pos, pos),
    });
    assert!(world.designations().is_empty());
    assert_eq!(world.zones(), vec![pos]);

    make_standable(&mut world, pos);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(pos, pos),
    });
    world.apply_command(SimCommand::RemoveStockpile {
        rect: rect(pos, pos),
    });
    assert_eq!(world.designations(), vec![(pos, DesignationKind::Channel)]);
    assert!(world.zones().is_empty());
}

#[test]
fn designations_keep_only_tiles_workable_by_their_kind() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let solid = Pos { x: 20, y: 20, z: 8 };
    let standable = Pos { x: 21, y: 20, z: 8 };
    let unsupported = Pos { x: 22, y: 20, z: 8 };
    assert!(world.set_tile(solid, Tile::Solid(Material::Stone)));
    make_standable(&mut world, standable);
    assert!(world.set_tile(unsupported, Tile::Empty));
    assert!(world.set_tile(
        Pos {
            z: unsupported.z - 1,
            ..unsupported
        },
        Tile::Empty,
    ));

    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(solid, unsupported),
    });
    assert_eq!(world.designations(), vec![(solid, DesignationKind::Dig)]);

    world.apply_command(SimCommand::CancelDesignation {
        rect: rect(solid, unsupported),
    });
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(solid, unsupported),
    });
    assert_eq!(
        world.designations(),
        vec![(standable, DesignationKind::Channel)]
    );

    world.apply_command(SimCommand::CancelDesignation {
        rect: rect(solid, unsupported),
    });
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(unsupported, unsupported),
    });
    assert!(world.designations().is_empty());
}

#[test]
fn designation_budget_refuses_new_tiles_but_updates_existing_tiles_after_them() {
    let mut world = World::generate(42, Dims::DEFAULT);
    for y in 0..32 {
        for x in 0..128 {
            assert!(world.set_tile(Pos { x, y, z: 8 }, Tile::Solid(Material::Stone)));
        }
    }
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(
            Pos { x: 0, y: 0, z: 8 },
            Pos {
                x: 127,
                y: 31,
                z: 8,
            },
        ),
    });
    assert_eq!(world.designations().len(), 4096);

    let extra = Pos { x: 0, y: 32, z: 8 };
    assert!(world.set_tile(extra, Tile::Solid(Material::Stone)));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(extra, extra),
    });
    assert!(!world.designations().iter().any(|(pos, _)| *pos == extra));

    let refused = Pos { x: 1, y: 0, z: 8 };
    let existing_after = Pos { x: 2, y: 0, z: 8 };
    world.apply_command(SimCommand::CancelDesignation {
        rect: rect(refused, refused),
    });
    make_standable(&mut world, extra);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(extra, extra),
    });
    make_standable(&mut world, refused);
    make_standable(&mut world, existing_after);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(refused, existing_after),
    });

    assert_eq!(world.designations().len(), 4096);
    assert!(!world.designations().iter().any(|(pos, _)| *pos == refused));
    assert!(
        world
            .designations()
            .contains(&(existing_after, DesignationKind::Channel))
    );
}

#[test]
fn stockpile_keeps_exactly_the_standable_tiles() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let first = Pos {
        x: 10,
        y: 10,
        z: 10,
    };
    let second = Pos {
        x: 11,
        y: 10,
        z: 10,
    };
    let unsupported = Pos {
        x: 12,
        y: 10,
        z: 10,
    };
    make_standable(&mut world, first);
    make_standable(&mut world, second);
    assert!(world.set_tile(unsupported, Tile::Empty));
    assert!(world.set_tile(
        Pos {
            z: unsupported.z - 1,
            ..unsupported
        },
        Tile::Empty,
    ));

    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(first, unsupported),
    });

    assert_eq!(world.zones(), vec![first, second]);
}

#[test]
fn stockpile_with_no_standable_tile_changes_nothing() {
    let mut world = World::generate(42, Dims::DEFAULT);
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(Pos { x: 0, y: 0, z: 0 }, Pos { x: 2, y: 2, z: 0 }),
    });

    assert!(world.zones().is_empty());
}

#[test]
fn applying_a_command_does_not_advance_the_world() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let tick = world.tick();
    let dwarves = world.dwarves();

    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(Pos { x: 1, y: 2, z: 3 }, Pos { x: 4, y: 5, z: 3 }),
    });

    assert_eq!(world.tick(), tick);
    assert_eq!(world.dwarves(), dwarves);
}

#[test]
fn same_seed_and_commands_remain_deterministic() {
    let mut first = World::generate(42, Dims::DEFAULT);
    let mut second = World::generate(42, Dims::DEFAULT);
    let channel_pos = first.dwarves()[0].1;
    let commands = [
        SimCommand::Designate {
            kind: DesignationKind::Channel,
            rect: rect(channel_pos, channel_pos),
        },
        SimCommand::PlaceStockpile {
            rect: rect(
                Pos { x: 0, y: 0, z: 0 },
                Pos {
                    x: 20,
                    y: 20,
                    z: 20,
                },
            ),
        },
    ];
    for command in commands {
        first.apply_command(command);
        second.apply_command(command);
    }

    for _ in 0..200 {
        first.step();
        second.step();
        assert_eq!(first.dwarves(), second.dwarves());
        assert_eq!(first.designations(), second.designations());
        assert_eq!(first.zones(), second.zones());
    }
}
