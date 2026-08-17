#![forbid(unsafe_code)]

use bevy::color::ColorToPacked;
use bevy::{
    MinimalPlugins,
    app::App,
    ecs::system::RunSystemOnce,
    prelude::{
        Assets, Entity as BevyEntity, Mesh, Mesh3d, MeshMaterial3d, PointLight, StandardMaterial,
        Transform,
    },
};
use client_core::Mirror;
use gui::{
    atmosphere::{Atmosphere, SNOWFLAKE_COUNT, STAR_COUNT, setup_atmosphere},
    ingest::{MirrorResource, ProjectionWork, projection_systems, reconcile_projection},
    project::{
        ClientLocal, ProjectedItem, SnowCap, TerrainTile, WorldProjected, setup_projection_assets,
    },
    transform::world_to_render,
};
use protocol::{
    Delta, Dims, Entity, EntityKind, Item, JobState, Material, MessageType, Snapshot, Speed, Tile,
    TileChange,
};

fn headless_app(snapshot: Snapshot) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .insert_resource(MirrorResource(Mirror::from_snapshot(snapshot).unwrap()))
        .insert_resource(ProjectionWork {
            snapshot: true,
            dirty_tiles: Default::default(),
        })
        .add_systems(bevy::app::Startup, setup_projection_assets);
    projection_systems(&mut app);
    app
}

fn apply_delta(app: &mut App, delta: Delta) {
    let (dirty_tiles, tick) = {
        let mut mirror = app.world_mut().resource_mut::<MirrorResource>();
        mirror.0.apply_delta(delta);
        (mirror.0.changes().tiles.clone(), mirror.0.tick())
    };
    {
        let mut work = app.world_mut().resource_mut::<ProjectionWork>();
        work.dirty_tiles.extend(dirty_tiles);
    }
    app.world_mut()
        .resource_mut::<gui::blend::TickClock>()
        .observe_tick(tick);
}

fn apply_snapshot(app: &mut App, snapshot: Snapshot) {
    let tick = {
        let mut mirror = app.world_mut().resource_mut::<MirrorResource>();
        mirror.0.apply_snapshot(snapshot).unwrap();
        mirror.0.tick()
    };
    {
        let mut work = app.world_mut().resource_mut::<ProjectionWork>();
        work.snapshot = true;
        work.dirty_tiles.clear();
    }
    app.world_mut()
        .resource_mut::<gui::blend::TickClock>()
        .reset(tick);
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
    delta_at(1, tiles, entities)
}

fn delta_at(tick: u64, tiles: Vec<TileChange>, entities: Vec<Entity>) -> Delta {
    Delta {
        msg_type: MessageType::Delta,
        tick,
        tiles,
        entities,
        designations: Vec::new(),
        zones: Vec::new(),
        items: Vec::new(),
        speed: Speed::Normal,
    }
}

fn projected_translation(app: &mut App, id: u32) -> bevy::prelude::Vec3 {
    app.world_mut()
        .query::<(&WorldProjected, &Transform)>()
        .iter(app.world())
        .find_map(|(projected, transform)| (projected.0 == id).then_some(transform.translation))
        .expect("the wire entity must have a projection")
}

