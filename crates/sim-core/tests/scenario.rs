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
        // the hauls take longer. The pin below says what happens after t=2,500.
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
    // 12.9 review run 2 re-pin (Wolf: land "never under any dwarf", file the jam, pin it): a
    // let-go stone now lands beside its hauler, and this seed jams (#189). Idle dwarf 2 stands on
    // the last free pile cell, boxed in, and the hauler beside it picks up and lets go every ~6
    // ticks forever. Was: 0 pick-ups after t=2500, 24 of 24 cells. Fixing #189 turns this red; then
    // restore those two figures.
    assert!(
        emitter_zones.is_empty() && pickups_after_full == 80,
        "zone on an emitter: {emitter_zones:?}; pick-ups after t=2500: {pickups_after_full} (pinned \
         80, the #189 jam)"
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
        zones.len() - 1,
        "the pile fills but for the #189 jam's cell, or every other assertion here holds with \
         nothing hauled"
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
    // 12.10b re-pin (was z 1): a channel is refused below z 2, so the rect sits at z 2.
    make_standable(&mut world, Pos { x: 0, y: 0, z: 2 });
    make_standable(&mut world, Pos { x: 1, y: 0, z: 2 });
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(Pos { x: -1, y: -1, z: 2 }, Pos { x: 1, y: 0, z: 2 }),
    });

    assert_eq!(
        world.designations(),
        vec![
            (Pos { x: 0, y: 0, z: 2 }, DesignationKind::Channel),
            (Pos { x: 1, y: 0, z: 2 }, DesignationKind::Channel),
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
    // 12.10b re-pin: a worked channel leaves the cell under the target open (was: a ramp).
    assert!(
        world
            .designations()
            .contains(&(t, DesignationKind::Channel))
            || world.tile(below) == Some(Tile::Empty),
        "the channel order vanished without being worked"
    );
}

/// AC13 (12.9 seat pass 1): a channel is worked from a walkable neighbour of the target, never from
/// the target itself.
// 12.10b re-pin (was `..._and_the_stone_lands_on_the_target`, and asserted a same-z neighbour): the
// stone now lands up top beside the worker, never on the target or in its column, and the cell
// under the target is opened.
#[test]
fn a_channel_is_worked_from_the_next_tile_and_the_stone_lands_beside_the_worker() {
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
    assert_eq!(from.x.abs_diff(t.x) + from.y.abs_diff(t.y), 1);
    assert_eq!(world.tile(Pos { z: t.z - 1, ..t }), Some(Tile::Empty));
    let items = world.items();
    assert_eq!(items.len(), 1, "one block was yielded: {items:?}");
    let landed = items[0].1;
    assert_eq!(landed.z, from.z, "the stone lands on the worker's level");
    assert!(
        (landed.x, landed.y) != (t.x, t.y),
        "the stone landed on the target or in its column: {landed:?}"
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
        // The dug tile keeps its floor, so the stone stays at the dig site for a local haul.
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
            clearing: None,
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
            clearing: None,
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
    let camp = first.camp_origin();
    let commands = [
        SimCommand::Designate {
            kind: DesignationKind::Cut,
            rect: rect(trunk, trunk),
        },
        SimCommand::Designate {
            kind: DesignationKind::Channel,
            rect: rect(channel_pos, channel_pos),
        },
        // 12.12: this was a 21 x 21 x 21 pile from the origin, and with it no log was stored in the
        // window (not in 1,500 ticks either). A pile at the camp stores the felled
        // wood by tick ~290, so `goals` compares a real count.
        SimCommand::PlaceStockpile {
            rect: rect(
                Pos {
                    x: camp.x + 3,
                    y: camp.y + 3,
                    ..camp
                },
                Pos {
                    x: camp.x + 5,
                    y: camp.y + 5,
                    ..camp
                },
            ),
        },
    ];
    for command in commands {
        assert_eq!(first.apply_command(command), None);
        second.apply_command(command);
    }

    let mut most_wood_stored = 0;
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
        assert_eq!(first.goals(), second.goals());
        most_wood_stored = most_wood_stored.max(first.goals()[0].progress);
    }
    // THE GUARD (12.12): `goals` compared a real count, not zeros.
    assert!(most_wood_stored > 0, "no wood was ever stored in 400 ticks");
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
    // 12.10b re-pin (was: the camp cell itself): the camp cell is the campfire's tile, where a
    // channel is now refused (D3), so the ground cell is the one beside it.
    let ground_cell = Pos {
        x: camp.x + 1,
        z: camp.z,
        ..camp
    };
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

// 12.9 review #182: the review's multi-seed probe. Seeds 0..=16, a 4x7 and a 6x10 channel beside
// the fire and a 3x3 pile near it, 4,000 ticks each. On 34b6783 a channel stone spawned under a
// dwarf, or a delivery walled one into a dead end, and the dwarf never moved again: 8 of 28 runs
// ended with a dwarf still for >= 2,251 ticks (longest still on 26185a0: <= 60). No stone lands on
// a dwarf, every dwarf still for long is one of the known one-level islands (#186), and the runs
// that leave marks are the pinned ones.
#[test]
fn no_dwarf_is_caged_by_stones_over_the_probe_seeds() {
    const TICKS: u64 = 4_000;
    const STILL_BOUND: u64 = 500;
    // (seed, channel width, dwarf). An idle dwarf whose every same-level neighbour is a stone,
    // rock, a drop or another dwarf: `wander` is same-z only, so it stands until a job routes it
    // out over a ramp. No stone is on it and A* still reaches it, so it is not walled in (#186).
    // 12.9 review run 2 re-pin: every dwarf over the bound is listed now, not the longest per run.
    // "Never under any dwarf" moved the runs: (6, 6, 0), (12, 6, 4) and (13, 6, 2) are gone and
    // (16, 4, 0) is new, an idle dwarf between foliage, two loose stones and a drop.
    // 12.10b re-pin: a channel opens a step down and its block lands beside the worker, so the
    // runs moved again. Was [(10, 4, 2), (10, 6, 3), (16, 4, 0)]; now (4, 4, 4) and (10, 6, 4) before D3,
    // read by hand at tick 4,000: each is an idle dwarf whose same-level neighbours are a loose
    // block, rock or a drop (the #186 shape), with no mark and no claim left in the run. Task 4
    // (the dig-face jam) owns the final record. (10, 6, 4) left once a channel was refused on the
    // campfire's tile (D3): both rects cover it, so seed 10's crew worked a different order.
    // 12.10b re-pin (AC 4 ruling, a channel's block never lands below the worker): was
    // [(4, 4, 4)], now (10, 4, 4), read by dump at tick 4,000: idle at [67, 76, 28] between two
    // loose blocks and two drops, no mark and no claim left (the #186 shape again).
    // 12.11 re-pin (was [(10, 4, 4)]), each run read by a per-item trace at tick 4,000. All are the
    // #186 shape: an idle dwarf on a cell whose other same-level sides are rock, a drop or a loose
    // block, no mark and no claim left. Moved items are disclosed where they border the cell.
    // (8,4,4) [69,67,7]: a block he set down himself at t1204 (clearing), a channel block at t1428
    // after he walked in at t1377. (12,4,4) [69,65,10]: his own clearing set-down at t1029 took one
    // of two exits, a later cut took the other. (12,4,1) [66,60,10]: a two-cell pocket
    // {[66,60,10], [66,61,10]} bordered by three blocks, two set aside by another miner (t565,
    // t614), one a hauler's drop (t180); he walked in up a ramp at t630, and a CHANNEL CUT of
    // [66,61,10] at t720 closed his last same-level exit (the opened step keeps him unstranded, so
    // the cut is allowed). Closed by a cut, not a placement: pinned by Wolf's ruling (2026-10-16),
    // because every placement rule that freed it left another run permanently unfinished.
    // (16,6,3) [99,62,16]: his own set-down (t3420) and a channel block (t3471) after he stopped at
    // t3408. (10,6,0) [62,78,28] stood 521 ticks, no item beside it, and moved again at t3996: a
    // pause, not a cage.
    // 12.11 seat re-pin (was the five above; "miners are throwing stones", Wolf): a miner lifts only
    // with a set-down, lets go only beside himself, and routes to it through the crew. (8,4,4) and
    // (16,6,3) are gone. (12,6,3) is new, read by dump at tick 4,000: idle at [64,69,10] since t1119,
    // no item beside him, no mark and no claim left; his sides are snow, snow and a drop, and the
    // one free side [65,69,10] lies 4 cells from his home [61,66,10], past WANDER_RADIUS (3). The
    // #186 shape with the radius as its fourth wall.
    // 12.11 review run 1 (Wolf, 2026-10-17): (12,4,4) stays pinned under the (12,4,1) ruling. His
    // own clearing set-down took one exit, but a channel cut closed the last one, so it is closed
    // by a cut, not a placement; noted on #186.
    const ONE_LEVEL_ISLANDS: [(u64, i32, usize); 4] =
        [(10, 6, 0), (12, 4, 1), (12, 4, 4), (12, 6, 3)];
    // (seed, channel width, marks left) after `TICKS` (12.9 review run 2: the sweep pins marks, so
    // a new refusal loop goes red). Every one of these ends with the pile full. Two shapes are
    // known: a mark whose miner is refused at completion over and over (#187), and a mark walled in
    // by unhauled stones with no work position left (#182 mode ii, the ruled #180 / FR8
    // never-drop shape). Base `26185a0` left no 4-wide run unfinished.
    // 12.10b re-pin: was fourteen unfinished runs (the #187 refusal loop and #182 mode ii). A
    // channel no longer spawns a block on its target, so neither shape arises: none is left. Two
    // intermediate runs ((0, 4, 1) and (4, 4, 1)) were a worker in the pit of a level above refused
    // forever by the wall-in check read BEFORE the step was cut; the check now reads after.
    // 12.11 re-pin (was none): three 6-wide runs are SLOW, not stuck. A clearing miner walks a stone
    // aside before he cuts, and the crew finishes at t4,080 (6), t4,350 (11) and t4,444 (12), read
    // by running the same runs to 16,000 ticks.
    // 12.11 seat re-pin (was the three above): still slow, not stuck. Run to 16,000 ticks, seed 6
    // finishes at t4,080 and seed 11 at t4,100; seed 12 now finishes inside the 4,000.
    // 12.11 dig-loop re-pin (was (6, 6, 2) too): a miner caged by one stone carries it out
    // (`caging_stone`), and seed 6 now finishes inside the 4,000.
    const UNFINISHED: [(u64, i32, usize); 1] = [(11, 6, 1)];
    let mut runs = 0;
    let mut frozen = Vec::new();
    let mut unfinished = Vec::new();
    let mut landed = Vec::new();
    let mut worked = 0;
    for seed in 0..=16_u64 {
        for (west, south, east, north) in [(1, -3, 4, 3), (-1, -5, 4, 4)] {
            let mut world = World::generate(seed, Dims::DEFAULT);
            let camp = world.camp_origin();
            let at = |dx: i32, dy: i32| Pos {
                x: camp.x + dx,
                y: camp.y + dy,
                z: camp.z,
            };
            let channel = rect(at(west, south), at(east, north));
            let width = channel.max.x - channel.min.x + 1;
            // The pile search of `occupancy_wire.py`. It may overlap the channel; this search
            // reproduces the review's seed 5 cage and seed 13 dead end tile for tile.
            let pile = (3..12).find_map(|r| {
                [(-r - 2, 0), (0, r), (0, -r - 2)]
                    .into_iter()
                    .map(|(dx, dy)| rect(at(dx, dy), at(dx + 2, dy + 2)))
                    .find(|pile| {
                        (pile.min.x..=pile.max.x).all(|x| {
                            (pile.min.y..=pile.max.y).all(|y| {
                                let cell = Pos { x, y, z: camp.z };
                                is_standable(&world, cell)
                            })
                        })
                    })
            });
            let Some(pile) = pile else { continue };
            runs += 1;
            world.apply_command(SimCommand::Designate {
                kind: DesignationKind::Channel,
                rect: channel,
            });
            assert!(
                world
                    .apply_command(SimCommand::PlaceStockpile { rect: pile })
                    .is_none()
            );
            let marks = world.designations().len();

            // Loose items by id.
            let loose = |world: &World| -> BTreeMap<u32, Pos> {
                let carried: BTreeSet<u32> = world
                    .carrying()
                    .iter()
                    .filter_map(|(_, item)| *item)
                    .collect();
                world
                    .items()
                    .into_iter()
                    .filter(|(id, _)| !carried.contains(&id.0))
                    .map(|(id, pos)| (id.0, pos))
                    .collect()
            };
            let mut last_move = [0_u64; 5];
            let mut longest = [0_u64; 5];
            let mut previous: Vec<Pos> = world.dwarves().iter().map(|(_, p, ..)| *p).collect();
            let mut loose_before = loose(&world);
            for _ in 0..TICKS {
                world.step();
                let now: Vec<Pos> = world.dwarves().iter().map(|(_, p, ..)| *p).collect();
                for dwarf in 0..5 {
                    if now[dwarf] != previous[dwarf] {
                        last_move[dwarf] = world.tick();
                    }
                    longest[dwarf] = longest[dwarf].max(world.tick() - last_move[dwarf]);
                }
                // A stone placed this tick (spawned, delivered or dropped) onto a dwarf, the
                // dropper's own tile included (12.9 review run 2, Wolf: never under any dwarf).
                let loose_now = loose(&world);
                for (item, pos) in &loose_now {
                    if loose_before.get(item) == Some(pos) {
                        continue;
                    }
                    if let Some(dwarf) = now.iter().position(|p| p == pos) {
                        landed.push((seed, width, world.tick(), *item, dwarf));
                    }
                }
                loose_before = loose_now;
                previous = now;
            }
            for (dwarf, still) in longest.iter().copied().enumerate() {
                if still >= STILL_BOUND {
                    frozen.push((seed, width, dwarf, still, previous[dwarf]));
                }
            }
            let marks_left = world.designations().len();
            if marks_left > 0 {
                unfinished.push((seed, width, marks_left));
            }
            let on_pile = world.items().iter().any(|(_, p)| {
                (pile.min.x..=pile.max.x).contains(&p.x) && (pile.min.y..=pile.max.y).contains(&p.y)
            });
            if world.designations().len() < marks && on_pile {
                worked += 1;
            }
        }
    }
    assert!(
        landed.is_empty(),
        "stones landed on a dwarf; (seed, channel width, tick, item, dwarf): {landed:?}"
    );
    assert_eq!(
        frozen
            .iter()
            .map(|(seed, width, dwarf, ..)| (*seed, *width, *dwarf))
            .collect::<Vec<_>>(),
        ONE_LEVEL_ISLANDS,
        "runs with a dwarf still for >= {STILL_BOUND} ticks; (seed, channel width, dwarf, longest \
         still, final tile): {frozen:?}"
    );
    assert_eq!(
        unfinished, UNFINISHED,
        "runs with marks left after {TICKS} ticks; (seed, channel width, marks left)"
    );
    // Vacuity: every run with a pile site really ran (the review's own pile search found 28, this
    // one 32), and in every one the crew worked.
    assert_eq!(runs, 32, "runs with a pile");
    assert_eq!(
        worked, runs,
        "runs where marks cleared and a stone reached the pile"
    );
}

/// 12.10: a standable cell beside dwarf 2 to dig or channel, a one-cell stockpile at dwarf 0 (12.9
/// items block their cell, so a dig with no pile would stop), and the pile cell.
fn ore_fixture(world: &mut World) -> (Pos, Pos) {
    let worker = world.dwarves()[2].1;
    let target = Pos {
        x: if worker.x + 1 < world.dims().x as i32 {
            worker.x + 1
        } else {
            worker.x - 1
        },
        ..worker
    };
    let pile = world.dwarves()[0].1;
    assert_ne!(pile, target);
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(pile, pile),
    });
    (target, pile)
}

