use std::collections::BTreeSet;

use bevy::prelude::{
    Assets, Commands, Component, Cuboid, Entity as BevyEntity, Handle, Mesh, Mesh3d,
    MeshMaterial3d, Or, PointLight, Query, ResMut, Resource, StandardMaterial, Transform, Vec3,
    With, Without,
};
use client_core::Mirror;
use protocol::{Dims, EntityKind, Material, Tile};

use crate::{
    appearance::{
        RIM_LEVELS, debris_color, entity_appearance, flicker_scale, foliage_snow_color,
        light_properties, material_color, rim_dissolved_color, snow_cap_color,
    },
    blend::{TickClock, blended_translation},
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

/// The light kind delivered for a projected emitter. Reconciliation only changes this when the
/// wire changes kind; presentation owns its animated intensity afterwards.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedLight(pub protocol::LightKind);

pub const CHIPS_PER_TILE: usize = 4;

/// A presentation-only chip from a tile removal. It deliberately has no simulation id.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigChip(pub [i32; 3]);

pub type DigChipQuery<'w, 's> = Query<'w, 's, (BevyEntity, &'static DigChip)>;

/// The terrain surfaces that get their own material, including the two presentation-only ones.
#[derive(Debug, Clone, Copy)]
enum TerrainSlot {
    Stone,
    Soil,
    Ice,
    Snow,
    TreeTrunk,
    TreeFoliage,
    FoliageCrown,
    SnowCap,
}

const TERRAIN_SLOTS: [TerrainSlot; 8] = [
    TerrainSlot::Stone,
    TerrainSlot::Soil,
    TerrainSlot::Ice,
    TerrainSlot::Snow,
    TerrainSlot::TreeTrunk,
    TerrainSlot::TreeFoliage,
    TerrainSlot::FoliageCrown,
    TerrainSlot::SnowCap,
];

impl TerrainSlot {
    fn base_color(self) -> bevy::prelude::Color {
        match self {
            TerrainSlot::Stone => material_color(Material::Stone),
            TerrainSlot::Soil => material_color(Material::Soil),
            TerrainSlot::Ice => material_color(Material::Ice),
            TerrainSlot::Snow => material_color(Material::Snow),
            TerrainSlot::TreeTrunk => material_color(Material::TreeTrunk),
            TerrainSlot::TreeFoliage => material_color(Material::TreeFoliage),
            TerrainSlot::FoliageCrown => foliage_snow_color(),
            TerrainSlot::SnowCap => snow_cap_color(),
        }
    }

    fn of(material: Material) -> Self {
        match material {
            Material::Stone => TerrainSlot::Stone,
            Material::Soil => TerrainSlot::Soil,
            Material::Ice => TerrainSlot::Ice,
            Material::Snow => TerrainSlot::Snow,
            Material::TreeTrunk => TerrainSlot::TreeTrunk,
            Material::TreeFoliage => TerrainSlot::TreeFoliage,
        }
    }
}

#[derive(Resource)]
pub struct ProjectionAssets {
    cube: Handle<Mesh>,
    snow_cap_mesh: Handle<Mesh>,
    /// One handle per (surface, rim step); see `rim_level`.
    terrain: [[Handle<StandardMaterial>; RIM_LEVELS]; TERRAIN_SLOTS.len()],
    dwarf: Handle<StandardMaterial>,
    torch: Handle<StandardMaterial>,
    campfire: Handle<StandardMaterial>,
    debris: Handle<StandardMaterial>,
}

pub fn setup_projection_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Mesh::from(Cuboid::default()));
    let snow_cap_mesh = meshes.add(Mesh::from(Cuboid::new(1.02, 0.08, 1.02)));
    let terrain = TERRAIN_SLOTS.map(|slot| {
        std::array::from_fn(|level| {
            materials.add(terrain_standard_material(rim_dissolved_color(
                slot.base_color(),
                level,
            )))
        })
    });
    commands.insert_resource(ProjectionAssets {
        cube,
        snow_cap_mesh,
        terrain,
        dwarf: materials.add(entity_standard_material(EntityKind::Dwarf)),
        torch: materials.add(entity_standard_material(EntityKind::Torch)),
        campfire: materials.add(entity_standard_material(EntityKind::Campfire)),
        debris: materials.add(terrain_standard_material(debris_color())),
    });
}

