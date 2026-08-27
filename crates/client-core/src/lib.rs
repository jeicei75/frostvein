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
        expected: u128,
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

    // NOTE: compared as u32 rather than casting dims to i32 — the dims come from the
    // wire, and an axis at or beyond 2^31 casts to a negative i32, which makes every
    // coordinate compare in-bounds-negative and silently reads back as absent.
    fn tile_index(&self, [x, y, z]: [i32; 3]) -> Option<usize> {
        let x = u32::try_from(x).ok()?;
        let y = u32::try_from(y).ok()?;
        let z = u32::try_from(z).ok()?;
        if x >= self.dims.x || y >= self.dims.y || z >= self.dims.z {
            return None;
        }
        Some(
            x as usize
                + y as usize * self.dims.x as usize
                + z as usize * self.dims.x as usize * self.dims.y as usize,
        )
    }
}

/// Whether the sim will KEEP a designation at `pos`, for the two commands it filters on
/// standability: `Designate { kind: Channel }` and `PlaceStockpile`.
///
/// The sim keeps only standable positions and **drops the rest without a word** — no error, no
/// ack, no diagnostic. Measured 2026-08-27 against the real daemon: a channel rect at a solid
/// cell yields 0 designations, the same rect one level up yields 9; a stockpile rect behaves
/// identically. A client that cannot answer this question has no way to tell "the sim refused
/// every tile" from "the feature is broken", which is exactly how every channel and every
/// stockpile the Bevy client had ever issued stayed inert through a full code review.
///
/// NOTE: this deliberately restates `sim-core`'s `Terrain::is_standable`. Clients must not depend
/// on `sim-core` (the gate probes for that edge), so the rule is duplicated here on purpose and
/// pinned against the real daemon by `crates/simd/tests/designation_targets.rs` — if the sim's
/// rule ever moves, that test goes red rather than this silently disagreeing.
pub fn is_standable(mirror: &Mirror, pos: [i32; 3]) -> bool {
    matches!(mirror.tile(pos), Some(Tile::Empty))
        && matches!(
            mirror.tile([pos[0], pos[1], pos[2] - 1]),
            Some(Tile::Solid(_) | Tile::Ramp(_))
        )
}

/// The standable cell in one column that a drag started at `near_z` should designate.
///
/// A column can hold several standable cells — the surface, and any cave floor beneath it. The
/// one nearest the height the drag began at is the one the boss is looking at, and it keeps a
/// drag that started on a ledge on that ledge instead of jumping to the clifftop above it. Ties
/// go to the higher cell, which is the one you can see.
fn standable_in_column(
    mirror: &Mirror,
    x: i32,
    y: i32,
    near_z: i32,
    level: i32,
) -> Option<[i32; 3]> {
    // `level + 1`, not `level`: the cell you stand on above the cut surface sits ONE ABOVE it, and
    // that is precisely what a top-face pick targets. Capping at `level` excluded it and made
    // every standable drag on a cut level designate nothing.
    (0..=level + 1)
        .map(|z| [x, y, z])
        .filter(|cell| is_standable(mirror, *cell))
        .min_by_key(|cell| ((cell[2] - near_z).abs(), -cell[2]))
}

/// Every cell a standable-target drag designates: one per column of the drag's footprint,
/// FOLLOWING THE GROUND rather than flattening to the anchor's height.
///
/// MEASURED 2026-08-27 on the real world: with AC4's single-z rect, a 6x6 stockpile drag on
/// natural terrain keeps a median 19.4% of its footprint and a 10x10 keeps 14.0% — because a
/// fixed z crosses a hillside in a thin band, and standable cells exist only where the surface IS
/// that height. That is Wolf's "stockpiling does pretty much nothing usually", and it had been
/// true since the AC was written. RULED 2026-08-27 (Wolf): the standable modes follow the
/// surface. Dig keeps the single-z rule, where cutting one level into a slope is the point.
pub fn surface_targets(mirror: &Mirror, level: i32, a: [i32; 3], b: [i32; 3]) -> Vec<[i32; 3]> {
    let mut cells = Vec::new();
    for y in a[1].min(b[1])..=a[1].max(b[1]) {
        for x in a[0].min(b[0])..=a[0].max(b[0]) {
            if let Some(cell) = standable_in_column(mirror, x, y, a[2], level) {
                cells.push(cell);
            }
        }
    }
    cells
}

