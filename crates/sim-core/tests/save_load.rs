use sim_core::{Dims, Pos, Tile, World};

const MUTATED_POS: Pos = Pos { x: 0, y: 0, z: 0 };

#[test]
fn save_load_then_tick_matches_never_saved() {
    let mut saved = World::generate(42, Dims::DEFAULT);
    let mut control = World::generate(42, Dims::DEFAULT);

    for _ in 0..37 {
        saved.step();
        control.step();
    }
    assert!(saved.set_tile(MUTATED_POS, Tile::Empty));
    assert!(control.set_tile(MUTATED_POS, Tile::Empty));

    let mut loaded = World::from_save(saved.to_save());
    for _ in 0..200 {
        loaded.step();
        control.step();
        assert_eq!(loaded.tick(), control.tick());
        assert_eq!(loaded.dwarves(), control.dwarves());
        assert_eq!(loaded.tile(MUTATED_POS), Some(Tile::Empty));
        assert_eq!(loaded.tile(MUTATED_POS), control.tile(MUTATED_POS));
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
