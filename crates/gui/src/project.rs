use std::collections::BTreeSet;

use bevy::prelude::{
    Assets, Commands, Component, Cuboid, Entity as BevyEntity, Handle, Mesh, Mesh3d,
    MeshMaterial3d, Or, PointLight, Query, ResMut, Resource, StandardMaterial, Transform, Vec3,
    With, Without,
};
use client_core::Mirror;
use protocol::{Dims, EntityKind, Material, Tile};

use crate::{
    appearance::{entity_appearance, light_properties, material_color},
    transform::world_to_render,
};

const NEIGHBOURS: [[i32; 3]; 6] = [
    [-1, 0, 0],
    [1, 0, 0],
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
];

/// Identifies a simulation-owned render entity by its authoritative simulation id.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldProjected(pub u32);

/// Marks camera and diagnostic entities that must never be touched by reconciliation.
#[derive(Component, Debug, Clone, Copy)]
pub struct ClientLocal;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainTile(pub [i32; 3]);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnowCap(pub [i32; 3]);

pub type TerrainQuery<'w, 's> = Query<
    'w,
    's,
    (
        BevyEntity,
        Option<&'static TerrainTile>,
        Option<&'static SnowCap>,
    ),
    Or<(With<TerrainTile>, With<SnowCap>)>,
>;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedItem(pub u32);

#[derive(Resource)]
pub struct ProjectionAssets {
    cube: Handle<Mesh>,
    snow_cap: Handle<Mesh>,
    stone: Handle<StandardMaterial>,
    soil: Handle<StandardMaterial>,
    ice: Handle<StandardMaterial>,
    snow: Handle<StandardMaterial>,
    tree_trunk: Handle<StandardMaterial>,
    tree_foliage: Handle<StandardMaterial>,
    dwarf: Handle<StandardMaterial>,
    torch: Handle<StandardMaterial>,
    campfire: Handle<StandardMaterial>,
}

pub fn setup_projection_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Mesh::from(Cuboid::default()));
    let snow_cap = meshes.add(Mesh::from(Cuboid::new(1.02, 0.08, 1.02)));
    commands.insert_resource(ProjectionAssets {
        cube,
        snow_cap,
        stone: materials.add(terrain_standard_material(Material::Stone)),
        soil: materials.add(terrain_standard_material(Material::Soil)),
        ice: materials.add(terrain_standard_material(Material::Ice)),
        snow: materials.add(terrain_standard_material(Material::Snow)),
        tree_trunk: materials.add(terrain_standard_material(Material::TreeTrunk)),
        tree_foliage: materials.add(terrain_standard_material(Material::TreeFoliage)),
        dwarf: materials.add(entity_standard_material(EntityKind::Dwarf)),
        torch: materials.add(entity_standard_material(EntityKind::Torch)),
        campfire: materials.add(entity_standard_material(EntityKind::Campfire)),
    });
}

fn terrain_standard_material(material: Material) -> StandardMaterial {
    StandardMaterial {
        base_color: material_color(material),
        perceptual_roughness: 0.9,
        ..Default::default()
    }
}

fn entity_standard_material(kind: EntityKind) -> StandardMaterial {
    let mut material = StandardMaterial {
        base_color: entity_appearance(kind).color,
        perceptual_roughness: 0.75,
        ..Default::default()
    };
    if let Some(light) = entity_light_kind(kind) {
        material.emissive = light_properties(light).color.to_linear();
    }
    material
}

fn entity_light_kind(kind: EntityKind) -> Option<protocol::LightKind> {
    match kind {
        EntityKind::Dwarf => None,
        EntityKind::Torch => Some(protocol::LightKind::Torch),
        EntityKind::Campfire => Some(protocol::LightKind::Campfire),
    }
}

fn point_light(kind: protocol::LightKind) -> PointLight {
    let properties = light_properties(kind);
    PointLight {
        color: properties.color,
        intensity: properties.intensity,
        range: properties.range,
        ..Default::default()
    }
}

pub fn is_exposed(mirror: &Mirror, position: [i32; 3]) -> bool {
    // Ramps are terrain: drawn and occluding, exactly as the AC13 oracle counts them.
    if !matches!(mirror.tile(position), Some(Tile::Solid(_) | Tile::Ramp(_))) {
        return false;
    }
    NEIGHBOURS.into_iter().any(|delta| {
        let neighbour = [
            position[0] + delta[0],
            position[1] + delta[1],
            position[2] + delta[2],
        ];
        !matches!(mirror.tile(neighbour), Some(Tile::Solid(_) | Tile::Ramp(_)))
    })
}

