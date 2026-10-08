use std::collections::{BTreeMap, BTreeSet};

use sim_core::{
    DesignationKind, Dims, DwarfColour, DwarfName, Id, Identity, ItemKind, Job, JobId, JobKind,
    JobState, Material, Pos, Profession, Rect, Refusal, SavedDwarf, SimCommand, Tile, World,
};

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

#[test]
fn a_stockpile_around_the_campfire_never_zones_or_receives_the_fire() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let camp = world.camp_origin();
    let emitters: BTreeSet<Pos> = world.emitters().into_iter().map(|(_, p, _)| p).collect();
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(
            Pos {
                x: camp.x - 2,
                y: camp.y - 2,
                ..camp
            },
            Pos {
                x: camp.x + 2,
                y: camp.y + 2,
                ..camp
            },
        ),
    });
    let emitter_zones: Vec<_> = world
        .zones()
        .into_iter()
        .filter(|p| emitters.contains(p))
        .collect();

    let mut digs = Vec::new();
    for y in camp.y - 7..=camp.y + 7 {
        for x in camp.x - 7..=camp.x + 7 {
            let pos = Pos { x, y, z: camp.z };
            if matches!(world.tile(pos), Some(Tile::Solid(material)) if !matches!(material, Material::TreeTrunk | Material::TreeFoliage))
                && [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                    .into_iter()
                    .any(|(nx, ny)| {
                        is_standable(
                            &world,
                            Pos {
                                x: nx,
                                y: ny,
                                z: camp.z,
                            },
                        )
                    })
            {
                digs.push(pos);
            }
        }
    }
    digs.sort_by_key(|p| (p.x.abs_diff(camp.x) + p.y.abs_diff(camp.y), *p));
    assert!(digs.len() >= 30, "only {} nearby digs", digs.len());
    for pos in digs.into_iter().take(30) {
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: rect(pos, pos),
        });
    }

    let mut previous = world.carrying();
    let mut pickups_after_full = 0;
    let mut max_stones_on_emitter = 0;
    for tick in 0..4_000 {
        world.step();
        let carrying = world.carrying();
        // 12.9 Task 10 (#162) re-pin 2,000 -> 2,500: items block and a pile fills deepest-first, so
        // the hauls take longer. Measured: 24 of 24 cells full by t~2,250 (was by t=2,000); it
        // then stays full with no further pick-up. Cause is the intended rule, not a defect.
        if tick >= 2_500 {
            pickups_after_full += previous
                .iter()
                .zip(&carrying)
                .filter(|((_, before), (_, after))| before.is_none() && after.is_some())
                .count();
        }
        previous = carrying;
        max_stones_on_emitter = max_stones_on_emitter.max(
            world
                .items()
                .iter()
                .filter(|(_, pos)| emitters.contains(pos))
                .count(),
        );
    }
    assert_eq!(max_stones_on_emitter, 0, "stone on an emitter cell");
    assert!(
        emitter_zones.is_empty() && pickups_after_full == 0,
        "zone on an emitter: {emitter_zones:?}; pick-ups after t=2500: {pickups_after_full} (expected 0)"
    );
    let zones = world.zones();
    let filled: BTreeSet<Pos> = world
        .items()
        .into_iter()
        .map(|(_, pos)| pos)
        .filter(|pos| zones.contains(pos))
        .collect();
    assert_eq!(
        filled.len(),
        zones.len(),
        "the pile must fill, or every other assertion here holds with nothing hauled"
    );
}

#[test]
fn a_full_stockpile_never_stacks_uncarried_stones() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let camp = world.camp_origin();
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(
            Pos {
                x: camp.x - 2,
                y: camp.y - 2,
                ..camp
            },
            Pos {
                x: camp.x + 2,
                y: camp.y + 2,
                ..camp
            },
        ),
    });
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(
            Pos {
                x: camp.x - 7,
                y: camp.y - 7,
                ..camp
            },
            Pos {
                x: camp.x + 7,
                y: camp.y + 7,
                ..camp
            },
        ),
    });
    let zones: BTreeSet<_> = world.zones().into_iter().collect();
    assert_eq!(zones.len(), 24);

    for tick in 1..=8_000 {
        world.step();
        let carried: BTreeSet<_> = world
            .carrying()
            .into_iter()
            .filter_map(|(_, id)| id)
            .collect();
        let mut counts: BTreeMap<Pos, usize> = BTreeMap::new();
        for (id, pos) in world.items() {
            if !carried.contains(&id.0) && zones.contains(&pos) {
                let count = counts.entry(pos).or_default();
                *count += 1;
                assert!(
                    *count <= 1,
                    "tick {tick}: {pos:?} holds {count} uncarried stones"
                );
            }
        }
        if tick == 8_000 {
            for zone in &zones {
                assert_eq!(counts.get(zone), Some(&1), "zone {zone:?} is not full");
            }
        }
    }
}

#[test]
fn idle_dwarves_stay_standable_and_inside_the_camp() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let camp = world.camp_origin();

    for _ in 0..200 {
        world.step();
        for (id, pos, _, _) in world.dwarves() {
            assert_eq!(world.tile(pos), Some(Tile::Empty));
            assert!(matches!(
                world.tile(Pos {
                    z: pos.z - 1,
                    ..pos
                }),
                Some(Tile::Solid(_) | Tile::Ramp(_))
            ));
            assert_eq!(pos.z, camp.z);
            assert!((pos.x - camp.x).abs() <= 3, "dwarf {id:?} escaped in x");
            assert!((pos.y - camp.y).abs() <= 3, "dwarf {id:?} escaped in y");
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

/// The trunk tile nearest the camp that has a standable cell beside it at its own level.
fn nearest_trunk_beside_open_ground(world: &World) -> Pos {
    let camp = world.camp_origin();
    let mut trunks = Vec::new();
    for z in 0..world.dims().z as i32 {
        for y in 0..world.dims().y as i32 {
            for x in 0..world.dims().x as i32 {
                let pos = Pos { x, y, z };
                if world.tile(pos) == Some(Tile::Solid(Material::TreeTrunk))
                    && [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                        .into_iter()
                        .any(|(nx, ny)| is_standable(world, Pos { x: nx, y: ny, z }))
                {
                    trunks.push(pos);
                }
            }
        }
    }
    trunks.sort_by_key(|pos| (pos.x.abs_diff(camp.x) + pos.y.abs_diff(camp.y), *pos));
    trunks[0]
}

#[test]
fn trees_do_not_enclose_the_camp_from_outside_cut_work() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let camp = world.camp_origin();
    let target = nearest_trunk_beside_open_ground(&world);
    assert!((target.x - camp.x).abs() > 3 || (target.y - camp.y).abs() > 3);

    // Trees are cut, never dug (12.7): the woodcutter fells the whole tree, trunk tile included.
    assert_eq!(cut(&mut world, target), None);
    for _ in 0..1_000 {
        world.step();
        if world.tile(target) == Some(Tile::Empty) {
            break;
        }
    }

    assert_eq!(world.tile(target), Some(Tile::Empty));
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
    let (id, home, _, _) = world.dwarves()[0];
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
            .find(|(dwarf_id, _, _, _)| *dwarf_id == id)
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
fn designated_tiles_become_one_job_each_only_when_the_schedule_runs() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let first = Pos { x: 30, y: 20, z: 8 };
    let second = Pos { x: 31, y: 20, z: 8 };
    assert!(world.set_tile(first, Tile::Solid(Material::Stone)));
    assert!(world.set_tile(second, Tile::Solid(Material::Soil)));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(first, second),
    });

    assert_eq!(world.designations().len(), 2);
    assert!(
        world.jobs().is_empty(),
        "paused intake must not derive jobs"
    );

    world.step();
    let jobs = world.jobs();
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[0].id, JobId(0));
    assert_eq!(jobs[0].kind, JobKind::Dig);
    assert_eq!(jobs[0].target, first);
    assert_eq!(jobs[0].created_tick, 1);
    assert_eq!(jobs[0].retry_after, 0);
    assert_eq!(jobs[1].id, JobId(1));
    assert_eq!(jobs[1].target, second);

    world.step();
    assert_eq!(world.jobs(), jobs);
}

#[test]
fn unreachable_job_stays_queued_and_retries_after_twenty_ticks() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let target = Pos { x: 40, y: 40, z: 2 };
    assert!(world.set_tile(target, Tile::Solid(Material::Stone)));
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        assert!(world.set_tile(
            Pos {
                x: target.x + dx,
                y: target.y + dy,
                z: target.z,
            },
            Tile::Solid(Material::Stone),
        ));
    }
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(target, target),
    });

    while world.jobs().first().is_none_or(|job| job.retry_after == 0) {
        assert!(world.tick() < 100, "unreachable job was never attempted");
        world.step();
    }
    let released_at = world.tick();
    let retry_after = released_at + 20;
    assert_eq!(world.jobs()[0].retry_after, retry_after);
    assert!(world.claims().iter().all(|(_, job)| job.is_none()));

    while world.tick() + 1 < retry_after {
        world.step();
        assert_eq!(world.jobs()[0].retry_after, retry_after);
        assert!(world.claims().iter().all(|(_, job)| job.is_none()));
    }

    while world.tick() < 200 {
        world.step();
        assert_eq!(world.jobs().len(), 1);
        assert!(world.claims().iter().all(|(_, job)| job.is_none()));
    }
    assert_eq!(world.tile(target), Some(Tile::Solid(Material::Stone)));
    assert!(world.items().is_empty());
}

