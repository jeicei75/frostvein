#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Independent of `appearance.rs` on purpose: a lantern dimmer than this cannot read as a warm
/// pool, whatever the table says. Deliberately far below the shipped 5,000,000 so it constrains
/// only the dark end and never has to move when the look is tuned.
const LANTERN_VISIBLE_INTENSITY_FLOOR: f32 = 1_000_000.0;

use bevy::color::ColorToPacked;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::{
    MinimalPlugins,
    app::App,
    ecs::system::RunSystemOnce,
    input::ButtonInput,
    prelude::{
        Assets, Entity as BevyEntity, KeyCode, Mesh, Mesh3d, MeshMaterial3d, PointLight,
        StandardMaterial, Text, Transform,
    },
};
use client_core::Mirror;
use gui::{
    atmosphere::{Atmosphere, SNOWFLAKE_COUNT, STAR_COUNT, setup_atmosphere},
    capture::{CaptureState, accumulate_motion},
    ingest::{
        MirrorResource, ProjectionSet, ProjectionWork, SliceReadout, projection_systems,
        reconcile_projection,
    },
    project::{
        ClientLocal, ProjectedItem, SnowCap, TerrainTile, WorldProjected, setup_projection_assets,
    },
    slice::SliceLevel,
    transform::world_to_render,
};
use protocol::{
    Delta, Dims, Entity, EntityKind, Item, JobState, Material, MessageType, Snapshot, Speed, Tile,
    TileChange,
};

fn headless_app(snapshot: Snapshot) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<KeyCode>>()
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

