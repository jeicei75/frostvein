#![forbid(unsafe_code)]

//! Shared client world-mirror state.

use std::collections::BTreeMap;

use protocol::{Delta, Designation, Dims, Entity, Item, Rect, Snapshot, Speed, Tile, Zone};
use thiserror::Error;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Changes {
    pub tiles: Vec<[i32; 3]>,
    pub spawned: Vec<u32>,
    pub despawned: Vec<u32>,
    pub changed: Vec<u32>,
}

#[derive(Debug, Error)]
pub enum MirrorError {
    #[error("snapshot has {actual} tiles but dims {x}x{y}x{z} need {expected}")]
    TileCount {
        actual: usize,
        x: u32,
        y: u32,
        z: u32,
        expected: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mirror {
    dims: Dims,
    tiles: Vec<Tile>,
    entities: BTreeMap<u32, Entity>,
    designations: Vec<Designation>,
    zones: Vec<Zone>,
    items: BTreeMap<u32, Item>,
    speed: Speed,
    tick: u64,
    previous_entities: BTreeMap<u32, Entity>,
    changes: Changes,
}

impl Mirror {
    pub fn from_snapshot(snapshot: Snapshot) -> Result<Self, MirrorError> {
        validate_snapshot(&snapshot)?;
        Ok(Self::replace(snapshot))
    }

    pub fn apply_snapshot(&mut self, snapshot: Snapshot) -> Result<(), MirrorError> {
        validate_snapshot(&snapshot)?;
        *self = Self::replace(snapshot);
        Ok(())
    }

    pub fn apply_delta(&mut self, delta: Delta) {
        let Delta {
            tick,
            tiles,
            entities,
            designations,
            zones,
            items,
            speed,
            ..
        } = delta;
        let mut changes = Changes::default();
        for change in tiles {
            if let Some(index) = self.tile_index(change.pos) {
                self.tiles[index] = change.tile;
                changes.tiles.push(change.pos);
            }
        }

        let next_entities: BTreeMap<_, _> = entities
            .into_iter()
            .map(|entity| (entity.id, entity))
            .collect();
        self.previous_entities.clear();
        for (&id, entity) in &self.entities {
            match next_entities.get(&id) {
                None => changes.despawned.push(id),
                Some(next) if next != entity => {
                    changes.changed.push(id);
                    self.previous_entities.insert(id, *entity);
                }
                Some(_) => {
                    self.previous_entities.insert(id, *entity);
                }
            }
        }
        for &id in next_entities.keys() {
            if !self.entities.contains_key(&id) {
                changes.spawned.push(id);
            }
        }

        self.entities = next_entities;
        self.designations = designations;
        self.zones = zones;
        self.items = items.into_iter().map(|item| (item.id, item)).collect();
        self.speed = speed;
        self.tick = tick;
        self.changes = changes;
    }

    pub fn dims(&self) -> Dims {
        self.dims
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn speed(&self) -> Speed {
        self.speed
    }

    pub fn tile(&self, pos: [i32; 3]) -> Option<Tile> {
        self.tile_index(pos).map(|index| self.tiles[index])
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.items.values()
    }

    pub fn designations(&self) -> &[Designation] {
        &self.designations
    }

    pub fn zones(&self) -> &[Zone] {
        &self.zones
    }

    pub fn previous_entity(&self, id: u32) -> Option<&Entity> {
        self.previous_entities.get(&id)
    }

    pub fn changes(&self) -> &Changes {
        &self.changes
    }

    fn replace(snapshot: Snapshot) -> Self {
        Self {
            dims: snapshot.dims,
            tiles: snapshot.tiles,
            entities: snapshot
                .entities
                .into_iter()
                .map(|entity| (entity.id, entity))
                .collect(),
            designations: snapshot.designations,
            zones: snapshot.zones,
            items: snapshot
                .items
                .into_iter()
                .map(|item| (item.id, item))
                .collect(),
            speed: snapshot.speed,
            tick: snapshot.tick,
            previous_entities: BTreeMap::new(),
            changes: Changes::default(),
        }
    }

    fn tile_index(&self, [x, y, z]: [i32; 3]) -> Option<usize> {
        if x < 0
            || y < 0
            || z < 0
            || x >= self.dims.x as i32
            || y >= self.dims.y as i32
            || z >= self.dims.z as i32
        {
            return None;
        }
        Some(
            x as usize
                + y as usize * self.dims.x as usize
                + z as usize * self.dims.x as usize * self.dims.y as usize,
        )
    }
}

pub fn rect_on_level(a: (i32, i32), b: (i32, i32), z: i32) -> Rect {
    Rect {
        min: [a.0.min(b.0), a.1.min(b.1), z],
        max: [a.0.max(b.0), a.1.max(b.1), z],
    }
}

fn validate_snapshot(snapshot: &Snapshot) -> Result<(), MirrorError> {
    let dims = snapshot.dims;
    let expected = u64::from(dims.x) * u64::from(dims.y) * u64::from(dims.z);
    if snapshot.tiles.len() as u64 != expected {
        return Err(MirrorError::TileCount {
            actual: snapshot.tiles.len(),
            x: dims.x,
            y: dims.y,
            z: dims.z,
            expected,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use protocol::{
        Delta, Dims, Entity, EntityKind, Item, JobState, Material, MessageType, Snapshot, Speed,
        Tile, TileChange,
    };

    use super::*;

    fn snapshot() -> Snapshot {
        Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Empty, Tile::Solid(Material::Ice)],
            entities: vec![Entity {
                id: 7,
                kind: EntityKind::Dwarf,
                pos: [0, 0, 0],
                state: JobState::Idle,
                light: None,
            }],
            designations: Vec::new(),
            zones: Vec::new(),
            items: vec![Item {
                id: 9,
                pos: [0, 0, 0],
            }],
            speed: Speed::Normal,
            tick: 9,
        }
    }

    #[test]
    fn delta_deletes_entities_and_items_absent_from_authoritative_lists() {
        let mut mirror = Mirror::from_snapshot(snapshot()).unwrap();

        mirror.apply_delta(Delta {
            msg_type: MessageType::Delta,
            tick: 10,
            tiles: vec![TileChange {
                pos: [1, 0, 0],
                tile: Tile::Solid(Material::Stone),
            }],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Fast,
        });

        assert_eq!(mirror.tile([1, 0, 0]), Some(Tile::Solid(Material::Stone)));
        assert!(mirror.entities().next().is_none());
        assert!(mirror.items().next().is_none());
        assert_eq!(mirror.tick(), 10);
        assert_eq!(mirror.speed(), Speed::Fast);
    }

    #[test]
    fn snapshot_replaces_everything_and_rejects_inconsistent_tiles() {
        let mut mirror = Mirror::from_snapshot(snapshot()).unwrap();
        mirror.apply_delta(Delta {
            msg_type: MessageType::Delta,
            tick: 10,
            tiles: Vec::new(),
            entities: vec![Entity {
                id: 8,
                kind: EntityKind::Dwarf,
                pos: [1, 0, 0],
                state: JobState::Walk,
                light: None,
            }],
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Fast,
        });

        let replacement = Snapshot {
            tick: 11,
            entities: Vec::new(),
            items: Vec::new(),
            ..snapshot()
        };
        mirror.apply_snapshot(replacement).unwrap();

        assert!(mirror.entities().next().is_none());
        assert!(mirror.items().next().is_none());
        assert_eq!(mirror.tick(), 11);
        let error = Mirror::from_snapshot(Snapshot {
            tiles: vec![Tile::Empty],
            ..snapshot()
        })
        .unwrap_err()
        .to_string();
        assert!(error.contains("1 tiles") && error.contains("2x1x1 need 2"));
    }
}
