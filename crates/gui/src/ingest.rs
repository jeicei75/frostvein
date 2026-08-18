use std::{
    collections::BTreeSet,
    ffi::OsString,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
    path::PathBuf,
    sync::{
        Mutex,
        mpsc::{self, Receiver, SyncSender, TryRecvError},
    },
    thread,
    time::Duration,
};

use anyhow::{Context, bail};
use bevy::{
    app::{App, AppExit, Startup, Update},
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::message::MessageWriter,
    ecs::schedule::IntoScheduleConfigs,
    input::ButtonInput,
    pbr::{DistanceFog, FogFalloff},
    prelude::{
        AmbientLight, Camera3d, ClearColor, Color, Commands, Component, DefaultPlugins,
        DirectionalLight, KeyCode, Node, PerspectiveProjection, PositionType, Projection, Query,
        Res, ResMut, Resource, Text, TextColor, TextFont, Time, Transform, With, Without, px,
    },
    render::renderer::RenderAdapterInfo,
};
use client_core::Mirror;
use protocol::{Delta, Snapshot};

use crate::{
    appearance::night_lighting,
    atmosphere::{aurora_light_transform, fall_snow, setup_atmosphere},
    blend::TickClock,
    camera::{BOOT_VERTICAL_FOV, CameraRig},
    capture::{CaptureState, accumulate_motion, capture_after_frames},
    project::{
        ClientLocal, DigChipQuery, ProjectionAssets, TerrainQuery, TerrainTile, WorldProjected,
        blend_entities, flicker_lights, reconcile, setup_projection_assets,
    },
    slice::SliceLevel,
};

const SNAPSHOT_READ_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;
const MESSAGE_QUEUE: usize = 16;

enum WireMessage {
    Snapshot(Box<Snapshot>),
    Delta(Box<Delta>),
}

#[derive(Resource)]
pub struct MirrorResource(pub Mirror);

#[derive(Resource)]
struct IngestReceiver(Mutex<Receiver<anyhow::Result<WireMessage>>>);

#[derive(Resource, Default)]
pub struct ProjectionWork {
    pub snapshot: bool,
    pub dirty_tiles: BTreeSet<[i32; 3]>,
}

pub fn run() -> anyhow::Result<()> {
    let args = parse_args()?;
    let address = format!("127.0.0.1:{}", args.port);
    let stream = TcpStream::connect(("127.0.0.1", args.port))
        .with_context(|| format!("could not connect to {address}"))?;
    stream
        .set_read_timeout(Some(SNAPSHOT_READ_TIMEOUT))
        .context("could not set snapshot read timeout")?;
    let mut reader = BufReader::new(stream);
    let mirror = Mirror::from_snapshot(read_snapshot(&mut reader)?)
        .context("could not build client mirror")?;
    let (sender, receiver) = mpsc::sync_channel(MESSAGE_QUEUE);
    thread::Builder::new()
        .name("server-read".to_string())
        .spawn(move || read_messages(reader, sender))
        .context("could not spawn server reader thread")?;

    let slice = args.slice_level.map_or_else(
        || SliceLevel::at_world_top(mirror.dims()),
        |level| SliceLevel::pinned(mirror.dims(), level),
    );
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(FpsOverlayPlugin {
            config: overlay_config_off(),
        })
        .insert_resource(MirrorResource(mirror))
        .insert_resource(slice)
        .insert_resource(IngestReceiver(Mutex::new(receiver)))
        .insert_resource(ProjectionWork {
            snapshot: true,
            dirty_tiles: BTreeSet::new(),
        })
        .insert_resource(ClearColor(night_lighting().sky))
        .add_systems(
            Startup,
            (
                setup_camera,
                setup_night_lighting,
                setup_projection_assets,
                setup_atmosphere,
                setup_slice_readout,
                log_adapter,
            ),
        )
        // Bevy's overlay plugin owns opaque UI component types. Every entity it creates is
        // still GUI-local, so classify the complete startup scene after all plugin setup.
        .add_systems(bevy::app::PostStartup, classify_client_local)
        .add_systems(
            Update,
            (
                camera_controls,
                update_fog_from_camera,
                update_slice_readout,
                toggle_overlay,
                fall_snow,
            ),
        );
    projection_systems(&mut app);
    if let Some(capture) = args.capture {
        // Capture output must never contain the diagnostic overlay.
        force_capture_overlay_off(&mut app);
        app.insert_resource(CaptureState::new(capture, args.frames, args.expect_work));
        // The instrument reads what the projection chain just wrote, so it must run after it.
        // Bevy's ambiguity detection defaults to `LogLevel::Ignore`, so an unordered read here
        // would be resolved silently and sample the frame at an undefined point.
        app.add_systems(
            Update,
            (accumulate_motion, capture_after_frames)
                .chain()
                .after(ProjectionSet),
        );
    }
    app.run();
    Ok(())
}