/// Reconciles the small dynamic set by simulation `Id`; terrain is rebuilt after a snapshot.
pub fn reconcile(
    commands: &mut Commands,
    mirror: &Mirror,
    rebuild_terrain: bool,
    dirty_tiles: &[[i32; 3]],
    projected: &Query<(BevyEntity, &WorldProjected), Without<TerrainTile>>,
    terrain: &TerrainQuery,
    assets: Option<&ProjectionAssets>,
) {
    if rebuild_terrain {
        for (entity, _, _) in terrain.iter() {
            commands.entity(entity).despawn();
        }
        let positions = terrain_positions(mirror);
        // The draw-set oracle instrument: the shipped seed must report 53,365 (AC13).
        println!("projected {} terrain cubes", positions.len());
        for position in positions {
            let entity = commands
                .spawn((
                    WorldProjected(terrain_id(position, mirror.dims())),
                    TerrainTile(position),
                    Transform::from_translation(world_to_render(position)),
                ))
                .id();
            if let Some(assets) = assets {
                commands.entity(entity).insert((
                    Mesh3d(assets.cube.clone()),
                    MeshMaterial3d(assets.terrain_material(mirror, position)),
                ));
                if has_snow_cap(mirror, position) {
                    spawn_snow_cap(commands, assets, position);
                }
            }
        }
    } else {
        let mut affected = BTreeSet::new();
        for position in dirty_tiles {
            affected.insert(*position);
            for delta in NEIGHBOURS {
                affected.insert([
                    position[0] + delta[0],
                    position[1] + delta[1],
                    position[2] + delta[2],
                ]);
            }
        }
        for position in affected {
            for (entity, tile, cap) in terrain.iter() {
                if tile.is_some_and(|tile| tile.0 == position)
                    || cap.is_some_and(|cap| cap.0 == position)
                {
                    commands.entity(entity).despawn();
                }
            }
            if is_exposed(mirror, position) {
                let entity = commands
                    .spawn((
                        WorldProjected(terrain_id(position, mirror.dims())),
                        TerrainTile(position),
                        Transform::from_translation(world_to_render(position)),
                    ))
                    .id();
                if let Some(assets) = assets {
                    commands.entity(entity).insert((
                        Mesh3d(assets.cube.clone()),
                        MeshMaterial3d(assets.terrain_material(mirror, position)),
                    ));
                    if has_snow_cap(mirror, position) {
                        spawn_snow_cap(commands, assets, position);
                    }
                }
            }
        }
    }

    let mut wanted: std::collections::BTreeMap<_, _> = mirror
        .entities()
        .map(|entity| (entity.id, (entity.pos, Some(*entity))))
        .collect();
    let item_ids: std::collections::BTreeSet<_> = mirror.items().map(|item| item.id).collect();
    wanted.extend(mirror.items().map(|item| (item.id, (item.pos, None))));
    for (bevy_entity, marker) in projected.iter() {
        if !terrain.get(bevy_entity).is_ok() && !wanted.contains_key(&marker.0) {
            commands.entity(bevy_entity).despawn();
        }
    }
    for (id, (position, mirror_entity)) in wanted {
        // NOTE: terrain and simulation entities retain the same marker component and
        // numeric range. Keep this query filtered to prevent a terrain id colliding
        // with a simulation id until a story needs separate marker types.
        if let Some((bevy_entity, _)) = projected.iter().find(|(_, marker)| marker.0 == id) {
            let mut transform = Transform::from_translation(world_to_render(position));
            if let Some(mirror_entity) = mirror_entity {
                transform.scale =
                    bevy::prelude::Vec3::splat(entity_appearance(mirror_entity.kind).scale);
            }
            commands.entity(bevy_entity).insert(transform);
            if let Some(light) = mirror_entity.and_then(|entity| entity.light) {
                commands.entity(bevy_entity).insert(point_light(light));
            } else {
                commands.entity(bevy_entity).remove::<PointLight>();
            }
        } else {
            let mut entity = commands.spawn((
                WorldProjected(id),
                Transform::from_translation(world_to_render(position)),
            ));
            if item_ids.contains(&id) {
                entity.insert(ProjectedItem(id));
            }
            if let Some(assets) = assets {
                if let Some(mirror_entity) = mirror_entity {
                    let appearance = entity_appearance(mirror_entity.kind);
                    entity.insert((
                        Mesh3d(assets.cube.clone()),
                        MeshMaterial3d(assets.entity_material(mirror_entity.kind)),
                        Transform::from_translation(world_to_render(position))
                            .with_scale(bevy::prelude::Vec3::splat(appearance.scale)),
                    ));
                    if let Some(light) = mirror_entity.light {
                        entity.insert(point_light(light));
                    }
                } else if item_ids.contains(&id) {
                    entity.insert((
                        Mesh3d(assets.cube.clone()),
                        MeshMaterial3d(assets.stone.clone()),
                    ));
                }
            }
        }
    }
}

