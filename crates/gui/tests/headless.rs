#![forbid(unsafe_code)]

use bevy::color::ColorToPacked;
use bevy::{
    MinimalPlugins,
    app::{App, Update},
    ecs::system::{Commands, Query, Res, ResMut},
    prelude::{
        Assets, Entity as BevyEntity, Mesh, Mesh3d, MeshMaterial3d, PointLight, Resource,
        StandardMaterial, Transform, Without,
    },
};
use client_core::Mirror;
use gui::{
    atmosphere::setup_atmosphere,
    project::{
        ClientLocal, ProjectedItem, ProjectionAssets, SnowCap, TerrainQuery, TerrainTile,
        WorldProjected, reconcile, setup_projection_assets,
    },
    transform::world_to_render,
};
use protocol::{
    Delta, Dims, Entity, EntityKind, Item, JobState, Material, MessageType, Snapshot, Speed, Tile,
    TileChange,
};

#[derive(Resource)]
struct TestMirror(Mirror);

#[derive(Resource)]
struct ProjectionWork {
    rebuild_terrain: bool,
}

fn headless_app(snapshot: Snapshot) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .insert_resource(TestMirror(Mirror::from_snapshot(snapshot).unwrap()))
        .insert_resource(ProjectionWork {
            rebuild_terrain: true,
        })
        .add_systems(bevy::app::Startup, setup_projection_assets)
        .add_systems(Update, reconcile_from_mirror);
    app
}

fn reconcile_from_mirror(
    mut commands: Commands,
    mirror: Res<TestMirror>,
    mut work: ResMut<ProjectionWork>,
    projected: Query<(BevyEntity, &WorldProjected), Without<TerrainTile>>,
    terrain: TerrainQuery,
    assets: Option<Res<ProjectionAssets>>,
) {
    let rebuild_terrain = std::mem::take(&mut work.rebuild_terrain);
    reconcile(
        &mut commands,
        &mirror.0,
        rebuild_terrain,
        &mirror.0.changes().tiles,
        &projected,
        &terrain,
        assets.as_deref(),
    );
}

fn snapshot(tiles: Vec<Tile>, entities: Vec<Entity>) -> Snapshot {
    snapshot_with_dims(Dims { x: 2, y: 1, z: 1 }, tiles, entities)
}

fn snapshot_with_dims(dims: Dims, tiles: Vec<Tile>, entities: Vec<Entity>) -> Snapshot {
    Snapshot {
        msg_type: MessageType::Snapshot,
        dims,
        tiles,
        entities,
        designations: Vec::new(),
        zones: Vec::new(),
        items: Vec::new(),
        speed: Speed::Normal,
        tick: 0,
    }
}

fn delta(tiles: Vec<TileChange>, entities: Vec<Entity>) -> Delta {
    Delta {
        msg_type: MessageType::Delta,
        tick: 1,
        tiles,
        entities,
        designations: Vec::new(),
        zones: Vec::new(),
        items: Vec::new(),
        speed: Speed::Normal,
    }
}

fn projected_scene(app: &mut App) -> Vec<(u32, Option<[i32; 3]>, [i32; 3])> {
    let mut scene = app
        .world_mut()
        .query::<(&WorldProjected, Option<&TerrainTile>, &Transform)>()
        .iter(app.world())
        .map(|(marker, terrain, transform)| {
            (
                marker.0,
                terrain.map(|tile| tile.0),
                gui::transform::render_to_world(transform.translation),
            )
        })
        .collect::<Vec<_>>();
    scene.sort_unstable();
    scene
}