/// The whole wire-to-presentation chain, so anything that reads its output can be ordered
/// behind it by name rather than by registration accident.
#[derive(bevy::prelude::SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectionSet;

/// Registers the load-bearing wire-to-presentation pipeline for both the live app and headless
/// tests. Keeping it in one place makes an omitted live system observable in the suite.
pub fn projection_systems(app: &mut App) {
    if !app.world().contains_resource::<SliceLevel>() {
        let dims = app.world().resource::<MirrorResource>().0.dims();
        app.insert_resource(SliceLevel::at_world_top(dims));
    }
    app.init_resource::<TickClock>().add_systems(
        Update,
        (
            ingest_messages,
            slice_controls,
            reconcile_projection,
            blend_projection,
            flicker_projection,
        )
            .chain()
            .in_set(ProjectionSet),
    );
}

fn force_capture_overlay_off(app: &mut App) {
    let mut config = app.world_mut().resource_mut::<FpsOverlayConfig>();
    config.enabled = false;
    config.frame_time_graph_config.enabled = false;
}

fn overlay_config_off() -> FpsOverlayConfig {
    let mut config = FpsOverlayConfig {
        enabled: false,
        ..Default::default()
    };
    config.frame_time_graph_config.enabled = false;
    config
}

struct Args {
    port: u16,
    capture: Option<PathBuf>,
    frames: u32,
    expect_work: bool,
    slice_level: Option<i32>,
}

fn parse_args() -> anyhow::Result<Args> {
    parse_args_from(std::env::args_os().skip(1))
}

fn parse_args_from(args: impl IntoIterator<Item = OsString>) -> anyhow::Result<Args> {
    let mut port = protocol::DEFAULT_PORT;
    let mut capture = None;
    let mut frames = None;
    let mut expect_work = false;
    let mut slice_level = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--capture" {
            let path = args.next().context("--capture requires a path")?;
            capture = Some(PathBuf::from(path));
        } else if arg == "--frames" {
            let value = args.next().context("--frames requires a positive count")?;
            frames = Some(
                value
                    .to_string_lossy()
                    .parse()
                    .context("invalid --frames count")?,
            );
        } else if arg == "--expect-work" {
            expect_work = true;
        } else if arg == "--z" {
            let value = args.next().context("--z requires a level")?;
            slice_level = Some(
                value
                    .to_string_lossy()
                    .parse()
                    .context("invalid --z level")?,
            );
        } else {
            port = arg.to_string_lossy().parse().context("invalid port")?;
        }
    }
    if capture.is_some() && frames.is_none() {
        bail!("--capture requires --frames N");
    }
    if capture.is_some() && frames == Some(0) {
        bail!("--capture --frames must be positive");
    }
    if expect_work && capture.is_none() {
        bail!("--expect-work requires --capture");
    }
    if slice_level.is_some() && capture.is_none() {
        bail!("--z requires --capture");
    }
    Ok(Args {
        port,
        capture,
        frames: frames.unwrap_or(0),
        expect_work,
        slice_level,
    })
}

fn setup_camera(mut commands: Commands) {
    let rig = CameraRig::new([64, 64, 9]);
    let (fog_start, fog_end) = fog_falloff(rig.distance);
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: BOOT_VERTICAL_FOV,
            ..Default::default()
        }),
        rig.transform(),
        rig,
        AmbientLight {
            color: night_lighting().ambient,
            brightness: night_lighting().ambient_brightness,
            ..Default::default()
        },
        DistanceFog {
            color: night_lighting().sky,
            falloff: FogFalloff::Linear {
                start: fog_start,
                end: fog_end,
            },
            ..Default::default()
        },
        ClientLocal,
    ));
}

