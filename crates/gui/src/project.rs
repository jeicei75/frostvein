use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::{
    Assets, Commands, Component, Cuboid, Entity as BevyEntity, Handle, Mesh, Mesh3d,
    MeshMaterial3d, Or, PointLight, Query, Res, ResMut, Resource, StandardMaterial, Transform,
    Vec3, With, Without,
};
use client_core::Mirror;
use protocol::{DesignationKind, Dims, EntityKind, Material, Tile};

use crate::{
    appearance::{
        RIM_LEVELS, STONE_ITEM_DROP, STONE_ITEM_SCALE, debris_color, designation_color,
        entity_appearance, flicker_scale, foliage_snow_color, hover_highlight_color,
        light_properties, material_color, rim_dissolved_color, snow_cap_color, zone_color,
    },
    blend::{TickClock, blended_translation},
    designate::DragAnchor,
    pick::PickedTile,
    slice::SliceLevel,
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

pub type DynamicProjectionQuery<'w, 's> = Query<
    'w,
    's,
    (
        BevyEntity,
        &'static WorldProjected,
        Option<&'static ProjectedLight>,
    ),
    (
        Without<TerrainTile>,
        Without<ProjectedDesignation>,
        Without<ProjectedZone>,
    ),
>;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedItem(pub u32);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedDesignation(pub [i32; 3]);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedDesignationKind(pub DesignationKind);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedZone(pub [i32; 3]);

/// The client-local slab drawn under the tile currently under the cursor.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoverHighlight(pub [i32; 3]);

/// Client-only slabs shown while a designation drag is held.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragPreview(pub [i32; 3]);

/// Leaves a visible gutter between neighbouring mark slabs. The mesh is 1.02 wide and this scale
/// is applied on top of it, so a slab covers 1.02 x 0.94 = 0.9588 of its tile — inset ~2% per
/// side, which is the gutter, not a reach to the tile edge.
///
/// NOTE: measured at the 2026-08-21 review, one tile step spans 48.8 px of a 1280-wide frame at
/// `--distance 30`, so the gutter is ~2 px at the working zoom and ~0.65 px at the boot vista.
/// Whether that reads as separate tiles or as anti-aliasing noise is a human call, and it is on
/// the list for Wolf's live viewing.
const MARK_FOOTPRINT_SCALE: f32 = 0.94;

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
    mark_mesh: Handle<Mesh>,
    /// One handle per (surface, rim step); see `rim_level`.
    terrain: [[Handle<StandardMaterial>; RIM_LEVELS]; TERRAIN_SLOTS.len()],
    dwarf: Handle<StandardMaterial>,
    torch: Handle<StandardMaterial>,
    campfire: Handle<StandardMaterial>,
    debris: Handle<StandardMaterial>,
    dig_mark: Handle<StandardMaterial>,
    channel_mark: Handle<StandardMaterial>,
    zone_mark: Handle<StandardMaterial>,
    hover_highlight: Handle<StandardMaterial>,
}

pub fn setup_projection_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Mesh::from(Cuboid::default()));
    let snow_cap_mesh = meshes.add(Mesh::from(Cuboid::new(1.02, 0.08, 1.02)));
    let mark_mesh = meshes.add(Mesh::from(Cuboid::new(1.02, 0.08, 1.02)));
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
        mark_mesh,
        terrain,
        dwarf: materials.add(entity_standard_material(EntityKind::Dwarf)),
        torch: materials.add(entity_standard_material(EntityKind::Torch)),
        campfire: materials.add(entity_standard_material(EntityKind::Campfire)),
        debris: materials.add(terrain_standard_material(debris_color())),
        dig_mark: materials.add(terrain_standard_material(designation_color(
            DesignationKind::Dig,
        ))),
        channel_mark: materials.add(terrain_standard_material(designation_color(
            DesignationKind::Channel,
        ))),
        zone_mark: materials.add(terrain_standard_material(zone_color())),
        hover_highlight: materials.add(terrain_standard_material(hover_highlight_color())),
    });
}

