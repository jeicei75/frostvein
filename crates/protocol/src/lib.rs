#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 7373;

/// Wire message discriminator. `Delta` joins in Story 2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Snapshot,
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
pub enum Speed {
    Paused,
    Normal,
    Fast,
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
    // NOTE: designation and zone shapes land in Story 3.1; `Vec<()>` keeps the
    // wire fields present and always empty without inventing their shape now.
    // NOTE: `Vec<()>` rejects `[1,2]` but DOES accept `[null,null]` as a length-2
    // vec — `()` deserializes from JSON null. It is not an "empty array only"
    // guarantee; Story 3.1 replaces these with real shapes anyway.
    pub designations: Vec<()>,
    pub zones: Vec<()>,
    pub speed: Speed,
    pub tick: u64,
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
        "entities": [{"id": 7, "kind": "dwarf", "pos": [4, 5, 6]}],
        "designations": [],
        "zones": [],
        "speed": "normal",
        "tick": 9
    }"#;

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
            }]
        );
        assert!(snapshot.designations.is_empty());
        assert!(snapshot.zones.is_empty());
        assert_eq!(snapshot.speed, Speed::Normal);
        assert_eq!(snapshot.tick, 9);
    }

    #[test]
    fn re_encodes_to_the_documented_wire_format() {
        let expected: serde_json::Value = serde_json::from_str(WIRE).unwrap();

        assert_eq!(serde_json::to_value(decoded()).unwrap(), expected);
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
        assert_eq!(
            serde_json::to_string(&EntityKind::Dwarf).unwrap(),
            "\"dwarf\""
        );
    }
}
