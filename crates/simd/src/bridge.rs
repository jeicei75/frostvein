pub fn snapshot(world: &sim_core::World) -> protocol::Snapshot {
    let dims = world.dims();
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
    use super::{snapshot, tile};

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

            assert_eq!(snap.tiles[index], tile(world.tile(pos).unwrap()));
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
        let solid_material = tiles
            .iter()
            .find_map(|tile| tile.get("solid").and_then(serde_json::Value::as_str))
            .expect("tiles must contain a solid material");
        assert!(!solid_material.is_empty());
        assert_eq!(solid_material, solid_material.to_lowercase());
    }

    #[test]
    fn snapshot_reads_tick_from_world() {
        let world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);

        assert_eq!(snapshot(&world).tick, world.tick());
    }
}
