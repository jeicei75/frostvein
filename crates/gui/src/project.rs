use bevy::prelude::{
    Assets, Commands, Component, Cuboid, Entity as BevyEntity, Handle, Mesh, Mesh3d,
    MeshMaterial3d, Query, ResMut, Resource, StandardMaterial, Transform,
};
use client_core::Mirror;
use protocol::{Dims, Tile};

use crate::transform::world_to_render;

/// Identifies a simulation-owned render entity by its authoritative simulation id.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldProjected(pub u32);

/// Marks camera and diagnostic entities that must never be touched by reconciliation.
#[derive(Component, Debug, Clone, Copy)]
pub struct ClientLocal;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainTile(pub [i32; 3]);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedItem(pub u32);

#[derive(Resource)]
pub struct ProjectionAssets {
    cube: Handle<Mesh>,
    materials: Vec<Handle<StandardMaterial>>,
}

pub fn setup_projection_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Mesh::from(Cuboid::default()));
    // Eight deliberately separate grey materials preserve batching by material while
    // leaving palette and light appearance to story 5.4.
    let materials = (0..8)
        .map(|_| materials.add(StandardMaterial::default()))
        .collect();
    commands.insert_resource(ProjectionAssets { cube, materials });
}

pub fn is_exposed(mirror: &Mirror, position: [i32; 3]) -> bool {
    if !matches!(mirror.tile(position), Some(Tile::Solid(_))) {
        return false;
    }
    const NEIGHBOURS: [[i32; 3]; 6] = [
        [-1, 0, 0],
        [1, 0, 0],
        [0, -1, 0],
        [0, 1, 0],
        [0, 0, -1],
        [0, 0, 1],
    ];
    NEIGHBOURS.into_iter().any(|delta| {
        let neighbour = [
            position[0] + delta[0],
            position[1] + delta[1],
            position[2] + delta[2],
        ];
        !matches!(mirror.tile(neighbour), Some(Tile::Solid(_)))
    })
}

/// Reconciles the small dynamic set by simulation `Id`; terrain is rebuilt after a snapshot.
pub fn reconcile(
    commands: &mut Commands,
    mirror: &Mirror,
    rebuild_terrain: bool,
    dirty_tiles: &[[i32; 3]],
    projected: &Query<(BevyEntity, &WorldProjected)>,
    terrain: &Query<(BevyEntity, &TerrainTile)>,
    assets: Option<&ProjectionAssets>,
) {
    if snapshot_needs_full_rebuild(rebuild_terrain) {
        for (entity, _) in terrain.iter() {
            commands.entity(entity).despawn();
        }
        for position in terrain_positions(mirror) {
            let mut entity = commands.spawn((
                WorldProjected(terrain_id(position, mirror.dims())),
                TerrainTile(position),
                Transform::from_translation(world_to_render(position)),
            ));
            if let Some(assets) = assets {
                entity.insert((
                    Mesh3d(assets.cube.clone()),
                    MeshMaterial3d(assets.materials[terrain_material(mirror, position)].clone()),
                ));
            }
        }
    } else {
        for position in dirty_tiles {
            for (entity, _) in terrain.iter().filter(|(_, tile)| tile.0 == *position) {
                commands.entity(entity).despawn();
            }
            if is_exposed(mirror, *position) {
                commands.spawn((
                    WorldProjected(terrain_id(*position, mirror.dims())),
                    TerrainTile(*position),
                    Transform::from_translation(world_to_render(*position)),
                ));
            }
        }
    }

    let mut wanted: std::collections::BTreeMap<_, _> = mirror
        .entities()
        .map(|entity| (entity.id, entity.pos))
        .collect();
    let item_ids: std::collections::BTreeSet<_> = mirror.items().map(|item| item.id).collect();
    wanted.extend(mirror.items().map(|item| (item.id, item.pos)));
    for (bevy_entity, marker) in projected.iter() {
        if !terrain.get(bevy_entity).is_ok() && !wanted.contains_key(&marker.0) {
            commands.entity(bevy_entity).despawn();
        }
    }
    for (id, position) in wanted {
        if let Some((bevy_entity, _)) = projected
            .iter()
            .find(|(_, marker)| marker_matches_id(**marker, id))
        {
            commands
                .entity(bevy_entity)
                .insert(Transform::from_translation(world_to_render(position)));
        } else {
            let mut entity = commands.spawn((
                WorldProjected(id),
                Transform::from_translation(world_to_render(position)),
            ));
            if item_ids.contains(&id) {
                entity.insert(ProjectedItem(id));
            }
            if let Some(assets) = assets {
                entity.insert((
                    Mesh3d(assets.cube.clone()),
                    MeshMaterial3d(assets.materials[0].clone()),
                ));
            }
        }
    }
}