/// Presses a key for exactly one frame. `MinimalPlugins` brings no `InputPlugin`, so nothing ever
/// runs `ButtonInput::clear()` and a pressed key stays JUST-pressed forever — a test that presses
/// once and then updates twice silently moves two levels and asserts against the wrong world.
/// Production is unaffected: `DefaultPlugins` carries `InputPlugin`.
fn press_once(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    // `release` before `clear`: clearing alone drops `just_pressed` but leaves the key in
    // `pressed`, and a later `press` of a key already held registers nothing at all — so the
    // second tap of the same key would silently do nothing.
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.release(key);
    input.clear();
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

fn projected_intensity(app: &mut App, id: u32) -> f32 {
    app.world_mut()
        .query::<(&WorldProjected, &PointLight)>()
        .iter(app.world())
        .find_map(|(projected, light)| (projected.0 == id).then_some(light.intensity))
        .expect("the emitter must have a point light")
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

fn lantern_dwarf(id: u32, pos: [i32; 3]) -> Entity {
    Entity {
        id,
        kind: EntityKind::Dwarf,
        pos,
        state: JobState::Idle,
        light: Some(protocol::LightKind::Lantern),
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
fn keyboard_slice_rebuilds_the_cut_face_and_hides_surface_entities() {
    let dims = Dims { x: 3, y: 3, z: 3 };
    let mut app = headless_app(snapshot_with_dims(
        dims,
        vec![Tile::Solid(Material::Stone); 27],
        vec![dwarf(91, [1, 1, 2]), dwarf(92, [1, 1, 1])],
    ));
    app.update();
    assert_eq!(
        app.world().resource::<SliceLevel>().level(),
        2,
        "the boot frame starts at the world top"
    );

    press_once(&mut app, KeyCode::Comma);

    let terrain = app
        .world_mut()
        .query::<&TerrainTile>()
        .iter(app.world())
        .map(|tile| tile.0)
        .collect::<BTreeSet<_>>();
    let expected = (0..3)
        .flat_map(|x| (0..3).map(move |y| [x, y, 0]))
        .chain((0..3).flat_map(|x| (0..3).map(move |y| [x, y, 1])))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        terrain, expected,
        "the z=1 cut must include its buried floor while hiding z=2 terrain"
    );

    let projected = app
        .world_mut()
        .query::<&WorldProjected>()
        .iter(app.world())
        .map(|marker| marker.0)
        .collect::<BTreeSet<_>>();
    assert!(
        !projected.contains(&91),
        "a surface dwarf must not float above a slice"
    );
    assert!(
        projected.contains(&92),
        "an entity at the slice remains visible"
    );
}

#[test]
fn the_level_readout_is_drawn_on_the_live_path_and_follows_the_cut() {
    // AC9's whole mechanism. Both its systems lived in `run()` only, where no test could reach
    // them: deleting them left the suite green while "always know which level you are on" was
    // gone. They now register through `projection_systems`, so this test holds them.
    let dims = Dims { x: 3, y: 3, z: 3 };
    // Rock only on the bottom two levels: z 2 is open sky, so a cut at z 1 has nothing above it.
    let mut tiles = vec![Tile::Solid(Material::Stone); 18];
    tiles.extend(vec![Tile::Empty; 9]);
    let mut app = headless_app(snapshot_with_dims(dims, tiles, Vec::new()));
    app.update();

    let readout = |app: &mut App| {
        app.world_mut()
            .query::<(&Text, &SliceReadout)>()
            .iter(app.world())
            .map(|(text, _)| text.0.clone())
            .collect::<Vec<_>>()
    };

    // Independent oracle: the expected strings are written here, not read back from SliceLevel.
    assert_eq!(
        readout(&mut app),
        vec!["Slice: z 2/2 — surface".to_string()],
        "the readout must exist at boot and name the level"
    );

    press_once(&mut app, KeyCode::Comma);
    assert_eq!(
        readout(&mut app),
        vec!["Slice: z 1/2 — surface".to_string()],
        "z 1 has only empty sky above it, so it is not underground"
    );

    press_once(&mut app, KeyCode::Comma);
    assert_eq!(
        readout(&mut app),
        vec!["Slice: z 0/2 — underground".to_string()],
        "z 0 is covered by the rock at z 1"
    );
}

#[test]
fn items_above_the_cut_are_hidden_with_the_entities() {
    // The entity filter is defended by the sabotage table; the item filter was not, and removing
    // both item filters left the whole suite green.
    let dims = Dims { x: 3, y: 3, z: 3 };
    let mut snapshot = snapshot_with_dims(
        dims,
        vec![Tile::Solid(Material::Stone); 27],
        vec![dwarf(91, [1, 1, 1])],
    );
    snapshot.items = vec![
        Item {
            id: 501,
            pos: [0, 0, 2],
        },
        Item {
            id: 502,
            pos: [0, 1, 1],
        },
    ];
    let mut app = headless_app(snapshot);
    app.update();

    press_once(&mut app, KeyCode::Comma);

    let projected = app
        .world_mut()
        .query::<&WorldProjected>()
        .iter(app.world())
        .map(|marker| marker.0)
        .collect::<BTreeSet<_>>();
    assert!(
        !projected.contains(&501),
        "an item above the cut must not float over the slice"
    );
    assert!(
        projected.contains(&502),
        "an item at or below the cut stays visible"
    );
}

#[test]
fn sliced_view_does_not_spawn_dig_chips_above_the_cut() {
    let mut app = headless_app(snapshot_with_dims(
        Dims { x: 1, y: 1, z: 2 },
        vec![Tile::Solid(Material::Stone); 2],
        Vec::new(),
    ));
    app.update();
    press_once(&mut app, KeyCode::Comma);

    apply_delta(
        &mut app,
        delta(
            vec![TileChange {
                pos: [0, 0, 1],
                tile: Tile::Empty,
            }],
            Vec::new(),
        ),
    );
    app.update();

    let chips = app
        .world_mut()
        .query::<&gui::project::DigChip>()
        .iter(app.world())
        .count();
    assert_eq!(
        chips, 0,
        "a later dig above the selected level must not leave floating debris"
    );
}

#[test]
fn top_slice_is_the_full_depth_draw_set_and_cannot_rise_above_the_world() {
    let mut app = headless_app(snapshot_with_dims(
        Dims { x: 3, y: 3, z: 3 },
        vec![Tile::Solid(Material::Stone); 27],
        Vec::new(),
    ));
    app.update();
    press_once(&mut app, KeyCode::Period);

    assert_eq!(app.world().resource::<SliceLevel>().level(), 2);
    let terrain = app
        .world_mut()
        .query::<&TerrainTile>()
        .iter(app.world())
        .map(|tile| tile.0)
        .collect::<BTreeSet<_>>();
    let expected = (0..3)
        .flat_map(|x| (0..3).flat_map(move |y| (0..3).map(move |z| [x, y, z])))
        .filter(|position| *position != [1, 1, 1])
        .collect::<BTreeSet<_>>();
    assert_eq!(
        terrain, expected,
        "the top level keeps the complete full-depth boundary, with no above-world slice"
    );
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
fn a_wire_declared_dwarf_lantern_uses_the_shared_appearance_table() {
    let id = 77;
    let mut app = headless_app(snapshot(
        vec![Tile::Empty, Tile::Empty],
        vec![lantern_dwarf(id, [0, 0, 0])],
    ));
    app.update();

    let (_, light) = app
        .world_mut()
        .query::<(&WorldProjected, &PointLight)>()
        .iter(app.world())
        .find(|(projected, _)| projected.0 == id)
        .expect("a wire-declared lantern must project a point light");
    let expected = gui::appearance::light_properties(protocol::LightKind::Lantern);
    assert_eq!(
        light.color.to_srgba().to_u8_array_no_alpha(),
        expected.color.to_srgba().to_u8_array_no_alpha()
    );
    assert_eq!(light.range, expected.range);
    assert!(
        (0.95 * expected.intensity..=1.05 * expected.intensity).contains(&light.intensity),
        "the projected lantern intensity must match the appearance table it is sourced from"
    );
    // The band above is the table checked against itself, which is the right oracle for AC10
    // (the value came from the table, not a draw site) and no oracle at all for AC9 (the pool is
    // visible): zero the table row and the band becomes 0.0..=0.0, which 0.0 satisfies. Nothing
    // else in the suite reads intensity — the capture instrument only reads `range` — and
    // WARM_PIXEL_FLOOR was baselined from the torches and campfire alone, before lanterns existed.
    // This literal floor is what stands between an invisible lantern and Wolf discovering it on a
    // vehicle session.
    assert!(
        light.intensity >= LANTERN_VISIBLE_INTENSITY_FLOOR,
        "the lantern must be bright enough to read as a warm pool, not merely present"
    );
}

#[test]
fn a_dwarf_lantern_stays_on_its_blended_projection_transform() {
    let id = 78;
    let mut app = headless_app(snapshot(
        vec![Tile::Empty, Tile::Empty],
        vec![lantern_dwarf(id, [0, 0, 0])],
    ));
    app.update();

    apply_delta(&mut app, delta(vec![], vec![lantern_dwarf(id, [2, 0, 0])]));
    app.world_mut()
        .resource_mut::<gui::blend::TickClock>()
        .advance(0.01);
    app.update();

    let (translation, has_light) = app
        .world_mut()
        .query::<(&WorldProjected, &Transform, Option<&PointLight>)>()
        .iter(app.world())
        .find_map(|(projected, transform, light)| {
            (projected.0 == id).then_some((transform.translation, light.is_some()))
        })
        .expect("the dwarf remains projected");
    assert!(has_light, "the blended dwarf owns its lantern point light");
    assert_eq!(
        translation,
        projected_translation(&mut app, id),
        "the lantern and rendered dwarf share one blended transform"
    );
    assert!(
        translation.x > world_to_render([0, 0, 0]).x
            && translation.x < world_to_render([2, 0, 0]).x,
        "the lantern must follow the mid-blend dwarf position, not snap: {translation:?}"
    );
}

#[test]
fn an_unlit_dwarf_gets_no_point_light() {
    let id = 79;
    let mut app = headless_app(snapshot(
        vec![Tile::Empty, Tile::Empty],
        vec![dwarf(id, [0, 0, 0])],
    ));
    app.update();

    let has_light = app
        .world_mut()
        .query::<(&WorldProjected, Option<&PointLight>)>()
        .iter(app.world())
        .find_map(|(projected, light)| (projected.0 == id).then_some(light.is_some()))
        .expect("the dwarf remains projected");
    assert!(
        !has_light,
        "point lights must be driven by wire light, not dwarf kind"
    );

    // The spawn frame is not enough. Reconciliation has a SECOND light-insertion arm for entities
    // that already exist, and sabotaging only that arm left the whole suite green: the same
    // dwarf-kind special-casing on a later frame was invisible. This is 6.1's defect class --
    // reconcile doing something wrong on a frame the spawn-frame test never reaches.
    app.update();
    app.world_mut()
        .run_system_once(reconcile_projection)
        .expect("production reconciliation must run");
    let has_light_later = app
        .world_mut()
        .query::<(&WorldProjected, Option<&PointLight>)>()
        .iter(app.world())
        .find_map(|(projected, light)| (projected.0 == id).then_some(light.is_some()))
        .expect("the dwarf remains projected");
    assert!(
        !has_light_later,
        "a later reconciliation pass must not light an unlit dwarf either"
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

/// The four seam tests above hand-drive `TickClock::advance`, so they pass whether or not the
/// production system ever moves the clock. Replacing `time.delta_secs()` with `0.0` in
/// `blend_projection` left the whole suite green; this is the test that catches it.
#[test]
fn production_drives_the_blend_clock_from_frame_time() {
    let id = 75;
    let mut app = headless_app(snapshot(
        vec![Tile::Empty, Tile::Empty],
        vec![dwarf(id, [0, 0, 0])],
    ));
    app.update();

    apply_delta(&mut app, delta(vec![], vec![dwarf(id, [2, 0, 0])]));
    let before = app.world().resource::<gui::blend::TickClock>().elapsed();
    assert_eq!(before, 0.0, "a delivered tick re-bases the clock");

    // Deliberately no hand-advanced clock: only the production blend system may move it.
    std::thread::sleep(std::time::Duration::from_millis(5));
    app.update();

    let after = app.world().resource::<gui::blend::TickClock>().elapsed();
    assert!(
        after > before,
        "the production blend system must advance the clock from frame time, was {before} then {after}"
    );
}

/// Same class one level down: `flickered_light_survives_...` only asserts the intensity differs
/// from the table value, which is true at t=0 from the per-id phase offset alone. Replacing
/// `time.elapsed_secs()` with `0.0` left the suite green — nothing asserted a light CHANGES.
#[test]
fn production_drives_the_flicker_from_elapsed_time() {
    let id = 76;
    let emitter = Entity {
        id,
        kind: EntityKind::Torch,
        pos: [0, 0, 0],
        state: JobState::Idle,
        light: Some(protocol::LightKind::Torch),
    };
    let mut app = headless_app(snapshot(vec![Tile::Empty, Tile::Empty], vec![emitter]));
    app.update();
    let first = projected_intensity(&mut app, id);

    std::thread::sleep(std::time::Duration::from_millis(20));
    app.update();
    let second = projected_intensity(&mut app, id);

    assert_ne!(
        first, second,
        "the production flicker system must animate the light from elapsed time"
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
fn a_camp_snapshot_lights_only_its_wire_declared_emitters_and_not_an_unlit_dwarf() {
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
    let mut app = headless_app(snapshot_with_dims(
        Dims { x: 2, y: 1, z: 10 },
        vec![Tile::Empty; 20],
        entities,
    ));

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

/// The test above keeps a dwarf the wire left unlit, so on its own the suite held no headless
/// picture of the camp as it NOW ships. Its name used to claim it was the recorded camp and assert
/// exactly five lights, while the live daemon emits ten — the stale-oracle shape AC7 forbids.
#[test]
fn the_camp_as_it_now_ships_lights_every_dwarf_as_well_as_every_emitter() {
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
        .chain((5..10).map(|id| lantern_dwarf(id, [60 + id as i32 - 5, 65, 9])))
        .collect();
    let mut app = headless_app(snapshot_with_dims(
        Dims { x: 2, y: 1, z: 10 },
        vec![Tile::Empty; 20],
        entities,
    ));

    app.update();

    let lantern = gui::appearance::light_properties(protocol::LightKind::Lantern);
    let lights = app
        .world_mut()
        .query::<&PointLight>()
        .iter(app.world())
        .collect::<Vec<_>>();
    assert_eq!(
        lights.len(),
        10,
        "five static emitters and five dwarf lanterns"
    );
    assert_eq!(
        lights
            .iter()
            .filter(|light| light.range == lantern.range)
            .count(),
        5,
        "every dwarf in the shipped camp carries a lantern"
    );
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

/// The live half of the same invariant, through the real wiring: reconcile spawns the item and
/// `blend_entities` then writes its translation every frame. Both must place it as rubble resting
/// on the tile floor. A bare `world_to_render` in either one lifts it back to the tile centre, and
/// a missing scale restores the terrain-sized block that made a dug tile read as untouched rock.
///
/// Literals are hand-written rather than read from the appearance table, so this cannot pass by
/// agreeing with whatever the table happens to say.
#[test]
fn a_projected_item_is_rubble_resting_on_the_tile_floor() {
    let mut snapshot = snapshot(vec![Tile::Empty, Tile::Empty], Vec::new());
    snapshot.items = vec![Item {
        id: 42,
        pos: [1, 0, 0],
    }];
    let mut app = headless_app(snapshot);

    // Twice: the first update spawns, the second lets the blend write over what the spawn set.
    app.update();
    app.update();

    let transform = *app
        .world_mut()
        .query::<(&ProjectedItem, &Transform)>()
        .iter(app.world())
        .find(|(item, _)| item.0 == 42)
        .expect("the snapshot item must be projected")
        .1;
    assert!(
        (transform.scale.x - 0.4).abs() < 1e-6,
        "a stone item must be rubble, not a terrain-sized block: {}",
        transform.scale.x
    );
    assert!(
        (transform.translation.y - (world_to_render([1, 0, 0]).y - 0.3)).abs() < 1e-6,
        "a stone item must rest on the tile floor, not float at its centre: {}",
        transform.translation.y
    );
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

/// Wolf, at the second live viewing: "digging .. I don't think anything changed". The dig ran
/// correctly; the presentation erased it. Every tile of the named site exposes soil, and the cap
/// rule drew fresh snow on it BRIGHTER than the snow that was dug away, so a finished excavation
/// read as untouched ground one voxel lower.
#[test]
fn a_dug_tile_leaves_bare_ground_not_fresh_snow() {
    let mut app = headless_app(snapshot_with_dims(
        Dims { x: 1, y: 1, z: 3 },
        vec![
            Tile::Solid(Material::Soil),
            Tile::Solid(Material::Snow),
            Tile::Empty,
        ],
        Vec::new(),
    ));
    app.update();
    let capped_before = app
        .world_mut()
        .query::<&SnowCap>()
        .iter(app.world())
        .filter(|cap| cap.0 == [0, 0, 0])
        .count();
    assert_eq!(
        capped_before, 0,
        "buried soil is not exposed and must never be capped"
    );

    // Dig the snow at z=1, exposing the soil beneath it to the sky.
    apply_delta(
        &mut app,
        delta(
            vec![TileChange {
                pos: [0, 0, 1],
                tile: Tile::Empty,
            }],
            vec![],
        ),
    );
    app.update();

    assert_eq!(
        app.world_mut()
            .query::<&SnowCap>()
            .iter(app.world())
            .filter(|cap| cap.0 == [0, 0, 0])
            .count(),
        0,
        "a dug tile must leave bare ground: capping the floor with snow brighter than the tile \
         that was removed makes the excavation invisible"
    );
}

#[test]
fn snow_still_caps_the_natural_surface() {
    let mut app = headless_app(snapshot_with_dims(
        Dims { x: 1, y: 1, z: 2 },
        vec![Tile::Solid(Material::Snow), Tile::Empty],
        Vec::new(),
    ));
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&SnowCap>()
            .iter(app.world())
            .count(),
        1,
        "excluding soil must not stop snow terrain being capped"
    );
}

#[test]
fn named_dig_site_stays_inside_the_boot_camera_frame() {
    // Re-picked at the live viewing (Wolf, 2026-08-18). The original [58,68,9]-[64,69,9]
    // straddled a slope, and slope tiles are `Tile::Ramp`, which is not diggable -- four of them
    // stood as a contiguous wall through the middle of the excavation. This site is the ONLY
    // 2x4 rect near the camp that is all-solid, sky-exposed, unoccluded from the boot camera and
    // in frame; 19 tiles in the whole neighbourhood meet all four constraints.
    let rig = gui::camera::CameraRig::new([64, 64, 9]);
    let mut projected = Vec::new();
    for x in 55..=56 {
        for y in 62..=65 {
            let point = rig
                .project_world_point([x, y, 9])
                .expect("dig site must be in front of the camera");
            assert!(
                (0.0..=1.0).contains(&point.x) && (0.0..=1.0).contains(&point.y),
                "[{x},{y},9] projects outside the boot frame at {point:?}"
            );
            projected.push(point);
        }
    }
    let min_x = projected.iter().map(|p| p.x).fold(f32::MAX, f32::min);
    let max_x = projected.iter().map(|p| p.x).fold(f32::MIN, f32::max);
    let min_y = projected.iter().map(|p| p.y).fold(f32::MAX, f32::min);
    let max_y = projected.iter().map(|p| p.y).fold(f32::MIN, f32::max);
    println!("dig site projects to u {min_x:.3}-{max_x:.3} v {min_y:.3}-{max_y:.3}");
}

/// AC16 asks for the instrument to be driven by a hand-built sequence of MIRROR STATES. Driving
/// `LanternStats` with region literals leaves the production extraction — the dwarf/lantern filter,
/// the `light.range` read and the terrain sweep that turns a mirror plus transforms into a lit
/// region — with no caller outside a run that needs a window, so deleting that whole block left the
/// suite green. These two tests drive `accumulate_motion` itself.
fn lantern_capture_app(dwarf_at: [i32; 3]) -> App {
    // The world must be WIDER than twice the lantern's 16-unit range (`world_to_render` is 1:1),
    // or every terrain tile sits inside the pool from every dwarf position and the lit region
    // cannot change no matter how far the dwarf walks. The first draft of this test used a 3x3x3
    // world and failed for exactly that reason — the instrument was right and the test was wrong.
    let mut app = headless_app(snapshot_with_dims(
        Dims { x: 40, y: 1, z: 1 },
        vec![Tile::Solid(Material::Stone); 40],
        vec![lantern_dwarf(0, dwarf_at)],
    ));
    app.insert_resource(CaptureState::new(
        PathBuf::from("/dev/null"),
        u32::MAX,
        false,
    ));
    app.add_systems(bevy::app::Update, accumulate_motion.after(ProjectionSet));
    app
}

#[test]
fn accumulate_motion_derives_a_moving_lit_region_from_mirror_states() {
    let mut app = lantern_capture_app([0, 0, 0]);
    app.update();
    apply_delta(
        &mut app,
        delta(Vec::new(), vec![lantern_dwarf(0, [17, 0, 0])]),
    );
    // The blend must be driven before the observation. `accumulate_motion` samples on a delivered
    // POSITION change, which lands at factor ~0 with the light still rendered at the old tile, so
    // without this the pool has not moved yet when the instrument looks at it.
    app.world_mut()
        .resource_mut::<gui::blend::TickClock>()
        .advance(1.0);
    app.update();

    let capture = app.world().resource::<CaptureState>();
    assert!(
        capture.lantern_lit_tiles() > 0,
        "the production sweep must light terrain inside the lantern's range"
    );
    assert!(
        capture.lantern_moved(),
        "a dwarf that changed tiles must move its own lit region"
    );
    assert_eq!(
        capture.lantern_positions(),
        &BTreeSet::from([[0, 0, 0], [17, 0, 0]]),
        "both delivered dwarf positions must be observed"
    );
}

#[test]
fn accumulate_motion_reports_no_movement_for_a_dwarf_that_never_moves() {
    let mut app = lantern_capture_app([0, 0, 0]);
    app.update();
    apply_delta(
        &mut app,
        delta(Vec::new(), vec![lantern_dwarf(0, [0, 0, 0])]),
    );
    app.update();

    let capture = app.world().resource::<CaptureState>();
    assert!(
        capture.lantern_lit_tiles() > 0,
        "a still dwarf still lights the terrain it stands on"
    );
    assert!(
        !capture.lantern_moved(),
        "a world whose dwarf never moves must not report a moved lit region"
    );
}
