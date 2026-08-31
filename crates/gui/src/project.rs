use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

use bevy::prelude::{
    Assets, Commands, Component, Cuboid, Entity as BevyEntity, Handle, Mesh, Mesh3d,
    MeshMaterial3d, Or, PointLight, Query, Res, ResMut, Resource, StandardMaterial, Transform,
    Vec3, With, Without,
};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
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
    designate::{DesignateMode, DragAnchor, DragMode, designation_target},
    pick::{PickedCell, PickedTile},
    slice::SliceLevel,
    transform::{world_point_to_render, world_to_render, world_vector_to_render},
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

/// A material/rim partition of one 16-cell terrain chunk, present only for `--subdiv N` where
/// N is greater than one. The default and `--subdiv 1` retain `TerrainTile` exactly.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainChunk(pub [i32; 3]);

/// An opt-in visual terrain subdivision. The client mirror remains authoritative; this only
/// changes how its terrain is drawn.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainSubdivision(pub u32);

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
    Or<(With<TerrainTile>, With<TerrainChunk>, With<SnowCap>)>,
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

#[derive(Resource, Default)]
/// The cells the preview currently covers, cached so an unchanged drag does not respawn its
/// slabs every frame. Cells rather than a rect since the standable modes follow the ground and
/// their footprint is no longer a single-z box.
pub struct DragPreviewCells(Option<Vec<[i32; 3]>>);

/// Leaves a visible gutter between neighbouring mark slabs. The mesh is 1.02 wide and this scale
/// is applied on top of it, so a slab covers 1.02 x 0.94 = 0.9588 of its tile — inset ~2% per
/// side, which is the gutter, not a reach to the tile edge.
///
/// NOTE: measured at the 2026-08-21 review, one tile step spans 48.8 px of a 1280-wide frame at
/// `--distance 30`, so the gutter is ~2 px at the working zoom and ~0.65 px at the boot vista.
/// Whether that reads as separate tiles or as anti-aliasing noise is a human call, and it is on
/// the list for Wolf's live viewing.
const MARK_FOOTPRINT_SCALE: f32 = 0.94;
const TERRAIN_CHUNK_EDGE: i32 = 16;
const DETAIL_SEED: u32 = 0xF005_7E1A;

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
#[allow(clippy::too_many_arguments)]
pub fn sync_drag_preview(
    mut commands: Commands,
    anchor: Res<DragAnchor>,
    drag_mode: Res<DragMode>,
    picked: Res<PickedTile>,
    mirror: Res<crate::ingest::MirrorResource>,
    slice: Res<SliceLevel>,
    assets: Option<Res<ProjectionAssets>>,
    previews: Query<BevyEntity, With<DragPreview>>,
    mut preview_cells_cache: ResMut<DragPreviewCells>,
) {
    let (Some(anchor), Some(assets)) = (anchor.0, assets) else {
        if preview_cells_cache.0.take().is_some() {
            for entity in &previews {
                commands.entity(entity).despawn();
            }
        }
        return;
    };
    // A drag stays live across frames whose ray misses terrain — cursor over sky, over a gap,
    // past the world edge. Despawning the whole preview on those frames left the boss dragging
    // BLIND while the drag went on committing on release; keep the last rect standing and let the
    // next frame that hits something update it.
    let Some(release) = picked.0 else {
        return;
    };
    let mode = drag_mode.0.unwrap_or(DesignateMode::None);
    // The preview is built from THE SAME functions the release path sends, so what is on screen
    // is what goes on the wire — the property that was missing when two inert modes previewed a
    // full rect and designated nothing.
    let cells = preview_cells(&mirror.0, slice.level(), anchor, release, mode);
    if preview_cells_cache.0.as_deref() == Some(cells.as_slice()) {
        return;
    }
    for entity in &previews {
        commands.entity(entity).despawn();
    }
    for tile in &cells {
        let (transform, material) =
            preview_appearance(mode, &mirror.0, *tile, slice.level(), &assets);
        commands.spawn((
            DragPreview(*tile),
            transform,
            Mesh3d(assets.mark_mesh.clone()),
            MeshMaterial3d(material),
            ClientLocal,
        ));
    }
    preview_cells_cache.0 = Some(cells);
}