/// #132: an unreachable dig whose work positions are standable made every idle dwarf's A* flood the
/// whole walkable component, so a handful of them ate the tick's shared budget, skipped the
/// `retry_after` stamp, and starved every job queued behind them.
#[test]
fn unreachable_digs_never_starve_a_reachable_one() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    // A sky plate far above the terrain: stone at z24, and at z25 alternating rows of standable
    // floor and stone targets. Each target's work positions are the floor cells beside it, which
    // are standable but unreachable from the valley.
    let (x0, y0) = (40, 40);
    let mut targets = Vec::new();
    for y in y0..y0 + 20 {
        for x in x0..x0 + 20 {
            let under = Pos { x, y, z: 24 };
            let top = Pos { x, y, z: 25 };
            assert_eq!(world.tile(under), Some(Tile::Empty));
            assert_eq!(world.tile(top), Some(Tile::Empty));
            assert!(world.set_tile(under, Tile::Solid(Material::Stone)));
            if (y - y0) % 2 == 0 {
                assert!(world.set_tile(top, Tile::Solid(Material::Stone)));
                targets.push(top);
            }
        }
    }
    assert_eq!(targets.len(), 200);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(
            Pos {
                x: x0,
                y: y0,
                z: 25,
            },
            Pos {
                x: x0 + 19,
                y: y0 + 19,
                z: 25,
            },
        ),
    });
    for _ in 0..100 {
        world.step();
    }
    assert_eq!(world.jobs().len(), 200);
    assert!(world.claims().iter().all(|(_, job)| job.is_none()));

    let reachable = Pos {
        x: 45,
        y: 62,
        z: 12,
    };
    assert_eq!(world.tile(reachable), Some(Tile::Solid(Material::Stone)));
    assert!(
        [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .any(|(dx, dy)| is_standable(
                &world,
                Pos {
                    x: reachable.x + dx,
                    y: reachable.y + dy,
                    ..reachable
                }
            )),
        "the reachable dig needs a standable neighbour"
    );
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(reachable, reachable),
    });
    let designated_at = world.tick();

    let mut claimed_at = None;
    let mut dug_at = None;
    for _ in 0..600 {
        world.step();
        if claimed_at.is_none() {
            let job = world.jobs().into_iter().find(|job| job.target == reachable);
            if job.is_some_and(|job| world.claims().iter().any(|(_, held)| *held == Some(job.id))) {
                claimed_at = Some(world.tick());
            }
        }
        if world.tile(reachable) == Some(Tile::Empty) {
            dug_at = Some(world.tick());
            break;
        }
        if claimed_at.is_none() {
            assert!(
                world.tick() <= designated_at + 40,
                "the reachable dig was not claimed within 40 ticks of designation"
            );
        }
    }
    let claimed_at = claimed_at.expect("the reachable dig was never claimed");
    assert!(claimed_at <= designated_at + 40);
    let dug_at = dug_at.expect("the reachable dig was not dug within 600 ticks");
    println!(
        "claimed +{} dug +{}",
        claimed_at - designated_at,
        dug_at - designated_at
    );

    // LAST, so no earlier assertion absorbs a mutation of the stamp: FR8, every unreachable dig is
    // still designated and queued, and was retried after the reachable dig was designated.
    let designations = world.designations();
    let jobs = world.jobs();
    for target in targets {
        assert!(designations.contains(&(target, DesignationKind::Dig)));
        let job = jobs
            .iter()
            .find(|job| job.target == target)
            .expect("an unreachable dig stays queued");
        // A stamp at tick t sets t + RETRY_COOLDOWN (20); more than designated_at + 20 means the
        // stamp itself came after designation.
        assert!(
            job.retry_after > designated_at + 20,
            "job {:?} at {:?} was never retried: retry_after={}",
            job.id,
            target,
            job.retry_after
        );
    }
}

/// FR51: a channel worker whose own ground is removed under it lets go of the job and the crew goes
/// on with other work. The channel job and its designation stay (Wolf's 2026-08-06 ruling).
#[test]
fn a_channel_worker_whose_support_is_removed_lets_go_and_the_crew_goes_on() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let dwarf = world.dwarves()[0].1;
    let t = Pos {
        x: dwarf.x + 2,
        ..dwarf
    };
    let below = Pos { z: t.z - 1, ..t };
    assert!(is_standable(&world, t));
    assert!(matches!(world.tile(below), Some(Tile::Solid(_))));
    assert!(matches!(
        world.tile(Pos {
            z: below.z - 1,
            ..t
        }),
        Some(Tile::Solid(_))
    ));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(t, t),
    });
    // 12.9 AC13: the worker stands beside the target, so the support removed is the one under HIS
    // tile (the target's own support stays).
    let (holder, stand) = loop {
        assert!(world.tick() < 200, "the channel was never worked");
        world.step();
        if let Some((id, pos, ..)) = world.dwarves().into_iter().find(|(_, pos, state, _)| {
            *state == JobState::Work
                && pos.z == t.z
                && pos.x.abs_diff(t.x) + pos.y.abs_diff(t.y) == 1
        }) {
            break (id, pos);
        }
    };
    assert!(world.set_tile(
        Pos {
            z: stand.z - 1,
            ..stand
        },
        Tile::Empty
    ));

    let reachable = Pos {
        x: 45,
        y: 62,
        z: 12,
    };
    assert_eq!(world.tile(reachable), Some(Tile::Solid(Material::Stone)));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(reachable, reachable),
    });
    let removed_at = world.tick();
    for _ in 0..2 {
        world.step();
    }
    assert_eq!(
        world.claims().into_iter().find(|(id, _)| *id == holder),
        Some((holder, None)),
        "the holder must let go within 2 ticks of losing its support"
    );
    assert!(world.tick() <= removed_at + 2);
    assert!(
        world
            .jobs()
            .iter()
            .any(|job| job.kind == JobKind::Channel && job.target == t)
    );
    assert!(
        world
            .designations()
            .contains(&(t, DesignationKind::Channel))
    );

    for _ in 0..600 {
        if world.tile(reachable) == Some(Tile::Empty) {
            break;
        }
        world.step();
    }
    assert_eq!(world.tile(reachable), Some(Tile::Empty));
    // 12.9 AC13: the target keeps its own support and has other neighbours, so another miner may
    // finish the channel from one of them. What must never happen is the order vanishing unworked.
    assert!(
        world
            .designations()
            .contains(&(t, DesignationKind::Channel))
            || matches!(world.tile(below), Some(Tile::Ramp(_))),
        "the channel order vanished without being worked"
    );
}

/// AC13 (12.9 seat pass 1): a channel is worked from a walkable same-z 4-neighbour of the target,
/// never from the target itself, and the stone still lands on the target.
#[test]
fn a_channel_is_worked_from_the_next_tile_and_the_stone_lands_on_the_target() {
    let (mut world, holder, _, job) = channel_held_by_a_miner();
    let t = job.target;
    let mut worked_from = None;
    for _ in 0..600 {
        let (_, pos, state, _) = world
            .dwarves()
            .into_iter()
            .find(|(id, ..)| *id == holder)
            .unwrap();
        assert_ne!(pos, t, "the channel miner stood on the cell it channels");
        if state == JobState::Work {
            worked_from = Some(pos);
        }
        if world.tile(Pos { z: t.z - 1, ..t }) != Some(Tile::Solid(Material::Stone))
            && !world
                .designations()
                .contains(&(t, DesignationKind::Channel))
        {
            break;
        }
        world.step();
    }
    let from = worked_from.expect("the channel was never worked");
    assert_eq!(from.z, t.z);
    assert_eq!(from.x.abs_diff(t.x) + from.y.abs_diff(t.y), 1);
    assert!(
        world.items().iter().any(|(_, pos)| *pos == t),
        "the stone did not land on the channelled cell"
    );
}

#[test]
fn cancelling_a_claimed_dig_releases_the_dwarf_without_touching_the_tile() {
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
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(target, target),
    });
    let holder = loop {
        assert!(world.tick() < 100, "reachable job was never claimed");
        world.step();
        if let Some((id, _)) = world.claims().into_iter().find(|(_, job)| job.is_some()) {
            break id;
        }
    };

    world.apply_command(SimCommand::CancelDesignation {
        rect: rect(target, target),
    });

    assert!(world.designations().is_empty());
    assert!(world.jobs().is_empty());
    assert_eq!(
        world.claims().into_iter().find(|(id, _)| *id == holder),
        Some((holder, None))
    );
    assert_eq!(
        world
            .dwarves()
            .into_iter()
            .find(|(id, _, _, _)| *id == holder)
            .map(|(_, _, state, _)| state),
        Some(JobState::Idle)
    );
    assert_eq!(world.tile(target), Some(Tile::Solid(Material::Stone)));
    assert!(world.items().is_empty());
}