/// Keeps one presentation-only hover slab in lockstep with the latest camera pick.
pub fn sync_hover_highlight(
    mut commands: Commands,
    picked: Res<crate::pick::PickedTile>,
    assets: Option<Res<ProjectionAssets>>,
    highlights: Query<(BevyEntity, &HoverHighlight)>,
) {
    if let Some(cell) = picked.0 {
        let position = cell.tile;
        let normal = cell.face.normal();
        if let Some((entity, _)) = highlights.iter().next() {
            commands.entity(entity).insert((
                HoverHighlight(position),
                Transform::from_translation(world_to_render(position) + normal * 0.55)
                    .with_rotation(bevy::prelude::Quat::from_rotation_arc(Vec3::Y, normal)),
            ));
        } else if let Some(assets) = assets {
            commands.spawn((
                HoverHighlight(position),
                Transform::from_translation(world_to_render(position) + normal * 0.55)
                    .with_rotation(bevy::prelude::Quat::from_rotation_arc(Vec3::Y, normal)),
                Mesh3d(assets.mark_mesh.clone()),
                MeshMaterial3d(assets.hover_highlight.clone()),
                ClientLocal,
            ));
        }
    } else {
        for (entity, _) in highlights.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Rebuilds the deliberately small preview set from the same single-z rect helper used on wire.
pub fn sync_drag_preview(
    mut commands: Commands,
    anchor: Res<DragAnchor>,
    picked: Res<PickedTile>,
    mirror: Res<crate::ingest::MirrorResource>,
    slice: Res<SliceLevel>,
    assets: Option<Res<ProjectionAssets>>,
    previews: Query<BevyEntity, With<DragPreview>>,
) {
    for entity in &previews {
        commands.entity(entity).despawn();
    }
    let (Some(anchor), Some(release), Some(assets)) = (anchor.0, picked.tile(), assets) else {
        return;
    };
    let rect =
        client_core::rect_on_level((anchor[0], anchor[1]), (release[0], release[1]), anchor[2]);
    for x in rect.min[0]..=rect.max[0] {
        for y in rect.min[1]..=rect.max[1] {
            let tile = [x, y, anchor[2]];
            commands.spawn((
                DragPreview(tile),
                slab_transform([x, y, dig_mark_level(&mirror.0, tile, slice.level())], 0.54),
                Mesh3d(assets.mark_mesh.clone()),
                MeshMaterial3d(assets.hover_highlight.clone()),
                ClientLocal,
            ));
        }
    }
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
    slice: SliceLevel,
    rebuild_terrain: bool,
    dirty_tiles: &[[i32; 3]],
    projected: &DynamicProjectionQuery,
    designations: &Query<(BevyEntity, &ProjectedDesignation, &ProjectedDesignationKind)>,
    zones: &Query<(BevyEntity, &ProjectedZone)>,
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
        let positions = terrain_positions_at(mirror, slice.level());
        // The draw-set oracle instrument: the shipped seed must report 53,365 (AC13).
        println!(
            "projected {} terrain cubes at z {}",
            positions.len(),
            slice.level()
        );
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
            if is_visible_at_slice(mirror, position, slice.level()) {
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
            if position[2] <= slice.level() && matches!(mirror.tile(*position), Some(Tile::Empty)) {
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
        .filter(|entity| entity.pos[2] <= slice.level())
        .map(|entity| (entity.id, (entity.pos, Some(*entity))))
        .collect();
    let visible_items: Vec<_> = mirror
        .items()
        .filter(|item| item.pos[2] <= slice.level())
        .map(|item| (item.id, item.pos))
        .collect();
    let item_ids: std::collections::BTreeSet<_> = visible_items.iter().map(|(id, _)| *id).collect();
    wanted.extend(visible_items.iter().map(|(id, pos)| (*id, (*pos, None))));
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
                        Transform::from_translation(item_translation(position))
                            .with_scale(Vec3::splat(STONE_ITEM_SCALE)),
                    ));
                }
            }
        }
    }

    let wanted_designations = mirror
        .designations()
        .iter()
        .filter(|designation| designation.pos[2] <= slice.level())
        .map(|designation| (designation.pos, designation.kind))
        .collect::<std::collections::BTreeMap<_, _>>();
    let wanted_zones = mirror
        .zones()
        .iter()
        .filter(|zone| zone.pos[2] <= slice.level())
        .map(|zone| zone.pos)
        .collect::<BTreeSet<_>>();
    // A zone slab and a designation slab can land on the SAME surface two ways, and both are
    // reachable from the sim. (1) A channel and a stockpile may occupy the same standable air
    // tile. (2) A stockpile at z sits on rock whose DIG mark is at z-1, and a dig slab resting on
    // its own top face is that same surface — measured at the 2026-08-21 review, designation
    // [9,9,9] and zone [9,9,10] both projected to (9.000, 9.540, -9.000) at identical scale with
    // the same mesh, both opaque. The story's own recipe hits case (2): the stockpile columns sit
    // inside the dig rect, so the stockpile tiles would z-fight against the digs beneath them
    // exactly while AC5 ("is a stockpile tellable from a dig") is being judged.
    let zone_mark_overlaps = wanted_zones
        .iter()
        .filter(|position| {
            wanted_designations.get(*position) == Some(&DesignationKind::Channel)
                || wanted_designations.get(&[position[0], position[1], position[2] - 1])
                    == Some(&DesignationKind::Dig)
        })
        .copied()
        .collect::<BTreeSet<_>>();
    let existing_designations = designations
        .iter()
        .map(|(entity, mark, kind)| (mark.0, (entity, kind.0)))
        .collect::<std::collections::BTreeMap<_, _>>();
    for (&position, &(entity, _)) in &existing_designations {
        if !wanted_designations.contains_key(&position) {
            commands.entity(entity).despawn();
        }
    }
    for (&position, &kind) in &wanted_designations {
        if let Some(&(entity, existing_kind)) = existing_designations.get(&position) {
            commands.entity(entity).insert(designation_mark_transform(
                mirror,
                position,
                kind,
                slice.level(),
            ));
            if existing_kind != kind {
                commands
                    .entity(entity)
                    .insert(ProjectedDesignationKind(kind));
            }
            if let Some(assets) = assets {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(assets.designation_material(kind)));
            }
        } else {
            let mut entity = commands.spawn((
                ProjectedDesignation(position),
                ProjectedDesignationKind(kind),
                designation_mark_transform(mirror, position, kind, slice.level()),
            ));
            if let Some(assets) = assets {
                entity.insert((
                    Mesh3d(assets.mark_mesh.clone()),
                    MeshMaterial3d(assets.designation_material(kind)),
                ));
            }
        }
    }

    let existing_zones = zones
        .iter()
        .map(|(entity, mark)| (mark.0, entity))
        .collect::<BTreeMap<_, _>>();
    for (&position, &entity) in &existing_zones {
        if !wanted_zones.contains(&position) {
            commands.entity(entity).despawn();
        }
    }
    for position in wanted_zones {
        let transform = zone_mark_transform(position, zone_mark_overlaps.contains(&position));
        if let Some(&entity) = existing_zones.get(&position) {
            commands.entity(entity).insert(transform);
        } else {
            let mut entity = commands.spawn((ProjectedZone(position), transform));
            if let Some(assets) = assets {
                entity.insert((
                    Mesh3d(assets.mark_mesh.clone()),
                    MeshMaterial3d(assets.zone_mark.clone()),
                ));
            }
        }
    }
}