fn terrain_material(mirror: &Mirror, position: [i32; 3]) -> Material {
    match mirror.tile(position) {
        Some(Tile::Solid(material) | Tile::Ramp(material)) => material,
        Some(Tile::Empty) | None => Material::Stone,
    }
}

fn spawn_snow_cap(commands: &mut Commands, assets: &ProjectionAssets, position: [i32; 3]) {
    commands.spawn((
        SnowCap(position),
        Mesh3d(assets.snow_cap.clone()),
        MeshMaterial3d(assets.snow.clone()),
        Transform::from_translation(world_to_render(position) + Vec3::Y * 0.54),
    ));
}

/// The cap is presentation-only: wire terrain remains its original material.
pub fn has_snow_cap(mirror: &Mirror, position: [i32; 3]) -> bool {
    matches!(mirror.tile(position), Some(Tile::Solid(_)))
        && !matches!(
            mirror.tile([position[0], position[1], position[2] + 1]),
            Some(Tile::Solid(_) | Tile::Ramp(_))
        )
}

impl ProjectionAssets {
    fn terrain_material(&self, mirror: &Mirror, position: [i32; 3]) -> Handle<StandardMaterial> {
        let material = terrain_material(mirror, position);
        match material {
            Material::Stone => self.stone.clone(),
            Material::Soil => self.soil.clone(),
            Material::Ice => self.ice.clone(),
            Material::Snow => self.snow.clone(),
            Material::TreeTrunk => self.tree_trunk.clone(),
            Material::TreeFoliage => self.tree_foliage.clone(),
        }
    }

    fn entity_material(&self, kind: EntityKind) -> Handle<StandardMaterial> {
        match kind {
            EntityKind::Dwarf => self.dwarf.clone(),
            EntityKind::Torch => self.torch.clone(),
            EntityKind::Campfire => self.campfire.clone(),
        }
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
    fn ramps_are_drawn_and_occlude_like_solids() {
        let ramp_edge = mirror(vec![
            Tile::Ramp(protocol::Material::Ice),
            Tile::Solid(protocol::Material::Ice),
        ]);
        assert!(
            is_exposed(&ramp_edge, [0, 0, 0]),
            "a boundary ramp belongs to the draw set"
        );
        assert_eq!(
            terrain_material(&ramp_edge, [0, 0, 0]),
            protocol::Material::Ice
        );
        let mut tiles = vec![Tile::Solid(protocol::Material::Ice); 27];
        tiles[12] = Tile::Ramp(protocol::Material::Ice); // [0, 1, 1], a face neighbour of the centre
        let enclosed = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 3, y: 3, z: 3 },
            tiles,
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap();
        assert!(
            !is_exposed(&enclosed, [1, 1, 1]),
            "a ramp neighbour occludes like a solid"
        );
    }

    #[test]
    fn snow_cap_marks_only_solid_tops_in_a_hand_built_toy_world() {
        let toy = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 1, y: 1, z: 3 },
            tiles: vec![
                Tile::Solid(protocol::Material::Stone),
                Tile::Solid(protocol::Material::TreeFoliage),
                Tile::Empty,
            ],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap();
        assert!(
            !has_snow_cap(&toy, [0, 0, 0]),
            "covered stone keeps its dark flank"
        );
        assert!(
            has_snow_cap(&toy, [0, 0, 1]),
            "exposed foliage carries a loaded cap"
        );
        assert!(!has_snow_cap(&toy, [0, 0, 2]), "air never receives a cap");
    }
}