#[test]
fn designate_delay_claim_walk_work_and_dig_complete_headlessly() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let worker = world.dwarves()[2].1;
    let dx = if worker.x + 10 < world.dims().x as i32 {
        1
    } else {
        -1
    };
    for distance in 1..10 {
        make_standable(
            &mut world,
            Pos {
                x: worker.x + dx * distance,
                ..worker
            },
        );
    }
    let target = Pos {
        x: worker.x + dx * 10,
        ..worker
    };
    assert!(world.set_tile(target, Tile::Solid(Material::Stone)));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(target, target),
    });

    let mut saw_delay = false;
    let mut saw_claim = false;
    let mut saw_walk = false;
    let mut saw_work = false;
    for _ in 0..300 {
        world.step();
        saw_delay |=
            !world.jobs().is_empty() && world.claims().iter().all(|(_, job)| job.is_none());
        if let Some((holder, _)) = world.claims().into_iter().find(|(_, job)| job.is_some()) {
            saw_claim = true;
            let state = world
                .dwarves()
                .into_iter()
                .find(|(id, _, _, _)| *id == holder)
                .expect("claim holder remains a dwarf")
                .2;
            saw_walk |= state == JobState::Walk;
            saw_work |= state == JobState::Work;
        }
        if world.tile(target) == Some(Tile::Empty) {
            break;
        }
    }

    assert!(saw_delay && saw_claim && saw_walk && saw_work);
    assert_eq!(world.tile(target), Some(Tile::Empty));
    assert_eq!(world.items(), vec![(sim_core::Id(10), target)]);
    assert!(world.jobs().is_empty());
    assert!(world.claims().iter().all(|(_, job)| job.is_none()));
    assert!(world.designations().is_empty());
}

/// The walking-skeleton sentence, headless: no client, no network. Order a dig, put a stockpile
/// down, and the stone ends up on the pile with the market empty — and two worlds built from the
/// same seed and given the same commands agree at every tick while it happens (FR26, FR15).
#[test]
fn designate_dig_stockpile_haul_and_the_stone_reaches_the_pile_headlessly() {
    let mut first = World::generate(42, Dims::DEFAULT);
    let mut second = World::generate(42, Dims::DEFAULT);
    let worker = first.dwarves()[2].1;
    let dx = if worker.x + 8 < first.dims().x as i32 {
        1
    } else {
        -1
    };
    let cell = |steps: i32| Pos {
        x: worker.x + dx * steps,
        ..worker
    };
    let target = cell(7);
    let pile = cell(2);
    for world in [&mut first, &mut second] {
        for steps in 1..=6 {
            make_standable(world, cell(steps));
        }
        // The dug tile keeps its floor, so the stone it drops lands on standable ground. Items
        // never fall, so without this the stone would be unreachable and the loop could not close.
        assert!(world.set_tile(
            Pos {
                z: target.z - 1,
                ..target
            },
            Tile::Solid(Material::Stone),
        ));
        assert!(world.set_tile(target, Tile::Solid(Material::Stone)));
        world.drain_dirty();
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: rect(target, target),
        });
        world.apply_command(SimCommand::PlaceStockpile {
            rect: rect(pile, pile),
        });
    }
    assert_eq!(first.tile(target), Some(Tile::Solid(Material::Stone)));

    // Bounded by a tick guard rather than a fixed step count: two reaction delays of 5..=30, two
    // walks and two WORK_TICKS runs is the budget this is sized against.
    loop {
        assert!(
            first.tick() < 400,
            "the loop never closed — jobs {:?}, items {:?}, carrying {:?}",
            first.jobs(),
            first.items(),
            first.carrying()
        );
        first.step();
        second.step();
        assert_eq!(first.dwarves(), second.dwarves());
        assert_eq!(first.jobs(), second.jobs());
        assert_eq!(first.claims(), second.claims());
        assert_eq!(first.carrying(), second.carrying());
        assert_eq!(first.identities(), second.identities());
        assert_eq!(first.professions(), second.professions());
        assert_eq!(first.items(), second.items());
        // AC10, checked on every tick of a real haul: a stone is only ever held by a dwarf that
        // is holding that stone's job.
        for (id, item) in first.carrying() {
            if item.is_some() {
                let held = first
                    .claims()
                    .into_iter()
                    .find(|(claim_id, _)| *claim_id == id)
                    .and_then(|(_, job)| job);
                let kind = held.and_then(|job| {
                    first
                        .jobs()
                        .into_iter()
                        .find(|queued| queued.id == job)
                        .map(|queued| queued.kind)
                });
                assert_eq!(
                    kind,
                    item.map(|item| JobKind::Haul { item }),
                    "dwarf {id:?} carries a stone whose haul job it does not hold"
                );
            }
        }
        if first.jobs().is_empty() && first.items().iter().any(|(_, pos)| *pos == pile) {
            break;
        }
    }

    // Compared once at the end rather than every tick: half a million tiles times 400 ticks is
    // real time, and `same_seed_and_commands_remain_deterministic` pins the per-tick case.
    assert_eq!(first.tiles(), second.tiles());
    assert_eq!(
        first.tile(target),
        Some(Tile::Empty),
        "the ordered tile was never dug"
    );
    assert_eq!(first.items(), vec![(sim_core::Id(10), pile)]);
    assert!(first.zones().contains(&pile));
    assert!(first.jobs().is_empty());
    assert!(first.claims().iter().all(|(_, job)| job.is_none()));
    assert!(first.carrying().iter().all(|(_, item)| item.is_none()));
    assert!(first.designations().is_empty());
}

#[test]
fn two_deep_dig_advances_from_the_exposed_face() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let worker = world.dwarves()[2].1;
    let dx = if worker.x + 8 < world.dims().x as i32 {
        1
    } else {
        -1
    };
    for distance in 1..6 {
        make_standable(
            &mut world,
            Pos {
                x: worker.x + dx * distance,
                ..worker
            },
        );
    }
    let outer = Pos {
        x: worker.x + dx * 6,
        ..worker
    };
    let inner = Pos {
        x: worker.x + dx * 7,
        ..worker
    };
    assert!(world.set_tile(
        Pos {
            z: outer.z - 1,
            ..outer
        },
        Tile::Solid(Material::Stone),
    ));
    assert!(world.set_tile(outer, Tile::Solid(Material::Stone)));
    assert!(world.set_tile(inner, Tile::Solid(Material::Stone)));
    for sealed in [
        Pos {
            x: inner.x + dx,
            ..inner
        },
        Pos {
            y: inner.y - 1,
            ..inner
        },
        Pos {
            y: inner.y + 1,
            ..inner
        },
    ] {
        assert!(world.set_tile(sealed, Tile::Solid(Material::Stone)));
    }
    // 12.9 Task 10 (#162), intended change: the outer dig's stone blocks the one-wide tunnel
    // until it is hauled, and without a pile the dig would stop there (FR8 never-drop). So a
    // one-cell pile stands beside the tunnel's mouth, and the inner dig now waits for the haul.
    let pile = [-1, 1]
        .into_iter()
        .map(|dy| Pos {
            y: worker.y + dy,
            ..worker
        })
        .find(|cell| is_standable(&world, *cell))
        .expect("open ground beside the worker for the pile");
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(pile, pile),
    });
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(outer, inner),
    });

    for _ in 0..1_500 {
        world.step();
        if world.tile(inner) == Some(Tile::Empty) {
            break;
        }
    }

    assert_eq!(world.tile(outer), Some(Tile::Empty));
    assert_eq!(world.tile(inner), Some(Tile::Empty));
    // The outer stone was hauled out of the way (the pile is full); the inner one lies where dug.
    assert!(world.items().iter().any(|(_, pos)| *pos == pile));
    assert!(world.items().iter().any(|(_, pos)| *pos == inner));
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

/// Digs the tile beside dwarf 2 and returns the position the stone landed on. There is no
/// stockpile yet, so no haul job is created while this runs.
fn dig_one_stone(world: &mut World) -> Pos {
    let worker = world.dwarves()[2].1;
    let target = Pos {
        x: if worker.x + 1 < world.dims().x as i32 {
            worker.x + 1
        } else {
            worker.x - 1
        },
        ..worker
    };
    assert!(world.set_tile(
        Pos {
            z: target.z - 1,
            ..target
        },
        Tile::Solid(Material::Stone),
    ));
    assert!(world.set_tile(target, Tile::Solid(Material::Stone)));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(target, target),
    });
    while world.items().is_empty() {
        assert!(
            world.tick() < 200,
            "the adjacent dig never produced a stone"
        );
        world.step();
    }
    assert!(world.jobs().is_empty());
    target
}

#[test]
fn a_new_stockpile_derives_no_haul_job_until_the_world_steps() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let stone = dig_one_stone(&mut world);
    let pile = world.dwarves()[0].1;
    assert_ne!(pile, stone);

    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(pile, pile),
    });

    assert!(
        world.jobs().is_empty(),
        "command intake derived work from a stone without a tick — a paused daemon would haul"
    );

    world.step();

    assert_eq!(world.jobs().len(), 1);
    assert_eq!(world.jobs()[0].kind, JobKind::Haul { item: 10 });
    assert_eq!(world.jobs()[0].target, stone);
}

#[test]
fn cancelling_marks_over_a_stone_never_drops_its_haul_job() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let stone = dig_one_stone(&mut world);
    let pile = world.dwarves()[0].1;
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(pile, pile),
    });
    world.step();
    let job = world.jobs()[0];
    assert_eq!(job.kind, JobKind::Haul { item: 10 });

    // `x` over the stone's tile. A haul job's `target` is a stone position, so a cancel that
    // matched on `target` would silently delete an order the player never gave.
    world.apply_command(SimCommand::CancelDesignation {
        rect: rect(stone, stone),
    });

    assert!(
        world
            .jobs()
            .iter()
            .any(|queued| queued.id == job.id && queued.kind == job.kind),
        "cancelling marks at the stone's tile dropped its haul job: {:?}",
        world.jobs()
    );
}

