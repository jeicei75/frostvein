use sim_core::{Dims, Material, Pos, Tile, World};

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