/// Where a stone item is drawn: the tile centre, dropped onto the tile floor. Both the spawn and
/// `blend_entities` call this so the two cannot drift apart.
fn item_translation(position: [i32; 3]) -> Vec3 {
    world_to_render(position) + Vec3::new(0.0, STONE_ITEM_DROP, 0.0)
}

/// The z a DIG slab is drawn at. A dig marks the top face of its own tile, but
/// `is_visible_at_slice` draws every solid or ramp tile AT the cut as a full cube regardless of
/// exposure, and that cube spans `[z - 0.5, z + 0.5]` — so a dig with rock directly above it was
/// sealed inside opaque geometry. This is the STEADY STATE, not an edge case: the dwarves dig the
/// reachable tiles first, and reachable means open sky above, so the marks that survive a capture
/// window are exactly the buried ones. Measured at the 2026-08-21 review on this story's own
/// recipe: 25 of 79 visible at t+2, 9 at t+46, 2 at t+64, and 0 of 50 from t+120 onward — while
/// the instrument correctly printed `designations=50`, because all 50 were projected.
///
/// RULED 2026-08-21 (Wolf): promote a buried dig to the top face of the rock covering it, so the
/// order stays readable from the surface the boss is actually looking at.
///
/// NOTE: only the CONTIGUOUS drawn column directly above the dig is walked. A dig under a gap is
/// left where it is — it is already visible through that gap, and hoisting it would put the mark
/// on rock that is not the rock it marks.
fn dig_mark_level(mirror: &Mirror, position: [i32; 3], level: i32) -> i32 {
    let [x, y, z] = position;
    let mut top = z;
    while top < level && is_visible_at_slice(mirror, [x, y, top + 1], level) {
        top += 1;
    }
    top
}