/// Steps once and asserts no stone jumped. A carried stone rides its carrier and a carrier moves
/// at most one tile per tick, so a stone that teleports means something read a stale position —
/// a haul job's `target` — instead of the stone's live one.
fn step_without_teleporting_a_stone(world: &mut World) {
    let before = world.items();
    world.step();
    for (id, pos) in world.items() {
        if let Some((_, was)) = before.iter().find(|(old, _)| *old == id) {
            let step = (pos.x - was.x)
                .abs()
                .max((pos.y - was.y).abs())
                .max((pos.z - was.z).abs());
            assert!(step <= 1, "stone {id:?} jumped from {was:?} to {pos:?}");
        }
    }
}

#[test]
fn removing_every_stockpile_drops_the_carried_stone_and_a_new_pile_revives_the_job() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let stone = dig_one_stone(&mut world);
    // Carved in whichever direction has six tiles of room, which need not be the direction the
    // dig went — seed 42's digger works near the eastern edge.
    let dx = if stone.x + 6 < world.dims().x as i32 {
        1
    } else {
        -1
    };
    for distance in 1..=6 {
        make_standable(
            &mut world,
            Pos {
                x: stone.x + dx * distance,
                ..stone
            },
        );
    }
    let pile = Pos {
        x: stone.x + dx * 6,
        ..stone
    };
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(pile, pile),
    });

    while world.carrying().iter().all(|(_, item)| item.is_none()) {
        assert!(world.tick() < 400, "nobody ever picked the stone up");
        step_without_teleporting_a_stone(&mut world);
    }
    // Let the carrier get at least two tiles from the tile it picked the stone up on, so the
    // stone is dropped somewhere its job's `target` no longer names — and far enough that a
    // pick-up that read the stale `target` would have to teleport the stone to reach it.
    let source = world.items()[0].1;
    let steps_from_source = |pos: Pos| {
        (pos.x - source.x)
            .abs()
            .max((pos.y - source.y).abs())
            .max((pos.z - source.z).abs())
    };
    while steps_from_source(world.items()[0].1) < 2 {
        assert!(
            world.tick() < 500,
            "the carrier never got two tiles from the tile it picked up on"
        );
        step_without_teleporting_a_stone(&mut world);
    }

    world.apply_command(SimCommand::RemoveStockpile {
        rect: rect(pile, pile),
    });
    // 40 ticks covered the carrier's remaining walk while a dwarf stepped once per tick. He now
    // rests STEP_REST_TICKS between steps, so the same walk takes eleven times as long; this is
    // the old budget scaled by that pacing, not a number raised until the test passed.
    for _ in 0..440 {
        step_without_teleporting_a_stone(&mut world);
    }
    // NOTE: `execute_jobs` only recomputes a path when the cached one runs out, so a carrier
    // whose goal set just emptied finishes the walk it was on and drops at the end of it. That is
    // benign — the drop itself still requires standing on a work position — but it does mean the
    // stone parks a walk away from where the pile was removed, not on the spot.
    let parked = world.items()[0].1;
    assert!(
        steps_from_source(parked) >= 2,
        "the stone was dropped {parked:?}, too close to the pick-up tile {source:?}"
    );

    assert!(world.zones().is_empty());
    assert_eq!(
        world.jobs().len(),
        1,
        "the haul job was dropped, not parked"
    );
    assert_eq!(world.jobs()[0].kind, JobKind::Haul { item: 10 });
    assert!(
        world.claims().iter().all(|(_, job)| job.is_none()),
        "a job with nowhere to deliver stayed claimed"
    );
    assert!(
        world.carrying().iter().all(|(_, item)| item.is_none()),
        "a dwarf kept holding the stone with the pile gone"
    );
    let dropped = world.items()[0].1;
    assert_eq!(world.tile(dropped), Some(Tile::Empty));

    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(pile, pile),
    });
    for _ in 0..400 {
        step_without_teleporting_a_stone(&mut world);
        if world.jobs().is_empty() {
            break;
        }
    }

    assert!(
        world.jobs().is_empty(),
        "the revived job never finished: {:?}",
        world.jobs()
    );
    assert_eq!(world.items(), vec![(sim_core::Id(10), pile)]);
    assert!(world.carrying().iter().all(|(_, item)| item.is_none()));
}

/// Two carriers converging on the LAST free stockpile tile is a real race, found at 3.3's review:
/// the first delivers, and the second — standing on a tile that has just left its goal set — is
/// retried, and `release_claim` drops its stone on a nearby open tile. The extra stone remains
/// loose and can be hauled once another stockpile cell opens.
#[test]
fn two_carriers_racing_for_the_last_tile_do_not_leave_a_permanent_stack() {
    let dims = Dims { x: 12, y: 3, z: 3 };
    let index = |pos: Pos| {
        pos.x as usize
            + pos.y as usize * dims.x as usize
            + pos.z as usize * dims.x as usize * dims.y as usize
    };
    let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    for x in 0..dims.x as i32 {
        for y in 0..dims.y as i32 {
            tiles[index(Pos { x, y, z: 0 })] = Tile::Solid(Material::Stone);
        }
    }
    let pile = Pos { x: 1, y: 1, z: 1 };
    let spare = Pos { x: 1, y: 2, z: 1 };
    let first = Pos { x: 2, y: 1, z: 1 };
    let second = Pos { x: 3, y: 1, z: 1 };

    // Both dwarves are already carrying and one tile from the pile — the state the race produces
    // in play, built directly so no timing accident can hide it.
    let mut save = World::generate(42, Dims::DEFAULT).to_save();
    save.dims = dims;
    save.tiles = tiles;
    save.tick = 100;
    save.designations.clear();
    save.zones = vec![pile];
    save.dwarves = vec![
        SavedDwarf {
            id: 0,
            pos: first,
            state: JobState::Walk,
            home: first,
            cooldown: 0,
            current_job: Some(0),
            work_progress: 0,
            carrying: Some(2),
            identity: Identity {
                name: DwarfName::Durin,
                colour: DwarfColour::Red,
            },
            profession: Profession::Hauler,
            path: Vec::new(),
        },
        SavedDwarf {
            id: 1,
            pos: second,
            state: JobState::Walk,
            home: second,
            cooldown: 0,
            current_job: Some(1),
            work_progress: 0,
            carrying: Some(3),
            identity: Identity {
                name: DwarfName::Nori,
                colour: DwarfColour::Blue,
            },
            profession: Profession::Hauler,
            path: Vec::new(),
        },
    ];
    save.items = vec![(2, first, ItemKind::Stone), (3, second, ItemKind::Stone)];
    save.next_id = 4;
    save.jobs = vec![
        Job {
            id: JobId(0),
            kind: JobKind::Haul { item: 2 },
            target: first,
            created_tick: 0,
            retry_after: 0,
        },
        Job {
            id: JobId(1),
            kind: JobKind::Haul { item: 3 },
            target: second,
            created_tick: 0,
            retry_after: 0,
        },
    ];
    save.next_job_id = 2;

    let mut world = World::from_save(save);
    let stored_on = |world: &World, tile: Pos| -> Vec<u32> {
        let carried: Vec<u32> = world
            .carrying()
            .into_iter()
            .filter_map(|(_, item)| item)
            .collect();
        world
            .items()
            .into_iter()
            .filter(|(id, pos)| *pos == tile && !carried.contains(&id.0))
            .map(|(id, _)| id.0)
            .collect()
    };

    // Let the race resolve. Neither delivery nor retry may leave two uncarried stones on the pile.
    for _ in 0..200 {
        world.step();
        assert!(stored_on(&world, pile).len() <= 1, "race stacked the pile");
    }
    assert_eq!(stored_on(&world, pile).len(), 1, "no delivery occurred");
    assert!(world.carrying().iter().all(|(_, item)| item.is_none()));
    assert_eq!(world.items().len(), 2, "the retry lost its stone");
    assert_eq!(world.jobs().len(), 1, "the loose stone lost its haul job");

    // Now give the pile somewhere to put the extra stone. It must sort itself out with no further
    // intervention, and end with one stone per tile.
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(spare, spare),
    });
    for _ in 0..400 {
        world.step();
        if world.jobs().is_empty() {
            break;
        }
    }

    assert_eq!(
        stored_on(&world, pile).len(),
        1,
        "the pile tile still holds a stack: {:?}",
        world.items()
    );
    assert_eq!(
        stored_on(&world, spare).len(),
        1,
        "the extra stone never reached the new tile: {:?}",
        world.items()
    );
    assert!(
        world.jobs().is_empty(),
        "jobs left over: {:?}",
        world.jobs()
    );
    assert!(world.carrying().iter().all(|(_, item)| item.is_none()));
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

