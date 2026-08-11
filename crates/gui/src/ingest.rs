use std::{
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
    app::{App, Startup, Update},
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::ButtonInput,
    prelude::{
        Camera3d, Commands, DefaultPlugins, KeyCode, Query, Res, ResMut, Resource, Transform,
    },
};
use client_core::Mirror;
use protocol::{Delta, Snapshot};

use crate::{
    camera::CameraRig,
    capture::{CaptureState, capture_after_frames},
    project::{
        ClientLocal, ProjectionAssets, TerrainTile, WorldProjected, reconcile,
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
    dirty: bool,
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
    reader
        .get_mut()
        .set_read_timeout(None)
        .context("could not clear snapshot timeout")?;
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
            dirty: false,
        })
        .add_systems(Startup, (setup_camera, setup_projection_assets))
        .add_systems(
            Update,
            (
                ingest_messages,
                reconcile_projection,
                camera_controls,
                toggle_overlay,
            ),
        );
    if let Some(capture) = args.capture {
        // Capture output must never contain the diagnostic overlay.
        app.insert_resource(CaptureState::new(capture, args.frames));
        app.add_systems(Update, capture_after_frames);
    }
    app.run();
    Ok(())
}

struct Args {
    port: u16,
    capture: Option<PathBuf>,
    frames: u32,
}

fn parse_args() -> anyhow::Result<Args> {
    let mut port = protocol::DEFAULT_PORT;
    let mut capture = None;
    let mut frames = None;
    let mut args = std::env::args_os().skip(1);
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
    Ok(Args {
        port,
        capture,
        frames: frames.unwrap_or(0),
    })
}

fn setup_camera(mut commands: Commands) {
    let rig = CameraRig::new([64, 64, 9]);
    commands.spawn((Camera3d::default(), rig.transform(), rig, ClientLocal));
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
) {
    loop {
        match receiver
            .0
            .lock()
            .expect("ingest receiver mutex poisoned")
            .try_recv()
        {
            Ok(Ok(WireMessage::Snapshot(snapshot))) => {
                if mirror.0.apply_snapshot(*snapshot).is_ok() {
                    work.snapshot = true;
                }
            }
            Ok(Ok(WireMessage::Delta(delta))) => {
                mirror.0.apply_delta(*delta);
                work.dirty = !mirror.0.changes().tiles.is_empty();
            }
            Ok(Err(error)) => eprintln!("server reader stopped: {error:#}"),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
        }
    }
}

fn reconcile_projection(
    mut commands: Commands,
    mirror: Res<MirrorResource>,
    mut work: ResMut<ProjectionWork>,
    projected: Query<(bevy::prelude::Entity, &WorldProjected)>,
    terrain: Query<(bevy::prelude::Entity, &TerrainTile)>,
    assets: Option<Res<ProjectionAssets>>,
) {
    let rebuild = std::mem::take(&mut work.snapshot);
    let dirty = std::mem::take(&mut work.dirty);
    let changes: &[[i32; 3]] = if dirty {
        &mirror.0.changes().tiles
    } else {
        &[]
    };
    reconcile(
        &mut commands,
        &mirror.0,
        rebuild,
        changes,
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
