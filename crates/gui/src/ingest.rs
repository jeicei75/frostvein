use std::{
    collections::BTreeSet,
    ffi::OsString,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        mpsc::{self, Receiver, SyncSender, TryRecvError},
    },
    thread,
    time::Duration,
};

use anyhow::{Context, bail};
use bevy::{
    app::PluginGroup,
    app::ScheduleRunnerPlugin,
    asset::{Assets, Handle},
    camera::RenderTarget,
    image::Image,
    render::render_resource::{TextureFormat, TextureUsages},
    window::{ExitCondition, WindowPlugin},
    winit::WinitPlugin,
};
use bevy::{
    app::{App, AppExit, PostUpdate, Startup, Update},
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::change_detection::DetectChanges,
    ecs::message::MessageWriter,
    ecs::schedule::IntoScheduleConfigs,
    input::{ButtonInput, mouse::MouseButton},
    pbr::{DistanceFog, FogFalloff},
    prelude::{
        AmbientLight, Camera3d, ClearColor, Color, Commands, Component, DefaultPlugins,
        DirectionalLight, GlobalZIndex, KeyCode, Node, PerspectiveProjection, PositionType,
        Projection, Query, Res, ResMut, Resource, Text, TextColor, TextFont, Time, Transform,
        TransformSystems, Vec2, Window, With, Without, px,
    },
    render::renderer::RenderAdapterInfo,
    window::PrimaryWindow,
};
use client_core::Mirror;
use protocol::{Delta, Dims, Snapshot};

use crate::{
    appearance::night_lighting,
    atmosphere::{fall_snow, setup_atmosphere, sun_light_transform},
    blend::TickClock,
    camera::{BOOT_VERTICAL_FOV, CameraRig},
    capture::{
        CaptureState, TreeCaptureVerification, accumulate_motion, capture_after_frames,
        update_tree_capture_verification,
    },
    command::send_commands,
    designate::{
        DesignateMode, DragAnchor, DragMode, designation_input, setup_designate_hint,
        update_designate_hint,
    },
    pick::{PickedTile, update_pick},
    project::{
        ClientLocal, DigChipQuery, DynamicProjectionQuery, ProjectedDesignation,
        ProjectedDesignationKind, ProjectedZone, ProjectionAssets, TerrainQuery,
        TerrainSubdivision, TerrainTile, TreeMeshQuery, WorldProjected, blend_entities,
        flicker_lights, has_terrain_above, reconcile, setup_projection_assets, sync_drag_preview,
        sync_hover_highlight,
    },
    slice::SliceLevel,
};

const SNAPSHOT_READ_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;
const MESSAGE_QUEUE: usize = 16;
const DEFAULT_AT_TICK_FRAME_BUDGET: u32 = 1_500;

/// The four independently inspectable contributors to the rendered valley.
///
/// This is deliberately a fixed seat-side instrument, not a lighting configuration surface:
/// F5--F8 are the complete public control and their state lasts only for this client run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightSource {
    Sun,
    Campfire,
    Lanterns,
    Ambient,
}

impl LightSource {
    const ALL: [Self; 4] = [Self::Sun, Self::Campfire, Self::Lanterns, Self::Ambient];

    fn key(self) -> KeyCode {
        match self {
            Self::Sun => KeyCode::F5,
            Self::Campfire => KeyCode::F6,
            Self::Lanterns => KeyCode::F7,
            Self::Ambient => KeyCode::F8,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Sun => "sun",
            Self::Campfire => "campfire",
            Self::Lanterns => "lanterns",
            Self::Ambient => "ambient",
        }
    }

    pub fn from_name(name: &str) -> anyhow::Result<Self> {
        match name {
            "sun" => Ok(Self::Sun),
            "campfire" => Ok(Self::Campfire),
            "lanterns" => Ok(Self::Lanterns),
            "ambient" => Ok(Self::Ambient),
            _ => {
                bail!("unknown light source {name:?}; expected sun, campfire, lanterns, or ambient")
            }
        }
    }
}

#[derive(Resource)]
pub struct LightingToggles {
    sun: bool,
    campfire: bool,
    lanterns: bool,
    ambient: bool,
}

impl Default for LightingToggles {
    fn default() -> Self {
        Self {
            sun: true,
            campfire: true,
            lanterns: true,
            ambient: true,
        }
    }
}

impl LightingToggles {
    fn enabled(&self, source: LightSource) -> bool {
        match source {
            LightSource::Sun => self.sun,
            LightSource::Campfire => self.campfire,
            LightSource::Lanterns => self.lanterns,
            LightSource::Ambient => self.ambient,
        }
    }

    fn toggle(&mut self, source: LightSource) {
        match source {
            LightSource::Sun => self.sun = !self.sun,
            LightSource::Campfire => self.campfire = !self.campfire,
            LightSource::Lanterns => self.lanterns = !self.lanterns,
            LightSource::Ambient => self.ambient = !self.ambient,
        }
    }
}

pub enum WireMessage {
    Snapshot(Box<Snapshot>),
    Delta(Box<Delta>),
}

#[derive(Resource)]
pub struct MirrorResource(pub Mirror);

#[derive(Resource)]
pub struct IngestReceiver(Mutex<Receiver<anyhow::Result<WireMessage>>>);

impl IngestReceiver {
    pub fn new(receiver: Receiver<anyhow::Result<WireMessage>>) -> Self {
        Self(Mutex::new(receiver))
    }
}

#[derive(Resource, Default)]
pub struct ProjectionWork {
    pub snapshot: bool,
    pub dirty_tiles: BTreeSet<[i32; 3]>,
}

/// The four pines, compiled INTO the binary and served from the `embedded://` asset source.
///
/// WHY EMBEDDED RATHER THAN SHIPPED BESIDE THE EXECUTABLE. `build.rs` stamps the commit SHA into
/// this binary because "every previous guard was a procedure, and a procedure is exactly what a
/// stale binary defeats". "Remember to copy `assets/` next to `gui.exe`" is that same shape of
/// procedure, and it failed the first time the vehicle used it: the fallback it lands on is a
/// path stamped at COMPILE time on the build machine, so on Windows it resolves to a Linux path
/// that cannot exist. Embedding removes the copy step instead of documenting it — the assets
/// cannot be left behind, and they cannot go stale against the binary that draws them.
///
/// ORDER IS LOAD-BEARING: these are indexed by `TreeVariant` in `project.rs::tree_scene`, so the
/// table and that match arm are one mapping in two places. `tree_asset_paths_match_the_loader`
/// asserts they agree rather than trusting them to.
pub const TREE_ASSETS: [(&str, &[u8]); 4] = [
    (
        "trees/SM_VoxelPine_Tree01.glb",
        include_bytes!("../../../assets/trees/SM_VoxelPine_Tree01.glb"),
    ),
    (
        "trees/SM_VoxelPine_Tree02.glb",
        include_bytes!("../../../assets/trees/SM_VoxelPine_Tree02.glb"),
    ),
    (
        "trees/SM_VoxelPine_Tree03.glb",
        include_bytes!("../../../assets/trees/SM_VoxelPine_Tree03.glb"),
    ),
    (
        "trees/SM_VoxelPine_Tree04R.glb",
        include_bytes!("../../../assets/trees/SM_VoxelPine_Tree04R.glb"),
    ),
];

/// How many embedded pines actually carry glTF bytes, and how many bytes in total.
///
/// A blob is counted only if it is non-empty AND opens with the binary-glTF magic, so an emptied
/// or truncated `include_bytes!` is visible on the first line of output rather than at the moment
/// a scene silently fails to load.
pub fn tree_asset_summary() -> (usize, usize) {
    let embedded = TREE_ASSETS
        .iter()
        .filter(|(_, bytes)| bytes.starts_with(b"glTF"))
        .count();
    (embedded, TREE_ASSETS.iter().map(|(_, b)| b.len()).sum())
}

/// Publish the embedded pines into the `embedded://` source before anything loads them.
///
/// `AssetPlugin::build` creates the registry and registers the source, so this must run AFTER
/// `DefaultPlugins` and before the startup system that loads the scenes.
fn register_tree_assets(app: &mut App) {
    let registry = app
        .world_mut()
        .resource_mut::<bevy::asset::io::embedded::EmbeddedAssetRegistry>();
    for (path, bytes) in TREE_ASSETS {
        registry.insert_asset(PathBuf::new(), Path::new(path), bytes);
    }
}

pub fn run() -> anyhow::Result<()> {
    let args = parse_args()?;
    // M2-7. FIRST line out, before the connect can fail: a session that cannot reach the daemon
    // still learns which binary it is holding, and that is exactly the case where the answer
    // usually turns out to be "a stale one". See `crate::BUILD_SHA`.
    eprintln!("gui build {}", crate::BUILD_SHA);
    // This line inspects the BLOBS, not `TREE_ASSETS.len()`. Printing the array length reported
    // "4 embedded" identically with every blob emptied, with the registration deleted, or with
    // `embedded://` resolution broken -- a confirmation step that could not observe the failure
    // the runbook asks it to confirm. Decoding is still Bevy's; this only proves bytes are here.
    let (embedded, bytes) = tree_asset_summary();
    eprintln!(
        "gui tree assets: {embedded} of {} embedded in this binary, {bytes} bytes",
        TREE_ASSETS.len()
    );
    let (mirror, receiver, writer) = connect_to_daemon(args.port)?;
    let mut app = App::new();
    if args.headless {
        // No window, and therefore no winit: WinitPlugin panics outright where there is no display
        // server, which is every devpod this project builds on. ScheduleRunnerPlugin drives the
        // loop instead. The renderer is untouched and still real — see `HeadlessTarget`.
        eprintln!(
            "gui running HEADLESS: offscreen {}x{} target, no window",
            HEADLESS_SIZE.0, HEADLESS_SIZE.1
        );
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..Default::default()
                })
                .disable::<WinitPlugin>(),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::ZERO))
        // The overlay plugin wants a window; its config resource is all the client systems read.
        .init_resource::<FpsOverlayConfig>();
    } else {
        app.add_plugins(DefaultPlugins)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(FpsOverlayPlugin {
                config: overlay_config_off(),
            });
    }
    register_tree_assets(&mut app);
    configure_client_app(&mut app, mirror, receiver, writer, args);
    // `App::run()` RETURNS the exit status and `AppExit` is not `#[must_use]`, so discarding it
    // compiles clean under `-D warnings` and silently turns every capture failure into exit 0.
    // AC16 requires a run that never reaches its tick to exit NON-ZERO; the flag was already set
    // (`capture.rs`, `AppExit::error()`) and only the consumer was missing.
    if let AppExit::Error(code) = app.run() {
        std::process::exit(code.get().into());
    }
    Ok(())
}