/// Steps until the first item exists and returns where it lies; panics after `limit` ticks.
fn first_item(world: &mut World, limit: u64, what: &str) -> Pos {
    while world.items().is_empty() {
        assert!(world.tick() < limit, "{what}: no item within {limit} ticks");
        world.step();
    }
    world.items()[0].1
}

#[test]
fn digging_ore_yields_an_ore_item_that_reaches_the_pile() {
    for (material, kind) in [
        (Material::Tetrahedrite, ItemKind::Tetrahedrite),
        (Material::Bloodstone, ItemKind::Bloodstone),
    ] {
        let mut world = World::generate(42, Dims::DEFAULT);
        let (target, pile) = ore_fixture(&mut world);
        assert!(world.set_tile(
            Pos {
                z: target.z - 1,
                ..target
            },
            Tile::Solid(Material::Stone),
        ));
        assert!(world.set_tile(target, Tile::Solid(material)));
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: rect(target, target),
        });

        let at = first_item(&mut world, 600, "dig");
        assert_eq!(at, target, "the dug item lies on the dug cell");
        assert_eq!(world.tile(target), Some(Tile::Empty));
        assert_eq!(
            kinds_at(&world, target, kind),
            1,
            "digging {material:?} must yield one {kind:?} item, got {:?}",
            world.item_kinds()
        );
        assert_eq!(kinds_at(&world, target, ItemKind::Stone), 0);

        for _ in 0..1_500 {
            if kinds_at(&world, pile, kind) == 1 {
                break;
            }
            world.step();
        }
        assert_eq!(
            kinds_at(&world, pile, kind),
            1,
            "the {kind:?} item never reached the pile"
        );
    }
}

/// 12.10 AC4's other half: only ore yields ore. Every other non-tree material still yields one stone.
#[test]
fn digging_soil_snow_and_ice_still_yields_stone() {
    for material in [Material::Soil, Material::Snow, Material::Ice] {
        let mut world = World::generate(42, Dims::DEFAULT);
        let (target, _pile) = ore_fixture(&mut world);
        assert!(world.set_tile(
            Pos {
                z: target.z - 1,
                ..target
            },
            Tile::Solid(Material::Stone),
        ));
        assert!(world.set_tile(target, Tile::Solid(material)));
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: rect(target, target),
        });

        let at = first_item(&mut world, 600, "dig");
        assert_eq!(at, target, "the dug item lies on the dug cell");
        assert_eq!(world.tile(target), Some(Tile::Empty));
        assert_eq!(
            kinds_at(&world, target, ItemKind::Stone),
            1,
            "digging {material:?} must yield one stone, got {:?}",
            world.item_kinds()
        );
        assert_eq!(world.items().len(), 1, "one dig, one item");
    }
}

#[test]
fn a_dig_mark_takes_a_ramp_and_the_ramp_is_dug_out() {
    for (material, kind) in [
        (Material::Stone, ItemKind::Stone),
        (Material::Tetrahedrite, ItemKind::Tetrahedrite),
    ] {
        let mut world = World::generate(42, Dims::DEFAULT);
        let (target, _pile) = ore_fixture(&mut world);
        assert!(world.set_tile(
            Pos {
                z: target.z - 1,
                ..target
            },
            Tile::Solid(Material::Stone),
        ));
        assert!(world.set_tile(target, Tile::Ramp(material)));
        let refusal = world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: rect(target, target),
        });
        assert_eq!(refusal, None, "a dig over a ramp must not be refused");
        assert_eq!(
            world.designations(),
            vec![(target, DesignationKind::Dig)],
            "the dig mark must land on the ramp"
        );

        let at = first_item(&mut world, 600, "ramp dig");
        assert_eq!(at, target);
        assert_eq!(world.tile(target), Some(Tile::Empty));
        assert_eq!(kinds_at(&world, target, kind), 1);
    }
}

// 12.10b re-pin (was `..._and_leaves_a_stone_ramp`, which pinned that a channel over ore left
// `Ramp(Stone)` and landed the ore on the target): the cell under the target is dug out and yields
// the ore, which lands beside the worker; the ramp is cut in the cell under THAT and keeps its own
// material, ore included (D1), since that cell was not dug and nothing is yielded twice.
#[test]
fn channelling_over_ore_yields_the_ore_and_opens_a_step_with_its_own_ramp() {
    for (material, kind, base_material) in [
        (
            Material::Tetrahedrite,
            ItemKind::Tetrahedrite,
            Material::Bloodstone,
        ),
        (
            Material::Bloodstone,
            ItemKind::Bloodstone,
            Material::Tetrahedrite,
        ),
    ] {
        let mut world = World::generate(42, Dims::DEFAULT);
        let (target, _pile) = ore_fixture(&mut world);
        let below = Pos {
            z: target.z - 1,
            ..target
        };
        let base = Pos {
            z: target.z - 2,
            ..target
        };
        make_standable(&mut world, target);
        assert!(world.set_tile(below, Tile::Solid(material)));
        assert!(world.set_tile(base, Tile::Solid(base_material)));
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Channel,
            rect: rect(target, target),
        });

        let at = first_item(&mut world, 600, "channel");
        assert_ne!(at, target, "the ore must land beside the worker");
        assert_eq!(world.tile(below), Some(Tile::Empty));
        assert_eq!(
            world.tile(base),
            Some(Tile::Ramp(base_material)),
            "the ramp keeps the material it was cut from"
        );
        assert_eq!(
            kinds_at(&world, at, kind),
            1,
            "a channel over {material:?} must yield the ore, got {:?}",
            world.item_kinds()
        );
        assert_eq!(kinds_at(&world, at, ItemKind::Stone), 0);
    }
}

// 12.10b: a channel opens a step down, so a crew can dig into the mountain.