/// Seed 42, a corridor beside the first miner with 20 reachable digs on either side of it and a
/// 2-cell pile at its near end. (RED before 12.4: the first delivery came with 0 marks left.)
fn backlog_world() -> (World, BTreeSet<Pos>) {
    let mut world = World::generate(42, Dims::DEFAULT);
    let miner = world
        .professions()
        .into_iter()
        .find(|(_, profession)| *profession == Profession::Miner)
        .unwrap()
        .0;
    let worker = world
        .dwarves()
        .into_iter()
        .find(|(id, ..)| *id == miner)
        .unwrap()
        .1;
    let dx = if worker.x + 16 < world.dims().x as i32 {
        1
    } else {
        -1
    };
    let cell = |s: i32, dy: i32| Pos {
        x: worker.x + dx * s,
        y: worker.y + dy,
        ..worker
    };
    for s in 1..=13 {
        make_standable(&mut world, cell(s, 0));
    }
    let mut targets = Vec::new();
    for s in 3..13 {
        for dy in [-1, 1] {
            let t = cell(s, dy);
            // The floor under the target, so the stone it yields lands standable.
            world.set_tile(Pos { z: t.z - 1, ..t }, Tile::Solid(Material::Stone));
            world.set_tile(t, Tile::Solid(Material::Stone));
            targets.push(t);
        }
    }
    world.drain_dirty();
    for t in &targets {
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: rect(*t, *t),
        });
    }
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(cell(1, 0), cell(2, 0)),
    });
    let zones: BTreeSet<Pos> = world.zones().into_iter().collect();
    (world, zones)
}

fn stone_on_the_pile(world: &World, zones: &BTreeSet<Pos>) -> bool {
    world.items().iter().any(|(_, pos)| zones.contains(pos))
}

#[test]
fn hauling_starts_while_the_dig_backlog_is_still_queued() {
    let (mut world, zones) = backlog_world();
    assert_eq!(world.designations().len(), 20);

    while !stone_on_the_pile(&world, &zones) {
        assert!(world.tick() < 5000, "no stone ever reached the pile");
        world.step();
    }

    let marks_left = world.designations().len();
    println!(
        "first delivery at tick {}: {marks_left} marks left",
        world.tick()
    );
    assert!(
        marks_left > 5,
        "the first delivery came with only {marks_left} of 20 marks left"
    );
}

#[test]
fn each_trade_holds_only_its_own_jobs_and_the_woodcutter_wanders() {
    let (mut world, zones) = backlog_world();
    let professions = world.professions();
    let woodcutter = professions
        .iter()
        .find(|(_, p)| *p == Profession::Woodcutter)
        .unwrap()
        .0;
    let position_of = |world: &World| {
        world
            .dwarves()
            .into_iter()
            .find(|(id, ..)| *id == woodcutter)
            .unwrap()
            .1
    };
    let mut woodcutter_moves = 0;
    let mut last = position_of(&world);
    let mut delivered_at = None;
    let mut holders = 0;
    loop {
        world.step();
        assert!(world.tick() < 5000, "no stone ever reached the pile");
        if delivered_at.is_none() && stone_on_the_pile(&world, &zones) {
            delivered_at = Some(world.tick());
        }
        if delivered_at.is_some_and(|tick| world.tick() >= tick + 200) {
            break;
        }
        let jobs = world.jobs();
        for (id, held) in world.claims() {
            let profession = professions.iter().find(|(i, _)| *i == id).unwrap().1;
            let Some(held) = held else { continue };
            holders += 1;
            let kind = jobs.iter().find(|job| job.id == held).unwrap().kind;
            match (profession, kind) {
                (Profession::Miner, JobKind::Dig | JobKind::Channel)
                | (Profession::Hauler, JobKind::Haul { .. }) => {}
                _ => panic!(
                    "tick {}: {profession:?} {id:?} holds {kind:?}",
                    world.tick()
                ),
            }
        }
        let now = position_of(&world);
        if now != last {
            woodcutter_moves += 1;
            last = now;
        }
    }

    assert!(
        holders > 0,
        "nobody ever held a job, so the check proved nothing"
    );
    assert!(
        woodcutter_moves >= 1,
        "the jobless woodcutter never wandered"
    );
}

/// DEFAULT_SEED, one reachable channel mark beside dwarf 0. Steps until a miner holds it AND the
/// other miner is eligible (every reaction delay is 5-30 ticks from `created_tick`) and idle.
/// Returns the world, the holder, the other miner and the job.
fn channel_held_by_a_miner() -> (World, Id, Id, Job) {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let dwarf = world.dwarves()[0].1;
    let t = Pos {
        x: dwarf.x + 2,
        ..dwarf
    };
    assert!(is_standable(&world, t));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(t, t),
    });
    let miners: Vec<Id> = world
        .professions()
        .into_iter()
        .filter(|(_, p)| *p == Profession::Miner)
        .map(|(id, _)| id)
        .collect();
    assert_eq!(miners.len(), 2);
    loop {
        assert!(world.tick() < 200, "no miner ever held the channel");
        world.step();
        let job = world
            .jobs()
            .into_iter()
            .find(|j| j.kind == JobKind::Channel);
        let Some(job) = job else { continue };
        if world.tick() < job.created_tick + 31 {
            continue;
        }
        let claims = world.claims();
        if let Some(holder) = miners
            .iter()
            .copied()
            .find(|m| claims.contains(&(*m, Some(job.id))))
        {
            let other = miners.iter().copied().find(|m| *m != holder).unwrap();
            assert!(
                claims.contains(&(other, None)),
                "the other miner must be idle: {claims:?}"
            );
            return (world, holder, other, job);
        }
    }
}

#[test]
fn a_reassigned_miner_lets_go_and_the_other_miner_takes_the_same_job() {
    let (mut world, holder, other, job) = channel_held_by_a_miner();
    let refusal = world.apply_command(SimCommand::SetProfession {
        dwarf: holder,
        profession: Profession::Hauler,
    });
    assert_eq!(refusal, None);
    world.step();
    let claims = world.claims();
    assert!(
        !claims
            .iter()
            .any(|(id, held)| *id == holder && held.is_some_and(|j| j == job.id)),
        "the reassigned miner still holds the channel: {claims:?}"
    );
    assert!(
        world.jobs().iter().any(|j| j.id == job.id),
        "the released job left the queue"
    );
    assert!(
        claims.contains(&(other, Some(job.id))),
        "the other miner did not take the same JobId in one step: {claims:?}"
    );
    for _ in 0..600 {
        if world.tile(job.target) != Some(Tile::Solid(Material::Stone))
            && !world
                .designations()
                .contains(&(job.target, DesignationKind::Channel))
        {
            break;
        }
        world.step();
    }
    assert!(
        !world
            .designations()
            .contains(&(job.target, DesignationKind::Channel)),
        "the channel mark was never worked off"
    );
}

#[test]
fn a_reassigned_hauler_puts_the_stone_down_and_another_hauler_delivers_it() {
    let (mut world, zones) = backlog_world();
    let (carrier, item) = loop {
        assert!(world.tick() < 5000, "no hauler ever carried a stone");
        world.step();
        let positions = world.dwarves();
        let found = world.carrying().into_iter().find_map(|(id, held)| {
            let item = held?;
            let pos = positions.iter().find(|d| d.0 == id)?.1;
            (!zones.contains(&pos)).then_some((id, item))
        });
        if let Some(found) = found {
            break found;
        }
    };
    world.apply_command(SimCommand::SetProfession {
        dwarf: carrier,
        profession: Profession::Miner,
    });
    assert!(
        world.carrying().contains(&(carrier, None)),
        "the reassigned hauler still carries"
    );
    assert!(world.items().iter().any(|(id, _)| id.0 == item));
    assert!(
        world
            .jobs()
            .iter()
            .any(|j| j.kind == (JobKind::Haul { item })),
        "the haul job for the dropped stone is gone"
    );
    assert!(world.claims().contains(&(carrier, None)));
    loop {
        assert!(
            world.tick() < 5000,
            "the dropped stone never reached the pile"
        );
        world.step();
        if world
            .items()
            .iter()
            .any(|(id, pos)| id.0 == item && zones.contains(pos))
        {
            break;
        }
    }
}

#[test]
fn a_trade_change_to_the_same_trade_keeps_the_job() {
    let (mut world, holder, _, job) = channel_held_by_a_miner();
    let before = world.claims();
    let refusal = world.apply_command(SimCommand::SetProfession {
        dwarf: holder,
        profession: Profession::Miner,
    });
    assert_eq!(refusal, None);
    assert_eq!(world.claims(), before);
    assert!(world.claims().contains(&(holder, Some(job.id))));
    world.step();
    assert!(world.claims().contains(&(holder, Some(job.id))));
}

#[test]
fn an_unknown_dwarf_is_refused_and_an_emptied_trade_is_allowed() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let before = world.professions();
    let refusal = world.apply_command(SimCommand::SetProfession {
        dwarf: Id(999),
        profession: Profession::Hauler,
    });
    assert_eq!(refusal, Some(Refusal::SetProfession { dwarf: Id(999) }));
    assert_eq!(world.professions(), before);

    for (id, _) in before.iter().filter(|(_, p)| *p == Profession::Hauler) {
        let refusal = world.apply_command(SimCommand::SetProfession {
            dwarf: *id,
            profession: Profession::Miner,
        });
        assert_eq!(refusal, None);
    }
    assert!(
        world
            .professions()
            .iter()
            .all(|(_, p)| *p != Profession::Hauler)
    );

    // NOTE: Wolf's ruling (12.6 Task 0.2): a trade nobody has leaves its jobs waiting, silently.
    let stone = dig_one_stone(&mut world);
    let pile = world.dwarves()[0].1;
    assert_ne!(pile, stone);
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(pile, pile),
    });
    world.step();
    assert!(
        world
            .jobs()
            .iter()
            .any(|j| matches!(j.kind, JobKind::Haul { .. }))
    );
    for _ in 0..300 {
        world.step();
        let jobs = world.jobs();
        assert!(
            world.claims().iter().all(|(_, held)| held.is_none_or(|j| {
                !matches!(
                    jobs.iter().find(|job| job.id == j).map(|job| job.kind),
                    Some(JobKind::Haul { .. })
                )
            })),
            "a haul job was claimed with no hauler alive"
        );
    }
    assert!(
        world
            .jobs()
            .iter()
            .any(|j| matches!(j.kind, JobKind::Haul { .. }))
    );
}