#[test]
fn snapshot_rebuild_reaches_reconcile_even_when_changes_are_empty() {
    let mut app = headless_app(snapshot(vec![Tile::Empty, Tile::Empty], Vec::new()));
    app.update();
    app.world_mut()
        .resource_mut::<TestMirror>()
        .0
        .apply_snapshot(snapshot(
            vec![Tile::Solid(Material::Ice), Tile::Empty],
            Vec::new(),
        ))
        .unwrap();
    assert!(
        app.world()
            .resource::<TestMirror>()
            .0
            .changes()
            .tiles
            .is_empty()
    );
    app.world_mut()
        .resource_mut::<ProjectionWork>()
        .rebuild_terrain = true;

    app.update();

    let terrain = app
        .world_mut()
        .query::<&TerrainTile>()
        .iter(app.world())
        .count();
    assert_eq!(terrain, 1, "a reset snapshot must fully rebuild terrain");
}

#[test]
fn terrain_ids_never_satisfy_a_simulation_id_lookup() {
    let dwarf_zero = Entity {
        id: 0,
        kind: EntityKind::Dwarf,
        pos: [1, 0, 0],
        state: JobState::Idle,
        light: None,
    };
    let dwarf_one = Entity {
        id: 1,
        kind: EntityKind::Dwarf,
        pos: [0, 0, 0],
        state: JobState::Idle,
        light: None,
    };
    let mut app = headless_app(snapshot(
        vec![Tile::Solid(Material::Ice), Tile::Empty],
        vec![dwarf_zero, dwarf_one],
    ));

    app.update();
    let dynamic = app
        .world_mut()
        .query::<(BevyEntity, &WorldProjected, Option<&TerrainTile>)>()
        .iter(app.world())
        .find_map(|(entity, marker, terrain)| {
            (marker.0 == 0 && terrain.is_none()).then_some(entity)
        })
        .expect("first reconciliation must create dwarf 0");
    app.world_mut().entity_mut(dynamic).despawn();
    // A second real reconciliation must not use terrain 0 as the lookup result for
    // dwarf 0. This recreates the second-frame collision without relying on query order.
    app.update();

    let mut terrain_at_origin = 0;
    let mut dwarf_at_position = 0;
    let mut second_dwarf_at_position = 0;
    let mut query = app
        .world_mut()
        .query::<(&WorldProjected, Option<&TerrainTile>, &Transform)>();
    for (marker, terrain, transform) in query.iter(app.world()) {
        if terrain.is_some() && transform.translation == world_to_render([0, 0, 0]) {
            terrain_at_origin += 1;
        }
        if terrain.is_none() && marker.0 == 0 && transform.translation == world_to_render([1, 0, 0])
        {
            dwarf_at_position += 1;
        }
        if terrain.is_none() && marker.0 == 1 && transform.translation == world_to_render([0, 0, 0])
        {
            second_dwarf_at_position += 1;
        }
    }
    assert_eq!(
        terrain_at_origin, 1,
        "the origin terrain cube must stay at the origin"
    );
    assert_eq!(
        dwarf_at_position, 1,
        "dwarf 0 needs its own projected entity"
    );
    assert_eq!(
        second_dwarf_at_position, 1,
        "dwarf 1 must remain keyed by its own simulation id"
    );
}

#[test]
fn dirty_delta_reprojects_only_the_dirty_terrain_cube() {
    let mut app = headless_app(snapshot(
        vec![Tile::Solid(Material::Ice), Tile::Solid(Material::Ice)],
        Vec::new(),
    ));
    app.update();
    let before = projected_scene(&mut app);
    let unchanged = before
        .iter()
        .find(|(_, terrain, _)| *terrain == Some([1, 0, 0]))
        .copied()
        .expect("the unaffected terrain cube must be projected first");

    app.world_mut()
        .resource_mut::<TestMirror>()
        .0
        .apply_delta(delta(
            vec![TileChange {
                pos: [0, 0, 0],
                tile: Tile::Empty,
            }],
            Vec::new(),
        ));
    assert_eq!(
        app.world().resource::<TestMirror>().0.changes().tiles,
        vec![[0, 0, 0]]
    );
    app.update();

    assert_eq!(projected_scene(&mut app), vec![unchanged]);
}

