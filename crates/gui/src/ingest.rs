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
    input::ButtonInput,
    pbr::{DistanceFog, FogFalloff},
    prelude::{
        AmbientLight, Camera3d, ClearColor, Commands, DefaultPlugins, DirectionalLight, KeyCode,
        Query, Res, ResMut, Resource, Transform, Without,
    },
    render::renderer::RenderAdapterInfo,
};
use client_core::Mirror;
use protocol::{Delta, Snapshot};

use crate::{
    appearance::night_lighting,
    atmosphere::{fall_snow, setup_atmosphere},
    camera::CameraRig,
    capture::{CaptureState, capture_after_frames},
    project::{
        ClientLocal, ProjectionAssets, TerrainQuery, TerrainTile, WorldProjected, reconcile,
        setup_projection_assets,
    },
};

const SNAPSHOT_READ_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;
const MESSAGE_QUEUE: usize = 16;

enum WireMessage {
    Snapshot(Box<Snapshot>),
    Delta(Box<Delta>),
}

#[derive(Resource)]
struct MirrorResource(Mirror);

#[derive(Resource)]
struct IngestReceiver(Mutex<Receiver<anyhow::Result<WireMessage>>>);

#[derive(Resource, Default)]
struct ProjectionWork {
    snapshot: bool,
    dirty_tiles: BTreeSet<[i32; 3]>,
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

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                enabled: false,
                ..Default::default()
            },
        })
        .insert_resource(MirrorResource(mirror))
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
                log_adapter,
            ),
        )
        // Bevy's overlay plugin owns opaque UI component types. Every entity it creates is
        // still GUI-local, so classify the complete startup scene after all plugin setup.
        .add_systems(bevy::app::PostStartup, classify_client_local)
        .add_systems(
            Update,
            (
                ingest_messages,
                reconcile_projection,
                camera_controls,
                toggle_overlay,
                fall_snow,
            ),
        );
    if let Some(capture) = args.capture {
        // Capture output must never contain the diagnostic overlay.
        force_capture_overlay_off(&mut app);
        app.insert_resource(CaptureState::new(capture, args.frames));
        app.add_systems(Update, capture_after_frames);
    }
    app.run();
    Ok(())
}

fn force_capture_overlay_off(app: &mut App) {
    app.world_mut().resource_mut::<FpsOverlayConfig>().enabled = false;
}

struct Args {
    port: u16,
    capture: Option<PathBuf>,
    frames: u32,
}

fn parse_args() -> anyhow::Result<Args> {
    parse_args_from(std::env::args_os().skip(1))
}

fn parse_args_from(args: impl IntoIterator<Item = OsString>) -> anyhow::Result<Args> {
    let mut port = protocol::DEFAULT_PORT;
    let mut capture = None;
    let mut frames = None;
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
    Ok(Args {
        port,
        capture,
        frames: frames.unwrap_or(0),
    })
}

fn setup_camera(mut commands: Commands) {
    let rig = CameraRig::new([64, 64, 9]);
    commands.spawn((
        Camera3d::default(),
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
                start: 85.0,
                end: 180.0,
            },
            ..Default::default()
        },
        ClientLocal,
    ));
}

fn setup_night_lighting(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            color: night_lighting().aurora,
            illuminance: night_lighting().directional_illuminance,
            shadow_maps_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(-20.0, 40.0, 20.0)
            .looking_at(bevy::prelude::Vec3::ZERO, bevy::prelude::Vec3::Y),
        ClientLocal,
    ));
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

fn toggle_overlay(keys: Res<ButtonInput<KeyCode>>, mut config: ResMut<FpsOverlayConfig>) {
    if keys.just_pressed(KeyCode::F3) {
        config.enabled = !config.enabled;
    }
}

/// The only GUI system that reads protocol message types; it mutates only the mirror.
fn ingest_messages(
    receiver: Res<IngestReceiver>,
    mut mirror: ResMut<MirrorResource>,
    mut work: ResMut<ProjectionWork>,
    mut exit: MessageWriter<AppExit>,
) {
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

fn reconcile_projection(
    mut commands: Commands,
    mirror: Res<MirrorResource>,
    mut work: ResMut<ProjectionWork>,
    projected: Query<(bevy::prelude::Entity, &WorldProjected), Without<TerrainTile>>,
    terrain: TerrainQuery,
    assets: Option<Res<ProjectionAssets>>,
) {
    let rebuild = std::mem::take(&mut work.snapshot);
    let changes = std::mem::take(&mut work.dirty_tiles)
        .into_iter()
        .collect::<Vec<_>>();
    reconcile(
        &mut commands,
        &mirror.0,
        rebuild,
        &changes,
        &projected,
        &terrain,
        assets.as_deref(),
    );
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
        classify_client_local, force_capture_overlay_off, ingest_messages,
    };
    use crate::project::WorldProjected;

    #[test]
    fn capture_forces_the_frame_time_overlay_off() {
        let mut app = App::new();
        app.insert_resource(FpsOverlayConfig {
            enabled: true,
            ..Default::default()
        });

        force_capture_overlay_off(&mut app);

        assert!(!app.world().resource::<FpsOverlayConfig>().enabled);
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
            .add_systems(Update, ingest_messages);

        app.update();

        assert_eq!(
            app.world().resource::<ProjectionWork>().dirty_tiles,
            [[0, 0, 0], [1, 0, 0]].into_iter().collect()
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