/// Opens the daemon socket, reads the opening snapshot, and leaves a reader thread feeding the
/// returned channel. The only part of `run()` that is I/O and therefore the only part a test
/// cannot reach.
fn connect_to_daemon(
    port: u16,
) -> anyhow::Result<(Mirror, Receiver<anyhow::Result<WireMessage>>, TcpStream)> {
    let address = format!("127.0.0.1:{port}");
    let stream = TcpStream::connect(("127.0.0.1", port))
        .with_context(|| format!("could not connect to {address}"))?;
    stream
        .set_read_timeout(Some(SNAPSHOT_READ_TIMEOUT))
        .context("could not set snapshot read timeout")?;
    let writer = stream
        .try_clone()
        .context("could not clone command writer")?;
    writer
        .set_write_timeout(Some(SNAPSHOT_READ_TIMEOUT))
        .context("could not set command write timeout")?;
    let mut reader = BufReader::new(stream);
    let mirror = Mirror::from_snapshot(read_snapshot(&mut reader)?)
        .context("could not build client mirror")?;
    let (sender, receiver) = mpsc::sync_channel(MESSAGE_QUEUE);
    thread::Builder::new()
        .name("server-read".to_string())
        .spawn(move || read_messages(reader, sender))
        .context("could not spawn server reader thread")?;
    Ok((mirror, receiver, writer))
}

/// Everything `run()` does to the App once its plugins are in: the world resources, the parsed
/// flags, the two registration points, and the capture branch.
///
/// It is a separate function because a CALL is a seam too. `insert_capture_resources` was
/// extracted at this story's round 1 for exactly this reason and tested directly — and the
/// review then showed that deleting the *call to it* from `run()` still left the whole suite
/// green, so `--cursor` and 7.2's `--distance` would both parse, validate and vanish. The
/// defect had moved one level out, not closed. `run()` needs a socket and a window and can
/// never be entered by a test; this can, so the wiring below is executable rather than merely
/// readable. What remains uncovered is the three lines of `run()` itself.
fn configure_client_app(
    app: &mut App,
    mirror: Mirror,
    receiver: Receiver<anyhow::Result<WireMessage>>,
    writer: TcpStream,
    args: Args,
) {
    let slice = initial_slice(mirror.dims(), args.slice_level);
    let capture_start_tick = mirror.tick();
    app.insert_resource(MirrorResource(mirror))
        .insert_resource(slice)
        .insert_resource(IngestReceiver::new(receiver))
        .insert_resource(crate::command::CommandSink(Mutex::new(writer)))
        .insert_resource(ProjectionWork {
            snapshot: true,
            dirty_tiles: BTreeSet::new(),
        })
        .insert_resource(ClearColor(night_lighting().sky));
    if args.headless {
        app.insert_resource(HeadlessRequested);
    }
    if let Some(subdiv) = args.subdiv {
        app.insert_resource(TerrainSubdivision(subdiv));
    }
    insert_capture_resources(app, &args);
    // NOT gated on `headless`: `expected_cut_face` adds the tree meshes unconditionally, so
    // without this resource the actual side never gains them and a WINDOWED capture asserts
    // 0 == 265 and panics before the screenshot. That is the exact command the vehicle sitting
    // card runs (`gui.exe <port> --capture <png> --frames N`, no `--headless`).
    if args.capture.is_some() {
        app.insert_resource(TreeCaptureVerification::default());
    }
    client_systems(app);
    projection_systems(app);
    if let Some(capture) = args.capture {
        // Capture output must never contain the diagnostic overlay.
        force_capture_overlay_off(app);
        let capture = match args.at_tick {
            Some(ticks_after_start) => CaptureState::at_tick(
                capture,
                args.frames,
                capture_start_tick,
                ticks_after_start,
                args.expect_work,
            ),
            None => CaptureState::new(capture, args.frames, args.expect_work),
        };
        app.insert_resource(capture);
        capture_systems(app);
    }
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
    // The readout names the cell under the pointer, so this set now depends on the pick. Defaulted
    // here rather than made `Option` in the system: a resource that is genuinely missing in
    // production should fail loudly, not quietly render "cursor -" forever.
    app.init_resource::<PickedTile>();
    app.init_resource::<crate::project::TreeReportState>();
    app.add_systems(Update, crate::project::report_tree_meshes_once);
    app.init_resource::<TickClock>()
        .add_systems(Startup, (setup_slice_readout, setup_lighting_readout))
        .add_systems(
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
        )
        // AC9's whole mechanism. It lived in `run()` only, where no test could reach it, and
        // deleting both systems left the suite green — 6.1's untested-drive-line defect on the
        // half of the story the readout exists for. It must read the level AFTER the keyboard has
        // written it, or the displayed level trails the cut by one frame.
        .add_systems(Update, update_slice_readout.after(ProjectionSet));
    // The toggles resource is initialised HERE, beside the systems that READ it, not only in
    // `client_systems`. Registering a system in one app-builder while its resource is created in
    // another is the same defect this function's own doc comment describes: `crates/gui/tests/
    // capture.rs` calls `projection_systems` alone, so both capture instruments panicked with
    // "Resource does not exist" the moment a lighting system read a resource nothing had created.
    // `init_resource` is idempotent, so `client_systems` keeping its own call is not a conflict —
    // each builder now stands up what it registers.
    app.init_resource::<LightingToggles>();
    app.add_systems(
        Update,
        (apply_lighting_toggles, update_lighting_readout)
            .chain()
            .after(ProjectionSet),
    );
}

/// Everything `run()` registers besides the projection chain: the startup scene, the
/// client-local classification pass, and the per-frame input and atmosphere systems.
///
/// Extracted for exactly the reason `projection_systems` was, and it is the same defect one
/// level further out. While these tuples lived inline in `run()` no test could reach them, so
/// dropping any system from either left the whole suite green. That is not hypothetical: story
/// 7.2's review found `--distance` parsed, validated, and then never reaching the camera rig,
/// with its only test NAMED for reaching the camera setup; 7.1's review found the entire
/// on-screen readout and the `--z` pin deletable the same way; 6.1 lost both projection systems
/// with 54 of 54 tests green. Five of eight Milestone 2 stories carried an instance of this
/// class, and the Milestone 2 retrospective ruled it closed at the root rather than caught a
/// sixth time.
pub fn client_systems(app: &mut App) {
    app.init_resource::<PickedTile>()
        .init_resource::<crate::project::DragPreviewCells>()
        .init_resource::<crate::command::PendingCommands>()
        .init_resource::<ButtonInput<bevy::input::mouse::MouseButton>>()
        .init_resource::<DesignateMode>()
        .init_resource::<DragMode>()
        .init_resource::<DragAnchor>()
        .init_resource::<LightingToggles>();
    app.add_systems(
        Startup,
        (
            setup_camera,
            setup_night_lighting,
            setup_projection_assets,
            setup_atmosphere,
            setup_designate_hint,
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
            light_controls,
            update_fog_from_camera,
            toggle_overlay,
            fall_snow,
        ),
    )
    .add_systems(
        PostUpdate,
        (
            apply_scripted_input.after(TransformSystems::Propagate),
            update_pick.after(apply_scripted_input),
            sync_hover_highlight.after(update_pick),
            designation_input.after(update_pick),
            sync_drag_preview.after(designation_input),
            send_commands.after(designation_input),
            update_designate_hint.after(designation_input),
        ),
    );
}

/// The capture instrument's registration, including the ordering edge that keeps it reading the
/// frame the projection chain just wrote. Bevy's ambiguity detection defaults to
/// `LogLevel::Ignore`, so an unordered read here would be resolved silently and sample the frame
/// at an undefined point — raised by three separate review layers at 6.1.
pub fn capture_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_tree_capture_verification,
            accumulate_motion,
            capture_after_frames,
        )
            .chain()
            .after(ProjectionSet),
    );
}

/// The `--z` pin has to REACH the resource, not merely parse. Inside `run()` it was unreachable
/// from any test, and a mutation that ignored the flag entirely stayed green.
pub fn initial_slice(dims: Dims, requested: Option<i32>) -> SliceLevel {
    requested.map_or_else(
        || SliceLevel::at_world_top(dims),
        |level| SliceLevel::pinned(dims, level),
    )
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
    distance: Option<f32>,
    cursor: Option<Vec2>,
    at_tick: Option<u64>,
    drag: Option<ScriptedDragSpec>,
    headless: bool,
    subdiv: Option<u32>,
}

/// Present only under `--headless`: the offscreen texture the camera draws into, and which the
/// capture screenshots instead of a window.
///
/// WHY THIS EXISTS. The premise that this project's devpods "cannot render" was measured on
/// 2026-08-11 and generalised too far: they cannot open a WINDOW (no display server, and winit
/// panics with "neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set"), but a CPU Vulkan
/// device IS present — Mesa's lavapipe, `llvmpipe (LLVM 19.1.7)`, Vulkan 1.4 — and wgpu creates a
/// device on it. Rendering needs a device, not a window. Verified before this was written.
///
/// THE CAVEAT THAT MATTERS, and it is not optional when reading any number this produces:
/// llvmpipe is a SOFTWARE rasteriser, so its pixels are not guaranteed identical to the vehicle's
/// GPU. A figure taken here may NOT be compared against one calibrated on the vehicle — story
/// 9.1's 0.6651 % blown-pool ceiling was measured on GPU-rendered committed PNGs and is not a
/// bar this path can be judged against. Compare llvmpipe against llvmpipe: render the baseline
/// commit and the candidate the same way and read the DELTA. That is the same Bevy-against-Bevy
/// discipline 9.1 used to calibrate in the first place.
///
/// FPS IS NOT MEASURABLE HERE AT ALL. NFR6 stays vehicle-bound; a software rasteriser's frame
/// time says nothing about the machine that will run this.
#[derive(Resource, Clone)]
pub struct HeadlessTarget(pub Handle<Image>);

