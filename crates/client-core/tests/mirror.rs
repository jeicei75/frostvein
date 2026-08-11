use client_core::Mirror;
use protocol::{Delta, EntityKind, Snapshot, Speed, Tile};

const SNAPSHOT_WIRE: &str = r#"{
    "type":"snapshot", "dims":{"x":2,"y":1,"z":1},
    "tiles":["empty",{"solid":"ice"}],
    "entities":[
        {"id":7,"kind":"dwarf","pos":[0,0,0],"state":"idle","light":null},
        {"id":2,"kind":"torch","pos":[1,0,0],"state":"idle","light":"torch"}
    ],
    "designations":[{"pos":[0,0,0],"kind":"dig"}], "zones":[], "items":[],
    "speed":"normal", "tick":9
}"#;

const FIRST_DELTA_WIRE: &str = r#"{
    "type":"delta", "tick":10,
    "tiles":[{"pos":[1,0,0],"tile":{"solid":"stone"}}],
    "entities":[
        {"id":7,"kind":"dwarf","pos":[1,0,0],"state":"walk","light":null},
        {"id":2,"kind":"torch","pos":[1,0,0],"state":"idle","light":"torch"}
    ],
    "designations":[], "zones":[{"pos":[1,0,0]}],
    "items":[{"id":9,"pos":[0,0,0]}], "speed":"fast"
}"#;

const SECOND_DELTA_WIRE: &str = r#"{
    "type":"delta", "tick":11, "tiles":[],
    "entities":[{"id":7,"kind":"dwarf","pos":[1,0,0],"state":"walk","light":null}],
    "designations":[], "zones":[], "items":[], "speed":"fast"
}"#;

const THIRD_DELTA_WIRE: &str = r#"{
    "type":"delta", "tick":12,
    "tiles":[{"pos":[0,0,0],"tile":{"ramp":"snow"}}],
    "entities":[{"id":7,"kind":"dwarf","pos":[1,0,0],"state":"walk","light":null}],
    "designations":[], "zones":[], "items":[], "speed":"fast"
}"#;

const RESET_SNAPSHOT_WIRE: &str = r#"{
    "type":"snapshot", "dims":{"x":1,"y":1,"z":1}, "tiles":["empty"],
    "entities":[{"id":99,"kind":"campfire","pos":[0,0,0],"state":"idle","light":"campfire"}],
    "designations":[], "zones":[], "items":[], "speed":"paused", "tick":20
}"#;

#[test]
fn recorded_wire_messages_build_the_expected_mirror() {
    let snapshot: Snapshot = serde_json::from_str(SNAPSHOT_WIRE).unwrap();
    let mut mirror = Mirror::from_snapshot(snapshot).unwrap();
    assert_eq!(mirror.dims().x, 2);
    assert_eq!(mirror.tick(), 9);
    assert_eq!(
        mirror.tile([1, 0, 0]),
        Some(Tile::Solid(protocol::Material::Ice))
    );
    assert_eq!(
        mirror
            .entities()
            .map(|entity| entity.id)
            .collect::<Vec<_>>(),
        vec![2, 7]
    );
    assert_eq!(mirror.entities().next().unwrap().kind, EntityKind::Torch);
    assert_eq!(mirror.designations().len(), 1);
    assert!(mirror.zones().is_empty());
    assert!(mirror.items().next().is_none());
    assert!(mirror.previous_entity(7).is_none());
    assert!(mirror.changes().tiles.is_empty());

    mirror.apply_delta(serde_json::from_str::<Delta>(FIRST_DELTA_WIRE).unwrap());
    assert_eq!(mirror.tick(), 10);
    assert_eq!(mirror.speed(), Speed::Fast);
    assert_eq!(
        mirror.tile([1, 0, 0]),
        Some(Tile::Solid(protocol::Material::Stone))
    );
    assert_eq!(mirror.changes().tiles, vec![[1, 0, 0]]);
    assert_eq!(mirror.changes().changed, vec![7]);
    assert!(mirror.changes().spawned.is_empty());
    assert!(mirror.changes().despawned.is_empty());
    assert_eq!(mirror.previous_entity(7).unwrap().pos, [0, 0, 0]);
    assert_eq!(mirror.previous_entity(2).unwrap().kind, EntityKind::Torch);
    assert_eq!(
        mirror.items().map(|item| item.id).collect::<Vec<_>>(),
        vec![9]
    );
    assert_eq!(mirror.zones()[0].pos, [1, 0, 0]);

    mirror.apply_delta(serde_json::from_str::<Delta>(SECOND_DELTA_WIRE).unwrap());
    assert_eq!(mirror.tick(), 11);
    assert_eq!(
        mirror
            .entities()
            .map(|entity| entity.id)
            .collect::<Vec<_>>(),
        vec![7]
    );
    assert_eq!(mirror.changes().despawned, vec![2]);
    assert!(mirror.changes().spawned.is_empty());
    assert!(mirror.changes().changed.is_empty());
    assert!(mirror.changes().despawned.iter().all(|id| *id != 7));
    assert!(mirror.previous_entity(2).is_none());
    assert_eq!(mirror.previous_entity(7).unwrap().pos, [1, 0, 0]);
    assert!(mirror.items().next().is_none());

    mirror.apply_delta(serde_json::from_str::<Delta>(THIRD_DELTA_WIRE).unwrap());
    assert_eq!(mirror.changes().tiles, vec![[0, 0, 0]]);
    assert!(mirror.changes().spawned.is_empty());
    assert!(mirror.changes().despawned.is_empty());
    assert!(mirror.changes().changed.is_empty());
    assert_eq!(
        mirror.tile([0, 0, 0]),
        Some(Tile::Ramp(protocol::Material::Snow))
    );
    assert_eq!(
        mirror.previous_entity(7).unwrap().state,
        protocol::JobState::Walk
    );

    mirror
        .apply_snapshot(serde_json::from_str::<Snapshot>(RESET_SNAPSHOT_WIRE).unwrap())
        .unwrap();
    assert_eq!(mirror.dims(), protocol::Dims { x: 1, y: 1, z: 1 });
    assert_eq!(mirror.tick(), 20);
    assert_eq!(mirror.speed(), Speed::Paused);
    assert_eq!(mirror.tile([0, 0, 0]), Some(Tile::Empty));
    assert_eq!(
        mirror
            .entities()
            .map(|entity| entity.id)
            .collect::<Vec<_>>(),
        vec![99]
    );
    assert_eq!(mirror.entities().next().unwrap().kind, EntityKind::Campfire);
    assert!(mirror.designations().is_empty());
    assert!(mirror.zones().is_empty());
    assert!(mirror.items().next().is_none());
    assert!(mirror.previous_entity(7).is_none());
    assert!(mirror.previous_entity(99).is_none());
    assert!(mirror.changes().tiles.is_empty());
    assert!(mirror.changes().spawned.is_empty());
    assert!(mirror.changes().despawned.is_empty());
    assert!(mirror.changes().changed.is_empty());
}