/// Exactly the cells the release will designate, for the mode that will commit.
///
/// Dig and clear keep AC4's single-z rect at the cells the ray hit; channel and stockpile follow
/// the ground. Both branches then drop whatever the sim would refuse, so the preview never
/// promises a mark that cannot appear.
fn preview_cells(
    mirror: &Mirror,
    level: i32,
    anchor: PickedCell,
    release: PickedCell,
    mode: DesignateMode,
) -> Vec<[i32; 3]> {
    match mode {
        DesignateMode::Channel | DesignateMode::Stockpile => client_core::surface_targets(
            mirror,
            level,
            designation_target(mirror, anchor, mode),
            designation_target(mirror, release, mode),
        ),
        _ => {
            let rect = client_core::rect_on_level(
                (anchor.tile[0], anchor.tile[1]),
                (release.tile[0], release.tile[1]),
                anchor.tile[2],
            );
            (rect.min[1]..=rect.max[1])
                .flat_map(|y| (rect.min[0]..=rect.max[0]).map(move |x| [x, y, rect.min[2]]))
                .filter(|tile| sim_will_keep(mirror, *tile, mode))
                .collect()
        }
    }
}

/// Whether the sim would KEEP a designation of this mode at this cell.
///
/// Mirrors the two filters in `sim-core`'s command handling: dig keeps `Tile::Solid`, channel and
/// stockpile keep standable cells, and both drop the remainder in silence.
fn sim_will_keep(mirror: &Mirror, tile: [i32; 3], mode: DesignateMode) -> bool {
    match mode {
        DesignateMode::Dig => matches!(mirror.tile(tile), Some(Tile::Solid(_))),
        DesignateMode::Channel | DesignateMode::Stockpile => {
            client_core::is_standable(mirror, tile)
        }
        // Clear removes rather than designates; there is nothing for the sim to filter.
        DesignateMode::Clear | DesignateMode::None => true,
    }
}