fn cell(origin: Pos, dx: i32, dz: i32) -> Pos {
    Pos {
        x: origin.x + dx,
        z: origin.z + dz,
        ..origin
    }
}

fn is_rock(world: &World, pos: Pos) -> bool {
    matches!(
        world.tile(pos),
        Some(Tile::Solid(material)) if !matches!(material, Material::TreeTrunk | Material::TreeFoliage)
    )
}

/// The nearest camp-level spot for a row of five cells running +x, with a standable y-neighbour on
/// both sides, no emitter on it and six solid cells under every cell: room for a staircase four
/// levels deep and two wide. DEFAULT_SEED gives [61, 62, 9].
fn staircase_site(world: &World) -> Pos {
    let camp = world.camp_origin();
    let emitters: Vec<Pos> = world.emitters().into_iter().map(|(_, p, _)| p).collect();
    let mut best: Option<(i32, Pos)> = None;
    for y in camp.y - 20..camp.y + 20 {
        for x in camp.x - 20..camp.x + 20 {
            let fits = (-1..6).all(|dx| {
                (-1..=1).all(|dy| {
                    let pos = Pos {
                        x: x + dx,
                        y: y + dy,
                        z: camp.z,
                    };
                    is_standable(world, pos)
                        && !emitters.contains(&pos)
                        && (1..7).all(|depth| is_rock(world, cell(pos, 0, -depth)))
                })
            });
            let distance = (x - camp.x).abs() + (y - camp.y).abs();
            if fits && distance > 3 && best.is_none_or(|(nearest, _)| distance < nearest) {
                best = Some((distance, Pos { x, y, z: camp.z }));
            }
        }
    }
    best.expect("DEFAULT_SEED has a staircase site by the camp")
        .1
}

/// A pile big enough for a four-level two-wide staircase: it digs 28 blocks and a pile cell holds
/// one (12.9).
fn place_creation_pile(world: &mut World) {
    let camp = world.camp_origin();
    let refused = world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(
            Pos {
                x: camp.x - 8,
                y: camp.y + 2,
                ..camp
            },
            Pos {
                x: camp.x + 8,
                y: camp.y + 9,
                ..camp
            },
        ),
    });
    assert_eq!(refused, None, "the creation pile was refused");
}

/// Steps until `done` or `limit` ticks have passed; true when it was done.
fn run_until(world: &mut World, limit: u64, mut done: impl FnMut(&World) -> bool) -> bool {
    let end = world.tick() + limit;
    while world.tick() < end {
        if done(world) {
            return true;
        }
        world.step();
    }
    done(world)
}

fn no_marks_left(world: &World) -> bool {
    world.designations().is_empty()
}

fn loose_items_on_air(world: &World) -> Vec<(Id, Pos)> {
    let carried: BTreeSet<u32> = world
        .carrying()
        .into_iter()
        .filter_map(|(_, item)| item)
        .collect();
    world
        .items()
        .into_iter()
        .filter(|(id, pos)| {
            !carried.contains(&id.0)
                && world.tile(*pos) == Some(Tile::Empty)
                && world.tile(Pos {
                    z: pos.z - 1,
                    ..*pos
                }) == Some(Tile::Empty)
        })
        .collect()
}

#[test]
fn a_loose_stone_falls_to_standable_ground_in_one_tick() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    let shaft = Pos {
        z: site.z - 1,
        ..site
    };
    assert!(world.set_tile(shaft, Tile::Empty));
    let mut save = world.to_save();
    let id = save.next_id;
    save.next_id += 1;
    save.items.push((id, site, ItemKind::Stone));
    let mut world = World::from_save(save);
    world.step();
    assert_eq!(
        world
            .items()
            .into_iter()
            .find(|(item, _)| item.0 == id)
            .unwrap()
            .1,
        shaft
    );
}

/// A one-cell shaft under the staircase site: its floor is a ramp, so a dwarf at the bottom can
/// step up onto the level-9 cells around the site. Returns (site, shaft bottom).
fn shaft_under_the_site(world: &mut World) -> (Pos, Pos) {
    let site = staircase_site(world);
    let bottom = cell(site, 0, -1);
    assert!(world.set_tile(bottom, Tile::Empty));
    assert!(world.set_tile(cell(site, 0, -2), Tile::Ramp(Material::Stone)));
    (site, bottom)
}

// AC 2: a fallen stone never rests on a dwarf's tile. A dwarf who cannot leave the shaft bottom
// holds the landing, so the stone must come to rest beside it, on the site's own level.
#[test]
fn a_fallen_stone_never_lands_on_a_dwarf() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let (site, bottom) = shaft_under_the_site(&mut world);
    let mut save = world.to_save();
    let id = save.next_id;
    save.next_id += 1;
    save.items.push((id, site, ItemKind::Stone));
    save.dwarves[0].pos = bottom;
    let mut world = World::from_save(save);
    world.step();
    assert_eq!(world.dwarves()[0].1, bottom, "the dwarf left the landing");
    let stone = world
        .items()
        .into_iter()
        .find(|(item, _)| item.0 == id)
        .unwrap()
        .1;
    assert!(
        world.dwarves().iter().all(|(_, pos, ..)| *pos != stone),
        "the stone landed on a dwarf at {stone:?}"
    );
    assert_eq!(stone.z, site.z, "the stone did not rest beside the shaft");
}

// AC 2 / FR50: a fallen stone never rests on an emitter's tile. A stone over the campfire falls
// onto the fire's own (standable) cell unless the landing refuses it, so it must rest beside the
// fire, on the fire's level.
#[test]
fn a_fallen_stone_never_lands_on_an_emitters_tile() {
    let world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let fire = world
        .emitters()
        .into_iter()
        .find(|(_, _, kind)| *kind == sim_core::LightKind::Campfire)
        .unwrap()
        .1;
    let above = Pos {
        z: fire.z + 1,
        ..fire
    };
    assert_eq!(world.tile(above), Some(Tile::Empty), "air over the fire");
    assert!(is_standable(&world, fire), "the fire's cell is a landing");
    let mut save = world.to_save();
    let id = save.next_id;
    save.next_id += 1;
    save.items.push((id, above, ItemKind::Stone));
    let mut world = World::from_save(save);
    world.step();
    let stone = world
        .items()
        .into_iter()
        .find(|(item, _)| item.0 == id)
        .unwrap()
        .1;
    assert_ne!(stone, fire, "the stone landed on the campfire's tile");
    assert_eq!(
        stone.z, fire.z,
        "the stone did not come down beside the fire"
    );
}

// AC 2: a fallen stone never comes to rest higher than the cell it fell from. Both cells the stone
// could use at its own level are held by dwarves who cannot leave, and the only free standable
// cell reachable is one level ABOVE the site, up a ramp step: the stone stays where it is.
#[test]
fn a_fallen_stone_never_lands_above_where_it_started() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let (site, bottom) = shaft_under_the_site(&mut world);
    let beside = cell(site, 1, 0);
    assert!(world.set_tile(cell(site, 1, -1), Tile::Ramp(Material::Stone)));
    for wall in [
        cell(site, -1, 0),
        Pos {
            y: site.y - 1,
            ..site
        },
        Pos {
            y: site.y + 1,
            ..site
        },
        Pos {
            y: site.y - 1,
            ..beside
        },
        Pos {
            y: site.y + 1,
            ..beside
        },
        cell(site, 2, 0),
    ] {
        assert!(world.set_tile(wall, Tile::Solid(Material::Stone)));
    }
    let rim = cell(site, 2, 1);
    assert!(is_standable(&world, rim), "the cell above the wall is free");
    let mut save = world.to_save();
    let id = save.next_id;
    save.next_id += 1;
    save.items.push((id, site, ItemKind::Stone));
    save.dwarves[0].pos = bottom;
    save.dwarves[1].pos = beside;
    let mut world = World::from_save(save);
    world.step();
    assert_eq!(world.dwarves()[0].1, bottom);
    assert_eq!(world.dwarves()[1].1, beside);
    let stone = world
        .items()
        .into_iter()
        .find(|(item, _)| item.0 == id)
        .unwrap()
        .1;
    assert!(
        stone.z <= site.z,
        "the stone came to rest above where it started: {stone:?}"
    );
}

// AC 4 / D6: the miner never sets the stone down in his target's column. His only work cell is east
// of the target and the cell nearest to him, once he lifts the stone, is the target itself.
#[test]
fn a_clearing_miner_never_sets_the_stone_down_in_the_target_column() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    for wall in [
        cell(site, -1, 0),
        Pos {
            y: site.y - 1,
            ..site
        },
        Pos {
            y: site.y + 1,
            ..site
        },
    ] {
        assert!(world.set_tile(wall, Tile::Solid(Material::Stone)));
    }
    let mut save = world.to_save();
    let id = save.next_id;
    save.next_id += 1;
    save.items.push((id, site, ItemKind::Stone));
    let mut world = World::from_save(save);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(site, site),
    });
    let in_column = |world: &World| {
        world
            .items()
            .into_iter()
            .any(|(item, pos)| item.0 == id && (pos.x, pos.y) == (site.x, site.y))
    };
    let mut lifted = false;
    for _ in 0..2_500 {
        world.step();
        lifted |= world.carrying().iter().any(|(_, item)| *item == Some(id));
        if lifted && world.designations().is_empty() {
            break;
        }
    }
    assert!(lifted, "the miner never lifted the stone");
    assert!(
        world.designations().is_empty(),
        "the channel never completed"
    );
    assert!(!in_column(&world), "the stone is in the target's column");
}