/// Set by `--headless` before startup, so `setup_camera` knows to build an offscreen target.
#[derive(Resource)]
pub struct HeadlessRequested;

/// The capture resolution, matched to the vehicle's committed PNGs (`boot7.png` and every other
/// signoff frame are 1280x720) so a headless frame and a vehicle frame are the same shape and the
/// pixel-region instruments read the same way on both.
const HEADLESS_SIZE: (u32, u32) = (1280, 720);

#[derive(Resource)]
pub struct CaptureDistance(pub f32);

/// A capture-only viewport position written before the camera pick runs.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ScriptedCursor(pub Vec2);

#[derive(Debug, Clone, Copy, PartialEq)]
struct ScriptedDragSpec {
    mode: DesignateMode,
    start: Vec2,
    end: Vec2,
}

/// A capture-only press-drag-release sequence. It writes the same input resources a window does;
/// it never constructs a rectangle, so the human and scripted paths share the mode machine.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ScriptedDrag {
    spec: ScriptedDragSpec,
    stage: ScriptedDragStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptedDragStage {
    Press,
    Hold,
    Release,
    Done,
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
    let mut distance = None;
    let mut cursor = None;
    let mut at_tick = None;
    let mut drag = None;
    let mut headless = false;
    let mut subdiv = None;
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
        } else if arg == "--headless" {
            headless = true;
        } else if arg == "--subdiv" {
            let value = args
                .next()
                .context("--subdiv requires a positive integer")?;
            let parsed: u32 = value
                .to_string_lossy()
                .parse()
                .context("invalid --subdiv")?;
            if parsed == 0 {
                bail!("--subdiv must be positive");
            }
            // The parser rejected only 0, so `--subdiv 3000000000` passed validation and then
            // panicked inside the mesher on `i32::try_from`. The ceiling is the render-side
            // twin of the bench's face limit; see `project::MAX_SUBDIV`.
            if parsed > crate::project::MAX_SUBDIV {
                bail!(
                    "--subdiv must be at most {}, got {parsed}",
                    crate::project::MAX_SUBDIV
                );
            }
            subdiv = Some(parsed);
        } else if arg == "--z" {
            let value = args.next().context("--z requires a level")?;
            slice_level = Some(
                value
                    .to_string_lossy()
                    .parse()
                    .context("invalid --z level")?,
            );
        } else if arg == "--distance" {
            let value = args.next().context("--distance requires a value")?;
            let parsed: f32 = value
                .to_string_lossy()
                .parse()
                .context("invalid --distance")?;
            if !parsed.is_finite() {
                bail!("--distance must be finite");
            }
            distance = Some(parsed);
        } else if arg == "--cursor" {
            let value = args.next().context("--cursor requires x,y")?;
            cursor = Some(parse_cursor(value)?);
        } else if arg == "--at-tick" {
            let value = args.next().context("--at-tick requires a tick count")?;
            at_tick = Some(
                value
                    .to_string_lossy()
                    .parse()
                    .context("invalid --at-tick count")?,
            );
        } else if arg == "--drag" {
            let value = args.next().context("--drag requires mode,x0,y0,x1,y1")?;
            drag = Some(parse_drag(value)?);
        } else {
            port = arg.to_string_lossy().parse().context("invalid port")?;
        }
    }
    if capture.is_some() && frames.is_none() && at_tick.is_none() {
        bail!("--capture requires --frames N or --at-tick N");
    }
    if capture.is_some() && frames == Some(0) {
        bail!("--capture --frames must be positive");
    }
    if expect_work && capture.is_none() {
        bail!("--expect-work requires --capture");
    }
    if distance.is_some() && capture.is_none() {
        bail!("--distance requires --capture");
    }
    if cursor.is_some() && capture.is_none() {
        bail!("--cursor requires --capture");
    }
    if at_tick.is_some() && capture.is_none() {
        bail!("--at-tick requires --capture");
    }
    if drag.is_some() && capture.is_none() {
        bail!("--drag requires --capture");
    }
    if drag.is_some() && cursor.is_some() {
        // `apply_scripted_input` takes the drag branch OR the cursor branch, never both, so a
        // `--cursor` passed alongside `--drag` is parsed, validated, inserted and then never
        // written to the window — while `capture_after_frames` still asserts the live pick
        // against it. That combination cannot succeed; every other bad pairing here bails, and
        // silently ignoring one of two flags the operator typed is the trap this parser exists
        // to close.
        bail!("--cursor and --drag are mutually exclusive; a scripted drag moves the cursor");
    }
    Ok(Args {
        port,
        capture,
        frames: frames.unwrap_or(DEFAULT_AT_TICK_FRAME_BUDGET),
        expect_work,
        slice_level,
        distance,
        cursor,
        at_tick,
        drag,
        headless,
        subdiv,
    })
}

fn parse_cursor(value: OsString) -> anyhow::Result<Vec2> {
    let value = value.to_string_lossy();
    let Some((x, y)) = value.split_once(',') else {
        bail!("invalid --cursor; expected x,y");
    };
    if y.contains(',') {
        bail!("invalid --cursor; expected x,y");
    }
    let x: f32 = x.parse().context("invalid --cursor x")?;
    let y: f32 = y.parse().context("invalid --cursor y")?;
    if !x.is_finite() || !y.is_finite() {
        bail!("invalid --cursor; coordinates must be finite");
    }
    Ok(Vec2::new(x, y))
}

fn parse_drag(value: OsString) -> anyhow::Result<ScriptedDragSpec> {
    let value = value.to_string_lossy();
    let parts = value.split(',').collect::<Vec<_>>();
    let [mode, x0, y0, x1, y1] = parts.as_slice() else {
        bail!("invalid --drag; expected mode,x0,y0,x1,y1");
    };
    let mode = match *mode {
        "dig" => DesignateMode::Dig,
        "channel" => DesignateMode::Channel,
        "stockpile" => DesignateMode::Stockpile,
        "clear" => DesignateMode::Clear,
        _ => bail!("invalid --drag mode; expected dig,channel,stockpile,clear"),
    };
    let parse_coordinate = |name: &str, value: &str| -> anyhow::Result<f32> {
        let value: f32 = value
            .parse()
            .with_context(|| format!("invalid --drag {name}"))?;
        if !value.is_finite() {
            bail!("invalid --drag {name}; coordinates must be finite");
        }
        Ok(value)
    };
    Ok(ScriptedDragSpec {
        mode,
        start: Vec2::new(parse_coordinate("x0", x0)?, parse_coordinate("y0", y0)?),
        end: Vec2::new(parse_coordinate("x1", x1)?, parse_coordinate("y1", y1)?),
    })
}

fn apply_scripted_input(
    cursor: Option<Res<ScriptedCursor>>,
    drag: Option<ResMut<ScriptedDrag>>,
    picked: Res<PickedTile>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    if let Some(mut drag) = drag {
        // The previous frame's pick answers "is the pick machinery live yet?" — camera, primary
        // window and viewport all resolved. On a cold first frame it is `None`, and pressing then
        // anchors on nothing: the drag is lost and NOTHING reports it. 8.1's `--cursor` rewrote
        // the cursor every frame and self-healed; three unconditional shots at the coldest moment
        // in the app's life do not. So the press and the release WAIT rather than fire blind.
        let pick_is_live = picked.tile().is_some();
        match drag.stage {
            ScriptedDragStage::Press => {
                window.set_cursor_position(Some(drag.spec.start));
                if pick_is_live {
                    keys.press(mode_key(drag.spec.mode));
                    mouse.press(MouseButton::Left);
                    drag.stage = ScriptedDragStage::Hold;
                }
            }
            ScriptedDragStage::Hold => {
                window.set_cursor_position(Some(drag.spec.end));
                keys.release(mode_key(drag.spec.mode));
                keys.clear();
                mouse.clear();
                drag.stage = ScriptedDragStage::Release;
            }
            ScriptedDragStage::Release => {
                window.set_cursor_position(Some(drag.spec.end));
                if pick_is_live {
                    mouse.release(MouseButton::Left);
                    drag.stage = ScriptedDragStage::Done;
                }
            }
            ScriptedDragStage::Done => {}
        }
    } else if let Some(cursor) = cursor {
        window.set_cursor_position(Some(cursor.0));
    }
}

impl ScriptedDrag {
    pub fn mode(&self) -> DesignateMode {
        self.spec.mode
    }

    /// Whether the scripted drag ran to completion. A drag still mid-stage at capture time never
    /// released, so the capture it is about to authorise shows no designation it created.
    pub fn completed(&self) -> bool {
        matches!(self.stage, ScriptedDragStage::Done)
    }
}

fn mode_key(mode: DesignateMode) -> KeyCode {
    match mode {
        DesignateMode::Dig => KeyCode::Digit1,
        DesignateMode::Channel => KeyCode::Digit2,
        DesignateMode::Stockpile => KeyCode::Digit3,
        DesignateMode::Clear => KeyCode::Digit4,
        DesignateMode::None => unreachable!("a scripted drag always names an active mode"),
    }
}

/// Writes the capture-only flags onto the app.
///
/// Extracted from `run()` so the seam from a parsed `Args` to the live resources is reachable by a
/// test: `run()` itself needs a socket and a window, so nothing could execute this wiring. Story
/// 8.1's mutation row proved the gap real -- replacing the cursor insert with `let _ = cursor;`
/// left the whole suite green, because the only test wrote `ScriptedCursor` by hand. That is the
/// same lie `--distance` told at 7.2: parsed, validated, and then silently dropped.
fn insert_capture_resources(app: &mut App, args: &Args) {
    if let Some(distance) = args.distance {
        app.insert_resource(CaptureDistance(distance));
    }
    if let Some(cursor) = args.cursor {
        app.insert_resource(ScriptedCursor(cursor));
    }
    if let Some(spec) = args.drag {
        app.insert_resource(ScriptedDrag {
            spec,
            stage: ScriptedDragStage::Press,
        });
    }
}

