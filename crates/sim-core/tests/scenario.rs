use std::collections::BTreeSet;

use sim_core::{
    DesignationKind, Dims, JobId, JobKind, JobState, Material, Pos, Rect, SimCommand, Tile, World,
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
            .find(|(id, _, _)| *id == holder)
            .map(|(_, _, state)| state),
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
                .find(|(id, _, _)| *id == holder)
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
    assert_eq!(world.items(), vec![(sim_core::Id(5), target)]);
    assert!(world.jobs().is_empty());
    assert!(world.claims().iter().all(|(_, job)| job.is_none()));
    assert!(world.designations().is_empty());
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
    world.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: rect(outer, inner),
    });

    for _ in 0..500 {
        world.step();
        if world.tile(inner) == Some(Tile::Empty) {
            break;
        }
    }

    assert_eq!(world.tile(outer), Some(Tile::Empty));
    assert_eq!(world.tile(inner), Some(Tile::Empty));
    assert!(world.items().iter().any(|(_, pos)| *pos == outer));
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
    assert_eq!(world.jobs()[0].kind, JobKind::Haul { item: 5 });
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
    assert_eq!(job.kind, JobKind::Haul { item: 5 });

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

#[test]
fn removing_every_stockpile_drops_the_carried_stone_and_a_new_pile_revives_the_job() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let stone = dig_one_stone(&mut world);
    let dx = (stone.x - world.dwarves()[2].1.x).signum();
    for distance in 1..=3 {
        make_standable(
            &mut world,
            Pos {
                x: stone.x + dx * distance,
                ..stone
            },
        );
    }
    let pile = Pos {
        x: stone.x + dx * 3,
        ..stone
    };
    world.apply_command(SimCommand::PlaceStockpile {
        rect: rect(pile, pile),
    });

    while world.carrying().iter().all(|(_, item)| item.is_none()) {
        assert!(world.tick() < 400, "nobody ever picked the stone up");
        world.step();
    }

    world.apply_command(SimCommand::RemoveStockpile {
        rect: rect(pile, pile),
    });
    for _ in 0..40 {
        world.step();
    }

    assert!(world.zones().is_empty());
    assert_eq!(
        world.jobs().len(),
        1,
        "the haul job was dropped, not parked"
    );
    assert_eq!(world.jobs()[0].kind, JobKind::Haul { item: 5 });
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
        world.step();
        if world.jobs().is_empty() {
            break;
        }
    }

    assert!(
        world.jobs().is_empty(),
        "the revived job never finished: {:?}",
        world.jobs()
    );
    assert_eq!(world.items(), vec![(sim_core::Id(5), pile)]);
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
        assert_eq!(first.jobs(), second.jobs());
        assert_eq!(first.claims(), second.claims());
        assert_eq!(first.items(), second.items());
        assert_eq!(first.tiles(), second.tiles());
        assert_eq!(first.designations(), second.designations());
        assert_eq!(first.zones(), second.zones());
    }
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
    let (_, first_home, _) = spawns[0];

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
        for (index, (id, pos, _)) in world.dwarves().into_iter().enumerate() {
            debug_assert_eq!(id, before[index].0, "dwarf order is stable");
            if pos != before[index].1 {
                moved[index] = true;
            }
        }
    }

    for (index, (id, pos, _)) in before.into_iter().enumerate() {
        assert!(
            moved[index],
            "dwarf {id:?} has not moved from {pos:?} in 200 ticks — it is stranded outside its \
             wander radius and can never move again"
        );
    }
}