/// Row 7's "the column alone is equivalent" was unproven for a dig (12.11 review run 1): a dig's
// target is solid at the miner's level, so the cell on its top is refused by the column clause
// ALONE. Here that cell is the miner's nearest set-down: the stone lies on the dig's one face, and
// ramp floors let him step up onto the target's top. Checked every tick, since after the dig a
// stone set there falls and can leave the column again.
#[test]
fn a_clearing_miner_never_sets_the_stone_down_on_his_dig_targets_top() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    // From the site's own cell: one cell east, dwarf 4 spawns on `at(4, 0, 0)`.
    let at = |dx: i32, dy: i32, dz: i32| Pos {
        x: site.x + dx,
        y: site.y + dy,
        z: site.z + dz,
    };
    let (miner, face, target, beyond) = (at(0, 0, 0), at(1, 0, 0), at(2, 0, 0), at(3, 0, 1));
    // The target's top keeps a second same-level exit, `aside`, and `beyond` and `aside` each have
    // one more of their own, so no stone set on any of the three closes the last exit of a miner
    // standing beside it (#186). Then only the column clause refuses the target's top.
    let aside = at(2, 1, 1);
    let mut solid = vec![target, at(3, 0, 0), at(2, 1, 0), at(4, 0, 0), at(2, 2, 0)];
    for (dx, dy) in [(-1, 0), (0, -1), (0, 1), (1, -1), (1, 1), (2, -1)] {
        solid.extend([at(dx, dy, 0), at(dx, dy, 1)]);
    }
    for cell in &solid {
        world.set_tile(*cell, Tile::Solid(Material::Stone));
    }
    for cell in [miner, face] {
        world.set_tile(
            Pos {
                z: cell.z - 1,
                ..cell
            },
            Tile::Ramp(Material::Stone),
        );
    }
    for cell in [
        miner,
        face,
        Pos {
            z: target.z + 1,
            ..target
        },
        beyond,
        aside,
        at(4, 0, 1),
        at(2, 2, 1),
    ] {
        assert_eq!(world.tile(cell), Some(Tile::Empty), "{cell:?} is not open");
    }
    let mut save = world.to_save();
    let stone = save.next_id;
    save.next_id += 1;
    save.items.push((stone, face, ItemKind::Stone));
    for (index, dwarf) in save.dwarves.iter_mut().enumerate() {
        assert!(
            !solid.contains(&dwarf.pos),
            "dwarf {index} is inside the fixture at {:?}",
            dwarf.pos
        );
        dwarf.profession = if index == 0 {
            Profession::Miner
        } else {
            Profession::Hauler
        };
    }
    save.dwarves[0].pos = miner;
    save.dwarves[0].home = miner;
    let mut world = World::from_save(save);
    assert_eq!(
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: rect(target, target),
        }),
        None
    );
    let mut lifted = false;
    while !world.designations().is_empty() {
        assert!(world.tick() < 1_000, "the dig never completed");
        world.step();
        let held = world.carrying()[0].1 == Some(stone);
        lifted |= held;
        let lies = world
            .items()
            .into_iter()
            .find(|(item, _)| item.0 == stone)
            .unwrap()
            .1;
        assert!(
            held || (lies.x, lies.y) != (target.x, target.y),
            "tick {}: the stone lies at {lies:?}, in the dig target's column",
            world.tick()
        );
    }
    assert!(lifted, "the miner never lifted the stone");
}

// 12.11 seat: the miner (dwarf 0) on `miner` beside a stone on a channel mark at `target`, shut in
/// `cells` at the staircase site's level by stone walls. Every other dwarf is a hauler and there is
/// no pile, so nobody else works; `extra` puts dwarf 1 on that cell. Returns the stone's id.
fn walled_clearing(cells: &[Pos], target: Pos, miner: Pos, extra: Option<Pos>) -> (World, u32) {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    for cell in cells {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let side = Pos {
                x: cell.x + dx,
                y: cell.y + dy,
                ..*cell
            };
            if !cells.contains(&side) {
                assert!(world.set_tile(side, Tile::Solid(Material::Stone)));
            }
        }
    }
    let mut save = world.to_save();
    let id = save.next_id;
    save.next_id += 1;
    save.items.push((id, target, ItemKind::Stone));
    for (index, dwarf) in save.dwarves.iter_mut().enumerate() {
        dwarf.profession = if index == 0 {
            Profession::Miner
        } else {
            Profession::Hauler
        };
    }
    for (index, pos) in [(0, Some(miner)), (1, extra)] {
        if let Some(pos) = pos {
            save.dwarves[index].pos = pos;
            save.dwarves[index].home = pos;
        }
    }
    let mut world = World::from_save(save);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(target, target),
    });
    (world, id)
}

/// Steps until the miner (dwarf 0) carries `stone`; panics after `limit` ticks.
fn until_the_miner_lifts(world: &mut World, stone: u32, limit: u64) {
    assert!(
        run_until(world, limit, |world| world.carrying()[0].1 == Some(stone)),
        "the miner never lifted the stone: claims {:?}, dwarves {:?}",
        world.claims(),
        world.dwarves()
    );
}

// Wolf at the seat, 12.11 ("throwing stones"): a miner lifts only a stone he has somewhere to set
// down. His whole ground is the mark and his own cell, so there is none: before the guard he lifted
// it anyway and stood holding it.
#[test]
fn a_miner_with_nowhere_to_set_a_stone_down_never_lifts_it() {
    let world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    let target = cell(site, 1, 0);
    let miner = cell(site, 2, 0);
    let (mut world, stone) = walled_clearing(&[target, miner], target, miner, None);
    let mut claimed = false;
    for _ in 0..600 {
        world.step();
        claimed |= world.claims()[0].1.is_some();
        assert_eq!(
            world.carrying()[0].1,
            None,
            "tick {}: the miner lifted a stone with nowhere to set it down",
            world.tick()
        );
    }
    assert!(claimed, "the miner never claimed the channel");
    assert!(world.items().contains(&(Id(stone), target)));
}

// A miner who lets go of his stone mid-clear (his trade changes) sets it down beside him or on his
// own tile. Before the guard the drop searched at any distance and, with his only sides the mark and
// a hauler shut in a pocket, landed three cells off at the corridor's end.
#[test]
fn a_miner_who_lets_go_mid_clear_sets_the_stone_down_beside_him() {
    let world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    let far = cell(site, -1, 0);
    let target = cell(site, 1, 0);
    let miner = cell(site, 2, 0);
    let pocket = Pos {
        y: site.y + 1,
        ..miner
    };
    let cells = [far, site, target, miner, pocket];
    let (mut world, stone) = walled_clearing(&cells, target, miner, Some(pocket));
    until_the_miner_lifts(&mut world, stone, 600);
    let at = world.dwarves()[0].1;
    assert_eq!(at, miner, "the miner lifted from another cell");
    let refused = world.apply_command(SimCommand::SetProfession {
        dwarf: world.dwarves()[0].0,
        profession: Profession::Hauler,
    });
    assert_eq!(refused, None);
    let landed = world
        .items()
        .into_iter()
        .find(|(item, _)| item.0 == stone)
        .unwrap()
        .1;
    assert!(
        (landed.x - at.x).abs().max((landed.y - at.y).abs()) <= 1 && (landed.z - at.z).abs() <= 1,
        "the stone landed at {landed:?}, more than a cell from the miner on {at:?}"
    );
}

// 12.11 review run 1: a cancel removes the job BEFORE `release_claim`, so the beside-him guard,
// keyed on the job, was skipped and the drop searched at any distance. The let-go above keeps its job.
#[test]
fn a_miner_whose_channel_is_cancelled_mid_clear_sets_the_stone_down_beside_him() {
    let world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    let far = cell(site, -1, 0);
    let target = cell(site, 1, 0);
    let miner = cell(site, 2, 0);
    let (mut world, stone) = walled_clearing(&[far, site, target, miner], target, miner, None);
    until_the_miner_lifts(&mut world, stone, 600);
    let at = world.dwarves()[0].1;
    assert_eq!(at, miner, "the miner lifted from another cell");
    // A hauler now stands on the mark, his one side, resting so it does not wander off: every cell
    // beside him is refused, so the stone stays on his own tile.
    let mut save = world.to_save();
    save.dwarves[1].pos = target;
    save.dwarves[1].home = target;
    save.dwarves[1].cooldown = 100;
    save.dwarves[1].path.clear();
    let mut world = World::from_save(save);
    let refused = world.apply_command(SimCommand::CancelDesignation {
        rect: rect(target, target),
    });
    assert_eq!(refused, None);
    assert_eq!(
        world.carrying()[0].1,
        None,
        "the cancel left him holding it"
    );
    let landed = world
        .items()
        .into_iter()
        .find(|(item, _)| item.0 == stone)
        .unwrap()
        .1;
    assert!(
        (landed.x - at.x).abs().max((landed.y - at.y).abs()) <= 1 && (landed.z - at.z).abs() <= 1,
        "the stone landed at {landed:?}, more than a cell from the miner on {at:?}"
    );
}

// 12.11 review run 2: the let-go above refuses every side, so the stone always landed on his own
// tile and "never below him" went unpinned. Here an opened pit step is his one free neighbour.
#[test]
fn a_miner_whose_channel_is_cancelled_mid_clear_never_sets_the_stone_down_below_him() {
    let world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    let far = cell(site, -1, 0);
    let target = cell(site, 1, 0);
    let miner = cell(site, 2, 0);
    let step = cell(site, 3, -1);
    let (mut world, stone) = walled_clearing(&[far, site, target, miner], target, miner, None);
    until_the_miner_lifts(&mut world, stone, 600);
    let at = world.dwarves()[0].1;
    assert_eq!(at, miner, "the miner lifted from another cell");
    // The hauler rests on the mark, his one side, and the step a channel opened lies east of him.
    let mut save = world.to_save();
    save.dwarves[1].pos = target;
    save.dwarves[1].home = target;
    save.dwarves[1].cooldown = 100;
    save.dwarves[1].path.clear();
    let mut world = World::from_save(save);
    assert!(world.set_tile(cell(site, 3, 0), Tile::Empty));
    assert!(world.set_tile(step, Tile::Empty));
    assert!(world.set_tile(cell(site, 3, -2), Tile::Ramp(Material::Stone)));
    // Its only way up is his tile: a step that also led out to open ground would be his way out,
    // and filling it would wall him in, a refusal of its own.
    for dy in [-1, 0, 1] {
        let up = Pos {
            y: site.y + dy,
            ..cell(site, 3 + (dy == 0) as i32, 0)
        };
        assert!(world.set_tile(up, Tile::Solid(Material::Stone)));
    }
    assert!(is_standable(&world, step), "the step is not open");
    let refused = world.apply_command(SimCommand::CancelDesignation {
        rect: rect(target, target),
    });
    assert_eq!(refused, None);
    assert_eq!(
        world.carrying()[0].1,
        None,
        "the cancel left him holding it"
    );
    assert!(
        world.items().contains(&(Id(stone), at)),
        "the stone left the miner's tile on {at:?}: {:?}",
        world.items().into_iter().find(|(item, _)| item.0 == stone)
    );
}

/// DEFAULT_SEED with a stone on a one-cell channel mark at the staircase site. Returns the stone.
fn a_stone_on_a_channel_mark() -> (World, u32) {
    let world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    let mut save = world.to_save();
    let id = save.next_id;
    save.next_id += 1;
    save.items.push((id, site, ItemKind::Stone));
    let mut world = World::from_save(save);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(site, site),
    });
    (world, id)
}

