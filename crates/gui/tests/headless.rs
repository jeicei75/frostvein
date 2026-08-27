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
    camera::{CameraProjection, RenderTargetInfo},
    dev_tools::fps_overlay::FpsOverlayConfig,
    ecs::system::RunSystemOnce,
    input::{ButtonInput, mouse::MouseButton},
    pbr::{DistanceFog, FogFalloff},
    prelude::{
        Assets, Camera, DirectionalLight, Entity as BevyEntity, GlobalTransform, KeyCode, Mesh,
        Mesh3d, MeshMaterial3d, Or, PointLight, Resource, StandardMaterial, Text, Transform, UVec2,
        Vec2, Window, With, Without,
    },
    window::{PrimaryWindow, WindowResolution},
};

#[derive(Resource)]
struct TestWireSender(std::sync::mpsc::SyncSender<anyhow::Result<WireMessage>>);
use client_core::Mirror;
use gui::{
    atmosphere::{Atmosphere, SNOWFLAKE_COUNT, STAR_COUNT, Snowflake, setup_atmosphere},
    camera::CameraRig,
    capture::{CaptureState, accumulate_motion},
    command::PendingCommands,
    designate::{DesignateHint, DesignateMode, DragAnchor, DragMode, designation_hint},
    ingest::{
        CaptureDistance, IngestReceiver, MirrorResource, ProjectionSet, ProjectionWork,
        ScriptedCursor, SliceReadout, WireMessage, client_systems, fog_falloff, projection_systems,
        reconcile_projection,
    },
    pick::{Face, PickedCell, PickedTile},
    project::{
        ClientLocal, DragPreview, HoverHighlight, ProjectedDesignation, ProjectedItem,
        ProjectedZone, SnowCap, TerrainTile, WorldProjected, setup_projection_assets,
        sync_hover_highlight,
    },
    slice::SliceLevel,
    transform::world_to_render,
};
use protocol::{
    Delta, Designation, DesignationKind, Dims, Entity, EntityKind, Item, JobState, Material,
    MessageType, Snapshot, Speed, Tile, TileChange, Zone,
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

/// AD-14's rebuild invariant applied to MARKS. It needs its own oracle because marks deliberately
/// do not carry `WorldProjected` (Decision D2 — that id space already mixes sim ids with synthetic
/// terrain ids), so `projected_scene` above is structurally blind to them: reviewed 2026-08-21,
/// AC11's test despawned `&WorldProjected` only and compared through an oracle keyed the same way,
/// leaving the newest projected entity types unguarded by the very assertion meant to cover them.
fn projected_marks(app: &mut App) -> Vec<([i32; 3], Option<&'static str>, [i32; 3])> {
    let mut marks = app
        .world_mut()
        .query::<(
            &ProjectedDesignation,
            &gui::project::ProjectedDesignationKind,
            &Transform,
        )>()
        .iter(app.world())
        .map(|(mark, kind, transform)| {
            (
                mark.0,
                Some(match kind.0 {
                    DesignationKind::Dig => "dig",
                    DesignationKind::Channel => "channel",
                }),
                gui::transform::render_to_world(transform.translation),
            )
        })
        .collect::<Vec<_>>();
    marks.extend(
        app.world_mut()
            .query::<(&ProjectedZone, &Transform)>()
            .iter(app.world())
            .map(|(zone, transform)| {
                (
                    zone.0,
                    None,
                    gui::transform::render_to_world(transform.translation),
                )
            }),
    );
    marks.sort_unstable();
    marks
}

#[test]
fn snapshot_marks_project_through_the_live_ingest_schedule() {
    let dims = Dims { x: 2, y: 2, z: 3 };
    let initial = snapshot_with_dims(dims, vec![Tile::Empty; 12], Vec::new());
    let mut app = headless_app(initial.clone());
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    app.insert_resource(IngestReceiver::new(receiver));
    let mut marked = initial;
    marked.designations = vec![Designation {
        pos: [0, 0, 1],
        kind: DesignationKind::Dig,
    }];
    marked.zones = vec![Zone { pos: [1, 0, 2] }];
    sender
        .send(Ok(WireMessage::Snapshot(Box::new(marked))))
        .unwrap();

    app.update();

    let designations = app
        .world_mut()
        .query::<&ProjectedDesignation>()
        .iter(app.world())
        .map(|mark| mark.0)
        .collect::<BTreeSet<_>>();
    let zones = app
        .world_mut()
        .query::<&ProjectedZone>()
        .iter(app.world())
        .map(|mark| mark.0)
        .collect::<BTreeSet<_>>();
    assert_eq!(designations, BTreeSet::from([[0, 0, 1]]));
    assert_eq!(zones, BTreeSet::from([[1, 0, 2]]));
}

/// AC12: a delivered designation must be visible at the end of the FIRST frame that ingests it.
/// A second update here would hide an extra-frame projection latency behind reconciliation.
#[test]
fn designation_delta_projects_in_the_same_update_that_ingests_it() {
    let dims = Dims { x: 2, y: 2, z: 2 };
    let initial = snapshot_with_dims(dims, vec![Tile::Solid(Material::Stone); 8], Vec::new());
    let mut app = headless_app(initial);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    app.insert_resource(IngestReceiver::new(receiver));
    sender
        .send(Ok(WireMessage::Delta(Box::new(Delta {
            msg_type: MessageType::Delta,
            tick: 1,
            tiles: Vec::new(),
            entities: Vec::new(),
            designations: vec![Designation {
                pos: [1, 0, 1],
                kind: DesignationKind::Dig,
            }],
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
        }))))
        .unwrap();

    app.update();

    assert_eq!(
        app.world_mut()
            .query::<&ProjectedDesignation>()
            .iter(app.world())
            .map(|mark| mark.0)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([[1, 0, 1]]),
        "the designation delta must project in its first ingested frame"
    );
}

#[test]
fn mark_slabs_rest_on_their_ordered_surfaces_and_layer_a_channel_zone_overlap() {
    let dims = Dims { x: 2, y: 1, z: 2 };
    let initial = snapshot_with_dims(
        dims,
        vec![
            Tile::Solid(Material::Stone),
            Tile::Solid(Material::Stone),
            Tile::Empty,
            Tile::Empty,
        ],
        Vec::new(),
    );
    let mut app = headless_app(initial.clone());
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    app.insert_resource(IngestReceiver::new(receiver));
    let mut marked = initial;
    marked.designations = vec![
        Designation {
            pos: [0, 0, 0],
            kind: DesignationKind::Dig,
        },
        Designation {
            pos: [0, 0, 1],
            kind: DesignationKind::Channel,
        },
    ];
    marked.zones = vec![Zone { pos: [0, 0, 1] }];
    sender
        .send(Ok(WireMessage::Snapshot(Box::new(marked))))
        .unwrap();
    app.update();

    let mut designations = app
        .world_mut()
        .query::<(&ProjectedDesignation, &Transform)>();
    let dig = designations
        .iter(app.world())
        .find_map(|(mark, transform)| (mark.0 == [0, 0, 0]).then_some(*transform))
        .expect("the dig mark must project");
    let channel = designations
        .iter(app.world())
        .find_map(|(mark, transform)| (mark.0 == [0, 0, 1]).then_some(*transform))
        .expect("the channel mark must project");
    let mut zones = app.world_mut().query::<(&ProjectedZone, &Transform)>();
    let zone = zones
        .iter(app.world())
        .find_map(|(mark, transform)| (mark.0 == [0, 0, 1]).then_some(*transform))
        .expect("the zone mark must project");

    assert!(
        (dig.translation.y - 0.54).abs() < 1e-6,
        "a dig slab must sit on top of its solid rock; got {}",
        dig.translation.y
    );
    assert!(
        (channel.translation.y - 0.54).abs() < 1e-6,
        "a channel slab rests on its empty tile floor; got {}",
        channel.translation.y
    );
    assert!(
        (zone.translation.y - 0.64).abs() < 1e-6,
        "an overlapping zone layers above its channel; got {}",
        zone.translation.y
    );
    assert!(
        (dig.scale.x - 0.94).abs() < 1e-6 && (channel.scale.x - 0.94).abs() < 1e-6,
        "ordinary mark slabs need a gutter between adjacent tiles; got dig={} channel={}",
        dig.scale.x,
        channel.scale.x
    );
    assert!(
        (zone.scale.x - 0.6768).abs() < 1e-6,
        "the raised zone leaves the channel rim readable; got {}",
        zone.scale.x
    );
    assert!(
        (zone.scale.z - 0.6768).abs() < 1e-6,
        "the raised zone stays square; got {}",
        zone.scale.z
    );
}

#[test]
fn a_designation_kind_change_restyles_the_existing_position_mark() {
    let dims = Dims { x: 2, y: 2, z: 2 };
    let initial = snapshot_with_dims(dims, vec![Tile::Empty; 8], Vec::new());
    let mut app = headless_app(initial.clone());
    let (sender, receiver) = std::sync::mpsc::sync_channel(2);
    app.insert_resource(IngestReceiver::new(receiver));
    let mut marked = initial;
    marked.designations = vec![Designation {
        pos: [1, 1, 1],
        kind: DesignationKind::Dig,
    }];
    sender
        .send(Ok(WireMessage::Snapshot(Box::new(marked))))
        .unwrap();
    app.update();
    sender
        .send(Ok(WireMessage::Delta(Box::new(Delta {
            msg_type: MessageType::Delta,
            tick: 1,
            tiles: Vec::new(),
            entities: Vec::new(),
            designations: vec![Designation {
                pos: [1, 1, 1],
                kind: DesignationKind::Channel,
            }],
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
        }))))
        .unwrap();
    app.update();

    let marks = app
        .world_mut()
        .query::<(
            &ProjectedDesignation,
            &gui::project::ProjectedDesignationKind,
        )>()
        .iter(app.world())
        .map(|(mark, kind)| (mark.0, kind.0))
        .collect::<Vec<_>>();
    assert_eq!(marks, vec![([1, 1, 1], DesignationKind::Channel)]);

    // AC10 asks that a kind change RESTYLES the mark, and the bookkeeping component above is not
    // the style. Reviewed 2026-08-21: deleting the update path's `MeshMaterial3d` insert left the
    // whole suite green, this test included, because nothing looked at the material — a dig
    // retuned to a channel would have kept dig blue forever with the sabotage table green too.
    // The handle is compared against the assets table's own channel handle, so this cannot be
    // satisfied by any material at all, only by the right one.
    let handle = app
        .world_mut()
        .query::<(&ProjectedDesignation, &MeshMaterial3d<StandardMaterial>)>()
        .iter(app.world())
        .map(|(_, material)| material.0.clone())
        .next()
        .expect("the restyled mark must still carry a material");
    let drawn = app
        .world()
        .resource::<Assets<StandardMaterial>>()
        .get(&handle)
        .expect("the mark's material handle must resolve")
        .base_color;
    assert_eq!(
        drawn.to_srgba().to_u8_array_no_alpha(),
        gui::appearance::designation_color(DesignationKind::Channel)
            .to_srgba()
            .to_u8_array_no_alpha(),
        "the mark must be WEARING the channel colour after the kind change"
    );
    assert_ne!(
        drawn.to_srgba().to_u8_array_no_alpha(),
        gui::appearance::designation_color(DesignationKind::Dig)
            .to_srgba()
            .to_u8_array_no_alpha(),
        "a restyled channel must not still be wearing dig blue"
    );
}

/// The defect that made this story's own capture recipe photograph an empty site and exit 0.
/// `is_visible_at_slice` draws every solid tile AT the cut as a full cube spanning `[z-0.5, z+0.5]`
/// regardless of exposure, so a dig slab resting at `z+0.54` on the tile BELOW that cube was
/// sealed inside opaque rock. Measured live at the 2026-08-21 review on the story's own recipe:
/// 0 of 50 surviving marks visible from t+120 onward, while the instrument correctly printed
/// `designations=50` — every one of them projected, none of them seeable.
///
/// RULED 2026-08-21 (Wolf): promote a buried dig to the top face of the rock covering it. The
/// oracle here is hand-written geometry, not a re-read of `dig_mark_level`.
#[test]
fn a_dig_buried_under_the_cut_is_drawn_on_the_rock_that_covers_it() {
    let dims = Dims { x: 2, y: 2, z: 3 };
    let index = |x: usize, y: usize, z: usize| x + y * 2 + z * 4;
    let mut tiles = vec![Tile::Empty; 12];
    // Column [0,0]: rock at the cut covers the dig one level below it.
    tiles[index(0, 0, 1)] = Tile::Solid(Material::Ice);
    tiles[index(0, 0, 2)] = Tile::Solid(Material::Ice);
    // Column [1,0]: open sky above the dig, so nothing hides it and it must NOT be moved.
    tiles[index(1, 0, 1)] = Tile::Solid(Material::Ice);

    let mut world = snapshot_with_dims(dims, tiles, Vec::new());
    world.designations = vec![
        Designation {
            pos: [0, 0, 1],
            kind: DesignationKind::Dig,
        },
        Designation {
            pos: [1, 0, 1],
            kind: DesignationKind::Dig,
        },
    ];
    let mut app = headless_app(world);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    app.insert_resource(IngestReceiver::new(receiver));
    drop(sender);
    app.update();

    let heights = app
        .world_mut()
        .query::<(&ProjectedDesignation, &Transform)>()
        .iter(app.world())
        .map(|(mark, transform)| (mark.0, transform.translation.y))
        .collect::<std::collections::BTreeMap<_, _>>();

    // The cut cube in column [0,0] spans [1.5, 2.5]; a slab at 1.54 is inside it, one at 2.54 is
    // on top of it. Both marks keep their own identity — only where they are DRAWN changes.
    let buried = heights[&[0, 0, 1]];
    assert!(
        buried > 2.5,
        "a dig under the cut-face cube must be drawn above that cube's top face, not at \
         {buried} where the rock encloses it"
    );
    assert!((buried - 2.54).abs() < 1e-5, "buried dig sat at {buried}");

    let open = heights[&[1, 0, 1]];
    assert!(
        (open - 1.54).abs() < 1e-5,
        "a dig with open sky above it is already visible and must stay on its own tile, not be \
         hoisted onto rock it does not mark; sat at {open}"
    );
}

/// A stockpile sits on standable ground: solid at `z-1`, air at `z`. So a stockpile at z and a
/// dig on the rock beneath it at `z-1` are marks on the SAME surface, and before this fix they
/// projected to byte-identical translations and scales with the same opaque mesh — measured at the
/// 2026-08-21 review as designation `[9,9,9]` and zone `[9,9,10]` both at `(9.000, 9.540, -9.000)`.
/// The story's own recipe hits it: the stockpile columns sit inside the dig rect, so the tiles
/// would z-fight exactly while AC5 ("is a stockpile tellable from a dig") is being judged.
#[test]
fn a_stockpile_over_a_dig_does_not_share_the_digs_surface() {
    let dims = Dims { x: 2, y: 2, z: 3 };
    let index = |x: usize, y: usize, z: usize| x + y * 2 + z * 4;
    let mut tiles = vec![Tile::Empty; 12];
    tiles[index(0, 0, 1)] = Tile::Solid(Material::Ice);

    let mut world = snapshot_with_dims(dims, tiles, Vec::new());
    world.designations = vec![Designation {
        pos: [0, 0, 1],
        kind: DesignationKind::Dig,
    }];
    world.zones = vec![Zone { pos: [0, 0, 2] }];
    let mut app = headless_app(world);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    app.insert_resource(IngestReceiver::new(receiver));
    drop(sender);
    app.update();

    let dig = app
        .world_mut()
        .query::<(&ProjectedDesignation, &Transform)>()
        .iter(app.world())
        .map(|(_, transform)| *transform)
        .next()
        .expect("the dig must project");
    let zone = app
        .world_mut()
        .query::<(&ProjectedZone, &Transform)>()
        .iter(app.world())
        .map(|(_, transform)| *transform)
        .next()
        .expect("the zone must project");

    assert_ne!(
        dig.translation, zone.translation,
        "a stockpile and the dig beneath it must not occupy one surface"
    );
    assert_ne!(
        dig.scale, zone.scale,
        "the zone over a dig must be inset so both marks stay readable"
    );
    assert!(
        zone.translation.y > dig.translation.y,
        "the stockpile is the upper tile, so its slab must sit above the dig's"
    );
}

#[test]
fn marks_follow_the_slice_control_at_and_below_the_cut() {
    let dims = Dims { x: 2, y: 2, z: 3 };
    let initial = snapshot_with_dims(dims, vec![Tile::Empty; 12], Vec::new());
    let mut app = headless_app(initial.clone());
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    app.insert_resource(IngestReceiver::new(receiver));
    let mut marked = initial;
    marked.designations = vec![
        Designation {
            pos: [0, 0, 1],
            kind: DesignationKind::Dig,
        },
        Designation {
            pos: [1, 0, 2],
            kind: DesignationKind::Channel,
        },
    ];
    sender
        .send(Ok(WireMessage::Snapshot(Box::new(marked))))
        .unwrap();
    app.update();
    press_once(&mut app, KeyCode::Comma);

    let marks = app
        .world_mut()
        .query::<&ProjectedDesignation>()
        .iter(app.world())
        .map(|mark| mark.0)
        .collect::<BTreeSet<_>>();
    assert_eq!(marks, BTreeSet::from([[0, 0, 1]]));
}

#[test]
fn marks_are_world_projected_never_client_local() {
    let dims = Dims { x: 2, y: 2, z: 2 };
    let initial = snapshot_with_dims(dims, vec![Tile::Empty; 8], Vec::new());
    let mut app = headless_app(initial.clone());
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    app.insert_resource(IngestReceiver::new(receiver));
    let mut marked = initial;
    marked.designations = vec![Designation {
        pos: [0, 0, 1],
        kind: DesignationKind::Dig,
    }];
    marked.zones = vec![Zone { pos: [1, 0, 1] }];
    sender
        .send(Ok(WireMessage::Snapshot(Box::new(marked))))
        .unwrap();
    app.update();

    let mut query = app.world_mut().query::<(
        BevyEntity,
        Option<&WorldProjected>,
        Option<&ProjectedDesignation>,
        Option<&ProjectedZone>,
        Option<&ClientLocal>,
    )>();
    for (entity, world, designation, zone, local) in query.iter(app.world()) {
        let is_world = world.is_some() || designation.is_some() || zone.is_some();
        if is_world || local.is_some() {
            assert_ne!(
                is_world,
                local.is_some(),
                "{entity:?} crossed the projection partition"
            );
        }
    }
}

#[test]
fn cancellation_delta_despawns_a_missing_designation() {
    let mut app = app_with_one_designation();
    send_empty_designation_delta(&mut app, Vec::new());
    assert_no_designations(&mut app);
}

#[test]
fn consumption_delta_despawns_a_missing_designation_by_the_same_path() {
    let mut app = app_with_one_designation();
    send_empty_designation_delta(
        &mut app,
        vec![TileChange {
            pos: [0, 0, 1],
            tile: Tile::Empty,
        }],
    );
    assert_no_designations(&mut app);
}

#[test]
fn snapshot_then_delta_in_one_frame_leaves_no_stale_marks() {
    let dims = Dims { x: 2, y: 2, z: 2 };
    let initial = snapshot_with_dims(dims, vec![Tile::Empty; 8], Vec::new());
    let mut app = headless_app(initial.clone());
    let (sender, receiver) = std::sync::mpsc::sync_channel(2);
    app.insert_resource(IngestReceiver::new(receiver));
    let mut marked = initial;
    marked.designations = vec![Designation {
        pos: [0, 0, 1],
        kind: DesignationKind::Dig,
    }];
    sender
        .send(Ok(WireMessage::Snapshot(Box::new(marked))))
        .unwrap();
    sender
        .send(Ok(WireMessage::Delta(Box::new(Delta {
            msg_type: MessageType::Delta,
            tick: 1,
            tiles: Vec::new(),
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
        }))))
        .unwrap();
    app.update();
    assert_no_designations(&mut app);
}

fn app_with_one_designation() -> App {
    let dims = Dims { x: 2, y: 2, z: 2 };
    let initial = snapshot_with_dims(dims, vec![Tile::Solid(Material::Stone); 8], Vec::new());
    let mut app = headless_app(initial.clone());
    let (sender, receiver) = std::sync::mpsc::sync_channel(2);
    app.insert_resource(IngestReceiver::new(receiver));
    let mut marked = initial;
    marked.designations = vec![Designation {
        pos: [0, 0, 1],
        kind: DesignationKind::Dig,
    }];
    sender
        .send(Ok(WireMessage::Snapshot(Box::new(marked))))
        .unwrap();
    app.world_mut().insert_resource(TestWireSender(sender));
    app.update();
    app
}

fn send_empty_designation_delta(app: &mut App, tiles: Vec<TileChange>) {
    app.world_mut()
        .resource::<TestWireSender>()
        .0
        .send(Ok(WireMessage::Delta(Box::new(Delta {
            msg_type: MessageType::Delta,
            tick: 1,
            tiles,
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
        }))))
        .unwrap();
    app.update();
}

fn assert_no_designations(app: &mut App) {
    assert_eq!(
        app.world_mut()
            .query::<&ProjectedDesignation>()
            .iter(app.world())
            .count(),
        0,
        "the replaced mirror list must remove its stale projection"
    );
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
        vec!["Slice: z 2/2 - surface".to_string()],
        "the readout must exist at boot and name the level"
    );

    press_once(&mut app, KeyCode::Comma);
    assert_eq!(
        readout(&mut app),
        vec!["Slice: z 1/2 - surface".to_string()],
        "z 1 has only empty sky above it, so it is not underground"
    );

    press_once(&mut app, KeyCode::Comma);
    assert_eq!(
        readout(&mut app),
        vec!["Slice: z 0/2 - underground".to_string()],
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
    // AC11 says "marks included", so the snapshot must actually carry marks — reviewed
    // 2026-08-21, this test used an empty designation and zone list, so every assertion in it was
    // true of a scene with no marks in it at all.
    let mut world = snapshot(vec![Tile::Solid(Material::Ice), Tile::Empty], vec![dwarf]);
    world.designations = vec![Designation {
        pos: [0, 0, 0],
        kind: DesignationKind::Dig,
    }];
    world.zones = vec![Zone { pos: [1, 0, 0] }];
    let mut app = headless_app(world);
    app.update();
    let expected = projected_scene(&mut app);
    let expected_marks = projected_marks(&mut app);
    assert!(
        !expected_marks.is_empty(),
        "the rebuild invariant is only tested if there are marks to rebuild"
    );
    let projected = app
        .world_mut()
        .query::<(BevyEntity, &WorldProjected)>()
        .iter(app.world())
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let marks = app
        .world_mut()
        .query_filtered::<BevyEntity, Or<(With<ProjectedDesignation>, With<ProjectedZone>)>>()
        .iter(app.world())
        .collect::<Vec<_>>();
    for entity in projected.into_iter().chain(marks) {
        app.world_mut().entity_mut(entity).despawn();
    }
    assert!(projected_scene(&mut app).is_empty());
    assert!(projected_marks(&mut app).is_empty());
    app.world_mut().resource_mut::<ProjectionWork>().snapshot = true;
    app.update();

    assert_eq!(projected_scene(&mut app), expected);
    assert_eq!(projected_marks(&mut app), expected_marks);
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

    // The COVERAGE half of the partition, which is the half that catches a NEW projected entity
    // type carrying neither marker. Reviewed 2026-08-21: this test still knew only
    // `WorldProjected`, and the disjointness test added alongside the marks skips any entity that
    // carries neither — so between them, an unmarked projected entity passed both. Marks are the
    // worked example: they are world-projected (AD-14, NFR5) and deliberately do NOT carry
    // `WorldProjected` (Decision D2), so they must be named here explicitly.
    let unclassified = app
        .world_mut()
        .query_filtered::<BevyEntity, (
            With<Transform>,
            Without<WorldProjected>,
            Without<ClientLocal>,
            Without<ProjectedDesignation>,
            Without<ProjectedZone>,
        )>()
        .iter(app.world())
        .count();
    assert_eq!(
        unclassified, 0,
        "every transformed entity must be world-projected or client-local; an entity carrying \
         neither marker is invisible to both halves of this partition"
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

// ===========================================================================================
// M2-1 — the live `App` that `run()` builds, made reachable from the suite.
//
// THE DEFECT THIS CLOSES, in one line: while `run()`'s registration tuples lived inline, no test
// could reach them, so deleting a system left the whole suite green. It was filed `[feature/MED]`
// and DEFERRED at story 5.4's review — and then produced the TOP-SEVERITY finding in the next
// four consecutive stories. 6.1 lost both projection systems with 54/54 green; 6.1's review found
// three production drive lines each of which killed wow beat 2 with 57/57 green; 6.2's
// `accumulate_motion` had zero test callers and had never executed anywhere; 7.1's whole on-screen
// readout and its `--z` pin were each deletable; 7.2's `--distance` parsed, validated and never
// reached the camera rig, with its only test NAMED for reaching the camera setup. Five of eight
// stories. The Milestone 2 retrospective ruled it closed at the root (Wolf, 2026-08-23).
//
// These tests drive `client_systems` — the SAME function `run()` calls — so a system dropped from
// either tuple fails here rather than shipping. Every assertion is an OBSERVABLE EFFECT, never
// "is it registered": a registration assertion would be the vacuity this project keeps re-finding.
// ===========================================================================================

/// Mirrors `run()`'s app as closely as `MinimalPlugins` allows. The resources supplied by hand are
/// exactly the ones `DefaultPlugins` and `FpsOverlayPlugin` provide in production; nothing else is
/// added, so the systems under test see the world they really run in.
fn live_app(
    snapshot: Snapshot,
) -> (
    App,
    std::sync::mpsc::SyncSender<anyhow::Result<WireMessage>>,
) {
    let (sender, receiver) = std::sync::mpsc::sync_channel(8);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        // The aurora curtain samples a procedurally built gradient image.
        .init_resource::<Assets<bevy::image::Image>>()
        // `FpsOverlayPlugin` inserts this in production; `toggle_overlay` needs it to exist.
        .init_resource::<FpsOverlayConfig>()
        .insert_resource(MirrorResource(Mirror::from_snapshot(snapshot).unwrap()))
        .insert_resource(ProjectionWork {
            snapshot: true,
            dirty_tiles: Default::default(),
        })
        .insert_resource(IngestReceiver::new(receiver));
    client_systems(&mut app);
    projection_systems(&mut app);
    (app, sender)
}

fn one_tile_snapshot() -> Snapshot {
    snapshot(vec![Tile::Solid(Material::Stone), Tile::Empty], vec![])
}

const PICK_VIEWPORT: UVec2 = UVec2::new(1920, 1080);

fn install_pick_camera(app: &mut App, rig: CameraRig, cursor: Vec2) {
    // Run Startup first: `live_app` deliberately drives the same registration point as `run()`.
    app.update();

    let camera_entity = app
        .world_mut()
        .query_filtered::<BevyEntity, With<CameraRig>>()
        .single(app.world())
        .expect("live startup must spawn exactly one camera rig");
    let mut camera = Camera::default();
    camera.computed.target_info = Some(RenderTargetInfo {
        physical_size: PICK_VIEWPORT,
        scale_factor: 1.0,
    });
    let mut projection = bevy::prelude::PerspectiveProjection {
        fov: gui::camera::BOOT_VERTICAL_FOV,
        ..Default::default()
    };
    projection.update(PICK_VIEWPORT.x as f32, PICK_VIEWPORT.y as f32);
    camera.computed.clip_from_view = projection.get_clip_from_view();
    let transform = rig.transform();
    app.world_mut().entity_mut(camera_entity).insert((
        camera,
        transform,
        GlobalTransform::from(transform),
        rig,
    ));

    let mut window = Window {
        resolution: WindowResolution::new(PICK_VIEWPORT.x, PICK_VIEWPORT.y),
        ..Default::default()
    };
    window.set_cursor_position(Some(cursor));
    app.world_mut().spawn((window, PrimaryWindow));
}

#[test]
fn a_cursor_at_a_visible_tiles_independent_projection_picks_that_tile() {
    let target = [1, 1, 0];
    let rig = CameraRig::new(target);
    let normalized = rig
        .project_world_point(target)
        .expect("the tile under the camera focus must project into the viewport");
    let mut app = live_app(snapshot_with_dims(
        Dims { x: 3, y: 3, z: 1 },
        vec![
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
            Tile::Solid(Material::Stone),
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
        ],
        vec![],
    ))
    .0;
    install_pick_camera(&mut app, rig, normalized * PICK_VIEWPORT.as_vec2());

    app.update();

    assert_eq!(
        app.world().resource::<PickedTile>().tile(),
        Some([1, 1, 0]),
        "the live client schedule must pick the visible tile under its projected cursor"
    );
}

#[test]
fn the_live_pick_spawns_a_client_local_highlight_and_despawns_it_without_a_pick() {
    let target = [1, 1, 0];
    let rig = CameraRig::new(target);
    let cursor = rig
        .project_world_point(target)
        .expect("the visible target must have a forward projection")
        * PICK_VIEWPORT.as_vec2();
    let mut app = live_app(snapshot_with_dims(
        Dims { x: 3, y: 3, z: 1 },
        vec![
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
            Tile::Solid(Material::Stone),
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
        ],
        vec![],
    ))
    .0;
    install_pick_camera(&mut app, rig, cursor);

    app.update();

    let highlights = app
        .world_mut()
        .query::<(&HoverHighlight, &ClientLocal, Option<&WorldProjected>)>()
        .iter(app.world())
        .collect::<Vec<_>>();
    assert_eq!(highlights.len(), 1, "a picked tile must gain one highlight");
    assert_eq!(highlights[0].0.0, [1, 1, 0]);
    assert!(
        highlights[0].2.is_none(),
        "a hover highlight is never simulation-projected"
    );

    app.world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .single_mut(app.world_mut())
        .expect("the pick harness owns one primary window")
        .set_cursor_position(None);
    app.update();

    assert_eq!(
        app.world_mut()
            .query::<&HoverHighlight>()
            .iter(app.world())
            .count(),
        0,
        "removing the cursor must remove the stale hover highlight"
    );
}

#[test]
fn a_vertical_hit_face_places_the_hover_slab_outside_the_cell_side() {
    let mut app = headless_app(one_tile_snapshot());
    app.update();
    let tile = [3, 4, 5];
    app.insert_resource(PickedTile(Some(PickedCell {
        tile,
        face: Face::East,
    })));
    app.world_mut()
        .run_system_once(sync_hover_highlight)
        .unwrap();
    app.world_mut().flush();

    let transform = *app
        .world_mut()
        .query::<(&HoverHighlight, &Transform)>()
        .single(app.world())
        .unwrap()
        .1;
    assert_eq!(
        transform.translation,
        world_to_render(tile) + bevy::prelude::Vec3::X * 0.55,
        "a side hit must hoist the thin mesh beyond that side, not onto the old top face"
    );
    // The offset alone is not the feature. `mark_mesh` is a THIN SLAB whose flat face is normal
    // to its local Y; hoisted beside a wall without being turned, it is a horizontal wafer seen
    // edge-on, which is invisible from exactly the viewpoint AC13 is about. This assertion reads
    // the rotation, which nothing did — deleting `with_rotation` from both call sites left the
    // whole suite green.
    let turned = transform.rotation * bevy::prelude::Vec3::Y;
    let normal = Face::East.normal();
    assert!(
        turned.distance(normal) < 1e-5,
        "the slab's thin axis must be turned onto the face normal {normal:?}, but it points \
         {turned:?} — the slab is lying flat against a vertical wall"
    );
}

fn picked_at(snapshot: Snapshot, rig: CameraRig, level: i32, cursor: Vec2) -> Option<[i32; 3]> {
    let mut app = live_app(snapshot).0;
    install_pick_camera(&mut app, rig, cursor);
    app.world_mut().resource_mut::<SliceLevel>().set(level);
    app.update();
    let picked = app.world().resource::<PickedTile>().tile();
    if picked.is_none() {
        assert_eq!(
            app.world_mut()
                .query::<&HoverHighlight>()
                .iter(app.world())
                .count(),
            0,
            "a no-pick frame must not leave a stale hover highlight"
        );
    }
    picked
}

fn solid_column_snapshot() -> Snapshot {
    let dims = Dims { x: 9, y: 9, z: 4 };
    let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    for z in 0..dims.z as usize {
        tiles[4 + 4 * dims.x as usize + z * dims.x as usize * dims.y as usize] =
            Tile::Solid(Material::Stone);
    }
    snapshot_with_dims(dims, tiles, vec![])
}

#[test]
fn camera_picking_covers_orbits_zoom_limits_and_sliced_levels() {
    let yaws = [-2.1, 0.0, 1.2];
    let distances = [4.0, 30.0, 500.0];
    let levels = [0, 1, 3];
    // AC3 says "any pitch", and these are the only pitches there are: `orbit()` clamps to
    // MIN_PITCH 0.15 ..= MAX_PITCH (FRAC_PI_2 - 0.15). The matrix used to run at -0.55, which
    // puts the camera BELOW the world looking up — a pose the rig cannot hold, so the clause
    // had zero coverage inside the reachable range. Both clamp ends and the boot pitch now run.
    let pitches = [0.15, 0.45, std::f32::consts::FRAC_PI_2 - 0.15];
    for yaw in yaws {
        for pitch in pitches {
            for distance in distances {
                for level in levels {
                    let target = [4, 4, level];
                    let rig = CameraRig {
                        focus: target,
                        yaw,
                        pitch,
                        distance,
                    };
                    let cursor = rig.project_world_point(target).expect(
                        "every test target must project through the independent camera rig",
                    ) * PICK_VIEWPORT.as_vec2();
                    assert_eq!(
                        picked_at(solid_column_snapshot(), rig, level, cursor),
                        Some(target),
                        "yaw={yaw}, pitch={pitch}, distance={distance}, slice={level} must pick \
                         literal target {target:?}"
                    );
                }
            }
        }
    }
}

/// AC4's occlusion clause: two slice-visible tiles on one ray, and the NEARER must win.
///
/// Every other picking scene is one isolated tile or one solid column, where "stop at the first
/// visible hit" cannot be told apart from "return any visible hit". The control half is what
/// makes this a test of ordering rather than of reachability: with the near tile removed, the
/// same camera and the same cursor must reach the far one.
#[test]
fn the_nearer_of_two_tiles_on_one_ray_is_the_one_picked() {
    fn tower(near: bool) -> Snapshot {
        let dims = Dims { x: 9, y: 9, z: 4 };
        let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
        let index = |[x, y, z]: [i32; 3]| {
            (x as u32 + y as u32 * dims.x + z as u32 * dims.x * dims.y) as usize
        };
        tiles[index([4, 4, 0])] = Tile::Solid(Material::Stone);
        if near {
            tiles[index([4, 4, 3])] = Tile::Solid(Material::Stone);
        }
        snapshot_with_dims(dims, tiles, vec![])
    }

    // Near-vertical, so one ray passes through both tiles' cells.
    let rig = CameraRig {
        focus: [4, 4, 3],
        yaw: 0.7,
        pitch: std::f32::consts::FRAC_PI_2 - 0.15,
        distance: 30.0,
    };
    let cursor = rig
        .project_world_point([4, 4, 3])
        .expect("the upper tile must project into the viewport")
        * PICK_VIEWPORT.as_vec2();

    assert_eq!(
        picked_at(tower(true), rig, 3, cursor),
        Some([4, 4, 3]),
        "the nearer of two tiles on one ray must occlude the farther one"
    );
    assert_eq!(
        picked_at(tower(false), rig, 3, cursor),
        Some([4, 4, 0]),
        "control: the same ray must REACH the far tile, so the answer above is ordering and \
         not reachability"
    );
}

#[test]
fn picking_nothing_leaves_no_hover_for_sky_hidden_tiles_and_outside_the_window() {
    let rig = CameraRig::new([1, 1, 0]);
    let sky = picked_at(
        snapshot_with_dims(
            Dims { x: 3, y: 3, z: 1 },
            vec![
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Solid(Material::Stone),
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
            ],
            vec![],
        ),
        rig,
        0,
        Vec2::ZERO,
    );
    assert_eq!(sky, None, "the top-left sky contains no terrain tile");

    let hidden_target = [1, 1, 1];
    let hidden_rig = CameraRig::new(hidden_target);
    let hidden_cursor = hidden_rig
        .project_world_point(hidden_target)
        .expect("the hidden tile would be in the camera frustum")
        * PICK_VIEWPORT.as_vec2();
    let hidden = picked_at(
        snapshot_with_dims(
            Dims { x: 3, y: 3, z: 2 },
            vec![
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Solid(Material::Stone),
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
            ],
            vec![],
        ),
        hidden_rig,
        0,
        hidden_cursor,
    );
    assert_eq!(hidden, None, "the slice must reject a tile above its cut");

    let outside = picked_at(solid_column_snapshot(), rig, 0, Vec2::new(-1.0, -1.0));
    assert_eq!(
        outside, None,
        "a cursor outside the viewport must not pick a default tile"
    );
}

#[test]
fn the_scripted_capture_cursor_reaches_the_live_pick_system() {
    let target = [1, 1, 0];
    let rig = CameraRig::new(target);
    let cursor = rig
        .project_world_point(target)
        .expect("the target must have an independent forward projection")
        * PICK_VIEWPORT.as_vec2();
    let mut app = live_app(snapshot_with_dims(
        Dims { x: 3, y: 3, z: 1 },
        vec![
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
            Tile::Solid(Material::Stone),
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
        ],
        vec![],
    ))
    .0;
    install_pick_camera(&mut app, rig, Vec2::ZERO);
    app.world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .single_mut(app.world_mut())
        .expect("the pick harness owns one primary window")
        .set_cursor_position(None);
    app.insert_resource(ScriptedCursor(cursor));

    app.update();

    assert_eq!(
        app.world().resource::<PickedTile>().tile(),
        Some([1, 1, 0]),
        "the parsed capture cursor must be written before the shared pick system runs"
    );
}

#[test]
fn mouse_drag_uses_the_anchor_level_and_clears_its_anchor_on_release() {
    let anchor = [1, 1, 1];
    let release = [2, 1, 0];
    let rig = CameraRig::new(anchor);
    let anchor_cursor = rig
        .project_world_point(anchor)
        .expect("the literal anchor must project")
        * PICK_VIEWPORT.as_vec2();
    let release_cursor = rig
        .project_world_point(release)
        .expect("the literal release tile must project")
        * PICK_VIEWPORT.as_vec2();
    let dims = Dims { x: 3, y: 3, z: 2 };
    let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    let index =
        |[x, y, z]: [i32; 3]| (x + y * dims.x as i32 + z * dims.x as i32 * dims.y as i32) as usize;
    tiles[index(anchor)] = Tile::Solid(Material::Stone);
    tiles[index(release)] = Tile::Solid(Material::Stone);
    let mut app = live_app(snapshot_with_dims(dims, tiles, vec![])).0;
    install_pick_camera(&mut app, rig, anchor_cursor);
    app.update();

    press_once(&mut app, KeyCode::Digit1);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert_eq!(
        app.world().resource::<DragAnchor>().0.map(|cell| cell.tile),
        Some(anchor)
    );
    // MinimalPlugins does not run InputPlugin's transition clearing between frames.
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();

    app.world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .single_mut(app.world_mut())
        .unwrap()
        .set_cursor_position(Some(release_cursor));
    {
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.release(MouseButton::Left);
    }
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();

    assert_eq!(app.world().resource::<DragAnchor>().0, None);
    assert_eq!(
        app.world()
            .resource::<PendingCommands>()
            .commands()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![protocol::Command::Designate {
            kind: DesignationKind::Dig,
            rect: protocol::Rect {
                min: [1, 1, 1],
                max: [2, 1, 1]
            },
        }],
        "a cross-height drag is one inclusive rectangle on the literal anchor level"
    );
}

/// AC11: this is the complete client round trip below the world top: a real mouse drag enters
/// the shared input schedule, the daemon's designation delta comes back, and the mark projects
/// at the pinned underground level rather than on the surface.
#[test]
fn mouse_designation_on_a_sliced_underground_level_round_trips_to_a_projected_mark() {
    let anchor = [1, 1, 1];
    let rig = CameraRig::new(anchor);
    let cursor = rig
        .project_world_point(anchor)
        .expect("the underground anchor must project")
        * PICK_VIEWPORT.as_vec2();
    let dims = Dims { x: 3, y: 3, z: 3 };
    let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    let index =
        |[x, y, z]: [i32; 3]| (x + y * dims.x as i32 + z * dims.x as i32 * dims.y as i32) as usize;
    tiles[index(anchor)] = Tile::Solid(Material::Stone);
    let (mut app, sender) = live_app(snapshot_with_dims(dims, tiles, vec![]));
    install_pick_camera(&mut app, rig, cursor);
    app.world_mut().resource_mut::<SliceLevel>().set(anchor[2]);
    app.update();

    press_once(&mut app, KeyCode::Digit1);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();

    assert_eq!(
        app.world()
            .resource::<PendingCommands>()
            .commands()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![protocol::Command::Designate {
            kind: DesignationKind::Dig,
            rect: protocol::Rect {
                min: anchor,
                max: anchor,
            },
        }],
        "the underground drag must send its literal picked level, not the world top"
    );
    sender
        .send(Ok(WireMessage::Delta(Box::new(Delta {
            msg_type: MessageType::Delta,
            tick: 1,
            tiles: Vec::new(),
            entities: Vec::new(),
            designations: vec![Designation {
                pos: anchor,
                kind: DesignationKind::Dig,
            }],
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
        }))))
        .unwrap();

    app.update();

    assert_eq!(
        app.world_mut()
            .query::<&ProjectedDesignation>()
            .iter(app.world())
            .map(|mark| mark.0)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([anchor]),
        "the returned designation must project on the pinned underground slice"
    );
}

#[test]
fn the_live_startup_scene_spawns_its_camera_lighting_and_atmosphere() {
    let (mut app, _sender) = live_app(one_tile_snapshot());
    app.update();

    assert_eq!(
        app.world_mut()
            .query::<&CameraRig>()
            .iter(app.world())
            .count(),
        1,
        "setup_camera did not spawn the rig — the whole view depends on it"
    );
    assert_eq!(
        app.world_mut()
            .query::<&DirectionalLight>()
            .iter(app.world())
            .count(),
        1,
        "setup_night_lighting did not spawn the directional fill"
    );
    assert_eq!(
        app.world_mut()
            .query::<&Snowflake>()
            .iter(app.world())
            .count(),
        SNOWFLAKE_COUNT,
        "setup_atmosphere did not spawn the snowfall"
    );
    assert_eq!(
        app.world_mut()
            .query::<&Atmosphere>()
            .iter(app.world())
            .count(),
        STAR_COUNT + SNOWFLAKE_COUNT + 1,
        "setup_atmosphere did not spawn the full sky — stars, snow and the aurora curtain"
    );
}

/// Story 7.2's review found `--distance` parsed, validated, and then never reaching the rig, while
/// its only test was NAMED for reaching the camera setup and asserted `parse_args_from` alone.
/// This drives the resource the flag writes, through the startup system that must read it.
#[test]
fn the_capture_distance_resource_reaches_the_camera_rig() {
    let (mut app, _sender) = live_app(one_tile_snapshot());
    app.insert_resource(CaptureDistance(30.0));
    app.update();

    let rig = app
        .world_mut()
        .query::<&CameraRig>()
        .iter(app.world())
        .next()
        .copied()
        .expect("the rig exists");
    assert_eq!(
        rig.distance, 30.0,
        "--distance did not reach the rig; the working-zoom capture would frame the boot vista"
    );
}

/// AD-14/AD-15's partition is only honest if it is TOTAL. `classify_client_local` runs at
/// PostStartup precisely so it sees the finished startup scene, plugin entities included.
#[test]
fn the_classification_pass_leaves_no_entity_outside_the_partition() {
    let (mut app, _sender) = live_app(one_tile_snapshot());
    app.update();

    let unmarked = app
        .world_mut()
        .query_filtered::<BevyEntity, (Without<WorldProjected>, Without<ClientLocal>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        unmarked, 0,
        "{unmarked} entities carry neither partition marker; AD-14's rule is not total"
    );
}

#[test]
fn camera_controls_drive_the_rig() {
    let (mut app, _sender) = live_app(one_tile_snapshot());
    app.update();
    let before = *app
        .world_mut()
        .query::<&CameraRig>()
        .iter(app.world())
        .next()
        .expect("the rig exists");

    press_once(&mut app, KeyCode::KeyE);

    let after = *app
        .world_mut()
        .query::<&CameraRig>()
        .iter(app.world())
        .next()
        .expect("the rig exists");
    assert!(
        after.distance != before.distance,
        "E did not zoom: camera_controls is not driving the rig ({} unchanged)",
        before.distance
    );
}

/// The fog register has to follow the zoom continuum or the vista is a flat sky-coloured
/// rectangle — story 5.4's review found exactly that before the coupling existed.
#[test]
fn fog_follows_the_camera_rig_every_frame() {
    let (mut app, _sender) = live_app(one_tile_snapshot());
    app.update();

    let distance = 240.0_f32;
    {
        let mut rigs = app.world_mut().query::<&mut CameraRig>();
        let mut rig = rigs
            .iter_mut(app.world_mut())
            .next()
            .expect("the rig exists");
        rig.distance = distance;
    }
    // Two frames: `update_fog_from_camera` shares an unordered tuple with `camera_controls`, so
    // one frame could sample either side of the write.
    app.update();
    app.update();

    let (start, end) = fog_falloff(distance);
    let fog = app
        .world_mut()
        .query::<&DistanceFog>()
        .iter(app.world())
        .next()
        .cloned()
        .expect("the camera carries fog");
    match fog.falloff {
        FogFalloff::Linear {
            start: got_start,
            end: got_end,
        } => {
            assert_eq!(
                (got_start, got_end),
                (start, end),
                "fog did not track the rig; update_fog_from_camera is not running"
            );
        }
        other => panic!("fog falloff is no longer linear: {other:?}"),
    }
}

#[test]
fn snow_falls_every_frame() {
    let (mut app, _sender) = live_app(one_tile_snapshot());
    app.update();
    let before: Vec<f32> = app
        .world_mut()
        .query_filtered::<&Transform, With<Snowflake>>()
        .iter(app.world())
        .map(|t| t.translation.y)
        .collect();
    assert!(!before.is_empty(), "no snowflakes to fall");

    app.update();

    let after: Vec<f32> = app
        .world_mut()
        .query_filtered::<&Transform, With<Snowflake>>()
        .iter(app.world())
        .map(|t| t.translation.y)
        .collect();
    assert!(
        before.iter().zip(&after).any(|(b, a)| a < b),
        "no flake descended; fall_snow is not running"
    );
}

#[test]
fn f3_toggles_the_diagnostic_overlay() {
    let (mut app, _sender) = live_app(one_tile_snapshot());
    app.update();
    let before = app.world().resource::<FpsOverlayConfig>().enabled;

    press_once(&mut app, KeyCode::F3);

    assert_eq!(
        app.world().resource::<FpsOverlayConfig>().enabled,
        !before,
        "F3 did not flip the overlay; toggle_overlay is not running"
    );
}

// ---------------------------------------------------------------------------------------------
// Designation interaction, driven THROUGH the shared registration point.
//
// Round 1 of this story's review found that three of the four modes named in the story title,
// both `Esc` transitions, the hint bar and the drag preview could each be broken or deleted with
// the whole suite green. Every test below therefore presses real keys and real mouse buttons on a
// `live_app` and reads the wire commands the client actually queued.
// ---------------------------------------------------------------------------------------------

/// A 3x3x3 world with one solid tile, the camera aimed at it, sliced to its level.
fn designation_app(
    anchor: [i32; 3],
) -> (
    App,
    std::sync::mpsc::SyncSender<anyhow::Result<WireMessage>>,
) {
    let rig = CameraRig::new(anchor);
    let cursor = rig
        .project_world_point(anchor)
        .expect("the anchor tile must project")
        * PICK_VIEWPORT.as_vec2();
    let dims = Dims { x: 3, y: 3, z: 3 };
    let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    let index =
        |[x, y, z]: [i32; 3]| (x + y * dims.x as i32 + z * dims.x as i32 * dims.y as i32) as usize;
    tiles[index(anchor)] = Tile::Solid(Material::Stone);
    let (mut app, sender) = live_app(snapshot_with_dims(dims, tiles, vec![]));
    install_pick_camera(&mut app, rig, cursor);
    app.world_mut().resource_mut::<SliceLevel>().set(anchor[2]);
    app.update();
    (app, sender)
}

fn preview_tiles(app: &mut App) -> Vec<[i32; 3]> {
    let mut tiles = app
        .world_mut()
        .query::<&DragPreview>()
        .iter(app.world())
        .map(|preview| preview.0)
        .collect::<Vec<_>>();
    tiles.sort_unstable();
    tiles
}

fn set_mouse(app: &mut App, act: impl FnOnce(&mut ButtonInput<MouseButton>)) {
    act(&mut app.world_mut().resource_mut::<ButtonInput<MouseButton>>());
}

/// MinimalPlugins does not run InputPlugin's transition clearing between frames.
fn clear_mouse(app: &mut App) {
    set_mouse(app, |mouse| mouse.clear());
}

fn queued(app: &App) -> Vec<protocol::Command> {
    app.world()
        .resource::<PendingCommands>()
        .commands()
        .iter()
        .copied()
        .collect()
}

/// Presses the mode key, then presses and releases the left button on the same tile.
fn drag_one_tile(app: &mut App, mode_key: KeyCode) {
    press_once(app, mode_key);
    set_mouse(app, |mouse| mouse.press(MouseButton::Left));
    app.update();
    clear_mouse(app);
    set_mouse(app, |mouse| mouse.release(MouseButton::Left));
    app.update();
    clear_mouse(app);
}

/// AC2, AC8 and AC10: each of the four mode keys issues ITS OWN wire command, AT THE CELL THE SIM
/// WILL ACCEPT. Collapsing the digit arms, or making channel emit dig, or making stockpile emit
/// nothing, all left the suite green — only `Digit1` was ever pressed anywhere.
///
/// The cell matters as much as the command, and that is what this test gained on 2026-08-27.
/// Picking resolves a SOLID cell; `sim-core` filters channel and stockpile on standability and
/// drops everything else in silence. Sending the picked cell therefore made both modes completely
/// inert against the real daemon — proven, not inferred: a channel rect at the solid cell yields
/// 0 designations and the same rect one level up yields 9. Asserting only the command variant, as
/// this test did before, cannot see that at all.
#[test]
fn each_mode_key_sends_its_own_command_at_the_cell_the_sim_accepts() {
    let picked = [1, 1, 1];
    // The ray enters the top face here, so the standable neighbour is the air cell directly
    // above. Asserted rather than assumed: if the rig ever changes, this fails legibly instead of
    // silently re-testing a different geometry.
    let (probe, _probe_sender) = designation_app(picked);
    assert_eq!(
        probe.world().resource::<gui::pick::PickedTile>().0,
        Some(gui::pick::PickedCell {
            tile: picked,
            face: gui::pick::Face::Top
        }),
        "this test's expectations are written for a top-face hit on the anchor"
    );
    let standable = [1, 1, 2];
    assert!(
        client_core::is_standable(&probe.world().resource::<MirrorResource>().0, standable),
        "the neighbour across the entered face must be a cell the sim will keep — an independent \
         check of the rule, not a re-run of the client's own arithmetic"
    );

    let at = |tile: [i32; 3]| protocol::Rect {
        min: tile,
        max: tile,
    };
    let expected: [(KeyCode, Vec<protocol::Command>); 4] = [
        (
            KeyCode::Digit1,
            vec![protocol::Command::Designate {
                kind: DesignationKind::Dig,
                rect: at(picked),
            }],
        ),
        (
            KeyCode::Digit2,
            vec![protocol::Command::Designate {
                kind: DesignationKind::Channel,
                rect: at(standable),
            }],
        ),
        (
            KeyCode::Digit3,
            vec![protocol::Command::PlaceStockpile {
                rect: at(standable),
            }],
        ),
        // Clear must reach BOTH cells: the dig at the cell the ray hit, and the channel or
        // stockpile one cell across the entered face. Dropping either rect leaves the boss able
        // to designate something he can never remove.
        (
            KeyCode::Digit4,
            vec![
                protocol::Command::CancelDesignation { rect: at(picked) },
                protocol::Command::CancelDesignation {
                    rect: at(standable),
                },
                protocol::Command::RemoveStockpile {
                    rect: at(standable),
                },
            ],
        ),
    ];
    for (key, want) in expected {
        let (mut app, _sender) = designation_app(picked);
        drag_one_tile(&mut app, key);
        assert_eq!(
            queued(&app),
            want,
            "mode key {key:?} must issue its own command, at the cell the sim accepts"
        );
    }
}

/// AC7, AC14: the right-button abort, driven through the shared registration point rather than
/// by `run_system_once` with a hand-inserted anchor.
#[test]
fn right_button_during_a_drag_abandons_it_and_sends_nothing() {
    let (mut app, _sender) = designation_app([1, 1, 1]);
    press_once(&mut app, KeyCode::Digit1);
    set_mouse(&mut app, |mouse| mouse.press(MouseButton::Left));
    app.update();
    assert!(app.world().resource::<DragAnchor>().0.is_some());
    clear_mouse(&mut app);

    set_mouse(&mut app, |mouse| {
        mouse.press(MouseButton::Right);
        mouse.release(MouseButton::Left);
    });
    app.update();
    clear_mouse(&mut app);

    assert_eq!(app.world().resource::<DragAnchor>().0, None);
    assert!(
        queued(&app).is_empty(),
        "an aborted drag must put nothing on the wire"
    );
}

/// AC7's `Esc` half. `Esc` appeared ONLY in production source before this: removing it from the
/// abort condition left every test green.
#[test]
fn escape_during_a_drag_abandons_it_and_sends_nothing() {
    let (mut app, _sender) = designation_app([1, 1, 1]);
    press_once(&mut app, KeyCode::Digit1);
    set_mouse(&mut app, |mouse| mouse.press(MouseButton::Left));
    app.update();
    assert!(app.world().resource::<DragAnchor>().0.is_some());
    clear_mouse(&mut app);

    press_once(&mut app, KeyCode::Escape);
    set_mouse(&mut app, |mouse| mouse.release(MouseButton::Left));
    app.update();
    clear_mouse(&mut app);

    assert_eq!(app.world().resource::<DragAnchor>().0, None);
    assert!(queued(&app).is_empty(), "Esc must abandon the drag");
    assert_eq!(
        *app.world().resource::<DesignateMode>(),
        DesignateMode::Dig,
        "Esc during a drag abandons the DRAG; it does not also leave the mode"
    );
}

/// AC8's second clause: `Esc` with no drag in progress leaves the mode. That `else if` branch was
/// reached by no test at all.
#[test]
fn escape_with_no_drag_leaves_the_mode() {
    let (mut app, _sender) = designation_app([1, 1, 1]);
    press_once(&mut app, KeyCode::Digit2);
    app.update();
    assert_eq!(
        *app.world().resource::<DesignateMode>(),
        DesignateMode::Channel
    );

    press_once(&mut app, KeyCode::Escape);
    app.update();
    assert_eq!(
        *app.world().resource::<DesignateMode>(),
        DesignateMode::None,
        "Esc outside a drag must leave the mode"
    );
}

/// Wolf's ruling at review: a drag commits in the mode it BEGAN in, so a mode key pressed
/// mid-drag takes effect on the next drag rather than silently changing what the release issues.
#[test]
fn a_drag_commits_in_the_mode_it_began_in() {
    let anchor = [1, 1, 1];
    let (mut app, _sender) = designation_app(anchor);
    press_once(&mut app, KeyCode::Digit1);
    set_mouse(&mut app, |mouse| mouse.press(MouseButton::Left));
    app.update();
    clear_mouse(&mut app);

    // Switch to channel WHILE the button is still held.
    press_once(&mut app, KeyCode::Digit2);
    app.update();
    clear_mouse(&mut app);

    set_mouse(&mut app, |mouse| mouse.release(MouseButton::Left));
    app.update();
    clear_mouse(&mut app);

    assert_eq!(
        queued(&app),
        vec![protocol::Command::Designate {
            kind: DesignationKind::Dig,
            rect: protocol::Rect {
                min: anchor,
                max: anchor
            },
        }],
        "the drag began in dig and must commit as dig, whatever was pressed mid-drag"
    );
    assert_eq!(
        *app.world().resource::<DesignateMode>(),
        DesignateMode::Channel,
        "the mode key still takes effect — for the NEXT drag"
    );
    assert_eq!(app.world().resource::<DragMode>().0, None);
}

/// AC6's no-pick clause. `PickedTile(None)` appeared nowhere in the suite.
#[test]
fn a_release_over_nothing_commits_nothing_and_leaves_no_anchor() {
    let (mut app, _sender) = designation_app([1, 1, 1]);
    press_once(&mut app, KeyCode::Digit1);
    set_mouse(&mut app, |mouse| mouse.press(MouseButton::Left));
    app.update();
    assert!(app.world().resource::<DragAnchor>().0.is_some());
    clear_mouse(&mut app);

    // Point at the sky: the cursor leaves the terrain entirely before the release.
    app.world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .single_mut(app.world_mut())
        .unwrap()
        .set_cursor_position(Some(Vec2::new(2.0, 2.0)));
    app.update();
    assert_eq!(
        app.world().resource::<PickedTile>().tile(),
        None,
        "the corner of the viewport must not pick a tile in this world"
    );
    clear_mouse(&mut app);

    set_mouse(&mut app, |mouse| mouse.release(MouseButton::Left));
    app.update();
    clear_mouse(&mut app);

    assert!(
        queued(&app).is_empty(),
        "a release over nothing must commit nothing"
    );
    assert_eq!(
        app.world().resource::<DragAnchor>().0,
        None,
        "a missed release must never leave a stale anchor for the next drag"
    );
}

/// AC7's preview clause and Task 4. `DragPreview` was referenced only by production code, so the
/// whole preview system could be unregistered with the suite green.
#[test]
fn the_drag_preview_appears_while_dragging_and_disappears_on_release() {
    let anchor = [1, 1, 1];
    let (mut app, _sender) = designation_app(anchor);
    assert_eq!(
        app.world_mut()
            .query::<&DragPreview>()
            .iter(app.world())
            .count(),
        0,
        "nothing is previewed before a drag starts"
    );

    press_once(&mut app, KeyCode::Digit1);
    set_mouse(&mut app, |mouse| mouse.press(MouseButton::Left));
    app.update();
    clear_mouse(&mut app);
    let previewed = app
        .world_mut()
        .query::<&DragPreview>()
        .iter(app.world())
        .map(|preview| preview.0)
        .collect::<Vec<_>>();
    assert_eq!(
        previewed,
        vec![anchor],
        "the held drag must preview exactly the tile it covers"
    );

    set_mouse(&mut app, |mouse| mouse.release(MouseButton::Left));
    app.update();
    clear_mouse(&mut app);
    assert_eq!(
        app.world_mut()
            .query::<&DragPreview>()
            .iter(app.world())
            .count(),
        0,
        "the preview must disappear when the drag commits"
    );
}

/// The preview is the ONLY feedback for what a drag will designate, and it was being destroyed
/// on any frame whose ray missed terrain — cursor over sky, over a gap, past the world edge —
/// while the drag stayed live and still committed on release. Wolf hit it by hand on 2026-08-27:
/// "sometimes it loses dragged tile color". Dragging blind is worse than a flicker.
#[test]
fn the_drag_preview_survives_a_frame_whose_ray_misses_terrain() {
    let anchor = [1, 1, 1];
    let (mut app, _sender) = designation_app(anchor);
    press_once(&mut app, KeyCode::Digit1);
    set_mouse(&mut app, |mouse| mouse.press(MouseButton::Left));
    app.update();
    clear_mouse(&mut app);
    assert_eq!(
        preview_tiles(&mut app),
        vec![anchor],
        "the held drag previews the tile it covers"
    );

    // The ray now hits nothing. The DRAG IS STILL LIVE.
    app.world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .single_mut(app.world_mut())
        .expect("the pick harness owns one primary window")
        .set_cursor_position(None);
    app.update();

    assert!(
        app.world().resource::<DragAnchor>().0.is_some(),
        "a missed ray must not end the drag"
    );
    assert_eq!(
        preview_tiles(&mut app),
        vec![anchor],
        "a frame whose ray misses terrain must not erase the preview — the drag is still live \
         and still commits on release, so erasing it leaves the boss dragging blind"
    );
}

/// The sim keeps channel and stockpile designations ONLY at standable cells and drops the rest in
/// silence. An unfiltered preview therefore promised marks that could never appear — which is how
/// two completely inert modes read as "fragile" rather than broken.
#[test]
fn the_preview_covers_only_the_cells_the_sim_will_keep() {
    // Two solid cells on a diagonal. The drag rect is their 2x2 bounding box, but the two off-
    // diagonal columns have no support, so their standable targets do not exist.
    let anchor = [1, 1, 1];
    let release = [2, 2, 1];
    let rig = CameraRig::new(anchor);
    let anchor_cursor = rig
        .project_world_point(anchor)
        .expect("the literal anchor must project")
        * PICK_VIEWPORT.as_vec2();
    let release_cursor = rig
        .project_world_point(release)
        .expect("the literal release tile must project")
        * PICK_VIEWPORT.as_vec2();
    let dims = Dims { x: 4, y: 4, z: 3 };
    let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    let index =
        |[x, y, z]: [i32; 3]| (x + y * dims.x as i32 + z * dims.x as i32 * dims.y as i32) as usize;
    tiles[index(anchor)] = Tile::Solid(Material::Stone);
    tiles[index(release)] = Tile::Solid(Material::Stone);
    let mut app = live_app(snapshot_with_dims(dims, tiles, vec![])).0;
    install_pick_camera(&mut app, rig, anchor_cursor);
    app.world_mut().resource_mut::<SliceLevel>().set(2);
    app.update();

    press_once(&mut app, KeyCode::Digit2);
    set_mouse(&mut app, |mouse| mouse.press(MouseButton::Left));
    app.update();
    clear_mouse(&mut app);
    app.world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .single_mut(app.world_mut())
        .expect("the pick harness owns one primary window")
        .set_cursor_position(Some(release_cursor));
    app.update();

    let previewed = preview_tiles(&mut app);
    let mirror = &app.world().resource::<MirrorResource>().0;
    assert!(
        !previewed.is_empty(),
        "the two supported columns must still preview"
    );
    for tile in &previewed {
        assert!(
            client_core::is_standable(mirror, *tile),
            "previewed {tile:?} is a cell the sim would discard without a word"
        );
    }
    assert_eq!(
        previewed.len(),
        2,
        "the 2x2 rect has only two supported columns; previewing all four promises marks that \
         will never appear. Got {previewed:?}"
    );
}

/// A channel mark sits at the bottom of its air cell, which seals it inside opaque geometry the
/// moment anything is drawn above — and the instruments cannot see it, because a slab inside rock
/// is projected and counted like any other. 7.2 measured 0 of 50 marks visible while the count
/// read 50 and fixed it FOR DIG ONLY. Deleting the climb here restores that defect for channel.
#[test]
fn a_buried_channel_mark_climbs_onto_the_rock_covering_it() {
    let dims = Dims { x: 3, y: 3, z: 4 };
    let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    let index =
        |[x, y, z]: [i32; 3]| (x + y * dims.x as i32 + z * dims.x as i32 * dims.y as i32) as usize;
    // Support beneath the channel cell, and two drawn cells stacked ON TOP of it.
    tiles[index([1, 1, 0])] = Tile::Solid(Material::Stone);
    tiles[index([1, 1, 2])] = Tile::Solid(Material::Stone);
    tiles[index([1, 1, 3])] = Tile::Solid(Material::Stone);
    let (mut app, sender) = live_app(snapshot_with_dims(dims, tiles, vec![]));
    app.update();
    app.world_mut().resource_mut::<SliceLevel>().set(3);
    app.update();
    sender
        .send(Ok(WireMessage::Delta(Box::new(Delta {
            msg_type: MessageType::Delta,
            tick: 1,
            tiles: Vec::new(),
            entities: Vec::new(),
            designations: vec![Designation {
                pos: [1, 1, 1],
                kind: DesignationKind::Channel,
            }],
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
        }))))
        .unwrap();
    app.update();

    let height = app
        .world_mut()
        .query::<(&ProjectedDesignation, &Transform)>()
        .iter(app.world())
        .map(|(_, transform)| transform.translation.y)
        .next()
        .expect("the channel mark must be projected");
    // Render Y is world Z. Unburied the slab would sit at 1 - 0.46 = 0.54, INSIDE the rock at
    // z 2 and z 3. Climbing puts it on the top face of the highest drawn cell above it.
    assert!(
        height > 3.0,
        "a buried channel mark must climb onto the rock covering it; it sits at y {height}, \
         which is inside that rock and invisible while every instrument still counts it"
    );
}

/// MEASURED on the real world: the face neighbour is standable for 100% of TOP-face hits and only
/// 8.5-11.8% of SIDE-face hits, because on flat ground the cell beside a block is another block.
/// Pointing at the front edge of a surface block instead of its top therefore designated nothing —
/// Wolf's "dragging might skip 2 first blocks". The fallback keeps the ledge case and fixes the
/// flat-ground case.
#[test]
fn a_side_face_hit_on_flat_ground_falls_back_to_the_cell_above() {
    let dims = Dims { x: 4, y: 4, z: 3 };
    let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    let index =
        |[x, y, z]: [i32; 3]| (x + y * dims.x as i32 + z * dims.x as i32 * dims.y as i32) as usize;
    // Flat ground: every column solid at z 1, open above.
    for y in 0..dims.y as i32 {
        for x in 0..dims.x as i32 {
            tiles[index([x, y, 1])] = Tile::Solid(Material::Stone);
        }
    }
    let mirror = client_core::Mirror::from_snapshot(snapshot_with_dims(dims, tiles, vec![]))
        .expect("the flat world must build a mirror");

    let picked = [1, 1, 1];
    let east = [2, 1, 1];
    assert!(
        !client_core::is_standable(&mirror, east),
        "on flat ground the cell beside a block is another block — this is the 8.5% case"
    );
    assert_eq!(
        gui::designate::designation_target(
            &mirror,
            gui::pick::PickedCell {
                tile: picked,
                face: gui::pick::Face::East
            },
            DesignateMode::Channel
        ),
        [1, 1, 2],
        "a side-face hit whose neighbour is solid must fall back to the cell above the block"
    );
    // The ledge case Wolf's ruling exists for is NOT sacrificed: where the face neighbour IS
    // standable, it still wins over the cell above.
    let mut ledged = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    ledged[index([1, 1, 1])] = Tile::Solid(Material::Stone);
    ledged[index([1, 1, 2])] = Tile::Solid(Material::Stone);
    ledged[index([2, 1, 0])] = Tile::Solid(Material::Stone);
    let mirror = client_core::Mirror::from_snapshot(snapshot_with_dims(dims, ledged, vec![]))
        .expect("the ledge world must build a mirror");
    assert_eq!(
        gui::designate::designation_target(
            &mirror,
            gui::pick::PickedCell {
                tile: picked,
                face: gui::pick::Face::East
            },
            DesignateMode::Channel
        ),
        [2, 1, 1],
        "pointing at a wall that borders a ledge must still target that ledge"
    );
}

/// AC4's single-z rect kept a median 19.4% of a 6x6 stockpile footprint on natural ground. RULED
/// 2026-08-27 (Wolf): the standable modes follow the surface instead. Dig keeps single-z, where
/// cutting one level into a slope is the point.
#[test]
fn a_channel_drag_across_a_step_follows_the_ground_while_dig_stays_on_one_level() {
    let anchor = [1, 1, 1];
    let release = [2, 1, 0];
    let rig = CameraRig::new(anchor);
    let anchor_cursor = rig
        .project_world_point(anchor)
        .expect("the literal anchor must project")
        * PICK_VIEWPORT.as_vec2();
    let release_cursor = rig
        .project_world_point(release)
        .expect("the literal release tile must project")
        * PICK_VIEWPORT.as_vec2();
    let dims = Dims { x: 3, y: 3, z: 3 };
    let mut tiles = vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize];
    let index =
        |[x, y, z]: [i32; 3]| (x + y * dims.x as i32 + z * dims.x as i32 * dims.y as i32) as usize;
    tiles[index(anchor)] = Tile::Solid(Material::Stone);
    tiles[index(release)] = Tile::Solid(Material::Stone);

    let drag = |key: KeyCode| {
        let mut app = live_app(snapshot_with_dims(dims, tiles.clone(), vec![])).0;
        install_pick_camera(&mut app, rig, anchor_cursor);
        app.world_mut().resource_mut::<SliceLevel>().set(2);
        app.update();
        press_once(&mut app, key);
        set_mouse(&mut app, |mouse| mouse.press(MouseButton::Left));
        app.update();
        clear_mouse(&mut app);
        app.world_mut()
            .query_filtered::<&mut Window, With<PrimaryWindow>>()
            .single_mut(app.world_mut())
            .expect("the pick harness owns one primary window")
            .set_cursor_position(Some(release_cursor));
        set_mouse(&mut app, |mouse| mouse.release(MouseButton::Left));
        app.update();
        clear_mouse(&mut app);
        queued(&app)
    };

    assert_eq!(
        drag(KeyCode::Digit2),
        vec![
            protocol::Command::Designate {
                kind: DesignationKind::Channel,
                rect: protocol::Rect {
                    min: [1, 1, 2],
                    max: [1, 1, 2]
                },
            },
            protocol::Command::Designate {
                kind: DesignationKind::Channel,
                rect: protocol::Rect {
                    min: [2, 1, 1],
                    max: [2, 1, 1]
                },
            },
        ],
        "a channel drag across a step must reach the ground at BOTH heights, not flatten to one"
    );
    assert_eq!(
        drag(KeyCode::Digit1),
        vec![protocol::Command::Designate {
            kind: DesignationKind::Dig,
            rect: protocol::Rect {
                min: [1, 1, 1],
                max: [2, 1, 1]
            },
        }],
        "dig is unchanged: one single-z rect at the anchor's level"
    );
}

/// AC9's load-bearing clause: the bar NAMES THE ACTIVE MODE. Neutering `update_designate_hint`
/// left the suite green and the bar reading its no-mode string forever.
#[test]
fn the_hint_bar_names_the_mode_that_will_commit() {
    let (mut app, _sender) = designation_app([1, 1, 1]);
    let read_hint = |app: &mut App| -> String {
        app.world_mut()
            .query_filtered::<&Text, With<DesignateHint>>()
            .single(app.world())
            .expect("the hint bar must exist")
            .0
            .clone()
    };
    assert_eq!(
        read_hint(&mut app),
        designation_hint(DesignateMode::None, false)
    );

    press_once(&mut app, KeyCode::Digit3);
    app.update();
    assert_eq!(
        read_hint(&mut app),
        designation_hint(DesignateMode::Stockpile, false),
        "the bar must name the mode the operator just selected"
    );

    set_mouse(&mut app, |mouse| mouse.press(MouseButton::Left));
    app.update();
    clear_mouse(&mut app);
    assert_eq!(
        read_hint(&mut app),
        designation_hint(DesignateMode::Stockpile, true),
        "the bar must switch to its dragging text while a drag is live"
    );
}
