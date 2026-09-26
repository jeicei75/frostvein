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
    anti_alias::fxaa::Fxaa,
    app::{App, AppExit, PostUpdate, Startup, Update},
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::change_detection::DetectChanges,
    ecs::message::{MessageReader, MessageWriter},
    ecs::schedule::IntoScheduleConfigs,
    input::{
        ButtonInput,
        mouse::{MouseButton, MouseMotion, MouseWheel},
    },
    light::{FogVolume, VolumetricFog, VolumetricLight},
    pbr::{DistanceFog, FogFalloff, ScreenSpaceAmbientOcclusion},
    post_process::{bloom::Bloom, dof::DepthOfField},
    prelude::{
        AmbientLight, Camera3d, ClearColor, Color, Commands, Component, DefaultPlugins,
        DirectionalLight, GlobalTransform, GlobalZIndex, Has, KeyCode, Node, PerspectiveProjection,
        PositionType, Projection, Query, Res, ResMut, Resource, Text, TextColor, TextFont, Time,
        Transform, TransformSystems, Vec2, Vec3, Window, With, Without, px,
    },
    render::renderer::RenderAdapterInfo,
    window::PrimaryWindow,
};
use bevy::{
    app::PluginGroup,
    app::ScheduleRunnerPlugin,
    asset::{AssetPlugin, Assets, Handle, RenderAssetUsages},
    camera::{Exposure, RenderTarget},
    image::{Image, ImageSampler},
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::Msaa,
    },
    window::{ExitCondition, WindowPlugin},
    winit::WinitPlugin,
};
use client_core::Mirror;
use protocol::{Delta, Dims, Snapshot};

use crate::{
    appearance::{day_weight, light_properties, lighting_at, mix_color, night_lighting},
    atmosphere::{fall_snow, setup_atmosphere, sun_light_transform},
    blend::TickClock,
    camera::{BOOT_VERTICAL_FOV, CameraRig, camera_readout_line},
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
        ProjectedDesignationKind, ProjectedZone, ProjectionAssets, SceneSource, TerrainQuery,
        TerrainSubdivision, TerrainTile, TreeMeshQuery, WorldProjected, blend_entities,
        flicker_lights, has_terrain_above, reconcile, setup_projection_assets, sync_drag_preview,
        sync_hover_highlight,
    },
    slice::SliceLevel,
    transform::world_to_render_f32,
};

const SNAPSHOT_READ_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;
// The frame drains this queue whole, so the gui absorbs at most MESSAGE_QUEUE x fps deltas a second.
// At 16 a lavapipe gui (~2.5 fps) fell behind Fast4x's ~185 deltas/s and the daemon evicted it
// SILENTLY 34 s in: the reader sat blocked on this queue and never reached the EOF (11.3).
const MESSAGE_QUEUE: usize = 256;
const DEFAULT_AT_TICK_FRAME_BUDGET: u32 = 1_500;

/// Ruled 2026-09-21: non-physical f/0.05 makes the valley read as a miniature at this scale.
const DOF_APERTURE_F_STOPS: f32 = 0.05;
/// Non-physical sky bound: caps background blur so the stars remain present in the frame.
const DOF_MAX_DEPTH: f32 = 120.0;
const FOG_DENSITY_FACTOR: f32 = 0.015;
const FOG_DENSITY_RAMP_HEIGHT: usize = 64;
const FOG_DENSITY_RAMP_FULL_TO: f32 = 0.54;
const FOG_DENSITY_RAMP_ZERO_BY: f32 = 0.83;

/// The four independently inspectable contributors to the rendered valley.
///
/// This is deliberately a fixed seat-side instrument, not a lighting configuration surface:
/// F5--F9 are the complete public control and their state lasts only for this client run.
/// Torches are their own source because the sim really spawns them (`sim-core/src/lib.rs:1573+`)
/// and they were previously hardcoded lit, so "everything off" still lit the camp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightSource {
    Sun,
    Campfire,
    Torches,
    Lanterns,
    Ambient,
}

impl LightSource {
    /// Ordered biggest reach to smallest, which is also the key order F8..F12 and the order the
    /// readout prints. Sun and ambient light the whole valley; the rest are placed sources.
    const ALL: [Self; 5] = [
        Self::Sun,
        Self::Ambient,
        Self::Campfire,
        Self::Torches,
        Self::Lanterns,
    ];

    fn key(self) -> KeyCode {
        match self {
            Self::Sun => KeyCode::F8,
            Self::Ambient => KeyCode::F9,
            Self::Campfire => KeyCode::F10,
            Self::Torches => KeyCode::F11,
            Self::Lanterns => KeyCode::F12,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Sun => "sun",
            Self::Campfire => "campfire",
            Self::Torches => "torches",
            Self::Lanterns => "lanterns",
            Self::Ambient => "ambient",
        }
    }

    pub fn from_name(name: &str) -> anyhow::Result<Self> {
        match name {
            "sun" => Ok(Self::Sun),
            "campfire" => Ok(Self::Campfire),
            "torches" => Ok(Self::Torches),
            "lanterns" => Ok(Self::Lanterns),
            "ambient" => Ok(Self::Ambient),
            _ => bail!(
                "unknown light source {name:?}; expected sun, campfire, torches, lanterns, or ambient"
            ),
        }
    }
}

#[derive(Resource)]
pub struct LightingToggles {
    sun: bool,
    campfire: bool,
    torches: bool,
    lanterns: bool,
    ambient: bool,
}

/// The five seat-toggleable camera effects. This stays a fixed instrument rather than a
/// registry: these are the only concrete effects the client currently ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CameraEffect {
    Fxaa,
    AmbientOcclusion,
    Bloom,
    Dof,
    Haze,
}

impl CameraEffect {
    /// Ordered widest effect on the frame to narrowest, which is also the key order F4..F7 and the
    /// order the readout prints: haze recolours the whole valley, depth of field restages it by
    /// distance, bloom acts on highlights, ambient occlusion only on contact creases. Fxaa is last
    /// because it has no key at all.
    const ALL: [Self; 5] = [
        Self::Haze,
        Self::Dof,
        Self::Bloom,
        Self::AmbientOcclusion,
        Self::Fxaa,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Fxaa => "fxaa",
            Self::AmbientOcclusion => "ao",
            Self::Bloom => "bloom",
            Self::Dof => "dof",
            Self::Haze => "haze",
        }
    }

    /// The seat key for this effect, or `None` for one that is CLI-only.
    ///
    /// **F1 and F2 are not ours to bind.** `DefaultPlugins` pulls in
    /// `bevy_dev_tools::render_debug::RenderDebugOverlayPlugin` whenever the `bevy_dev_tools` and
    /// `bevy_pbr` features are on (`bevy_internal-0.19.0/src/default_plugins.rs:95`), and its
    /// `handle_input` hardcodes F1 to cycle the depth/normal debug overlay and F2 to cycle that
    /// overlay's opacity (`bevy_dev_tools-0.19.0/src/render_debug.rs:107` and `:110`). 11.2 put
    /// dof and haze there, so at the seat those keys ALSO drove Bevy's overlay: the frame went
    /// black (depth), then green/pink/blue (normals), then sat at half and 80% opacity, while this
    /// readout cheerfully reported `F1 dof on`. Every test here runs on `MinimalPlugins`, which
    /// does not include that plugin, so nothing in this suite could see the collision.
    ///
    /// Fxaa gives up its key rather than the story's two new effects losing theirs (Wolf,
    /// 2026-09-22: "fxaa switch is not needed in F10 right now"). It remains fully controllable
    /// with `--fx-off fxaa`; only the live toggle is gone. The whole keymap is due a rethink --
    /// issue #118.
    fn key(self) -> Option<KeyCode> {
        match self {
            Self::Haze => Some(KeyCode::F4),
            Self::Dof => Some(KeyCode::F5),
            Self::Bloom => Some(KeyCode::F6),
            Self::AmbientOcclusion => Some(KeyCode::F7),
            Self::Fxaa => None,
        }
    }

    fn from_name(name: &str) -> anyhow::Result<Self> {
        match name {
            "fxaa" => Ok(Self::Fxaa),
            "ao" => Ok(Self::AmbientOcclusion),
            "bloom" => Ok(Self::Bloom),
            "dof" => Ok(Self::Dof),
            "haze" => Ok(Self::Haze),
            _ => bail!("unknown effect {name:?}; expected fxaa, ao, bloom, dof, or haze"),
        }
    }
}

/// Which fixed camera effects start absent. The camera itself remains the source of truth:
/// toggling always inserts or removes its component rather than retaining a dormant component.
#[derive(Default, Resource)]
struct EffectsOff {
    fxaa: bool,
    ambient_occlusion: bool,
    bloom: bool,
    dof: bool,
    haze: bool,
}

impl EffectsOff {
    fn is_off(&self, effect: CameraEffect) -> bool {
        match effect {
            CameraEffect::Fxaa => self.fxaa,
            CameraEffect::AmbientOcclusion => self.ambient_occlusion,
            CameraEffect::Bloom => self.bloom,
            CameraEffect::Dof => self.dof,
            CameraEffect::Haze => self.haze,
        }
    }

    fn set_off(&mut self, effect: CameraEffect, off: bool) {
        match effect {
            CameraEffect::Fxaa => self.fxaa = off,
            CameraEffect::AmbientOcclusion => self.ambient_occlusion = off,
            CameraEffect::Bloom => self.bloom = off,
            CameraEffect::Dof => self.dof = off,
            CameraEffect::Haze => self.haze = off,
        }
    }

    fn toggle(&mut self, effect: CameraEffect) {
        self.set_off(effect, !self.is_off(effect));
    }

    fn with_off(off: &[CameraEffect]) -> Self {
        let mut effects = Self::default();
        for &effect in off {
            effects.set_off(effect, true);
        }
        effects
    }
}

/// Pins emitter flicker to a reproducible phase for a captured frame.
#[derive(Default, Resource)]
struct LightsSteady(bool);

impl Default for LightingToggles {
    fn default() -> Self {
        Self {
            sun: true,
            campfire: true,
            torches: true,
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
            LightSource::Torches => self.torches,
            LightSource::Lanterns => self.lanterns,
            LightSource::Ambient => self.ambient,
        }
    }

    fn set(&mut self, source: LightSource, enabled: bool) {
        match source {
            LightSource::Sun => self.sun = enabled,
            LightSource::Campfire => self.campfire = enabled,
            LightSource::Torches => self.torches = enabled,
            LightSource::Lanterns => self.lanterns = enabled,
            LightSource::Ambient => self.ambient = enabled,
        }
    }

    fn toggle(&mut self, source: LightSource) {
        self.set(source, !self.enabled(source));
    }