fn setup_night_lighting(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            color: night_lighting().directional,
            illuminance: night_lighting().directional_illuminance,
            shadow_maps_enabled: true,
            ..Default::default()
        },
        aurora_light_transform(),
        ClientLocal,
    ));
}

#[derive(Component)]
struct SliceReadout;

fn setup_slice_readout(mut commands: Commands, slice: Res<SliceLevel>) {
    commands.spawn((
        Text::new(slice.readout()),
        TextFont::from_font_size(22.0),
        TextColor(Color::srgb(0.86, 0.91, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            ..Default::default()
        },
        SliceReadout,
        ClientLocal,
    ));
}

fn update_slice_readout(slice: Res<SliceLevel>, mut readout: Query<&mut Text, With<SliceReadout>>) {
    for mut text in &mut readout {
        *text = Text::new(slice.readout());
    }
}

fn classify_client_local(
    mut commands: Commands,
    unclassified: Query<bevy::prelude::Entity, (Without<WorldProjected>, Without<ClientLocal>)>,
) {
    for entity in &unclassified {
        commands.entity(entity).insert(ClientLocal);
    }
}

fn log_adapter(adapter: Option<Res<RenderAdapterInfo>>) {
    if let Some(adapter) = adapter {
        println!(
            "backend={:?} adapter={:?} device_type={:?} driver={:?} driver_info={:?}",
            adapter.backend, adapter.name, adapter.device_type, adapter.driver, adapter.driver_info
        );
    } else {
        eprintln!("renderer adapter information is unavailable");
    }
}

fn camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<(&mut CameraRig, &mut Transform)>,
) {
    let yaw = (keys.pressed(KeyCode::KeyD) as i8 - keys.pressed(KeyCode::KeyA) as i8) as f32 * 0.02;
    let pitch =
        (keys.pressed(KeyCode::KeyW) as i8 - keys.pressed(KeyCode::KeyS) as i8) as f32 * 0.02;
    let zoom = (keys.pressed(KeyCode::KeyE) as i8 - keys.pressed(KeyCode::KeyQ) as i8) as f32 * 1.0;
    for (mut rig, mut transform) in &mut cameras {
        rig.orbit(yaw, pitch);
        rig.zoom(zoom);
        *transform = rig.transform();
    }
}

/// `<` / `>` use the comma and period keys today. The planned wheel zoom remains unclaimed until
/// UX-DR2 lands, so this client-local binding does not create a future migration.
fn slice_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mirror: Res<MirrorResource>,
    mut slice: ResMut<SliceLevel>,
    mut work: ResMut<ProjectionWork>,
) {
    let mut changed = slice.rebind(mirror.0.dims());
    if keys.just_pressed(KeyCode::Comma) {
        changed |= slice.step(-1);
    }
    if keys.just_pressed(KeyCode::Period) {
        changed |= slice.step(1);
    }
    if changed {
        work.snapshot = true;
    }
}

/// Aerial perspective, measured against the boot framing: the camp reads at depth 71 and the
/// deepest in-frame terrain at 148, so fog opens just past the camp and saturates just past the
/// far valley. It is NOT the world-edge treatment — the silhouette starts at depth 86, and fog
/// tight enough to hide that would erase the valley with it. `rim_level` dissolves the edge.
///
/// NOTE: the vehicle comparison still chooses the final edge treatment; this keeps the fog
/// register valid across the pinned 4-500 zoom clamp.
pub fn fog_falloff(camera_distance: f32) -> (f32, f32) {
    (
        70.0_f32.max(camera_distance - 20.0),
        210.0_f32.max(camera_distance * 1.7),
    )
}

/// The share of a surface's colour replaced by fog at `depth`, for the linear falloff above.
pub fn fog_fraction(camera_distance: f32, depth: f32) -> f32 {
    let (start, end) = fog_falloff(camera_distance);
    ((depth - start) / (end - start)).clamp(0.0, 1.0)
}