#[test]
fn same_seed_and_commands_remain_deterministic() {
    let mut first = World::generate(42, Dims::DEFAULT);
    let mut second = World::generate(42, Dims::DEFAULT);
    let channel_pos = first.dwarves()[0].1;
    let trunk = nearest_trunk_beside_open_ground(&first);
    let commands = [
        SimCommand::Designate {
            kind: DesignationKind::Cut,
            rect: rect(trunk, trunk),
        },
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

    for tick in 0..400 {
        if tick == 60 {
            let command = SimCommand::SetProfession {
                dwarf: first.professions()[0].0,
                profession: Profession::Hauler,
            };
            first.apply_command(command);
            second.apply_command(command);
        }
        // Seed 42's woodcutter is dwarf 0, who was just made a hauler: hand him the axe back, so
        // the cut is felled inside the window and `item_kinds` compares real wood.
        if tick == 120 {
            let command = SimCommand::SetProfession {
                dwarf: first.professions()[0].0,
                profession: Profession::Woodcutter,
            };
            first.apply_command(command);
            second.apply_command(command);
        }
        first.step();
        second.step();
        assert_eq!(first.professions(), second.professions());
        assert_eq!(first.dwarves(), second.dwarves());
        assert_eq!(first.jobs(), second.jobs());
        assert_eq!(first.claims(), second.claims());
        assert_eq!(first.carrying(), second.carrying());
        assert_eq!(first.identities(), second.identities());
        assert_eq!(first.items(), second.items());
        assert_eq!(first.item_kinds(), second.item_kinds());
        assert_eq!(first.emitters(), second.emitters());
        assert_eq!(first.tiles(), second.tiles());
        assert_eq!(first.designations(), second.designations());
        assert_eq!(first.zones(), second.zones());
    }
    // THE GUARD: the cut was felled inside the window, so `item_kinds` compared real wood.
    assert!(
        first
            .item_kinds()
            .iter()
            .any(|(_, kind)| *kind == ItemKind::Wood),
        "the cut never produced wood in 400 ticks"
    );
}

#[test]
fn a_dwarf_that_travelled_to_a_distant_job_still_wanders_afterwards() {
    // Wolf found this by playing 3.2: after digging, dwarves stop moving for good.
    // `wander` only accepts tiles within WANDER_RADIUS of `Wander::home`, which is written
    // once at spawn. A* has no such limit, so a dwarf will walk any distance to a job — and
    // once it ends up 5+ away from home EVERY neighbour is still outside the radius, so the
    // candidate set is empty on every future tick and it never moves again. Distance 4 is the
    // boundary: from there one step inward reaches 3 and it recovers, which is why this only
    // shows up on a genuinely distant job.
    //
    // The invariant asserted here is the weakest honest one: no dwarf is PERMANENTLY
    // motionless. It deliberately does not pin where a dwarf ends up, only that it is still
    // alive to the wander rule.
    let mut world = World::generate(42, Dims::DEFAULT);
    let spawns = world.dwarves();
    let (_, first_home, _, _) = spawns[0];

    // A solid tile far from spawn that ALSO has a standable face — a tile buried in rock has
    // no work position, so nobody would ever walk to it and the test would pass vacuously.
    let standable = |world: &World, pos: Pos| {
        matches!(world.tile(pos), Some(Tile::Empty))
            && matches!(
                world.tile(Pos {
                    z: pos.z - 1,
                    ..pos
                }),
                Some(Tile::Solid(_) | Tile::Ramp(_))
            )
    };
    let target = (8..40)
        .flat_map(|d| {
            [
                (d, 0),
                (0, d),
                (-d, 0),
                (0, -d),
                (d, d),
                (-d, -d),
                (d, -d),
                (-d, d),
            ]
        })
        .map(|(dx, dy)| Pos {
            x: first_home.x + dx,
            y: first_home.y + dy,
            z: first_home.z,
        })
        .find(|pos| {
            matches!(world.tile(*pos), Some(Tile::Solid(_)))
                && [(-1, 0), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .any(|(dx, dy)| {
                        standable(
                            &world,
                            Pos {
                                x: pos.x + dx,
                                y: pos.y + dy,
                                z: pos.z,
                            },
                        )
                    })
        })
        .expect("seed 42 has a workable rock face within 40 tiles of the first dwarf");

    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(target, target),
    });

    // Long enough to cover the reaction delay (5..=30), a cross-map walk and WORK_TICKS.
    for _ in 0..900 {
        world.step();
    }

    assert!(
        !world.items().is_empty(),
        "the distant dig never completed, so nobody travelled and this test proves nothing"
    );

    let before = world.dwarves();
    let mut moved: Vec<bool> = vec![false; before.len()];
    for _ in 0..200 {
        world.step();
        for (index, (id, pos, _, _)) in world.dwarves().into_iter().enumerate() {
            debug_assert_eq!(id, before[index].0, "dwarf order is stable");
            if pos != before[index].1 {
                moved[index] = true;
            }
        }
    }

    for (index, (id, pos, _, _)) in before.into_iter().enumerate() {
        assert!(
            moved[index],
            "dwarf {id:?} has not moved from {pos:?} in 200 ticks — it is stranded outside its \
             wander radius and can never move again"
        );
    }
}

/// A world with one stone and a one-cell stockpile whose cell is standable but walled in on all four
/// sides. Returns the world, the pile cell and its four walls.
fn sealed_pile_world() -> (World, Pos, Vec<Pos>) {
    let mut world = World::generate(42, Dims::DEFAULT);
    let stone = dig_one_stone(&mut world);
    // The pile cell and its four walls, well clear of the dwarves and the stone.
    let pile = Pos {
        y: stone.y + 6,
        ..stone
    };
    let walls: Vec<Pos> = [(-1, 0), (1, 0), (0, -1), (0, 1)]
        .iter()
        .map(|(dx, dy)| Pos {
            x: pile.x + dx,
            y: pile.y + dy,
            ..pile
        })
        .collect();
    for cell in std::iter::once(&pile).chain(&walls) {
        assert!(
            is_standable(&world, *cell),
            "fixture cell {cell:?} was not open ground: {:?}",
            world.tile(*cell)
        );
    }
    for wall in &walls {
        assert!(world.set_tile(*wall, Tile::Solid(Material::Stone)));
    }
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(pile, pile),
    });
    assert_eq!(world.zones().len(), 1);
    (world, pile, walls)
}

/// Issue #132 / 12.1 handover: the only free pile cell is standable but sealed off from every
/// dwarf. The pick-up leg's goal set is non-empty (`free` is), so a dwarf used to claim, carry the
/// stone, fail to path to the pile, and have `release_claim` drop it — every 20 ticks, forever.
#[test]
fn a_sealed_off_pile_cell_does_not_cycle_a_stone_forever() {
    let (mut world, _pile, _walls) = sealed_pile_world();

    let mut pickups = Vec::new();
    let mut drops = Vec::new();
    let mut stamps = BTreeSet::new();
    let mut held = false;
    for _ in 0..400 {
        world.step();
        let carriers: Vec<_> = world
            .carrying()
            .into_iter()
            .filter(|(_, item)| item.is_some())
            .collect();
        let now_held = !carriers.is_empty();
        if now_held && !held {
            pickups.push((world.tick(), carriers[0].0));
        }
        if !now_held && held {
            drops.push((world.tick(), world.items()[0].1));
        }
        held = now_held;
        let job = world.jobs()[0];
        assert_eq!(
            job.kind,
            JobKind::Haul { item: 10 },
            "FR8: the haul job was dropped"
        );
        stamps.insert(job.retry_after);
    }
    println!("pickups {pickups:?}\ndrops {drops:?}\nretry_after stamps {stamps:?}");
    assert!(
        pickups.is_empty(),
        "the stone was picked up {} times: {pickups:?}, dropped {drops:?}",
        pickups.len()
    );
    assert!(
        stamps.len() > 1,
        "FR8: the job was never retried: {stamps:?}"
    );
}

/// #132 in its haul form: every idle dwarf's delivery search for the sealed pile floods the whole
/// valley, so without reusing that component across dwarves the shared budget runs out on the
/// haul job every tick and the dig queued behind it is never attempted.
#[test]
fn a_sealed_off_pile_cell_does_not_starve_a_reachable_dig() {
    let (mut world, _pile, _walls) = sealed_pile_world();
    for _ in 0..100 {
        world.step();
    }
    assert!(world.claims().iter().all(|(_, job)| job.is_none()));
    let worker = world.dwarves()[0].1;
    let target = Pos {
        x: worker.x + 1,
        ..worker
    };
    assert!(world.set_tile(
        Pos {
            z: target.z - 1,
            ..target
        },
        Tile::Solid(Material::Stone),
    ));
    assert!(world.set_tile(target, Tile::Solid(Material::Stone)));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(target, target),
    });
    let designated_at = world.tick();
    for _ in 0..40 {
        world.step();
        let dig = world
            .jobs()
            .into_iter()
            .find(|job| job.target == target)
            .expect("the dig is queued")
            .id;
        if world.claims().iter().any(|(_, job)| *job == Some(dig)) {
            return;
        }
    }
    panic!(
        "the reachable dig was not claimed within 40 ticks of designation (tick {designated_at})"
    );
}

