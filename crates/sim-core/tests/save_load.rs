use sim_core::{DesignationKind, Dims, Material, Pos, Rect, SimCommand, Tile, World};

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
