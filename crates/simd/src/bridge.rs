pub fn snapshot(world: &sim_core::World) -> protocol::Snapshot {
    let dims = world.dims();
    // The wire contract tells clients to index tiles as x + y*dims.x + z*dims.x*dims.y
    // (protocol::Snapshot::tiles). If the two accessors ever disagree that formula
    // reads out of bounds in the client, so tie them together here.
    debug_assert_eq!(
        world.tiles().len(),
        dims.x as usize * dims.y as usize * dims.z as usize,
        "tile count must match the dims the wire advertises"
    );
    protocol::Snapshot {
        msg_type: protocol::MessageType::Snapshot,
        dims: protocol::Dims {
            x: dims.x,
            y: dims.y,
            z: dims.z,
        },
        tiles: world.tiles().iter().copied().map(tile).collect(),
        entities: world
            .dwarves()
            .into_iter()
            .map(|(id, pos)| protocol::Entity {
                id: id.0,
                kind: protocol::EntityKind::Dwarf,
                pos: [pos.x, pos.y, pos.z],
            })
            .collect(),
        designations: Vec::new(),
        zones: Vec::new(),
        speed: protocol::Speed::Normal,
        tick: world.tick(),
    }
}

pub fn delta(world: &mut sim_core::World) -> protocol::Delta {
    protocol::Delta {
        msg_type: protocol::MessageType::Delta,
        tick: world.tick(),
        tiles: world
            .drain_dirty()
            .into_iter()
            .map(|(pos, tile_value)| protocol::TileChange {
                pos: [pos.x, pos.y, pos.z],
                tile: tile(tile_value),
            })
            .collect(),
        entities: world
            .dwarves()
            .into_iter()
            .map(|(id, pos)| protocol::Entity {
                id: id.0,
                kind: protocol::EntityKind::Dwarf,
                pos: [pos.x, pos.y, pos.z],
            })
            .collect(),
        designations: Vec::new(),
        zones: Vec::new(),
        speed: protocol::Speed::Normal,
    }
}

fn tile(tile: sim_core::Tile) -> protocol::Tile {
    match tile {
        sim_core::Tile::Empty => protocol::Tile::Empty,
        sim_core::Tile::Solid(material_value) => protocol::Tile::Solid(material(material_value)),
        sim_core::Tile::Ramp(material_value) => protocol::Tile::Ramp(material(material_value)),
    }
}

fn material(material: sim_core::Material) -> protocol::Material {
    match material {
        sim_core::Material::Stone => protocol::Material::Stone,
        sim_core::Material::Soil => protocol::Material::Soil,
        sim_core::Material::Ice => protocol::Material::Ice,
        sim_core::Material::Snow => protocol::Material::Snow,
    }
}

#[cfg(test)]
mod tests {
    use super::{delta, snapshot};

    /// The wire mapping, restated independently of the code under test.
    ///
    /// Deliberately NOT `super::tile`: asserting `snap.tiles[i] == tile(world.tile(p))`
    /// runs both sides through the function being tested, so it proves ordering but
    /// never the mapping — swapping two variants stays green. This table is the oracle.
    fn expected_tile(value: sim_core::Tile) -> protocol::Tile {
        match value {
            sim_core::Tile::Empty => protocol::Tile::Empty,
            sim_core::Tile::Solid(m) => protocol::Tile::Solid(expected_material(m)),
            sim_core::Tile::Ramp(m) => protocol::Tile::Ramp(expected_material(m)),
        }
    }

    fn expected_material(value: sim_core::Material) -> protocol::Material {
        match value {
            sim_core::Material::Stone => protocol::Material::Stone,
            sim_core::Material::Soil => protocol::Material::Soil,
            sim_core::Material::Ice => protocol::Material::Ice,
            sim_core::Material::Snow => protocol::Material::Snow,
        }
    }

    #[test]
    fn every_tile_maps_to_its_named_wire_variant() {
        let world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let snap = snapshot(&world);

        assert_eq!(snap.tiles.len(), world.tiles().len());
        for (index, world_tile) in world.tiles().iter().enumerate() {
            assert_eq!(
                snap.tiles[index],
                expected_tile(*world_tile),
                "tile {index} ({world_tile:?}) crossed the bridge as the wrong variant"
            );
        }
    }

