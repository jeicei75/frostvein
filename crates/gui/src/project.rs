use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

use bevy::prelude::{
    AssetServer, Assets, Commands, Component, Cuboid, Entity as BevyEntity, Handle, Mesh, Mesh3d,
    MeshMaterial3d, Or, PointLight, Query, Res, ResMut, Resource, StandardMaterial, Transform,
    Vec3, With, Without,
};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    world_serialization::{WorldAsset, WorldAssetRoot},
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

/// One presentation mesh re-derived from a contiguous trunk column in the client mirror.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeMesh(pub [i32; 3]);

/// The coarse cells one chunk's meshes actually carry geometry for.
///
/// `--subdiv N > 1` collapses tens of thousands of `TerrainTile` entities into a handful of
/// chunk meshes, and the capture's draw-set oracle counts `TerrainTile`. It therefore saw 198
/// tiles where the mirror held 11,325 and panicked on a cut it had drawn correctly. This
/// records the same observation the oracle always made — which cells reached a mesh, taken
/// from the meshing loop's own output rather than from the list it was handed — in the one
/// place the fine path can still answer it.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct TerrainChunkCells(pub Vec<[i32; 3]>);

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
        Option<&'static TerrainChunk>,
    ),
    Or<(
        With<TerrainTile>,
        With<TerrainChunk>,
        With<TerrainChunkCells>,
        With<SnowCap>,
    )>,
>;

