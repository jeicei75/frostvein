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
    pub designations: Vec<()>,
    pub zones: Vec<()>,
    pub speed: Speed,
    pub tick: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, de::DeserializeOwned};

    fn assert_wire_type<T: Serialize + DeserializeOwned>() {}

    #[test]
    fn protocol_shapes_are_serde_types() {
        assert_wire_type::<MessageType>();
        assert_wire_type::<Material>();
        assert_wire_type::<Tile>();
        assert_wire_type::<EntityKind>();
        assert_wire_type::<Speed>();
        assert_wire_type::<Dims>();
        assert_wire_type::<Entity>();
        assert_wire_type::<Snapshot>();

        let snapshot = Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 1, y: 2, z: 3 },
            tiles: vec![Tile::Empty, Tile::Solid(Material::Stone)],
            entities: vec![Entity {
                id: 7,
                kind: EntityKind::Dwarf,
                pos: [4, 5, 6],
            }],
            designations: Vec::new(),
            zones: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        };

        assert_eq!(snapshot.tiles.len(), 2);
    }
}