fn update_fog_from_camera(mut cameras: Query<(&CameraRig, &mut DistanceFog)>) {
    for (rig, mut fog) in &mut cameras {
        let (start, end) = fog_falloff(rig.distance);
        fog.falloff = FogFalloff::Linear { start, end };
    }
}

fn toggle_overlay(keys: Res<ButtonInput<KeyCode>>, mut config: ResMut<FpsOverlayConfig>) {
    if keys.just_pressed(KeyCode::F3) {
        let enabled = !config.enabled;
        config.enabled = enabled;
        config.frame_time_graph_config.enabled = enabled;
    }
}

/// The only GUI system that reads protocol message types; it mutates only the mirror.
fn ingest_messages(
    receiver: Option<Res<IngestReceiver>>,
    mut mirror: ResMut<MirrorResource>,
    mut work: ResMut<ProjectionWork>,
    mut clock: ResMut<TickClock>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(receiver) = receiver else {
        return;
    };
    loop {
        match receiver
            .0
            .lock()
            .expect("ingest receiver mutex poisoned")
            .try_recv()
        {
            Ok(Ok(WireMessage::Snapshot(snapshot))) => match mirror.0.apply_snapshot(*snapshot) {
                Ok(()) => {
                    work.snapshot = true;
                    work.dirty_tiles.clear();
                    clock.reset(mirror.0.tick());
                }
                Err(error) => {
                    // A frozen window with no diagnostic is worse than a loud exit; the
                    // sibling client bails on this same condition.
                    eprintln!("could not apply server snapshot: {error}");
                    exit.write(AppExit::error());
                }
            },
            Ok(Ok(WireMessage::Delta(delta))) => {
                mirror.0.apply_delta(*delta);
                clock.observe_tick(mirror.0.tick());
                work.dirty_tiles
                    .extend(mirror.0.changes().tiles.iter().copied());
            }
            Ok(Err(error)) => {
                eprintln!("server reader stopped: {error:#}");
                exit.write(AppExit::error());
            }
            Err(TryRecvError::Disconnected) => {
                eprintln!("server connection lost");
                exit.write(AppExit::error());
                break;
            }
            Err(TryRecvError::Empty) => break,
        }
    }
}

fn blend_projection(
    mirror: Res<MirrorResource>,
    mut clock: ResMut<TickClock>,
    time: Res<Time>,
    mut projected: Query<(&WorldProjected, &mut Transform), Without<TerrainTile>>,
) {
    blend_entities(&mirror.0, &mut clock, time.delta_secs(), &mut projected);
}

// Each parameter is a distinct ECS partition; bundling them solely to reduce the system signature
// would obscure the client-local slice boundary from the mirror and projection queries.
#[allow(clippy::too_many_arguments)]
pub fn reconcile_projection(
    mut commands: Commands,
    mirror: Res<MirrorResource>,
    slice: Res<SliceLevel>,
    mut work: ResMut<ProjectionWork>,
    projected: Query<
        (
            bevy::prelude::Entity,
            &WorldProjected,
            Option<&crate::project::ProjectedLight>,
        ),
        Without<TerrainTile>,
    >,
    terrain: TerrainQuery,
    chips: DigChipQuery,
    assets: Option<Res<ProjectionAssets>>,
) {
    let rebuild = std::mem::take(&mut work.snapshot);
    let changes = std::mem::take(&mut work.dirty_tiles)
        .into_iter()
        .collect::<Vec<_>>();
    reconcile(
        &mut commands,
        &mirror.0,
        *slice,
        rebuild,
        &changes,
        &projected,
        &terrain,
        &chips,
        assets.as_deref(),
    );
}

fn flicker_projection(
    time: Res<Time>,
    mut lights: Query<(
        &WorldProjected,
        &crate::project::ProjectedLight,
        &mut bevy::prelude::PointLight,
    )>,
) {
    flicker_lights(time.elapsed_secs(), &mut lights);
}

fn read_snapshot(reader: &mut dyn BufRead) -> anyhow::Result<Snapshot> {
    match read_message(reader)? {
        Some(WireMessage::Snapshot(snapshot)) => Ok(*snapshot),
        Some(WireMessage::Delta(_)) => bail!("server sent a delta before its snapshot"),
        None => bail!("server closed before sending a snapshot"),
    }
}