// D7 / AC 15, 12.11 review run 1: `clearing()` (the wire's `haul`) names a miner from the start of
// the lift run to the set-down, not while he walks to the stone empty-handed.
#[test]
fn a_miner_reads_as_clearing_only_while_he_lifts_or_carries() {
    let (mut world, stone) = a_stone_on_a_channel_mark();
    let (mut lifting, mut carrying) = (false, false);
    while !world.designations().is_empty() {
        assert!(world.tick() < 2_000, "the channel never completed");
        world.step();
        let held: BTreeMap<Id, Option<u32>> = world.carrying().into_iter().collect();
        let states: BTreeMap<Id, JobState> = world
            .dwarves()
            .into_iter()
            .map(|(id, _, state, _)| (id, state))
            .collect();
        for (miner, item) in world.clearing() {
            assert_eq!(item, stone);
            if held[&miner] == Some(item) {
                carrying = true;
            } else {
                assert_eq!(
                    states[&miner],
                    JobState::Work,
                    "tick {}: dwarf {miner:?} reads as clearing, holding nothing and not lifting",
                    world.tick()
                );
                lifting = true;
            }
        }
    }
    assert!(lifting, "no miner read as clearing during his lift run");
    assert!(
        carrying,
        "no miner read as clearing while he carried the stone"
    );
}

// AC 15, 12.11 review run 1: the set-down leaves him walking, as a haul's pick-up does. Left in
// `Work`, the next delta read `channel` + `work`: one frame of the dig swing.
#[test]
fn a_miner_who_sets_his_stone_down_is_walking_not_working() {
    let (mut world, stone) = a_stone_on_a_channel_mark();
    let mut carrier = None;
    let mut set_down = false;
    while !world.designations().is_empty() && !set_down {
        assert!(world.tick() < 2_000, "the channel never completed");
        world.step();
        let now = world
            .carrying()
            .into_iter()
            .find(|(_, item)| *item == Some(stone))
            .map(|(id, _)| id);
        if let (Some(miner), None) = (carrier, now) {
            let state = world
                .dwarves()
                .into_iter()
                .find(|(id, ..)| *id == miner)
                .unwrap()
                .2;
            assert_eq!(
                state,
                JobState::Walk,
                "tick {}: set down while working",
                world.tick()
            );
            set_down = true;
        }
        carrier = now;
    }
    assert!(set_down, "the miner never set the stone down");
}

// A miner holding his stone when a dwarf steps onto his only set-down side keeps it and looks again,
// rather than give the job up. Before the guard he let go on the next tick, and the drop put it
// under him, where 12.9's rule says a stone never lands.
#[test]
fn a_miner_whose_set_down_side_is_taken_keeps_the_stone() {
    let world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    let far = cell(site, -1, 0);
    let target = cell(site, 1, 0);
    let miner = cell(site, 2, 0);
    let (mut world, stone) = walled_clearing(&[far, site, target, miner], target, miner, None);
    until_the_miner_lifts(&mut world, stone, 600);
    // A hauler now stands on the one side of the set-down cell, resting so it does not wander off.
    let mut save = world.to_save();
    save.dwarves[1].pos = site;
    save.dwarves[1].home = site;
    save.dwarves[1].cooldown = 100;
    save.dwarves[1].path.clear();
    let mut world = World::from_save(save);
    for _ in 0..20 {
        world.step();
        assert_eq!(
            world.carrying()[0].1,
            Some(stone),
            "tick {}: the miner let go of the stone while his set-down side was taken",
            world.tick()
        );
    }
}

#[test]
fn a_channel_miner_carries_its_target_stone_aside_before_cutting() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    let mut save = world.to_save();
    let id = save.next_id;
    save.next_id += 1;
    save.items.push((id, site, ItemKind::Stone));
    world = World::from_save(save);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(site, site),
    });
    let mut saw_miner_carry = false;
    for _ in 0..2_000 {
        world.step();
        saw_miner_carry |= world.carrying().iter().any(|(dwarf, item)| {
            *item == Some(id) && world.professions().contains(&(*dwarf, Profession::Miner))
        });
        if world.designations().is_empty() {
            break;
        }
    }
    assert!(
        saw_miner_carry,
        "channel miner never carried the target stone"
    );
    assert!(
        world.designations().is_empty(),
        "channel mark never completed: claims {:?}, carrying {:?}, dwarves {:?}, items {:?}",
        world.claims(),
        world.carrying(),
        world.dwarves(),
        world.items()
    );
    assert!(
        loose_items_on_air(&world).is_empty(),
        "channel left floating items: {:?}",
        loose_items_on_air(&world)
    );
}

// D6: a hauler already holding the haul job of the stone a miner lifts lets go, so the stone never
// has two holders; the haul job is claimed again once the stone is down and the stone reaches the
// pile. On DEFAULT_SEED hauler 3 claims the job at t40 and miner 2 lifts the stone at t50.
#[test]
fn a_hauler_holding_the_lifted_stones_job_lets_go_and_the_stone_is_delivered_later() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let camp = world.camp_origin();
    let site = staircase_site(&world);
    let mut save = world.to_save();
    let id = save.next_id;
    save.next_id += 1;
    save.items.push((id, site, ItemKind::Stone));
    world = World::from_save(save);
    let pile = rect(
        Pos {
            x: camp.x - 8,
            y: camp.y + 2,
            ..camp
        },
        Pos {
            x: camp.x - 6,
            y: camp.y + 4,
            ..camp
        },
    );
    assert_eq!(
        world.apply_command(SimCommand::PlaceStockpile { rect: pile }),
        None
    );
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(site, site),
    });
    let mut hauler_held_first = false;
    let mut miner_lifted = false;
    let mut delivered = false;
    for _ in 0..3_000 {
        world.step();
        let claims = world.claims();
        let jobs = world.jobs();
        let haulers: Vec<_> = claims
            .iter()
            .filter(|(_, job)| {
                job.and_then(|job| jobs.iter().find(|candidate| candidate.id == job))
                    .is_some_and(|job| job.kind == JobKind::Haul { item: id })
            })
            .map(|(dwarf, _)| *dwarf)
            .collect();
        let carriers: Vec<_> = world
            .carrying()
            .into_iter()
            .filter(|(_, item)| *item == Some(id))
            .map(|(dwarf, _)| dwarf)
            .collect();
        let miner_carries = carriers
            .iter()
            .any(|dwarf| world.professions().contains(&(*dwarf, Profession::Miner)));
        assert!(
            carriers.len() <= 1
                && (carriers.is_empty() || haulers.iter().all(|h| carriers.contains(h))),
            "tick {}: the stone has two holders; haulers {haulers:?}, carriers {carriers:?}",
            world.tick()
        );
        if !miner_lifted && !haulers.is_empty() && carriers.is_empty() {
            hauler_held_first = true;
        }
        miner_lifted |= miner_carries;
        let at = world
            .items()
            .into_iter()
            .find(|(item, _)| item.0 == id)
            .unwrap()
            .1;
        if miner_lifted && carriers.is_empty() && world.zones().contains(&at) {
            delivered = true;
            break;
        }
    }
    assert!(
        hauler_held_first,
        "no hauler held the stone's job before the lift"
    );
    assert!(miner_lifted, "the miner never lifted the stone");
    assert!(delivered, "the stone never reached the pile");
}

// AC 3 (12.11 review run 1: no sim test pinned it). The OLD world: a haul job created while the
// stone lay on its ground, its target that ground. The floor then goes out from under it; the stone
// falls a level, and a hauler lifts it from its live position (the ramp under it is the step up)
// and delivers it. Before 12.11 it hung over air and its job retried for good.
#[test]
fn a_fallen_stone_with_a_haul_job_is_delivered_to_the_pile() {
    let world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    let ground = cell(site, 2, 0);
    let mut save = world.to_save();
    let stone = save.next_id;
    save.next_id += 1;
    save.items.push((stone, ground, ItemKind::Stone));
    let mut world = World::from_save(save);
    place_creation_pile(&mut world);
    world.step();
    let job = world
        .jobs()
        .into_iter()
        .find(|job| job.kind == JobKind::Haul { item: stone })
        .expect("the stone has a haul job");
    assert_eq!(
        job.target, ground,
        "the haul job was made with the stone on its ground"
    );
    assert!(
        world.claims().iter().all(|(_, held)| *held != Some(job.id)),
        "a hauler already holds the job, so it was not made in the old world"
    );
    let below = cell(ground, 0, -1);
    assert!(world.set_tile(below, Tile::Empty));
    assert!(world.set_tile(cell(ground, 0, -2), Tile::Ramp(Material::Stone)));
    world.step();
    assert!(
        world.items().contains(&(Id(stone), below)),
        "the stone did not fall a level: {:?}",
        world.items().into_iter().find(|(id, _)| id.0 == stone)
    );
    let zones: BTreeSet<Pos> = world.zones().into_iter().collect();
    let delivered = run_until(&mut world, 3_000, |world| {
        world
            .carrying()
            .iter()
            .all(|(_, held)| *held != Some(stone))
            && world
                .items()
                .iter()
                .any(|(id, at)| id.0 == stone && zones.contains(at))
    });
    assert!(
        delivered,
        "the fallen stone never reached the pile: {:?}, jobs {:?}",
        world.items().into_iter().find(|(id, _)| id.0 == stone),
        world.jobs()
    );
}

#[test]
fn a_dig_under_a_stone_does_not_leave_it_floating() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    let target = Pos {
        z: site.z - 1,
        ..site
    };
    let pit = Pos {
        y: site.y + 1,
        ..target
    };
    world.set_tile(pit, Tile::Empty);
    let ramp = Pos {
        x: pit.x + 1,
        ..pit
    };
    world.set_tile(ramp, Tile::Empty);
    world.set_tile(
        Pos {
            z: ramp.z - 1,
            ..ramp
        },
        Tile::Ramp(Material::Stone),
    );
    let mut save = world.to_save();
    let id = save.next_id;
    save.next_id += 1;
    save.items.push((id, site, ItemKind::Stone));
    let mut world = World::from_save(save);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(target, target),
    });
    assert!(
        run_until(&mut world, 4_000, |world| world.tile(target)
            == Some(Tile::Empty)),
        "dig did not complete"
    );
    assert!(
        loose_items_on_air(&world).is_empty(),
        "dig left floating items: {:?}",
        loose_items_on_air(&world)
    );
}

#[test]
fn digging_hillside_tree_ground_fells_the_tree() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let ground = Pos { x: 103, y: 2, z: 8 };
    let base = Pos { z: 9, ..ground };
    assert_eq!(world.tile(base), Some(Tile::Solid(Material::TreeTrunk)));
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(ground, ground),
    });
    assert!(
        run_until(&mut world, 4_000, |world| world.tile(ground)
            == Some(Tile::Empty)),
        "tree ground was not dug"
    );
    assert_eq!(
        world.tile(base),
        Some(Tile::Empty),
        "tree base remains over the dug ground"
    );
}

