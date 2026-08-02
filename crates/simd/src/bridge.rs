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
    use super::snapshot;

    #[test]
    fn snapshot_reads_tick_from_world() {
        let world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);

        assert_eq!(snapshot(&world).tick, world.tick());
    }
}