fn read_message(reader: &mut dyn BufRead) -> anyhow::Result<Option<WireMessage>> {
    let mut line = String::new();
    let bytes = reader
        .take(MAX_SNAPSHOT_BYTES)
        .read_line(&mut line)
        .context("could not read server message")?;
    if bytes == 0 {
        return Ok(None);
    }
    if !line.ends_with('\n') {
        if bytes as u64 >= MAX_SNAPSHOT_BYTES {
            bail!("server message exceeded {MAX_SNAPSHOT_BYTES} bytes");
        }
        bail!("server closed before terminating its message line");
    }
    let value: serde_json::Value =
        serde_json::from_str(&line).context("could not decode server message")?;
    match value.get("type").and_then(serde_json::Value::as_str) {
        Some("snapshot") => Ok(Some(WireMessage::Snapshot(Box::new(
            serde_json::from_value(value)?,
        )))),
        Some("delta") => Ok(Some(WireMessage::Delta(Box::new(serde_json::from_value(
            value,
        )?)))),
        Some(kind) => bail!("unknown server message type {kind:?}"),
        None => bail!("server message has no string type field"),
    }
}

fn read_messages(
    mut reader: BufReader<TcpStream>,
    sender: SyncSender<anyhow::Result<WireMessage>>,
) {
    loop {
        let message = match read_message(&mut reader) {
            Ok(Some(message)) => Ok(message),
            Ok(None) => Err(anyhow::anyhow!("server closed the connection")),
            Err(error) => Err(error),
        };
        let done = message.is_err();
        if sender.send(message).is_err() || done {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, mpsc};

    use bevy::{
        app::{App, Update},
        dev_tools::fps_overlay::FpsOverlayConfig,
    };
    use client_core::Mirror;
    use protocol::{Delta, Dims, MessageType, Snapshot, Speed, Tile, TileChange};

    use super::{
        ClientLocal, IngestReceiver, MirrorResource, ProjectionWork, WireMessage,
        classify_client_local, fog_falloff, fog_fraction, force_capture_overlay_off,
        ingest_messages,
    };
    use crate::blend::TickClock;
    use crate::project::WorldProjected;

    #[test]
    fn capture_forces_the_frame_time_overlay_off() {
        let mut app = App::new();
        let mut overlay = FpsOverlayConfig {
            enabled: true,
            ..Default::default()
        };
        overlay.frame_time_graph_config.enabled = true;
        app.insert_resource(overlay);

        force_capture_overlay_off(&mut app);

        assert!(!app.world().resource::<FpsOverlayConfig>().enabled);
        assert!(
            !app.world()
                .resource::<FpsOverlayConfig>()
                .frame_time_graph_config
                .enabled
        );
    }

    #[test]
    fn capture_requires_a_positive_frame_count() {
        assert!(
            super::parse_args_from([
                std::ffi::OsString::from("--capture"),
                std::ffi::OsString::from("out.png"),
                std::ffi::OsString::from("--frames"),
                std::ffi::OsString::from("0"),
            ])
            .is_err(),
            "a zero-frame capture must be rejected before opening a socket"
        );
    }

    #[test]
    fn capture_slice_level_requires_capture_and_is_retained_for_pinning() {
        assert!(
            super::parse_args_from([
                std::ffi::OsString::from("--z"),
                std::ffi::OsString::from("9"),
            ])
            .is_err(),
            "a level without a capture has no reproducible instrument to pin"
        );
        let args = super::parse_args_from([
            std::ffi::OsString::from("7451"),
            std::ffi::OsString::from("--capture"),
            std::ffi::OsString::from("slice.png"),
            std::ffi::OsString::from("--frames"),
            std::ffi::OsString::from("12"),
            std::ffi::OsString::from("--z"),
            std::ffi::OsString::from("9"),
        ])
        .expect("a capture level must parse");
        assert_eq!(args.slice_level, Some(9));
    }

    #[test]
    fn fog_range_tracks_the_camera_without_erasing_the_far_edge() {
        assert_eq!(fog_falloff(4.0), (70.0, 210.0));
        assert_eq!(fog_falloff(90.0), (70.0, 210.0));
        assert_eq!(fog_falloff(500.0), (480.0, 850.0));

        // Depths measured off the ROUND-8 boot framing (skyline moved to 24%): camp 60,
        // nearest skyline 80, deepest in-frame terrain 138. Assert the FRACTION, so a range
        // that technically "ends later than the world" but greys the valley still fails.
        const BOOT: f32 = 90.0;
        assert_eq!(
            fog_fraction(BOOT, 60.0),
            0.0,
            "the camp must sit completely clear of the fog"
        );
        let skyline = fog_fraction(BOOT, 80.0);
        assert!(
            (0.03..0.30).contains(&skyline),
            "the near skyline needs air without being erased; {skyline}"
        );
        // A BAND, not a floor: at 0.94 fogged (the round-6 range) the ridge vanished into the
        // sky and the apparent horizon dropped — but the aurora is supposed to BACKLIGHT the
        // skyline (UX-DR12/5.1), which needs a visible silhouette. The world edge belongs to
        // the rim dissolve now, not to fog.
        let far = fog_fraction(BOOT, 138.0);
        assert!(
            (0.35..0.70).contains(&far),
            "the far valley must read as distance yet keep its backlit silhouette; {far}"
        );

        // At full vista the world must survive: the whole map inside one fog range would be
        // the flat sky-coloured rectangle the review found at the fixed range.
        assert!(
            fog_fraction(500.0, 500.0) <= 0.10,
            "the vista must not fog out the world it is meant to show"
        );
    }

    #[test]
    fn startup_entities_without_world_projection_are_client_local() {
        let mut app = App::new();
        app.add_systems(bevy::app::PostStartup, classify_client_local);
        app.world_mut().spawn(WorldProjected(7));
        app.world_mut().spawn_empty();

        app.update();

        let mut unclassified = app.world_mut().query_filtered::<bevy::prelude::Entity, (
            bevy::ecs::query::Without<WorldProjected>,
            bevy::ecs::query::Without<ClientLocal>,
        )>();
        assert_eq!(
            unclassified.iter(app.world()).count(),
            0,
            "overlay and other startup entities must be structurally client-local"
        );
        let mut projected = app
            .world_mut()
            .query::<(&WorldProjected, Option<&ClientLocal>)>();
        assert!(
            projected
                .iter(app.world())
                .all(|(_, local)| local.is_none()),
            "world projection must stay disjoint from client-local entities"
        );
    }

    #[test]
    fn ingestion_accumulates_dirty_tiles_from_queued_deltas() {
        let mirror = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Empty, Tile::Empty],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap();
        let (sender, receiver) = mpsc::sync_channel(2);
        for position in [[0, 0, 0], [1, 0, 0]] {
            sender
                .send(Ok(WireMessage::Delta(Box::new(Delta {
                    msg_type: MessageType::Delta,
                    tick: 1,
                    tiles: vec![TileChange {
                        pos: position,
                        tile: Tile::Solid(protocol::Material::Ice),
                    }],
                    entities: Vec::new(),
                    designations: Vec::new(),
                    zones: Vec::new(),
                    items: Vec::new(),
                    speed: Speed::Normal,
                }))))
                .unwrap();
        }
        let mut app = App::new();
        app.insert_resource(MirrorResource(mirror))
            .insert_resource(IngestReceiver(Mutex::new(receiver)))
            .init_resource::<ProjectionWork>()
            .init_resource::<TickClock>()
            .add_systems(Update, ingest_messages);

        app.update();

        assert_eq!(
            app.world().resource::<ProjectionWork>().dirty_tiles,
            [[0, 0, 0], [1, 0, 0]].into_iter().collect()
        );
    }

    /// The headless seam tests drive `observe_tick` by hand, so nothing asserted that the
    /// production ingest path re-bases the clock: deleting `clock.observe_tick(...)` here left
    /// the whole suite green while every dwarf snapped tile to tile on the vehicle.
    #[test]
    fn ingesting_a_delta_rebases_the_blend_clock_from_the_wire() {
        let mirror = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Empty, Tile::Empty],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        sender
            .send(Ok(WireMessage::Delta(Box::new(protocol::Delta {
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
        let mut app = App::new();
        app.insert_resource(MirrorResource(mirror))
            .insert_resource(IngestReceiver(Mutex::new(receiver)))
            .init_resource::<ProjectionWork>()
            .init_resource::<TickClock>()
            .add_systems(Update, ingest_messages);
        // A full cadence has already elapsed on the client when the delta lands.
        app.world_mut().resource_mut::<TickClock>().advance(0.1);

        app.update();

        let clock = app.world().resource::<TickClock>();
        assert_eq!(
            clock.last_tick(),
            1,
            "ingest must re-base the blend clock on the delivered tick"
        );
        assert_eq!(clock.elapsed(), 0.0, "the new interval starts at the delta");
        assert_eq!(
            clock.interval(),
            0.1,
            "the cadence is measured from the wire, never assumed"
        );
    }

    #[test]
    fn a_wire_snapshot_arms_the_full_rebuild_and_drops_stale_dirty_tiles() {
        let mirror = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Empty, Tile::Empty],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        sender
            .send(Ok(WireMessage::Snapshot(Box::new(Snapshot {
                msg_type: MessageType::Snapshot,
                dims: Dims { x: 2, y: 1, z: 1 },
                tiles: vec![Tile::Solid(protocol::Material::Ice), Tile::Empty],
                entities: Vec::new(),
                designations: Vec::new(),
                zones: Vec::new(),
                items: Vec::new(),
                speed: Speed::Normal,
                tick: 2,
            }))))
            .unwrap();
        let mut app = App::new();
        app.insert_resource(MirrorResource(mirror))
            .insert_resource(IngestReceiver(Mutex::new(receiver)))
            .insert_resource(ProjectionWork {
                snapshot: false,
                dirty_tiles: [[1, 0, 0]].into_iter().collect(),
            })
            .init_resource::<TickClock>()
            .add_systems(Update, ingest_messages);

        app.update();

        let work = app.world().resource::<ProjectionWork>();
        assert!(
            work.snapshot,
            "a wire snapshot must arm the full terrain rebuild"
        );
        assert!(
            work.dirty_tiles.is_empty(),
            "stale dirty tiles must not survive a snapshot"
        );
        assert_eq!(app.world().resource::<MirrorResource>().0.tick(), 2);
    }

    #[test]
    fn recorded_wire_data_mutates_only_the_mirror() {
        let mirror = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Empty, Tile::Empty],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap();
        let (sender, receiver) = mpsc::sync_channel(2);
        let mut app = App::new();
        app.insert_resource(MirrorResource(mirror))
            .insert_resource(IngestReceiver(Mutex::new(receiver)))
            .init_resource::<ProjectionWork>()
            .init_resource::<TickClock>()
            .add_systems(Update, ingest_messages);
        // Settle schedule and system entities before ingesting anything.
        app.update();
        let baseline = app.world().entities().len();

        let recorded: Snapshot = serde_json::from_str(
            r#"{
                "type":"snapshot", "dims":{"x":2,"y":1,"z":1},
                "tiles":[{"solid":"ice"},"empty"],
                "entities":[{"id":7,"kind":"dwarf","pos":[1,0,0],"state":"idle","light":null}],
                "designations":[], "zones":[], "items":[], "speed":"normal", "tick":4
            }"#,
        )
        .unwrap();
        let recorded_delta: Delta = serde_json::from_str(
            r#"{
                "type":"delta", "tick":5,
                "tiles":[{"pos":[0,0,0],"tile":{"solid":"ice"}}],
                "entities":[], "designations":[], "zones":[], "items":[], "speed":"normal"
            }"#,
        )
        .unwrap();
        sender
            .send(Ok(WireMessage::Snapshot(Box::new(recorded))))
            .unwrap();
        sender
            .send(Ok(WireMessage::Delta(Box::new(recorded_delta))))
            .unwrap();
        app.update();

        assert_eq!(
            app.world().entities().len(),
            baseline,
            "ingestion must never spawn or despawn a Bevy entity"
        );
        assert_eq!(
            app.world().resource::<MirrorResource>().0.tick(),
            5,
            "the mirror must have consumed the recorded wire data"
        );
        assert!(app.world().resource::<ProjectionWork>().snapshot);
    }
}