    /// The starting position `--lights-off` asks for. Named sources start dark; the keys still
    /// move them afterwards, because this is the same instrument F5-F9 drive -- a capture just
    /// needs to state where it begins. Idempotent in the source list, so `sun,sun` is `sun`.
    fn with_off(off: &[LightSource]) -> Self {
        let mut toggles = Self::default();
        for &source in off {
            toggles.set(source, false);
        }
        toggles
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
    /// How many tiles the last reconcile DRAINED, kept because the count outlives the set.
    ///
    /// `dirty_tiles` is emptied by `reconcile_projection` in `Update`; the perf row is written by
    /// `record_perf_frame` in `Last`, which Bevy always runs afterwards. Reading the set there
    /// therefore always reads an empty one, and the column that exists to separate edit frames
    /// from steady ones reported `0` on every row of every run. Set unconditionally on each
    /// reconcile, so a steady frame resets it rather than inheriting the last edit's count.
    pub drained_tiles: usize,
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

/// The authored dwarf, embedded the same way the pines are.
///
/// Kept as its own const rather than appended to `TREE_ASSETS`: that array is indexed by
/// `TreeVariant` order and `tree_asset_paths_match_the_loader` pins it to `TREE_SCENE_PATHS`,
/// so a fifth entry would silently break both.
pub const DWARF_ASSET: (&str, &[u8]) = (
    "gltf/SM_VoxelDwarf_Miner01.glb",
    include_bytes!("../../../assets/gltf/SM_VoxelDwarf_Miner01.glb"),
);

/// Is the dwarf blob really here, and which one is it?
///
/// The trees have announced themselves since M2-7 and the dwarf never did, which cost a real
/// session on 2026-09-13: the r8 figure was promoted into `assets/gltf/` and verified three ways
/// on the orchestrator's side, and the operator -- holding a `gui.exe` built before that commit --
/// reasonably concluded the wrong dwarf had been used. Nothing in the binary could settle it.
///
/// The BYTE COUNT is the identifying figure, not a boolean: r3's generated voxel dwarf is
/// 1,009,476 bytes and r8's box-modelled one is 401,832, so the line says which model this
/// executable actually carries. Same rule as the tree line -- the magic is checked so an emptied
/// or truncated `include_bytes!` reads as absent rather than passing.
pub fn dwarf_asset_summary() -> (bool, usize) {
    (DWARF_ASSET.1.starts_with(b"glTF"), DWARF_ASSET.1.len())
}

/// Does the embedded dwarf actually carry an animation clip?
///
/// Read out of the BYTES this binary ships, not off a handle, because the failure it exists to
/// catch is a stale runtime slot: round 18's walk cycle went into the .blend and the GLB in
/// `assets/gltf/` was not re-promoted, so the figure loaded, skinned, posed at bind, and simply
/// never walked. The only complaint anywhere was an asset-server line about a missing
/// `Animation0` label, buried in a frame log. A dwarf that cannot walk should say so at startup,
/// for the same reason the byte count says WHICH dwarf this is.
///
/// Scans the JSON chunk for the `animations` array rather than parsing glTF: this is a
/// present/absent signal to a human reading the startup lines, and `check_asset.py`'s animation
/// clauses are what actually judge a clip.
pub fn dwarf_clip_summary() -> bool {
    let data = DWARF_ASSET.1;
    if data.len() < 20 || !data.starts_with(b"glTF") {
        return false;
    }
    let length = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
    let Some(json) = data.get(20..20 + length) else {
        return false;
    };
    json.windows(13).any(|window| window == br#""animations":"#)
}

/// Publish the embedded pines into the `embedded://` source before anything loads them.
///
/// `AssetPlugin::build` creates the registry and registers the source, so this must run AFTER
/// `DefaultPlugins` and before the startup system that loads the scenes.
fn register_tree_assets(app: &mut App) {
    let registry = app
        .world_mut()
        .resource_mut::<bevy::asset::io::embedded::EmbeddedAssetRegistry>();
    for &(path, bytes) in TREE_ASSETS.iter().chain(std::iter::once(&DWARF_ASSET)) {
        registry.insert_asset(PathBuf::new(), Path::new(path), bytes);
    }
}

pub fn run() -> anyhow::Result<()> {
    if print_version_and_exit_if_asked() {
        return Ok(());
    }
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
    let (dwarf_present, dwarf_bytes) = dwarf_asset_summary();
    eprintln!(
        "gui dwarf asset: {} in this binary, {dwarf_bytes} bytes, walk clip {}",
        if dwarf_present {
            "embedded"
        } else {
            "MISSING OR TRUNCATED"
        },
        if dwarf_clip_summary() {
            "present"
        } else {
            "ABSENT -- this dwarf cannot walk; the runtime GLB is behind the .blend"
        }
    );
    let (mirror, receiver, writer) = connect_to_daemon(args.port)?;
    let mut app = App::new();
    // The asset source is decided ONCE, here, and both the plugin config and the startup lines
    // read that one decision. Deciding it twice is how a client reports one source and reads
    // another.
    let scene_source = match &args.assets {
        None => SceneSource::Embedded,
        Some(dir) => SceneSource::Disk(dir.clone()),
    };
    // SECOND LINE OUT, beside the build stamp, and for the same reason: a session must be able to
    // SEE which asset tree it read rather than be told which one it should have read. Two candidate
    // asset trees is the stale-artifact shape this project keeps paying for.
    eprintln!("gui assets: source={}", scene_source.label());
    let asset_plugin = asset_plugin_for(args.assets.as_deref());
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
                .set(asset_plugin)
                .disable::<WinitPlugin>(),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::ZERO))
        // The overlay plugin wants a window; its config resource is all the client systems read.
        .init_resource::<FpsOverlayConfig>();
    } else {
        app.add_plugins(DefaultPlugins.set(asset_plugin))
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(FpsOverlayPlugin {
                config: overlay_config_on(),
            });
    }
    // Registered on BOTH paths, deliberately. The embedded blobs cost nothing to publish and the
    // `--assets` prefix simply stops naming them, so the disk path does not depend on this being
    // skipped -- which keeps one code path rather than two.
    // NOTE: this is NOT a fallback, and it used to say it was. Under `SceneSource::Disk` the load
    // prefix is `""`, so a missing file on the disk tree simply fails to load; nothing reaches for
    // the embedded copy. The other limitation is unchanged: a disk run cannot prove the embedded
    // copy is absent, only that it was not the thing read, and AC2 measures the bytes.
    register_tree_assets(&mut app);
    app.insert_resource(scene_source);
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
const DEFAULT_TERRAIN_SUBDIV: u32 = 4;

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
            ..Default::default()
        })
        .insert_resource(ClearColor(night_lighting().sky));
    if args.headless {
        app.insert_resource(HeadlessRequested);
    }
    app.insert_resource(TerrainSubdivision(
        args.subdiv.unwrap_or(DEFAULT_TERRAIN_SUBDIV),
    ));
    // MOVED OUT OF `run()`, where it was the project's own named antipattern: the flag parsed,
    // validated and then reached the app from a place no test could drive, so deleting the two
    // lines left the entire suite green. It belongs with the other resources the extracted builder
    // stands up.
    if let Some(path) = args.perf_log.clone() {
        eprintln!("gui perf-log: recording to {}", path.display());
        // The asset label is READ from the resource rather than recomputed from `args`: deciding
        // the source twice is exactly how a client reports one tree and reads another.
        let assets = app
            .world()
            .get_resource::<crate::project::SceneSource>()
            .cloned()
            .unwrap_or_default()
            .label();
        app.insert_resource(
            crate::perf::PerfLog::new(path).with_run(crate::perf::RunProvenance {
                build: crate::BUILD_SHA.to_string(),
                assets,
                // Recorded as the number rather than left blank, so the line never omits the fact
                // that decides how much geometry the frametime beside it was paying for.
                subdiv: args.subdiv.unwrap_or(DEFAULT_TERRAIN_SUBDIV),
                // Headless has no window and therefore no present mode. A windowed run takes
                // Bevy's default, which is `PresentMode::Fifo` -- vsync ON, and the reason a
                // frametime from the seat can be measuring the monitor rather than the scene.
                vsync: !args.headless,
            }),
        );
    }
    // UNCONDITIONAL, and inserted BEFORE `client_systems`/`projection_systems` so their
    // idempotent `init_resource` finds it already there. Unconditional because a wiring step that
    // only runs when a flag is present is the Milestone 2 defect class this file's own doc
    // comments catalogue: with no `--lights-off` this inserts exactly `Default`, so the absent
    // flag and the present one take the SAME path and neither can rot while the other is tested.
    app.insert_resource(LightingToggles::with_off(&args.lights_off));
    app.insert_resource(crate::clock::ClockPin(
        args.clock
            .or(args.capture.as_ref().map(|_| crate::clock::BOOT_HOUR)),
    ));
    if args.clock.is_some() {
        app.insert_resource(crate::clock::ExplicitClock);
    }
    app.insert_resource(EffectsOff::with_off(&args.fx_off));
    app.insert_resource(LightsSteady(args.lights_steady));
    app.insert_resource(crate::command::StaticWorld(args.static_world));
    app.insert_resource(crate::command::StaticWorldPause::new(args.static_world));
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
        }
        .with_static_world(args.static_world);
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
        .init_resource::<crate::project::DwarfHeadings>()
        .add_systems(Startup, (setup_slice_readout, setup_lighting_readout))
        .add_systems(
            Update,
            (
                ingest_messages,
                slice_controls,
                reconcile_projection,
                blend_projection,
                flicker_projection,
                crate::project::hold_walk_phase_in_static_world,
                // Chained after `blend_projection` deliberately: that is the sole writer of
                // `WalkPhase`, so reading it earlier in the same frame would drive every dwarf
                // from the previous tick's movement.
                crate::project::start_dwarf_walk,
                crate::project::drive_dwarf_walk,
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
    app.init_resource::<crate::clock::ClockPin>();
    app.init_resource::<crate::project::LastRimSky>();
    app.init_resource::<LightsSteady>();
    app.init_resource::<EffectsOff>();
    app.add_systems(
        Update,
        (apply_lighting_toggles, update_lighting_readout)
            .chain()
            .after(ProjectionSet),
    );
    app.add_systems(
        PostUpdate,
        (update_clock_sky, crate::project::update_rim_for_sky).chain(),
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
        .init_resource::<crate::command::SimPaused>()
        .init_resource::<crate::command::StaticWorld>()
        .init_resource::<crate::command::StaticWorldPause>()
        .init_resource::<ButtonInput<bevy::input::mouse::MouseButton>>()
        .init_resource::<DesignateMode>()
        .init_resource::<DragMode>()
        .init_resource::<DragAnchor>()
        .init_resource::<LightingToggles>()
        .init_resource::<LastCameraReadout>()
        .init_resource::<crate::pick::SelectedDwarf>()
        .add_message::<MouseMotion>()
        .add_message::<MouseWheel>();
    app.add_systems(
        Startup,
        (
            setup_camera,
            setup_night_lighting,
            setup_fog_volume,
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
            effect_controls,
            sync_haze_light.after(effect_controls),
            update_fog_from_camera,
            crate::perf::mark_perf_frame_on_key,
            fall_snow,
            // `Update`, not `Startup`: the tick the pause is SENT at is what makes the frozen
            // world reproducible, and a `Startup` command is sent before the client has heard a
            // single delta. `confirm_` runs after, reading the DAEMON's reported speed rather
            // than the client's belief about what it asked for.
            crate::command::pause_static_world,
            crate::command::confirm_static_world_pause.after(crate::command::pause_static_world),
        ),
    )
    // Ordered, not merely registered. `select_dwarf` must read the rig AFTER `camera_controls`
    // has moved it, or a click is judged against last frame's framing; `frame_selected_dwarf`
    // must run after `ProjectionSet`, because `blend_entities` inside it is the sole writer of
    // the dwarf's drawn position and that position is what gets centred. Both stay in `Update`
    // so the camera Transform they write is propagated this frame rather than trailing by one —
    // moving them beside `designation_input` in `PostUpdate` would put them after
    // `TransformSystems::Propagate` and the followed dwarf would lag the camera.
    .add_systems(
        Update,
        (crate::pick::select_dwarf, crate::pick::frame_selected_dwarf)
            .chain()
            .after(camera_controls)
            .after(ProjectionSet),
    )
    // The readout reads the rig every one of these three write. Bevy does NOT order a conflicting
    // read/write pair by declaration order, and in an unordered tuple the edge layer reproduced
    // reader-before-writer EVERY frame in a standalone Bevy 0.19 crate — so `C` printed the
    // framing of the frame before the one the operator was looking at. An instrument that
    // misreports by a frame is still an instrument that misreports.
    .add_systems(
        Update,
        camera_readout
            .after(camera_controls)
            .after(crate::pick::frame_selected_dwarf),
    )
    .add_systems(
        PostUpdate,
        (
            apply_scripted_input.after(TransformSystems::Propagate),
            // After propagation, so focus reads THIS frame's camera. From `Update` it read the
            // previous frame's `GlobalTransform`: a dwarf click drew one frame focused about 60
            // units out while the camera already stood about 20 from him.
            update_dof_from_camera.after(TransformSystems::Propagate),
            update_pick.after(apply_scripted_input),
            sync_hover_highlight.after(update_pick),
            designation_input.after(update_pick),
            sync_drag_preview.after(designation_input),
            // Before `send_commands`, so a space press reaches the daemon on the same frame it is
            // read rather than the next one. The `.before` is EXPLICIT: this comment claimed the
            // ordering for three stories while the schedule only constrained `toggle_pause`
            // against `update_pick`, so the press actually reached the socket a frame late.
            crate::command::toggle_pause
                .after(update_pick)
                .before(send_commands),
            crate::command::step_speed
                .after(crate::command::toggle_pause)
                .before(send_commands),
            // Before `send_commands`, so the hand-back reaches the socket on the frame the
            // exit is requested rather than never.
            crate::command::restore_speed_on_exit.before(send_commands),
            send_commands.after(designation_input),
            update_designate_hint.after(designation_input),
        ),
    )
    // `Last`, so the row describes a frame that has actually been drawn.
    .add_systems(bevy::app::Last, record_perf_frame);
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

/// The diagnostic overlay is simply ON for an interactive run (Wolf, 2026-09-22: "FPS could be on
/// all the time now also .. does not harm"). It had a toggle on F3, which cost a key on a row that
/// had run out of them. Captures are unaffected: `force_capture_overlay_off` turns it off whenever
/// `--capture` is given, on the windowed and headless paths alike, so no measured frame can carry
/// it.
fn overlay_config_on() -> FpsOverlayConfig {
    let mut config = FpsOverlayConfig {
        enabled: true,
        ..Default::default()
    };
    config.frame_time_graph_config.enabled = true;
    config
}

struct Args {
    port: u16,
    clock: Option<f32>,
    capture: Option<PathBuf>,
    frames: u32,
    expect_work: bool,
    static_world: bool,
    slice_level: Option<i32>,
    distance: Option<f32>,
    camera: Option<CameraStart>,
    cursor: Option<Vec2>,
    at_tick: Option<u64>,
    drag: Option<ScriptedDragSpec>,
    headless: bool,
    subdiv: Option<u32>,
    lights_off: Vec<LightSource>,
    fx_off: Vec<CameraEffect>,
    lights_steady: bool,
    /// `--assets <dir>`: read glTF scenes from this directory instead of the embedded blobs.
    /// Dev-only, absolute, and never a default.
    assets: Option<PathBuf>,
    /// `--perf-log <path>`: append one CSV row per frame, to be read after the run.
    perf_log: Option<PathBuf>,
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
///
/// DERIVED, not restated: `capture::CAPTURE_SIZE` is the same contract, and a windowed capture is
/// now REFUSED unless it matches. Two literals could disagree, and the headless side would keep
/// passing while the vehicle silently measured a different frame.
const HEADLESS_SIZE: (u32, u32) = crate::capture::CAPTURE_SIZE;

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

/// The `AssetPlugin` for this run — extracted so a test can read the decision it makes.
///
/// It was inline in `run()`, and the test that claimed to pin it asserted `args.assets.is_some()`
/// instead: re-testing the PARSER while naming the watcher. The mutation table caught it —
/// forcing `watch_for_changes_override` to `Some(true)` changed nothing the test could see.
fn asset_plugin_for(assets: Option<&Path>) -> AssetPlugin {
    AssetPlugin {
        // Absolute, so `get_base_path().join(..)` resolves to exactly this directory. Checked at
        // parse time.
        file_path: assets.map_or_else(
            || AssetPlugin::default().file_path,
            |dir| dir.display().to_string(),
        ),
        // `file_watcher` is compiled in, and Bevy then defaults watching ON. With no `--assets`
        // there is nothing on disk to watch, so every headless test in the gate would pay for a
        // `notify` thread that can never fire. Explicit in BOTH directions rather than inherited.
        watch_for_changes_override: Some(assets.is_some()),
        ..Default::default()
    }
}

fn parse_args() -> anyhow::Result<Args> {
    parse_args_from(std::env::args_os().skip(1))
}

/// `--version` prints the build stamp and exits, WITHOUT connecting to a daemon or opening a window.
///
/// It exists for the pwsh launcher (`scripts/launch-gui.ps1`, M2-7 / issue #46), which must compare
/// the running binary's commit against the checkout it is about to serve as `--assets`. The stamp is
/// already printed at startup, but reading it there means starting the client, parsing stderr and
/// killing it — a race, on the one check whose whole job is to be trustworthy. Handled before
/// `parse_args` so it works on a machine with no daemon at all.
fn print_version_and_exit_if_asked() -> bool {
    if std::env::args_os().skip(1).any(|arg| arg == "--version") {
        println!("gui build {}", crate::BUILD_SHA);
        return true;
    }
    false
}

fn parse_args_from(args: impl IntoIterator<Item = OsString>) -> anyhow::Result<Args> {
    let mut port = protocol::DEFAULT_PORT;
    let mut capture = None;
    let mut frames = None;
    let mut expect_work = false;
    let mut static_world = false;
    let mut slice_level = None;
    let mut distance = None;
    let mut camera = None;
    let mut cursor = None;
    let mut at_tick = None;
    let mut drag = None;
    let mut headless = false;
    let mut subdiv = None;
    let mut lights_off = Vec::new();
    let mut fx_off = Vec::new();
    let mut lights_steady = false;
    let mut assets = None;
    let mut perf_log = None;
    let mut clock = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--capture" {
            let path = args.next().context("--capture requires a path")?;
            capture = Some(PathBuf::from(path));
        } else if arg == "--clock" {
            let value = args.next().context("--clock requires 0 <= hour < 24")?;
            let hour = value.to_string_lossy().parse::<f32>().ok();
            clock = Some(match hour {
                Some(hour) if hour.is_finite() && (0.0..24.0).contains(&hour) => hour,
                _ => bail!("--clock requires 0 <= hour < 24"),
            });
        } else if arg == "--frames" {
            let value = args.next().context("--frames requires a positive count")?;
            frames = Some(
                value
                    .to_string_lossy()
                    .parse()
                    .context("invalid --frames count")?,
            );
        } else if arg == "--static-world" {
            // For capturing a PAUSED world. The motion instrument exists to catch a client that
            // has stopped updating, and a deliberately still world is a false positive for it.
            // Explicit and never a default, so it cannot silently disable the guard on a run that
            // was supposed to be moving.
            static_world = true;
        } else if arg == "--assets" {
            // Dev-only disk loading, so an authored asset can be iterated on without a
            // cross-compile. ABSOLUTE ONLY: Bevy resolves a relative `AssetPlugin::file_path`
            // against `get_base_path()` -- BEVY_ASSET_ROOT, then CARGO_MANIFEST_DIR, then the
            // EXE'S OWN DIRECTORY -- so a relative path quietly means something different on the
            // Windows vehicle than it does here. An absolute path replaces that base outright
            // (`Path::join`), which is the whole mechanism. Refusing is better than resolving
            // against a base the operator never stated.
            let dir = PathBuf::from(args.next().context("--assets requires a directory")?);
            if !dir.is_absolute() {
                bail!(
                    "--assets requires an ABSOLUTE directory; got {}",
                    dir.display()
                );
            }
            assets = Some(dir);
        } else if arg == "--perf-log" {
            // Off unless asked, like --capture and --static-world. A run that was not asked to
            // measure itself writes no file and pays nothing.
            perf_log = Some(PathBuf::from(
                args.next().context("--perf-log requires a path")?,
            ));
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
        } else if arg == "--camera" {
            let value = args
                .next()
                .context("--camera requires yaw,pitch,distance,fx,fy,fz")?;
            camera = Some(parse_camera(value)?);
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
        } else if arg == "--lights-off" {
            let value = args
                .next()
                .context("--lights-off requires a comma-separated source list")?;
            for name in value.to_string_lossy().split(',') {
                lights_off.push(LightSource::from_name(name.trim())?);
            }
        } else if arg == "--fx-off" {
            let value = args
                .next()
                .context("--fx-off requires a comma-separated effect list")?;
            for name in value.to_string_lossy().split(',') {
                fx_off.push(CameraEffect::from_name(name.trim())?);
            }
        } else if arg == "--lights-steady" {
            lights_steady = true;
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
    // NOTE: deliberately NO `--camera requires --capture` gate. `--distance` has one because it
    // only ever existed to pin a capture; `--camera` is how the operator flies the live seat to a
    // framing and reads it back, so gating it on --capture would break its primary use.
    if camera.is_some() && distance.is_some() {
        // `setup_camera` places the whole framing and THEN overwrites the distance from
        // `CaptureDistance`, so the pasted line's yaw, pitch and focus survive while its zoom is
        // silently clobbered — an operator who pastes a readout onto a command line that already
        // carries `--distance` gets a framing the line does not describe. Verified in review:
        // `--camera 0.7,0.45,4,64,64,9` panics on the close zoom, and the same command plus
        // `--distance 90` succeeds. Bail rather than pick a winner, exactly as `--cursor` and
        // `--drag` do: the readout line carries a distance of its own and is the save format.
        bail!(
            "--camera and --distance are mutually exclusive; the --camera line carries its own distance"
        );
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
        clock,
        capture,
        frames: frames.unwrap_or(DEFAULT_AT_TICK_FRAME_BUDGET),
        expect_work,
        static_world,
        slice_level,
        distance,
        camera,
        cursor,
        at_tick,
        drag,
        headless,
        subdiv,
        lights_off,
        fx_off,
        lights_steady,
        assets,
        perf_log,
    })
}

/// An operator-chosen opening framing, from `--camera yaw,pitch,distance,fx,fy,fz`.
///
/// One type doing double duty as the parsed argument and the resource `setup_camera` reads, so
/// there is no second shape to keep in step.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct CameraStart {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub focus: Vec3,
}

fn parse_camera(value: OsString) -> anyhow::Result<CameraStart> {
    let value = value.to_string_lossy();
    let parts = value.split(',').collect::<Vec<_>>();
    let [yaw, pitch, distance, fx, fy, fz] = parts.as_slice() else {
        bail!("invalid --camera; expected yaw,pitch,distance,fx,fy,fz");
    };
    let yaw: f32 = yaw.parse().context("invalid --camera yaw")?;
    let pitch: f32 = pitch.parse().context("invalid --camera pitch")?;
    let distance: f32 = distance.parse().context("invalid --camera distance")?;
    let fx: f32 = fx.parse().context("invalid --camera focus x")?;
    let fy: f32 = fy.parse().context("invalid --camera focus y")?;
    let fz: f32 = fz.parse().context("invalid --camera focus z")?;
    if ![yaw, pitch, distance, fx, fy, fz]
        .iter()
        .all(|value| value.is_finite())
    {
        bail!("invalid --camera; every value must be finite");
    }
    Ok(CameraStart {
        yaw,
        pitch,
        distance,
        focus: Vec3::new(fx, fy, fz),
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
    if let Some(camera) = args.camera {
        app.insert_resource(camera);
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
    start: Option<Res<CameraStart>>,
    headless: Option<Res<HeadlessRequested>>,
    effects_off: Option<Res<EffectsOff>>,
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
    if let Some(start) = start {
        rig.place(start.yaw, start.pitch, start.distance, start.focus);
    }
    if let Some(distance) = distance {
        rig.distance = distance.0.clamp(4.0, 500.0);
    }
    let (fog_start, fog_end) = fog_falloff(rig.distance);
    let transform = rig.transform();
    let dof = depth_of_field(transform.translation, &rig);
    let camera = commands
        .spawn((
            Camera3d::default(),
            Msaa::Off,
            Exposure { ev100: 10.5 },
            Fxaa::default(),
            ScreenSpaceAmbientOcclusion::default(),
            Bloom::default(),
            dof,
            volumetric_fog(),
            Projection::Perspective(PerspectiveProjection {
                fov: BOOT_VERTICAL_FOV,
                ..Default::default()
            }),
            transform,
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
    // The removal loop stays behind the `--fx-off` check purely to leave the default path as it
    // was: `apply_effect` would rebuild depth of field from BOOT framing over the rig-aware
    // component spawned above. MEASURED 2026-09-22, that is survivable either way --
    // `update_dof_from_camera` corrects it in `PostUpdate` of this same first update, so no test
    // that looks after `app.update()` can tell the two apart, and no frame is drawn wrongly focused.
    // `sync_prepasses` always runs: the camera spawns with every effect on, so the prepasses must
    // match whatever set survives this loop rather than rely on AO's `require` to supply them.
    let all_on = EffectsOff::default();
    let effects = effects_off.as_deref().unwrap_or(&all_on);
    if effects_off.is_some() {
        for effect in CameraEffect::ALL {
            apply_effect(&mut commands, camera, effect, !effects.is_off(effect));
        }
    }
    sync_prepasses(&mut commands, camera, effects);
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
        VolumetricLight,
        SunLight,
        ClientLocal,
    ));
}

fn setup_fog_volume(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let density_texture = images.add(fog_density_ramp_image());
    commands.spawn((
        FogVolume {
            density_factor: FOG_DENSITY_FACTOR,
            density_texture: Some(density_texture),
            light_intensity: night_lighting().haze_light_intensity,
            ..Default::default()
        },
        Transform::from_xyz(64.0, 18.0, -64.0).with_scale(Vec3::new(160.0, 48.0, 160.0)),
        ClientLocal,
    ));
}

#[derive(Component)]
struct SunLight;

#[derive(Component)]
pub struct SliceReadout;

#[derive(Component)]
pub struct LightingReadout;

fn lighting_readout(toggles: &LightingToggles, effects_off: &EffectsOff) -> String {
    // Printed in KEY ORDER: F1/F2 belong to bevy_dev_tools and F3 marks the perf log, then the
    // effects F4..F7, then the lights F8..F12, each block widest-acting first.
    let mut entries = CameraEffect::ALL
        .into_iter()
        .map(|effect| {
            format!(
                "{}{} {}",
                match effect.key() {
                    Some(KeyCode::F4) => "F4 ",
                    Some(KeyCode::F5) => "F5 ",
                    Some(KeyCode::F6) => "F6 ",
                    Some(KeyCode::F7) => "F7 ",
                    // An effect with no key still reports its state; it just cannot be toggled here.
                    None => "",
                    Some(_) => unreachable!("the fixed effect keys are F4 through F7"),
                },
                effect.name(),
                if effects_off.is_off(effect) {
                    "off"
                } else {
                    "on"
                }
            )
        })
        .collect::<Vec<_>>();
    entries.extend(LightSource::ALL.into_iter().map(|source| {
        format!(
            "{} {} {}",
            match source.key() {
                KeyCode::F8 => "F8",
                KeyCode::F9 => "F9",
                KeyCode::F10 => "F10",
                KeyCode::F11 => "F11",
                KeyCode::F12 => "F12",
                _ => unreachable!("the fixed lighting keys are F8 through F12"),
            },
            source.name(),
            if toggles.enabled(source) { "on" } else { "off" }
        )
    }));
    entries.join("  ")
}

fn setup_lighting_readout(
    mut commands: Commands,
    toggles: Res<LightingToggles>,
    effects_off: Res<EffectsOff>,
) {
    commands.spawn((
        Text::new(lighting_readout(&toggles, &effects_off)),
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
    effects_off: Res<EffectsOff>,
    mut readout: Query<&mut Text, With<LightingReadout>>,
) {
    if !toggles.is_changed() && !effects_off.is_changed() {
        return;
    }
    let text = lighting_readout(&toggles, &effects_off);
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

fn effect_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut effects_off: ResMut<EffectsOff>,
    cameras: Query<bevy::prelude::Entity, With<CameraRig>>,
) {
    for effect in CameraEffect::ALL {
        let Some(key) = effect.key() else {
            continue;
        };
        if !keys.just_pressed(key) {
            continue;
        }
        effects_off.toggle(effect);
        for camera in &cameras {
            apply_effect(&mut commands, camera, effect, !effects_off.is_off(effect));
            sync_prepasses(&mut commands, camera, &effects_off);
        }
    }
}

/// Applies one of the fixed effects at both creation and live-toggle sites.
///
/// `remove_with_requires` is deliberately never used: it breaks the renderer's sync closure.
/// AO owns its prepasses; Bloom deliberately leaves `Hdr` installed for a marginal comparison.
fn apply_effect(
    commands: &mut Commands,
    camera: bevy::prelude::Entity,
    effect: CameraEffect,
    on: bool,
) {
    let mut camera = commands.entity(camera);
    match (effect, on) {
        (CameraEffect::Fxaa, true) => {
            camera.insert(Fxaa::default());
        }
        (CameraEffect::Fxaa, false) => {
            camera.remove::<Fxaa>();
        }
        (CameraEffect::AmbientOcclusion, true) => {
            camera.insert(ScreenSpaceAmbientOcclusion::default());
        }
        (CameraEffect::AmbientOcclusion, false) => {
            camera.remove::<ScreenSpaceAmbientOcclusion>();
        }
        // Plain removal deliberately preserves Bloom's required Hdr component.
        (CameraEffect::Bloom, true) => {
            camera.insert(Bloom::default());
        }
        (CameraEffect::Bloom, false) => {
            camera.remove::<Bloom>();
        }
        (CameraEffect::Dof, true) => {
            camera.insert(depth_of_field_for_boot_camera());
        }
        (CameraEffect::Dof, false) => {
            camera.remove::<DepthOfField>();
        }
        (CameraEffect::Haze, true) => {
            camera.insert(volumetric_fog());
        }
        (CameraEffect::Haze, false) => {
            camera.remove::<VolumetricFog>();
        }
    };
}

/// Puts the prepasses on the camera that the currently-enabled effects actually sample.
///
/// The prepasses are SHARED, and only `ScreenSpaceAmbientOcclusion` declares them
/// (`#[require(DepthPrepass, NormalPrepass)]`). Depth is kept for `DepthOfField` and
/// `VolumetricFog` as INSURANCE, not as a proven dependency: in the Bevy 0.19.0 source both bind
/// `ViewDepthTexture`, the main depth buffer (`dof/mod.rs:766-830`, `volumetric_fog/render.rs:
/// 285-372`), and neither names a prepass. The seat symptom this was written for turned out to be
/// the F1/F2 key collision. Until #121 settles it at the seat, `--fx-off ao` still pays for a depth
/// pass that may be read by nothing, so AC8's AO cost delta may be understated by that pass.
/// Ownership is computed from the whole effect set here rather than guessed at any one toggle site.
fn sync_prepasses(
    commands: &mut Commands,
    camera: bevy::prelude::Entity,
    effects_off: &EffectsOff,
) {
    let on = |effect| !effects_off.is_off(effect);
    let ambient_occlusion = on(CameraEffect::AmbientOcclusion);
    // Depth has three consumers; normals have exactly one.
    let depth = ambient_occlusion || on(CameraEffect::Dof) || on(CameraEffect::Haze);
    let mut camera = commands.entity(camera);
    if depth {
        camera.insert(DepthPrepass);
    } else {
        camera.remove::<DepthPrepass>();
    }
    if ambient_occlusion {
        camera.insert(NormalPrepass);
    } else {
        camera.remove::<NormalPrepass>();
    }
}

/// Puts `VolumetricLight` on the sun only while the haze is on.
///
/// Removing `VolumetricFog` from the camera is NOT enough to turn the haze off live. Bevy 0.19's
/// `extract_volumetric_fog` (`bevy_pbr/src/volumetric_fog/render.rs:240`) only ever INSERTS the
/// component on the render-world camera, and nothing syncs its removal, so the render world kept
/// drawing the fog after F4 while the readout said `haze off`. Its one cleanup path fires when no
/// light carries `VolumetricLight`, so that marker is what the toggle has to move.
///
/// What that path actually clears matters. It visits only cameras that STILL carry
/// `VolumetricFog` in the main world, and `apply_effect` has already removed it, so the
/// render-world camera KEEPS its fog components and pipelines. The fog stops drawing because the
/// same branch strips every `FogVolume` from the render world, and the fog is drawn only inside a
/// volume. So F4-off leaves stale render state that `--fx-off haze` never creates. Keep the
/// `FogVolume` path working; it is what switches the haze off. The marker only feeds the fog
/// shader; the sun's own lighting and shadows do not read it.
fn sync_haze_light(
    mut commands: Commands,
    effects_off: Res<EffectsOff>,
    sun: Query<(bevy::prelude::Entity, Has<VolumetricLight>), With<SunLight>>,
) {
    let haze = !effects_off.is_off(CameraEffect::Haze);
    for (entity, has) in &sun {
        if haze && !has {
            commands.entity(entity).insert(VolumetricLight);
        } else if !haze && has {
            commands.entity(entity).remove::<VolumetricLight>();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_lighting_toggles(
    toggles: Res<LightingToggles>,
    mirror: Res<MirrorResource>,
    pin: Res<crate::clock::ClockPin>,
    mut ambient: Query<&mut AmbientLight, With<Camera3d>>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<SunLight>>,
    mut points: Query<(
        &crate::project::ProjectedLight,
        &mut bevy::prelude::PointLight,
    )>,
    assets: Option<Res<crate::project::ProjectionAssets>>,
    mut materials: Option<ResMut<bevy::prelude::Assets<bevy::prelude::StandardMaterial>>>,
) {
    let hour = crate::clock::current_hour(&mirror, &pin);
    let (direction, color, illuminance) = crate::atmosphere::key_at(hour);
    let lighting = lighting_at(hour);
    for mut light in &mut ambient {
        light.brightness = if toggles.enabled(LightSource::Ambient) {
            lighting.ambient_brightness
        } else {
            0.0
        };
    }
    for (mut light, mut transform) in &mut sun {
        light.color = color;
        light.illuminance = if toggles.enabled(LightSource::Sun) {
            illuminance
        } else {
            0.0
        };
        *transform = Transform::from_translation(Vec3::ZERO).looking_to(direction, Vec3::Y);
    }
    for (kind, mut light) in &mut points {
        if !point_light_enabled(&toggles, kind.0) {
            light.intensity = 0.0;
        }
    }
    // The emissive FACE is a second thing the same source owns, baked at spawn from
    // `light_properties`. Switching only the point light leaves the emitter glowing, which is
    // what "everything off and the campfire still lights" looked like on the vehicle.
    let (Some(assets), Some(materials)) = (assets, materials.as_deref_mut()) else {
        return;
    };
    for (kind, handle) in assets.emissive_materials() {
        let Some(mut material) = materials.get_mut(&handle) else {
            continue;
        };
        material.emissive = if point_light_enabled(&toggles, kind) {
            light_properties(kind).color.to_linear()
        } else {
            bevy::color::LinearRgba::BLACK
        };
    }
}

#[allow(clippy::too_many_arguments)]
fn update_clock_sky(
    mirror: Res<MirrorResource>,
    pin: Res<crate::clock::ClockPin>,
    mut clear: Option<ResMut<ClearColor>>,
    mut cameras: Query<
        (
            &mut AmbientLight,
            &mut DistanceFog,
            Option<&mut VolumetricFog>,
        ),
        With<Camera3d>,
    >,
    mut volumes: Query<&mut FogVolume>,
    mut moon: Query<
        (&mut Transform, &mut bevy::prelude::Visibility),
        With<crate::atmosphere::Moon>,
    >,
    handles: Option<Res<crate::atmosphere::AtmosphereMaterials>>,
    mut materials: Option<ResMut<Assets<bevy::prelude::StandardMaterial>>>,
) {
    let hour = crate::clock::current_hour(&mirror, &pin);
    let lighting = lighting_at(hour);
    if let Some(clear) = clear.as_deref_mut()
        && clear.0 != lighting.sky
    {
        clear.0 = lighting.sky;
    }
    for (mut ambient, mut fog, haze) in &mut cameras {
        if ambient.color != lighting.ambient {
            ambient.color = lighting.ambient;
        }
        if fog.color != lighting.sky {
            fog.color = lighting.sky;
        }
        if let Some(mut haze) = haze {
            let intensity = 0.1 * lighting.ambient_brightness / 80.0;
            if haze.ambient_color != lighting.ambient {
                haze.ambient_color = lighting.ambient;
            }
            if haze.ambient_intensity != intensity {
                haze.ambient_intensity = intensity;
            }
        }
    }
    for mut volume in &mut volumes {
        if volume.light_intensity != lighting.haze_light_intensity {
            volume.light_intensity = lighting.haze_light_intensity;
        }
    }
    let moon_position = crate::atmosphere::moon_position(hour);
    for (mut transform, mut visibility) in &mut moon {
        let wanted = if moon_position.is_some() {
            bevy::prelude::Visibility::Inherited
        } else {
            bevy::prelude::Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
        if let Some(position) = moon_position
            && transform.translation != position
        {
            transform.translation = position;
        }
    }
    let (Some(handles), Some(materials)) = (handles, materials.as_deref_mut()) else {
        return;
    };
    let weight = day_weight(hour);
    let star_color = if weight == 0.0 {
        lighting.star
    } else if weight == 1.0 {
        lighting.sky
    } else {
        mix_color(night_lighting().star, lighting.sky, weight)
    };
    if materials
        .get(&handles.star)
        .is_some_and(|material| material.base_color != star_color)
    {
        materials.get_mut(&handles.star).unwrap().base_color = star_color;
    }
    // The moon fades into the sky exactly as the stars do.
    let moon_color = mix_color(crate::appearance::moon_color(), lighting.sky, weight);
    if materials
        .get(&handles.moon)
        .is_some_and(|material| material.base_color != moon_color)
    {
        materials.get_mut(&handles.moon).unwrap().base_color = moon_color;
    }
    let aurora_color = Color::srgba(1.0, 1.0, 1.0, 1.0 - weight);
    if materials
        .get(&handles.aurora)
        .is_some_and(|material| material.base_color != aurora_color)
    {
        materials.get_mut(&handles.aurora).unwrap().base_color = aurora_color;
    }
}

fn point_light_enabled(toggles: &LightingToggles, kind: protocol::LightKind) -> bool {
    match kind {
        protocol::LightKind::Campfire => toggles.enabled(LightSource::Campfire),
        protocol::LightKind::Lantern => toggles.enabled(LightSource::Lanterns),
        protocol::LightKind::Torch => toggles.enabled(LightSource::Torches),
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
            // Below the fps overlay, which Bevy pins to the origin at font size 32. The two must be
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

/// The camera's own readout key. Prints the framing AND records it, because a `println!` alone
/// is not reachable by a test: the instrument rule here is that an evidence channel must itself
/// be tested, and an untested one manufactures false evidence rather than merely missing true
/// evidence. `LastCameraReadout` is that seam and nothing else reads it.
fn camera_readout(
    keys: Res<ButtonInput<KeyCode>>,
    cameras: Query<&CameraRig>,
    mut last: ResMut<LastCameraReadout>,
) {
    if !keys.just_pressed(KeyCode::KeyC) {
        return;
    }
    for rig in &cameras {
        let line = camera_readout_line(rig);
        println!("{line}");
        last.0 = Some(line);
    }
}

/// The last line [`camera_readout`] printed, so a test can assert what the operator saw.
#[derive(Resource, Default, Debug)]
pub struct LastCameraReadout(pub Option<String>);

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
    mouse: Res<ButtonInput<MouseButton>>,
    mut motions: MessageReader<MouseMotion>,
    mut wheels: MessageReader<MouseWheel>,
    time: Res<Time>,
    mut cameras: Query<(&mut CameraRig, &mut Transform)>,
) {
    const ORBIT_RATE: f32 = 1.2;
    const ZOOM_RATE: f32 = 60.0;
    const MOUSE_ORBIT_RATE: f32 = 0.01;
    // Cells of ground per pixel of cursor motion AT THE BOOT ZOOM; `pan_scale` carries it to
    // every other zoom. Reachable at last: this rate was dead until 10.10's review, because
    // shift is what SELECTS pan and the pan branch then multiplied by the shift multiplier
    // unconditionally, so 0.48 was the only pan speed the seat ever felt.
    const MOUSE_PAN_RATE: f32 = 0.12;
    // One notch was 1.0, which needed ~86 of them to cross the boot-to-closest range. Raised
    // to 6.0 on Wolf's verdict from the seat (2026-09-17): 14 notches boot-to-closest, and 4
    // with shift held.
    const WHEEL_ZOOM_STEP: f32 = 6.0;
    const SHIFT_MULTIPLIER: f32 = 4.0;
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let multiplier = if shift { SHIFT_MULTIPLIER } else { 1.0 };
    // Pan takes its multiplier from CONTROL, not shift. Shift is the modifier that SELECTS pan,
    // so `multiplier` inside the pan branch was always 4.0 and AC3's "shift multiplies the rate"
    // was unobservable there. Ctrl is Wolf's ruling (2026-09-17) over Alt, which most Linux
    // window managers take for themselves before the client ever sees the drag.
    let pan_multiplier =
        if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
            SHIFT_MULTIPLIER
        } else {
            1.0
        };
    let key_scale = time.delta_secs() * multiplier;
    let yaw = (keys.pressed(KeyCode::KeyD) as i8 - keys.pressed(KeyCode::KeyA) as i8) as f32
        * ORBIT_RATE
        * key_scale;
    let pitch = (keys.pressed(KeyCode::KeyW) as i8 - keys.pressed(KeyCode::KeyS) as i8) as f32
        * ORBIT_RATE
        * key_scale;
    let zoom = (keys.pressed(KeyCode::KeyE) as i8 - keys.pressed(KeyCode::KeyQ) as i8) as f32
        * ZOOM_RATE
        * key_scale;
    let motion = motions.read().map(|motion| motion.delta).sum::<Vec2>();
    let wheel = wheels.read().map(|wheel| wheel.y).sum::<f32>();
    for (mut rig, mut transform) in &mut cameras {
        if mouse.pressed(MouseButton::Middle) {
            if shift {
                let rate = MOUSE_PAN_RATE * rig.pan_scale() * pan_multiplier;
                rig.pan(-motion.x * rate, motion.y * rate);
            } else {
                rig.orbit(
                    yaw - motion.x * MOUSE_ORBIT_RATE * multiplier,
                    pitch - motion.y * MOUSE_ORBIT_RATE * multiplier,
                );
            }
        } else {
            rig.orbit(yaw, pitch);
        }
        rig.zoom(zoom + wheel * WHEEL_ZOOM_STEP * multiplier);
        *transform = rig.transform();
    }
}

/// `<` / `>` use the comma and period keys today; the wheel belongs to the camera.
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

fn depth_of_field(camera_translation: Vec3, rig: &CameraRig) -> DepthOfField {
    DepthOfField {
        focal_distance: dof_focal_distance(camera_translation, rig),
        aperture_f_stops: DOF_APERTURE_F_STOPS,
        max_depth: DOF_MAX_DEPTH,
        ..Default::default()
    }
}

/// The transform and aim point are the framing's authoritative geometry. `rig.distance` is the
/// orbit radius, not the focus distance: at boot it is 90 while the camp is about 61.7 away.
fn dof_focal_distance(camera_translation: Vec3, rig: &CameraRig) -> f32 {
    camera_translation.distance(world_to_render_f32(rig.focus))
}

fn depth_of_field_for_boot_camera() -> DepthOfField {
    let rig = CameraRig::new([64, 64, 9]);
    depth_of_field(rig.transform().translation, &rig)
}

fn volumetric_fog() -> VolumetricFog {
    VolumetricFog {
        ambient_color: night_lighting().ambient,
        // Bevy's 0.1 default matches AmbientLight::default().brightness = 80.0.
        ambient_intensity: 0.1 * night_lighting().ambient_brightness / 80.0,
        ..Default::default()
    }
}

fn fog_density_ramp_pixels() -> Vec<u8> {
    (0..FOG_DENSITY_RAMP_HEIGHT)
        .map(|row| {
            let v = row as f32 / (FOG_DENSITY_RAMP_HEIGHT - 1) as f32;
            let density = if v <= FOG_DENSITY_RAMP_FULL_TO {
                1.0
            } else if v >= FOG_DENSITY_RAMP_ZERO_BY {
                0.0
            } else {
                1.0 - (v - FOG_DENSITY_RAMP_FULL_TO)
                    / (FOG_DENSITY_RAMP_ZERO_BY - FOG_DENSITY_RAMP_FULL_TO)
            };
            (density * 255.0).round() as u8
        })
        .collect()
}

fn fog_density_ramp_image() -> Image {
    let mut image = Image::new(
        Extent3d {
            width: 1,
            height: FOG_DENSITY_RAMP_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D3,
        fog_density_ramp_pixels(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}

/// The point depth of field must focus, in render space, or `None` for the rig's own aim point.
///
/// A SELECTED DWARF is the subject, and he is not where the aim point is. `frame_selected_dwarf`
/// centres him through `CameraRig::frame_render_point`, which writes the focus offset from him by
/// the composition push -- about 7.3 units at `SELECT_DISTANCE`. At boot there is no selection and
/// the subject IS `world_to_render(rig.focus)`, which is why every boot-framing figure in this
/// story's record stands unchanged.
fn dof_subject(
    selected: &crate::pick::SelectedDwarf,
    drawn: &crate::pick::DrawnEntities,
) -> Option<Vec3> {
    let id = selected.0?;
    drawn
        .iter()
        .find(|(marker, _)| marker.0 == id)
        .map(|(_, transform)| transform.translation)
}

fn update_dof_from_camera(
    selected: Res<crate::pick::SelectedDwarf>,
    drawn: crate::pick::DrawnEntities,
    mut cameras: Query<(&GlobalTransform, &CameraRig, &mut DepthOfField)>,
) {
    let subject = dof_subject(&selected, &drawn);
    for (transform, rig, mut dof) in &mut cameras {
        dof.focal_distance = match subject {
            Some(point) => transform.translation().distance(point),
            None => dof_focal_distance(transform.translation(), rig),
        };
    }
}

/// One CSV row per frame, if `--perf-log` asked for it.
///
/// Runs in `Last` so the row describes a frame that has actually been drawn, and reads the same
/// queries the startup instrument counts -- one source for both, so the log and the console cannot
/// disagree about what was on screen.
fn record_perf_frame(
    log: Option<ResMut<crate::perf::PerfLog>>,
    terrain_tiles: Query<&TerrainTile>,
    terrain_chunks: Query<&crate::project::TerrainChunk>,
    trees: Query<&crate::project::TreeMesh>,
    dwarves: Query<&bevy::world_serialization::WorldAssetRoot, With<WorldProjected>>,
    work: Option<Res<ProjectionWork>>,
) {
    let Some(mut log) = log else {
        return;
    };
    log.record(
        std::time::Instant::now(),
        crate::perf::FrameCounts {
            terrain: terrain_tiles.iter().count() + terrain_chunks.iter().count(),
            trees: trees.iter().count(),
            dwarves: dwarves.iter().count(),
            // The DRAINED count, not the live set. `reconcile_projection` empties `dirty_tiles`
            // in `Update` and this runs in `Last`, so the set is always empty by now -- reading it
            // pinned this column to 0 on every row and left AC10's steady/edit split unable to
            // fire. `the_perf_row_reports_the_tiles_the_reconcile_actually_drained` runs both
            // systems in one schedule, which is the only place the ordering is visible.
            dirty_tiles: work.map_or(0, |work| work.drained_tiles),
        },
    );
}

/// The only GUI system that reads protocol message types; it mutates only the mirror.
/// Everything one delta changes on the client, in one place so the headless tests drive the SAME
/// path as the socket. Their helper used to restate these lines by hand, and the copy silently
/// missed the facing record when it was added.
pub fn apply_wire_delta(
    mirror: &mut Mirror,
    work: &mut ProjectionWork,
    clock: &mut TickClock,
    headings: &mut crate::project::DwarfHeadings,
    delta: Delta,
) {
    mirror.apply_delta(delta);
    headings.record(mirror);
    clock.observe_tick(mirror.tick());
    work.dirty_tiles
        .extend(mirror.changes().tiles.iter().copied());
}

/// The snapshot twin of [`apply_wire_delta`].
pub fn apply_wire_snapshot(
    mirror: &mut Mirror,
    work: &mut ProjectionWork,
    clock: &mut TickClock,
    headings: &mut crate::project::DwarfHeadings,
    snapshot: Snapshot,
) -> Result<(), client_core::MirrorError> {
    mirror.apply_snapshot(snapshot)?;
    work.snapshot = true;
    work.dirty_tiles.clear();
    headings.clear();
    clock.reset(mirror.tick());
    Ok(())
}

fn ingest_messages(
    receiver: Option<Res<IngestReceiver>>,
    mut mirror: ResMut<MirrorResource>,
    mut work: ResMut<ProjectionWork>,
    mut clock: ResMut<TickClock>,
    mut headings: ResMut<crate::project::DwarfHeadings>,
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
            Ok(Ok(WireMessage::Snapshot(snapshot))) => match apply_wire_snapshot(
                &mut mirror.0,
                &mut work,
                &mut clock,
                &mut headings,
                *snapshot,
            ) {
                Ok(()) => {}
                Err(error) => {
                    // A frozen window with no diagnostic is worse than a loud exit; the
                    // sibling client bails on this same condition.
                    eprintln!("could not apply server snapshot: {error}");
                    exit.write(AppExit::error());
                }
            },
            Ok(Ok(WireMessage::Delta(delta))) => {
                apply_wire_delta(&mut mirror.0, &mut work, &mut clock, &mut headings, *delta);
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
    headings: Res<crate::project::DwarfHeadings>,
    time: Res<Time>,
    mut projected: Query<
        (
            &WorldProjected,
            &mut Transform,
            Option<&mut crate::project::WalkPhase>,
        ),
        Without<TerrainTile>,
    >,
) {
    blend_entities(
        &mirror.0,
        &headings,
        &mut clock,
        time.delta_secs(),
        &mut projected,
    );
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
    // The perf row is written in `Last`, after this drain, so the count has to survive it.
    work.drained_tiles = changes.len();
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
    steady: Res<LightsSteady>,
    mut lights: Query<(
        &WorldProjected,
        &crate::project::ProjectedLight,
        &mut bevy::prelude::PointLight,
    )>,
) {
    const STEADY_FLICKER_SECONDS: f32 = 0.0;
    let seconds = if steady.0 {
        STEADY_FLICKER_SECONDS
    } else {
        time.elapsed_secs()
    };
    flicker_lights(seconds, &mut lights);
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
        ffi::OsString,
        io::{BufRead, BufReader},
        sync::{Mutex, mpsc},
        time::Duration,
    };

    use bevy::{
        anti_alias::fxaa::Fxaa,
        app::{App, Update},
        camera::{CameraProjection, RenderTargetInfo},
        dev_tools::fps_overlay::FpsOverlayConfig,
        input::{ButtonInput, mouse::MouseButton},
        prelude::{
            Camera, Camera3d, GlobalTransform, KeyCode, Text, Time, UVec2, Vec3, Window, With,
        },
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

    /// The interactive overlay is ON, and nothing but a capture may turn it off.
    ///
    /// It used to have a toggle on F3; that key now marks the perf log, so if this default is ever
    /// flipped back to `false` there is no longer any way to get the overlay back at the seat and
    /// nothing else would notice. The capture half is the test below: the two belong together,
    /// because "always on" is only safe while every measured frame is still guaranteed to be
    /// without it.
    #[test]
    fn the_interactive_overlay_is_on_and_has_no_key_to_restore_it() {
        let config = super::overlay_config_on();
        assert!(config.enabled, "the fps overlay must start enabled");
        assert!(
            config.frame_time_graph_config.enabled,
            "the frame-time graph must start enabled alongside it"
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
        // `TransformPlugin` because `MinimalPlugins` has none: without it every `GlobalTransform`
        // stays at the origin, and a test reading the camera's position through it certifies
        // nothing about where the camera is.
        app.add_plugins((bevy::MinimalPlugins, bevy::transform::TransformPlugin))
            .init_resource::<bevy::input::ButtonInput<bevy::prelude::KeyCode>>()
            .init_resource::<bevy::input::ButtonInput<bevy::input::mouse::MouseButton>>()
            .init_resource::<bevy::asset::Assets<bevy::prelude::Mesh>>()
            .init_resource::<bevy::asset::Assets<bevy::prelude::StandardMaterial>>()
            .init_resource::<bevy::asset::Assets<bevy::image::Image>>()
            .init_resource::<FpsOverlayConfig>();
        super::configure_client_app(&mut app, mirror, receiver, writer, parsed);
        (app, sender, server)
    }

    #[test]
    fn clock_flag_accepts_hours_and_rejects_out_of_range_values() {
        for (text, expected) in [("0", 0.0), ("12", 12.0), ("23.99", 23.99)] {
            let args = super::parse_args_from(["--clock".into(), text.into()]).unwrap();
            assert_eq!(
                args.clock,
                Some(expected),
                "--clock {text} must be retained"
            );
        }
        for text in ["24", "-1", "x"] {
            let result = super::parse_args_from(["--clock".into(), text.into()]);
            assert!(
                result.is_err_and(|error| error.to_string().contains("0 <= hour < 24")),
                "--clock {text} must name the allowed range"
            );
        }
    }

    #[test]
    fn clock_pin_reaches_the_live_app_and_capture_defaults_to_boot() {
        let (seat, _sender, _server) = configured_app(&[]);
        assert_eq!(
            seat.world()
                .get_resource::<crate::clock::ClockPin>()
                .map(|pin| pin.0),
            Some(None),
            "an unpinned seat must follow the daemon tick"
        );
        let (mut capture, _sender, _server) =
            configured_app(&["--capture", "/tmp/clock-test.png", "--frames", "60"]);
        assert_eq!(
            capture
                .world()
                .get_resource::<crate::clock::ClockPin>()
                .map(|pin| pin.0),
            Some(Some(crate::clock::BOOT_HOUR)),
            "a capture without --clock must pin the boot hour"
        );
        let (noon, _sender, _server) = configured_app(&["--clock", "12"]);
        assert_eq!(
            noon.world()
                .get_resource::<crate::clock::ClockPin>()
                .map(|pin| pin.0),
            Some(Some(12.0)),
            "--clock must reach the app resource"
        );
        let late_snapshot = Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Solid(protocol::Material::Stone), Tile::Empty],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 1_000,
        };
        capture
            .world_mut()
            .resource_mut::<super::MirrorResource>()
            .0 = Mirror::from_snapshot(late_snapshot.clone()).unwrap();
        assert_eq!(
            crate::clock::current_hour(
                capture.world().resource::<super::MirrorResource>(),
                capture.world().resource::<crate::clock::ClockPin>()
            ),
            crate::clock::BOOT_HOUR,
            "a running capture must stay at 22 after the wire advances 1,000 ticks"
        );
        let (late_capture, _sender, _server) = configured_app_with_snapshot(
            &["--capture", "/tmp/clock-test.png", "--frames", "60"],
            late_snapshot,
        );
        assert_eq!(
            crate::clock::current_hour(
                late_capture.world().resource::<super::MirrorResource>(),
                late_capture.world().resource::<crate::clock::ClockPin>()
            ),
            crate::clock::BOOT_HOUR,
            "a capture must hold the boot hour even when the wire tick is 1,000"
        );
    }

    #[test]
    fn an_unpinned_seat_follows_two_wire_snapshots_one_hour_apart() {
        let snapshot = |tick| Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Solid(protocol::Material::Stone), Tile::Empty],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick,
        };
        let (early, _sender, _server) = configured_app_with_snapshot(&[], snapshot(0));
        let (late, _sender, _server) = configured_app_with_snapshot(&[], snapshot(1_000));
        let hour = |app: &App| {
            crate::clock::current_hour(
                app.world().resource::<super::MirrorResource>(),
                app.world().resource::<crate::clock::ClockPin>(),
            )
        };
        assert_eq!(hour(&late) - hour(&early), 1.0);
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
    fn configured_camera_disables_msaa_on_the_live_rig() {
        let (mut app, _sender, _server) = configured_app(&[]);
        app.update();

        let msaa = app
            .world_mut()
            .query_filtered::<&bevy::render::view::Msaa, With<CameraRig>>()
            .single(app.world())
            .expect("startup must spawn the one live camera rig");

        assert_eq!(*msaa, bevy::render::view::Msaa::Off);
    }

    #[test]
    fn configured_camera_carries_the_chosen_ev100_on_the_live_rig() {
        let (mut app, _sender, _server) = configured_app(&[]);
        app.update();

        let exposure = app
            .world_mut()
            .query_filtered::<&bevy::camera::Exposure, With<CameraRig>>()
            .single(app.world())
            .expect("startup must spawn the one live camera rig");

        assert_eq!(exposure.ev100, 10.5);
    }

    #[test]
    fn configured_camera_starts_with_fxaa() {
        let (mut app, _sender, _server) = configured_app(&[]);
        app.update();
        let fxaa = app
            .world_mut()
            .query_filtered::<&bevy::anti_alias::fxaa::Fxaa, With<CameraRig>>()
            .single(app.world())
            .expect("the default live camera must carry FXAA");
        assert!(fxaa.enabled);
    }

    /// A snapshot identical to `configured_app`'s but stopped at a chosen tick, so a test can put
    /// the world on either side of `--static-world`'s pause tick.
    fn snapshot_at_tick(tick: u64, speed: Speed) -> Snapshot {
        Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Solid(protocol::Material::Stone), Tile::Empty],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed,
            tick,
        }
    }

    fn read_one_command(server: &std::net::TcpStream) -> String {
        use std::io::{BufRead, BufReader};
        let mut line = String::new();
        let _ = BufReader::new(server).read_line(&mut line);
        line.trim().to_string()
    }

    /// `--static-world` must reach the DAEMON, not merely the capture's assertion switch -- and it
    /// must reach it at a CHOSEN tick rather than whichever one the scheduler allowed.
    ///
    /// Issue #105. The flag is documented as "freeze the sim so two captures differ only by what
    /// you changed" and for three stories it froze nothing: it silenced the capture's motion
    /// assertions while the daemon kept ticking and the dwarves kept walking, and `capture.rs`
    /// announced "the simulation is paused" over a world that was not. The camp window's measured
    /// "flicker" floor turned out to be mostly those dwarves and their lanterns.
    ///
    /// A test asserting the RESOURCE would have passed throughout that entire period, because the
    /// resource was always set correctly -- it just went nowhere. So this reads the SOCKET: the
    /// only thing in this system that can stop a dwarf is a command the daemon actually receives.
    ///
    /// The review that followed found the remaining half: the pause was queued in `Startup`, so it
    /// landed at whatever tick won the race and two captures of one binary froze different worlds.
    /// That half was first addressed by holding the command back until a chosen tick -- which made
    /// the freeze point REPRODUCIBLE but not EXACT, because the daemon still applied it on arrival
    /// and kept ticking in the meantime. Measured afterwards, one fresh daemon per capture: idle
    /// froze at tick 39-40, under CPU load at 80, and against a daemon given a 15 s head start at
    /// 225, swinging camp near-white by 0.3636 pp -- 2.2x the signal the AC4 guard measures (#111).
    ///
    /// So the contract INVERTED, and this test with it. The command now goes out as EARLY as
    /// possible and names the tick it wants (`at_tick`); the daemon holds it and stops the world
    /// on exactly that tick. Sending early is no longer the defect -- **failing to name the tick
    /// is**, because an unscheduled command is applied wherever the round trip puts it. That is
    /// what the wire assertion below pins.
    #[test]
    fn static_world_schedules_its_pause_at_a_chosen_tick_over_the_wire() {
        // EARLY IS NOW CORRECT: the command only has to ARRIVE before its tick, so it is sent on
        // the first update rather than held back. What makes the freeze point a decision instead
        // of a race outcome is `at_tick`, not the moment of sending.
        let (mut app, _sender, server) =
            configured_app_with_snapshot(&["--static-world"], snapshot_at_tick(0, Speed::Normal));
        app.update();
        assert_eq!(
            read_one_command(&server),
            r#"{"type":"set_speed","speed":"paused","at_tick":120}"#,
            "--static-world must SCHEDULE its pause: a command without at_tick is applied on \
             arrival, which is latency-bound and freezes a different world under load (#111)"
        );
        assert!(
            !app.world()
                .resource::<crate::command::StaticWorldPause>()
                .landed(),
            "nothing may be considered landed before the daemon has reported a pause"
        );
        assert!(
            app.world().resource::<crate::command::SimPaused>().0,
            "the client's pause state must agree with what it asked the daemon for"
        );

        // The control carries as much weight as the case. This flag defaults OFF, and a pause
        // leaking into a run that never asked for one would freeze every other capture in the
        // suite -- silently, and looking exactly like a stall.
        let (mut running, _sender, server) =
            configured_app_with_snapshot(&[], snapshot_at_tick(8, Speed::Normal));
        running.update();
        assert!(
            read_one_command(&server).is_empty(),
            "without --static-world the daemon must be sent nothing"
        );
        assert!(
            !running.world().resource::<crate::command::SimPaused>().0,
            "without --static-world the client must not believe the sim is paused"
        );
    }

    /// The freeze is confirmed from the DAEMON's report, never from the client's own belief.
    ///
    /// `SimPaused` records what we ASKED for and was correct throughout the entire period the
    /// world was not actually pausing, so it cannot be the oracle. `Mirror::speed()` is the
    /// daemon's own speed off the wire -- which `SimPaused`'s doc comment wrongly claimed did not
    /// exist, for as long as the defect did.
    #[test]
    fn static_world_confirms_the_pause_from_the_daemons_own_report() {
        let (mut app, sender, _server) =
            configured_app_with_snapshot(&["--static-world"], snapshot_at_tick(8, Speed::Normal));
        app.update();
        assert!(
            !app.world()
                .resource::<crate::command::StaticWorldPause>()
                .landed(),
            "asking is not landing: with the daemon still reporting Normal, the capture must wait"
        );

        sender
            .send(Ok(WireMessage::Delta(Box::new(protocol::Delta {
                msg_type: MessageType::Delta,
                tick: 9,
                tiles: Vec::new(),
                entities: Vec::new(),
                designations: Vec::new(),
                zones: Vec::new(),
                items: Vec::new(),
                speed: Speed::Paused,
            }))))
            .unwrap();
        // Two frames: one ingests the delta into the mirror, the next reads the mirror's speed.
        app.update();
        app.update();
        assert_eq!(
            app.world()
                .resource::<crate::command::StaticWorldPause>()
                .landed_at(),
            Some(9),
            "the landing tick must be the one the DAEMON reported it stopped at"
        );
    }

    /// The pause timeout is WALL CLOCK. It was 600 frames, which a fast GPU runs through long
    /// before the daemon's 10 Hz tick reaches 120 -- the vehicle refused every `--static-world`
    /// capture at tick 93. Many fast frames must not end the run; enough real time must.
    #[test]
    fn static_world_pause_timeout_counts_time_not_frames() {
        let (mut app, _sender, _server) =
            configured_app_with_snapshot(&["--static-world"], snapshot_at_tick(8, Speed::Normal));
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_millis(1),
        ));
        for _ in 0..700 {
            app.update();
        }
        assert!(
            app.should_exit().is_none(),
            "700 frames in 0.7 s is a fast GPU, not a daemon that failed to pause"
        );

        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs(60),
        ));
        app.update();
        assert!(
            app.should_exit().is_some_and(|exit| exit.is_error()),
            "a pause still unreported after 60 s must fail the run loudly"
        );
    }

    /// Space must not quietly resume a run that asked for a frozen world.
    ///
    /// `toggle_pause` took no notice of the flag and queued `SetSpeed { Normal }` on any press, so
    /// one keystroke at the seat broke the guarantee every figure in the capture is measured
    /// against, with nothing said.
    #[test]
    fn space_cannot_resume_a_static_world_run() {
        let (mut app, _sender, server) =
            configured_app_with_snapshot(&["--static-world"], snapshot_at_tick(8, Speed::Normal));
        app.update();
        assert_eq!(
            read_one_command(&server),
            r#"{"type":"set_speed","speed":"paused","at_tick":120}"#
        );

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();
        assert!(
            read_one_command(&server).is_empty(),
            "Space must send the daemon nothing under --static-world"
        );
        assert!(
            app.world().resource::<crate::command::SimPaused>().0,
            "Space must not clear the client's pause state under --static-world"
        );

        // Control: without the flag, Space still works. A guard that disabled the key outright
        // would pass the assertions above and break the seat.
        let (mut seat, _sender, server) =
            configured_app_with_snapshot(&[], snapshot_at_tick(8, Speed::Normal));
        seat.update();
        seat.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        seat.update();
        assert_eq!(
            read_one_command(&server),
            r#"{"type":"set_speed","speed":"paused"}"#,
            "without --static-world, Space must still pause the daemon"
        );
    }

    #[test]
    fn speed_keys_step_from_the_daemon_speed_and_ignore_the_ends() {
        for (speed, key, expected) in [
            (Speed::Paused, KeyCode::Equal, Some("normal")),
            (Speed::Normal, KeyCode::NumpadAdd, Some("fast")),
            (Speed::Fast, KeyCode::Equal, Some("fast2x")),
            (Speed::Fast2x, KeyCode::NumpadAdd, Some("fast4x")),
            (Speed::Fast4x, KeyCode::Equal, None),
            (Speed::Fast4x, KeyCode::Minus, Some("fast2x")),
            (Speed::Fast2x, KeyCode::NumpadSubtract, Some("fast")),
            (Speed::Fast, KeyCode::Minus, Some("normal")),
            (Speed::Normal, KeyCode::NumpadSubtract, Some("paused")),
            (Speed::Paused, KeyCode::Minus, None),
        ] {
            let (mut app, _sender, server) =
                configured_app_with_snapshot(&[], snapshot_at_tick(8, speed));
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            app.update();
            let actual = read_one_command(&server);
            let expected = expected.map_or(String::new(), |speed| {
                format!(r#"{{"type":"set_speed","speed":"{speed}"}}"#)
            });
            assert_eq!(actual, expected, "{key:?} from {speed:?}");
        }
    }

    #[test]
    fn speed_keys_refuse_static_world_and_space_uses_the_new_pause_state() {
        let (mut frozen, _sender, server) =
            configured_app_with_snapshot(&["--static-world"], snapshot_at_tick(8, Speed::Normal));
        frozen.update();
        assert_eq!(
            read_one_command(&server),
            r#"{"type":"set_speed","speed":"paused","at_tick":120}"#
        );
        frozen
            .world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Equal);
        frozen.update();
        assert_eq!(
            read_one_command(&server),
            "",
            "speed key must refuse --static-world"
        );

        let (mut seat, _sender, server) =
            configured_app_with_snapshot(&[], snapshot_at_tick(8, Speed::Paused));
        seat.update();
        // The local flag must start in the same paused state that the daemon reported.
        seat.world_mut()
            .resource_mut::<crate::command::SimPaused>()
            .0 = true;
        seat.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Equal);
        seat.update();
        assert_eq!(
            read_one_command(&server),
            r#"{"type":"set_speed","speed":"normal"}"#
        );
        {
            let mut keys = seat.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::Equal);
            keys.clear();
            keys.press(KeyCode::Space);
        }
        seat.update();
        assert_eq!(
            read_one_command(&server),
            r#"{"type":"set_speed","speed":"paused"}"#,
            "Space after + must pause the newly running sim"
        );
    }

    /// A `--static-world` run hands the daemon back at Normal when it ends.
    ///
    /// The daemon's speed is ONE global shared by every client, so before this a `--static-world`
    /// run left it frozen for good: the next client to connect without the flag rendered a dead
    /// world, failed its motion assertions, exited 101 and wrote no PNG -- blaming the dwarves.
    #[test]
    fn a_static_world_run_hands_the_daemon_back_at_normal() {
        let (mut app, _sender, server) =
            configured_app_with_snapshot(&["--static-world"], snapshot_at_tick(8, Speed::Normal));
        app.update();
        assert_eq!(
            read_one_command(&server),
            r#"{"type":"set_speed","speed":"paused","at_tick":120}"#
        );

        app.world_mut().write_message(bevy::app::AppExit::Success);
        app.update();
        assert_eq!(
            read_one_command(&server),
            r#"{"type":"set_speed","speed":"normal"}"#,
            "a --static-world run must hand the daemon back, or every later client is frozen"
        );

        // Control: a run that never asked for a pause must not send one on the way out either.
        let (mut plain, _sender, server) =
            configured_app_with_snapshot(&[], snapshot_at_tick(8, Speed::Normal));
        plain.update();
        plain.world_mut().write_message(bevy::app::AppExit::Success);
        plain.update();
        assert!(
            read_one_command(&server).is_empty(),
            "a run that never paused must send nothing on exit"
        );
    }

    #[test]
    fn dof_focus_is_derived_from_the_camera_transform_and_tracks_framing() {
        let boot = CameraRig::new([64, 64, 9]);
        let boot_distance = super::dof_focal_distance(boot.transform().translation, &boot);
        assert!(
            (boot_distance - 61.7).abs() <= 1.0,
            "boot focus should be about 61.7, not the rig orbit radius: {boot_distance}"
        );
        assert_eq!(
            boot.distance, 90.0,
            "fixture must retain the wrong orbit radius"
        );

        let mut orbit = boot;
        orbit.yaw += 0.4;
        let orbit_distance = super::dof_focal_distance(orbit.transform().translation, &orbit);
        assert!(
            (orbit_distance - boot_distance).abs() > 0.01,
            "an orbit must recompute the transform-to-focus distance"
        );

        let mut zoom = boot;
        zoom.distance = 40.0;
        let zoom_distance = super::dof_focal_distance(zoom.transform().translation, &zoom);
        assert!(
            (zoom_distance - boot_distance).abs() > 10.0,
            "a zoom must recompute the transform-to-focus distance"
        );
    }

    #[test]
    fn fog_density_ramp_fades_above_the_skyline_instead_of_ending_at_a_box_face() {
        let ramp = super::fog_density_ramp_pixels();
        assert_eq!(ramp.len(), 64);
        assert_eq!(ramp[0], 255, "the valley floor must retain full density");
        assert_eq!(ramp[34], 255, "the ramp must stay full through v=0.54");
        assert!(
            ramp[43] > 0 && ramp[43] < 255,
            "the fade band must contain a positive intermediate value"
        );
        assert_eq!(ramp[53], 0, "density must be zero above v=0.83");
    }

    #[test]
    fn volumetric_fog_ambient_intensity_tracks_the_scenes_ambient_budget() {
        let fog = super::volumetric_fog();
        assert_eq!(fog.ambient_intensity, 1.875);
        assert_eq!(
            fog.ambient_color,
            crate::appearance::night_lighting().ambient
        );
    }

    #[test]
    fn fx_off_reaches_the_live_camera_and_rejects_unknown_effects() {
        let camera_effects = |app: &mut App| {
            (
                app.world_mut()
                    .query_filtered::<&Fxaa, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&super::ScreenSpaceAmbientOcclusion, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&super::Bloom, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&super::DepthOfField, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&super::VolumetricFog, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
            )
        };
        let (mut default, _sender, _server) = configured_app(&[]);
        default.update();
        assert_eq!(camera_effects(&mut default), (1, 1, 1, 1, 1));
        for (name, expected) in [
            ("fxaa", (0, 1, 1, 1, 1)),
            ("ao", (1, 0, 1, 1, 1)),
            ("bloom", (1, 1, 0, 1, 1)),
            ("dof", (1, 1, 1, 0, 1)),
            ("haze", (1, 1, 1, 1, 0)),
            ("fxaa, ao,bloom,dof,haze", (0, 0, 0, 0, 0)),
        ] {
            let (mut disabled, _sender, _server) = configured_app(&["--fx-off", name]);
            disabled.update();
            assert_eq!(
                camera_effects(&mut disabled),
                expected,
                "--fx-off {name:?} must remove only its named live camera components"
            );
        }
        // The REQUIRED components, which a plain `remove` leaves behind. `--fx-off ao` must drop
        // every pass nothing else samples, or "ao off" pays a full-scene GPU pass for no one and
        // AC8's vehicle cost delta over-states what turning AO off actually buys. Since 11.2 that
        // is the NORMAL prepass only: depth of field and haze read the DEPTH prepass and declare
        // nothing, so it is shared and stays. AO's marginal cost is measured with them on, and a
        // pass the other two would pay anyway is not part of it. Bloom is the deliberate
        // opposite: its required `Hdr` must SURVIVE, because `--fx-off bloom` is the control
        // AC4's marginal figures are measured against.
        let prepasses = |app: &mut App| {
            (
                app.world_mut()
                    .query_filtered::<&bevy::core_pipeline::prepass::DepthPrepass, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&bevy::core_pipeline::prepass::NormalPrepass, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&bevy::camera::Hdr, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
            )
        };
        assert_eq!(
            prepasses(&mut default),
            (1, 1, 1),
            "AO's prepasses and bloom's Hdr must all be present by default"
        );
        let (mut no_ao, _sender, _server) = configured_app(&["--fx-off", "ao"]);
        no_ao.update();
        assert_eq!(
            prepasses(&mut no_ao),
            (1, 0, 1),
            "--fx-off ao must drop the normal prepass only, keep the depth prepass dof and haze \
             sample, and leave bloom's Hdr alone"
        );
        let (mut bare, _sender, _server) = configured_app(&["--fx-off", "ao,dof,haze"]);
        bare.update();
        assert_eq!(
            prepasses(&mut bare),
            (0, 0, 1),
            "with all three depth consumers off the depth prepass must go too"
        );
        let (mut no_bloom, _sender, _server) = configured_app(&["--fx-off", "bloom"]);
        no_bloom.update();
        assert_eq!(
            prepasses(&mut no_bloom),
            (1, 1, 1),
            "--fx-off bloom must LEAVE Hdr in place: it is the control AC4 is measured against"
        );

        let error =
            match super::parse_args_from([OsString::from("--fx-off"), OsString::from("taa")]) {
                Ok(_) => panic!("unknown effects must fail"),
                Err(error) => error,
            };
        assert_eq!(
            error.to_string(),
            "unknown effect \"taa\"; expected fxaa, ao, bloom, dof, or haze"
        );
    }

    #[test]
    fn lights_steady_reaches_the_live_flicker_system() {
        let snapshot = Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Solid(protocol::Material::Stone), Tile::Empty],
            entities: vec![protocol::Entity {
                id: 1,
                kind: protocol::EntityKind::Campfire,
                pos: [0, 0, 0],
                state: protocol::JobState::Idle,
                light: Some(protocol::LightKind::Campfire),
            }],
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        };
        let intensity = |app: &mut App| {
            app.world_mut()
                .query::<(&crate::project::ProjectedLight, &bevy::prelude::PointLight)>()
                .iter(app.world())
                .find_map(|(kind, light)| {
                    (kind.0 == protocol::LightKind::Campfire).then_some(light.intensity)
                })
                .expect("the fixture must spawn the campfire point light")
        };
        let step = |app: &mut App| {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs(1));
            app.update();
        };

        let (mut default, _sender, _server) = configured_app_with_snapshot(&[], snapshot.clone());
        default.update();
        let flickering_before = intensity(&mut default);
        step(&mut default);
        let flickering_after = intensity(&mut default);
        assert_ne!(
            flickering_before, flickering_after,
            "without --lights-steady the live flicker system must vary a PointLight across stepped frames"
        );

        let (mut steady, _sender, _server) =
            configured_app_with_snapshot(&["--lights-steady"], snapshot);
        steady.update();
        let steady_before = intensity(&mut steady);
        step(&mut steady);
        let steady_after = intensity(&mut steady);
        assert_eq!(
            steady_before, steady_after,
            "--lights-steady must pin the PointLight intensity the live flicker system writes"
        );
    }

    /// Depth of field and haze sample the depth prepass, but only AO declares it.
    ///
    /// `ScreenSpaceAmbientOcclusion` is `#[require(DepthPrepass, NormalPrepass)]`; `DepthOfField`
    /// and `VolumetricFog` declare NOTHING and simply read whatever depth prepass the camera
    /// happens to carry. Before 11.2 that was harmless -- AO was the only consumer, so "AO off"
    /// could take both prepasses with it. With two more consumers on the camera, the unconditional
    /// removal pulls the depth buffer out from under effects that are still switched ON, and the
    /// readout keeps reporting "F1 dof on". Only toggling AO back on restores it, which is exactly
    /// what it looked like from the seat: cycling F1 changes nothing, cycling F11 fixes it.
    ///
    /// Normals stay AO-only: nothing else samples them, so "AO off" must still drop that pass.
    #[test]
    fn turning_ambient_occlusion_off_keeps_the_depth_prepass_dof_and_haze_sample() {
        let prepasses = |app: &mut App| {
            (
                app.world_mut()
                    .query_filtered::<&bevy::core_pipeline::prepass::DepthPrepass, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&bevy::core_pipeline::prepass::NormalPrepass, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
            )
        };
        // `configured_app` runs no input-clearing system, so `just_pressed` is STICKY: clear after
        // every tap or a two-tap sequence silently becomes four toggles.
        let tap = |app: &mut App, key: KeyCode| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            app.update();
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release(key);
            input.clear();
        };

        let (mut app, _sender, _server) = configured_app(&[]);
        app.update();
        assert_eq!(
            prepasses(&mut app),
            (1, 1),
            "both prepasses are present while every effect is on"
        );

        tap(&mut app, KeyCode::F7); // ambient occlusion off
        assert_eq!(
            prepasses(&mut app),
            (1, 0),
            "AO off must keep the depth prepass while dof and haze still sample it, and drop \
             only the normal prepass nothing else reads"
        );

        tap(&mut app, KeyCode::F5); // depth of field off
        tap(&mut app, KeyCode::F4); // haze off
        assert_eq!(
            prepasses(&mut app),
            (0, 0),
            "with AO, dof and haze all off nothing samples depth, so the pass must go"
        );

        tap(&mut app, KeyCode::F5); // depth of field back on, AO still off
        assert_eq!(
            prepasses(&mut app),
            (1, 0),
            "re-enabling dof must bring back the depth prepass it reads, without needing AO"
        );

        // The same hole at boot: `--fx-off ao` shipped a depth-less depth of field.
        let (mut boot_no_ao, _sender, _server) = configured_app(&["--fx-off", "ao"]);
        boot_no_ao.update();
        assert_eq!(
            prepasses(&mut boot_no_ao),
            (1, 0),
            "--fx-off ao must leave the depth prepass for dof and haze"
        );
    }

    /// F4 must take `VolumetricLight` off the sun, not only `VolumetricFog` off the camera.
    ///
    /// The camera removal alone left the render world drawing the fog: Bevy never syncs that
    /// removal, and its only cleanup fires when no light is volumetric. Wolf saw it at the seat --
    /// F4 changed nothing, and "haze off" looked like a headless haze-on capture. This test cannot
    /// see the render world, so it pins the one main-world input Bevy's cleanup reads.
    #[test]
    fn the_haze_key_moves_the_suns_volumetric_marker() {
        let volumetric_suns = |app: &mut App| {
            app.world_mut()
                .query_filtered::<(), (With<super::SunLight>, With<super::VolumetricLight>)>()
                .iter(app.world())
                .count()
        };
        let tap = |app: &mut App, key: KeyCode| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            app.update();
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release(key);
            input.clear();
        };

        let (mut app, _sender, _server) = configured_app(&[]);
        app.update();
        assert_eq!(
            volumetric_suns(&mut app),
            1,
            "haze on at boot: the sun lights the fog"
        );
        tap(&mut app, KeyCode::F4);
        app.update();
        assert_eq!(
            volumetric_suns(&mut app),
            0,
            "haze off: with no volumetric light Bevy strips the fog from the render world"
        );
        tap(&mut app, KeyCode::F4);
        app.update();
        assert_eq!(
            volumetric_suns(&mut app),
            1,
            "haze back on: the marker must return"
        );

        let (mut boot_off, _sender, _server) = configured_app(&["--fx-off", "haze"]);
        boot_off.update();
        boot_off.update();
        assert_eq!(
            volumetric_suns(&mut boot_off),
            0,
            "--fx-off haze must agree with F4"
        );
    }

    #[test]
    fn effect_keys_toggle_the_live_camera_and_readout() {
        let camera_effects = |app: &mut App| {
            (
                app.world_mut()
                    .query_filtered::<&Fxaa, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&super::ScreenSpaceAmbientOcclusion, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&super::Bloom, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&super::DepthOfField, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
                app.world_mut()
                    .query_filtered::<&super::VolumetricFog, With<CameraRig>>()
                    .iter(app.world())
                    .count(),
            )
        };
        // F1 and F2 are deliberately absent: bevy_dev_tools owns them (see `CameraEffect::key`).
        // Fxaa has no key at all now, so it appears in the readout without one and cannot be
        // toggled here -- `--fx-off fxaa` is its only control.
        for (key, expected_effects, expected_readout) in [
            (
                KeyCode::F7,
                (1, 0, 1, 1, 1),
                "F4 haze on  F5 dof on  F6 bloom on  F7 ao off  fxaa on  F8 sun on  F9 ambient on  F10 campfire on  F11 torches on  F12 lanterns on",
            ),
            (
                KeyCode::F6,
                (1, 1, 0, 1, 1),
                "F4 haze on  F5 dof on  F6 bloom off  F7 ao on  fxaa on  F8 sun on  F9 ambient on  F10 campfire on  F11 torches on  F12 lanterns on",
            ),
            (
                KeyCode::F5,
                (1, 1, 1, 0, 1),
                "F4 haze on  F5 dof off  F6 bloom on  F7 ao on  fxaa on  F8 sun on  F9 ambient on  F10 campfire on  F11 torches on  F12 lanterns on",
            ),
            (
                KeyCode::F4,
                (1, 1, 1, 1, 0),
                "F4 haze off  F5 dof on  F6 bloom on  F7 ao on  fxaa on  F8 sun on  F9 ambient on  F10 campfire on  F11 torches on  F12 lanterns on",
            ),
        ] {
            let (mut app, _sender, _server) = configured_app(&[]);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            app.update();
            let readout = app
                .world_mut()
                .query_filtered::<&Text, With<super::LightingReadout>>()
                .single(app.world())
                .unwrap()
                .0
                .clone();
            assert_eq!(
                readout, expected_readout,
                "{key:?} must update the live readout"
            );
            assert_eq!(
                camera_effects(&mut app),
                expected_effects,
                "{key:?} must remove only its named live camera component"
            );
        }
    }

    /// No control of ours may sit on a key something else has already claimed.
    ///
    /// This is the check that did not exist when 11.2 put dof and haze on F1 and F2 -- keys
    /// `bevy_dev_tools::render_debug` binds automatically through `DefaultPlugins`. Both handlers
    /// then ran on every press: ours updated the readout, Bevy's cycled a depth/normal debug
    /// overlay over the whole frame. `configured_app` builds on `MinimalPlugins`, which does not
    /// include that plugin, so every key test in this file passed against a keymap that was
    /// unusable at the seat. A test cannot see a collision with a plugin it never loads, so the
    /// reserved keys are written down here instead, with the source that proves them.
    #[test]
    fn the_client_keymap_avoids_keys_other_plugins_have_claimed() {
        // `bevy_dev_tools-0.19.0/src/render_debug.rs:107` and `:110`. Reached through
        // `DefaultPlugins` whenever the `bevy_dev_tools` and `bevy_pbr` features are both on --
        // `bevy_internal-0.19.0/src/default_plugins.rs:95` -- which is this client's build.
        const RESERVED_BY_BEVY: [(KeyCode, &str); 2] = [
            (
                KeyCode::F1,
                "bevy_dev_tools render debug overlay: cycle mode",
            ),
            (
                KeyCode::F2,
                "bevy_dev_tools render debug overlay: cycle opacity",
            ),
        ];

        let mut bound: Vec<(KeyCode, String)> = Vec::new();
        for source in super::LightSource::ALL {
            bound.push((source.key(), format!("light {}", source.name())));
        }
        for effect in super::CameraEffect::ALL {
            if let Some(key) = effect.key() {
                bound.push((key, format!("effect {}", effect.name())));
            }
        }
        // EVERY other key this client binds, with its site. This list is the maintenance burden
        // and it is the point: the first version of this guard listed only the lights, the effects
        // and the perf mark, so it cheerfully accepted haze on F3 -- at the time the fps overlay's
        // toggle key. A guard that knows about only some of the keymap certifies
        // the rest. Anything added with `just_pressed` OR held with `pressed` belongs here.
        for (key, site) in [
            (KeyCode::F3, "perf-log frame mark (perf.rs)"),
            (KeyCode::KeyC, "print the camera readout (ingest.rs)"),
            (KeyCode::Comma, "slice down (ingest.rs)"),
            (KeyCode::Period, "slice up (ingest.rs)"),
            (KeyCode::Space, "pause / resume the sim (command.rs)"),
            (KeyCode::Equal, "step speed faster (command.rs)"),
            (KeyCode::NumpadAdd, "step speed faster (command.rs)"),
            (KeyCode::Minus, "step speed slower (command.rs)"),
            (KeyCode::NumpadSubtract, "step speed slower (command.rs)"),
            (KeyCode::KeyA, "yaw, held (camera_controls)"),
            (KeyCode::KeyD, "yaw, held (camera_controls)"),
            (KeyCode::KeyW, "pitch, held (camera_controls)"),
            (KeyCode::KeyS, "pitch, held (camera_controls)"),
            (KeyCode::KeyE, "zoom, held (camera_controls)"),
            (KeyCode::KeyQ, "zoom, held (camera_controls)"),
            (
                KeyCode::ShiftLeft,
                "rate multiplier / select pan (camera_controls)",
            ),
            (
                KeyCode::ShiftRight,
                "rate multiplier / select pan (camera_controls)",
            ),
            (KeyCode::ControlLeft, "pan multiplier (camera_controls)"),
            (KeyCode::ControlRight, "pan multiplier (camera_controls)"),
            (
                KeyCode::Escape,
                "abort designation / clear selection (designate.rs, pick.rs)",
            ),
            (KeyCode::Digit1, "designate dig (designate.rs)"),
            (KeyCode::Digit2, "designate channel (designate.rs)"),
            (KeyCode::Digit3, "designate stockpile (designate.rs)"),
            (KeyCode::Digit4, "designate clear (designate.rs)"),
        ] {
            bound.push((key, site.to_string()));
        }

        for (key, reserved_for) in RESERVED_BY_BEVY {
            if let Some((_, ours)) = bound.iter().find(|(bound_key, _)| *bound_key == key) {
                panic!(
                    "{key:?} is bound to {ours} but is already taken by {reserved_for}; both \
                     handlers will run on every press and ours will look broken at the seat"
                );
            }
        }

        for i in 0..bound.len() {
            for j in (i + 1)..bound.len() {
                assert_ne!(
                    bound[i].0, bound[j].0,
                    "{} and {} are both bound to {:?}",
                    bound[i].1, bound[j].1, bound[i].0
                );
            }
        }
    }

    /// Re-inserting an effect must rebuild it from the LIVE camera, not from boot framing.
    ///
    /// `apply_effect` built `DepthOfField` from a fresh `CameraRig::new([64, 64, 9])`, so toggling
    /// depth of field off and on again after the operator had moved the camera restored focus to
    /// the BOOT distance rather than the distance to what the camera is now aimed at.
    /// `update_dof_from_camera` then ran in `Update` and repaired it on the FOLLOWING frame, so the
    /// defect rendered exactly one wrongly-focused frame -- a visible pop at the seat, and invisible to any test that only
    /// counts components, which is what `effect_keys_toggle_the_live_camera_and_readout` does.
    #[test]
    fn toggling_dof_back_on_focuses_the_live_camera_not_boot_framing() {
        let (mut app, _sender, _server) = configured_app(&[]);
        app.update();

        // Leave boot framing: zoom in and orbit, the two motions AC3 names.
        {
            let world = app.world_mut();
            let mut rigs = world.query::<&mut CameraRig>();
            let mut rig = rigs.single_mut(world).expect("one camera rig");
            rig.distance = 40.0;
            rig.yaw += 0.7;
        }
        app.update();

        // The oracle is the rig's own geometry, not the camera's `GlobalTransform`: this test
        // must fail if the system reads a camera position that never moved.
        let expected = {
            let world = app.world_mut();
            let mut rigs = world.query::<&CameraRig>();
            let rig = rigs.single(world).expect("one camera rig");
            rig.transform()
                .translation
                .distance(super::world_to_render_f32(rig.focus))
        };
        let boot = super::depth_of_field_for_boot_camera().focal_distance;
        assert!(
            (expected - boot).abs() > 5.0,
            "the zoom must move the focal distance away from boot framing or this test cannot \
             tell them apart: live {expected}, boot {boot}"
        );

        // `configured_app` runs no input-clearing system, so `just_pressed` is STICKY here: an
        // update with a stale press toggles the effect again. Clear it after every tap, or a
        // two-tap sequence silently becomes four toggles.
        let tap = |app: &mut App, key: KeyCode| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            app.update();
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release(key);
            input.clear();
        };
        let count = |app: &mut App| {
            let world = app.world_mut();
            let mut q = world.query_filtered::<&super::DepthOfField, With<CameraRig>>();
            q.iter(world).count()
        };
        tap(&mut app, KeyCode::F5); // depth of field off
        assert_eq!(
            count(&mut app),
            0,
            "the first tap must remove depth of field"
        );
        tap(&mut app, KeyCode::F5); // and back on

        let world = app.world_mut();
        let mut dofs = world.query_filtered::<&super::DepthOfField, With<CameraRig>>();
        let dof = dofs
            .single(world)
            .expect("depth of field is back on the camera");
        assert!(
            (dof.focal_distance - expected).abs() < 1.0,
            "re-enabling depth of field must focus the LIVE camera: focal_distance={} but the \
             camera is {expected} from its aim point (boot framing would read about {})",
            dof.focal_distance,
            super::depth_of_field_for_boot_camera().focal_distance,
        );
    }

    /// `--assets` is a RESOLVER: it decides which of two asset trees the client reads. The
    /// decision must be CONSUMED, not merely parsed, so this asserts the branch-changing path in
    /// both directions -- the load prefix and the reported label move together, and neither moves
    /// when the flag is absent.
    ///
    /// The relative-path refusal is the half that would otherwise rot silently: Bevy resolves a
    /// relative `file_path` against `get_base_path()`, whose last fallback is THE EXE'S OWN
    /// DIRECTORY. A relative `--assets` would therefore mean one directory here and a different
    /// one on the Windows vehicle, and neither would be the one the operator typed.
    #[test]
    fn assets_switches_the_scene_source_and_refuses_a_relative_directory() {
        let default = super::parse_args_from([std::ffi::OsString::from("7451")])
            .expect("no --assets must parse");
        assert_eq!(
            default.assets, None,
            "--assets must never be a silent default"
        );

        let disk = super::parse_args_from([
            std::ffi::OsString::from("7451"),
            std::ffi::OsString::from("--assets"),
            std::ffi::OsString::from("/tmp/frostvein-checkout"),
        ])
        .expect("an absolute --assets must parse");
        assert_eq!(
            disk.assets,
            Some(std::path::PathBuf::from("/tmp/frostvein-checkout"))
        );

        // `Args` is deliberately not `Debug`, so match rather than `expect_err`.
        let refused = match super::parse_args_from([
            std::ffi::OsString::from("7451"),
            std::ffi::OsString::from("--assets"),
            std::ffi::OsString::from("some/relative/dir"),
        ]) {
            Ok(_) => panic!(
                "a relative --assets must stop the run, not resolve against an unstated base"
            ),
            Err(error) => error,
        };
        assert_eq!(
            refused.to_string(),
            "--assets requires an ABSOLUTE directory; got some/relative/dir"
        );

        // The decision is CONSUMED. Hand-written expectations, not derived from the code under
        // test: embedded scenes carry the `embedded://` scheme and disk scenes carry none.
        let embedded = super::SceneSource::Embedded;
        let on_disk = super::SceneSource::Disk(std::path::PathBuf::from("/tmp/frostvein-checkout"));
        assert_eq!(embedded.prefix(), "embedded://");
        assert_eq!(on_disk.prefix(), "");
        assert_eq!(embedded.label(), "embedded");
        assert_eq!(on_disk.label(), "disk:/tmp/frostvein-checkout");
        assert_ne!(
            embedded.label(),
            on_disk.label(),
            "the reported source must MOVE when the source moves; `source=embedded` was a literal"
        );
    }

    /// AC5. `file_watcher` is compiled in, and Bevy then defaults watching ON -- so the cost of a
    /// `notify` thread would land on every headless test in the gate for a capability nothing
    /// exercises. This pins the override in BOTH directions rather than trusting the default.
    ///
    /// IT READS THE PLUGIN, not the parser. The first version of this test asserted
    /// `args.assets.is_some()`, which is the flag going in rather than the decision coming out --
    /// so forcing `watch_for_changes_override` to `Some(true)` left it green. The mutation table
    /// found that; a green test named for the watcher had pinned nothing about the watcher.
    #[test]
    fn the_file_watcher_is_armed_only_when_there_is_a_disk_tree_to_watch() {
        let embedded = super::asset_plugin_for(None);
        assert_eq!(
            embedded.watch_for_changes_override,
            Some(false),
            "with no disk tree the watcher must be explicitly OFF, not left to Bevy's default"
        );
        assert_eq!(
            embedded.file_path,
            bevy::asset::AssetPlugin::default().file_path,
            "no flag must leave the asset root exactly as it ships"
        );

        let dir = std::path::Path::new("/tmp/frostvein-checkout");
        let on_disk = super::asset_plugin_for(Some(dir));
        assert_eq!(
            on_disk.watch_for_changes_override,
            Some(true),
            "a disk tree is the only thing that arms the watcher"
        );
        // Hand-written, not re-derived from the expression under test.
        assert_eq!(on_disk.file_path, "/tmp/frostvein-checkout");
    }

    /// `--lights-off` is what gives `from_name` a caller the shipped binary reaches. Before it,
    /// the "unknown source is refused loudly" clause was satisfied only by the test above calling
    /// the function directly -- a guard with no production path, which is the shape this project
    /// keeps finding. This asserts the whole chain: the flag parses, an unknown name is refused
    /// BY THE PARSER, and the named sources arrive in the resource the keys drive.
    #[test]
    fn lights_off_parses_refuses_an_unknown_source_and_reaches_the_toggles_the_keys_drive() {
        let args = super::parse_args_from([
            std::ffi::OsString::from("7451"),
            std::ffi::OsString::from("--lights-off"),
            std::ffi::OsString::from("sun, campfire ,torches"),
        ])
        .expect("a comma-separated source list must parse, whitespace and all");
        assert_eq!(
            args.lights_off,
            vec![
                super::LightSource::Sun,
                super::LightSource::Campfire,
                super::LightSource::Torches
            ]
        );

        // Refused by the PARSER, not merely by the function: this is the reachable path.
        // `Args` is deliberately not `Debug`, so match rather than `expect_err`.
        let refused = match super::parse_args_from([
            std::ffi::OsString::from("7451"),
            std::ffi::OsString::from("--lights-off"),
            std::ffi::OsString::from("sun,moon"),
        ]) {
            Ok(_) => panic!("an unknown source must stop the run, not be silently dropped"),
            Err(error) => error,
        };
        assert_eq!(
            refused.to_string(),
            "unknown light source \"moon\"; expected sun, campfire, torches, lanterns, or ambient"
        );

        // And the wiring: a flag that parses into a field nothing reads is the inert mechanism.
        let (app, _sender, _server) = configured_app(&["7451", "--lights-off", "sun,ambient"]);
        let toggles = app
            .world()
            .get_resource::<super::LightingToggles>()
            .expect("the toggles resource must exist on the configured app");
        assert!(!toggles.enabled(super::LightSource::Sun));
        assert!(!toggles.enabled(super::LightSource::Ambient));
        assert!(
            toggles.enabled(super::LightSource::Campfire)
                && toggles.enabled(super::LightSource::Torches)
                && toggles.enabled(super::LightSource::Lanterns),
            "only the NAMED sources start dark; the rest are untouched"
        );
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
            super::LightSource::from_name("torches").unwrap(),
            super::LightSource::Torches
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
            "unknown light source \"moon\"; expected sun, campfire, torches, lanterns, or ambient"
        );
    }

    /// AC5's guard, one level DOWN: on the light that is actually installed, not on the pure
    /// function that computes its aim. `the_approved_sun_lights_downward` asserts
    /// `sun_direction()`; nothing asserted what `setup_night_lighting` spawned, so pointing the
    /// spawn at any other transform left every sun test green. That is the
    /// verification-defect-relocates pattern -- close the hole at the formula and it reopens in
    /// what feeds the light. Same hand-written floor, deliberately not derived from
    /// `SUN_ELEVATION_DEGREES`, applied to the entity Bevy actually renders from.
    #[test]
    fn the_installed_sun_entity_aims_downward_onto_the_valley() {
        let (mut app, _sender, _server) = configured_app(&["--clock", "22"]);
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<&bevy::prelude::Transform, With<super::SunLight>>();
        let transforms = query.iter(app.world()).collect::<Vec<_>>();
        assert_eq!(
            transforms.len(),
            1,
            "exactly one sun must be installed; found {}",
            transforms.len()
        );

        let forward = transforms[0].forward().as_vec3();
        assert!(
            (forward.length() - 1.0).abs() < 1e-5,
            "the installed sun's forward must be unit length: {forward:?}"
        );
        assert!(
            forward.y <= crate::atmosphere::APPROVED_DOWNWARD_FLOOR,
            "the INSTALLED sun must travel downward onto the valley; y={} exceeds the approved \
             floor {}",
            forward.y,
            crate::atmosphere::APPROVED_DOWNWARD_FLOOR
        );
    }

    #[test]
    fn clock_drives_the_installed_key_direction_color_and_illuminance() {
        let (mut app, _sender, _server) = configured_app(&["--clock", "12"]);
        app.update();
        let (light, transform) = app
            .world_mut()
            .query_filtered::<(&bevy::prelude::DirectionalLight, &bevy::prelude::Transform), With<super::SunLight>>()
            .single(app.world())
            .unwrap();
        let (direction, color, lux) = crate::atmosphere::key_at(12.0);
        assert!(
            transform.forward().as_vec3().distance(direction) < 1e-6,
            "installed key must aim along the clock direction"
        );
        assert_eq!(light.color, color);
        assert_eq!(light.illuminance, lux);
    }

    #[test]
    fn f8_restores_the_clock_key_at_noon() {
        let (mut app, _sender, _server) = configured_app(&["--clock", "12"]);
        app.update();
        let lux = |app: &mut App| {
            app.world_mut()
                .query_filtered::<&bevy::prelude::DirectionalLight, With<super::SunLight>>()
                .single(app.world())
                .unwrap()
                .illuminance
        };
        let press = |app: &mut App| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::F8);
            app.update();
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::F8);
            keys.clear();
        };
        press(&mut app);
        assert_eq!(lux(&mut app), 0.0);
        press(&mut app);
        assert_eq!(
            lux(&mut app),
            crate::appearance::day_lighting().directional_illuminance,
            "F8-on must restore noon's key budget"
        );
    }

    #[test]
    fn noon_sky_and_distance_fog_share_the_day_colour() {
        let (mut app, _sender, _server) = configured_app(&["--clock", "12"]);
        app.update();
        let day = crate::appearance::day_lighting();
        assert_eq!(
            app.world().resource::<bevy::prelude::ClearColor>().0,
            day.sky
        );
        let fog = app
            .world_mut()
            .query_filtered::<&super::DistanceFog, With<CameraRig>>()
            .single(app.world())
            .unwrap();
        assert_eq!(fog.color, day.sky);
        let ambient = app
            .world_mut()
            .query_filtered::<&bevy::prelude::AmbientLight, With<CameraRig>>()
            .single(app.world())
            .unwrap();
        assert_eq!(ambient.color, day.ambient);
        assert_eq!(ambient.brightness, day.ambient_brightness);
    }

    #[test]
    fn noon_rim_materials_dissolve_toward_the_live_sky() {
        let (mut app, _sender, _server) = configured_app(&["--clock", "12"]);
        app.update();
        let assets = app.world().resource::<crate::project::ProjectionAssets>();
        let materials = app
            .world()
            .resource::<bevy::prelude::Assets<bevy::prelude::StandardMaterial>>();
        let sky = app.world().resource::<bevy::prelude::ClearColor>().0;
        for slot in &assets.terrain {
            let rim = materials
                .get(&slot[crate::appearance::RIM_LEVELS - 1])
                .unwrap();
            assert_eq!(rim.base_color, sky, "every rim target must be the live sky");
        }
        let (mut night, _sender, _server) = configured_app(&["--clock", "22"]);
        night.update();
        let assets = night.world().resource::<crate::project::ProjectionAssets>();
        let materials = night
            .world()
            .resource::<bevy::prelude::Assets<bevy::prelude::StandardMaterial>>();
        let sky = night.world().resource::<bevy::prelude::ClearColor>().0;
        for slot in &assets.terrain {
            let rim = materials
                .get(&slot[crate::appearance::RIM_LEVELS - 1])
                .unwrap();
            assert_eq!(
                rim.base_color, sky,
                "night rim target must equal the live sky exactly"
            );
        }
    }

    #[test]
    fn noon_stars_and_aurora_fade_from_the_live_shared_materials() {
        let (mut app, _sender, _server) = configured_app(&["--clock", "12"]);
        app.update();
        let handles = app
            .world()
            .get_resource::<crate::atmosphere::AtmosphereMaterials>();
        assert!(
            handles.is_some(),
            "star and aurora handles must reach the live app"
        );
        let handles = handles.unwrap();
        let materials = app
            .world()
            .resource::<bevy::prelude::Assets<bevy::prelude::StandardMaterial>>();
        assert_eq!(
            materials.get(&handles.star).unwrap().base_color,
            crate::appearance::day_lighting().sky
        );
        assert_eq!(
            materials
                .get(&handles.aurora)
                .unwrap()
                .base_color
                .to_srgba()
                .alpha,
            0.0
        );
    }

    #[test]
    fn noon_haze_ambient_survives_f4_off_and_on() {
        let (mut app, _sender, _server) = configured_app(&["--clock", "12"]);
        app.update();
        let day = crate::appearance::day_lighting();
        let haze = |app: &mut App| {
            let fog = app
                .world_mut()
                .query_filtered::<&super::VolumetricFog, With<CameraRig>>()
                .single(app.world())
                .unwrap();
            (fog.ambient_color, fog.ambient_intensity)
        };
        assert_eq!(
            haze(&mut app),
            (day.ambient, 0.1 * day.ambient_brightness / 80.0)
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F4);
        app.update();
        assert_eq!(
            app.world_mut()
                .query_filtered::<&super::VolumetricFog, With<CameraRig>>()
                .iter(app.world())
                .count(),
            0
        );
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::F4);
            keys.clear();
        }
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F4);
        app.update();
        assert_eq!(
            haze(&mut app),
            (day.ambient, 0.1 * day.ambient_brightness / 80.0),
            "F4-on must restore the clock's haze ambient"
        );
    }

    /// The night gain and the moon disc follow the clock in the live app. The moon is checked
    /// against the key light the app actually installed, not against `moon_position`.
    #[test]
    fn the_clock_drives_the_haze_gain_and_the_moon() {
        let installed = |app: &mut App| {
            let gain = app
                .world_mut()
                .query::<&super::FogVolume>()
                .single(app.world())
                .unwrap()
                .light_intensity;
            let key = app
                .world_mut()
                .query_filtered::<&bevy::prelude::Transform, With<super::SunLight>>()
                .single(app.world())
                .unwrap()
                .forward()
                .as_vec3();
            let (moon, visibility) = app
                .world_mut()
                .query_filtered::<(&bevy::prelude::Transform, &bevy::prelude::Visibility), With<crate::atmosphere::Moon>>()
                .single(app.world())
                .unwrap();
            (gain, key, moon.translation, *visibility)
        };
        let (mut night, _night_sender, _night_server) = configured_app(&["--clock", "20"]);
        night.update();
        let (gain, key, moon, visibility) = installed(&mut night);
        assert_eq!(gain, 14.0, "the night haze scatters the moon 14x");
        assert_eq!(visibility, bevy::prelude::Visibility::Inherited);
        let expected = crate::atmosphere::SKY_CENTRE - key * 640.0;
        assert!(
            moon.distance(expected) < 0.01,
            "the disc must hang back along the installed moonlight: {moon} vs {expected}"
        );

        let (mut noon, _noon_sender, _noon_server) = configured_app(&["--clock", "12"]);
        noon.update();
        let (gain, _, _, visibility) = installed(&mut noon);
        assert_eq!(gain, 1.0, "the sun keeps 11.2's haze");
        assert_eq!(visibility, bevy::prelude::Visibility::Hidden);
    }

    /// Depth of field must focus the SELECTED DWARF, not the rig's aim point.
    ///
    /// `frame_selected_dwarf` centres him with `CameraRig::frame_render_point`, which writes the
    /// focus OFFSET from him by the composition push -- so `rig.focus` lands about 7.3 units short
    /// of the dwarf at `SELECT_DISTANCE` (33 * 20/90). Focusing `rig.focus` therefore puts the
    /// figure the operator just picked OUTSIDE the focal plane at the ruled f/0.05, while every
    /// boot-framing measurement stays correct, because at boot the subject IS the aim point.
    /// Wolf reported it from the seat, 2026-09-21.
    #[test]
    fn depth_of_field_focuses_the_selected_dwarf_not_the_rigs_aim_point() {
        let snapshot = Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 2, y: 1, z: 1 },
            tiles: vec![Tile::Solid(protocol::Material::Stone), Tile::Empty],
            entities: vec![protocol::Entity {
                id: 2,
                kind: protocol::EntityKind::Dwarf,
                pos: [1, 0, 0],
                state: protocol::JobState::Idle,
                light: None,
            }],
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        };
        let (mut app, _sender, _server) = configured_app_with_snapshot(&[], snapshot);
        app.update();
        app.world_mut()
            .insert_resource(crate::pick::SelectedDwarf(Some(2)));
        // ONE update, on purpose: the frame of the selection is the one that must already be
        // focused. `frame_selected_dwarf` moves the camera in `Update`, so focus computed before
        // transform propagation read last frame's camera and drew that frame blurred.
        app.update();

        let dwarf = {
            let world = app.world_mut();
            let mut q = world.query::<(&WorldProjected, &bevy::prelude::Transform)>();
            q.iter(world)
                .find(|(marker, _)| marker.0 == 2)
                .map(|(_, transform)| transform.translation)
                .expect("the selected dwarf must be drawn")
        };
        let (camera, rig_aim, focal) = {
            let world = app.world_mut();
            let mut q = world.query::<(&CameraRig, &super::DepthOfField)>();
            let (rig, dof) = q.single(world).expect("one camera");
            // The rig's own geometry, not `GlobalTransform`, so a system reading a camera that
            // never moved cannot agree with it by accident.
            let camera = rig.transform().translation;
            (
                camera,
                camera.distance(super::world_to_render_f32(rig.focus)),
                dof.focal_distance,
            )
        };
        let to_dwarf = camera.distance(dwarf);

        // Discrimination FIRST: if the aim point and the dwarf were the same distance away this
        // test could not tell the two rules apart, and would pass against either.
        assert!(
            (to_dwarf - rig_aim).abs() > 1.0,
            "this fixture cannot separate the two rules: dwarf at {to_dwarf}, aim point at \
             {rig_aim}"
        );
        assert!(
            (focal - to_dwarf).abs() < 0.25,
            "a selected dwarf must be the focal subject: focal_distance={focal} but he stands \
             {to_dwarf} away (the rig's aim point is {rig_aim})"
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
            "F4 haze on  F5 dof on  F6 bloom on  F7 ao on  fxaa on  F8 sun on  F9 ambient on  F10 campfire on  F11 torches on  F12 lanterns on"
        );
        for (key, source) in [
            (KeyCode::F8, super::LightSource::Sun),
            (KeyCode::F10, super::LightSource::Campfire),
            (KeyCode::F11, super::LightSource::Torches),
            (KeyCode::F12, super::LightSource::Lanterns),
            (KeyCode::F9, super::LightSource::Ambient),
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
            "F4 haze on  F5 dof on  F6 bloom on  F7 ao on  fxaa on  F8 sun off  F9 ambient off  F10 campfire off  F11 torches off  F12 lanterns off"
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

        // THE EMISSIVE FACE, not just the light. Wolf found this on the vehicle: with every
        // toggle off the campfire's place still glowed, because the emissive is baked into the
        // entity material at spawn and no toggle touched it. A source owns both.
        let emissive = |app: &mut App, kind| {
            let handle = app
                .world()
                .resource::<crate::project::ProjectionAssets>()
                .emissive_materials()
                .into_iter()
                .find_map(|(actual, handle)| (actual == kind).then_some(handle))
                .expect("the named emissive material must exist");
            app.world()
                .resource::<bevy::prelude::Assets<bevy::prelude::StandardMaterial>>()
                .get(&handle)
                .expect("the material handle must resolve")
                .emissive
        };
        assert_eq!(
            emissive(&mut app, protocol::LightKind::Campfire),
            bevy::color::LinearRgba::BLACK,
            "F6 must black the campfire's emissive face, not only its point light"
        );
        assert_eq!(
            emissive(&mut app, protocol::LightKind::Torch),
            bevy::color::LinearRgba::BLACK,
            "F9 must black the torch emissive faces"
        );

        // AND IT MUST COME BACK. Nothing pinned the restore before: point-light intensity is
        // rewritten every frame by `flicker_projection` inside `ProjectionSet`, so re-enabling
        // worked only by that grace, and the emissive has no such benefactor at all.
        for (key, _) in [
            (KeyCode::F8, ()),
            (KeyCode::F9, ()),
            (KeyCode::F10, ()),
            (KeyCode::F11, ()),
            (KeyCode::F12, ()),
        ] {
            press(&mut app, key);
        }
        assert_eq!(
            readout(&mut app),
            "F4 haze on  F5 dof on  F6 bloom on  F7 ao on  fxaa on  F8 sun on  F9 ambient on  F10 campfire on  F11 torches on  F12 lanterns on"
        );
        assert_eq!(
            emissive(&mut app, protocol::LightKind::Campfire),
            crate::appearance::light_properties(protocol::LightKind::Campfire)
                .color
                .to_linear(),
            "toggling the campfire back on must restore its emissive face"
        );
        assert_ne!(
            app.world_mut()
                .query_filtered::<&bevy::prelude::DirectionalLight, With<super::SunLight>>()
                .single(app.world())
                .unwrap()
                .illuminance,
            0.0,
            "toggling the sun back on must restore the directional light"
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
    /// The stale-runtime-slot guard, and the one that would have saved a session.
    ///
    /// Round 18 authored the walk cycle into the .blend and `assets/gltf/` was not re-promoted,
    /// so the embedded GLB had a skin and no clip. Everything loaded, the dwarf posed at bind,
    /// and the only symptom was an asset-server line about a missing `Animation0` label. The
    /// wiring was correct the whole time; the artifact was behind it.
    ///
    /// `include_bytes!` means the promoted file IS this test's subject, so promoting a dwarf
    /// exported before its clip turns this red instead of shipping a figure that cannot walk.
    #[test]
    fn the_embedded_dwarf_carries_its_walk_clip() {
        assert!(
            super::dwarf_clip_summary(),
            "the promoted dwarf must carry an animation clip; re-export the blend and promote it"
        );
    }

    #[test]
    fn the_startup_line_says_which_dwarf_this_binary_carries() {
        let (present, bytes) = super::dwarf_asset_summary();
        assert!(present, "the embedded dwarf must carry binary-glTF bytes");
        // The figure has to IDENTIFY the model, not merely prove one is present: r3's generated
        // voxel dwarf is 1,009,476 bytes and r8's box-modelled one 401,832. A line that could not
        // tell those apart is what left an operator unable to check whether the dwarf promoted on
        // 2026-09-13 was in the build they were holding.
        assert!(
            (100_000..2_000_000).contains(&bytes),
            "{bytes} bytes is not a plausible dwarf GLB; the blob is empty, truncated or wrong"
        );
        assert_eq!(
            bytes,
            super::DWARF_ASSET.1.len(),
            "the reported size must be the blob's own length, not a constant beside it"
        );
    }

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
            "F4 haze on  F5 dof on  F6 bloom on  F7 ao on  fxaa on  F8 sun on  F9 ambient on  F10 campfire on  F11 torches on  F12 lanterns on"
                .to_string(),
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
    fn absent_subdiv_flag_installs_the_shipped_default_four() {
        let (app, _, _) = configured_app(&[]);

        assert_eq!(
            app.world().resource::<TerrainSubdivision>().0,
            4,
            "starting the client without --subdiv must install the ruled shipped subdivision of four"
        );
    }

    /// AC11: `--subdiv 1` still renders, and its own instrument still reports a real number.
    ///
    /// The sibling test above proves the FLAG reaches the mesher. This one is about the subdiv-1
    /// path's shape surviving 10.9: `TerrainTile` entities are spawned, no chunk mesh is built
    /// (`chunks=0` is literal in that row), and `triangles_derived=` is non-zero. It asserts the
    /// production helper the `println!` itself calls, not a copy of its arithmetic.
    #[test]
    fn subdiv_one_still_spawns_terrain_and_reports_a_derived_triangle_count() {
        let (mut app, _, _) = configured_app(&["--subdiv", "1"]);
        app.update();

        let tiles = app
            .world_mut()
            .query::<&TerrainTile>()
            .iter(app.world())
            .count();
        let caps = app
            .world_mut()
            .query::<&SnowCap>()
            .iter(app.world())
            .count();
        let chunks = app
            .world_mut()
            .query::<&crate::project::TerrainChunk>()
            .iter(app.world())
            .count();

        assert!(tiles > 0, "--subdiv 1 spawned no TerrainTile entities");
        assert_eq!(
            chunks, 0,
            "--subdiv 1 built {chunks} chunk meshes; that path reports chunks=0 and draws shared \
             unit cuboids instead"
        );

        let derived = crate::project::derived_triangle_count(tiles, caps);
        assert!(
            derived > 0,
            "--subdiv 1 reported triangles_derived=0 from {tiles} cubes and {caps} snow caps"
        );
        assert!(
            derived >= tiles * 12,
            "triangles_derived={derived} is below the {} triangles {tiles} cubes alone contribute",
            tiles * 12
        );
    }

    #[test]
    fn subdiv_flag_reaches_the_rendered_terrain_and_four_keeps_the_shipped_scene() {
        let (mut default, _, _) = configured_app(&[]);
        let (mut one, _, _) = configured_app(&["--subdiv", "1"]);
        let (mut two, _, _) = configured_app(&["--subdiv", "2"]);
        let (mut four, _, _) = configured_app(&["--subdiv", "4"]);
        default.update();
        one.update();
        two.update();
        four.update();

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
            terrain_tiles(&mut four),
            terrain_tiles(&mut default),
            "--subdiv 4 must retain the shipped fine terrain scene byte-for-byte"
        );
        assert_eq!(
            snow_caps(&mut four),
            snow_caps(&mut default),
            "--subdiv 4 must retain the shipped snow-cap scene byte-for-byte"
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
        for args in [
            vec!["--subdiv", "1"],
            vec!["--subdiv", "2"],
            vec!["--subdiv", "4"],
        ] {
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
            let fine = args[1] != "1";
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

    /// `setup_camera` places the whole `--camera` framing and THEN overwrites the distance from
    /// `CaptureDistance`, so a pasted readout line's yaw, pitch and focus survived while its zoom
    /// was silently clobbered. Verified live in the 10.10 review: `--camera 0.7,0.45,4,64,64,9`
    /// panics on the close zoom, and the same command plus `--distance 90` succeeds — the
    /// operator got a framing the line he pasted does not describe, with no warning.
    ///
    /// The readout line IS the save format and carries a distance of its own, so there is nothing
    /// for `--distance` to add and no winner worth picking. Bail, exactly as `--cursor` and
    /// `--drag` do.
    #[test]
    fn a_pasted_camera_line_and_a_capture_distance_are_mutually_exclusive() {
        let both = super::parse_args_from([
            std::ffi::OsString::from("--capture"),
            std::ffi::OsString::from("working.png"),
            std::ffi::OsString::from("--frames"),
            std::ffi::OsString::from("30"),
            std::ffi::OsString::from("--camera"),
            std::ffi::OsString::from("0.7,0.45,20,64,64,9"),
            std::ffi::OsString::from("--distance"),
            std::ffi::OsString::from("90"),
        ]);
        let Err(error) = both else {
            panic!("a --camera line carries its own distance; accepting both silently drops one");
        };
        assert!(
            error.to_string().contains("mutually exclusive"),
            "the rejection must say WHY, not merely fail: {error}"
        );
        // Either one alone still parses: the pairing is what is rejected, not the flags.
        assert!(
            super::parse_args_from([
                std::ffi::OsString::from("--camera"),
                std::ffi::OsString::from("0.7,0.45,20,64,64,9"),
            ])
            .is_ok()
        );
        assert!(
            super::parse_args_from([
                std::ffi::OsString::from("--capture"),
                std::ffi::OsString::from("working.png"),
                std::ffi::OsString::from("--frames"),
                std::ffi::OsString::from("30"),
                std::ffi::OsString::from("--distance"),
                std::ffi::OsString::from("90"),
            ])
            .is_ok()
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

    /// `--camera`'s half of the same lie, and the reason this test exists at all: `--distance`'s
    /// own docstring above records that replacing its assignment with `let _ = distance;` left
    /// all 106 tests green. This runs the REAL `setup_camera` and reads the whole framing back
    /// off the spawned rig, so a `--camera` that parses and then evaporates cannot pass.
    #[test]
    fn the_camera_flag_reaches_the_camera_rig_rather_than_merely_parsing() {
        fn placed(requested: Option<super::CameraStart>) -> CameraRig {
            let mut app = App::new();
            if let Some(start) = requested {
                app.insert_resource(start);
            }
            app.world_mut()
                .run_system_once(super::setup_camera)
                .expect("the camera setup must run");
            *app.world_mut()
                .query::<&CameraRig>()
                .iter(app.world())
                .next()
                .expect("the camera setup must spawn a rig")
        }

        // Independent oracle: every expected value is written here by hand, never read back from
        // `CameraStart`, `BOOT_*` or the rig itself.
        let rig = placed(Some(super::CameraStart {
            yaw: 1.25,
            pitch: 0.6,
            distance: 42.0,
            focus: Vec3::new(20.0, 30.0, 4.0),
        }));
        assert_eq!(rig.yaw, 1.25);
        assert_eq!(rig.pitch, 0.6);
        assert_eq!(rig.distance, 42.0);
        assert_eq!(rig.focus, Vec3::new(20.0, 30.0, 4.0));

        // A second, different framing: one value reaching the rig proves nothing about the rest,
        // and a flag that carried only its distance would pass a single-case assertion.
        let other = placed(Some(super::CameraStart {
            yaw: -0.5,
            pitch: 0.3,
            distance: 120.0,
            focus: Vec3::new(1.0, 2.0, 3.0),
        }));
        assert_eq!(other.yaw, -0.5);
        assert_eq!(other.pitch, 0.3);
        assert_eq!(other.distance, 120.0);
        assert_eq!(other.focus, Vec3::new(1.0, 2.0, 3.0));

        // No flag means the boot framing, written by hand rather than read from the constants,
        // so the test fails when a BOOT_* constant moves (AC1's rule, same reason).
        let boot = placed(None);
        assert_eq!(boot.yaw, 0.7);
        assert_eq!(boot.pitch, 0.45);
        assert_eq!(boot.distance, 90.0);
        assert_eq!(boot.focus, Vec3::new(64.0, 64.0, 9.0));

        // And the flag goes through the SAME clamps the live controls use, so it cannot reach a
        // framing the operator could not fly to by hand.
        let clamped = placed(Some(super::CameraStart {
            yaw: 0.0,
            pitch: 9.0,
            distance: 9_000.0,
            focus: Vec3::new(-50.0, 900.0, 900.0),
        }));
        assert_eq!(clamped.pitch, std::f32::consts::FRAC_PI_2 - 0.15);
        assert_eq!(clamped.distance, 500.0);
        assert_eq!(clamped.focus, Vec3::new(0.0, 127.0, 31.0));
    }

    /// `--camera` is an interactive flag as well as a capture one, so it must NOT inherit
    /// `--distance`'s `requires --capture` gate. That gate is the one line of `--distance`'s
    /// plumbing this flag deliberately does not copy.
    #[test]
    fn the_camera_flag_parses_without_capture_and_rejects_malformed_values() {
        let args = super::parse_args_from([
            std::ffi::OsString::from("7451"),
            "--camera".into(),
            "0.7,0.45,90,64,64,9".into(),
        ])
        .expect("--camera must parse with no --capture anywhere on the command line");
        assert_eq!(
            args.camera,
            Some(super::CameraStart {
                yaw: 0.7,
                pitch: 0.45,
                distance: 90.0,
                focus: Vec3::new(64.0, 64.0, 9.0),
            })
        );

        for bad in [
            "0.7,0.45,90,64,64",
            "0.7,0.45,90,64,64,9,1",
            "0.7,0.45,90,64,64,nan",
            "",
        ] {
            assert!(
                super::parse_args_from([
                    std::ffi::OsString::from("7451"),
                    "--camera".into(),
                    bad.into(),
                ])
                .is_err(),
                "--camera must reject {bad:?}"
            );
        }
    }

    /// AC6: the printed line IS the save format. Paste its `--camera` token back and the rig
    /// returns EXACTLY -- float equality, not a tolerance, because a readout that only
    /// approximately reproduces a framing cannot be used to compare a look change.
    #[test]
    fn the_camera_readout_round_trips_through_the_flag_exactly() {
        for (yaw, pitch, distance, focus) in [
            (0.7_f32, 0.45_f32, 90.0_f32, Vec3::new(64.0, 64.0, 9.0)),
            (
                1.2345678,
                0.6543211,
                37.77777,
                Vec3::new(1.5, 126.25, 30.125),
            ),
            (-0.3333333, 0.15, 4.0, Vec3::ZERO),
        ] {
            let mut original = CameraRig::new([0, 0, 0]);
            original.place(yaw, pitch, distance, focus);
            let line = crate::camera::camera_readout_line(&original);

            // Take the flag argument straight out of the printed line, exactly as an operator
            // copying the tail of it would.
            let argument = line
                .split("--camera ")
                .nth(1)
                .expect("the readout line must carry a --camera argument");
            let parsed = super::parse_camera(std::ffi::OsString::from(argument))
                .expect("the readout's own argument must parse");
            let mut reproduced = CameraRig::new([0, 0, 0]);
            reproduced.place(parsed.yaw, parsed.pitch, parsed.distance, parsed.focus);

            assert_eq!(reproduced.yaw, original.yaw, "{line}");
            assert_eq!(reproduced.pitch, original.pitch, "{line}");
            assert_eq!(reproduced.distance, original.distance, "{line}");
            assert_eq!(reproduced.focus, original.focus, "{line}");
        }
    }

    /// AC6 for the case the review found, and Wolf's ruling on it (2026-09-17). Framing a dwarf
    /// writes an unclamped AIM POINT — `frame_render_point` deliberately skips the world-bounds
    /// clamp, because clamping it would decentre exactly the dwarves near an edge, which are the
    /// hardest to see. Printing that raw focus produced a line `place()` clamped on the way back
    /// in, so the round trip AC6 requires was NOT exact for those dwarves: a probe measured focus
    /// y -19.2592 printed and 0.0 restored. The ruling: clamp what the readout PRINTS and leave
    /// the aim point free.
    ///
    /// The cost is named rather than hidden, and asserted here: the printed line restores a view
    /// NEAR his, not the identical one. What it may never do is round-trip inexactly.
    #[test]
    fn the_readout_round_trips_even_from_an_aim_point_outside_the_world() {
        let mut rig = CameraRig::new([4, 4, 1]);
        rig.distance = 20.0;
        // The real 10.10 case: the drawn translation of a dwarf at world [2, 2, 1], which is
        // `world_to_render` plus the -0.5 Y the dwarf is drawn at.
        rig.frame_render_point(Vec3::new(2.0, 0.5, -2.0));
        assert!(
            rig.focus.y < 0.0,
            "the framing solve must leave the world for this test to mean anything; got {:?}",
            rig.focus
        );

        let line = crate::camera::camera_readout_line(&rig);
        let argument = line
            .split("--camera ")
            .nth(1)
            .expect("the readout line must carry a --camera argument");
        let parsed = super::parse_camera(std::ffi::OsString::from(argument))
            .expect("the readout's own argument must parse");
        let mut reproduced = CameraRig::new([0, 0, 0]);
        reproduced.place(parsed.yaw, parsed.pitch, parsed.distance, parsed.focus);

        // EXACT, by float equality, against the framing the line describes.
        let printed = rig.placed();
        assert_eq!(reproduced.yaw, printed.yaw, "{line}");
        assert_eq!(reproduced.pitch, printed.pitch, "{line}");
        assert_eq!(reproduced.distance, printed.distance, "{line}");
        assert_eq!(reproduced.focus, printed.focus, "{line}");
        // Hand-written: the out-of-world y is printed AT the bound, not past it and not rounded.
        assert_eq!(parsed.focus.y, 0.0, "{line}");
        // And the aim point itself is untouched, so the edge dwarf stays centred.
        assert!(rig.focus.y < 0.0, "{:?}", rig.focus);
    }

    /// AC7: two rigs that are not equal never print the same line. A readout that collapsed any
    /// field -- rounded it, or left it out -- would make two different framings indistinguishable
    /// in the record, which is the one thing this instrument exists to prevent.
    #[test]
    fn the_camera_readout_differs_whenever_the_rig_differs() {
        let base = (0.7_f32, 0.45_f32, 90.0_f32, Vec3::new(64.0, 64.0, 9.0));
        let variants = [
            base,
            // One field moved at a time, each by a hair, so a rounded or dropped field shows up.
            (0.700_01, base.1, base.2, base.3),
            (base.0, 0.450_01, base.2, base.3),
            (base.0, base.1, 90.000_1, base.3),
            (base.0, base.1, base.2, Vec3::new(64.000_1, 64.0, 9.0)),
            (base.0, base.1, base.2, Vec3::new(64.0, 64.000_1, 9.0)),
            (base.0, base.1, base.2, Vec3::new(64.0, 64.0, 9.000_1)),
        ];
        let lines = variants
            .iter()
            .map(|&(yaw, pitch, distance, focus)| {
                let mut rig = CameraRig::new([0, 0, 0]);
                rig.place(yaw, pitch, distance, focus);
                crate::camera::camera_readout_line(&rig)
            })
            .collect::<Vec<_>>();
        let unique = lines.iter().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            unique.len(),
            lines.len(),
            "each distinct rig must print a distinct line; got {lines:#?}"
        );
        // And a POSITIVE assertion about what the line actually says, so this cannot pass by
        // printing seven different but meaningless strings.
        assert_eq!(
            lines[0],
            "camera: yaw=0.7 pitch=0.45 distance=90 focus=64,64,9 --camera 0.7,0.45,90,64,64,9"
        );
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
            .init_resource::<crate::project::DwarfHeadings>()
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
            .init_resource::<crate::project::DwarfHeadings>()
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
                ..Default::default()
            })
            .init_resource::<TickClock>()
            .init_resource::<crate::project::DwarfHeadings>()
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
            .init_resource::<crate::project::DwarfHeadings>()
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