#[test]
fn digging_a_marked_trees_ground_releases_its_woodcutter() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let ground = Pos { x: 103, y: 2, z: 8 };
    let base = Pos { z: 9, ..ground };
    let cut = rect(base, base);
    let dig = rect(ground, ground);
    assert!(
        world
            .apply_command(SimCommand::Designate {
                kind: DesignationKind::Cut,
                rect: cut
            })
            .is_none()
    );
    assert!(
        world
            .apply_command(SimCommand::Designate {
                kind: DesignationKind::Dig,
                rect: dig
            })
            .is_none()
    );
    world.step();
    let cut_job = world
        .jobs()
        .into_iter()
        .find(|job| job.kind == JobKind::Cut)
        .expect("cut job")
        .id;
    let mut save = world.to_save();
    let woodcutter = save
        .dwarves
        .iter_mut()
        .find(|dwarf| dwarf.profession == sim_core::Profession::Woodcutter)
        .expect("woodcutter");
    woodcutter.current_job = Some(cut_job.0);
    world = World::from_save(save);
    assert!(
        run_until(&mut world, 4_000, |world| world.tile(ground)
            == Some(Tile::Empty)),
        "ground was not dug"
    );
    assert_eq!(world.tile(base), Some(Tile::Empty));
    assert!(
        run_until(&mut world, 4_000, |world| world
            .jobs()
            .iter()
            .all(|job| job.kind != JobKind::Cut)),
        "held cut did not complete its no-tree no-op"
    );
    assert!(
        world
            .claims()
            .into_iter()
            .all(|(_, job)| job != Some(cut_job)),
        "woodcutter kept the removed tree's job"
    );
}

#[test]
fn a_miner_clears_a_four_mark_tunnel_without_a_pile() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let worker = world.dwarves()[2].1;
    let dx = if worker.x + 10 < world.dims().x as i32 {
        1
    } else {
        -1
    };
    for step in 1..=5 {
        make_standable(
            &mut world,
            Pos {
                x: worker.x + dx * step,
                ..worker
            },
        );
    }
    let first = Pos {
        x: worker.x + dx * 6,
        ..worker
    };
    let last = Pos {
        x: worker.x + dx * 9,
        ..worker
    };
    for step in 6..=9 {
        let target = Pos {
            x: worker.x + dx * step,
            ..worker
        };
        world.set_tile(
            Pos {
                z: target.z - 1,
                ..target
            },
            Tile::Solid(Material::Stone),
        );
        world.set_tile(target, Tile::Solid(Material::Stone));
        for dy in [-1, 1] {
            world.set_tile(
                Pos {
                    y: target.y + dy,
                    ..target
                },
                Tile::Solid(Material::Stone),
            );
        }
    }
    world.set_tile(
        Pos {
            x: last.x + dx,
            ..last
        },
        Tile::Solid(Material::Stone),
    );
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(first, last),
    });
    assert!(
        run_until(&mut world, 4_000, |world| world.tile(last)
            == Some(Tile::Empty)),
        "miner stopped before the fourth mark; remaining {:?}",
        world.designations()
    );
    assert!(world.zones().is_empty());
}

// Wolf's 12.11 seat save (`12-11-dig-loop.save`, miner Dori at 52,63,8): a miner in a two-cell
// pocket whose only exit holds a stone, the pile full, so no hauler ever takes it. His channel in
// the pocket finds no cell for its block, is refused, and he swings at it forever. He must carry
// the stone out of the way himself, as he does a stone on his target's face (D6).
#[test]
fn a_miner_caged_by_one_stone_carries_it_out_and_cuts() {
    let dims = Dims { x: 14, y: 8, z: 4 };
    let index = |pos: Pos| {
        pos.x as usize
            + pos.y as usize * dims.x as usize
            + pos.z as usize * dims.x as usize * dims.y as usize
    };
    let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    for x in 0..dims.x as i32 {
        for y in 0..dims.y as i32 {
            for z in 0..2 {
                tiles[index(Pos { x, y, z })] = Tile::Solid(Material::Stone);
            }
            // The pocket's walls: x 0..=2 is rock at the dwarf's level but for the pocket.
            if x <= 2 {
                tiles[index(Pos { x, y, z: 2 })] = Tile::Solid(Material::Stone);
            }
        }
    }
    let miner = Pos { x: 1, y: 3, z: 2 };
    let target = Pos { x: 2, y: 3, z: 2 };
    let exit = Pos { x: 3, y: 3, z: 2 };
    tiles[index(miner)] = Tile::Empty;
    tiles[index(target)] = Tile::Empty;

    let mut save = World::generate(42, Dims::DEFAULT).to_save();
    save.dims = dims;
    save.tiles = tiles;
    save.designations.clear();
    save.zones.clear();
    save.jobs.clear();
    save.emitters.clear();
    save.dwarves = vec![SavedDwarf {
        id: 0,
        pos: miner,
        state: JobState::Idle,
        home: miner,
        cooldown: 0,
        current_job: None,
        work_progress: 0,
        carrying: None,
        identity: Identity {
            name: DwarfName::Durin,
            colour: DwarfColour::Red,
        },
        profession: Profession::Miner,
        path: Vec::new(),
        clearing: None,
    }];
    save.items = vec![(1, exit, ItemKind::Stone)];
    save.next_id = 2;
    let mut world = World::from_save(save);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(target, target),
    });

    let mut miner_carried = false;
    for _ in 0..1_000 {
        world.step();
        miner_carried |= world.carrying().iter().any(|(_, item)| *item == Some(1));
        if world.designations().is_empty() {
            break;
        }
    }
    assert!(
        miner_carried,
        "the miner never lifted the stone in his exit"
    );
    assert!(
        world.designations().is_empty(),
        "the caged miner's channel never completed: dwarves {:?}, items {:?}",
        world.dwarves(),
        world.items()
    );
    assert!(
        loose_items_on_air(&world).is_empty(),
        "floating items: {:?}",
        loose_items_on_air(&world)
    );
}

// RED on 015548f: the cell under a channel became a Ramp, so nothing opened and the pit below was
// never there to walk into.
#[test]
fn a_channel_opens_a_step_down_and_a_dwarf_walks_down_and_back() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(site, site),
    });
    assert!(
        run_until(&mut world, 3_000, no_marks_left),
        "the channel at {site:?} was never worked"
    );
    assert_eq!(world.tile(cell(site, 0, -1)), Some(Tile::Empty));
    assert!(
        matches!(world.tile(cell(site, 0, -2)), Some(Tile::Ramp(_))),
        "the cell two under the target must be a ramp, got {:?}",
        world.tile(cell(site, 0, -2))
    );

    // A dig workable only from inside the pit: the cell beside the opened one, one level down.
    let wall = Pos {
        y: site.y + 1,
        ..cell(site, 0, -1)
    };
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(wall, wall),
    });
    let mut pit_dwarf = None;
    let dug = run_until(&mut world, 4_000, |world| {
        for (id, pos, ..) in world.dwarves() {
            if pos.z == site.z - 1 {
                pit_dwarf = Some(id);
            }
        }
        world.tile(wall) == Some(Tile::Empty)
    });
    assert!(dug, "the dig at {wall:?} from the pit never finished");
    let pit_dwarf = pit_dwarf.expect("a dwarf stood in the pit to dig it");

    // Walk it back up: the other miner takes another trade and a channel is marked up top, which a
    // dwarf in the pit may still hold.
    for (id, profession) in world.professions() {
        if profession == Profession::Miner && id != pit_dwarf {
            world.apply_command(SimCommand::SetProfession {
                dwarf: id,
                profession: Profession::Hauler,
            });
        }
    }
    let up_top = cell(site, 4, 0);
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Channel,
        rect: rect(up_top, up_top),
    });
    assert!(
        run_until(&mut world, 4_000, |world| {
            world
                .dwarves()
                .iter()
                .any(|(id, pos, ..)| *id == pit_dwarf && pos.z == site.z)
        }),
        "the dwarf in the pit never climbed back to z {}",
        site.z
    );
}

// Wolf at the seat, 12.11: "miners are throwing stones quite wildly". RED on 3d9488b from delay 0: a
// miner lifted a stone, found no way to a set-down side, retried, and `release_claim` dropped it on
// the nearest free cell at any distance, up to five cells off and three levels up, on the pit rim.
// Green on main (56bfbe5): every carrier there, haulers included, sets its stone down beside it.
#[test]
fn a_carried_stone_is_always_set_down_beside_its_carrier() {
    let mut thrown = Vec::new();
    for delay in 0..24u64 {
        let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
        let site = staircase_site(&world);
        place_creation_pile(&mut world);
        // The stones carried at the end of the last tick, and where (on the carrier's tile). Read
        // every tick, the designation delays included: a record left stale over them made haulers
        // look as if they threw.
        let mut carried: BTreeMap<u32, Pos> = BTreeMap::new();
        let mut carries = 0;
        let mut watch = |world: &World| {
            let items: BTreeMap<u32, Pos> = world
                .items()
                .into_iter()
                .map(|(id, pos)| (id.0, pos))
                .collect();
            let held: BTreeSet<u32> = world
                .carrying()
                .into_iter()
                .filter_map(|(_, item)| item)
                .collect();
            for (item, was) in &carried {
                let now = items[item];
                if !held.contains(item)
                    && ((now.x - was.x).abs().max((now.y - was.y).abs()) > 1
                        || (now.z - was.z).abs() > 1)
                {
                    thrown.push((delay, world.tick(), *was, now));
                }
            }
            carries += held
                .iter()
                .filter(|item| !carried.contains_key(item))
                .count();
            carried = held.iter().map(|item| (*item, items[item])).collect();
        };
        for level in 0..4 {
            for _ in 0..delay {
                world.step();
                watch(&world);
            }
            let first = cell(site, level, -level);
            let last = Pos {
                y: site.y + 1,
                ..cell(site, 4, -level)
            };
            world.apply_command(SimCommand::Designate {
                kind: DesignationKind::Channel,
                rect: rect(first, last),
            });
            assert!(
                run_until(&mut world, 6_000, |world| {
                    watch(world);
                    no_marks_left(world)
                }),
                "delay {delay}: level {level} was never worked"
            );
        }
        // Latent vacuity (12.11 review run 1): with nothing carried, nothing can be thrown.
        assert!(carries > 0, "delay {delay}: no stone was ever carried");
    }
    assert!(
        thrown.is_empty(),
        "stones landed more than a cell from their carrier (delay, tick, carrier tile, landing): {thrown:?}"
    );
}