fn setup_camera(
    mut commands: Commands,
    distance: Option<Res<CaptureDistance>>,
    headless: Option<Res<HeadlessRequested>>,
    images: Option<ResMut<Assets<Image>>>,
) {
    // Built BEFORE the spawn so the camera can be pointed at it in the same system, and so a
    // headless run that cannot allocate the texture fails here rather than rendering to nowhere.
    let headless_target = match (headless.is_some(), images) {
        (true, Some(mut images)) => {
            let mut image = Image::new_target_texture(
                HEADLESS_SIZE.0,
                HEADLESS_SIZE.1,
                TextureFormat::Rgba8UnormSrgb,
                None,
            );
            // COPY_SRC is what makes the frame readable back to the CPU; without it the
            // screenshot silently has nothing to copy.
            image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
            Some(images.add(image))
        }
        _ => None,
    };
    let mut rig = CameraRig::new([64, 64, 9]);
    if let Some(distance) = distance {
        rig.distance = distance.0.clamp(4.0, 500.0);
    }
    let (fog_start, fog_end) = fog_falloff(rig.distance);
    let camera = commands
        .spawn((
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
        ))
        .id();
    if let Some(handle) = headless_target {
        // In Bevy 0.19 the render target is its own COMPONENT, not a field on Camera.
        commands
            .entity(camera)
            .insert(RenderTarget::Image(handle.clone().into()));
        commands.insert_resource(HeadlessTarget(handle));
    }
}

fn setup_night_lighting(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            color: night_lighting().directional,
            illuminance: night_lighting().directional_illuminance,
            shadow_maps_enabled: true,
            ..Default::default()
        },
        sun_light_transform(),
        SunLight,
        ClientLocal,
    ));
}

#[derive(Component)]
struct SunLight;

#[derive(Component)]
pub struct SliceReadout;

#[derive(Component)]
pub struct LightingReadout;

fn lighting_readout(toggles: &LightingToggles) -> String {
    LightSource::ALL
        .into_iter()
        .map(|source| {
            format!(
                "{} {} {}",
                match source.key() {
                    KeyCode::F5 => "F5",
                    KeyCode::F6 => "F6",
                    KeyCode::F7 => "F7",
                    KeyCode::F8 => "F8",
                    _ => unreachable!("the fixed lighting keys are F5 through F8"),
                },
                source.name(),
                if toggles.enabled(source) { "on" } else { "off" }
            )
        })
        .collect::<Vec<_>>()
        .join("  ")
}

