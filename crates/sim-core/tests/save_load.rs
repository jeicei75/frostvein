use sim_core::{
    DesignationKind, Dims, Job, JobId, JobKind, JobState, LightKind, Material, Pos, Rect,
    SavedDwarf, SimCommand, Tile, WORK_TICKS, World,
};

const MUTATED_POS: Pos = Pos { x: 0, y: 0, z: 0 };

#[test]
fn save_load_then_tick_matches_never_saved() {
    let mut saved = World::generate(42, Dims::DEFAULT);
    let mut control = World::generate(42, Dims::DEFAULT);
    let worker = saved.dwarves()[2].1;
    assert_eq!(worker, control.dwarves()[2].1);
    let dx = if worker.x + 15 < saved.dims().x as i32 {
        1
    } else {
        -1
    };
    for distance in 1..15 {
        let pos = Pos {
            x: worker.x + dx * distance,
            ..worker
        };
        for world in [&mut saved, &mut control] {
            assert!(world.set_tile(
                Pos {
                    z: pos.z - 1,
                    ..pos
                },
                Tile::Solid(Material::Stone),
            ));
            assert!(world.set_tile(pos, Tile::Empty));
        }
    }
    let designation_pos = Pos {
        x: worker.x + dx * 15,
        ..worker
    };
    for world in [&mut saved, &mut control] {
        // Solid floor under the dug tile, so the stone it drops is on standable ground and can
        // actually be picked up — without it the mid-haul save point below is unreachable.
        assert!(world.set_tile(
            Pos {
                z: designation_pos.z - 1,
                ..designation_pos
            },
            Tile::Solid(Material::Stone),
        ));
        assert!(world.set_tile(designation_pos, Tile::Solid(Material::Stone)));
    }
    saved.drain_dirty();
    control.drain_dirty();
    let designation = SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: Rect {
            min: designation_pos,
            max: designation_pos,
        },
    };
    let stockpile_pos = saved.dwarves()[0].1;
    let stockpile = SimCommand::PlaceStockpile {
        rect: Rect {
            min: stockpile_pos,
            max: stockpile_pos,
        },
    };
    saved.apply_command(designation);
    control.apply_command(designation);
    saved.apply_command(stockpile);
    control.apply_command(stockpile);

    // AC11: the save point is a stepped CONDITION, never a magic tick count — the first tick a
    // dwarf is actually holding a stone. That covers the dig, the haul job's creation, its claim
    // and the pick-up in one, and it moves with the sim instead of going quietly vacuous.
    while saved.carrying().iter().all(|(_, item)| item.is_none()) {
        assert!(saved.tick() < 600, "no dwarf ever picked the stone up");
        saved.step();
        control.step();
        assert_eq!(saved.claims(), control.claims());
        assert_eq!(saved.carrying(), control.carrying());
    }
    let carrier = saved
        .carrying()
        .into_iter()
        .find_map(|(id, item)| item.map(|item| (id, item)))
        .expect("a dwarf holds a stone");
    let held = saved
        .claims()
        .into_iter()
        .find(|(id, _)| *id == carrier.0)
        .and_then(|(_, job)| job)
        .expect("a carrying dwarf holds that stone's haul job");
    assert_eq!(
        saved
            .jobs()
            .iter()
            .find(|job| job.id == held)
            .map(|job| job.kind),
        Some(JobKind::Haul { item: carrier.1 })
    );
    assert!(saved.set_tile(MUTATED_POS, Tile::Empty));
    assert!(control.set_tile(MUTATED_POS, Tile::Empty));

    let mut loaded = World::from_save(saved.to_save());
    for _ in 0..200 {
        loaded.step();
        control.step();
        assert_eq!(loaded.tick(), control.tick());
        assert_eq!(loaded.dwarves(), control.dwarves());
        assert_eq!(loaded.tiles(), control.tiles());
        assert_eq!(loaded.jobs(), control.jobs());
        assert_eq!(loaded.claims(), control.claims());
        assert_eq!(loaded.carrying(), control.carrying());
        assert_eq!(loaded.items(), control.items());
        assert_eq!(loaded.emitters(), control.emitters());
        assert_eq!(loaded.designations(), control.designations());
        assert_eq!(loaded.zones(), control.zones());
    }
}