// RED on 015548f (0 of 24: a channel only ramps the floor, nothing opens).
#[test]
fn a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay() {
    let mut worked = 0;
    let mut stalled = Vec::new();
    let mut floated = Vec::new();
    for delay in 0..24u64 {
        let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
        let site = staircase_site(&world);
        place_creation_pile(&mut world);
        let mut levels = 0;
        let mut air = BTreeSet::new();
        for level in 0..4 {
            for _ in 0..delay {
                world.step();
                air.extend(loose_items_on_air(&world));
            }
            // Each level is one row shorter and one level lower, inside the previous one.
            let first = cell(site, level, -level);
            let last = Pos {
                y: site.y + 1,
                ..cell(site, 4, -level)
            };
            world.apply_command(SimCommand::Designate {
                kind: DesignationKind::Channel,
                rect: rect(first, last),
            });
            // Worked means every mark is gone AND every floor under the level is open: on
            // 015548f a channel only ramps the floor, which clears the marks and opens nothing.
            let opened = |world: &World| {
                no_marks_left(world)
                    && (first.x..=last.x).all(|x| {
                        (first.y..=last.y).all(|y| {
                            world.tile(cell(Pos { x, y, ..first }, 0, -1)) == Some(Tile::Empty)
                        })
                    })
            };
            if !run_until(&mut world, 6_000, |world| {
                air.extend(loose_items_on_air(world));
                for (_, pos, ..) in world.dwarves() {
                    assert!(
                        is_standable(world, pos)
                            || world.tile(Pos {
                                z: pos.z - 1,
                                ..pos
                            }) == Some(Tile::Empty),
                        "dwarf on unsupported non-falling tile at tick {}: {pos:?}",
                        world.tick()
                    );
                }
                opened(world)
            }) {
                break;
            }
            levels = level + 1;
        }
        if !air.is_empty() {
            floated.push((delay, air.len()));
        }
        if levels == 4 {
            worked += 1;
        } else {
            stalled.push((delay, levels));
        }
    }
    assert_eq!(
        worked, 24,
        "staircase worked for {worked} of 24 designation delays; stalled (delay, levels done): {stalled:?}"
    );
    assert!(floated.is_empty(), "floating items by delay: {floated:?}");
}

/// One step, panicking on the tick two dwarves share a tile or trade tiles (no swap, 12.9 Task 0
/// Q1b). Notes every dwarf seen at `low_z` or lower in `low`, and returns whether one of them is now
/// back on `top_z`.
fn step_and_watch(world: &mut World, low: &mut BTreeSet<u32>, low_z: i32, top_z: i32) -> bool {
    let before: Vec<Pos> = world.dwarves().iter().map(|dwarf| dwarf.1).collect();
    world.step();
    let now: Vec<Pos> = world.dwarves().iter().map(|dwarf| dwarf.1).collect();
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
                "dwarves {a} and {b} swapped tiles at tick {tick}"
            );
        }
    }
    let mut back_up = false;
    for (id, pos, ..) in world.dwarves() {
        if pos.z <= low_z {
            low.insert(id.0);
        } else if pos.z == top_z && low.contains(&id.0) {
            back_up = true;
        }
    }
    back_up
}

// #194 (12.10b review, ruled into the story by Wolf): RED before the fix at delays 0-3 and 8, level 3
// never worked: an idle miner parks on the one-wide stair's last step, the holder's only work cell
// is past it, and the line behind the holder leaves it no way out.
#[test]
fn a_one_wide_staircase_reaches_four_levels_down_and_a_dwarf_climbs_back_for_every_delay() {
    let mut failed = Vec::new();
    for delay in 0..24u64 {
        let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
        let site = staircase_site(&world);
        place_creation_pile(&mut world);
        let (mut low, mut back_up) = (BTreeSet::new(), false);
        let mut levels = 0;
        for level in 0..4 {
            for _ in 0..delay {
                back_up |= step_and_watch(&mut world, &mut low, site.z - 3, site.z);
            }
            let (first, last) = (cell(site, level, -level), cell(site, 4, -level));
            world.apply_command(SimCommand::Designate {
                kind: DesignationKind::Channel,
                rect: rect(first, last),
            });
            let opened = |world: &World| {
                no_marks_left(world)
                    && (first.x..=last.x).all(|x| {
                        world.tile(Pos {
                            x,
                            z: first.z - 1,
                            ..first
                        }) == Some(Tile::Empty)
                    })
            };
            let end = world.tick() + 6_000;
            while !opened(&world) && world.tick() < end {
                back_up |= step_and_watch(&mut world, &mut low, site.z - 3, site.z);
            }
            if !opened(&world) {
                break;
            }
            levels = level + 1;
        }
        // The way up. Haulers usually take it, lifting blocks; when no block needed lifting from
        // that deep, work marked up top calls a dwarf from the bottom (as in AC 3).
        if levels == 4 && !back_up {
            let up_top = Pos {
                y: site.y - 1,
                ..cell(site, 3, 0)
            };
            world.apply_command(SimCommand::Designate {
                kind: DesignationKind::Channel,
                rect: rect(up_top, up_top),
            });
            let end = world.tick() + 4_000;
            while !back_up && world.tick() < end {
                back_up = step_and_watch(&mut world, &mut low, site.z - 3, site.z);
            }
        }
        if levels < 4 || !back_up {
            failed.push((delay, levels, back_up));
        }
    }
    assert_eq!(
        failed,
        Vec::new(),
        "one-wide staircase failed (delay, levels done, a dwarf back up)"
    );
}

// #196 (12.10b review run 2, ruled into the story by Wolf): RED before the fix, level 2 never worked.
// Level 0 is marked 190 ticks in and each later level 8 after the last is open (the wire's timing:
// a daemon's uptime at connect, then its read lag). A hauler at (62,62,7) heads for the dead end
// (64,62,6) to lift a block, and the miner standing on its own mark (63,62,7) heads for (62,62,7)
// to work it. Head-on, both holding jobs, neither with an escape: both waited forever, while the
// miner's other work cell, the dead end itself, was free.
#[test]
fn job_holders_head_on_in_a_one_wide_staircase_do_not_wait_forever() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let site = staircase_site(&world);
    place_creation_pile(&mut world);
    let mut low = BTreeSet::new();
    for _ in 0..190 {
        step_and_watch(&mut world, &mut low, site.z - 3, site.z);
    }
    let mut levels = 0;
    for level in 0..4 {
        if level > 0 {
            for _ in 0..8 {
                step_and_watch(&mut world, &mut low, site.z - 3, site.z);
            }
        }
        let (first, last) = (cell(site, level, -level), cell(site, 4, -level));
        world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Channel,
            rect: rect(first, last),
        });
        let opened = |world: &World| {
            no_marks_left(world)
                && (first.x..=last.x).all(|x| {
                    world.tile(Pos {
                        x,
                        z: first.z - 1,
                        ..first
                    }) == Some(Tile::Empty)
                })
        };
        let end = world.tick() + 6_000;
        while !opened(&world) && world.tick() < end {
            step_and_watch(&mut world, &mut low, site.z - 3, site.z);
        }
        if !opened(&world) {
            break;
        }
        levels = level + 1;
    }
    assert_eq!(
        levels,
        4,
        "levels worked; still marked at tick {}: {:?}",
        world.tick(),
        world.designations()
    );
}

// ---- 12.12: the goal chain on DEFAULT_SEED ----
//
// The plan is `12-12-signoff/plan-10-10-1.json` (made by `goals_wire.py` from the snapshot at tick 6
// on the wire, byte-identical on 3 of 3 daemons), ported as literals: 46 digs to the nearest
// tetrahedrite and bloodstone, 3 cuts, and 3 compact piles of 30/30/18 cells that enclose nothing,
// so a pile-fill regression cannot masquerade as a goals failure (Task 2b's ring test judges that).

const CHAIN_DIGS: [(i32, i32, i32); 46] = [
    (73, 64, 11),
    (74, 64, 11),
    (75, 64, 11),
    (76, 64, 11),
    (77, 64, 11),
    (77, 65, 11),
    (78, 64, 11),
    (68, 76, 14),
    (68, 77, 14),
    (68, 78, 14),
    (68, 79, 14),
    (69, 79, 14),
    (69, 73, 12),
    (69, 74, 12),
    (69, 75, 12),
    (69, 76, 12),
    (69, 77, 12),
    (69, 78, 12),
    (69, 79, 12),
    (51, 72, 12),
    (51, 73, 12),
    (51, 74, 12),
    (51, 75, 12),
    (51, 76, 12),
    (51, 77, 12),
    (50, 77, 12),
    (49, 77, 12),
    (48, 77, 12),
    (48, 78, 12),
    (48, 79, 12),
    (48, 80, 12),
    (47, 80, 12),
    (46, 80, 12),
    (46, 81, 12),
    (46, 82, 12),
    (46, 83, 12),
    (55, 74, 13),
    (55, 75, 13),
    (55, 76, 13),
    (55, 77, 13),
    (55, 78, 13),
    (55, 79, 13),
    (55, 80, 13),
    (55, 81, 13),
    (55, 82, 13),
    (55, 83, 13),
];
const CHAIN_CUTS: [(i32, i32, i32); 3] = [(65, 56, 10), (61, 74, 13), (65, 73, 12)];
const CHAIN_PILES: [&[(i32, i32, i32)]; 3] = [
    &[
        (67, 63, 9),
        (66, 63, 9),
        (68, 63, 9),
        (67, 62, 9),
        (67, 64, 9),
        (66, 62, 9),
        (68, 62, 9),
        (68, 64, 9),
        (67, 61, 9),
        (67, 65, 9),
        (65, 62, 9),
        (66, 61, 9),
        (68, 65, 9),
        (67, 60, 9),
        (66, 65, 9),
        (67, 66, 9),
        (65, 61, 9),
        (66, 60, 9),
        (66, 66, 9),
        (67, 67, 9),
        (64, 61, 9),
        (65, 60, 9),
        (66, 59, 9),
        (65, 66, 9),
        (66, 67, 9),
        (63, 61, 9),
        (64, 60, 9),
        (65, 59, 9),
        (65, 67, 9),
        (62, 61, 9),
    ],
    &[
        (61, 65, 9),
        (60, 65, 9),
        (62, 65, 9),
        (61, 64, 9),
        (61, 66, 9),
        (59, 65, 9),
        (60, 64, 9),
        (60, 66, 9),
        (62, 66, 9),
        (61, 63, 9),
        (61, 67, 9),
        (58, 65, 9),
        (59, 64, 9),
        (59, 66, 9),
        (60, 63, 9),
        (60, 67, 9),
        (63, 66, 9),
        (62, 67, 9),
        (62, 63, 9),
        (57, 65, 9),
        (58, 64, 9),
        (58, 66, 9),
        (59, 63, 9),
        (59, 67, 9),
        (60, 62, 9),
        (63, 67, 9),
        (57, 64, 9),
        (58, 63, 9),
        (59, 62, 9),
        (60, 61, 9),
    ],
    &[
        (62, 69, 10),
        (61, 69, 10),
        (63, 69, 10),
        (62, 68, 10),
        (62, 70, 10),
        (60, 69, 10),
        (61, 68, 10),
        (64, 69, 10),
        (63, 70, 10),
        (59, 69, 10),
        (60, 68, 10),
        (65, 69, 10),
        (64, 70, 10),
        (58, 69, 10),
        (59, 68, 10),
        (66, 69, 10),
        (65, 70, 10),
        (58, 68, 10),
    ],
];