#[test]
fn dirty_delta_reprojects_newly_exposed_neighbours() {
    let mut app = headless_app(snapshot_with_dims(
        Dims { x: 5, y: 5, z: 5 },
        vec![Tile::Solid(Material::Ice); 125],
        Vec::new(),
    ));
    app.update();
    assert!(
        !projected_scene(&mut app)
            .iter()
            .any(|(_, terrain, _)| *terrain == Some([2, 2, 1])),
        "the interior neighbour is initially hidden"
    );

    app.world_mut()
        .resource_mut::<TestMirror>()
        .0
        .apply_delta(delta(
            vec![TileChange {
                pos: [2, 2, 2],
                tile: Tile::Empty,
            }],
            Vec::new(),
        ));
    app.update();

    let terrain = projected_scene(&mut app)
        .into_iter()
        .filter_map(|(_, terrain, _)| terrain)
        .collect::<Vec<_>>();
    assert!(terrain.contains(&[2, 2, 1]));
    assert_eq!(
        terrain,
        gui::project::terrain_positions(&app.world().resource::<TestMirror>().0)
    );
    let newly_exposed = app
        .world_mut()
        .query::<(
            &TerrainTile,
            Option<&Mesh3d>,
            Option<&MeshMaterial3d<StandardMaterial>>,
        )>()
        .iter(app.world())
        .find(|(tile, _, _)| tile.0 == [2, 2, 1])
        .expect("the newly exposed neighbour must be projected");
    assert!(
        newly_exposed.1.is_some() && newly_exposed.2.is_some(),
        "a terrain cube exposed by a delta must carry its render mesh and material"
    );
}

#[test]
fn out_of_bounds_dirty_tiles_leave_the_projection_equal_to_a_full_repaint() {
    let mut app = headless_app(snapshot(
        vec![Tile::Solid(Material::Ice), Tile::Empty],
        Vec::new(),
    ));
    app.update();
    app.world_mut()
        .resource_mut::<TestMirror>()
        .0
        .apply_delta(delta(
            vec![TileChange {
                pos: [2, 0, 0],
                tile: Tile::Solid(Material::Ice),
            }],
            Vec::new(),
        ));
    assert!(
        app.world()
            .resource::<TestMirror>()
            .0
            .changes()
            .tiles
            .is_empty()
    );
    app.update();

    let full_repaint = gui::project::terrain_positions(&app.world().resource::<TestMirror>().0);
    let projected_terrain = projected_scene(&mut app)
        .into_iter()
        .filter_map(|(_, terrain, _)| terrain)
        .collect::<Vec<_>>();
    assert_eq!(projected_terrain, full_repaint);
}

#[test]
fn despawning_world_projection_then_reconciling_recreates_the_same_scene() {
    let dwarf = Entity {
        id: 11,
        kind: EntityKind::Dwarf,
        pos: [1, 0, 0],
        state: JobState::Idle,
        light: None,
    };
    let mut app = headless_app(snapshot(
        vec![Tile::Solid(Material::Ice), Tile::Empty],
        vec![dwarf],
    ));
    app.update();
    let expected = projected_scene(&mut app);
    let projected = app
        .world_mut()
        .query::<(BevyEntity, &WorldProjected)>()
        .iter(app.world())
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    for entity in projected {
        app.world_mut().entity_mut(entity).despawn();
    }
    assert!(projected_scene(&mut app).is_empty());
    app.world_mut()
        .resource_mut::<ProjectionWork>()
        .rebuild_terrain = true;
    app.update();

    assert_eq!(projected_scene(&mut app), expected);
}

#[test]
fn world_and_client_local_markers_are_a_structural_partition() {
    let mut app = headless_app(snapshot(
        vec![Tile::Solid(Material::Ice), Tile::Empty],
        Vec::new(),
    ));
    app.world_mut().spawn(ClientLocal);
    app.update();

    let mut projected = app
        .world_mut()
        .query::<(&WorldProjected, Option<&ClientLocal>)>();
    assert!(
        projected
            .iter(app.world())
            .all(|(_, local)| local.is_none())
    );
    let mut local = app
        .world_mut()
        .query::<(&ClientLocal, Option<&WorldProjected>)>();
    assert!(
        local
            .iter(app.world())
            .all(|(_, projected)| projected.is_none())
    );
}