fn dwarf(id: u32, pos: [i32; 3]) -> Entity {
    Entity {
        id,
        kind: EntityKind::Dwarf,
        pos,
        state: JobState::Idle,
        light: None,
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
fn projection_pipeline_blends_at_a_midpoint() {
    let id = 71;
    let mut app = headless_app(snapshot(
        vec![Tile::Empty, Tile::Empty],
        vec![dwarf(id, [0, 0, 0])],
    ));
    app.update();

    apply_delta(&mut app, delta(vec![], vec![dwarf(id, [2, 0, 0])]));
    app.world_mut()
        .resource_mut::<gui::blend::TickClock>()
        .advance(0.01);
    app.update();

    let midpoint = projected_translation(&mut app, id);
    assert!(
        midpoint.x > 0.0 && midpoint.x < 2.0,
        "the shared projection schedule must run the blend: {midpoint:?}"
    );
}

#[test]
fn later_production_reconciliation_does_not_clobber_a_blended_translation() {
    let id = 72;
    let mut app = headless_app(snapshot(
        vec![Tile::Empty, Tile::Empty],
        vec![dwarf(id, [0, 0, 0])],
    ));
    app.update();

    apply_delta(&mut app, delta(vec![], vec![dwarf(id, [2, 0, 0])]));
    app.world_mut()
        .resource_mut::<gui::blend::TickClock>()
        .advance(0.01);
    app.update();
    let midpoint = projected_translation(&mut app, id);
    assert!(midpoint.x > 0.0 && midpoint.x < 2.0);

    app.world_mut()
        .run_system_once(reconcile_projection)
        .expect("production reconciliation must run");
    assert_eq!(
        projected_translation(&mut app, id),
        midpoint,
        "reconciliation must leave translation to the blend after spawn"
    );
}

#[test]
fn snapshot_rewind_snaps_at_a_mid_blend_clock() {
    let id = 73;
    let mut app = headless_app(snapshot(
        vec![Tile::Empty, Tile::Empty],
        vec![dwarf(id, [0, 0, 0])],
    ));
    app.update();

    apply_delta(&mut app, delta(vec![], vec![dwarf(id, [2, 0, 0])]));
    app.world_mut()
        .resource_mut::<gui::blend::TickClock>()
        .advance(0.01);
    app.update();
    let midpoint = projected_translation(&mut app, id);
    assert!(midpoint.x > 0.0 && midpoint.x < 2.0);

    let mut rewind = snapshot(vec![Tile::Empty, Tile::Empty], vec![dwarf(id, [19, 0, 0])]);
    rewind.tick = 2;
    apply_snapshot(&mut app, rewind);
    app.world_mut()
        .resource_mut::<gui::blend::TickClock>()
        .advance(0.01);
    app.update();

    assert_eq!(
        projected_translation(&mut app, id),
        world_to_render([19, 0, 0]),
        "a snapshot must snap even while the clock is half way through an interval"
    );
}

#[test]
fn flickered_light_survives_a_later_production_reconciliation() {
    let id = 74;
    let emitter = Entity {
        id,
        kind: EntityKind::Torch,
        pos: [0, 0, 0],
        state: JobState::Idle,
        light: Some(protocol::LightKind::Torch),
    };
    let mut app = headless_app(snapshot(vec![Tile::Empty, Tile::Empty], vec![emitter]));
    app.update();

    let flickered = app
        .world_mut()
        .query::<(&WorldProjected, &PointLight)>()
        .iter(app.world())
        .find_map(|(projected, light)| (projected.0 == id).then_some(light.intensity))
        .expect("the emitter must have a point light");
    assert_ne!(
        flickered,
        gui::appearance::light_properties(protocol::LightKind::Torch).intensity,
        "the shared projection schedule must run the flicker"
    );

    app.world_mut()
        .run_system_once(reconcile_projection)
        .expect("production reconciliation must run");
    let after_reconcile = app
        .world_mut()
        .query::<(&WorldProjected, &PointLight)>()
        .iter(app.world())
        .find_map(|(projected, light)| (projected.0 == id).then_some(light.intensity))
        .expect("reconciliation must retain the point light");
    assert_eq!(after_reconcile, flickered);
}

#[test]
fn snapshot_rebuild_reaches_reconcile_even_when_changes_are_empty() {
    let mut app = headless_app(snapshot(vec![Tile::Empty, Tile::Empty], Vec::new()));
    app.update();
    apply_snapshot(
        &mut app,
        snapshot(vec![Tile::Solid(Material::Ice), Tile::Empty], Vec::new()),
    );
    assert!(
        app.world()
            .resource::<MirrorResource>()
            .0
            .changes()
            .tiles
            .is_empty()
    );
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

    apply_delta(
        &mut app,
        delta(
            vec![TileChange {
                pos: [0, 0, 0],
                tile: Tile::Empty,
            }],
            Vec::new(),
        ),
    );
    assert_eq!(
        app.world().resource::<MirrorResource>().0.changes().tiles,
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

    apply_delta(
        &mut app,
        delta(
            vec![TileChange {
                pos: [2, 2, 2],
                tile: Tile::Empty,
            }],
            Vec::new(),
        ),
    );
    app.update();

    let terrain = projected_scene(&mut app)
        .into_iter()
        .filter_map(|(_, terrain, _)| terrain)
        .collect::<Vec<_>>();
    assert!(terrain.contains(&[2, 2, 1]));
    assert_eq!(
        terrain,
        gui::project::terrain_positions(&app.world().resource::<MirrorResource>().0)
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
    apply_delta(
        &mut app,
        delta(
            vec![TileChange {
                pos: [2, 0, 0],
                tile: Tile::Solid(Material::Ice),
            }],
            Vec::new(),
        ),
    );
    assert!(
        app.world()
            .resource::<MirrorResource>()
            .0
            .changes()
            .tiles
            .is_empty()
    );
    app.update();

    let full_repaint = gui::project::terrain_positions(&app.world().resource::<MirrorResource>().0);
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
    app.world_mut().resource_mut::<ProjectionWork>().snapshot = true;
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
    // A 56-wide world so the tested tile sits at [27, 27, 0], outside the 26-tile world-edge
    // dissolve. On a small toy every tile is boundary and every colour is correctly sky.
    const SPAN: usize = 56;
    let dims = Dims {
        x: SPAN as u32,
        y: SPAN as u32,
        z: 2,
    };
    let mut tiles = vec![Tile::Empty; SPAN * SPAN * 2];
    tiles[27 + 27 * SPAN] = Tile::Solid(Material::Stone);
    let mut app = headless_app(snapshot_with_dims(dims, tiles, Vec::new()));

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

    assert_eq!(colors, vec![[60, 70, 92], [146, 158, 184]]);
    let caps = app
        .world_mut()
        .query::<(&SnowCap, Option<&ClientLocal>)>()
        .iter(app.world())
        .map(|(cap, local)| {
            assert!(
                local.is_some(),
                "snow caps belong to the client-local partition"
            );
            cap.0
        })
        .collect::<Vec<_>>();
    assert_eq!(
        caps,
        vec![[27, 27, 0]],
        "the capped tile needs one snow slab"
    );
}

#[test]
fn atmosphere_entities_are_client_local_and_never_world_projected() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Assets<bevy::image::Image>>()
        .add_systems(bevy::app::Startup, setup_atmosphere);
    app.update();

    let mut atmosphere =
        app.world_mut()
            .query::<(&Atmosphere, Option<&ClientLocal>, Option<&WorldProjected>)>();
    let entities = atmosphere.iter(app.world()).collect::<Vec<_>>();
    // Stars + the aurora curtain + snowflakes. Pinned exactly: a count threshold would
    // tolerate the marker being dropped from a whole class of atmosphere entity.
    assert_eq!(
        entities.len(),
        STAR_COUNT + 1 + SNOWFLAKE_COUNT,
        "the atmosphere spawn count is pinned"
    );
    assert!(
        entities
            .iter()
            .all(|(_, local, projected)| local.is_some() && projected.is_none())
    );
}

#[test]
fn empty_tile_delta_leaves_deterministic_client_local_chips_and_snapshot_clears_them() {
    let mut app = headless_app(snapshot(
        vec![Tile::Solid(Material::Ice), Tile::Empty],
        Vec::new(),
    ));
    app.update();
    apply_delta(
        &mut app,
        delta(
            vec![TileChange {
                pos: [0, 0, 0],
                tile: Tile::Empty,
            }],
            Vec::new(),
        ),
    );
    app.update();
    let mut chips = app.world_mut().query::<(
        &gui::project::DigChip,
        Option<&ClientLocal>,
        Option<&WorldProjected>,
    )>();
    assert_eq!(
        chips.iter(app.world()).count(),
        gui::project::CHIPS_PER_TILE
    );
    assert!(
        chips
            .iter(app.world())
            .all(|(_, local, world)| local.is_some() && world.is_none())
    );

    apply_snapshot(
        &mut app,
        snapshot(vec![Tile::Solid(Material::Ice), Tile::Empty], Vec::new()),
    );
    app.update();
    assert_eq!(chips.iter(app.world()).count(), 0);
}

#[test]
fn named_dig_site_stays_inside_the_boot_camera_frame() {
    let rig = gui::camera::CameraRig::new([64, 64, 9]);
    for x in 58..=64 {
        for y in 68..=69 {
            let point = rig
                .project_world_point([x, y, 9])
                .expect("dig site must be in front of the camera");
            assert!((0.0..=1.0).contains(&point.x) && (0.0..=1.0).contains(&point.y));
        }
    }
}