fn terrain_standard_material(base_color: bevy::prelude::Color) -> StandardMaterial {
    StandardMaterial {
        base_color,
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
// The three query parameters are kept explicit: they are distinct ECS partitions, and bundling
// them solely to satisfy this lint would obscure the `ClientLocal` / `WorldProjected` boundary.
#[allow(clippy::too_many_arguments)]
pub fn reconcile(
    commands: &mut Commands,
    mirror: &Mirror,
    rebuild_terrain: bool,
    dirty_tiles: &[[i32; 3]],
    projected: &Query<(BevyEntity, &WorldProjected, Option<&ProjectedLight>), Without<TerrainTile>>,
    terrain: &TerrainQuery,
    chips: &DigChipQuery,
    assets: Option<&ProjectionAssets>,
) {
    if rebuild_terrain {
        for (entity, _) in chips.iter() {
            commands.entity(entity).despawn();
        }
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
                    terrain_transform(mirror, position),
                ))
                .id();
            if let Some(assets) = assets {
                commands.entity(entity).insert((
                    Mesh3d(assets.cube.clone()),
                    MeshMaterial3d(assets.terrain_material(mirror, position)),
                ));
                if has_snow_cap(mirror, position) {
                    spawn_snow_cap(commands, assets, mirror, position);
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
                        terrain_transform(mirror, position),
                    ))
                    .id();
                if let Some(assets) = assets {
                    commands.entity(entity).insert((
                        Mesh3d(assets.cube.clone()),
                        MeshMaterial3d(assets.terrain_material(mirror, position)),
                    ));
                    if has_snow_cap(mirror, position) {
                        spawn_snow_cap(commands, assets, mirror, position);
                    }
                }
            }
        }
    }

    if !rebuild_terrain {
        for position in dirty_tiles {
            for (entity, chip) in chips.iter() {
                if chip.0 == *position {
                    commands.entity(entity).despawn();
                }
            }
            if matches!(mirror.tile(*position), Some(Tile::Empty)) {
                for offset in chip_offsets() {
                    let mut entity = commands.spawn((
                        DigChip(*position),
                        ClientLocal,
                        Transform::from_translation(world_to_render(*position) + offset)
                            .with_scale(Vec3::splat(0.14)),
                    ));
                    if let Some(assets) = assets {
                        entity.insert((
                            Mesh3d(assets.cube.clone()),
                            MeshMaterial3d(assets.debris.clone()),
                        ));
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
    for (bevy_entity, marker, _) in projected.iter() {
        if !terrain.get(bevy_entity).is_ok() && !wanted.contains_key(&marker.0) {
            commands.entity(bevy_entity).despawn();
        }
    }
    for (id, (position, mirror_entity)) in wanted {
        // NOTE: terrain and simulation entities retain the same marker component and
        // numeric range. Keep this query filtered to prevent a terrain id colliding
        // with a simulation id until a story needs separate marker types.
        if let Some((bevy_entity, _, projected_light)) =
            projected.iter().find(|(_, marker, _)| marker.0 == id)
        {
            // Translation belongs solely to `blend_entities` after spawn. Re-inserting it here
            // makes an otherwise-correct blend present-but-inert on the next reconcile.
            if let Some(light) = mirror_entity.and_then(|entity| entity.light) {
                if projected_light.is_none_or(|existing| existing.0 != light) {
                    commands
                        .entity(bevy_entity)
                        .insert((point_light(light), ProjectedLight(light)));
                }
            } else {
                commands
                    .entity(bevy_entity)
                    .remove::<(PointLight, ProjectedLight)>();
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
                        entity.insert((point_light(light), ProjectedLight(light)));
                    }
                } else if item_ids.contains(&id) {
                    entity.insert((
                        Mesh3d(assets.cube.clone()),
                        MeshMaterial3d(assets.slot(TerrainSlot::Stone, 0)),
                    ));
                }
            }
        }
    }
}

fn chip_offsets() -> [Vec3; CHIPS_PER_TILE] {
    [
        Vec3::new(-0.26, -0.32, -0.16),
        Vec3::new(0.19, -0.26, -0.24),
        Vec3::new(-0.12, -0.20, 0.21),
        Vec3::new(0.27, -0.30, 0.14),
    ]
}

/// Keeps point lights alive as presentation state rather than re-inserting table intensity.
pub fn flicker_lights(
    seconds: f32,
    lights: &mut Query<(&WorldProjected, &ProjectedLight, &mut PointLight)>,
) {
    for (id, kind, mut light) in lights.iter_mut() {
        light.intensity = light_properties(kind.0).intensity * flicker_scale(kind.0, id.0, seconds);
    }
}

/// Applies presentation interpolation to dynamic wire projections only.
pub fn blend_entities(
    mirror: &Mirror,
    clock: &mut TickClock,
    elapsed_seconds: f32,
    projected: &mut Query<(&WorldProjected, &mut Transform), Without<TerrainTile>>,
) {
    clock.advance(elapsed_seconds);
    let entities = mirror
        .entities()
        .map(|entity| (entity.id, entity))
        .collect::<std::collections::BTreeMap<_, _>>();
    let items = mirror
        .items()
        .map(|item| (item.id, item.pos))
        .collect::<std::collections::BTreeMap<_, _>>();
    for (marker, mut transform) in projected.iter_mut() {
        if let Some(entity) = entities.get(&marker.0) {
            transform.translation = blended_translation(
                mirror
                    .previous_entity(marker.0)
                    .map(|previous| previous.pos),
                entity.pos,
                clock.factor(),
            );
        } else if let Some(position) = items.get(&marker.0) {
            // Items have no previous wire state; snapping is the only wire-true presentation.
            transform.translation = world_to_render(*position);
        }
    }
}

fn terrain_material(mirror: &Mirror, position: [i32; 3]) -> Material {
    match mirror.tile(position) {
        Some(Tile::Solid(material) | Tile::Ramp(material)) => material,
        Some(Tile::Empty) | None => Material::Stone,
    }
}

fn terrain_transform(mirror: &Mirror, position: [i32; 3]) -> Transform {
    Transform::from_translation(world_to_render(position))
        .with_scale(Vec3::splat(foliage_scale(mirror, position)))
}

/// Keeps cube foliage readable as sparse spruce branches instead of a solid square canopy.
pub fn foliage_scale(mirror: &Mirror, position: [i32; 3]) -> f32 {
    if terrain_material(mirror, position) != Material::TreeFoliage {
        return 1.0;
    }
    let foliage_above = (1..=2)
        .take_while(|offset| {
            matches!(
                mirror.tile([position[0], position[1], position[2] + offset]),
                Some(Tile::Solid(Material::TreeFoliage))
            )
        })
        .count();
    // Trimmed at round 7 — full-scale crowns made the boot4 foreground read as clutter.
    match foliage_above {
        0 => 0.62,
        1 => 0.78,
        _ => 0.95,
    }
}

fn spawn_snow_cap(
    commands: &mut Commands,
    assets: &ProjectionAssets,
    mirror: &Mirror,
    position: [i32; 3],
) {
    commands.spawn((
        SnowCap(position),
        ClientLocal,
        Mesh3d(assets.snow_cap_mesh.clone()),
        MeshMaterial3d(assets.slot(TerrainSlot::SnowCap, rim_level(position, mirror.dims()))),
        Transform::from_translation(world_to_render(position) + Vec3::Y * 0.54),
    ));
}

/// How far into the world-edge dissolve a tile sits: 0 for the interior, `RIM_LEVELS - 1` at
/// the boundary itself. The whole visible skyline at the boot framing IS the map boundary
/// (measured: silhouette depths 86-145 units against a camp at 71), so distance fog tight
/// enough to hide it also erases the valley. Keying the dissolve to world position instead of
/// camera depth removes the edge at every zoom without touching the interior.
pub fn rim_level(position: [i32; 3], dims: Dims) -> usize {
    let to_edge = position[0]
        .min(dims.x as i32 - 1 - position[0])
        .min(position[1])
        .min(dims.y as i32 - 1 - position[1])
        .max(0);
    if to_edge >= RIM_WIDTH {
        return 0;
    }
    // Quadratic ease: the dissolve stays subtle through its inner half and commits to the sky
    // only near the boundary. The linear 5-step/10-tile ramp read as a hard band on the boot4
    // vehicle capture (Wolf: "falloff to sky is too sharp").
    let steps = (RIM_LEVELS - 1) as i32;
    let inward = RIM_WIDTH - to_edge;
    ((inward * inward * steps) / (RIM_WIDTH * RIM_WIDTH)).clamp(0, steps) as usize
}

/// How many tiles inward the dissolve reaches.
pub const RIM_WIDTH: i32 = 26;

/// An exposed spruce crown catches snow light. This is a MATERIAL swap, not a terrain cap:
/// capping foliage puts a bright slab on every ground-level skirt tile and buries the landform.
pub fn has_snow_laden_crown(mirror: &Mirror, position: [i32; 3]) -> bool {
    terrain_material_at(mirror, position) == Some(Material::TreeFoliage)
        && !matches!(
            mirror.tile([position[0], position[1], position[2] + 1]),
            Some(Tile::Solid(_) | Tile::Ramp(_))
        )
}

/// The material actually present, distinguishing air from the `terrain_material` fallback.
fn terrain_material_at(mirror: &Mirror, position: [i32; 3]) -> Option<Material> {
    match mirror.tile(position) {
        Some(Tile::Solid(material) | Tile::Ramp(material)) => Some(material),
        Some(Tile::Empty) | None => None,
    }
}

/// The cap is presentation-only: wire terrain remains its original material.
pub fn has_snow_cap(mirror: &Mirror, position: [i32; 3]) -> bool {
    matches!(
        mirror.tile(position),
        Some(Tile::Solid(material) | Tile::Ramp(material))
            if material != Material::Ice && material != Material::TreeFoliage
    ) && !matches!(
        mirror.tile([position[0], position[1], position[2] + 1]),
        Some(Tile::Solid(_) | Tile::Ramp(_))
    )
}

impl ProjectionAssets {
    fn slot(&self, slot: TerrainSlot, level: usize) -> Handle<StandardMaterial> {
        self.terrain[slot as usize][level.min(RIM_LEVELS - 1)].clone()
    }

    fn terrain_material(&self, mirror: &Mirror, position: [i32; 3]) -> Handle<StandardMaterial> {
        let level = rim_level(position, mirror.dims());
        if has_snow_laden_crown(mirror, position) {
            return self.slot(TerrainSlot::FoliageCrown, level);
        }
        self.slot(TerrainSlot::of(terrain_material(mirror, position)), level)
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
    fn snow_caps_follow_material_and_exposure_in_a_seed_shaped_toy_world() {
        let toy = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 10, y: 1, z: 2 },
            tiles: vec![
                Tile::Solid(protocol::Material::Ice),
                Tile::Solid(protocol::Material::Snow),
                Tile::Solid(protocol::Material::TreeFoliage),
                Tile::Solid(protocol::Material::Stone),
                Tile::Solid(protocol::Material::Stone),
                Tile::Solid(protocol::Material::Soil),
                Tile::Ramp(protocol::Material::Ice),
                Tile::Ramp(protocol::Material::Snow),
                Tile::Ramp(protocol::Material::Stone),
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Solid(protocol::Material::Ice),
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
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
            "exposed ice keeps its blue top"
        );
        assert!(has_snow_cap(&toy, [1, 0, 0]), "snow can settle on snow");
        assert!(
            !has_snow_cap(&toy, [2, 0, 0]),
            "foliage stays dark so ground-level skirts cannot read as snow slabs"
        );
        assert!(has_snow_cap(&toy, [4, 0, 0]), "stone can carry snow");
        assert!(has_snow_cap(&toy, [5, 0, 0]), "soil can carry snow");
        assert!(
            !has_snow_cap(&toy, [3, 0, 0]),
            "covered terrain keeps its dark flank"
        );
        assert!(
            !has_snow_cap(&toy, [6, 0, 0]),
            "ice ramps keep their blue tops"
        );
        assert!(has_snow_cap(&toy, [7, 0, 0]), "snow ramps are capped");
        assert!(has_snow_cap(&toy, [8, 0, 0]), "stone ramps are capped");
        assert!(!has_snow_cap(&toy, [9, 0, 0]), "air never receives a cap");
    }

    #[test]
    fn foliage_tapers_from_wide_mid_crown_to_narrow_tip_and_skirt() {
        let spruce = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 1, y: 1, z: 6 },
            tiles: vec![
                Tile::Solid(Material::TreeFoliage),
                Tile::Solid(Material::TreeTrunk),
                Tile::Solid(Material::TreeFoliage),
                Tile::Solid(Material::TreeFoliage),
                Tile::Solid(Material::TreeFoliage),
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

        assert_eq!(foliage_scale(&spruce, [0, 0, 0]), 0.62, "skirt");
        assert_eq!(foliage_scale(&spruce, [0, 0, 2]), 0.95, "mid crown");
        assert_eq!(foliage_scale(&spruce, [0, 0, 3]), 0.78, "upper crown");
        assert_eq!(foliage_scale(&spruce, [0, 0, 4]), 0.62, "crown tip");
    }

    #[test]
    fn the_world_edge_dissolves_inward_and_leaves_the_interior_alone() {
        let dims = Dims {
            x: 128,
            y: 128,
            z: 32,
        };
        // The interior is untouched, whichever axis you approach from.
        assert_eq!(rim_level([64, 64, 9], dims), 0);
        assert_eq!(rim_level([RIM_WIDTH, 64, 9], dims), 0);
        assert_eq!(rim_level([64, RIM_WIDTH, 9], dims), 0);

        // Every boundary face reaches the last step, or one side of the map keeps a raw edge.
        for corner in [[0, 64, 9], [127, 64, 9], [64, 0, 9], [64, 127, 9]] {
            assert_eq!(
                rim_level(corner, dims),
                RIM_LEVELS - 1,
                "the map boundary at {corner:?} must dissolve completely"
            );
        }

        // Monotonic inward, so the dissolve reads as a gradient rather than a ring.
        let walk: Vec<usize> = (0..=RIM_WIDTH)
            .map(|x| rim_level([x, 64, 9], dims))
            .collect();
        for pair in walk.windows(2) {
            assert!(
                pair[1] <= pair[0],
                "the dissolve must ease inward; {walk:?}"
            );
        }
        assert_eq!(walk.first(), Some(&(RIM_LEVELS - 1)));
        assert_eq!(walk.last(), Some(&0));

        // Eased, not linear: at half depth the dissolve must still be gentle, or the ramp
        // reads as a hard band the way the linear version did on the boot4 capture.
        let halfway = rim_level([RIM_WIDTH / 2, 64, 9], dims);
        assert!(
            halfway <= (RIM_LEVELS - 1) / 3,
            "the inner half of the rim must stay subtle; level {halfway} at half depth"
        );
    }

    /// Wolf's ruling, review-patch round 5: trees read as dark clumps because the approved
    /// artifact carries snow on its spruce layer tops and cube foliage had no equivalent. This
    /// is a MATERIAL choice on the exposed crown, deliberately not a terrain cap — capping
    /// foliage was round 3's defect (it buried the landform under ~9,500 bright slabs).
    #[test]
    fn only_the_exposed_crown_of_a_spruce_catches_snow_light() {
        let spruce = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 3 },
            tiles: vec![
                Tile::Solid(Material::TreeFoliage),
                Tile::Solid(Material::Snow),
                Tile::Solid(Material::TreeFoliage),
                Tile::Empty,
                Tile::Empty,
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
            has_snow_laden_crown(&spruce, [0, 0, 1]),
            "the topmost exposed foliage cube catches the snow light"
        );
        assert!(
            !has_snow_laden_crown(&spruce, [0, 0, 0]),
            "foliage with foliage above it stays dark inside the crown"
        );
        assert!(
            !has_snow_laden_crown(&spruce, [1, 0, 0]),
            "snow terrain is not foliage and keeps the terrain cap path"
        );
        assert!(
            !has_snow_laden_crown(&spruce, [0, 0, 2]),
            "air never catches snow"
        );
        // The crown must not ALSO take a terrain cap — that combination was round 3's defect.
        assert!(
            !has_snow_cap(&spruce, [0, 0, 1]),
            "a snow-laden crown is a material, never a terrain slab"
        );
    }
}