/// Positive control for the test above: the same fixture with one wall opened onto open ground
/// hauls the stone into the pile, so the sealed test cannot pass because hauling is simply broken.
#[test]
fn an_opened_pile_cell_receives_the_stone() {
    let (mut world, pile, walls) = sealed_pile_world();
    assert!(world.set_tile(walls[0], Tile::Empty));
    for _ in 0..600 {
        world.step();
        if world.jobs().is_empty() {
            break;
        }
    }
    assert!(world.jobs().is_empty(), "never hauled once opened");
    assert_eq!(world.items(), vec![(sim_core::Id(10), pile)]);
}

// ---- Story 12.7: timber (DEFAULT_SEED's measured trees) ----

/// Tree A and tree B stand at x 73. Their crowns touch (story 12.7, "Found at creation").
const TREE_A: (i32, i32) = (73, 59);
const TREE_B: (i32, i32) = (73, 56);

/// An independent oracle for the tree rule, written from the story's wording and not from the
/// sim's `tree_of`: the trunk column, plus the foliage in the 3x3 column around it, from the base
/// to one above the top trunk cell. Returns the base and every tile.
fn tree_tiles(world: &World, column: (i32, i32)) -> (Pos, BTreeSet<Pos>) {
    let (x, y) = column;
    let trunk: Vec<i32> = (0..world.dims().z as i32)
        .filter(|z| world.tile(Pos { x, y, z: *z }) == Some(Tile::Solid(Material::TreeTrunk)))
        .collect();
    assert!(
        !trunk.is_empty(),
        "no trunk at {column:?}: pinned to DEFAULT_SEED"
    );
    let (base, top) = (trunk[0], *trunk.last().unwrap());
    let mut tiles: BTreeSet<Pos> = trunk.iter().map(|z| Pos { x, y, z: *z }).collect();
    for z in base..=top + 1 {
        for fy in y - 1..=y + 1 {
            for fx in x - 1..=x + 1 {
                let pos = Pos { x: fx, y: fy, z };
                if world.tile(pos) == Some(Tile::Solid(Material::TreeFoliage)) {
                    tiles.insert(pos);
                }
            }
        }
    }
    (Pos { x, y, z: base }, tiles)
}

fn kinds_at(world: &World, pos: Pos, kind: ItemKind) -> usize {
    let kinds: BTreeMap<Id, ItemKind> = world.item_kinds().into_iter().collect();
    world
        .items()
        .iter()
        .filter(|(id, at)| *at == pos && kinds[id] == kind)
        .count()
}

fn is_tree_material(tile: Option<Tile>) -> bool {
    matches!(
        tile,
        Some(Tile::Solid(Material::TreeTrunk | Material::TreeFoliage))
    )
}

fn cut(world: &mut World, pos: Pos) -> Option<Refusal> {
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Cut,
        rect: rect(pos, pos),
    })
}

fn woodcutter(world: &World) -> Id {
    world
        .professions()
        .into_iter()
        .find_map(|(id, profession)| (profession == Profession::Woodcutter).then_some(id))
        .expect("DEFAULT_SEED has a woodcutter")
}

/// Which dwarf holds a job of this kind right now, if any.
fn holder_of(world: &World, kind: JobKind) -> Option<Id> {
    let jobs = world.jobs();
    world.claims().into_iter().find_map(|(id, held)| {
        let job = jobs.iter().find(|job| Some(job.id) == held)?;
        (job.kind == kind).then_some(id)
    })
}

#[test]
fn cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let (a_base, a_tiles) = tree_tiles(&world, TREE_A);
    let (_, b_tiles) = tree_tiles(&world, TREE_B);
    assert!(a_tiles.is_disjoint(&b_tiles));
    let b_before: Vec<_> = b_tiles.iter().map(|p| (*p, world.tile(*p))).collect();
    let nain = woodcutter(&world);
    let trunk_cells = a_tiles
        .iter()
        .filter(|p| world.tile(**p) == Some(Tile::Solid(Material::TreeTrunk)))
        .count();
    assert_eq!(trunk_cells, 4, "tree A has four trunk cells (z 12-15)");

    assert_eq!(cut(&mut world, a_base), None);
    assert_eq!(
        world.designations(),
        vec![(a_base, DesignationKind::Cut)],
        "one mark, at the base"
    );
    let mut ever_held = false;
    for _ in 0..3_000 {
        world.step();
        if let Some(holder) = holder_of(&world, JobKind::Cut) {
            assert_eq!(holder, nain, "only the woodcutter holds a cut");
            ever_held = true;
        }
        if a_tiles.iter().all(|p| world.tile(*p) == Some(Tile::Empty)) {
            break;
        }
    }
    assert!(ever_held, "the woodcutter never held the cut");
    for tile in &a_tiles {
        assert_eq!(
            world.tile(*tile),
            Some(Tile::Empty),
            "{tile:?} still stands"
        );
    }
    for (pos, before) in b_before {
        assert_eq!(world.tile(pos), before, "tree B changed at {pos:?}");
    }
    assert_eq!(
        kinds_at(&world, a_base, ItemKind::Wood),
        4,
        "one log per trunk cell, at the base"
    );
    assert!(
        world
            .item_kinds()
            .iter()
            .all(|(_, kind)| *kind == ItemKind::Wood),
        "a cut leaves no stone"
    );
    assert!(world.designations().is_empty());
    assert!(world.jobs().iter().all(|job| job.kind != JobKind::Cut));
    assert!(
        world
            .claims()
            .iter()
            .all(|(id, held)| *id != nain || held.is_none())
    );
}

#[test]
fn a_cut_is_hauled_to_the_pile_as_wood() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let camp = world.camp_origin();
    let (a_base, _) = tree_tiles(&world, TREE_A);
    let pile = (3..12)
        .flat_map(|r| {
            [
                (camp.x - r - 2, camp.y),
                (camp.x, camp.y + r),
                (camp.x, camp.y - r - 2),
                (camp.x + r, camp.y),
            ]
        })
        .find(|(px, py)| {
            (*px..*px + 3)
                .all(|x| (*py..*py + 3).all(|y| is_standable(&world, Pos { x, y, z: camp.z })))
        })
        .expect("a 3x3 standable pile near the camp");
    let corner = Pos {
        x: pile.0,
        y: pile.1,
        z: camp.z,
    };
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(
            corner,
            Pos {
                x: corner.x + 2,
                y: corner.y + 2,
                ..corner
            },
        ),
    });
    let zones: BTreeSet<Pos> = world.zones().into_iter().collect();
    assert_eq!(zones.len(), 9);
    assert_eq!(cut(&mut world, a_base), None);

    let mut on_pile = 0;
    for _ in 0..8_000 {
        world.step();
        on_pile = world
            .items()
            .iter()
            .filter(|(_, pos)| zones.contains(pos))
            .count();
        if on_pile == 4 {
            break;
        }
    }
    assert_eq!(on_pile, 4, "all four logs reach the pile");
    let kinds: BTreeMap<Id, ItemKind> = world.item_kinds().into_iter().collect();
    for (id, pos) in world.items() {
        assert_eq!(kinds[&id], ItemKind::Wood);
        assert!(
            zones.contains(&pos),
            "log {id:?} is at {pos:?}, off the pile"
        );
    }
}

