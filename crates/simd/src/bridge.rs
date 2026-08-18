pub fn snapshot(world: &sim_core::World, speed: protocol::Speed) -> protocol::Snapshot {
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
            .map(|(id, pos, state, light)| protocol::Entity {
                id: id.0,
                kind: protocol::EntityKind::Dwarf,
                pos: [pos.x, pos.y, pos.z],
                state: job_state(state),
                light: Some(light_kind(light)),
            })
            .chain(world.emitters().into_iter().map(emitter_entity))
            .collect(),
        designations: world
            .designations()
            .into_iter()
            .map(|(pos, kind)| protocol::Designation {
                pos: pos_out(pos),
                kind: designation_kind_out(kind),
            })
            .collect(),
        zones: world
            .zones()
            .into_iter()
            .map(|pos| protocol::Zone { pos: pos_out(pos) })
            .collect(),
        items: world
            .items()
            .into_iter()
            .map(|(id, pos)| protocol::Item {
                id: id.0,
                pos: pos_out(pos),
            })
            .collect(),
        speed,
        tick: world.tick(),
    }
}

/// NOTE: destructive and NOT idempotent — it drains the dirty set as a side effect of
/// encoding. Calling it twice for the same tick silently yields an empty `tiles` the
/// second time and loses those changes for good. Safe today because the tick loop
/// encodes once and shares the resulting `Arc<String>`; Story 2.2 is the first to have
/// a real producer of dirty tiles, so keep the single-call discipline.
pub fn delta(world: &mut sim_core::World, speed: protocol::Speed) -> protocol::Delta {
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
            .map(|(id, pos, state, light)| protocol::Entity {
                id: id.0,
                kind: protocol::EntityKind::Dwarf,
                pos: [pos.x, pos.y, pos.z],
                state: job_state(state),
                light: Some(light_kind(light)),
            })
            .chain(world.emitters().into_iter().map(emitter_entity))
            .collect(),
        // NOTE: AD-8 full-resends every mark in every delta, but sim-core bounds the
        // authoritative set at MAX_DESIGNATIONS, so amplification is finite.
        designations: world
            .designations()
            .into_iter()
            .map(|(pos, kind)| protocol::Designation {
                pos: pos_out(pos),
                kind: designation_kind_out(kind),
            })
            .collect(),
        zones: world
            .zones()
            .into_iter()
            .map(|pos| protocol::Zone { pos: pos_out(pos) })
            .collect(),
        items: world
            .items()
            .into_iter()
            .map(|(id, pos)| protocol::Item {
                id: id.0,
                pos: pos_out(pos),
            })
            .collect(),
        speed,
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
        sim_core::Material::TreeTrunk => protocol::Material::TreeTrunk,
        sim_core::Material::TreeFoliage => protocol::Material::TreeFoliage,
    }
}

fn emitter_entity(
    (id, pos, light): (sim_core::Id, sim_core::Pos, sim_core::LightKind),
) -> protocol::Entity {
    protocol::Entity {
        id: id.0,
        kind: entity_kind(light),
        pos: [pos.x, pos.y, pos.z],
        state: protocol::JobState::Idle,
        light: Some(light_kind(light)),
    }
}

fn entity_kind(light: sim_core::LightKind) -> protocol::EntityKind {
    match light {
        sim_core::LightKind::Torch => protocol::EntityKind::Torch,
        sim_core::LightKind::Campfire => protocol::EntityKind::Campfire,
        sim_core::LightKind::Lantern => unreachable!("lanterns are not live emitters"),
    }
}

fn light_kind(light: sim_core::LightKind) -> protocol::LightKind {
    match light {
        sim_core::LightKind::Torch => protocol::LightKind::Torch,
        sim_core::LightKind::Campfire => protocol::LightKind::Campfire,
        sim_core::LightKind::Lantern => protocol::LightKind::Lantern,
    }
}

fn job_state(state: sim_core::JobState) -> protocol::JobState {
    match state {
        sim_core::JobState::Idle => protocol::JobState::Idle,
        sim_core::JobState::Walk => protocol::JobState::Walk,
        sim_core::JobState::Work => protocol::JobState::Work,
    }
}

pub(crate) fn designation_kind_in(kind: protocol::DesignationKind) -> sim_core::DesignationKind {
    match kind {
        protocol::DesignationKind::Dig => sim_core::DesignationKind::Dig,
        protocol::DesignationKind::Channel => sim_core::DesignationKind::Channel,
    }
}

fn designation_kind_out(kind: sim_core::DesignationKind) -> protocol::DesignationKind {
    match kind {
        sim_core::DesignationKind::Dig => protocol::DesignationKind::Dig,
        sim_core::DesignationKind::Channel => protocol::DesignationKind::Channel,
    }
}

pub(crate) fn rect_in(rect: protocol::Rect) -> sim_core::Rect {
    sim_core::Rect {
        min: pos_in(rect.min),
        max: pos_in(rect.max),
    }
}