#[test]
fn recorded_camp_snapshot_projects_exactly_five_warm_point_lights() {
    let entities = (0..4)
        .map(|id| Entity {
            id,
            kind: EntityKind::Torch,
            pos: [60 + id as i32, 64, 9],
            state: JobState::Idle,
            light: Some(protocol::LightKind::Torch),
        })
        .chain(std::iter::once(Entity {
            id: 4,
            kind: EntityKind::Campfire,
            pos: [64, 64, 9],
            state: JobState::Idle,
            light: Some(protocol::LightKind::Campfire),
        }))
        .chain(std::iter::once(Entity {
            id: 5,
            kind: EntityKind::Dwarf,
            pos: [64, 65, 9],
            state: JobState::Idle,
            light: None,
        }))
        .collect();
    let mut app = headless_app(snapshot(vec![Tile::Empty, Tile::Empty], entities));

    app.update();

    let lights = app
        .world_mut()
        .query::<&PointLight>()
        .iter(app.world())
        .collect::<Vec<_>>();
    assert_eq!(
        lights.len(),
        5,
        "only wire-declared emitters receive lights"
    );
    assert!(lights.iter().all(|light| {
        let channels = light.color.to_srgba().to_u8_array_no_alpha();
        channels[0] > channels[2]
    }));
}

#[test]
fn snapshot_item_receives_a_render_mesh() {
    let mut snapshot = snapshot(vec![Tile::Empty, Tile::Empty], Vec::new());
    snapshot.items = vec![Item {
        id: 42,
        pos: [1, 0, 0],
    }];
    let mut app = headless_app(snapshot);

    app.update();

    let item = app
        .world_mut()
        .query::<(&WorldProjected, &ProjectedItem, Option<&Mesh3d>)>()
        .iter(app.world())
        .find(|(projected, _, _)| projected.0 == 42)
        .expect("the snapshot item must be projected");
    assert_eq!(item.1.0, 42);
    assert!(item.2.is_some(), "a projected item must carry a mesh");
}

#[test]
fn capped_stone_keeps_its_bare_cube_beneath_a_snow_cap() {
    let mut app = headless_app(snapshot(
        vec![Tile::Solid(Material::Stone), Tile::Empty],
        Vec::new(),
    ));

    app.update();

    let handles = app
        .world_mut()
        .query::<&MeshMaterial3d<StandardMaterial>>()
        .iter(app.world())
        .map(|material| material.0.clone())
        .collect::<Vec<_>>();
    let materials = app.world().resource::<Assets<StandardMaterial>>();
    let mut colors = handles
        .iter()
        .map(|handle| {
            materials
                .get(handle)
                .expect("projected materials must be loaded")
                .base_color
                .to_srgba()
                .to_u8_array_no_alpha()
        })
        .collect::<Vec<_>>();
    colors.sort_unstable();

    assert_eq!(colors, vec![[40, 57, 82], [118, 139, 157]]);
    let caps = app
        .world_mut()
        .query::<&SnowCap>()
        .iter(app.world())
        .map(|cap| cap.0)
        .collect::<Vec<_>>();
    assert_eq!(caps, vec![[0, 0, 0]], "the capped tile needs one snow slab");
}

#[test]
fn atmosphere_entities_are_client_local_and_never_world_projected() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_systems(bevy::app::Startup, setup_atmosphere);
    app.update();

    let mut atmosphere = app
        .world_mut()
        .query::<(&ClientLocal, Option<&WorldProjected>)>();
    let entities = atmosphere.iter(app.world()).collect::<Vec<_>>();
    assert!(
        entities.len() >= 20,
        "stars, aurora, and restrained snow must be present"
    );
    assert!(entities.iter().all(|(_, projected)| projected.is_none()));
}
