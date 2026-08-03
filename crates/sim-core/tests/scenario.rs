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
    assert!(world.drain_dirty().is_empty());
}

#[test]
fn dirty_tiles_are_sorted_and_out_of_bounds_writes_do_nothing() {
    let mut world = World::generate(42, Dims::DEFAULT);
    let later = Pos { x: 9, y: 2, z: 3 };
    let earlier = Pos { x: 1, y: 2, z: 3 };
    let out_of_bounds = Pos {
        x: world.dims().x as i32,
        y: 0,
        z: 0,
    };

    assert!(world.set_tile(later, Tile::Empty));
    assert!(world.set_tile(earlier, Tile::Ramp(Material::Snow)));
    assert!(!world.set_tile(out_of_bounds, Tile::Solid(Material::Stone)));
    assert_eq!(world.tile(out_of_bounds), None);
    assert_eq!(
        world.drain_dirty(),
        vec![(earlier, Tile::Ramp(Material::Snow)), (later, Tile::Empty),]
    );
    assert!(world.drain_dirty().is_empty());
}