fn setup_lighting_readout(mut commands: Commands, toggles: Res<LightingToggles>) {
    commands.spawn((
        Text::new(lighting_readout(&toggles)),
        TextFont::from_font_size(22.0),
        TextColor(Color::srgb(0.86, 0.91, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: px(72),
            left: px(16),
            ..Default::default()
        },
        GlobalZIndex(i32::MAX - 16),
        LightingReadout,
        ClientLocal,
    ));
}

fn update_lighting_readout(
    toggles: Res<LightingToggles>,
    mut readout: Query<&mut Text, With<LightingReadout>>,
) {
    if !toggles.is_changed() {
        return;
    }
    let text = lighting_readout(&toggles);
    for mut readout in &mut readout {
        *readout = Text::new(text.clone());
    }
}

fn light_controls(keys: Res<ButtonInput<KeyCode>>, mut toggles: ResMut<LightingToggles>) {
    for source in LightSource::ALL {
        if keys.just_pressed(source.key()) {
            toggles.toggle(source);
        }
    }
}

fn apply_lighting_toggles(
    toggles: Res<LightingToggles>,
    mut ambient: Query<&mut AmbientLight, With<Camera3d>>,
    mut sun: Query<&mut DirectionalLight, With<SunLight>>,
    mut points: Query<(
        &crate::project::ProjectedLight,
        &mut bevy::prelude::PointLight,
    )>,
) {
    for mut light in &mut ambient {
        light.brightness = if toggles.enabled(LightSource::Ambient) {
            night_lighting().ambient_brightness
        } else {
            0.0
        };
    }
    for mut light in &mut sun {
        light.illuminance = if toggles.enabled(LightSource::Sun) {
            night_lighting().directional_illuminance
        } else {
            0.0
        };
    }
    for (kind, mut light) in &mut points {
        let enabled = match kind.0 {
            protocol::LightKind::Campfire => toggles.enabled(LightSource::Campfire),
            protocol::LightKind::Lantern => toggles.enabled(LightSource::Lanterns),
            protocol::LightKind::Torch => true,
        };
        if !enabled {
            light.intensity = 0.0;
        }
    }
}

fn setup_slice_readout(
    mut commands: Commands,
    slice: Res<SliceLevel>,
    mirror: Res<MirrorResource>,
) {
    let covered = has_terrain_above(&mirror.0, slice.level());
    commands.spawn((
        Text::new(slice.readout(covered, None)),
        TextFont::from_font_size(22.0),
        TextColor(Color::srgb(0.86, 0.91, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            // Below the F3 overlay, which Bevy pins to the origin at font size 32. The two must be
            // readable together: AC14's fps reading is taken AT a slice level.
            top: px(44),
            left: px(16),
            ..Default::default()
        },
        // The overlay claims `i32::MAX - 32`. Without an explicit index this node defaults to 0
        // and is drawn underneath it, covering the level number itself.
        GlobalZIndex(i32::MAX - 16),
        SliceReadout,
        ClientLocal,
    ));
}

fn update_slice_readout(
    slice: Res<SliceLevel>,
    mirror: Res<MirrorResource>,
    picked: Res<PickedTile>,
    cameras: Query<&CameraRig>,
    mut covered: bevy::prelude::Local<Option<bool>>,
    mut readout: Query<&mut Text, With<SliceReadout>>,
) {
    // `has_terrain_above` walks the world, so it must not run every frame. Its two inputs are
    // change-detected and cached; the cursor half changes far more often and costs nothing, so
    // the two are tracked separately rather than making the world walk follow the pointer.
    if slice.is_changed() || mirror.is_changed() || covered.is_none() {
        *covered = Some(has_terrain_above(&mirror.0, slice.level()));
    }
    // The camera is not change-detected here: it orbits continuously while `A`/`D` are held, and
    // a compass that only refreshes when the slice or the pick changes would sit on a stale
    // bearing for exactly as long as the camera is moving — which is when it is read.
    let north = cameras
        .single()
        .map_or("?", |rig| crate::camera::north_on_screen(rig));
    let text = format!(
        "{}  N {north}",
        slice.readout(covered.unwrap_or(false), picked.tile())
    );
    for mut readout in &mut readout {
        *readout = Text::new(text.clone());
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
    projected: DynamicProjectionQuery,
    designations: Query<(
        bevy::prelude::Entity,
        &ProjectedDesignation,
        &ProjectedDesignationKind,
    )>,
    zones: Query<(bevy::prelude::Entity, &ProjectedZone)>,
    terrain: TerrainQuery,
    trees: TreeMeshQuery,
    chips: DigChipQuery,
    assets: Option<Res<ProjectionAssets>>,
    mut meshes: Option<ResMut<Assets<bevy::prelude::Mesh>>>,
    subdiv: Option<Res<TerrainSubdivision>>,
) {
    let rebuild = std::mem::take(&mut work.snapshot);
    let changes = std::mem::take(&mut work.dirty_tiles)
        .into_iter()
        .collect::<Vec<_>>();
    // A chunk mesh is a whole surface, not a set of mutable per-cell entities, so a terrain delta
    // cannot be edited in place the way a cube entity can -- but it does not need the WORLD
    // rebuilt either. `reconcile` rebuilds only the chunks the changed cells can reach. This line
    // used to promote every delta to a full rebuild, which cost a whole mesh build per dug tile
    // and, because a dwarf digs continuously, froze every other dwarf for the length of the job.
    reconcile(
        &mut commands,
        &mirror.0,
        *slice,
        rebuild,
        &changes,
        &projected,
        &designations,
        &zones,
        &terrain,
        &trees,
        &chips,
        assets.as_deref(),
        meshes.as_deref_mut(),
        subdiv.as_deref(),
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
    use std::{
        io::{BufRead, BufReader},
        sync::{Mutex, mpsc},
        time::Duration,
    };

    use bevy::{
        app::{App, Update},
        camera::{CameraProjection, RenderTargetInfo},
        dev_tools::fps_overlay::FpsOverlayConfig,
        input::{ButtonInput, mouse::MouseButton},
        prelude::{Camera, Camera3d, GlobalTransform, KeyCode, UVec2, Window, With},
        window::{PrimaryWindow, WindowResolution},
    };
    use client_core::Mirror;
    use protocol::{Delta, Dims, MessageType, Snapshot, Speed, Tile, TileChange};

    use super::{
        ClientLocal, IngestReceiver, MirrorResource, ProjectionWork, WireMessage,
        classify_client_local, fog_falloff, fog_fraction, force_capture_overlay_off,
        ingest_messages,
    };
    use crate::blend::TickClock;
    use crate::camera::CameraRig;
    use crate::project::{SnowCap, TerrainChunk, TerrainSubdivision, TerrainTile, WorldProjected};
    use bevy::ecs::system::RunSystemOnce;

    /// Every pine must actually be INSIDE the binary, and be a real GLB.
    ///
    /// A length check alone would pass on four empty files, which is the shape of this project's
    /// recorded silent failures: the previous filesystem loader failed on the vehicle with a
    /// green suite behind it. The glTF magic is the independent oracle — it comes from the file
    /// content, not from anything this module asserts about itself.
    #[test]
    fn every_tree_variant_is_embedded_in_the_binary_as_a_real_glb() {
        assert_eq!(
            super::TREE_ASSETS.len(),
            4,
            "one embedded pine per TreeVariant"
        );
        for (path, bytes) in super::TREE_ASSETS {
            assert!(
                bytes.len() > 100_000,
                "{path} is {} bytes — too small to be a shipped pine",
                bytes.len()
            );
            assert_eq!(
                &bytes[0..4],
                b"glTF",
                "{path} does not carry the glTF magic, so it is not a GLB"
            );
        }
    }

    /// The embedded table and the loader's paths are ONE mapping written in two places.
    ///
    /// `project.rs::tree_scene` indexes `ProjectionAssets::trees` by `TreeVariant`, and that array
    /// is built from these paths in order. If the two ever disagree, every tree draws as the wrong
    /// species and nothing else goes red.
    #[test]
    fn tree_asset_paths_match_the_loader() {
        let embedded = super::TREE_ASSETS.map(|(path, _)| path);
        assert_eq!(
            embedded,
            crate::project::TREE_SCENE_PATHS,
            "the embedded table and the loader disagree about which pine is which"
        );
    }

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

    /// Builds the client app through the SAME function `run()` calls, on a real parsed `Args`.
    ///
    /// `--frames 60` on purpose: `capture_after_frames` fires on the frame its count reaches, and
    /// this harness renders nothing for it to assert about.
    fn configured_app(
        args: &[&str],
    ) -> (
        App,
        mpsc::SyncSender<anyhow::Result<WireMessage>>,
        std::net::TcpStream,
    ) {
        configured_app_with_snapshot(
            args,
            Snapshot {
                msg_type: MessageType::Snapshot,
                dims: Dims { x: 2, y: 1, z: 1 },
                tiles: vec![Tile::Solid(protocol::Material::Stone), Tile::Empty],
                entities: Vec::new(),
                designations: Vec::new(),
                zones: Vec::new(),
                items: Vec::new(),
                speed: Speed::Normal,
                tick: 0,
            },
        )
    }

    fn configured_app_with_snapshot(
        args: &[&str],
        snapshot: Snapshot,
    ) -> (
        App,
        mpsc::SyncSender<anyhow::Result<WireMessage>>,
        std::net::TcpStream,
    ) {
        let parsed = super::parse_args_from(
            args.iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>(),
        )
        .expect("the arguments under test must parse");
        let mirror = Mirror::from_snapshot(snapshot).unwrap();
        let (sender, receiver) = mpsc::sync_channel(2);
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let writer = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (server, _) = listener.accept().unwrap();
        server
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .init_resource::<bevy::input::ButtonInput<bevy::prelude::KeyCode>>()
            .init_resource::<bevy::asset::Assets<bevy::prelude::Mesh>>()
            .init_resource::<bevy::asset::Assets<bevy::prelude::StandardMaterial>>()
            .init_resource::<bevy::asset::Assets<bevy::image::Image>>()
            .init_resource::<FpsOverlayConfig>();
        super::configure_client_app(&mut app, mirror, receiver, writer, parsed);
        (app, sender, server)
    }

    #[test]
    fn a_capture_carries_its_tree_accounting_whether_or_not_it_is_headless() {
        // `expected_cut_face` adds the tree meshes unconditionally, so if this resource is
        // missing the ACTUAL side never gains them and the cut-face assert reads 0 == 265. It
        // was gated on `--headless`, which is exactly the flag the vehicle sitting card does NOT
        // pass -- so the one run a human performs was the one run that could not capture.
        for args in [
            vec!["7451", "--capture", "/tmp/unused.png", "--frames", "1"],
            vec![
                "7451",
                "--headless",
                "--capture",
                "/tmp/unused.png",
                "--frames",
                "1",
            ],
        ] {
            let headless = args.contains(&"--headless");
            let (app, _sender, _server) = configured_app(&args);
            assert!(
                app.world()
                    .get_resource::<crate::capture::TreeCaptureVerification>()
                    .is_some(),
                "a capture must carry its tree accounting (headless={headless})"
            );
        }
    }

    #[test]
    fn light_source_names_are_closed_and_an_unknown_one_is_refused_loudly() {
        assert_eq!(
            super::LightSource::from_name("sun").unwrap(),
            super::LightSource::Sun
        );
        assert_eq!(
            super::LightSource::from_name("campfire").unwrap(),
            super::LightSource::Campfire
        );
        assert_eq!(
            super::LightSource::from_name("lanterns").unwrap(),
            super::LightSource::Lanterns
        );
        assert_eq!(
            super::LightSource::from_name("ambient").unwrap(),
            super::LightSource::Ambient
        );
        assert_eq!(
            super::LightSource::from_name("moon")
                .unwrap_err()
                .to_string(),
            "unknown light source \"moon\"; expected sun, campfire, lanterns, or ambient"
        );
    }

    #[test]
    fn lighting_keys_change_the_live_scene_and_its_readout() {
        let snapshot = Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Solid(protocol::Material::Stone), Tile::Empty],
            entities: vec![
                protocol::Entity {
                    id: 1,
                    kind: protocol::EntityKind::Campfire,
                    pos: [0, 0, 0],
                    state: protocol::JobState::Idle,
                    light: Some(protocol::LightKind::Campfire),
                },
                protocol::Entity {
                    id: 2,
                    kind: protocol::EntityKind::Dwarf,
                    pos: [1, 0, 0],
                    state: protocol::JobState::Idle,
                    light: Some(protocol::LightKind::Lantern),
                },
            ],
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        };
        let (mut app, _sender, _server) = configured_app_with_snapshot(&[], snapshot);
        app.update();

        let press = |app: &mut App, key| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            app.update();
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(key);
            keys.clear();
        };
        let readout = |app: &mut App| {
            app.world_mut()
                .query_filtered::<&bevy::prelude::Text, With<super::LightingReadout>>()
                .single(app.world())
                .unwrap()
                .0
                .clone()
        };

        assert_eq!(
            readout(&mut app),
            "F5 sun on  F6 campfire on  F7 lanterns on  F8 ambient on"
        );
        for (key, source) in [
            (KeyCode::F5, super::LightSource::Sun),
            (KeyCode::F6, super::LightSource::Campfire),
            (KeyCode::F7, super::LightSource::Lanterns),
            (KeyCode::F8, super::LightSource::Ambient),
        ] {
            press(&mut app, key);
            assert!(
                !app.world()
                    .resource::<super::LightingToggles>()
                    .enabled(source),
                "{key:?} must alter the source the seat-side instrument names"
            );
        }
        assert_eq!(
            readout(&mut app),
            "F5 sun off  F6 campfire off  F7 lanterns off  F8 ambient off"
        );

        assert_eq!(
            app.world_mut()
                .query_filtered::<&bevy::prelude::DirectionalLight, With<super::SunLight>>()
                .single(app.world())
                .unwrap()
                .illuminance,
            0.0,
            "F5 must remove the directional light the renderer reads"
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<&bevy::prelude::AmbientLight, With<Camera3d>>()
                .single(app.world())
                .unwrap()
                .brightness,
            0.0,
            "F8 must remove the camera ambient fill the renderer reads"
        );
        let points = app
            .world_mut()
            .query::<(&crate::project::ProjectedLight, &bevy::prelude::PointLight)>()
            .iter(app.world())
            .map(|(kind, light)| (kind.0, light.intensity))
            .collect::<Vec<_>>();
        let intensity = |kind| {
            points
                .iter()
                .find_map(|(actual, intensity)| (*actual == kind).then_some(*intensity))
                .expect("the fixture must retain its named point light")
        };
        assert_eq!(
            intensity(protocol::LightKind::Campfire),
            0.0,
            "F6 must remove campfire pixels"
        );
        assert_eq!(
            intensity(protocol::LightKind::Lantern),
            0.0,
            "F7 must remove lantern pixels"
        );
    }

    #[test]
    fn the_startup_asset_line_reads_the_blobs_rather_than_the_array_length() {
        let (embedded, bytes) = super::tree_asset_summary();
        assert_eq!(
            embedded,
            super::TREE_ASSETS.len(),
            "every embedded pine must carry binary-glTF bytes"
        );
        assert!(
            bytes > 1_000_000,
            "the four pines are ~1.28 MB; {bytes} bytes means a blob is empty or truncated"
        );
        // The discriminator: the count must come from the BYTES, so an emptied blob moves it.
        // Reading `TREE_ASSETS.len()` would report 4 with every blob emptied, which is what the
        // startup line used to do and what made it unable to observe its own failure.
        assert_eq!(
            super::TREE_ASSETS
                .iter()
                .filter(|(_, blob)| blob.starts_with(b"glTF"))
                .count(),
            embedded
        );
    }

    fn scripted_drag_line(start: [i32; 3], end: [i32; 3]) -> String {
        let viewport = UVec2::new(1920, 1080);
        let rig = CameraRig::new([0, 0, 0]);
        let start_cursor = rig
            .project_world_point(start)
            .expect("the literal start tile must project")
            * viewport.as_vec2();
        let end_cursor = rig
            .project_world_point(end)
            .expect("the literal end tile must project")
            * viewport.as_vec2();
        let drag = format!(
            "dig,{},{},{},{}",
            start_cursor.x, start_cursor.y, end_cursor.x, end_cursor.y
        );
        let snapshot = Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 3, y: 2, z: 1 },
            tiles: vec![Tile::Solid(protocol::Material::Stone); 6],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        };
        let (mut app, _sender, server) = configured_app_with_snapshot(
            &[
                "--capture",
                "working.png",
                "--frames",
                "60",
                "--drag",
                &drag,
            ],
            snapshot,
        );

        app.update();
        let camera_entity = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<CameraRig>>()
            .single(app.world())
            .unwrap();
        let mut camera = Camera::default();
        camera.computed.target_info = Some(RenderTargetInfo {
            physical_size: viewport,
            scale_factor: 1.0,
        });
        let mut projection = bevy::prelude::PerspectiveProjection::default();
        projection.update(viewport.x as f32, viewport.y as f32);
        camera.computed.clip_from_view = projection.get_clip_from_view();
        let transform = rig.transform();
        app.world_mut().entity_mut(camera_entity).insert((
            Camera3d::default(),
            camera,
            transform,
            GlobalTransform::from(transform),
            rig,
        ));
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(viewport.x, viewport.y),
                ..Default::default()
            },
            PrimaryWindow,
        ));

        // Press, move while held, then release. The press now WAITS for the first live pick
        // instead of firing on frame 1 regardless, so the stage count is no longer fixed — drive
        // until the drag reports completion, which also fails loudly if it never does.
        for _ in 0..8 {
            app.update();
            if app.world().resource::<super::ScriptedDrag>().completed() {
                break;
            }
        }
        assert!(
            app.world().resource::<super::ScriptedDrag>().completed(),
            "scripted drag never reached its release stage"
        );

        let mut line = String::new();
        BufReader::new(server).read_line(&mut line).unwrap();
        line
    }

    /// The wiring CALLS, not just the functions they call.
    ///
    /// Round 1 of this story's review extracted `insert_capture_resources` so the resource write
    /// was testable — and the review then deleted the *call to it* from `run()` and watched the
    /// whole suite stay green. Same for `client_systems` and `projection_systems`, which the
    /// headless harness invoked itself. Every expectation below is hand-written here.
    #[test]
    fn the_production_wiring_runs_every_call_run_makes_after_its_plugins() {
        let (mut app, _sender, _server) = configured_app(&[
            "--capture",
            "working.png",
            "--frames",
            "60",
            "--cursor",
            "960,540",
        ]);

        assert_eq!(
            app.world()
                .get_resource::<super::ScriptedCursor>()
                .map(|cursor| cursor.0),
            Some(bevy::prelude::Vec2::new(960.0, 540.0)),
            "the parsed --cursor must reach the pick's resource through the call run() makes"
        );
        assert!(
            app.world()
                .get_resource::<crate::capture::CaptureState>()
                .is_some(),
            "the capture branch must run for a --capture argument"
        );

        app.update();

        assert_eq!(
            app.world_mut()
                .query::<&CameraRig>()
                .iter(app.world())
                .count(),
            1,
            "client_systems must be registered from the production path — its startup scene is \
             the whole view"
        );
        assert_eq!(
            app.world()
                .get_resource::<crate::slice::SliceLevel>()
                .map(|slice| slice.level()),
            Some(0),
            "the world resources must reach the app through the same call"
        );
        // `projection_systems` owns the on-screen level readout's startup spawn — 7.1's review
        // found that whole readout deletable with the suite green, so it is observed here rather
        // than asserted to be registered.
        // NOTE: compared as a set — query iteration order follows archetype order, which a new
        // client-local resource reshuffles; the claim is which readouts exist, not their order.
        let mut spawned = app
            .world_mut()
            .query::<&bevy::prelude::Text>()
            .iter(app.world())
            .map(|text| text.0.clone())
            .collect::<Vec<_>>();
        spawned.sort();
        // The bearing is part of the expectation on purpose. This readout was found DELETABLE
        // with the suite green once already, and the compass is the newest thing hanging off it:
        // a `N ?` here would mean the production path spawned the readout but never resolved a
        // camera, which is the silent half-working state this test exists to catch.
        let mut expected = vec![
            format!(
                "{}  N down-left",
                app.world()
                    .resource::<crate::slice::SliceLevel>()
                    .readout(false, None)
            ),
            "1 dig  2 channel  3 stockpile  4 clear".to_string(),
            "F5 sun on  F6 campfire on  F7 lanterns on  F8 ambient on".to_string(),
        ];
        expected.sort();
        assert_eq!(
            spawned, expected,
            "projection_systems must be registered from the production path too"
        );
        assert!(
            spawned.iter().all(|text| !text.contains("N ?")),
            "the compass must resolve a real camera on the production path, not report unknown"
        );
    }

    #[test]
    fn subdiv_flag_reaches_the_rendered_terrain_and_one_keeps_the_shipped_scene() {
        let (mut default, _, _) = configured_app(&[]);
        let (mut one, _, _) = configured_app(&["--subdiv", "1"]);
        let (mut two, _, _) = configured_app(&["--subdiv", "2"]);
        default.update();
        one.update();
        two.update();

        let terrain_tiles = |app: &mut App| {
            let mut tiles = app
                .world_mut()
                .query::<(
                    &TerrainTile,
                    &bevy::prelude::Transform,
                    &bevy::prelude::Mesh3d,
                    &bevy::prelude::MeshMaterial3d<bevy::prelude::StandardMaterial>,
                )>()
                .iter(app.world())
                .map(|(tile, transform, mesh, material)| {
                    (tile.0, *transform, mesh.0.clone(), material.0.clone())
                })
                .collect::<Vec<_>>();
            tiles.sort_by_key(|(tile, _, _, _)| *tile);
            tiles
        };
        let snow_caps = |app: &mut App| {
            let mut caps = app
                .world_mut()
                .query::<(
                    &SnowCap,
                    &bevy::prelude::Transform,
                    &bevy::prelude::Mesh3d,
                    &bevy::prelude::MeshMaterial3d<bevy::prelude::StandardMaterial>,
                )>()
                .iter(app.world())
                .map(|(cap, transform, mesh, material)| {
                    (cap.0, *transform, mesh.0.clone(), material.0.clone())
                })
                .collect::<Vec<_>>();
            caps.sort_by_key(|(cap, _, _, _)| *cap);
            caps
        };
        assert_eq!(
            terrain_tiles(&mut one),
            terrain_tiles(&mut default),
            "--subdiv 1 must retain the hand-written per-cell scene byte-for-byte"
        );
        assert_eq!(
            snow_caps(&mut one),
            snow_caps(&mut default),
            "--subdiv 1 must retain the snow-cap scene byte-for-byte"
        );
        assert!(
            default
                .world()
                .get_resource::<TerrainSubdivision>()
                .is_none(),
            "no flag must not even install the opt-in terrain resource"
        );
        assert_eq!(
            one.world().resource::<TerrainSubdivision>().0,
            1,
            "the parsed control must reach the projection resource"
        );
        assert_eq!(
            two.world().resource::<TerrainSubdivision>().0,
            2,
            "the parsed fine setting must reach the projection resource"
        );
        assert!(
            terrain_tiles(&mut two).is_empty(),
            "--subdiv 2 must replace the drawn cube entities, not merely accept an inert flag"
        );
        assert!(
            two.world_mut()
                .query::<&TerrainChunk>()
                .iter(two.world())
                .count()
                > 0,
            "--subdiv 2 must produce chunk mesh render entities"
        );
    }

    /// A 40x4x4 stepped slab, wide enough to span three 16-cell chunks on x.
    fn wide_snapshot() -> Snapshot {
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
        Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 40, y: 4, z: 4 },
            tiles,
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        }
    }

    /// A snow cap must never outlive the tile it caps.
    ///
    /// Wolf, from the vehicle: "after digging those big caps stay floating over empty space". A
    /// cap is `ClientLocal` presentation pinned to a cell top, so if a dig empties the cell and
    /// the cap is not despawned it hangs in the air over the hole. The invariant is the same at
    /// every subdivision: every `SnowCap` sits on a solid or ramp cell with nothing solid above.
    #[test]
    fn a_dug_tile_takes_its_snow_cap_with_it() {
        for args in [vec!["--subdiv", "1"], vec!["--subdiv", "2"], vec![]] {
            let (mut app, sender, _server) = configured_app_with_snapshot(&args, wide_snapshot());
            app.update();
            let dug = [8, 1, 1];
            let caps = |app: &mut App| {
                app.world_mut()
                    .query::<&crate::project::SnowCap>()
                    .iter(app.world())
                    .map(|cap| cap.0)
                    .collect::<std::collections::BTreeSet<_>>()
            };
            let fine = args.contains(&"2");
            if fine {
                // The fine path paints snow onto the top faces instead of spawning slabs, so
                // there is nothing to leave floating. That IS the fix; assert it rather than
                // skipping the case.
                assert!(
                    caps(&mut app).is_empty(),
                    "{args:?}: the fine path must spawn no snow-cap entities at all"
                );
            } else {
                assert!(
                    caps(&mut app).contains(&dug),
                    "{args:?}: the fixture must cap {dug:?} before the dig, or this proves nothing"
                );
            }
            sender
                .send(Ok(WireMessage::Delta(Box::new(protocol::Delta {
                    msg_type: MessageType::Delta,
                    tick: 1,
                    tiles: vec![protocol::TileChange {
                        pos: dug,
                        tile: Tile::Empty,
                    }],
                    entities: Vec::new(),
                    designations: Vec::new(),
                    zones: Vec::new(),
                    items: Vec::new(),
                    speed: Speed::Normal,
                }))))
                .unwrap();
            app.update();
            app.update();

            let after = caps(&mut app);
            let mirror = &app.world().resource::<MirrorResource>().0;
            let solid = |position: [i32; 3]| {
                matches!(mirror.tile(position), Some(Tile::Solid(_) | Tile::Ramp(_)))
            };
            assert!(
                !solid(dug),
                "{args:?}: the delta must have emptied the tile"
            );
            let floating = after
                .into_iter()
                .filter(|cap| !solid(*cap))
                .collect::<Vec<_>>();
            assert!(
                floating.is_empty(),
                "{args:?}: {floating:?} still carry a snow cap over empty space"
            );
        }
    }

    /// A frame with no terrain change must leave the fine terrain entities alone.
    ///
    /// The first cut of the incremental path ran on every frame — "not a full rebuild" is the
    /// common case, not the dig case — and scanned the whole world for the draw set each time:
    /// ~130 ms per frame, 400 times in a two-minute run, far worse than the stall it was written
    /// to remove. Caught by watching the live log, not by a test.
    ///
    /// **This test does not pin the guard that fixed it**, and the mutation table says so rather
    /// than carrying a row that reads green. With the guard removed the branch still computes an
    /// empty chunk set, despawns nothing and spawns nothing, so the ECS is identical and only the
    /// wasted work differs — and a test cannot see wasted work. What this does pin is that a
    /// quiet frame never *destroys* terrain, which is the failure mode that would be visible.
    #[test]
    fn quiet_frames_leave_the_fine_terrain_alone() {
        let (mut two, _, _) = configured_app_with_snapshot(&["--subdiv", "2"], wide_snapshot());
        two.update();
        let before = two
            .world_mut()
            .query::<(bevy::prelude::Entity, &TerrainChunk)>()
            .iter(two.world())
            .map(|(entity, chunk)| (chunk.0, entity))
            .collect::<Vec<_>>();
        assert!(!before.is_empty());
        for _ in 0..3 {
            two.update();
        }
        let after = two
            .world_mut()
            .query::<(bevy::prelude::Entity, &TerrainChunk)>()
            .iter(two.world())
            .map(|(entity, chunk)| (chunk.0, entity))
            .collect::<Vec<_>>();
        assert_eq!(
            before, after,
            "three quiet frames respawned terrain; the incremental path is running when nothing \
             changed"
        );
    }

    /// One changed tile must rebuild only the chunks it can reach — not the world.
    ///
    /// It used to rebuild the world: `reconcile_projection` promoted any terrain delta to a full
    /// rebuild at `--subdiv N > 1`. That is one whole mesh build per dug tile, and because a dwarf
    /// digs continuously it froze every other dwarf for the length of the job. Wolf reported it
    /// twice from the vehicle, the second time as "when one dwarf digs all other movement is in
    /// halt". `project::tests::partial_rebuild_matches_the_whole_world_build` carries the safety
    /// half — that a partial rebuild is indistinguishable from a whole one.
    #[test]
    fn one_dirty_tile_rebuilds_only_the_chunks_it_can_reach() {
        let chunk_entities = |app: &mut App| {
            app.world_mut()
                .query::<(bevy::prelude::Entity, &TerrainChunk)>()
                .iter(app.world())
                .map(|(entity, chunk)| (chunk.0, entity))
                .collect::<Vec<_>>()
        };
        let (mut two, _, _) = configured_app_with_snapshot(&["--subdiv", "2"], wide_snapshot());
        two.update();
        let before = chunk_entities(&mut two);
        let chunks_before = before
            .iter()
            .map(|(c, _)| *c)
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            chunks_before.len() >= 3,
            "the fixture must span several chunks or this test cannot fail; got {chunks_before:?}"
        );

        two.world_mut()
            .resource_mut::<ProjectionWork>()
            .dirty_tiles
            .insert([2, 2, 1]);
        two.update();
        let after = chunk_entities(&mut two);

        let survived = before
            .iter()
            .filter(|entry| after.contains(entry))
            .map(|(chunk, _)| *chunk)
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            !survived.is_empty(),
            "every chunk was rebuilt for one tile in chunk [0,0,0] — the whole-world rebuild is back"
        );
        assert!(
            !survived.contains(&[0, 0, 0]),
            "the chunk holding the changed tile must be rebuilt, not left stale"
        );
        assert_eq!(
            after
                .iter()
                .map(|(c, _)| *c)
                .collect::<std::collections::BTreeSet<_>>(),
            chunks_before,
            "the rebuild must leave the same set of chunks present"
        );
    }

    #[test]
    fn configured_app_sends_a_real_mouse_drags_command_to_the_daemon_socket() {
        let (mut app, _sender, server) = configured_app(&[]);
        let viewport = UVec2::new(1920, 1080);
        let rig = CameraRig::new([0, 0, 0]);
        let cursor = rig
            .project_world_point([0, 0, 0])
            .expect("the literal visible tile must project")
            * viewport.as_vec2();

        app.update();
        let camera_entity = app
            .world_mut()
            .query_filtered::<bevy::prelude::Entity, With<CameraRig>>()
            .single(app.world())
            .unwrap();
        let mut camera = Camera::default();
        camera.computed.target_info = Some(RenderTargetInfo {
            physical_size: viewport,
            scale_factor: 1.0,
        });
        let mut projection = bevy::prelude::PerspectiveProjection::default();
        projection.update(viewport.x as f32, viewport.y as f32);
        camera.computed.clip_from_view = projection.get_clip_from_view();
        let transform = rig.transform();
        app.world_mut().entity_mut(camera_entity).insert((
            Camera3d::default(),
            camera,
            transform,
            GlobalTransform::from(transform),
            rig,
        ));
        let mut window = Window {
            resolution: WindowResolution::new(viewport.x, viewport.y),
            ..Default::default()
        };
        window.set_cursor_position(Some(cursor));
        app.world_mut().spawn((window, PrimaryWindow));
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit1);
        app.update();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::Digit1);
        keys.clear();

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

        let mut line = String::new();
        BufReader::new(server).read_line(&mut line).unwrap();
        assert_eq!(
            line,
            "{\"type\":\"designate\",\"kind\":\"dig\",\"rect\":{\"min\":[0,0,0],\"max\":[0,0,0]}}\n",
            "the production configuration must carry the mouse path all the way to daemon bytes"
        );
    }

    #[test]
    fn parsed_capture_drags_send_their_own_rectangles_to_the_daemon_socket() {
        let first = scripted_drag_line([0, 0, 0], [1, 0, 0]);
        let second = scripted_drag_line([1, 0, 0], [2, 1, 0]);

        assert_eq!(
            first,
            "{\"type\":\"designate\",\"kind\":\"dig\",\"rect\":{\"min\":[0,0,0],\"max\":[1,0,0]}}\n",
            "the first parsed --drag must send its literal anchor-level rectangle"
        );
        assert_eq!(
            second,
            "{\"type\":\"designate\",\"kind\":\"dig\",\"rect\":{\"min\":[1,0,0],\"max\":[2,1,0]}}\n",
            "the second parsed --drag must send its literal anchor-level rectangle"
        );
        assert_ne!(
            first, second,
            "different parsed --drag values must not collapse to the same wire rectangle"
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
    fn the_z_flag_reaches_the_slice_resource_rather_than_merely_parsing() {
        // Independent oracle: the expected level is written here, not read back from `Args`.
        // Replacing `SliceLevel::pinned` with `at_world_top` left the whole suite green, so
        // `--z 9 --capture` would have silently photographed the full-depth view.
        let dims = Dims { x: 4, y: 4, z: 32 };
        assert_eq!(super::initial_slice(dims, Some(9)).level(), 9);
        assert_eq!(super::initial_slice(dims, None).level(), 31);
        // And the pin is clamped by the same rule as every other level change.
        assert_eq!(super::initial_slice(dims, Some(999)).level(), 31);
        assert_eq!(super::initial_slice(dims, Some(-5)).level(), 0);
    }

    /// The devpods this project builds on have no display server, so winit panics and every
    /// pixel AC has been vehicle-bound since 2026-08-11. They DO have a CPU Vulkan device
    /// (lavapipe), and rendering needs a device rather than a window — `--headless` is what turns
    /// that into a measurement. Verified live: the headless client reproduced the draw-set oracle
    /// on both the pre-9.4 world (53,365) and the mesh-tree draw set (39,936).
    #[test]
    fn headless_is_off_by_default_and_on_only_when_asked() {
        let interactive = super::parse_args_from([std::ffi::OsString::from("7451")])
            .expect("a bare port must parse");
        assert!(
            !interactive.headless,
            "a client asked for nothing special must still open a window"
        );
        let headless = super::parse_args_from([
            std::ffi::OsString::from("7451"),
            std::ffi::OsString::from("--headless"),
            std::ffi::OsString::from("--capture"),
            std::ffi::OsString::from("boot.png"),
            std::ffi::OsString::from("--frames"),
            std::ffi::OsString::from("220"),
        ])
        .expect("a headless capture must parse");
        assert!(headless.headless);
        assert_eq!(headless.capture, Some(std::path::PathBuf::from("boot.png")));
    }

    /// The camera must actually be pointed somewhere. A headless run whose camera still targets a
    /// window renders nothing and the screenshot is empty — silently, which is the failure shape
    /// this project keeps meeting.
    #[test]
    fn a_headless_camera_draws_into_an_offscreen_target_and_a_windowed_one_does_not() {
        use bevy::prelude::World as BevyWorld;

        for headless in [false, true] {
            let mut world = BevyWorld::new();
            world.init_resource::<bevy::asset::Assets<bevy::image::Image>>();
            if headless {
                world.insert_resource(super::HeadlessRequested);
            }
            world
                .run_system_once(super::setup_camera)
                .expect("setup_camera must run");

            // Count IMAGE targets specifically. A camera always carries a RenderTarget — the
            // default one points at a window — so counting the component discriminates nothing.
            // The first draft of this test did exactly that and passed for the wrong reason.
            let targets = world
                .query::<&bevy::camera::RenderTarget>()
                .iter(&world)
                .filter(|target| matches!(target, bevy::camera::RenderTarget::Image(_)))
                .count();
            let resource = world.get_resource::<super::HeadlessTarget>().is_some();
            assert_eq!(
                targets, headless as usize,
                "headless={headless}: expected the offscreen render target only when headless"
            );
            assert_eq!(
                resource, headless,
                "headless={headless}: the capture reads HeadlessTarget to know what to screenshot"
            );
        }
    }

    #[test]
    fn capture_slice_level_requires_capture_and_is_retained_for_pinning() {
        // `--z` no longer requires `--capture`: the interactive client must be able to boot at a
        // level, or reaching the dig site is 22 keypresses and the vehicle recipe cannot run.
        let interactive = super::parse_args_from([
            std::ffi::OsString::from("--z"),
            std::ffi::OsString::from("9"),
        ])
        .expect("a level without a capture boots the interactive client pinned");
        assert_eq!(interactive.slice_level, Some(9));
        assert!(interactive.capture.is_none());
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
    fn capture_distance_requires_capture_and_is_retained_for_pinning() {
        assert!(
            super::parse_args_from([
                std::ffi::OsString::from("--distance"),
                std::ffi::OsString::from("30")
            ])
            .is_err()
        );
        let args = super::parse_args_from([
            std::ffi::OsString::from("--capture"),
            std::ffi::OsString::from("working.png"),
            std::ffi::OsString::from("--frames"),
            std::ffi::OsString::from("12"),
            std::ffi::OsString::from("--distance"),
            std::ffi::OsString::from("30"),
        ])
        .expect("a capture distance must parse");
        assert_eq!(args.distance, Some(30.0));
        assert!(
            super::parse_args_from([
                std::ffi::OsString::from("--capture"),
                std::ffi::OsString::from("working.png"),
                std::ffi::OsString::from("--frames"),
                std::ffi::OsString::from("12"),
                std::ffi::OsString::from("--distance"),
                std::ffi::OsString::from("NaN"),
            ])
            .is_err(),
            "a camera distance must be finite"
        );
    }

    /// The scripted drag used to advance Press -> Hold -> Release on the first three `Update`s
    /// UNCONDITIONALLY, whether or not the pick had resolved. On a cold first frame — camera
    /// `computed.target_info` not yet written, no primary window resolved yet, `viewport_to_world`
    /// failing — the press anchored on `None`, no command was ever built, and NOTHING reported it.
    /// 8.1's `--cursor` rewrote the cursor every frame and self-healed; three unconditional shots
    /// at the coldest moment in the app's life do not.
    #[test]
    fn a_scripted_drag_waits_for_a_live_pick_instead_of_pressing_into_the_dark() {
        use bevy::prelude::Vec2;

        use crate::{
            designate::DesignateMode,
            pick::{Face, PickedCell, PickedTile},
        };

        let spec = super::ScriptedDragSpec {
            mode: DesignateMode::Dig,
            start: Vec2::new(100.0, 100.0),
            end: Vec2::new(200.0, 200.0),
        };
        let mut app = bevy::app::App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<PickedTile>()
            .insert_resource(super::ScriptedDrag {
                spec,
                stage: super::ScriptedDragStage::Press,
            });
        app.world_mut().spawn((Window::default(), PrimaryWindow));

        // No pick has ever resolved: the drag must hold at Press however many frames pass.
        for _ in 0..5 {
            app.world_mut()
                .run_system_once(super::apply_scripted_input)
                .unwrap();
        }
        assert!(
            matches!(
                app.world().resource::<super::ScriptedDrag>().stage,
                super::ScriptedDragStage::Press
            ),
            "the drag pressed before any pick resolved; it would anchor on nothing and vanish"
        );
        assert!(
            !app.world()
                .resource::<ButtonInput<MouseButton>>()
                .pressed(MouseButton::Left),
            "no button may be pressed while the pick machinery is still cold"
        );

        // The moment a pick is live, the drag proceeds.
        app.world_mut().insert_resource(PickedTile(Some(PickedCell {
            tile: [1, 1, 1],
            face: Face::Top,
        })));
        app.world_mut()
            .run_system_once(super::apply_scripted_input)
            .unwrap();
        assert!(
            matches!(
                app.world().resource::<super::ScriptedDrag>().stage,
                super::ScriptedDragStage::Hold
            ),
            "with a live pick the drag must advance rather than stalling forever"
        );
    }

    /// `apply_scripted_input` takes the drag branch OR the cursor branch, never both, so a
    /// `--cursor` alongside `--drag` was parsed, validated, inserted and then never written to
    /// the window — while the capture still asserted the live pick against it. A guaranteed
    /// spurious failure, in a parser where every other bad pairing bails.
    #[test]
    fn a_scripted_cursor_and_a_scripted_drag_are_mutually_exclusive() {
        let both = super::parse_args_from([
            std::ffi::OsString::from("--capture"),
            std::ffi::OsString::from("working.png"),
            std::ffi::OsString::from("--frames"),
            std::ffi::OsString::from("30"),
            std::ffi::OsString::from("--cursor"),
            std::ffi::OsString::from("960,540"),
            std::ffi::OsString::from("--drag"),
            std::ffi::OsString::from("dig,10,10,20,20"),
        ]);
        assert!(
            both.is_err(),
            "a scripted drag moves the cursor itself; accepting both silently ignores one flag \
             the operator typed"
        );
        let Err(error) = both else {
            unreachable!("asserted above to be rejected");
        };
        assert!(
            error.to_string().contains("mutually exclusive"),
            "the rejection must say WHY, not merely fail: {error}"
        );
    }

    /// AC16 requires a run that never reaches its tick to exit NON-ZERO. `App::run()` RETURNS the
    /// status and `AppExit` is not `#[must_use]`, so `app.run();` compiled clean under
    /// `-D warnings` while throwing every capture failure away.
    ///
    /// Asserted against the source because `run()` needs a socket AND a window: no test in this
    /// environment can execute it, and a process exit code is not observable from inside the
    /// process that would set it. This is the same include_str! shape `designate.rs` uses for the
    /// rect helper — weaker than an execution, and far stronger than the nothing that was here.
    #[test]
    fn run_consumes_the_runners_exit_status_rather_than_discarding_it() {
        let source = include_str!("ingest.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the production module precedes its tests");
        assert!(
            !source.contains("    app.run();\n"),
            "`app.run();` discards the AppExit, so a failed capture exits 0"
        );
        assert!(
            source.contains("if let AppExit::Error(code) = app.run()"),
            "run() must inspect the runner's exit status and propagate a non-zero code"
        );
    }

    #[test]
    fn capture_cursor_requires_capture_and_rejects_an_invalid_coordinate() {
        let args = super::parse_args_from([
            std::ffi::OsString::from("--capture"),
            std::ffi::OsString::from("working.png"),
            std::ffi::OsString::from("--frames"),
            std::ffi::OsString::from("1"),
            std::ffi::OsString::from("--cursor"),
            std::ffi::OsString::from("960,540"),
        ])
        .expect("a capture cursor must parse");
        assert_eq!(args.cursor, Some(bevy::prelude::Vec2::new(960.0, 540.0)));
        assert!(
            super::parse_args_from([
                std::ffi::OsString::from("--cursor"),
                std::ffi::OsString::from("960,540"),
            ])
            .is_err(),
            "a scripted cursor without a capture has no valid instrument to drive"
        );
        let invalid = super::parse_args_from([
            std::ffi::OsString::from("--capture"),
            std::ffi::OsString::from("working.png"),
            std::ffi::OsString::from("--frames"),
            std::ffi::OsString::from("1"),
            std::ffi::OsString::from("--cursor"),
            std::ffi::OsString::from("not-a-coordinate"),
        ]);
        assert!(
            invalid.is_err(),
            "a malformed --cursor must not fall through to the port parser"
        );
        let Err(error) = invalid else {
            unreachable!("the malformed cursor was asserted above to be rejected");
        };
        assert!(error.to_string().contains("invalid --cursor"));
    }

    /// The other half of `--cursor`, and the half story 8.1's mutation table caught SURVIVING.
    /// The scripted-cursor test above writes `ScriptedCursor` by hand, so it pins the resource ->
    /// pick seam and says nothing about whether `run()` ever writes that resource: replacing the
    /// insert with `let _ = cursor;` left the whole suite green. This runs the REAL wiring on a
    /// REAL parsed `Args`, which is the only way the flag's own path is observed.
    #[test]
    fn the_cursor_flag_reaches_a_live_resource_rather_than_merely_parsing() {
        fn scripted(args: &[&str]) -> Option<bevy::prelude::Vec2> {
            let parsed = super::parse_args_from(
                args.iter()
                    .map(std::ffi::OsString::from)
                    .collect::<Vec<_>>(),
            )
            .expect("the arguments under test must parse");
            let mut app = App::new();
            super::insert_capture_resources(&mut app, &parsed);
            app.world()
                .get_resource::<super::ScriptedCursor>()
                .map(|cursor| cursor.0)
        }

        // Independent oracle: the expected coordinates are written here, not read back from `Args`.
        assert_eq!(
            scripted(&[
                "--capture",
                "working.png",
                "--frames",
                "1",
                "--cursor",
                "960,540"
            ]),
            Some(bevy::prelude::Vec2::new(960.0, 540.0)),
            "a parsed --cursor must reach the resource the pick system reads"
        );
        assert_eq!(
            scripted(&[
                "--capture",
                "working.png",
                "--frames",
                "1",
                "--cursor",
                "12,34"
            ]),
            Some(bevy::prelude::Vec2::new(12.0, 34.0)),
            "the resource must carry the coordinate given, not a fixed one"
        );
        // No flag means no resource at all, so the live cursor is left alone.
        assert_eq!(
            scripted(&["--capture", "working.png", "--frames", "1"]),
            None,
            "without --cursor nothing may overwrite the real cursor position"
        );
    }

    /// The half the parse test could not reach, and the reason its NAME was a lie. Reviewed
    /// 2026-08-21: replacing `setup_camera`'s assignment with `let _ = distance;` left all 106
    /// tests passing, so `--distance 30` would have parsed, validated, and then silently captured
    /// at `BOOT_DISTANCE` — the flag exists precisely because a capture at the boot framing put
    /// 6.1's dig site at 0.30 % of the frame and Wolf's reaction was "did not see the difference".
    /// This runs the REAL `setup_camera` system and reads the distance back off the spawned rig.
    #[test]
    fn the_distance_flag_reaches_the_camera_rig_rather_than_merely_parsing() {
        fn rig_distance(requested: Option<f32>) -> f32 {
            let mut app = App::new();
            if let Some(distance) = requested {
                app.insert_resource(super::CaptureDistance(distance));
            }
            app.world_mut()
                .run_system_once(super::setup_camera)
                .expect("the camera setup must run");
            app.world_mut()
                .query::<&CameraRig>()
                .iter(app.world())
                .map(|rig| rig.distance)
                .next()
                .expect("the camera setup must spawn a rig")
        }

        // Independent oracle: the expected distances are written here, not read back from `Args`.
        assert_eq!(rig_distance(Some(30.0)), 30.0);
        assert_eq!(rig_distance(Some(4.5)), 4.5);
        // No flag means the boot framing, unchanged. Written by hand rather than read back from
        // `BOOT_DISTANCE`, on the same principle as the `--z` test above: an oracle that reads the
        // constant it is checking cannot fail when the constant moves.
        assert_eq!(rig_distance(None), 90.0);
        // And the pin is clamped by the same rule the flag documents.
        assert_eq!(rig_distance(Some(0.0)), 4.0);
        assert_eq!(rig_distance(Some(9_000.0)), 500.0);
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