#[test]
fn save_round_trip_preserves_emitters() {
    let world = World::generate(42, Dims::DEFAULT);
    let expected = world.emitters();

    let loaded = World::from_save(world.to_save());

    assert_eq!(loaded.emitters(), expected);
}

#[test]
fn save_round_trip_restores_the_uniform_dwarf_lantern_without_saved_lantern_state() {
    let world = World::generate(42, Dims::DEFAULT);
    let expected = world.dwarves();
    let save = world.to_save();

    assert!(
        expected
            .iter()
            .all(|(_, _, _, light)| *light == LightKind::Lantern),
        "the generated world gives every dwarf the uniform lantern"
    );
    assert_eq!(
        save.dwarves.len(),
        expected.len(),
        "the existing saved dwarf records remain the complete saved dwarf state"
    );
    assert_eq!(World::from_save(save).dwarves(), expected);
}

#[test]
fn save_round_trip_preserves_a_mid_haul_carry() {
    let mut save = World::generate(42, Dims::DEFAULT).to_save();
    let stone = Pos { x: 9, y: 8, z: 7 };
    save.next_id = 13;
    save.items = vec![(12, stone)];
    save.jobs = vec![Job {
        id: JobId(7),
        kind: JobKind::Haul { item: 12 },
        target: stone,
        created_tick: 3,
        retry_after: 0,
    }];
    save.next_job_id = 8;
    save.dwarves[0].current_job = Some(7);
    save.dwarves[0].carrying = Some(12);

    let round_trip = World::from_save(save).to_save();

    assert_eq!(
        round_trip.dwarves.len(),
        5,
        "a dwarf carrying nothing must still reach the save"
    );
    assert_eq!(round_trip.dwarves[0].carrying, Some(12));
    assert!(
        round_trip.dwarves[1..]
            .iter()
            .all(|dwarf| dwarf.carrying.is_none())
    );
    assert_eq!(round_trip.jobs[0].kind, JobKind::Haul { item: 12 });
    assert_eq!(round_trip.items, vec![(12, stone)]);
}

#[test]
fn loading_does_not_reuse_entity_ids() {
    let world = World::generate(42, Dims::DEFAULT);
    let loaded = World::from_save(world.to_save());

    assert_eq!(loaded.to_save().next_id, 10);
}

#[test]
fn save_round_trip_preserves_items_and_current_job() {
    let mut save = World::generate(42, Dims::DEFAULT).to_save();
    let target = Pos { x: 9, y: 8, z: 7 };
    save.next_id = 13;
    save.items = vec![(12, target)];
    save.jobs = vec![Job {
        id: JobId(7),
        kind: JobKind::Dig,
        target,
        created_tick: 3,
        retry_after: 29,
    }];
    save.next_job_id = 8;
    save.dwarves[0].current_job = Some(7);
    save.dwarves[0].work_progress = 3;

    let round_trip = World::from_save(save).to_save();

    assert_eq!(round_trip.items, vec![(12, target)]);
    assert_eq!(round_trip.jobs.len(), 1);
    assert_eq!(round_trip.jobs[0].id, JobId(7));
    assert_eq!(round_trip.jobs[0].kind, JobKind::Dig);
    assert_eq!(round_trip.jobs[0].target, target);
    assert_eq!(round_trip.jobs[0].created_tick, 3);
    assert_eq!(round_trip.jobs[0].retry_after, 29);
    assert_eq!(round_trip.next_job_id, 8);
    assert_eq!(round_trip.dwarves[0].current_job, Some(7));
    assert_eq!(round_trip.dwarves[0].work_progress, 3);
}

