use sim_core::{
    DesignationKind, Dims, Job, JobId, JobKind, JobState, Material, Pos, Rect, SimCommand, Tile,
    World,
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
    assert!(saved.set_tile(designation_pos, Tile::Solid(Material::Stone)));
    assert!(control.set_tile(designation_pos, Tile::Solid(Material::Stone)));
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

    while saved.claims().iter().all(|(_, job)| job.is_none()) {
        assert!(saved.tick() < 100, "corridor dig was never claimed");
        saved.step();
        control.step();
        assert_eq!(saved.claims(), control.claims());
    }
    let claimed = saved
        .claims()
        .into_iter()
        .find_map(|(id, job)| job.map(|job| (id, job)))
        .expect("a dwarf holds the dig");
    for _ in 0..3 {
        saved.step();
        control.step();
        assert!(saved.jobs().iter().any(|job| job.id == claimed.1));
        assert_eq!(
            saved.claims().into_iter().find(|(id, _)| *id == claimed.0),
            Some((claimed.0, Some(claimed.1)))
        );
    }
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
        assert_eq!(loaded.items(), control.items());
        assert_eq!(loaded.designations(), control.designations());
        assert_eq!(loaded.zones(), control.zones());
    }
}

#[test]
fn loading_does_not_reuse_entity_ids() {
    let world = World::generate(42, Dims::DEFAULT);
    let loaded = World::from_save(world.to_save());

    assert_eq!(loaded.to_save().next_id, 5);
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
        .any(|(_, _, state)| *state == JobState::Work)
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