fn designation_mark_transform(
    mirror: &Mirror,
    position: [i32; 3],
    kind: DesignationKind,
    level: i32,
) -> Transform {
    match kind {
        DesignationKind::Dig => {
            let [x, y, _] = position;
            slab_transform([x, y, dig_mark_level(mirror, position, level)], 0.54)
        }
        DesignationKind::Channel => slab_transform(position, -0.46),
    }
}

fn zone_mark_transform(position: [i32; 3], overlaps_mark: bool) -> Transform {
    if !overlaps_mark {
        return slab_transform(position, -0.46);
    }
    // Raise and inset the zone only where another mark shares its surface — a channel in the same
    // air tile, or a dig on the rock directly beneath it — so the zone's neutral centre and the
    // other mark's cold rim both stay readable instead of z-fighting at one opaque surface.
    slab_transform(position, -0.36).with_scale(Vec3::new(
        MARK_FOOTPRINT_SCALE * 0.72,
        MARK_FOOTPRINT_SCALE,
        MARK_FOOTPRINT_SCALE * 0.72,
    ))
}

fn slab_transform(position: [i32; 3], vertical_offset: f32) -> Transform {
    Transform::from_translation(world_to_render(position) + Vec3::Y * vertical_offset)
        .with_scale(Vec3::splat(MARK_FOOTPRINT_SCALE))
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
            // Must go through `item_translation` for the same reason the spawn does: this is the
            // sole writer of translation after spawn, so a bare `world_to_render` here would lift
            // every item back off the tile floor on the frame after it appeared.
            transform.translation = item_translation(*position);
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

/// Foliage is DRAWN at 0.62-0.95 of its cell (`foliage_scale`) so a crown reads as sparse
/// branches rather than a block. The pick marches full unit cells, so treating foliage as
/// pickable would claim the ~62% of the cell face the player is plainly seeing through, and
/// would occlude whatever is behind it. The pick excludes it so pick geometry and drawn
/// geometry agree.
pub(crate) fn is_tree_foliage(mirror: &Mirror, position: [i32; 3]) -> bool {
    // NOTE: matched directly rather than through `terrain_material_at`, whose exact expression
    // is story 5.4's sabotage anchor for the snow-cap swap. Sharing it made that row ambiguous
    // and it stopped applying — a row that cannot apply pins nothing.
    matches!(
        mirror.tile(position),
        Some(Tile::Solid(Material::TreeFoliage) | Tile::Ramp(Material::TreeFoliage))
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
/// NOTE: soil is excluded because soil is never the natural surface here — the world's skin is
/// snow and ice, and soil only becomes sky-exposed when something DUG the tile above it. Capping
/// it drew fresh snow (146,158,184) on the trench floor, brighter than the snow it replaced
/// (136,150,178), so an excavation erased itself the moment it finished. Measured before the
/// change: 1,016 soil tiles are already exposed at boot and NONE of them carried a cap (they all
/// have solid rock above), so this alters exactly 0 of the 5,716 caps in the approved boot frame.
pub fn has_snow_cap(mirror: &Mirror, position: [i32; 3]) -> bool {
    matches!(
        mirror.tile(position),
        Some(Tile::Solid(material) | Tile::Ramp(material))
            if material != Material::Ice
                && material != Material::TreeFoliage
                && material != Material::Soil
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

    fn designation_material(&self, kind: DesignationKind) -> Handle<StandardMaterial> {
        match kind {
            DesignationKind::Dig => self.dig_mark.clone(),
            DesignationKind::Channel => self.channel_mark.clone(),
        }
    }
}

pub fn terrain_positions(mirror: &Mirror) -> Vec<[i32; 3]> {
    terrain_positions_at(mirror, mirror.dims().z.saturating_sub(1) as i32)
}

/// The client-local draw set at a slice: retain full-depth exposure, then add the terrain floor
/// at the selected z. The latter arm is what makes a cut a filled cross-section rather than a
/// hollow shell; `is_exposed` remains the full-depth rule for ramps and the existing oracle.
pub fn terrain_positions_at(mirror: &Mirror, level: i32) -> Vec<[i32; 3]> {
    let level = level.clamp(0, mirror.dims().z.saturating_sub(1) as i32);
    let mut positions = Vec::new();
    for_each_position(mirror.dims(), |position| {
        if is_visible_at_slice(mirror, position, level) {
            positions.push(position);
        }
    });
    positions
}

/// Whether any solid or ramp tile sits strictly above `level`. This is what makes the readout's
/// surface/underground claim true — `level == top` only says where the cut is, never whether
/// anything covers it. Scans top-down and returns on the first hit, so the common case (rock
/// directly overhead) costs one lookup.
pub fn has_terrain_above(mirror: &Mirror, level: i32) -> bool {
    let dims = mirror.dims();
    let top = dims.z.saturating_sub(1) as i32;
    for z in ((level + 1)..=top).rev() {
        for y in 0..dims.y as i32 {
            for x in 0..dims.x as i32 {
                if matches!(mirror.tile([x, y, z]), Some(Tile::Solid(_) | Tile::Ramp(_))) {
                    return true;
                }
            }
        }
    }
    false
}

pub(crate) fn is_visible_at_slice(mirror: &Mirror, position: [i32; 3], level: i32) -> bool {
    position[2] <= level
        && (is_exposed(mirror, position)
            || (position[2] == level
                && matches!(mirror.tile(position), Some(Tile::Solid(_) | Tile::Ramp(_)))))
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

    /// The invariant Wolf's eye caught on the vehicle and no instrument could: a stone item must
    /// not swallow the debris chips that share its tile. At the scale-1.0 the item branch used to
    /// inherit, every chip lies inside the item's own volume and AC8's debris is unseeable.
    ///
    /// The item's own size is pinned in the headless test, against a hand-written literal read
    /// back off the spawned entity — asserting `scale == STONE_ITEM_SCALE` here would hold for ANY
    /// value including the 1.0 that caused the defect. This is the fourth story bitten by that shape.
    #[test]
    fn a_stone_item_never_encloses_its_chips() {
        // Every chip must lie OUTSIDE the item's own volume, on some axis, or it is invisible
        // wherever an item stands.
        let half = STONE_ITEM_SCALE / 2.0;
        for offset in chip_offsets() {
            let drop = Vec3::new(0.0, STONE_ITEM_DROP, 0.0);
            let separation = offset - drop;
            assert!(
                separation.x.abs() > half || separation.y.abs() > half || separation.z.abs() > half,
                "chip at {offset:?} sits inside the item cube and can never be seen"
            );
        }
    }

    /// The item must rest on the tile floor, not float at its centre — and the blend, which is the
    /// sole writer of translation after spawn, must agree with the spawn about where that is.
    #[test]
    fn an_item_rests_on_the_tile_floor_at_spawn_and_after_the_blend() {
        let position = [3, 4, 5];
        let resting = world_to_render(position).y - 0.3;
        assert!((item_translation(position).y - resting).abs() < 1e-6);
        assert_eq!(item_translation(position).x, world_to_render(position).x);
        assert_eq!(item_translation(position).z, world_to_render(position).z);
    }

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
        // CHANGED 2026-08-18 from "soil can carry snow" (5.4's rule) at Wolf's second live
        // viewing: soil is never the natural surface on this seed, so a sky-exposed soil tile is
        // excavated ground, and capping it drew snow brighter than the tile that was dug away.
        // NOTE: stone has the same latent shape, but no named dig site exposes it, so the rule
        // stays narrow rather than general.
        assert!(
            !has_snow_cap(&toy, [5, 0, 0]),
            "sky-exposed soil is dug ground and must stay bare"
        );
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