pub type TreeMeshQuery<'w, 's> = Query<'w, 's, (BevyEntity, &'static TreeMesh)>;

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
/// The largest `--subdiv` the client will attempt. Cost is O(N²) per exposed coarse face and
/// nothing bounded it: `--subdiv 3000000000` passed CLI validation and then panicked inside the
/// mesher, and `--subdiv 8` built silently for 17 seconds. The offline bench guards the same way
/// and names which resource ran out; this is that guard on the render side.
pub const MAX_SUBDIV: u32 = 16;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TreeVariant {
    Tree01,
    Tree02,
    Tree03,
    Tree04R,
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
    trees: [Handle<WorldAsset>; 4],
}

/// Scene paths in `TreeVariant` order, served from the `embedded://` source.
///
/// The bytes live in `ingest::TREE_ASSETS`; this is the order `tree_scene` indexes by, and
/// `ingest::tree_asset_paths_match_the_loader` pins the two together.
pub const TREE_SCENE_PATHS: [&str; 4] = [
    "trees/SM_VoxelPine_Tree01.glb",
    "trees/SM_VoxelPine_Tree02.glb",
    "trees/SM_VoxelPine_Tree03.glb",
    "trees/SM_VoxelPine_Tree04R.glb",
];

pub fn setup_projection_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Option<Res<AssetServer>>,
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
        trees: asset_server.map_or_else(
            || std::array::from_fn(|_| Handle::default()),
            |asset_server| {
                TREE_SCENE_PATHS.map(|path| asset_server.load(format!("embedded://{path}#Scene0")))
            },
        ),
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

impl ProjectionAssets {
    /// The materials that carry an emissive glow, paired with the light whose colour they take.
    ///
    /// Toggling a source off must black these too. The emissive is baked at spawn from
    /// `light_properties`, so a "campfire off" frame otherwise still shows the campfire GLOWING —
    /// the residual emitter Wolf found on the vehicle with every toggle off. A point light and an
    /// emissive face are two different things the same source owns; the instrument has to switch
    /// both or it answers the wrong question.
    pub fn emissive_materials(&self) -> [(protocol::LightKind, Handle<StandardMaterial>); 2] {
        [
            (protocol::LightKind::Torch, self.torch.clone()),
            (protocol::LightKind::Campfire, self.campfire.clone()),
        ]
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
    /// Coarse cells that reached a mesh. This is the number the shipped path reports as
    /// "projected N terrain cubes", kept comparable across both paths on purpose.
    cells: usize,
    /// Fine faces before the greedy merge. Triangles are partition-dependent -- chunk and rim
    /// splits inflate them -- so they cannot answer "is this the same surface as the bench's".
    /// The face count can: it is exactly what the offline oracle counts, and any hole in the
    /// fine surface shows up here as a shortfall.
    faces: usize,
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
    cells: BTreeSet<[i32; 3]>,
}

/// The coarse cell a run of fine faces belongs to, resolved once rather than per face.
struct FaceOwner {
    chunk: [i32; 3],
    slot: usize,
    rim: usize,
}

impl FaceOwner {
    fn new(mirror: &Mirror, position: [i32; 3]) -> Self {
        Self {
            chunk: chunk_of(position),
            slot: terrain_slot_at(mirror, position) as usize,
            rim: rim_level(position, mirror.dims()),
        }
    }
}

/// Whether a neighbouring cell hides the face that points at it.
///
/// The subdivided mesher asked the DRAWN set this question, which is a different question: a
/// solid cell that nothing exposes is absent from the draw set, so its neighbour emitted a face
/// buried inside the rock. On the real world that was 48,952 of 110,094 submitted faces — 44%,
/// in the very path built to demonstrate that a fine surface is cheaper than whole cubes. The
/// slice half matters too: nothing above the cut is drawn, so rock above it cannot occlude.
fn occludes(mirror: &Mirror, position: [i32; 3], level: i32) -> bool {
    position[2] <= level && matches!(mirror.tile(position), Some(Tile::Solid(_) | Tile::Ramp(_)))
}

/// Occlusion as the TERRAIN mesher must see it. A mesh-drawn tree cell is drawn by its own mesh
/// and is skipped outright by `build_chunk_meshes`, so it can hide nothing on the terrain's
/// behalf: counting it as an occluder drops a face that then NOTHING emits, and the camera sees
/// straight through to the sky. `occludes` alone answers only "is this cell solid", which is why
/// the hole survived -- and why it appeared only at `--subdiv > 1`, the coarse path drawing every
/// cell as a complete `Cuboid` that culls nothing.
fn occludes_terrain(mirror: &Mirror, position: [i32; 3], level: i32, cover: &TreeCover) -> bool {
    occludes(mirror, position, level) && !is_mesh_drawn_tree(mirror, position, cover)
}

/// Fine column heights inside one coarse cell, in fine voxels, or `None` for a full cube.
///
/// This is the same heightfield `scripts/bench/resolution_bench.py` measures: a cell whose top
/// is drawn carries the detail pits, every other cell is solid to its ceiling. Both sides pin
/// the same `detail_depth` vector, which is what makes "one rule" a test rather than a comment.
fn column_heights(
    mirror: &Mirror,
    position: [i32; 3],
    subdiv: i32,
    level: i32,
) -> Option<Vec<i32>> {
    let above = [position[0], position[1], position[2] + 1];
    // Foliage keeps the shipped whole-cube path, so it is never carved -- and therefore never
    // uncovers a neighbour either. Carving it emitted faces on the rock behind an opaque cube,
    // and made 9 tree trunks fully enclosed by their own crown look like exposed surface.
    // DELIBERATELY plain `occludes`, not `occludes_terrain`. `None` here means "do not carve,
    // draw the full cube", which is the hole-FREE answer. Routing this decision through the
    // tree-aware occluder made cells under a mesh tree carry detail pits instead of a solid top
    // and measured 1,370 NEW interior-sky pixels at the world edges against a 2-pixel noise
    // floor -- more holes, not fewer. The tree-aware occluder belongs on the face-EMISSION
    // decisions below, where suppressing a face is what leaves nothing drawn.
    if subdiv == 1 || occludes(mirror, above, level) || is_tree_foliage(mirror, position) {
        return None;
    }
    let plane = (position[2] + 1) * subdiv;
    let mut heights = Vec::with_capacity((subdiv * subdiv) as usize);
    for du in 0..subdiv {
        for dv in 0..subdiv {
            heights.push(
                subdiv
                    - detail_depth(
                        plane,
                        position[0] * subdiv + du,
                        position[1] * subdiv + dv,
                        subdiv,
                    ),
            );
        }
    }
    Some(heights)
}

fn column_height(heights: Option<&Vec<i32>>, du: i32, dv: i32, subdiv: i32) -> i32 {
    heights.map_or(subdiv, |heights| heights[(du * subdiv + dv) as usize])
}

#[allow(clippy::too_many_arguments)]
fn push_face(
    chunks: &mut BTreeMap<[i32; 3], ChunkMesh>,
    owner: &FaceOwner,
    cell: [i32; 3],
    axis: usize,
    sign: i32,
    plane: i32,
    u: i32,
    v: i32,
    targets: Option<&BTreeSet<[i32; 3]>>,
) {
    // A cell's faces can be attributed to a NEIGHBOUR's chunk (the carve-uncovered branch), so a
    // partial rebuild has to process cells outside the chunks it is rebuilding and then discard
    // what lands elsewhere. Dropping here rather than at the call site keeps that one rule in
    // one place.
    if targets.is_some_and(|targets| !targets.contains(&owner.chunk)) {
        return;
    }
    let mesh = chunks.entry(owner.chunk).or_default();
    mesh.masks
        .entry(MeshKey {
            chunk: owner.chunk,
            slot: owner.slot,
            rim: owner.rim,
            axis,
            sign,
            plane,
        })
        .or_default()
        .insert((u, v));
    mesh.cells.insert(cell);
}

/// Builds the opt-in fine terrain from the client mirror. The coarse cells are still the only
/// authority: every fine face begins with the same visible-cell set the shipped cube path uses.
///
/// NOTE: the small deterministic top pits are a measurement stand-in for 10.4's authored terrain
/// look, not a visual decision. They make `--subdiv N` measure non-flat fine surfaces rather than
/// merely tessellating an otherwise identical plane.
/// The chunks a set of changed cells can alter.
///
/// A changed cell alters its own faces and those of its six neighbours — the neighbour's face
/// toward it appears or disappears, and a change to its top exposure changes which of its
/// neighbours its pit uncovers. Nothing further away can move.
/// `the_dirty_chunk_set_covers_every_chunk_a_change_can_alter` checks that claim by diffing two
/// whole-world builds rather than by re-deriving it.
fn dirty_chunks(dirty_tiles: &[[i32; 3]]) -> BTreeSet<[i32; 3]> {
    // TWO steps, not one, and the second step is not padding. Digging a cell changes whether its
    // face NEIGHBOURS are drawn at all; a newly drawn neighbour then emits faces attributed to
    // ITS neighbours, which is two cells from the dug one. A one-step set is faithful for the
    // chunks it names (`partial_rebuild_matches_the_whole_world_build` still passed) and simply
    // misses chunks, so the rebuild leaves stale geometry behind rather than building it wrong --
    // invisible to any fixture too thin to put a chunk seam two cells from a dug cell.
    let mut cells = BTreeSet::new();
    for position in dirty_tiles {
        cells.insert(*position);
        for delta in NEIGHBOURS {
            cells.insert([
                position[0] + delta[0],
                position[1] + delta[1],
                position[2] + delta[2],
            ]);
        }
    }
    let mut targets = BTreeSet::new();
    for position in &cells {
        targets.insert(chunk_of(*position));
        for delta in NEIGHBOURS {
            targets.insert(chunk_of([
                position[0] + delta[0],
                position[1] + delta[1],
                position[2] + delta[2],
            ]));
        }
    }
    targets
}

/// Whether processing this cell could put a face in any of `targets`.
///
/// A cell emits faces owned by itself and, where its pit uncovers one, by a side neighbour. So
/// the cells that can contribute to a chunk are the ones in it plus the ones one step outside its
/// boundary — no further. This is what makes a partial rebuild produce byte-identical chunks to a
/// whole-world one, which `partial_rebuild_matches_the_whole_world_build` asserts directly.
fn touches_chunks(position: [i32; 3], targets: &BTreeSet<[i32; 3]>) -> bool {
    if targets.contains(&chunk_of(position)) {
        return true;
    }
    NEIGHBOURS.into_iter().any(|delta| {
        let neighbour = [
            position[0] + delta[0],
            position[1] + delta[1],
            position[2] + delta[2],
        ];
        targets.contains(&chunk_of(neighbour))
    })
}

fn chunk_of(position: [i32; 3]) -> [i32; 3] {
    [
        position[0] / TERRAIN_CHUNK_EDGE,
        position[1] / TERRAIN_CHUNK_EDGE,
        position[2] / TERRAIN_CHUNK_EDGE,
    ]
}

/// Emits every fine face the drawn coarse cells produce, grouped by chunk.
///
/// Split out of the spawning pass so it can be counted without a renderer: the mesher shipped
/// with no numeric oracle at all — the only test asserted entity presence and handle equality —
/// which is why the buried-face, hash and cross-cell-connector defects were all shippable.
fn build_chunk_meshes(
    mirror: &Mirror,
    positions: &[[i32; 3]],
    subdiv: i32,
    level: i32,
    targets: Option<&BTreeSet<[i32; 3]>>,
    cover: &TreeCover,
) -> BTreeMap<[i32; 3], ChunkMesh> {
    let mut chunks = BTreeMap::<[i32; 3], ChunkMesh>::new();
    for &position in positions {
        if is_mesh_drawn_tree(mirror, position, cover) {
            continue;
        }
        if targets.is_some_and(|targets| !touches_chunks(position, targets)) {
            continue;
        }
        let owner = FaceOwner::new(mirror, position);
        let own = column_heights(mirror, position, subdiv, level);

        let above = [position[0], position[1], position[2] + 1];
        if !occludes_terrain(mirror, above, level, cover) {
            // Settled snow is PAINT on the top faces here, not the shipped path's separate slab.
            // A slab is cell-scale: it sits at the coarse cell top while the fine surface is a
            // pit deeper, it covers 102% of the cell so it hides 17.9% of the very detail this
            // path exists to draw, and it is `ClientLocal`, so it is not a tile and cannot be
            // dug -- which read as big plates lying on top of everything that nothing could
            // touch. As a material on the real surface it follows every fine column exactly,
            // hides nothing, adds no geometry, and costs 8,145 fewer entities.
            let top = if has_snow_cap(mirror, position) {
                FaceOwner {
                    chunk: owner.chunk,
                    slot: TerrainSlot::SnowCap as usize,
                    rim: owner.rim,
                }
            } else {
                FaceOwner {
                    chunk: owner.chunk,
                    slot: owner.slot,
                    rim: owner.rim,
                }
            };
            for du in 0..subdiv {
                for dv in 0..subdiv {
                    let plane = position[2] * subdiv + column_height(own.as_ref(), du, dv, subdiv);
                    let (u, v) = (position[0] * subdiv + du, position[1] * subdiv + dv);
                    push_face(&mut chunks, &top, position, 2, 1, plane, u, v, targets);
                }
            }
        }
        let under = [position[0], position[1], position[2] - 1];
        if !occludes_terrain(mirror, under, level, cover) {
            let plane = position[2] * subdiv;
            for du in 0..subdiv {
                for dv in 0..subdiv {
                    let (u, v) = (position[0] * subdiv + du, position[1] * subdiv + dv);
                    push_face(&mut chunks, &owner, position, 2, -1, plane, u, v, targets);
                }
            }
        }

        for (axis, sign) in [(0usize, -1i32), (0, 1), (1, -1), (1, 1)] {
            let mut neighbour = position;
            neighbour[axis] += sign;
            let solid = occludes_terrain(mirror, neighbour, level, cover);
            let other = solid
                .then(|| column_heights(mirror, neighbour, subdiv, level))
                .flatten();
            // Only paid for when this cell's pit actually uncovers the neighbour.
            let mut neighbour_owner = None;
            let plane = (position[axis] + i32::from(sign > 0)) * subdiv;
            let near = if sign > 0 { subdiv - 1 } else { 0 };
            let far = if sign > 0 { 0 } else { subdiv - 1 };
            for step in 0..subdiv {
                let (du, dv) = if axis == 0 {
                    (near, step)
                } else {
                    (step, near)
                };
                let top = column_height(own.as_ref(), du, dv, subdiv);
                let floor = if solid {
                    let (odu, odv) = if axis == 0 { (far, step) } else { (step, far) };
                    column_height(other.as_ref(), odu, odv, subdiv)
                } else {
                    0
                };
                let (low, high, face_sign, cell) = if top > floor {
                    (floor, top, sign, position)
                } else if is_mesh_drawn_tree(mirror, neighbour, cover) {
                    // A MESHED tree cell is carried by its one mesh, never by the terrain mesher.
                    // A tree cell no mesh carries is ordinary terrain here and must not be
                    // skipped, or the fallback would be as invisible as the hole it closes.
                    continue;
                } else {
                    // The NEIGHBOUR's face, uncovered by this cell's pit. It can be buried at
                    // the coarse scale and so absent from `positions` altogether, in which case
                    // nothing else in this pass would ever emit it.
                    (top, floor, -sign, neighbour)
                };
                if low >= high {
                    continue;
                }
                let owner = if cell == position {
                    &owner
                } else {
                    neighbour_owner.get_or_insert_with(|| FaceOwner::new(mirror, neighbour))
                };
                for fine in low..high {
                    let (u, v) = if axis == 0 {
                        (position[1] * subdiv + step, position[2] * subdiv + fine)
                    } else {
                        (position[2] * subdiv + fine, position[0] * subdiv + step)
                    };
                    push_face(
                        &mut chunks,
                        owner,
                        cell,
                        axis,
                        face_sign,
                        plane,
                        u,
                        v,
                        targets,
                    );
                }
            }
        }

        if let Some(heights) = own.as_ref() {
            for du in 0..subdiv {
                for dv in 0..subdiv {
                    let top = heights[(du * subdiv + dv) as usize];
                    for (step_u, step_v, axis) in [(1, 0, 0usize), (0, 1, 1usize)] {
                        let (nu, nv) = (du + step_u, dv + step_v);
                        if nu >= subdiv || nv >= subdiv {
                            continue;
                        }
                        let other = heights[(nu * subdiv + nv) as usize];
                        if top == other {
                            continue;
                        }
                        let sign = if top > other { 1 } else { -1 };
                        let plane = if axis == 0 {
                            position[0] * subdiv + du + 1
                        } else {
                            position[1] * subdiv + dv + 1
                        };
                        for fine in top.min(other)..top.max(other) {
                            let (u, v) = if axis == 0 {
                                (position[1] * subdiv + dv, position[2] * subdiv + fine)
                            } else {
                                (position[2] * subdiv + fine, position[0] * subdiv + du)
                            };
                            push_face(
                                &mut chunks,
                                &owner,
                                position,
                                axis,
                                sign,
                                plane,
                                u,
                                v,
                                targets,
                            );
                        }
                    }
                }
            }
        }
    }

    chunks
}

/// Spawns the fine terrain, either whole or for just the chunks in `targets`.
///
/// `targets` is what keeps a dig cheap. Rebuilding the world for one changed tile is correct but
/// unbounded, and it stopped the client dead for the length of a mesh build on every dug tile --
/// which, since a dwarf digs continuously, meant every other dwarf froze for the whole job.
#[allow(clippy::too_many_arguments)]
fn spawn_subdivided_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &ProjectionAssets,
    mirror: &Mirror,
    positions: &[[i32; 3]],
    subdiv: u32,
    level: i32,
    targets: Option<&BTreeSet<[i32; 3]>>,
    cover: &TreeCover,
) -> SubdividedTerrainStats {
    let subdiv = i32::from(u16::try_from(subdiv.min(MAX_SUBDIV)).expect("--subdiv is bounded"));
    let mut stats = SubdividedTerrainStats::default();
    for (chunk, chunk_mesh) in build_chunk_meshes(mirror, positions, subdiv, level, targets, cover)
    {
        let mut material_meshes = BTreeMap::<(usize, usize), MeshBuilder>::new();
        stats.faces += chunk_mesh.masks.values().map(BTreeSet::len).sum::<usize>();
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
        stats.cells += chunk_mesh.cells.len();
        stats.chunks += 1;
        commands.spawn((
            TerrainChunk(chunk),
            TerrainChunkCells(chunk_mesh.cells.into_iter().collect()),
        ));
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
///
// NOTE: This is a MEASUREMENT STAND-IN for 10.4's authored terrain look, not a visual
/// decision. It exists only so a flat cell top stops being one greedy quad and fineness
/// becomes measurable; 10.4 owns the real look and this is the copy it will replace. The
/// figure it drives is placeholder-dominated -- uncorrelated noise is 96.8% of the adopted
/// k=4 triangle budget, and sampling the same rule coherently over a cell moves that budget
/// 11.5x -- so no number derived from it may be read as the cost of authored terrain.
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
    // Rows before columns, matching `scripts/bench/resolution_bench.py`. Rectangle choice is
    // traversal-order dependent -- the bench measured 19,353 quads column-first against 19,264
    // row-first on the same world -- and a `BTreeSet<(u, v)>` iterates columns first. The two
    // meshers have to make the same choice or the offline number cannot predict this one.
    let mut ordered = mask.iter().copied().collect::<Vec<_>>();
    ordered.sort_by_key(|&(u, v)| (v, u));
    for (u, v) in ordered {
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
    // The two orders were the wrong way round, on every axis and both signs, since the chunk
    // mesher shipped: each quad was wound to face opposite its own normal, and the default
    // `cull_mode: Some(Face::Back)` then deleted the entire terrain surface. `--subdiv N > 1`
    // drew the world as snow caps and tree cubes floating over a void.
    let order = if key.sign > 0 {
        [0, 1, 2, 3]
    } else {
        [0, 3, 2, 1]
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
    trees: &TreeMeshQuery,
    chips: &DigChipQuery,
    assets: Option<&ProjectionAssets>,
    meshes: Option<&mut Assets<Mesh>>,
    subdivision: Option<&TerrainSubdivision>,
) {
    // The cover both the terrain branches and the tree branch draw against, derived ONCE. It has
    // to exist before the terrain branches: a tree that has just become unrepresentable must have
    // its cube fallback spawned in the SAME pass that despawns its mesh, or it vanishes for a
    // frame. On the incremental path this costs the dirty columns, not the world.
    let live_tree_bases = trees.iter().map(|(_, tree)| tree.0).collect::<Vec<_>>();
    let trees_now = if rebuild_terrain {
        IncrementalTrees {
            cover: tree_cover_at(mirror, slice.level()),
            rederived: BTreeMap::new(),
            fallback_cells: Vec::new(),
        }
    } else {
        incremental_tree_cover(&live_tree_bases, mirror, slice.level(), dirty_tiles)
    };
    let tree_cover = &trees_now.cover;
    let rederived_columns = &trees_now.rederived;
    let tree_fallback_cells = &trees_now.fallback_cells;

    if rebuild_terrain {
        for (entity, _) in chips.iter() {
            commands.entity(entity).despawn();
        }
        for (entity, _, _, _) in terrain.iter() {
            commands.entity(entity).despawn();
        }
        for (entity, _) in trees.iter() {
            commands.entity(entity).despawn();
        }
        if let Some(assets) = assets {
            spawn_tree_meshes(commands, assets, mirror, slice.level());
        }
        let positions = terrain_positions_at_with_cover(mirror, slice.level(), tree_cover);
        // The draw-set oracle instrument (AC13). The shipped seed reports 39,936 terrain cubes
        // after 10.4 moved 5,048 tree cells to meshes; the simulation census remains 44,984
        // exposed cells. Read it as "did the rim, slice, or draw path silently drop terrain?",
        // never as a fixed constant.
        let started = Instant::now();
        let subdiv = subdivision.map_or(1, |subdivision| subdivision.0);
        if subdiv > 1 {
            // This used to `return` when the render assets were absent, AFTER despawning all
            // terrain — which also skipped dynamic-entity, item, designation and zone
            // reconciliation, silently, for the rest of the frame.
            if let Some((assets, meshes)) = assets.zip(meshes) {
                let stats = spawn_subdivided_terrain(
                    commands,
                    meshes,
                    assets,
                    mirror,
                    &positions,
                    subdiv,
                    slice.level(),
                    None,
                    tree_cover,
                );
                println!(
                    "subdiv {subdiv}: projected {} terrain cubes at z {} entities={} chunks={} \
                     faces={} triangles={} mesh_build_ms={}",
                    stats.cells,
                    slice.level(),
                    stats.entities,
                    stats.chunks,
                    stats.faces,
                    stats.triangles,
                    started.elapsed().as_millis()
                );
            } else {
                println!("subdiv {subdiv}: no render assets, terrain not respawned");
            }
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
                // `triangles_derived`, not `triangles`: this path draws shared unit cuboids and
                // snow-cap meshes, so its count is arithmetic over the entity list, while every
                // k>1 row above is counted off the indices actually built. Putting the two under
                // one name is exactly the blend AC6 forbids for Axis B.
                println!(
                    "subdiv 1: projected {} terrain cubes at z {} entities={} chunks=0 \
                     triangles_derived={} mesh_build_ms={}",
                    positions.len(),
                    slice.level(),
                    positions.len() + snow_caps,
                    (positions.len() + snow_caps) * 12,
                    started.elapsed().as_millis()
                );
            }
        }
    } else if subdivision.map_or(1, |subdivision| subdivision.0) > 1 && !dirty_tiles.is_empty() {
        // The fine path's incremental branch. A chunk mesh is a whole surface, so a changed cell
        // cannot be edited in place the way a per-cell cube entity can -- but only the chunks it
        // can reach need rebuilding, not the world. One dug tile touched 121 chunks and every
        // dwarf on screen stopped for the length of a full mesh build; it now touches at most a
        // handful, and `partial_rebuild_matches_the_whole_world_build` pins that the result is
        // identical either way.
        // Falls THROUGH rather than `return`ing: an absent asset set must not skip the
        // dig-chip, dynamic-entity, item, designation and zone reconciliation below. That is
        // exactly the defect the full-rebuild branch above was fixed for, and it relocated into
        // this branch because this branch did not exist when that fix was written.
        if let Some((assets, meshes)) = assets.zip(meshes) {
            let subdiv = subdivision.map_or(1, |subdivision| subdivision.0);
            let mut targets = dirty_chunks(dirty_tiles);
            // A tree that swapped between mesh and cubes changes chunks no dirty TILE names.
            targets.extend(dirty_chunks(tree_fallback_cells));
            for (entity, tile, cap, chunk) in terrain.iter() {
                let owner = tile
                    .map(|tile| tile.0)
                    .or_else(|| cap.map(|cap| cap.0))
                    .map(chunk_of);
                // Chunk meshes and their cell records carry no cell of their own; they are matched
                // by the chunk they were spawned for.
                let hit = match owner {
                    Some(chunk) => targets.contains(&chunk),
                    None => chunk.is_some_and(|chunk| targets.contains(&chunk.0)),
                };
                if hit {
                    commands.entity(entity).despawn();
                }
            }
            let started = Instant::now();
            // Scanning the whole world for the draw set costs ~130 ms here and dwarfs the meshing
            // it feeds. Only cells in the target chunks, plus one step outside them, can contribute
            // a face to those chunks -- the same rule `touches_chunks` applies.
            let positions = terrain_positions_near(mirror, slice.level(), &targets);
            let stats = spawn_subdivided_terrain(
                commands,
                meshes,
                assets,
                mirror,
                &positions,
                subdiv,
                slice.level(),
                Some(&targets),
                tree_cover,
            );
            println!(
                "subdiv {subdiv}: rebuilt {} of {} chunks for {} changed tiles, entities={} \
                 faces={} triangles={} mesh_build_ms={}",
                stats.chunks,
                targets.len(),
                dirty_tiles.len(),
                stats.entities,
                stats.faces,
                stats.triangles,
                started.elapsed().as_millis()
            );
        } else {
            println!("subdiv: no render assets, terrain not rebuilt");
        }
    } else {
        let mut affected = BTreeSet::new();
        affected.extend(tree_fallback_cells.iter().copied());
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
            for (entity, tile, cap, _) in terrain.iter() {
                if tile.is_some_and(|tile| tile.0 == position)
                    || cap.is_some_and(|cap| cap.0 == position)
                {
                    commands.entity(entity).despawn();
                }
            }
            if is_visible_at_slice(mirror, position, slice.level())
                && !is_mesh_drawn_tree(mirror, position, tree_cover)
            {
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

    if !rebuild_terrain && !dirty_tiles.is_empty() {
        // Every re-derived column is rebuilt whole: despawn whatever mesh it had, then spawn what
        // the mirror now says it should have. `tree_meshes`' whole-world sweep used to run here on
        // every tree-touching delta -- measured 43-63 ms on a 128x128x32 world, the same stall
        // class the terrain path carries `targets` to avoid, and logged by nothing.
        for (entity, tree) in trees.iter() {
            if rederived_columns.contains_key(&[tree.0[0], tree.0[1]]) {
                commands.entity(entity).despawn();
            }
        }
        if let Some(assets) = assets {
            for (base, variant) in rederived_columns.values().flatten() {
                spawn_tree_mesh(commands, assets, *base, *variant);
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

fn is_tree(mirror: &Mirror, position: [i32; 3]) -> bool {
    matches!(
        terrain_material_at(mirror, position),
        Some(Material::TreeTrunk | Material::TreeFoliage)
    )
}

/// A tree's cells occupy its trunk column and the one-cell crown ring. A delta arrives after it
/// has changed the mirror, so checking `is_tree` alone loses a just-dug foliage cell; retain the
/// mesh root long enough to rebuild or retire it from this conservative footprint.
fn tree_mesh_might_cover(base: [i32; 3], position: [i32; 3]) -> bool {
    position[2] >= base[2]
        && (position[0] - base[0]).abs() <= 1
        && (position[1] - base[1]).abs() <= 1
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

    fn tree_scene(&self, variant: TreeVariant) -> Handle<WorldAsset> {
        self.trees[match variant {
            TreeVariant::Tree01 => 0,
            TreeVariant::Tree02 => 1,
            TreeVariant::Tree03 => 2,
            TreeVariant::Tree04R => 3,
        }]
        .clone()
    }

    pub fn tree_scenes_loaded(&self, asset_server: &AssetServer) -> bool {
        self.trees
            .iter()
            .all(|scene| asset_server.is_loaded_with_dependencies(scene.id()))
    }
}

/// The trunk columns a mesh actually carries, keyed by column so a cell can ask in O(1).
///
/// Every tree cell OUTSIDE this cover falls back to the cube path. That fallback is the point:
/// `tree_meshes` REJECTS a column it cannot represent, and both spawn paths filter tree cells out
/// of the terrain, so before this existed a rejected column was drawn by the mesh path and the
/// cube path neither — the tree simply vanished, silently, with the cut-face oracle rejecting it
/// on both sides at once and reporting a match.
#[derive(Default)]
pub(crate) struct TreeCover {
    bases: BTreeMap<[i32; 2], i32>,
}

impl TreeCover {
    pub(crate) fn from_meshes<'a>(meshes: impl IntoIterator<Item = &'a [i32; 3]>) -> Self {
        Self {
            bases: meshes
                .into_iter()
                .map(|base| ([base[0], base[1]], base[2]))
                .collect(),
        }
    }

    /// A cell is carried by a mesh when a meshed column inside the one-cell crown ring starts at
    /// or below it — the same conservative footprint `tree_mesh_might_cover` uses for dirt, and
    /// safe to reuse because `place_trees` keeps trunks three cells apart in Chebyshev, so two
    /// rings never overlap.
    ///
    /// NOTE: this answers "does a mesh draw this cell", NOT "is this cell a tree". It is
    /// deliberately conservative and returns true for the plain terrain inside a crown ring, so
    /// it must only ever be consulted for cells that are already known to be tree cells.
    pub(crate) fn covers(&self, position: [i32; 3]) -> bool {
        (-1..=1).any(|dx| {
            (-1..=1).any(|dy| {
                let column = [position[0] + dx, position[1] + dy];
                self.bases.get(&column).is_some_and(|base_z| {
                    tree_mesh_might_cover([column[0], column[1], *base_z], position)
                })
            })
        })
    }
}

/// Whether a mesh draws this cell, and therefore the terrain path must not. Its complement over
/// tree cells is the fallback: a tree cell no mesh carries is drawn as cubes, exactly as it was
/// before mesh trees.
fn is_mesh_drawn_tree(mirror: &Mirror, position: [i32; 3], cover: &TreeCover) -> bool {
    is_tree(mirror, position) && cover.covers(position)
}

/// The single mesh rule, shared by the whole-world sweep and by the re-derivation of one changed
/// column, so the incremental path can never classify a tree differently from a full rebuild.
fn classify_trunk_column(
    x: i32,
    y: i32,
    base_z: i32,
    top_z: i32,
    cells: i32,
    slice_level: i32,
) -> Option<([i32; 3], TreeVariant)> {
    if base_z > slice_level {
        return None;
    }
    // `TreeMesh`'s doc comment promised a CONTIGUOUS column and nothing checked it. A dwarf
    // digging one mid-trunk cell leaves min and max untouched, so the client redrew an unbroken
    // pine over a hole the sim had already stored — no panic, no log, no test. A gapped column is
    // rejected here and falls back to cubes, which is exactly what drew it before mesh trees.
    if top_z - base_z + 1 != cells {
        return None;
    }
    let height = top_z - base_z + 2;
    let variant = match height {
        4 => TreeVariant::Tree01,
        5 if tree_variant_hash(x, y).is_multiple_of(2) => TreeVariant::Tree02,
        5 => TreeVariant::Tree03,
        6 => TreeVariant::Tree04R,
        // Worldgen has pinned this range since story 9.4. A malformed snapshot must
        // not silently stretch a signed-off mesh into a new resolution contract.
        _ => return None,
    };
    Some(([x, y, base_z], variant))
}

/// The trunk extent of one column: lowest cell, highest cell, and how many cells there actually
/// are. The third value is what makes a gap visible — without it a dug column is indistinguishable
/// from a whole one.
fn trunk_column_extent(mirror: &Mirror, x: i32, y: i32) -> Option<(i32, i32, i32)> {
    let mut extent: Option<(i32, i32, i32)> = None;
    for z in 0..mirror.dims().z as i32 {
        if terrain_material_at(mirror, [x, y, z]) == Some(Material::TreeTrunk) {
            extent = Some(match extent {
                Some((base, _, cells)) => (base, z, cells + 1),
                None => (z, z, 1),
            });
        }
    }
    extent
}

/// Re-derives ONE column. The incremental path uses this instead of `tree_meshes` because a
/// whole-world sweep on every tree-touching delta is the ~130 ms stall the terrain path already
/// carries `targets` to avoid; measured at 43-63 ms here on a 128x128x32 world.
fn tree_mesh_for_column(
    mirror: &Mirror,
    x: i32,
    y: i32,
    slice_level: i32,
) -> Option<([i32; 3], TreeVariant)> {
    let (base_z, top_z, cells) = trunk_column_extent(mirror, x, y)?;
    classify_trunk_column(x, y, base_z, top_z, cells, slice_level)
}

/// The cover an incremental pass draws with, and the cells whose draw path that pass changes.
///
/// No whole-world sweep: live `TreeMesh` entities already name every meshed column, so only the
/// columns the dirty tiles can reach are re-derived from the mirror. Returns the re-derivations
/// too, so the tree branch respawns from the same answer the terrain branch drew against — if the
/// two disagree, a tree is drawn twice or not at all.
struct IncrementalTrees {
    cover: TreeCover,
    /// Every column the dirty tiles could reach, re-derived: `Some` is the mesh it should now
    /// have, `None` means the mesh rule rejects it and the cube fallback owns it.
    rederived: BTreeMap<[i32; 2], Option<([i32; 3], TreeVariant)>>,
    /// The tree cells whose DRAW PATH changed, which the terrain branches must respawn.
    fallback_cells: Vec<[i32; 3]>,
}

fn incremental_tree_cover(
    live: &[[i32; 3]],
    mirror: &Mirror,
    slice_level: i32,
    dirty_tiles: &[[i32; 3]],
) -> IncrementalTrees {
    let mut touched = BTreeSet::<[i32; 2]>::new();
    for position in dirty_tiles {
        for dx in -1..=1 {
            for dy in -1..=1 {
                touched.insert([position[0] + dx, position[1] + dy]);
            }
        }
    }
    let rederived = touched
        .iter()
        .map(|&[x, y]| ([x, y], tree_mesh_for_column(mirror, x, y, slice_level)))
        .collect::<BTreeMap<_, _>>();
    let mut bases = live
        .iter()
        .map(|base| ([base[0], base[1]], base[2]))
        .collect::<BTreeMap<[i32; 2], i32>>();
    // A column that gains or loses its mesh changes how EVERY cell of that tree is drawn, not
    // only the cell that was dug: the mesh carried a whole 3x3 crown ring, so the cube fallback
    // has to be spawned across all of it or the tree comes back as a fragment.
    let mut fallback_cells = Vec::new();
    for (column, result) in &rederived {
        let was = bases.get(column).copied();
        let now = result.map(|(base, _)| base[2]);
        match now {
            Some(base_z) => {
                bases.insert(*column, base_z);
            }
            None => {
                bases.remove(column);
            }
        }
        if was == now {
            continue;
        }
        for dx in -1..=1 {
            for dy in -1..=1 {
                for z in 0..mirror.dims().z as i32 {
                    let position = [column[0] + dx, column[1] + dy, z];
                    if is_tree(mirror, position) {
                        fallback_cells.push(position);
                    }
                }
            }
        }
    }
    IncrementalTrees {
        cover: TreeCover { bases },
        rederived,
        fallback_cells,
    }
}

/// The cover a full rebuild draws with. O(world), which is what a full rebuild already is.
pub(crate) fn tree_cover_at(mirror: &Mirror, level: i32) -> TreeCover {
    TreeCover::from_meshes(tree_meshes(mirror, level).iter().map(|(base, _)| base))
}

/// The sim sends tree tiles, not tree identities. Rebuild one mesh tree from every trunk column
/// so presentation stays off the wire and BTreeMap's coordinate order fixes the result.
fn tree_meshes(mirror: &Mirror, slice_level: i32) -> Vec<([i32; 3], TreeVariant)> {
    let mut columns = BTreeMap::<[i32; 2], (i32, i32, i32)>::new();
    for_each_position(mirror.dims(), |[x, y, z]| {
        if terrain_material_at(mirror, [x, y, z]) == Some(Material::TreeTrunk) {
            columns
                .entry([x, y])
                .and_modify(|(base, top, cells)| {
                    *base = (*base).min(z);
                    *top = (*top).max(z);
                    *cells += 1;
                })
                .or_insert((z, z, 1));
        }
    });
    columns
        .into_iter()
        .filter_map(|([x, y], (base_z, top_z, cells))| {
            classify_trunk_column(x, y, base_z, top_z, cells, slice_level)
        })
        .collect()
}

/// NOTE: this shares `tree_meshes` with the spawn path, so it can only catch a despawn or
/// incremental leak — never a defect INSIDE the mesh rule, which would move both sides together.
/// The independent check is `assert_no_tree_is_undrawn`, which compares the MIRROR against what
/// was actually drawn and routes through neither.
pub fn expected_tree_mesh_count(mirror: &Mirror, slice_level: i32) -> usize {
    tree_meshes(mirror, slice_level).len()
}

/// Every tree cell at or below the cut, from the mirror alone. The capture oracle's independent
/// side: it knows nothing about the mesh rule, so a column that stops being drawn by BOTH paths
/// shows up here as a cell nothing accounts for.
pub fn tree_cells_at_or_below(mirror: &Mirror, slice_level: i32) -> Vec<[i32; 3]> {
    let mut cells = Vec::new();
    for_each_position(mirror.dims(), |position| {
        if position[2] <= slice_level && is_tree(mirror, position) {
            cells.push(position);
        }
    });
    cells
}

/// Stable across Rust releases, unlike `DefaultHasher`. FNV-1a over the column, then over each
/// extra word.
///
/// NOTE: `extra` is EMPTY for the variant draw, not a zero word. FNV-1a mixes every byte, so
/// appending a zero salt is not a no-op — it would have reshuffled which pine each column gets,
/// a look change nobody asked for, while looking like a pure refactor.
fn tree_hash(x: i32, y: i32, extra: &[u32]) -> u32 {
    let mut hash = 0x811C_9DC5_u32;
    for value in [x as u32, y as u32].iter().chain(extra) {
        for byte in value.to_le_bytes() {
            hash ^= u32::from(byte);
            hash = hash.wrapping_mul(0x0100_0193);
        }
    }
    hash
}

/// Only x/y shape the 5-cell tie-break. Unsalted, and it must stay that way: this is the shipped
/// species assignment.
fn tree_variant_hash(x: i32, y: i32) -> u32 {
    tree_hash(x, y, &[])
}

/// A SEPARATE draw, so yaw does not correlate with variant. Keying both off one value would make
/// every Tree02 face the same way, which is the repetition this exists to break.
fn tree_yaw_hash(x: i32, y: i32) -> u32 {
    tree_hash(x, y, &[YAW_SALT])
}

const YAW_SALT: u32 = 0x5941_5721;

/// One-shot state for `report_tree_meshes_once`.
#[derive(Resource, Default)]
pub struct TreeReportState {
    reported: bool,
    frames: u32,
}

/// Reports what the client ACTUALLY drew, once, on EVERY run -- windowed included.
///
/// The startup line can only ever speak for bytes compiled in. This is the line that says the
/// scenes decoded and the meshes reached the world, and until now that evidence was produced only
/// under `--headless --capture` -- i.e. never in the windowed run the vehicle sitting actually
/// does, which is the one run where a human is looking for it.
pub fn report_tree_meshes_once(
    mut state: ResMut<TreeReportState>,
    trees: Query<&TreeMesh>,
    assets: Option<Res<ProjectionAssets>>,
    asset_server: Option<Res<AssetServer>>,
) {
    if state.reported {
        return;
    }
    state.frames += 1;
    let loaded = assets
        .zip(asset_server)
        .is_some_and(|(assets, asset_server)| assets.tree_scenes_loaded(&asset_server));
    let spawned = trees.iter().count();
    // Report on success, or give up and report the FAILURE rather than staying silent: a line
    // that only ever appears when things worked is not an instrument.
    if (loaded && spawned > 0) || state.frames >= TREE_REPORT_DEADLINE_FRAMES {
        state.reported = true;
        eprintln!(
            "gui trees: meshes={spawned} scenes_loaded={loaded} source=embedded frames={}",
            state.frames
        );
    }
}

/// Long enough for the embedded scenes to decode on a software renderer, short enough that a
/// human sees the verdict during a sitting.
const TREE_REPORT_DEADLINE_FRAMES: u32 = 600;

fn spawn_tree_meshes(
    commands: &mut Commands,
    assets: &ProjectionAssets,
    mirror: &Mirror,
    slice_level: i32,
) {
    for (base, variant) in tree_meshes(mirror, slice_level) {
        spawn_tree_mesh(commands, assets, base, variant);
    }
}

fn spawn_tree_mesh(
    commands: &mut Commands,
    assets: &ProjectionAssets,
    base: [i32; 3],
    variant: TreeVariant,
) {
    // NOTE: slice rendering shows a whole tree once its base is at or below the cut; meshes
    // are not clipped to the slice because mesh clipping has no second use yet.
    // Quarter turns about the vertical, so 265 copies of four meshes do not all face the camera
    // identically. The bench has always done this and said why; the client did not, so the frame
    // Wolf approved as candidate D differed from the one the client actually draws.
    let yaw = (tree_yaw_hash(base[0], base[1]) % 4) as f32 * std::f32::consts::FRAC_PI_2;
    commands.spawn((
        TreeMesh(base),
        WorldAssetRoot(assets.tree_scene(variant)),
        Transform::from_translation(world_to_render(base) - Vec3::Y * 0.5)
            .with_rotation(bevy::math::Quat::from_rotation_y(yaw))
            .with_scale(Vec3::splat(0.625)),
    ));
}

pub fn terrain_positions(mirror: &Mirror) -> Vec<[i32; 3]> {
    terrain_positions_at(mirror, mirror.dims().z.saturating_sub(1) as i32)
}

/// The client-local draw set at a slice: retain full-depth exposure, then add the terrain floor
/// at the selected z. The latter arm is what makes a cut a filled cross-section rather than a
/// hollow shell; `is_exposed` remains the full-depth rule for ramps and the existing oracle.
/// The draw set restricted to the cells that can feed `targets`.
///
/// A partial rebuild must not pay for a whole-world scan; that scan was ~130 ms on the real world
/// against ~76 ms of meshing for the chunks it fed. The bounds are each target chunk's cell range
/// grown by one, which is exactly the reach `touches_chunks` allows.
fn terrain_positions_near(
    mirror: &Mirror,
    level: i32,
    targets: &BTreeSet<[i32; 3]>,
) -> Vec<[i32; 3]> {
    let dims = mirror.dims();
    let level = level.clamp(0, dims.z.saturating_sub(1) as i32);
    let limit = [dims.x as i32, dims.y as i32, dims.z as i32];
    let mut positions = Vec::new();
    let mut visited = BTreeSet::new();
    for chunk in targets {
        let low: [i32; 3] =
            std::array::from_fn(|axis| (chunk[axis] * TERRAIN_CHUNK_EDGE - 1).max(0));
        let high: [i32; 3] = std::array::from_fn(|axis| {
            ((chunk[axis] + 1) * TERRAIN_CHUNK_EDGE).min(limit[axis] - 1)
        });
        for z in low[2]..=high[2] {
            for y in low[1]..=high[1] {
                for x in low[0]..=high[0] {
                    let position = [x, y, z];
                    if visited.insert(position) && is_visible_at_slice(mirror, position, level) {
                        positions.push(position);
                    }
                }
            }
        }
    }
    positions
}

pub fn terrain_positions_at(mirror: &Mirror, level: i32) -> Vec<[i32; 3]> {
    terrain_positions_at_with_cover(mirror, level, &tree_cover_at(mirror, level))
}

/// The same draw set against a cover the caller already has, so a rebuild derives it once rather
/// than sweeping the world twice for the same answer.
pub(crate) fn terrain_positions_at_with_cover(
    mirror: &Mirror,
    level: i32,
    cover: &TreeCover,
) -> Vec<[i32; 3]> {
    let level = level.clamp(0, mirror.dims().z.saturating_sub(1) as i32);
    let mut positions = Vec::new();
    for_each_position(mirror.dims(), |position| {
        if is_visible_at_slice(mirror, position, level)
            && !is_mesh_drawn_tree(mirror, position, cover)
        {
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

    fn world(dims: Dims, tiles: Vec<Tile>) -> Mirror {
        Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims,
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

    /// Stepped columns: cliffs, coplanar tops across a cell boundary, and cells that are buried
    /// at the coarse scale until a neighbour's pit uncovers them.
    fn staircase() -> Mirror {
        let dims = Dims { x: 4, y: 4, z: 4 };
        let mut tiles = Vec::new();
        for z in 0..4 {
            for _ in 0..4 {
                for x in 0..4 {
                    tiles.push(if z <= x {
                        Tile::Solid(protocol::Material::Stone)
                    } else {
                        Tile::Empty
                    });
                }
            }
        }
        world(dims, tiles)
    }

    fn fine_geometry(mirror: &Mirror, subdiv: i32) -> (usize, usize) {
        let level = mirror.dims().z.saturating_sub(1) as i32;
        let positions = terrain_positions_at(mirror, level);
        let mut faces = 0;
        let mut triangles = 0;
        for (_, chunk_mesh) in build_chunk_meshes(
            mirror,
            &positions,
            subdiv,
            level,
            None,
            &tree_cover_at(mirror, level),
        ) {
            for (key, mask) in &chunk_mesh.masks {
                faces += mask.len();
                let mut builder = MeshBuilder::default();
                greedy_mask_into_mesh(&mut builder, *key, mask, subdiv);
                triangles += builder.indices.len() / 3;
            }
        }
        (faces, triangles)
    }

    /// A 40x4x4 stepped slab: wide enough to span three 16-cell chunks on x.
    fn wide_terrain() -> Mirror {
        let dims = Dims { x: 40, y: 4, z: 4 };
        let mut tiles = Vec::new();
        for z in 0..4 {
            for _ in 0..4 {
                for x in 0..40 {
                    tiles.push(if z <= (x / 8) % 4 {
                        Tile::Solid(protocol::Material::Stone)
                    } else {
                        Tile::Empty
                    });
                }
            }
        }
        world(dims, tiles)
    }

    /// What a dig actually costs, measured on the real exported world.
    ///
    /// Manual: needs `scripts/bench/export_world.py` to have written a snapshot first.
    #[test]
    #[ignore = "manual dig-cost measurement; run with --ignored --nocapture"]
    fn resolution_bench_times_a_one_tile_rebuild_against_a_whole_one() {
        let path = std::env::var("FROSTVEIN_WORLD").expect("set FROSTVEIN_WORLD to a snapshot");
        let raw = std::fs::read_to_string(path).expect("snapshot readable");
        let snapshot: protocol::Snapshot = serde_json::from_str(&raw).expect("snapshot parses");
        let mirror = Mirror::from_snapshot(snapshot).expect("mirror builds");
        let level = mirror.dims().z.saturating_sub(1) as i32;
        let positions = terrain_positions_at(&mirror, level);
        for subdiv in [2, 4, 8] {
            let started = Instant::now();
            let whole = build_chunk_meshes(
                &mirror,
                &positions,
                subdiv,
                level,
                None,
                &tree_cover_at(&mirror, level),
            );
            let whole_ms = started.elapsed().as_millis();
            // One dug tile: its chunk plus the chunks its six neighbours fall in.
            let dug = [64, 64, 8];
            let mut targets = BTreeSet::from([chunk_of(dug)]);
            for delta in NEIGHBOURS {
                targets.insert(chunk_of([
                    dug[0] + delta[0],
                    dug[1] + delta[1],
                    dug[2] + delta[2],
                ]));
            }
            let started = Instant::now();
            let partial = build_chunk_meshes(
                &mirror,
                &positions,
                subdiv,
                level,
                Some(&targets),
                &tree_cover_at(&mirror, level),
            );
            let partial_ms = started.elapsed().as_millis();
            println!(
                "dig-cost k={subdiv} whole={whole_ms}ms ({} chunks) partial={partial_ms}ms \
                 ({} chunks) speedup={:.1}x",
                whole.len(),
                partial.len(),
                whole_ms as f64 / partial_ms.max(1) as f64
            );
        }
    }

    fn wide_terrain_without(dug: [i32; 3]) -> Mirror {
        let dims = Dims { x: 40, y: 4, z: 4 };
        let mut tiles = Vec::new();
        for z in 0..4 {
            for y in 0..4 {
                for x in 0..40 {
                    let solid = z <= (x / 8) % 4 && [x, y, z] != dug;
                    tiles.push(if solid {
                        Tile::Solid(protocol::Material::Stone)
                    } else {
                        Tile::Empty
                    });
                }
            }
        }
        world(dims, tiles)
    }

    /// A world that crosses a chunk boundary on ALL THREE axes.
    ///
    /// `wide_terrain` is 40x4x4, so only x ever crosses the 16-cell chunk grid; a neighbour-rule
    /// omission on y or z is invisible to it. This is 20x20x20 with a diagonal surface, so
    /// x = 15/16, y = 15/16 and z = 15/16 are all real chunk seams.
    fn boxy_terrain() -> Mirror {
        boxy_terrain_without([-1, -1, -1])
    }

    fn boxy_terrain_without(dug: [i32; 3]) -> Mirror {
        let dims = Dims {
            x: 20,
            y: 20,
            z: 20,
        };
        let mut tiles = Vec::new();
        for z in 0..20 {
            for y in 0..20 {
                for x in 0..20 {
                    // (x + y) / 2 reaches 19, so the surface genuinely spans the z = 16 seam.
                    // A divisor that capped it lower would leave the z chunk boundary empty and
                    // the seam cells below would silently test nothing.
                    let solid = z <= (x + y) / 2 && [x, y, z] != dug;
                    tiles.push(if solid {
                        Tile::Solid(protocol::Material::Stone)
                    } else {
                        Tile::Empty
                    });
                }
            }
        }
        world(dims, tiles)
    }

    /// Rebuilding SOME chunks must produce exactly what rebuilding all of them would.
    ///
    /// This is the whole safety argument for the incremental dig path. A chunk mesh is a whole
    /// surface, so the old code rebuilt the world on any terrain delta rather than risk an old
    /// face welded into a chunk — correct, and it cost a full mesh build per dug tile. A partial
    /// build is only allowed to replace that if it is indistinguishable from the whole one, which
    /// is not obvious: a cell's faces can be attributed to a NEIGHBOUR's chunk when its pit
    /// uncovers buried rock, so the cells that feed a chunk extend one step past its boundary.
    #[test]
    fn partial_rebuild_matches_the_whole_world_build() {
        for mirror in [wide_terrain(), staircase()] {
            let level = mirror.dims().z.saturating_sub(1) as i32;
            let positions = terrain_positions_at(&mirror, level);
            for subdiv in [1, 2, 4] {
                let whole = build_chunk_meshes(
                    &mirror,
                    &positions,
                    subdiv,
                    level,
                    None,
                    &tree_cover_at(&mirror, level),
                );
                assert!(!whole.is_empty(), "the fixture must produce chunks");
                for chunk in whole.keys() {
                    let targets = BTreeSet::from([*chunk]);
                    // Fed by the RESTRICTED scan, exactly as `reconcile` feeds it. Using the
                    // whole draw set here would leave the bounds of `terrain_positions_near`
                    // untested, and a scan one cell too tight loses faces silently.
                    let near = terrain_positions_near(&mirror, level, &targets);
                    let partial = build_chunk_meshes(
                        &mirror,
                        &near,
                        subdiv,
                        level,
                        Some(&targets),
                        &tree_cover_at(&mirror, level),
                    );
                    assert_eq!(
                        partial.keys().collect::<Vec<_>>(),
                        vec![chunk],
                        "a targeted build must touch no other chunk"
                    );
                    assert_eq!(
                        partial[chunk].masks, whole[chunk].masks,
                        "k={subdiv} chunk {chunk:?}: partial faces differ from the whole build"
                    );
                    assert_eq!(
                        partial[chunk].cells, whole[chunk].cells,
                        "k={subdiv} chunk {chunk:?}: partial cell record differs"
                    );
                }
                // And a multi-chunk subset, which is what a dig near a boundary actually asks for.
                let pair = whole.keys().take(2).copied().collect::<BTreeSet<_>>();
                let near = terrain_positions_near(&mirror, level, &pair);
                let partial = build_chunk_meshes(
                    &mirror,
                    &near,
                    subdiv,
                    level,
                    Some(&pair),
                    &tree_cover_at(&mirror, level),
                );
                for chunk in &pair {
                    assert_eq!(partial[chunk].masks, whole[chunk].masks);
                }
            }
        }
    }

    /// The dirty-chunk set must contain every chunk a change can actually alter.
    ///
    /// `partial_rebuild_matches_the_whole_world_build` proves a targeted build is faithful FOR THE
    /// CHUNKS IT IS GIVEN. That is only half the argument: if the set is too small, the rebuild is
    /// perfectly faithful and the world still keeps a stale chunk. This is the other half, and it
    /// is checked by diffing two whole-world builds of worlds that differ in one cell — not by
    /// restating the neighbour rule.
    #[test]
    fn the_dirty_chunk_set_covers_every_chunk_a_change_can_alter() {
        // x = 15 is the last cell of chunk 0 and x = 16 the first of chunk 1, so a change here
        // must reach both. A rule that only took the cell's own chunk would pass on any interior
        // cell and fail exactly here. `wide_terrain` is 40x4x4, so it can only ever exercise the
        // X seam -- `boxy_terrain` is 20x20x20 and carries the Y and Z seams too, because a
        // neighbour rule can be wrong on one axis and right on the others.
        let wide = [[15, 2, 1], [16, 2, 1], [15, 2, 0], [8, 2, 1]]
            .map(|dug| (dug, false))
            .to_vec();
        // The y and z seams, which `wide_terrain` is too thin to contain at all.
        let boxy = [
            [15, 15, 3],
            [16, 15, 3],
            [8, 15, 5],
            [8, 16, 5],
            [16, 16, 15],
            [16, 16, 16],
        ]
        .map(|dug| (dug, true))
        .to_vec();
        // The fixture must actually STRADDLE all three seams, or the six cases above test the
        // interior twice over and prove nothing about y or z. Assert it rather than trust the
        // arithmetic: every dug cell must be solid before it is dug, and the boxy world's
        // chunk set must span more than one index on each axis.
        {
            let boxy_world = boxy_terrain();
            for (dug, boxed) in &boxy {
                assert!(boxed);
                assert!(
                    matches!(boxy_world.tile(*dug), Some(Tile::Solid(_))),
                    "seam fixture cell {dug:?} is not solid, so digging it changes nothing"
                );
            }
            let boxy_level = boxy_world.dims().z.saturating_sub(1) as i32;
            let chunks = build_chunk_meshes(
                &boxy_world,
                &terrain_positions_at(&boxy_world, boxy_level),
                2,
                boxy_level,
                None,
                &tree_cover_at(&boxy_world, boxy_level),
            );
            for axis in 0..3 {
                let spread = chunks
                    .keys()
                    .map(|chunk| chunk[axis])
                    .collect::<BTreeSet<_>>();
                assert!(
                    spread.len() > 1,
                    "boxy fixture occupies one chunk on axis {axis}: {spread:?}"
                );
            }
        }
        for (dug, boxed) in [wide, boxy].concat() {
            let before = if boxed {
                boxy_terrain()
            } else {
                wide_terrain()
            };
            let after = if boxed {
                boxy_terrain_without(dug)
            } else {
                wide_terrain_without(dug)
            };
            let level = before.dims().z.saturating_sub(1) as i32;
            for subdiv in [2, 4] {
                let whole_before = build_chunk_meshes(
                    &before,
                    &terrain_positions_at(&before, level),
                    subdiv,
                    level,
                    None,
                    &tree_cover_at(&before, level),
                );
                let whole_after = build_chunk_meshes(
                    &after,
                    &terrain_positions_at(&after, level),
                    subdiv,
                    level,
                    None,
                    &tree_cover_at(&after, level),
                );
                let mut altered = BTreeSet::new();
                for chunk in whole_before.keys().chain(whole_after.keys()) {
                    let a = whole_before.get(chunk).map(|mesh| &mesh.masks);
                    let b = whole_after.get(chunk).map(|mesh| &mesh.masks);
                    if a != b {
                        altered.insert(*chunk);
                    }
                }
                let targets = dirty_chunks(&[dug]);
                assert!(
                    altered.is_subset(&targets),
                    "k={subdiv} digging {dug:?} altered {:?}, which the dirty set {targets:?} \
                     does not cover -- a stale chunk would survive the rebuild",
                    altered.difference(&targets).collect::<Vec<_>>()
                );
            }
        }
    }

    /// The fixture has to actually span several chunks, or the test above proves nothing.
    #[test]
    fn the_partial_rebuild_fixture_spans_more_than_one_chunk() {
        let mirror = wide_terrain();
        let level = mirror.dims().z.saturating_sub(1) as i32;
        let positions = terrain_positions_at(&mirror, level);
        let chunks = build_chunk_meshes(
            &mirror,
            &positions,
            2,
            level,
            None,
            &tree_cover_at(&mirror, level),
        );
        assert!(
            chunks.len() >= 3,
            "wide_terrain spans {} chunks; a one-chunk world cannot tell a partial rebuild from \
             a whole one",
            chunks.len()
        );
    }

    /// Snow is painted on the top faces, and ONLY the top faces.
    ///
    /// The sides and bottom of a capped cell are still rock: a cap is settled snow lying on a
    /// surface, not a change of material. Getting this wrong would silver the walls of every
    /// trench. Asserted on the mask keys, which is where the material partition actually lives.
    #[test]
    fn a_capped_cell_paints_snow_on_its_top_faces_and_rock_everywhere_else() {
        let dims = Dims { x: 3, y: 3, z: 2 };
        let mut tiles = vec![Tile::Empty; 18];
        for y in 0..3 {
            for x in 0..3 {
                tiles[x + y * 3] = Tile::Solid(protocol::Material::Stone);
            }
        }
        let mirror = world(dims, tiles);
        let capped = [1, 1, 0];
        assert!(
            has_snow_cap(&mirror, capped),
            "the fixture must cap {capped:?} or this proves nothing"
        );
        let positions = terrain_positions_at(&mirror, 1);
        let snow = TerrainSlot::SnowCap as usize;
        let rock = TerrainSlot::of(protocol::Material::Stone) as usize;
        let mut tops = 0;
        let mut sides = 0;
        for (_, mesh) in
            build_chunk_meshes(&mirror, &positions, 2, 1, None, &tree_cover_at(&mirror, 1))
        {
            for (key, mask) in &mesh.masks {
                if key.axis == 2 && key.sign > 0 {
                    assert_eq!(
                        key.slot, snow,
                        "a top face of snow-capped rock must be snow"
                    );
                    tops += mask.len();
                } else {
                    assert_eq!(
                        key.slot, rock,
                        "axis {} sign {} was painted snow; only tops carry it",
                        key.axis, key.sign
                    );
                    sides += mask.len();
                }
            }
        }
        assert_eq!(tops, 36, "9 cells x 2x2 fine columns of top face");
        assert!(sides > 0, "the block must still have rock sides");
    }

    /// Every quad must be WOUND to face the way its own normal says it does.
    ///
    /// It was not, for every axis and both signs, since the chunk mesher shipped. The mesh is
    /// drawn with `StandardMaterial`'s default `cull_mode: Some(Face::Back)`, so the whole
    /// terrain surface was culled and `--subdiv N > 1` rendered the world as snow caps and tree
    /// cubes floating over a void. Wolf saw it from the vehicle both times and called it holes.
    ///
    /// No count could see this. Faces, quads, triangles, cells and chunks are all winding-blind,
    /// and the offline bench has no winding at all -- it counts a surface, it does not draw one.
    /// This asserts the one property that is only about drawing: cross the first triangle's edges
    /// and compare with the stored vertex normal.
    #[test]
    fn every_quad_is_wound_to_face_the_way_its_normal_points() {
        for axis in 0..3 {
            for sign in [-1, 1] {
                let key = MeshKey {
                    chunk: [0, 0, 0],
                    slot: 0,
                    rim: 0,
                    axis,
                    sign,
                    plane: 3,
                };
                let mut builder = MeshBuilder::default();
                let mask = BTreeSet::from([(1, 1)]);
                greedy_mask_into_mesh(&mut builder, key, &mask, 1);
                let corner = |index: usize| Vec3::from_array(builder.positions[index]);
                let (a, b, c) = (
                    corner(builder.indices[0] as usize),
                    corner(builder.indices[1] as usize),
                    corner(builder.indices[2] as usize),
                );
                let wound = (b - a).cross(c - b).normalize();
                let declared = Vec3::from_array(builder.normals[0]);
                assert!(
                    wound.dot(declared) > 0.9,
                    "axis {axis} sign {sign}: wound {wound:?} faces away from its own \
                     normal {declared:?} -- back-face culling deletes this quad"
                );
            }
        }
    }

    /// The detail rule is a MEASUREMENT STAND-IN shared with `scripts/bench/resolution_bench.py`,
    /// and the two sides are only one rule if they agree bit for bit. They did not: this side
    /// `wrapping_mul`s in u32 and the bench multiplied unbounded Python integers, so the two
    /// agreed at chance for every k > 1 while k=1 — where the clamp forces every depth to zero —
    /// stayed identical. The same vector is pinned in that file's test suite.
    #[test]
    fn the_detail_rule_matches_the_benchs_pinned_vector() {
        let vector = [[0, 0, 0], [1, 2, 3], [8, 5, 1], [9, 9, 9], [64, 17, 5]];
        let depths: Vec<i32> = vector
            .iter()
            .map(|point| detail_depth(point[0], point[1], point[2], 4))
            .collect();
        assert_eq!(depths, vec![1, 0, 1, 1, 2]);
        for point in vector {
            assert_eq!(
                detail_depth(point[0], point[1], point[2], 1),
                0,
                "k=1 has no room for a pit, which is why the k=1 control cannot see this rule"
            );
        }
    }

    /// Hand-written counts for the whole fine surface, at k=1 and above.
    ///
    /// The mesher shipped with no count assertion anywhere: its only test compared entity
    /// presence and mesh handles, so a face buried in rock, a missing cross-cell connector and a
    /// diverged hash were all invisible. These literals come from the offline bench, which is in
    /// turn checked against a brute-force fine-voxel oracle -- two independent implementations
    /// have to agree before either is believed.
    #[test]
    fn the_fine_mesher_reproduces_the_benchs_face_and_triangle_counts() {
        let prism = mirror(vec![
            Tile::Solid(protocol::Material::Stone),
            Tile::Solid(protocol::Material::Stone),
        ]);
        assert_eq!(fine_geometry(&prism, 1), (10, 12));
        assert_eq!(fine_geometry(&prism, 2), (32, 24));
        assert_eq!(fine_geometry(&prism, 4), (176, 144));
    }

    /// The client mesher agrees with the offline bench on a world with rim levels in it.
    ///
    /// SPLIT OUT of the prism test deliberately. Three mutation rows are named for this
    /// bench-parity claim -- `side faces ignore the pit that carved them away`, `cross-cell
    /// connectors are dropped and the fine surface cracks` and `greedy tie-break drifts away
    /// from the bench's row order` -- and all three also move the prism counts, so with both
    /// halves in one test every row died on the FIRST assertion and the staircase comparison
    /// they name never ran. KILLED named the test, not the assertion. Keeping them apart is
    /// what makes those rows evidence for the thing they claim.
    #[test]
    fn the_fine_mesher_reproduces_the_benchs_staircase_counts() {
        // The staircase is 4 cells wide, so `rim_level` gives its cells three different
        // world-edge dissolve levels and the client partitions the masks by material AND rim
        // before merging. The bench has no rim, so it merges across all of it. The SURFACE is
        // the same either way -- that is the geometry claim -- while the client's triangle
        // count can only ever be greater, never smaller, because partitioning splits rectangles
        // and never joins them. Both halves are asserted; the prism above has one rim level and
        // pins the triangle counts exactly.
        // The triangle column is pinned EXACTLY, not by an inequality. `>= bench_triangles` let
        // the greedy tie-break drift freely -- the row named for that drift SURVIVED against it,
        // because reordering the merge changes which rectangles form and so the count, while
        // still leaving it above the unpartitioned bench figure. An inequality that every
        // plausible regression satisfies pins nothing.
        for (subdiv, faces, bench_triangles, client_triangles) in
            [(1, 84, 36, 60), (2, 334, 154, 174), (4, 1608, 926, 942)]
        {
            let (drawn, triangles) = fine_geometry(&staircase(), subdiv);
            assert_eq!(
                drawn, faces,
                "fine surface disagrees with the bench at k={subdiv}"
            );
            assert_eq!(
                triangles, client_triangles,
                "k={subdiv}: the client's partitioned triangle count moved"
            );
            assert!(
                triangles >= bench_triangles,
                "k={subdiv}: {triangles} triangles is FEWER than the unpartitioned \
                 {bench_triangles}, which partitioning cannot produce"
            );
        }
    }

    /// The defect that made every k>1 figure wrong: the mesher culled against the DRAWN set, so
    /// a solid cell nothing exposes was treated as air and its neighbour emitted a face sealed
    /// inside the rock. A 3x1x1 prism has a fully-enclosed middle cell only once it is 3 wide in
    /// every axis, so use a 3x3x3 block: exactly one cell is buried, and its six faces must not
    /// be drawn from either side.
    #[test]
    fn buried_rock_contributes_no_faces_from_either_side() {
        let dims = Dims { x: 3, y: 3, z: 3 };
        let block = world(dims, vec![Tile::Solid(protocol::Material::Stone); 27]);
        assert!(
            !is_exposed(&block, [1, 1, 1]),
            "the centre cell of a 3x3x3 block is enclosed"
        );
        // A cube's surface is 6 * 3 * 3 = 54 coarse faces. The enclosed cell adds none of its
        // own and hides none of its neighbours': every one of the 54 is a boundary face.
        assert_eq!(fine_geometry(&block, 1).0, 54);
        // At k=2 the flat sides expand to 4 samples each; the top carries pits, so the total is
        // measured rather than 54 * 4. What must NOT happen is the buried cell contributing.
        let (faces, _) = fine_geometry(&block, 2);
        assert!(
            faces < 54 * 4 + 4 * 4,
            "{faces} faces at k=2 is more than the outer surface can hold -- \
             something inside the block is being drawn"
        );
    }

    /// A mesh-drawn tree cell is drawn by ITS MESH and by nothing else, so it must not be
    /// allowed to hide a terrain face either. Wolf found the consequence by eye on the vehicle
    /// after 10.7 raised the sun: hard dark quads at the base of nearly every trunk at
    /// `--subdiv > 1`, and none at `--subdiv 1`. They were measured to be `rgb(5, 12, 28)` --
    /// exactly `SKY_RGB` -- so they were never shadows or a dark material. They were HOLES, and
    /// rebuilding with `shadow_maps_enabled: false` moved the count by 15 pixels out of 9,700.
    ///
    /// The mechanism: `build_chunk_meshes` skips a mesh-drawn tree cell outright, while
    /// `occludes` knows only "is this cell solid". So the ground cell beneath a trunk asked
    /// whether the cell above it was solid, got yes, and dropped its own top face -- and the
    /// trunk cell that "hid" it emitted nothing, having been skipped. Neither path drew it. The
    /// coarse path never had this defect because a `Cuboid` is a complete six-face box that
    /// culls nothing, which is exactly why `--subdiv 1` looked clean.
    ///
    /// INDEPENDENT ORACLE, deliberately not derived from the mesher: the terrain the mesher
    /// emits around a MESH-DRAWN tree must be identical to the terrain it emits for the same
    /// world with those tree cells EMPTY. The terrain neither draws the tree nor may pretend
    /// the tree hides anything on its behalf. Same defect class as 10.4's "drawn by NEITHER
    /// path" column, one render path over.
    #[test]
    fn a_mesh_drawn_tree_hides_no_terrain_face() {
        let dims = Dims { x: 3, y: 3, z: 8 };
        let at = |x: usize, y: usize, z: usize| x + y * 3 + z * 9;
        let mut tiles = vec![Tile::Empty; 3 * 3 * 8];
        for x in 0..3 {
            for y in 0..3 {
                tiles[at(x, y, 0)] = Tile::Solid(protocol::Material::Stone);
            }
        }
        // A contiguous four-cell trunk standing on the ground at the centre column, which
        // `classify_trunk_column` accepts, so a mesh really does carry it.
        let mut bare = tiles.clone();
        for z in 1..5 {
            tiles[at(1, 1, z)] = Tile::Solid(protocol::Material::TreeTrunk);
            bare[at(1, 1, z)] = Tile::Empty;
        }

        let with_tree = world(dims, tiles);
        let level = dims.z.saturating_sub(1) as i32;
        assert!(
            tree_cover_at(&with_tree, level).covers([1, 1, 1]),
            "the fixture is only meaningful if a mesh actually carries the trunk"
        );

        // A STONE column in the same place is the control, and it is what makes this a
        // discriminating test rather than a tautology: stone above IS drawn by the terrain, so
        // suppressing the face beneath it is correct. The mesh tree is not drawn by the terrain,
        // so suppressing the same face is a hole. Same geometry, opposite correct answers.
        let mut stone = bare.clone();
        for z in 1..5 {
            stone[at(1, 1, z)] = Tile::Solid(protocol::Material::Stone);
        }
        let with_stone = world(dims, stone);

        assert_eq!(
            face_quads(&with_stone, 2, [1, 1, 0], 2, 1),
            0,
            "terrain-drawn stone above SHOULD hide the face beneath it -- if this fails the \
             control is wrong and the assertion below proves nothing"
        );
        assert_eq!(
            face_quads(&with_tree, 2, [1, 1, 0], 2, 1),
            4,
            "the cell beneath a MESH-drawn trunk must keep its top face, ALL FOUR sub-quads of \
             it: the mesh does not draw terrain, so a face suppressed here is drawn by nothing \
             and reads as sky"
        );
    }

    /// The other TWO substituted call sites. `occludes_terrain` replaced `occludes` at three
    /// face-emission decisions -- the `above` top face, the `under` bottom face, and the four
    /// side-axis neighbours -- but the test above reaches only the first. The review found the
    /// bottom and side substitutions shipping with no coverage at all, which is how a fix gets
    /// half-proved: the mechanism is right, and two thirds of it is pinned by nothing.
    ///
    /// Each case carries its own STONE control, for the same reason the top-face test does. Stone
    /// is terrain-drawn, so it SHOULD hide the neighbouring face; a mesh-drawn trunk is not, so
    /// hiding the same face leaves it drawn by nothing. Same geometry, opposite correct answers --
    /// without the control the assertions would merely restate the mesher back to itself.
    #[test]
    fn a_mesh_drawn_tree_hides_neither_the_face_below_it_nor_the_face_beside_it() {
        let dims = Dims { x: 5, y: 3, z: 8 };
        let at = |x: usize, y: usize, z: usize| x + y * 5 + z * 15;
        let mut tiles = vec![Tile::Empty; 5 * 3 * 8];
        for x in 0..5 {
            for y in 0..3 {
                tiles[at(x, y, 0)] = Tile::Solid(protocol::Material::Stone);
            }
        }
        // A mesh-carried trunk at (1,1), and a terrain-drawn stone column at (3,1) as its twin.
        for z in 1..5 {
            tiles[at(1, 1, z)] = Tile::Solid(protocol::Material::TreeTrunk);
            tiles[at(3, 1, z)] = Tile::Solid(protocol::Material::Stone);
        }
        // Caps sitting ON each column: the BOTTOM face of each cap asks `occludes_terrain` about
        // the cell underneath it, which is a mesh trunk in one case and terrain stone in the other.
        tiles[at(1, 1, 5)] = Tile::Solid(protocol::Material::Stone);
        tiles[at(3, 1, 5)] = Tile::Solid(protocol::Material::Stone);
        // Neighbours BESIDE each column at the same height: the +X side face of each asks
        // `occludes_terrain` about the column cell next to it.
        tiles[at(0, 1, 1)] = Tile::Solid(protocol::Material::Stone);
        tiles[at(2, 1, 1)] = Tile::Solid(protocol::Material::Stone);
        // ...capped, so `column_heights` returns `None` for both and each side cell is a full
        // cube. Uncapped they carve to the exposed surface, the face runs shorter than the cell,
        // and the expected sub-quad count stops being `subdiv * subdiv` for a reason that has
        // nothing to do with trees. Both sides carry the cap so the comparison stays like-for-like.
        tiles[at(0, 1, 2)] = Tile::Solid(protocol::Material::Stone);
        tiles[at(2, 1, 2)] = Tile::Solid(protocol::Material::Stone);

        let world = world(dims, tiles);
        let level = dims.z.saturating_sub(1) as i32;
        assert!(
            tree_cover_at(&world, level).covers([1, 1, 1]),
            "the fixture is only meaningful if a mesh actually carries the trunk"
        );

        // --- the `under` site (axis 2, sign -1) ---
        assert_eq!(
            face_quads(&world, 2, [3, 1, 5], 2, -1),
            0,
            "terrain-drawn stone below SHOULD hide the cap's bottom face -- if this control is \
             wrong the assertion below proves nothing"
        );
        assert_eq!(
            face_quads(&world, 2, [1, 1, 5], 2, -1),
            4,
            "the cap resting on a MESH-drawn trunk must keep its bottom face: the trunk mesh does \
             not draw terrain, so a face suppressed here is drawn by nothing and reads as sky"
        );

        // --- the side-neighbour site (axis 0, sign +1) ---
        assert_eq!(
            face_quads(&world, 2, [2, 1, 1], 0, 1),
            0,
            "a terrain-drawn stone column beside it SHOULD hide the +X face -- if this control is \
             wrong the assertion below proves nothing"
        );
        assert_eq!(
            face_quads(&world, 2, [0, 1, 1], 0, 1),
            4,
            "the cell beside a MESH-drawn trunk must keep its +X face, all four sub-quads of it"
        );
    }

    /// How many fine sub-quads the terrain mesher emits for one cell's face on `(axis, sign)`.
    ///
    /// Asked of the emitted masks, not of a face count -- AC12 is about what is DRAWN, and a
    /// count cannot tell a missing face from a differently-merged one. It returns the COUNT and
    /// not a bool for the reason the review found: `any` is true when even one of the
    /// `subdiv * subdiv` sub-quads survives, so a regression dropping three of four at
    /// `--subdiv 2` reads exactly like a healthy face.
    ///
    /// Mask coordinates follow `emit_quad`'s corner mapping, which is NOT the same on every
    /// axis: axis 2 -> (fine x, fine y); axis 0 -> (fine y, fine z); axis 1 -> (fine z, fine x).
    fn face_quads(mirror: &Mirror, subdiv: i32, cell: [i32; 3], axis: usize, sign: i32) -> usize {
        let level = mirror.dims().z.saturating_sub(1) as i32;
        let positions = terrain_positions_at(mirror, level);
        let plane = (cell[axis] + i32::from(sign > 0)) * subdiv;
        let (ua, va) = match axis {
            0 => (1, 2),
            1 => (2, 0),
            _ => (0, 1),
        };
        let us = (cell[ua] * subdiv)..(cell[ua] * subdiv + subdiv);
        let vs = (cell[va] * subdiv)..(cell[va] * subdiv + subdiv);
        build_chunk_meshes(
            mirror,
            &positions,
            subdiv,
            level,
            None,
            &tree_cover_at(mirror, level),
        )
        .values()
        .map(|chunk_mesh| {
            chunk_mesh
                .masks
                .iter()
                .filter(|(key, _)| key.axis == axis && key.sign == sign && key.plane == plane)
                .map(|(_, mask)| {
                    mask.iter()
                        .filter(|(u, v)| us.contains(u) && vs.contains(v))
                        .count()
                })
                .sum::<usize>()
        })
        .sum()
    }

    /// `--subdiv N > 1` draws no `TerrainTile`, which is what broke the capture's draw-set
    /// oracle. Every drawn coarse cell must still be recorded on some chunk, and exactly once.
    #[test]
    fn every_drawn_cell_is_recorded_on_exactly_one_chunk() {
        let mirror = staircase();
        let level = mirror.dims().z.saturating_sub(1) as i32;
        let positions = terrain_positions_at(&mirror, level);
        let mut recorded = Vec::new();
        for (_, chunk_mesh) in build_chunk_meshes(
            &mirror,
            &positions,
            4,
            level,
            None,
            &tree_cover_at(&mirror, level),
        ) {
            recorded.extend(chunk_mesh.cells);
        }
        let unique = recorded.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(recorded.len(), unique.len(), "a cell was recorded twice");
        let drawn = positions.iter().copied().collect::<BTreeSet<_>>();
        assert!(
            drawn.is_subset(&unique),
            "a cell in the draw set reached no mesh at all"
        );
        // The fine mesher draws MORE coarse cells than the coarse draw set, and must: a pit
        // carved into one cell's top uncovers the side of a neighbour that is fully buried at
        // the coarse scale and so never appears in `positions`. Nothing else would emit those
        // faces, and the gap would be a hole in the rock. Every extra must be exactly that.
        for cell in unique.difference(&drawn) {
            assert!(
                !is_exposed(&mirror, *cell),
                "{cell:?} is exposed, so it belongs in the draw set, not outside it"
            );
            assert!(
                [[-1, 0], [1, 0], [0, -1], [0, 1]].iter().any(|[dx, dy]| {
                    let side = [cell[0] + dx, cell[1] + dy, cell[2]];
                    column_heights(&mirror, side, 4, level).is_some()
                }),
                "{cell:?} is buried and has no carved neighbour to uncover it"
            );
        }
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
    fn tree_yaw_varies_between_columns_without_disturbing_the_variant_draw() {
        // The client drew all 265 pines at identity rotation while the bench randomised quarter
        // turns and said why, so the frame approved as candidate D differed from the one the
        // client draws. Three properties, and the third is the one a careless refactor breaks.
        let columns = (0..40).flat_map(|x| (0..40).map(move |y| (x, y)));

        // 1. Deterministic.
        assert_eq!(tree_yaw_hash(7, 11), tree_yaw_hash(7, 11));

        // 2. It actually varies -- all four quarter turns must appear, or the forest still reads
        //    as one repeated tree.
        let turns = columns
            .clone()
            .map(|(x, y)| tree_yaw_hash(x, y) % 4)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            turns,
            BTreeSet::from([0, 1, 2, 3]),
            "every quarter turn must occur across a 40x40 patch"
        );

        // 3. Yaw is an INDEPENDENT draw from variant. Salting one hash to serve both would make
        //    every column of a given species face the same way.
        assert!(
            columns
                .clone()
                .any(|(x, y)| tree_yaw_hash(x, y) % 4 != tree_variant_hash(x, y) % 4),
            "yaw must not be a restatement of the variant draw"
        );

        // 4. The variant draw is UNCHANGED by adding yaw. FNV-1a mixes every byte, so routing the
        //    variant through a salted hash -- even with a zero salt -- silently reshuffles which
        //    pine each column gets. These are the pre-yaw values, computed by hand from FNV-1a
        //    over the two little-endian words alone.
        let mut expected = 0x811C_9DC5_u32;
        for value in [7_u32, 11_u32] {
            for byte in value.to_le_bytes() {
                expected ^= u32::from(byte);
                expected = expected.wrapping_mul(0x0100_0193);
            }
        }
        assert_eq!(
            tree_variant_hash(7, 11),
            expected,
            "the species assignment must be byte-identical to the unsalted hash"
        );
    }

    #[test]
    fn a_gapped_trunk_column_is_rejected_and_falls_back_to_cubes() {
        // The gap is PRE-EXISTING in the snapshot, never produced by digging through the new
        // code. The fallback exists for a world that ALREADY holds a column the mesh rule cannot
        // represent, and a fixture built by the new path would only prove that path agrees with
        // itself. Columns sit 4 apart because `place_trees` keeps trunks 3 apart in Chebyshev and
        // the cover's crown ring reaches one cell -- adjacent fixtures would overlap and the
        // assertions below would be measuring the wrong thing.
        let dims = Dims { x: 5, y: 1, z: 7 };
        let mut tiles = vec![Tile::Empty; 5 * 7];
        let at = |x: usize, z: usize| x + z * 5;
        for z in 0..3 {
            tiles[at(0, z)] = Tile::Solid(Material::TreeTrunk);
        }
        // Same base, same top, one cell missing in the middle: min and max are IDENTICAL to a
        // whole four-cell column, so height alone reads 5 and the rule accepted it before.
        for z in [0, 1, 3] {
            tiles[at(4, z)] = Tile::Solid(Material::TreeTrunk);
        }
        let mirror = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims,
            tiles,
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap();

        let level = 6;
        assert_eq!(
            tree_meshes(&mirror, level)
                .iter()
                .map(|(base, _)| *base)
                .collect::<Vec<_>>(),
            vec![[0, 0, 0]],
            "only the contiguous column may be meshed"
        );
        // Spelled out so the discriminator is visible: at the SAME extent, an ungapped column is
        // accepted. The extent is not what rejects the gapped one -- the cell count is.
        assert!(
            classify_trunk_column(4, 0, 0, 3, 4, level).is_some(),
            "a four-cell column spanning z0..z3 is representable, so extent is not the rejector"
        );

        let cover = tree_cover_at(&mirror, level);
        let positions = terrain_positions_at(&mirror, level);
        for z in [0, 1, 3] {
            assert!(
                !cover.covers([4, 0, z]),
                "no mesh draws the gapped column at z {z}"
            );
            assert!(
                positions.contains(&[4, 0, z]),
                "a tree cell no mesh draws must fall back to the cube path, not vanish"
            );
        }
        for z in 0..3 {
            assert!(
                !positions.contains(&[0, 0, z]),
                "a meshed tree cell must not ALSO be drawn as a cube"
            );
        }
    }

    #[test]
    fn trunk_columns_choose_the_height_matched_mesh_and_keep_the_base_below_the_slice() {
        let trees = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 3, y: 1, z: 7 },
            tiles: vec![
                // z = 0
                Tile::Solid(Material::TreeTrunk),
                Tile::Solid(Material::TreeTrunk),
                Tile::Solid(Material::TreeTrunk),
                // z = 1
                Tile::Solid(Material::TreeTrunk),
                Tile::Solid(Material::TreeTrunk),
                Tile::Solid(Material::TreeTrunk),
                // z = 2
                Tile::Solid(Material::TreeTrunk),
                Tile::Solid(Material::TreeTrunk),
                Tile::Solid(Material::TreeTrunk),
                // z = 3
                Tile::Empty,
                Tile::Solid(Material::TreeTrunk),
                Tile::Solid(Material::TreeTrunk),
                // z = 4
                Tile::Empty,
                Tile::Empty,
                Tile::Solid(Material::TreeTrunk),
                // z = 5
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                // z = 6
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

        let fixture_heights = (0..3)
            .map(|x| {
                let trunk_levels = (0..7)
                    .filter(|&z| {
                        terrain_material_at(&trees, [x, 0, z]) == Some(Material::TreeTrunk)
                    })
                    .collect::<Vec<_>>();
                trunk_levels.last().unwrap() - trunk_levels.first().unwrap() + 2
            })
            .collect::<Vec<_>>();
        assert_eq!(
            fixture_heights,
            vec![4, 5, 6],
            "fixture must express three mesh heights"
        );
        assert!(
            tree_variant_hash(1, 0).is_multiple_of(2),
            "the height-five fixture column selects Tree02 only when its verified hash is even"
        );

        let columns = tree_meshes(&trees, 4);
        assert_eq!(
            columns,
            vec![
                ([0, 0, 0], TreeVariant::Tree01),
                ([1, 0, 0], TreeVariant::Tree02),
                ([2, 0, 0], TreeVariant::Tree04R),
            ],
            "the literal trunk heights map to the three signed-off mesh heights"
        );
    }

    #[test]
    fn a_dirty_crown_cell_retires_its_owning_tree_mesh() {
        let base = [10, 20, 7];
        assert!(
            tree_mesh_might_cover(base, [9, 19, 11]),
            "a diagonally offset crown cell must rebuild the mesh after it becomes empty"
        );
        assert!(
            tree_mesh_might_cover(base, [10, 20, 7]),
            "the trunk base remains part of its tree mesh"
        );
        assert!(
            !tree_mesh_might_cover(base, [8, 20, 11]),
            "a tile outside the literal one-cell crown ring must not rebuild this tree"
        );
        assert!(
            !tree_mesh_might_cover(base, [10, 20, 6]),
            "terrain below the trunk base belongs to the ground, not the tree mesh"
        );
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