/// Packs the followed surface into rects, merging each row of same-height neighbours into one.
///
/// NOTE: exact rather than a bounding box — a box would also cover cells this drag did NOT
/// choose, and the sim would silently keep any that happen to be standable, which is a cave floor
/// zoned underground and out of sight. That is the same silent-wrong-cell class this whole round
/// exists to close, so it is not traded away for fewer commands. Merging runs keeps a 10x10 drag
/// near ten commands instead of a hundred, well clear of the 256 bound.
pub fn rects_for_cells(cells: &[[i32; 3]]) -> Vec<Rect> {
    let mut rects: Vec<Rect> = Vec::new();
    for cell in cells {
        match rects.last_mut() {
            Some(last)
                if last.max[1] == cell[1]
                    && last.max[2] == cell[2]
                    && last.max[0] + 1 == cell[0] =>
            {
                last.max[0] = cell[0];
            }
            _ => rects.push(Rect {
                min: *cell,
                max: *cell,
            }),
        }
    }
    rects
}

pub fn rect_on_level(a: (i32, i32), b: (i32, i32), z: i32) -> Rect {
    Rect {
        min: [a.0.min(b.0), a.1.min(b.1), z],
        max: [a.0.max(b.0), a.1.max(b.1), z],
    }
}

fn validate_snapshot(snapshot: &Snapshot) -> Result<(), MirrorError> {
    let dims = snapshot.dims;
    let expected = u128::from(dims.x) * u128::from(dims.y) * u128::from(dims.z);
    if snapshot.tiles.len() as u128 != expected {
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

    #[test]
    fn rejects_dimension_products_beyond_u64_without_panicking() {
        let result = Mirror::from_snapshot(Snapshot {
            dims: Dims {
                x: 2_147_483_648,
                y: 2_147_483_648,
                z: 4,
            },
            tiles: Vec::new(),
            ..snapshot()
        });

        assert!(result.is_err());
    }

    #[test]
    fn changes_partition_entities_and_keep_one_previous_generation() {
        let mut initial = snapshot();
        initial.entities.push(Entity {
            id: 2,
            kind: EntityKind::Dwarf,
            pos: [1, 0, 0],
            state: JobState::Idle,
            light: None,
        });
        initial.entities.push(Entity {
            id: 3,
            kind: EntityKind::Dwarf,
            pos: [0, 0, 0],
            state: JobState::Idle,
            light: None,
        });
        let mut mirror = Mirror::from_snapshot(initial).unwrap();

        mirror.apply_delta(Delta {
            msg_type: MessageType::Delta,
            tick: 10,
            tiles: Vec::new(),
            entities: vec![
                Entity {
                    id: 7,
                    kind: EntityKind::Dwarf,
                    pos: [1, 0, 0],
                    state: JobState::Walk,
                    light: None,
                },
                Entity {
                    id: 2,
                    kind: EntityKind::Dwarf,
                    pos: [1, 0, 0],
                    state: JobState::Idle,
                    light: None,
                },
                Entity {
                    id: 8,
                    kind: EntityKind::Dwarf,
                    pos: [0, 0, 0],
                    state: JobState::Idle,
                    light: None,
                },
            ],
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
        });

        assert_eq!(mirror.changes().spawned, vec![8]);
        assert_eq!(mirror.changes().despawned, vec![3]);
        assert_eq!(mirror.changes().changed, vec![7]);
        assert_eq!(mirror.previous_entity(7).unwrap().state, JobState::Idle);
        assert!(mirror.previous_entity(2).is_some());
        assert!(mirror.previous_entity(3).is_none());
        assert!(mirror.changes().spawned.iter().all(|id| *id != 3));
        assert!(mirror.changes().changed.iter().all(|id| *id != 2));

        mirror.apply_snapshot(snapshot()).unwrap();
        assert!(mirror.previous_entity(7).is_none());
        assert!(mirror.previous_entity(2).is_none());
    }

    #[test]
    fn tile_lookup_rejects_negative_and_out_of_range_coordinates() {
        let mirror = Mirror::from_snapshot(snapshot()).unwrap();

        assert_eq!(mirror.tile([0, 0, 0]), Some(Tile::Empty));
        assert_eq!(mirror.tile([1, 0, 0]), Some(Tile::Solid(Material::Ice)));
        for outside in [
            [-1, 0, 0],
            [0, -1, 0],
            [0, 0, -1],
            [2, 0, 0],
            [0, 1, 0],
            [0, 0, 1],
        ] {
            assert_eq!(mirror.tile(outside), None, "{outside:?} must be outside");
        }
    }

    #[test]
    fn rect_helper_normalizes_two_corners_on_one_level() {
        assert_eq!(
            rect_on_level((4, -2), (1, 3), 9),
            protocol::Rect {
                min: [1, -2, 9],
                max: [4, 3, 9],
            }
        );
    }
}