const CHAIN_SNAPSHOT_TICK: u64 = 6;
/// Measured: wood completes at tick 1,847, tetrahedrite and (by the cascade, bloodstone was stored
/// at 6,352) bloodstone at 9,828, the same ticks as the wire oracle. Pinned at 10,500: a 672-tick
/// (6.8%) margin, which is what the unpaused wire form drifted by (10,454).
const CHAIN_BOUND_TICK: u64 = 10_500;

fn at3((x, y, z): (i32, i32, i32)) -> Pos {
    Pos { x, y, z }
}

/// The stored count per kind from public readers only: 3.3's rule (uncarried, on a zone cell, the
/// lowest id on that cell), re-derived here so it does not share code with the sim.
fn stored_by_kind(world: &World) -> BTreeMap<ItemKind, u32> {
    let carried: BTreeSet<u32> = world.carrying().iter().filter_map(|(_, c)| *c).collect();
    let kinds: BTreeMap<Id, ItemKind> = world.item_kinds().into_iter().collect();
    let zones: BTreeSet<Pos> = world.zones().into_iter().collect();
    let mut occupied = BTreeSet::new();
    let mut stored = BTreeMap::new();
    for (id, pos) in world.items() {
        if carried.contains(&id.0) || !zones.contains(&pos) || !occupied.insert(pos) {
            continue;
        }
        *stored.entry(kinds[&id]).or_insert(0) += 1;
    }
    stored
}

#[test]
fn the_default_world_completes_the_goal_chain_with_scripted_commands() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    for _ in 0..CHAIN_SNAPSHOT_TICK {
        world.step();
    }
    for cell in CHAIN_DIGS {
        let pos = at3(cell);
        let refused = world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Dig,
            rect: rect(pos, pos),
        });
        assert_eq!(refused, None, "dig {pos:?} was refused");
    }
    for cell in CHAIN_CUTS {
        let pos = at3(cell);
        let refused = world.apply_command(SimCommand::Designate {
            kind: DesignationKind::Cut,
            rect: rect(pos, pos),
        });
        assert_eq!(refused, None, "cut {pos:?} was refused");
    }
    let pile_rects: Vec<Rect> = CHAIN_PILES
        .iter()
        .flat_map(|pile| pile.iter())
        .map(|&cell| rect(at3(cell), at3(cell)))
        .collect();
    assert_eq!(pile_rects.len(), 78);
    assert_eq!(
        world.place_stockpile(&pile_rects),
        None,
        "the piles were refused"
    );

    let fresh = world.goals();
    assert_eq!(
        fresh.iter().map(|g| (g.kind, g.target)).collect::<Vec<_>>(),
        vec![
            (ItemKind::Wood, 10),
            (ItemKind::Tetrahedrite, 10),
            (ItemKind::Bloodstone, 1)
        ]
    );

    let mut done_at: [Option<u64>; 3] = [None; 3];
    while world.tick() < CHAIN_BOUND_TICK && done_at[2].is_none() {
        world.step();
        let goals = world.goals();
        let stored = stored_by_kind(&world);
        for (k, goal) in goals.iter().enumerate() {
            // The oracle: progress IS the stored count of the kind, every tick.
            let want = stored.get(&goal.kind).copied().unwrap_or(0);
            assert_eq!(
                goal.progress,
                want,
                "tick {}: {:?} progress {} but {want} are stored",
                world.tick(),
                goal.kind,
                goal.progress
            );
            // Never earlier than the goal ahead, and never undone.
            if goal.completed {
                assert!(
                    k == 0 || goals[k - 1].completed,
                    "tick {}: goal {k} is completed before goal {}",
                    world.tick(),
                    k - 1
                );
                if done_at[k].is_none() {
                    done_at[k] = Some(world.tick());
                    assert!(
                        want >= goal.target,
                        "tick {}: {:?} completed on {want} of {}",
                        world.tick(),
                        goal.kind,
                        goal.target
                    );
                }
            } else {
                assert!(done_at[k].is_none(), "goal {k} went back to incomplete");
            }
        }
    }
    assert!(
        done_at[2].is_some(),
        "the chain was not done by tick {CHAIN_BOUND_TICK}: {:?}",
        world.goals()
    );

    let done_tick = world.tick();
    for _ in 0..200 {
        world.step();
    }
    assert_eq!(world.tick(), done_tick + 200, "the tick has climbed");
    assert!(
        world.goals().iter().all(|g| g.completed),
        "a goal un-completed after the chain: {:?}",
        world.goals()
    );
}

// ---- 12.12 Task 2b: a pile round walkable ground it does not zone ----

/// The creation shape (12.12 Q3): a pile band ringing the campfire, with the fire's 4 neighbours
/// left out of the zone. Those 4 cells and the fire are walkable ground the zone surrounds.
/// Chebyshev distance 1..=2 from the fire, minus the 4 side neighbours: 20 pile cells.
fn ring_pile(fire: Pos) -> Vec<Pos> {
    let mut cells = Vec::new();
    for dy in -2..=2i32 {
        for dx in -2..=2i32 {
            let ring = dx.abs().max(dy.abs());
            let hole = dx.abs() + dy.abs() == 1;
            if ring >= 1 && !hole {
                cells.push(Pos {
                    x: fire.x + dx,
                    y: fire.y + dy,
                    ..fire
                });
            }
        }
    }
    cells
}

/// Places the pile `ring` makes round the fire on flat ground, turns `rock` (cells of the unzoned
/// hole) to `Solid(Stone)`, lays 26 loose stones south of it, and runs until it is full or 6,000
/// ticks pass. Asserts AC 11 by 3.3's stored rule (`World::goals` counts by it; stones have no
/// goal, so this counts the same way: one uncarried item per pile cell): every cell stored, and no
/// haul job sat in retry cooldown for long while a cell was still free. A sealed ring retried
/// every tick for good; a fill that works sees at most a cooldown or two while haulers jostle.
fn assert_ring_fills(ring: fn(Pos) -> Vec<Pos>, rock: &[Pos]) {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let fire = world.camp_origin();
    // Flat open ground round the camp, so nothing but the zone shapes the fill.
    for y in fire.y - 10..=fire.y + 10 {
        for x in fire.x - 10..=fire.x + 10 {
            let pos = Pos { x, y, z: fire.z };
            if !is_standable(&world, pos) {
                make_standable(&mut world, pos);
            }
        }
    }
    let pile = ring(fire);
    let rects: Vec<Rect> = pile.iter().map(|&cell| rect(cell, cell)).collect();
    assert_eq!(world.place_stockpile(&rects), None, "the ring was refused");
    for &cell in rock {
        assert!(
            !pile.contains(&cell) && cell != fire,
            "{cell:?} is not in the hole"
        );
        assert!(world.set_tile(cell, Tile::Solid(Material::Stone)));
    }

    // 26 loose stones in two rows south of the ring, more than there are pile cells.
    let mut save = world.to_save();
    let mut id = save.next_id;
    for y in fire.y + 5..=fire.y + 6 {
        for x in fire.x - 6..fire.x + 7 {
            save.items
                .push((id, Pos { x, y, z: fire.z }, ItemKind::Stone));
            id += 1;
        }
    }
    save.next_id = id;
    assert!(save.items.len() >= 26);
    let mut world = World::from_save(save);

    let zones: BTreeSet<Pos> = world.zones().into_iter().collect();
    let stored = |world: &World| {
        uncarried_item_tiles(world)
            .iter()
            .filter(|pos| zones.contains(pos))
            .count()
    };
    let mut last_rise = 0;
    let mut best = 0;
    let end = world.tick() + 6_000;
    let mut retry_ticks = 0;
    while world.tick() < end && best < pile.len() {
        world.step();
        let now = stored(&world);
        if world
            .jobs()
            .iter()
            .any(|job| matches!(job.kind, JobKind::Haul { .. }) && job.retry_after > world.tick())
        {
            retry_ticks += 1;
        }
        if now > best {
            best = now;
            last_rise = world.tick();
        }
    }
    let tiles = uncarried_item_tiles(&world);
    let free: Vec<Pos> = pile
        .iter()
        .copied()
        .filter(|cell| !tiles.contains(cell))
        .collect();
    let hauls = world
        .jobs()
        .iter()
        .filter(|job| matches!(job.kind, JobKind::Haul { .. }))
        .count();
    assert!(
        free.is_empty(),
        "the ring sealed itself: {best} of {} cells stored, last at tick {last_rise} (now {}); \
         {} free pile cells {free:?} while {hauls} haul jobs retry",
        pile.len(),
        world.tick(),
        free.len()
    );
    assert!(
        retry_ticks <= 100,
        "haul jobs sat in retry cooldown for {retry_ticks} ticks while a pile cell was free"
    );
}

#[test]
fn a_pile_round_enclosed_ground_still_fills_every_cell() {
    assert_ring_fills(ring_pile, &[]);
}

/// 12.12 review run 1, Decision 1 (a): a hole is a pocket when its flood runs dry, whatever borders
/// it. One boulder at the fire's west neighbour made the hole touch rock, which read as a way out:
/// 4 cells sealed and 10 haul jobs retried for good.
#[test]
fn a_pile_round_ground_beside_rock_still_fills_every_cell() {
    let fire = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT).camp_origin();
    assert_ring_fills(
        ring_pile,
        &[Pos {
            x: fire.x - 1,
            ..fire
        }],
    );
}

/// 12.12 review run 2, Decision 1 (a): two piles that ring a hole together, touching only at
/// corners. The west bar is a pile of its own, and its box (one column) holds none of the hole; the
/// other pile's box holds all of it. A flood bounded by its own pile's box called the hole open
/// from the bar's side, and that sealed the band as a hole read as a way in did (Task 2b).
/// The ring's cells less the four that touch the bar: 3 + 13 pile cells round a 7-cell hole.
fn joint_ring_pile(fire: Pos) -> Vec<Pos> {
    let bar = [(-2, -1), (-2, 0), (-2, 1)];
    let cut = [(-2, -2), (-2, 2), (-1, -1), (-1, 1)];
    let cells: Vec<(i32, i32)> = ring_pile(fire)
        .into_iter()
        .map(|cell| (cell.x - fire.x, cell.y - fire.y))
        .filter(|cell| !cut.contains(cell))
        .collect();
    assert_eq!(cells.len(), 16);
    // Two piles: no cell of the bar has a side neighbour in the rest.
    for (x, y) in bar {
        for n in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            assert!(
                bar.contains(&n) || !cells.contains(&n),
                "{n:?} touches the bar"
            );
        }
    }
    cells
        .into_iter()
        .map(|(dx, dy)| Pos {
            x: fire.x + dx,
            y: fire.y + dy,
            ..fire
        })
        .collect()
}

#[test]
fn two_piles_that_ring_ground_together_still_fill_every_cell() {
    assert_ring_fills(joint_ring_pile, &[]);
}