#[test]
fn save_load_preserves_in_progress_work() {
    let mut control = World::generate(42, Dims::DEFAULT);
    let worker = control.dwarves()[2].1;
    let target = Pos {
        x: worker.x + 1,
        ..worker
    };
    assert!(control.set_tile(target, Tile::Solid(Material::Stone)));
    control.drain_dirty();
    control.apply_command(SimCommand::Designate {
        kind: DesignationKind::Dig,
        rect: Rect {
            min: target,
            max: target,
        },
    });
    while !control
        .dwarves()
        .iter()
        .any(|(_, _, state, _)| *state == JobState::Work)
    {
        assert!(control.tick() < 100, "adjacent dig never reached work");
        control.step();
    }
    control.step();

    let mut loaded = World::from_save(control.to_save());
    for _ in 0..6 {
        control.step();
        loaded.step();
        assert_eq!(loaded.dwarves(), control.dwarves());
        assert_eq!(loaded.jobs(), control.jobs());
        assert_eq!(loaded.claims(), control.claims());
        assert_eq!(loaded.items(), control.items());
        assert_eq!(loaded.tile(target), control.tile(target));
    }
}

#[test]
fn save_load_recomputes_every_path_invalidated_by_another_dig() {
    let dims = Dims { x: 7, y: 3, z: 3 };
    let index = |pos: Pos| {
        pos.x as usize
            + pos.y as usize * dims.x as usize
            + pos.z as usize * dims.x as usize * dims.y as usize
    };
    let mut tiles = vec![Tile::Solid(Material::Stone); (dims.x * dims.y * dims.z) as usize];
    for x in 0..=5 {
        tiles[index(Pos { x, y: 1, z: 2 })] = Tile::Empty;
    }
    let walking_start = Pos { x: 0, y: 1, z: 2 };
    let walking_target = Pos { x: 6, y: 1, z: 2 };
    let digging_start = Pos { x: 3, y: 0, z: 1 };
    let digging_target = Pos { x: 3, y: 1, z: 1 };
    tiles[index(digging_start)] = Tile::Empty;

    let mut save = World::generate(42, Dims::DEFAULT).to_save();
    save.tick = 100;
    save.dims = dims;
    save.tiles = tiles;
    save.next_id = 2;
    save.dwarves = vec![
        SavedDwarf {
            id: 0,
            pos: walking_start,
            state: JobState::Walk,
            home: walking_start,
            cooldown: 10,
            current_job: Some(0),
            work_progress: 0,
            carrying: None,
        },
        SavedDwarf {
            id: 1,
            pos: digging_start,
            state: JobState::Work,
            home: digging_start,
            cooldown: 10,
            current_job: Some(1),
            work_progress: WORK_TICKS,
            carrying: None,
        },
    ];
    save.designations = vec![
        (walking_target, DesignationKind::Dig),
        (digging_target, DesignationKind::Dig),
    ];
    save.zones.clear();
    save.jobs = vec![
        Job {
            id: JobId(0),
            kind: JobKind::Dig,
            target: walking_target,
            created_tick: 0,
            retry_after: 0,
        },
        Job {
            id: JobId(1),
            kind: JobKind::Dig,
            target: digging_target,
            created_tick: 0,
            retry_after: 0,
        },
    ];
    save.next_job_id = 2;
    save.items.clear();

    let mut control = World::from_save(save);
    control.step();
    assert_eq!(control.tile(digging_target), Some(Tile::Empty));
    let mut loaded = World::from_save(control.to_save());

    for _ in 0..5 {
        control.step();
        loaded.step();
        assert_eq!(loaded.dwarves(), control.dwarves());
        assert_eq!(loaded.jobs(), control.jobs());
        assert_eq!(loaded.claims(), control.claims());
        assert_eq!(loaded.items(), control.items());
        assert_eq!(loaded.tiles(), control.tiles());
    }
}

#[test]
fn loading_starts_with_no_dirty_tiles() {
    let mut world = World::generate(42, Dims::DEFAULT);
    assert!(world.set_tile(MUTATED_POS, Tile::Empty));

    let mut loaded = World::from_save(world.to_save());

    assert!(loaded.drain_dirty().is_empty());
}

#[test]
fn save_orders_dwarves_by_id() {
    let world = World::generate(42, Dims::DEFAULT);
    let mut save = world.to_save();
    save.dwarves.reverse();

    let ids: Vec<_> = World::from_save(save)
        .to_save()
        .dwarves
        .into_iter()
        .map(|dwarf| dwarf.id)
        .collect();

    assert_eq!(ids, vec![0, 1, 2, 3, 4]);
}