#[test]
fn a_cut_over_no_tree_is_refused_and_a_dig_never_marks_a_tree() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let camp = world.camp_origin();
    let (d_base, d_tiles) = tree_tiles(&world, (65, 56));
    let air = rect(
        Pos {
            x: camp.x - 2,
            y: camp.y - 2,
            ..camp
        },
        Pos {
            x: camp.x - 1,
            y: camp.y - 1,
            ..camp
        },
    );
    let refusal = world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Cut,
        rect: air,
    });
    assert_eq!(
        refusal,
        Some(Refusal::Designate {
            kind: DesignationKind::Cut,
            rect: air
        })
    );
    assert!(world.designations().is_empty());

    // A dig over a trunk, and over a foliage tile, marks nothing and is refused.
    let foliage = *d_tiles
        .iter()
        .find(|p| world.tile(**p) == Some(Tile::Solid(Material::TreeFoliage)))
        .unwrap();
    for target in [d_base, foliage] {
        let refused = world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: rect(target, target),
        });
        assert_eq!(
            refused,
            Some(Refusal::Designate {
                kind: DesignationKind::Dig,
                rect: rect(target, target)
            })
        );
    }
    assert!(world.designations().is_empty(), "no dig mark on a tree");

    // A dig over the whole tree and the stone under it marks the stone only.
    let floor = rect(
        Pos {
            x: d_base.x - 1,
            y: d_base.y - 1,
            z: d_base.z - 1,
        },
        Pos {
            x: d_base.x + 1,
            y: d_base.y + 1,
            z: d_tiles.iter().map(|p| p.z).max().unwrap(),
        },
    );
    assert_eq!(
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: floor
        }),
        None
    );
    let marked = world.designations();
    assert!(!marked.is_empty(), "the ground under the tree takes a dig");
    for (pos, kind) in &marked {
        assert_eq!(*kind, DesignationKind::Dig);
        assert!(
            !is_tree_material(world.tile(*pos)),
            "dig mark on tree {pos:?}"
        );
    }
    // Re-marking an existing mark counts as applied.
    let (again, _) = marked[0];
    assert_eq!(
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: rect(again, again)
        }),
        None
    );
    world.apply_command(SimCommand::CancelDesignation { rect: floor });
    assert!(world.designations().is_empty());

    // A channel over a cell standing on a tree (the air over a crown tip) marks nothing.
    let dims = world.dims();
    let on_tree = (0..dims.z as i32)
        .flat_map(|z| {
            (0..dims.y as i32).flat_map(move |y| (0..dims.x as i32).map(move |x| Pos { x, y, z }))
        })
        .find(|pos| {
            is_standable(&world, *pos)
                && is_tree_material(world.tile(Pos {
                    z: pos.z - 1,
                    ..*pos
                }))
        })
        .expect("a cell standing on a treetop");
    assert_eq!(
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Channel,
            rect: rect(on_tree, on_tree)
        }),
        Some(Refusal::Designate {
            kind: DesignationKind::Channel,
            rect: rect(on_tree, on_tree)
        })
    );
    assert!(world.designations().is_empty(), "no channel mark on a tree");
    // A channel on ordinary ground still marks.
    let ground_cell = Pos { z: camp.z, ..camp };
    assert!(is_standable(&world, ground_cell));
    assert_eq!(
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Channel,
            rect: rect(ground_cell, ground_cell)
        }),
        None
    );
    assert_eq!(
        world.designations(),
        vec![(ground_cell, DesignationKind::Channel)]
    );
}

#[test]
fn a_cancel_touching_a_marked_tree_removes_its_mark_and_releases_the_woodcutter() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let (a_base, a_tiles) = tree_tiles(&world, TREE_A);
    let (b_base, b_tiles) = tree_tiles(&world, TREE_B);
    let nain = woodcutter(&world);
    let crown = *a_tiles
        .iter()
        .rev()
        .find(|p| world.tile(**p) == Some(Tile::Solid(Material::TreeFoliage)))
        .unwrap();
    assert_ne!(crown, a_base);
    cut(&mut world, a_base);
    cut(&mut world, b_base);
    assert_eq!(world.designations().len(), 2);

    // A rect over a tile of B only leaves A's mark alone, even where the crowns touch.
    let b_only = *b_tiles
        .iter()
        .find(|p| world.tile(**p) == Some(Tile::Solid(Material::TreeFoliage)))
        .unwrap();
    world.apply_command(SimCommand::CancelDesignation {
        rect: rect(b_only, b_only),
    });
    assert_eq!(world.designations(), vec![(a_base, DesignationKind::Cut)]);

    for _ in 0..600 {
        if holder_of(&world, JobKind::Cut) == Some(nain) {
            break;
        }
        world.step();
    }
    assert_eq!(holder_of(&world, JobKind::Cut), Some(nain));

    // A rect holding a CROWN tile, not the base, removes the mark, its job and the claim.
    world.apply_command(SimCommand::CancelDesignation {
        rect: rect(crown, crown),
    });
    assert!(world.designations().is_empty());
    assert!(world.jobs().iter().all(|job| job.kind != JobKind::Cut));
    assert_eq!(holder_of(&world, JobKind::Cut), None);
    assert!(world.claims().iter().all(|(_, held)| held.is_none()));
    for _ in 0..200 {
        world.step();
    }
    assert!(
        a_tiles.iter().all(|p| is_tree_material(world.tile(*p))),
        "the cancelled tree stands"
    );

    // A rect over the base alone also removes a mark.
    cut(&mut world, a_base);
    world.apply_command(SimCommand::CancelDesignation {
        rect: rect(a_base, a_base),
    });
    assert!(world.designations().is_empty());
}

#[test]
fn a_cut_against_a_full_cap_is_refused_and_adds_no_mark() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
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
    let (a_base, _) = tree_tiles(&world, TREE_A);
    assert_eq!(
        cut(&mut world, a_base),
        Some(Refusal::Designate {
            kind: DesignationKind::Cut,
            rect: rect(a_base, a_base)
        })
    );
    assert_eq!(world.designations().len(), 4096);
    assert!(
        !world
            .designations()
            .iter()
            .any(|(_, kind)| *kind == DesignationKind::Cut)
    );
}

fn uncarried_item_tiles(world: &World) -> BTreeSet<Pos> {
    let carried: BTreeSet<u32> = world
        .carrying()
        .into_iter()
        .filter_map(|(_, item)| item)
        .collect();
    world
        .items()
        .into_iter()
        .filter(|(id, _)| !carried.contains(&id.0))
        .map(|(_, pos)| pos)
        .collect()
}

// 12.9 AC1 (#133): the busy crew of the story's Found-at-creation probe -- a 4x7 channel east of
// the fire and a 3x3 pile west of it -- run for 3,000 ticks with no two dwarves ever on one tile.
// Red on 26185a0 (332 shared ticks). The vacuity asserts come LAST so a mutant that freezes the
// crew dies on "the crew still works" and not on the occupancy assert it trivially passes.
#[test]
fn a_busy_crew_never_shares_a_tile_and_still_works() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let channel = rect(Pos { x: 65, y: 61, z: 9 }, Pos { x: 68, y: 67, z: 9 });
    let pile = rect(Pos { x: 59, y: 64, z: 9 }, Pos { x: 61, y: 66, z: 9 });
    let pile_cells: BTreeSet<Pos> = (pile.min.y..=pile.max.y)
        .flat_map(|y| (pile.min.x..=pile.max.x).map(move |x| Pos { x, y, z: 9 }))
        .collect();
    assert!(
        pile_cells.iter().all(|cell| is_standable(&world, *cell)),
        "the pile site must be standable ground"
    );
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: channel,
    });
    assert!(
        world
            .apply_command(SimCommand::PlaceStockpile { rect: pile })
            .is_none()
    );
    assert_eq!(
        world.designations().len(),
        25,
        "the channel is 25 workable marks"
    );

    let mut previous: Vec<Pos> = world.dwarves().iter().map(|(_, pos, ..)| *pos).collect();
    let mut shared_ticks = Vec::new();
    let mut moves = 0_usize;
    let mut marks_cleared_at = None;
    let mut first_stone_on_pile = None;
    // AC10: where each uncarried item lay at the end of the previous tick.
    let mut previous_items = uncarried_item_tiles(&world);
    let mut item_entries = Vec::new();
    for _ in 0..3_000 {
        world.step();
        let tick = world.tick();
        let now: Vec<Pos> = world.dwarves().iter().map(|(_, pos, ..)| *pos).collect();
        for (dwarf, (after, before)) in now.iter().zip(&previous).enumerate() {
            if after != before && previous_items.contains(after) {
                item_entries.push((tick, dwarf, *before, *after));
            }
        }
        previous_items = uncarried_item_tiles(&world);
        if now.iter().collect::<BTreeSet<_>>().len() != now.len() {
            shared_ticks.push((tick, now.clone()));
        }
        moves += now.iter().zip(&previous).filter(|(a, b)| a != b).count();
        if marks_cleared_at.is_none() && world.designations().is_empty() {
            marks_cleared_at = Some(tick);
        }
        if first_stone_on_pile.is_none()
            && world
                .items()
                .iter()
                .any(|(_, pos)| pile_cells.contains(pos))
        {
            first_stone_on_pile = Some(tick);
        }
        previous = now;
    }

    assert!(
        shared_ticks.is_empty(),
        "{} ticks had two dwarves on one tile; first {:?}",
        shared_ticks.len(),
        shared_ticks.first()
    );
    // AC10 (#162): no dwarf move lands on a tile that held an uncarried item at the end of the
    // previous tick.
    assert!(
        item_entries.is_empty(),
        "{} dwarf moves entered an item's cell; first (tick, dwarf, from, to) {:?}",
        item_entries.len(),
        item_entries.first()
    );

    // Vacuity: the crew still works.
    assert!(
        marks_cleared_at.is_some_and(|tick| tick <= 2_500),
        "channel marks cleared at {marks_cleared_at:?}, bound 2500"
    );
    assert!(
        first_stone_on_pile.is_some_and(|tick| tick <= 600),
        "first stone on the pile at {first_stone_on_pile:?}, bound 600"
    );
    assert!(moves >= 600, "dwarf moves {moves}, bound 600");
}

// 12.9 AC2: the spawn guard, green on 26185a0 (`spawn_dwarves` draws with `swap_remove`).
#[test]
fn spawn_places_five_dwarves_on_distinct_tiles_for_every_small_seed() {
    for seed in 0..64_u64 {
        let world = World::generate(seed, Dims::DEFAULT);
        let tiles: BTreeSet<Pos> = world.dwarves().iter().map(|(_, pos, ..)| *pos).collect();
        assert_eq!(world.dwarves().len(), 5, "seed {seed}");
        assert_eq!(
            tiles.len(),
            5,
            "seed {seed} spawned two dwarves on one tile"
        );
    }
}
