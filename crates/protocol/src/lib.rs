#![forbid(unsafe_code)]

//! Protocol v0 wire types: [`Snapshot`], [`Delta`], and [`Command`].

use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 7373;

/// Wire message discriminator. `Delta` joins in Story 2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Snapshot,
    Delta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Material {
    Stone,
    Soil,
    Ice,
    Snow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tile {
    Empty,
    Solid(Material),
    Ramp(Material),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Dwarf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Idle,
    Walk,
    Work,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Speed {
    Paused,
    Normal,
    Fast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesignationKind {
    Dig,
    Channel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    SetSpeed { speed: Speed },
    Save,
    Load,
    Quit,
    Designate { kind: DesignationKind, rect: Rect },
    CancelDesignation { rect: Rect },
    PlaceStockpile { rect: Rect },
    RemoveStockpile { rect: Rect },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dims {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub id: u32,
    pub kind: EntityKind,
    pub pos: [i32; 3],
    pub state: JobState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileChange {
    pub pos: [i32; 3],
    pub tile: Tile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Designation {
    pub pos: [i32; 3],
    pub kind: DesignationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    pub pos: [i32; 3],
}

/// Full world state, sent on connect (AD-3). Field order is wire order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub dims: Dims,
    /// Flat row-major: index = x + y*dims.x + z*dims.x*dims.y
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
    pub designations: Vec<Designation>,
    pub zones: Vec<Zone>,
    pub speed: Speed,
    pub tick: u64,
}

/// One per loop iteration (AD-8). `tiles` is the dirty set; everything else is a
/// full authoritative resend — absence is deletion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delta {
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub tick: u64,
    pub tiles: Vec<TileChange>,
    pub entities: Vec<Entity>,
    pub designations: Vec<Designation>,
    pub zones: Vec<Zone>,
    pub speed: Speed,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wire format as an external client sees it, written out by hand.
    ///
    /// This is the point of the test: encoding and decoding with our own types
    /// agrees with itself even if every name changes together, so a symmetric
    /// rename (dropping `rename = "type"`, or `snake_case`) would pass a
    /// round-trip test while breaking every client. A literal cannot.
    const WIRE: &str = r#"{
        "type": "snapshot",
        "dims": {"x": 2, "y": 1, "z": 1},
        "tiles": ["empty", {"solid": "stone"}],
        "entities": [{"id": 7, "kind": "dwarf", "pos": [4, 5, 6], "state": "idle"}],
        "designations": [{"pos": [1, 2, 3], "kind": "dig"}],
        "zones": [{"pos": [1, 2, 4]}],
        "speed": "normal",
        "tick": 9
    }"#;

    const DELTA_WIRE: &str = r#"{
        "type": "delta",
        "tick": 10,
        "tiles": [{"pos": [1, 2, 3], "tile": {"solid": "ice"}}],
        "entities": [{"id": 7, "kind": "dwarf", "pos": [4, 5, 6], "state": "walk"}],
        "designations": [{"pos": [1, 2, 3], "kind": "dig"}],
        "zones": [{"pos": [1, 2, 4]}],
        "speed": "fast"
    }"#;

    const COMMAND_WIRE: &str = r#"{"type":"set_speed","speed":"paused"}"#;

    fn decoded() -> Snapshot {
        serde_json::from_str(WIRE).expect("the documented wire format must decode")
    }

    #[test]
    fn decodes_the_documented_wire_format() {
        let snapshot = decoded();

        assert_eq!(snapshot.msg_type, MessageType::Snapshot);
        assert_eq!(snapshot.dims, Dims { x: 2, y: 1, z: 1 });
        assert_eq!(
            snapshot.tiles,
            vec![Tile::Empty, Tile::Solid(Material::Stone)]
        );
        assert_eq!(
            snapshot.entities,
            vec![Entity {
                id: 7,
                kind: EntityKind::Dwarf,
                pos: [4, 5, 6],
                state: JobState::Idle,
            }]
        );
        assert_eq!(
            serde_json::to_string(&snapshot.entities[0]).unwrap(),
            r#"{"id":7,"kind":"dwarf","pos":[4,5,6],"state":"idle"}"#
        );
        assert_eq!(
            snapshot.designations,
            vec![Designation {
                pos: [1, 2, 3],
                kind: DesignationKind::Dig,
            }]
        );
        assert_eq!(snapshot.zones, vec![Zone { pos: [1, 2, 4] }]);
        assert_eq!(snapshot.speed, Speed::Normal);
        assert_eq!(snapshot.tick, 9);
    }

    #[test]
    fn re_encodes_to_the_documented_wire_format() {
        let expected: serde_json::Value = serde_json::from_str(WIRE).unwrap();

        assert_eq!(serde_json::to_value(decoded()).unwrap(), expected);
    }

    #[test]
    fn decodes_the_documented_delta_wire_format() {
        let delta: Delta =
            serde_json::from_str(DELTA_WIRE).expect("the documented delta wire format must decode");

        assert_eq!(delta.msg_type, MessageType::Delta);
        assert_eq!(delta.tick, 10);
        assert_eq!(
            delta.tiles,
            vec![TileChange {
                pos: [1, 2, 3],
                tile: Tile::Solid(Material::Ice),
            }]
        );
        assert_eq!(
            delta.entities,
            vec![Entity {
                id: 7,
                kind: EntityKind::Dwarf,
                pos: [4, 5, 6],
                state: JobState::Walk,
            }]
        );
        assert_eq!(
            delta.designations,
            vec![Designation {
                pos: [1, 2, 3],
                kind: DesignationKind::Dig,
            }]
        );
        assert_eq!(delta.zones, vec![Zone { pos: [1, 2, 4] }]);
        assert_eq!(delta.speed, Speed::Fast);
    }

    #[test]
    fn decodes_and_reencodes_the_documented_command_wire_format() {
        for (wire, expected) in [
            (
                COMMAND_WIRE,
                Command::SetSpeed {
                    speed: Speed::Paused,
                },
            ),
            (r#"{"type":"save"}"#, Command::Save),
            (r#"{"type":"load"}"#, Command::Load),
            (r#"{"type":"quit"}"#, Command::Quit),
            (
                r#"{"type":"designate","kind":"dig","rect":{"min":[1,2,3],"max":[4,5,3]}}"#,
                Command::Designate {
                    kind: DesignationKind::Dig,
                    rect: Rect {
                        min: [1, 2, 3],
                        max: [4, 5, 3],
                    },
                },
            ),
            (
                r#"{"type":"cancel_designation","rect":{"min":[1,2,3],"max":[4,5,3]}}"#,
                Command::CancelDesignation {
                    rect: Rect {
                        min: [1, 2, 3],
                        max: [4, 5, 3],
                    },
                },
            ),
            (
                r#"{"type":"place_stockpile","rect":{"min":[1,2,3],"max":[4,5,3]}}"#,
                Command::PlaceStockpile {
                    rect: Rect {
                        min: [1, 2, 3],
                        max: [4, 5, 3],
                    },
                },
            ),
            (
                r#"{"type":"remove_stockpile","rect":{"min":[1,2,3],"max":[4,5,3]}}"#,
                Command::RemoveStockpile {
                    rect: Rect {
                        min: [1, 2, 3],
                        max: [4, 5, 3],
                    },
                },
            ),
        ] {
            let command: Command =
                serde_json::from_str(wire).expect("the documented command wire format must decode");
            assert_eq!(command, expected);
            assert_eq!(
                serde_json::to_value(command).unwrap(),
                serde_json::from_str::<serde_json::Value>(wire).unwrap()
            );
        }
        assert!(
            serde_json::from_str::<Command>(r#"{"type":"set_rate","speed":"paused"}"#).is_err()
        );
        assert!(serde_json::from_str::<Command>(r#"{"type":"store"}"#).is_err());
        assert!(
            serde_json::from_str::<Command>(
                r#"{"type":"designate","kind":"mine","rect":{"min":[0,0,0],"max":[0,0,0]}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn every_material_and_tile_variant_has_a_pinned_wire_name() {
        for (value, wire) in [
            (Material::Stone, "\"stone\""),
            (Material::Soil, "\"soil\""),
            (Material::Ice, "\"ice\""),
            (Material::Snow, "\"snow\""),
        ] {
            assert_eq!(serde_json::to_string(&value).unwrap(), wire);
        }
        for (value, wire) in [
            (Tile::Empty, "\"empty\""),
            (Tile::Solid(Material::Ice), "{\"solid\":\"ice\"}"),
            (Tile::Ramp(Material::Snow), "{\"ramp\":\"snow\"}"),
        ] {
            assert_eq!(serde_json::to_string(&value).unwrap(), wire);
        }
        for (value, wire) in [
            (Speed::Paused, "\"paused\""),
            (Speed::Normal, "\"normal\""),
            (Speed::Fast, "\"fast\""),
        ] {
            assert_eq!(serde_json::to_string(&value).unwrap(), wire);
        }
        for (value, wire) in [
            (JobState::Idle, "\"idle\""),
            (JobState::Walk, "\"walk\""),
            (JobState::Work, "\"work\""),
        ] {
            assert_eq!(serde_json::to_string(&value).unwrap(), wire);
        }
        assert_eq!(
            serde_json::to_string(&EntityKind::Dwarf).unwrap(),
            "\"dwarf\""
        );
        for (value, wire) in [
            (DesignationKind::Dig, "\"dig\""),
            (DesignationKind::Channel, "\"channel\""),
        ] {
            assert_eq!(serde_json::to_string(&value).unwrap(), wire);
        }
        assert_eq!(
            serde_json::to_value(Command::SetSpeed {
                speed: Speed::Paused
            })
            .unwrap()["type"],
            "set_speed"
        );
        for (value, wire) in [
            (Command::Save, "save"),
            (Command::Load, "load"),
            (Command::Quit, "quit"),
        ] {
            assert_eq!(serde_json::to_value(value).unwrap()["type"], wire);
        }
        for (value, wire) in [
            (
                Command::Designate {
                    kind: DesignationKind::Dig,
                    rect: Rect {
                        min: [0, 0, 0],
                        max: [0, 0, 0],
                    },
                },
                "designate",
            ),
            (
                Command::CancelDesignation {
                    rect: Rect {
                        min: [0, 0, 0],
                        max: [0, 0, 0],
                    },
                },
                "cancel_designation",
            ),
            (
                Command::PlaceStockpile {
                    rect: Rect {
                        min: [0, 0, 0],
                        max: [0, 0, 0],
                    },
                },
                "place_stockpile",
            ),
            (
                Command::RemoveStockpile {
                    rect: Rect {
                        min: [0, 0, 0],
                        max: [0, 0, 0],
                    },
                },
                "remove_stockpile",
            ),
        ] {
            assert_eq!(serde_json::to_value(value).unwrap()["type"], wire);
        }
    }
}