/// Where a pending drag tile will sit, and what it will look like, IN THE MODE THAT WILL COMMIT.
/// Task 4 requires the preview to sit where the committed marks will sit, so each arm mirrors the
/// matching arm of `designation_mark_transform` / `zone_mark_transform` rather than assuming dig.
/// Clear commits nothing, so it keeps the neutral hover material at the dig height.
fn preview_appearance(
    mode: DesignateMode,
    mirror: &Mirror,
    tile: [i32; 3],
    level: i32,
    assets: &ProjectionAssets,
) -> (Transform, Handle<StandardMaterial>) {
    let [x, y, _] = tile;
    match mode {
        DesignateMode::Dig => (
            slab_transform([x, y, dig_mark_level(mirror, tile, level)], 0.54),
            assets.dig_mark.clone(),
        ),
        DesignateMode::Channel => (
            channel_slab(mirror, tile, level),
            assets.channel_mark.clone(),
        ),
        DesignateMode::Stockpile => (slab_transform(tile, -0.46), assets.zone_mark.clone()),
        DesignateMode::None | DesignateMode::Clear => (
            slab_transform([x, y, dig_mark_level(mirror, tile, level)], 0.54),
            assets.hover_highlight.clone(),
        ),
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
        // NOTE: only the campfire casts point-light shadows; each additional emitter costs six
        // cube-map faces, and this story's measured defect is confined to the campfire.
        shadow_maps_enabled: matches!(kind, protocol::LightKind::Campfire),
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SubdividedTerrainStats {
    entities: usize,
    chunks: usize,
    triangles: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MeshKey {
    chunk: [i32; 3],
    slot: usize,
    rim: usize,
    axis: usize,
    sign: i32,
    plane: i32,
}

#[derive(Default)]
struct ChunkMesh {
    masks: BTreeMap<MeshKey, BTreeSet<(i32, i32)>>,
}

/// Builds the opt-in fine terrain from the client mirror. The coarse cells are still the only
/// authority: every fine face begins with the same visible-cell set the shipped cube path uses.
///
/// NOTE: the small deterministic top pits are a measurement stand-in for 10.4's authored terrain
/// look, not a visual decision. They make `--subdiv N` measure non-flat fine surfaces rather than
/// merely tessellating an otherwise identical plane.
fn spawn_subdivided_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &ProjectionAssets,
    mirror: &Mirror,
    positions: &[[i32; 3]],
    subdiv: u32,
) -> SubdividedTerrainStats {
    let subdiv = i32::try_from(subdiv).expect("--subdiv fits signed mesh coordinates");
    let visible = positions.iter().copied().collect::<BTreeSet<_>>();
    let mut chunks = BTreeMap::<[i32; 3], ChunkMesh>::new();
    let mut foliage_entities = 0;
    for &position in positions {
        if is_tree_foliage(mirror, position) {
            // Foliage is intentionally non-cubic presentation geometry. Keep its shipped path
            // rather than pretending a greedy cuboid preserves the sparse crown silhouette.
            let entity = commands
                .spawn((
                    WorldProjected(terrain_id(position, mirror.dims())),
                    TerrainTile(position),
                    terrain_transform(mirror, position),
                ))
                .id();
            commands.entity(entity).insert((
                Mesh3d(assets.cube.clone()),
                MeshMaterial3d(assets.terrain_material(mirror, position)),
            ));
            foliage_entities += 1;
            continue;
        }
        let chunk = [
            position[0] / TERRAIN_CHUNK_EDGE,
            position[1] / TERRAIN_CHUNK_EDGE,
            position[2] / TERRAIN_CHUNK_EDGE,
        ];
        let slot = terrain_slot_at(mirror, position) as usize;
        let rim = rim_level(position, mirror.dims());
        for delta in NEIGHBOURS {
            let axis = delta
                .iter()
                .position(|coordinate| *coordinate != 0)
                .expect("each terrain neighbour changes one axis");
            let neighbour = [
                position[0] + delta[0],
                position[1] + delta[1],
                position[2] + delta[2],
            ];
            if visible.contains(&neighbour) {
                continue;
            }
            let sign = delta[axis];
            let plane = (position[axis] + i32::from(sign > 0)) * subdiv;
            let mesh = chunks.entry(chunk).or_default();
            for du in 0..subdiv {
                for dv in 0..subdiv {
                    let u = position[(axis + 1) % 3] * subdiv + du;
                    let v = position[(axis + 2) % 3] * subdiv + dv;
                    let top_depth = if axis == 2 && sign > 0 {
                        detail_depth(plane, u, v, subdiv)
                    } else {
                        0
                    };
                    let key = MeshKey {
                        chunk,
                        slot,
                        rim,
                        axis,
                        sign,
                        plane: plane - top_depth,
                    };
                    mesh.masks.entry(key).or_default().insert((u, v));
                    if axis == 2 && sign > 0 {
                        add_detail_connectors(mesh, key, position, du, dv, subdiv);
                    }
                }
            }
        }
    }

    let mut stats = SubdividedTerrainStats::default();
    stats.entities = foliage_entities;
    for (chunk, chunk_mesh) in chunks {
        let mut material_meshes = BTreeMap::<(usize, usize), MeshBuilder>::new();
        for (key, mask) in chunk_mesh.masks {
            let builder = material_meshes.entry((key.slot, key.rim)).or_default();
            greedy_mask_into_mesh(builder, key, &mask, subdiv);
        }
        for ((slot, rim), builder) in material_meshes {
            if builder.indices.is_empty() {
                continue;
            }
            stats.triangles += builder.indices.len() / 3;
            stats.entities += 1;
            let mesh = meshes.add(builder.finish());
            commands.spawn((
                TerrainChunk(chunk),
                Mesh3d(mesh),
                MeshMaterial3d(assets.slot(TERRAIN_SLOTS[slot], rim)),
            ));
        }
        stats.chunks += 1;
    }
    for &position in positions {
        if has_snow_cap(mirror, position) {
            spawn_snow_cap(commands, assets, mirror, position);
            stats.entities += 1;
        }
    }
    stats
}

fn terrain_slot_at(mirror: &Mirror, position: [i32; 3]) -> TerrainSlot {
    if has_snow_laden_crown(mirror, position) {
        TerrainSlot::FoliageCrown
    } else {
        TerrainSlot::of(terrain_material(mirror, position))
    }
}

/// Hash-compatible with the measurement instrument's small value-noise rule.
fn detail_depth(plane: i32, u: i32, v: i32, subdiv: i32) -> i32 {
    let mut value = DETAIL_SEED
        ^ (plane as u32).wrapping_mul(0x9E37_79B1)
        ^ (u as u32).wrapping_mul(0x85EB_CA77)
        ^ (v as u32).wrapping_mul(0xC2B2_AE3D);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    let offset = (value % 5) as i32 - 2;
    offset.abs().min(subdiv - 1)
}

fn add_detail_connectors(
    mesh: &mut ChunkMesh,
    material_key: MeshKey,
    position: [i32; 3],
    du: i32,
    dv: i32,
    subdiv: i32,
) {
    let plane = (position[2] + 1) * subdiv;
    let depth = detail_depth(
        plane,
        position[0] * subdiv + du,
        position[1] * subdiv + dv,
        subdiv,
    );
    for (step_u, step_v, axis) in [(1, 0, 0), (0, 1, 1)] {
        if du + step_u >= subdiv || dv + step_v >= subdiv {
            continue;
        }
        let other = detail_depth(
            plane,
            position[0] * subdiv + du + step_u,
            position[1] * subdiv + dv + step_v,
            subdiv,
        );
        if depth == other {
            continue;
        }
        let lower = plane - depth.max(other);
        let higher = plane - depth.min(other);
        let sign = if depth < other { 1 } else { -1 };
        let face_plane = if axis == 0 {
            position[0] * subdiv + du + 1
        } else {
            position[1] * subdiv + dv + 1
        };
        let (u, v) = if axis == 0 {
            (position[1] * subdiv + dv, lower)
        } else {
            (lower, position[0] * subdiv + du)
        };
        for offset in 0..higher - lower {
            let coordinate = if axis == 0 {
                (u, v + offset)
            } else {
                (u + offset, v)
            };
            mesh.masks
                .entry(MeshKey {
                    chunk: material_key.chunk,
                    slot: material_key.slot,
                    rim: material_key.rim,
                    axis,
                    sign,
                    plane: face_plane,
                })
                .or_default()
                .insert(coordinate);
        }
    }
}

#[derive(Default)]
struct MeshBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    fn finish(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

fn greedy_mask_into_mesh(
    builder: &mut MeshBuilder,
    key: MeshKey,
    mask: &BTreeSet<(i32, i32)>,
    subdiv: i32,
) {
    let mut used = BTreeSet::new();
    for &(u, v) in mask {
        if used.contains(&(u, v)) {
            continue;
        }
        let mut width = 1;
        while mask.contains(&(u + width, v)) && !used.contains(&(u + width, v)) {
            width += 1;
        }
        let mut height = 1;
        while (0..width).all(|offset| {
            mask.contains(&(u + offset, v + height)) && !used.contains(&(u + offset, v + height))
        }) {
            height += 1;
        }
        for du in 0..width {
            for dv in 0..height {
                used.insert((u + du, v + dv));
            }
        }
        append_quad(builder, key, u, v, width, height, subdiv);
    }
}

fn append_quad(
    builder: &mut MeshBuilder,
    key: MeshKey,
    u: i32,
    v: i32,
    width: i32,
    height: i32,
    subdiv: i32,
) {
    let point = |u, v| {
        let mut coordinate = [0; 3];
        coordinate[key.axis] = key.plane;
        coordinate[(key.axis + 1) % 3] = u;
        coordinate[(key.axis + 2) % 3] = v;
        world_point_to_render(Vec3::new(
            coordinate[0] as f32 / subdiv as f32 - 0.5,
            coordinate[1] as f32 / subdiv as f32 - 0.5,
            coordinate[2] as f32 / subdiv as f32 - 0.5,
        ))
    };
    let corners = [
        point(u, v),
        point(u + width, v),
        point(u + width, v + height),
        point(u, v + height),
    ];
    let normal_world = match key.axis {
        0 => Vec3::X,
        1 => Vec3::Y,
        2 => Vec3::Z,
        _ => unreachable!("mesh axes are limited to x, y, z"),
    } * key.sign as f32;
    let normal = world_vector_to_render(normal_world).to_array();
    let base = u32::try_from(builder.positions.len()).expect("terrain mesh index fits u32");
    let order = if key.sign > 0 {
        [0, 3, 2, 1]
    } else {
        [0, 1, 2, 3]
    };
    for index in order {
        builder.positions.push(corners[index].to_array());
        builder.normals.push(normal);
    }
    builder
        .indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
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
    meshes: Option<&mut Assets<Mesh>>,
    subdivision: Option<&TerrainSubdivision>,
) {
    if rebuild_terrain {
        for (entity, _) in chips.iter() {
            commands.entity(entity).despawn();
        }
        for (entity, _, _) in terrain.iter() {
            commands.entity(entity).despawn();
        }
        let positions = terrain_positions_at(mirror, slice.level());
        // The draw-set oracle instrument (AC13). The shipped seed reports 44,984 after story
        // 9.4; it was 53,365 before it, 45,261 between its two halves. This number tracks world
        // CONTENT -- read it as "did the rim or a slice silently drop tiles?", never as a fixed
        // constant. It moved twice in one story, which is the whole argument.
        let started = Instant::now();
        let subdiv = subdivision.map_or(1, |subdivision| subdivision.0);
        if subdiv > 1 {
            let Some((assets, meshes)) = assets.zip(meshes) else {
                return;
            };
            let stats =
                spawn_subdivided_terrain(commands, meshes, assets, mirror, &positions, subdiv);
            println!(
                "subdiv {subdiv}: entities={} chunks={} triangles={} mesh_build_ms={}",
                stats.entities,
                stats.chunks,
                stats.triangles,
                started.elapsed().as_millis()
            );
        } else {
            // The control path is deliberately kept verbatim: no flag and `--subdiv 1` retain
            // the shipped one-entity-per-cell scene, including its snow caps and pick markers.
            println!(
                "projected {} terrain cubes at z {}",
                positions.len(),
                slice.level()
            );
            for position in positions.iter().copied() {
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
            if subdivision.is_some() {
                let snow_caps = positions
                    .iter()
                    .filter(|position| has_snow_cap(mirror, **position))
                    .count();
                println!(
                    "subdiv 1: entities={} chunks=0 triangles={} mesh_build_ms={}",
                    positions.len() + snow_caps,
                    (positions.len() + snow_caps) * 12,
                    started.elapsed().as_millis()
                );
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
        DesignationKind::Channel => channel_slab(mirror, position, level),
    }
}

/// A channel mark sits at the BOTTOM of its air cell — the top face of the rock it will turn into
/// a ramp. That is right until something is drawn above it, and then the slab is sealed inside
/// opaque geometry, exactly as buried digs were before 7.2 fixed them. The instruments cannot see
/// it: a slab inside rock is projected and counted like any other, and 7.2 measured 0 of 50 marks
/// visible while the count correctly read 50. Dig got that fix; channel never did. This is it.
fn channel_slab(mirror: &Mirror, position: [i32; 3], level: i32) -> Transform {
    let [x, y, z] = position;
    let top = dig_mark_level(mirror, position, level);
    if top == z {
        slab_transform(position, -0.46)
    } else {
        slab_transform([x, y, top], 0.54)
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
///
/// RULED 2026-08-29 (Wolf, at 9.4's review): the sky-exposure test ALONE did exactly what the line
/// above says it was chosen to avoid. `place_trees` stamps a foliage ring at `surface + 1`, and
/// those skirt cells have open sky beside the trunk, so 1,246 of 1,824 ground-level skirt cells
/// (68 %) were taking the bright crown colour — a lit ring sitting ON the ground around every
/// trunk, which is what "snow cover" read wrong as. Snow now additionally requires the cell NOT to
/// rest directly on the ground: a crown sits on more tree, never on terrain.
pub fn has_snow_laden_crown(mirror: &Mirror, position: [i32; 3]) -> bool {
    terrain_material_at(mirror, position) == Some(Material::TreeFoliage)
        && !matches!(
            mirror.tile([position[0], position[1], position[2] + 1]),
            Some(Tile::Solid(_) | Tile::Ramp(_))
        )
        && !rests_on_the_ground(mirror, position)
}

/// Whether the cell directly below is terrain rather than tree. Foliage resting on the ground is a
/// SKIRT, not a crown — see `has_snow_laden_crown`.
fn rests_on_the_ground(mirror: &Mirror, position: [i32; 3]) -> bool {
    matches!(
        terrain_material_at(mirror, [position[0], position[1], position[2] - 1]),
        Some(Material::Stone | Material::Soil | Material::Ice | Material::Snow)
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

    /// Wolf's ruling at 9.4's review: sky exposure alone put snow on the SKIRT. `place_trees`
    /// stamps a foliage ring at `surface + 1` whose cells have open sky beside the trunk, so
    /// 1,246 of 1,824 ground-level skirt cells were rendering at the bright crown colour — a lit
    /// ring around the base of every tree, which is the "bright slab on every ground-level skirt
    /// tile" the predicate's own doc comment says the material swap exists to avoid. Measured on
    /// the shipped world after this fix: ground-resting bright cells 1,029 -> 0.
    #[test]
    fn foliage_resting_on_the_ground_is_a_skirt_and_never_catches_snow() {
        // THE SKIRT CELL MUST HAVE OPEN SKY. A first draft of this fixture stacked foliage
        // directly above it, so the pre-existing sky-exposure clause returned false first and the
        // assertion passed without ever reaching the ground-rest clause — vacuous, and the
        // sabotage row caught it. Column x=0 is the skirt (on terrain, open sky); column x=1 is a
        // real crown (on foliage, open sky).
        let sapling = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 4 },
            tiles: vec![
                Tile::Solid(Material::Snow),
                Tile::Solid(Material::Snow),
                Tile::Solid(Material::TreeFoliage),
                Tile::Solid(Material::TreeFoliage),
                Tile::Empty,
                Tile::Solid(Material::TreeFoliage),
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
            !has_snow_laden_crown(&sapling, [0, 0, 1]),
            "foliage sitting directly on terrain is a skirt, not a crown, however open the sky \
             beside it"
        );
        assert!(
            has_snow_laden_crown(&sapling, [1, 0, 2]),
            "foliage standing on more foliage with open sky above it is still a crown"
        );
    }
}