fn pos_in(pos: [i32; 3]) -> sim_core::Pos {
    sim_core::Pos {
        x: pos[0],
        y: pos[1],
        z: pos[2],
    }
}

fn pos_out(pos: sim_core::Pos) -> [i32; 3] {
    [pos.x, pos.y, pos.z]
}

#[cfg(test)]
mod tests {
    use super::{delta, designation_kind_in, snapshot};

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
            sim_core::Material::TreeTrunk => protocol::Material::TreeTrunk,
            sim_core::Material::TreeFoliage => protocol::Material::TreeFoliage,
        }
    }

    /// AD-6's independent oracle. It restates each sim state's hand-written wire NAME and
    /// decodes that, rather than repeating the production match — a second copy of the same
    /// arms would pass even when the author got a mapping wrong and copied it.
    fn expected_job_state(value: sim_core::JobState) -> protocol::JobState {
        let wire = match value {
            sim_core::JobState::Idle => r#""idle""#,
            sim_core::JobState::Walk => r#""walk""#,
            sim_core::JobState::Work => r#""work""#,
        };
        serde_json::from_str(wire).expect("hand-written wire name must decode")
    }

    #[test]
    fn every_job_state_maps_to_its_named_wire_variant() {
        for (value, wire) in [
            (sim_core::JobState::Idle, r#""idle""#),
            (sim_core::JobState::Walk, r#""walk""#),
            (sim_core::JobState::Work, r#""work""#),
        ] {
            assert_eq!(
                serde_json::to_string(&super::job_state(value)).unwrap(),
                wire,
                "{value:?} crossed the bridge as the wrong wire state"
            );
        }
    }

    #[test]
    fn every_designation_kind_maps_to_its_named_wire_variant() {
        for (value, wire) in [
            (sim_core::DesignationKind::Dig, r#""dig""#),
            (sim_core::DesignationKind::Channel, r#""channel""#),
        ] {
            assert_eq!(
                serde_json::to_string(&super::designation_kind_out(value)).unwrap(),
                wire
            );
            let protocol_kind: protocol::DesignationKind =
                serde_json::from_str(wire).expect("hand-written wire name must decode");
            assert_eq!(designation_kind_in(protocol_kind), value);
        }
    }

    #[test]
    fn every_tile_maps_to_its_named_wire_variant() {
        let world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let snap = snapshot(&world, protocol::Speed::Normal);

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
        let snap = snapshot(&world, protocol::Speed::Normal);
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
        let mut world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        world.step();
        let snap = snapshot(&world, protocol::Speed::Normal);
        let dwarves = world.dwarves();

        assert_eq!(snap.entities.len(), 10);
        let dwarf_entities = &snap.entities[..dwarves.len()];
        assert_eq!(
            dwarf_entities
                .iter()
                .map(|entity| entity.id)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
        assert_eq!(
            dwarf_entities
                .iter()
                .map(|entity| entity.pos)
                .collect::<Vec<_>>(),
            dwarves
                .iter()
                .map(|(_, pos, _, _)| [pos.x, pos.y, pos.z])
                .collect::<Vec<_>>()
        );
        assert!(
            dwarf_entities
                .iter()
                .all(|entity| entity.kind == protocol::EntityKind::Dwarf)
        );
        assert_eq!(
            dwarf_entities
                .iter()
                .map(|entity| entity.state)
                .collect::<Vec<_>>(),
            dwarves
                .iter()
                .map(|(_, _, state, _)| expected_job_state(*state))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn every_dwarf_carries_a_lantern_in_snapshot_and_delta_without_duplication() {
        let mut world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let snapshot = snapshot(&world, protocol::Speed::Normal);
        let delta = delta(&mut world, protocol::Speed::Normal);

        for (name, entities) in [("snapshot", snapshot.entities), ("delta", delta.entities)] {
            let dwarves: Vec<_> = entities
                .iter()
                .filter(|entity| entity.kind == protocol::EntityKind::Dwarf)
                .collect();
            assert_eq!(dwarves.len(), 5, "{name} duplicated or lost a dwarf");
            assert!(
                dwarves
                    .iter()
                    .all(|entity| entity.light == Some(protocol::LightKind::Lantern)),
                "every {name} dwarf must carry the lantern wire value"
            );
        }
    }

    #[test]
    fn dwarf_lanterns_never_enter_the_static_emitter_path() {
        let world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);

        assert_eq!(
            world.emitters().len(),
            5,
            "the camp has only static emitters"
        );
        assert!(world.emitters().iter().all(|(_, _, light)| {
            matches!(
                light,
                sim_core::LightKind::Torch | sim_core::LightKind::Campfire
            )
        }));

        let snapshot = snapshot(&world, protocol::Speed::Normal);
        assert_eq!(
            snapshot.entities.len(),
            10,
            "dwarves must not be emitted twice"
        );
        assert!(snapshot.entities[..5].iter().all(|entity| {
            entity.kind == protocol::EntityKind::Dwarf
                && entity.light == Some(protocol::LightKind::Lantern)
        }));
    }

    #[test]
    #[should_panic(expected = "lanterns are not live emitters")]
    fn static_lantern_emitters_remain_rejected_by_the_bridge_guard() {
        super::emitter_entity((
            sim_core::Id(99),
            sim_core::Pos { x: 1, y: 2, z: 3 },
            sim_core::LightKind::Lantern,
        ));
    }

    #[test]
    fn snapshot_and_delta_carry_the_same_emitters() {
        let mut world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let expected: Vec<_> = world
            .emitters()
            .into_iter()
            .map(|(id, pos, light)| protocol::Entity {
                id: id.0,
                kind: match light {
                    sim_core::LightKind::Torch => protocol::EntityKind::Torch,
                    sim_core::LightKind::Campfire => protocol::EntityKind::Campfire,
                    sim_core::LightKind::Lantern => unreachable!("lanterns are not spawned"),
                },
                pos: [pos.x, pos.y, pos.z],
                state: protocol::JobState::Idle,
                light: Some(match light {
                    sim_core::LightKind::Torch => protocol::LightKind::Torch,
                    sim_core::LightKind::Campfire => protocol::LightKind::Campfire,
                    sim_core::LightKind::Lantern => unreachable!("lanterns are not spawned"),
                }),
            })
            .collect();

        let snap = snapshot(&world, protocol::Speed::Normal);
        let update = delta(&mut world, protocol::Speed::Normal);
        assert_eq!(&snap.entities[5..], expected);
        assert_eq!(&update.entities[5..], expected);
        assert!(
            snap.entities[..5]
                .iter()
                .all(|entity| entity.light == Some(protocol::LightKind::Lantern))
        );
        assert!(
            update.entities[..5]
                .iter()
                .all(|entity| entity.light == Some(protocol::LightKind::Lantern))
        );
    }

    #[test]
    fn snapshot_json_obeys_wire_conventions() {
        let world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let value = serde_json::to_value(snapshot(&world, protocol::Speed::Normal)).unwrap();

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

        // Every solid/ramp payload must be one of the six named materials — "some
        // lowercase string" would accept a renamed or swapped variant.
        let named = ["stone", "soil", "ice", "snow", "tree_trunk", "tree_foliage"];
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

        assert_eq!(snapshot(&world, protocol::Speed::Normal).tick, 2);
    }

    #[test]
    fn delta_carries_dirty_tiles_and_full_authoritative_state() {
        let mut world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let pos = sim_core::Pos { x: 1, y: 2, z: 3 };
        assert!(world.set_tile(pos, sim_core::Tile::Solid(sim_core::Material::Ice)));
        world.step();

        let update = delta(&mut world, protocol::Speed::Fast);
        assert_eq!(update.msg_type, protocol::MessageType::Delta);
        assert_eq!(update.tick, 1);
        assert_eq!(
            update.tiles,
            vec![protocol::TileChange {
                pos: [1, 2, 3],
                tile: protocol::Tile::Solid(protocol::Material::Ice),
            }]
        );
        assert_eq!(update.entities.len(), 10);
        assert_eq!(
            update.entities[..world.dwarves().len()]
                .iter()
                .map(|entity| entity.state)
                .collect::<Vec<_>>(),
            world
                .dwarves()
                .iter()
                .map(|(_, _, state, _)| expected_job_state(*state))
                .collect::<Vec<_>>()
        );
        assert!(update.designations.is_empty());
        assert!(update.zones.is_empty());
        assert_eq!(update.speed, protocol::Speed::Fast);

        world.step();
        assert!(delta(&mut world, protocol::Speed::Fast).tiles.is_empty());
    }

    #[test]
    fn snapshot_and_delta_carry_the_worlds_real_marks() {
        let mut world = sim_core::World::generate(42, sim_core::Dims::DEFAULT);
        let zone_pos = world.dwarves()[0].1;
        let designation_pos = zone_pos;
        world.apply_command(sim_core::SimCommand::Designate {
            kind: sim_core::DesignationKind::Channel,
            rect: sim_core::Rect {
                min: designation_pos,
                max: designation_pos,
            },
        });
        world.apply_command(sim_core::SimCommand::PlaceStockpile {
            rect: sim_core::Rect {
                min: zone_pos,
                max: zone_pos,
            },
        });
        let expected_designations = vec![protocol::Designation {
            pos: [designation_pos.x, designation_pos.y, designation_pos.z],
            kind: protocol::DesignationKind::Channel,
        }];
        let expected_zones = vec![protocol::Zone {
            pos: [zone_pos.x, zone_pos.y, zone_pos.z],
        }];

        let snap = snapshot(&world, protocol::Speed::Normal);
        assert_eq!(snap.designations, expected_designations);
        assert_eq!(snap.zones, expected_zones);

        let update = delta(&mut world, protocol::Speed::Normal);
        assert_eq!(update.designations, expected_designations);
        assert_eq!(update.zones, expected_zones);
    }
}