fn snapshot_needs_full_rebuild(rebuild_terrain: bool) -> bool {
    rebuild_terrain
}

fn marker_matches_id(marker: WorldProjected, id: u32) -> bool {
    marker.0 == id
}

fn terrain_material(mirror: &Mirror, position: [i32; 3]) -> usize {
    match mirror.tile(position) {
        Some(Tile::Solid(protocol::Material::Stone)) => 0,
        Some(Tile::Solid(protocol::Material::Soil)) => 1,
        Some(Tile::Solid(protocol::Material::Ice)) => 2,
        Some(Tile::Solid(protocol::Material::Snow)) => 3,
        Some(Tile::Solid(protocol::Material::TreeTrunk)) => 4,
        Some(Tile::Solid(protocol::Material::TreeFoliage)) => 5,
        Some(Tile::Ramp(_)) => 6,
        Some(Tile::Empty) | None => 7,
    }
}

pub fn terrain_positions(mirror: &Mirror) -> Vec<[i32; 3]> {
    let mut positions = Vec::new();
    for_each_position(mirror.dims(), |position| {
        if is_exposed(mirror, position) {
            positions.push(position);
        }
    });
    positions
}

fn terrain_id([x, y, z]: [i32; 3], dims: Dims) -> u32 {
    x as u32 + y as u32 * dims.x + z as u32 * dims.x * dims.y
}

fn for_each_position(dims: Dims, mut visit: impl FnMut([i32; 3])) {
    // NOTE: `Dims` comes from validated snapshots; this loop is intentionally direct
    // rather than adding a one-use terrain iterator abstraction.
    for z in 0..dims.z as i32 {
        for y in 0..dims.y as i32 {
            for x in 0..dims.x as i32 {
                visit([x, y, z]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use client_core::Mirror;
    use protocol::{Dims, MessageType, Snapshot, Speed, Tile};

    use super::*;

    fn mirror(tiles: Vec<Tile>) -> Mirror {
        Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles,
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap()
    }

    #[test]
    fn exposed_predicate_keeps_boundary_solids_but_hides_fully_enclosed_ones() {
        let edge = mirror(vec![
            Tile::Solid(protocol::Material::Ice),
            Tile::Solid(protocol::Material::Ice),
        ]);
        assert!(is_exposed(&edge, [0, 0, 0]));
        let enclosed = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 3, y: 3, z: 3 },
            tiles: vec![Tile::Solid(protocol::Material::Ice); 27],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap();
        assert!(!is_exposed(&enclosed, [1, 1, 1]));
    }

    #[test]
    fn snapshot_rebuild_projects_terrain_even_when_the_mirror_reports_no_changes() {
        let initial = Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Empty, Tile::Empty],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        };
        let mut mirror = Mirror::from_snapshot(initial).unwrap();
        mirror
            .apply_snapshot(Snapshot {
                tiles: vec![Tile::Solid(protocol::Material::Ice), Tile::Empty],
                tick: 1,
                ..Snapshot {
                    msg_type: MessageType::Snapshot,
                    dims: Dims { x: 2, y: 1, z: 1 },
                    tiles: Vec::new(),
                    entities: Vec::new(),
                    designations: Vec::new(),
                    zones: Vec::new(),
                    items: Vec::new(),
                    speed: Speed::Normal,
                    tick: 0,
                }
            })
            .unwrap();

        assert!(mirror.changes().tiles.is_empty());
        assert!(snapshot_needs_full_rebuild(true));
        assert_eq!(terrain_positions(&mirror), vec![[0, 0, 0]]);
    }

    #[test]
    fn reconciliation_identity_is_the_simulation_id() {
        assert!(marker_matches_id(WorldProjected(42), 42));
        assert!(!marker_matches_id(WorldProjected(42), 41));
    }
}