    #[test]
    fn snapshot_mirrors_world_grid() {
        let world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let snap = snapshot(&world);
        let mut empty = None;
        let mut solid = None;
        let mut ramp = None;

        for (index, world_tile) in world.tiles().iter().enumerate() {
            match world_tile {
                sim_core::Tile::Empty if empty.is_none() => empty = Some(index),
                sim_core::Tile::Solid(_) if solid.is_none() => solid = Some(index),
                sim_core::Tile::Ramp(_) if ramp.is_none() => ramp = Some(index),
                _ => {}
            }
            if empty.is_some() && solid.is_some() && ramp.is_some() {
                break;
            }
        }

        let probes = [
            empty.expect("generated world must contain an empty tile"),
            solid.expect("generated world must contain a solid tile"),
            ramp.expect("generated world must contain a ramp tile"),
        ];
        let dims = world.dims();
        let width = dims.x as usize;
        let layer_size = width * dims.y as usize;

        for index in probes {
            let z = index / layer_size;
            let within_layer = index % layer_size;
            let y = within_layer / width;
            let x = within_layer % width;
            let pos = sim_core::Pos {
                x: x as i32,
                y: y as i32,
                z: z as i32,
            };

            assert_eq!(snap.tiles[index], expected_tile(world.tile(pos).unwrap()));
        }

        // Dims::DEFAULT is square and the probes above cluster at low x/y, so a
        // transposed index formula would survive them. These have x != y.
        for (x, y, z) in [(1usize, 2usize, 0usize), (5, 9, 3), (17, 64, 20)] {
            let index = x + y * width + z * layer_size;
            let pos = sim_core::Pos {
                x: x as i32,
                y: y as i32,
                z: z as i32,
            };

            assert_eq!(
                snap.tiles[index],
                expected_tile(world.tile(pos).unwrap()),
                "row-major index disagrees with world.tile at ({x}, {y}, {z})"
            );
        }
    }

    #[test]
    fn entities_mirror_dwarves() {
        let world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let snap = snapshot(&world);
        let dwarves = world.dwarves();

        assert_eq!(snap.entities.len(), 5);
        assert_eq!(
            snap.entities
                .iter()
                .map(|entity| entity.id)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
        assert_eq!(
            snap.entities
                .iter()
                .map(|entity| entity.pos)
                .collect::<Vec<_>>(),
            dwarves
                .iter()
                .map(|(_, pos)| [pos.x, pos.y, pos.z])
                .collect::<Vec<_>>()
        );
        assert!(
            snap.entities
                .iter()
                .all(|entity| entity.kind == protocol::EntityKind::Dwarf)
        );
    }

    #[test]
    fn snapshot_json_obeys_wire_conventions() {
        let world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let value = serde_json::to_value(snapshot(&world)).unwrap();

        assert_eq!(value["type"], "snapshot");
        assert_eq!(value["speed"], "normal");
        assert_eq!(value["tick"], 0);
        assert_eq!(value["designations"], serde_json::json!([]));
        assert_eq!(value["zones"], serde_json::json!([]));
        assert_eq!(
            value["dims"],
            serde_json::json!({"x": 128, "y": 128, "z": 32})
        );
        assert_eq!(value["entities"][0]["kind"], "dwarf");

        let pos = value["entities"][0]["pos"]
            .as_array()
            .expect("entity position must be an array");
        assert_eq!(pos.len(), 3);
        assert!(pos.iter().all(serde_json::Value::is_number));

        let tiles = value["tiles"].as_array().expect("tiles must be an array");
        assert!(tiles.iter().any(|tile| tile == "empty"));

        // Every solid/ramp payload must be one of the four named materials — "some
        // lowercase string" would accept a renamed or swapped variant.
        let named = ["stone", "soil", "ice", "snow"];
        let mut saw_solid = false;
        for tile in tiles {
            for key in ["solid", "ramp"] {
                if let Some(material) = tile.get(key) {
                    let material = material
                        .as_str()
                        .unwrap_or_else(|| panic!("{key} payload must be a string"));
                    assert!(
                        named.contains(&material),
                        "{key} carried unknown material {material:?}"
                    );
                    saw_solid |= key == "solid";
                }
            }
        }
        assert!(saw_solid, "world must contain at least one solid tile");
    }

    #[test]
    fn snapshot_uses_the_current_world_tick() {
        let mut world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        world.step();
        world.step();

        assert_eq!(snapshot(&world).tick, 2);
    }

    #[test]
    fn delta_carries_dirty_tiles_and_full_authoritative_state() {
        let mut world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let pos = sim_core::Pos { x: 1, y: 2, z: 3 };
        assert!(world.set_tile(pos, sim_core::Tile::Solid(sim_core::Material::Ice)));
        world.step();

        let update = delta(&mut world);
        assert_eq!(update.msg_type, protocol::MessageType::Delta);
        assert_eq!(update.tick, 1);
        assert_eq!(
            update.tiles,
            vec![protocol::TileChange {
                pos: [1, 2, 3],
                tile: protocol::Tile::Solid(protocol::Material::Ice),
            }]
        );
        assert_eq!(update.entities.len(), 5);
        assert!(update.designations.is_empty());
        assert!(update.zones.is_empty());
        assert_eq!(update.speed, protocol::Speed::Normal);

        world.step();
        assert!(delta(&mut world).tiles.is_empty());
    }
}
