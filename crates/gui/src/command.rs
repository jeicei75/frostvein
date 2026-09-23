use std::{collections::VecDeque, net::TcpStream, sync::Mutex, time::Duration};

use bevy::prelude::{ButtonInput, KeyCode, Real, Res, ResMut, Resource, Time};
use protocol::{Command, Speed};

/// The GUI's upstream half of the daemon connection. `TcpStream` is Send but not Sync, so the
/// mutex is the same resource boundary used by `IngestReceiver`.
#[derive(Resource)]
pub struct CommandSink(pub Mutex<TcpStream>);

/// Bounded commands awaiting the next frame's upstream write.
#[derive(Resource, Default)]
pub struct PendingCommands {
    queue: VecDeque<Command>,
    dropped: usize,
}

const MAX_PENDING_COMMANDS: usize = 256;

impl PendingCommands {
    pub fn push(&mut self, command: Command) {
        if self.queue.len() == MAX_PENDING_COMMANDS {
            eprintln!("command queue full; dropping command");
            self.dropped += 1;
            return;
        }
        self.queue.push_back(command);
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn commands(&self) -> &VecDeque<Command> {
        &self.queue
    }

    /// Commands that were built by the input systems and never reached the socket. COUNTED, not
    /// merely logged: a designation the player dragged can vanish for three different reasons
    /// (queue full, poisoned writer, failed write) and stderr is not an observable a test or a
    /// future on-screen notice can read. A silent drop is the failure shape this client keeps
    /// shipping; this is the trace it leaves.
    pub fn dropped(&self) -> usize {
        self.dropped
    }
}

/// Whether the client believes the simulation is paused.
///
/// This mirrors what we last ASKED for, not what the daemon reports, so it can be wrong and a
/// reconnect would desync it.
///
/// CORRECTED 2026-09-19: this comment previously justified itself with "the wire carries no speed
/// in the snapshot". That was false and had always been false -- `protocol::Snapshot` and
/// `protocol::Delta` both carry `speed`, and `client_core::Mirror::speed()` has exposed it all
/// along. Nothing had ever looked. That false premise is why every instrument downstream trusted a
/// client-side belief instead of the daemon's own report, which is the mechanism behind #105.
/// Anything that must know the world is ACTUALLY still reads `Mirror::speed()` -- see
/// [`confirm_static_world_pause`] -- and not this.
#[derive(Resource, Default)]
pub struct SimPaused(pub bool);

/// Whether this session was asked to freeze the simulation for its whole run (`--static-world`).
#[derive(Resource, Default)]
pub struct StaticWorld(pub bool);

/// The tick `--static-world` aims to freeze the world at.
///
/// NOT zero, and not "as early as the wire allows". Before this, the pause was queued in `Startup`
/// and landed wherever the scheduler let it -- measured at three ticks and eight dwarf position
/// changes on an idle box, and further under load. Two captures of ONE binary therefore froze two
/// different worlds, and the camp window's "noise floor" was largely the dwarves standing
/// somewhere else. A fixed target makes the freeze point a DECISION instead of a race outcome.
///
/// EXACT, not merely reproducible, since the `pause at tick N` command this note used to defer now
/// exists (`protocol::Command::SetSpeed::at_tick`). The daemon holds the change until the world
/// stands on this tick and stops it there, so the freeze point no longer depends on when the
/// command arrived. Measured before the fix, same recipe, one fresh daemon each: idle froze at
/// 39-40, under CPU load at 80, against a daemon given a 15 s head start at 225 -- and camp
/// near-white swung 0.3636 pp across those worlds, 2.2x the signal the AC4 guard measures (#111).
///
/// 120 rather than 8 because the command must ARRIVE before its tick: the client spends seconds
/// booting Bevy and connecting, and the worst measured fresh-daemon arrival was tick 80 under 24
/// CPU burners. 120 leaves 50% headroom over that and costs ~4 s, against a capture that already
/// waits for 100 delivered ticks. If it is ever too tight the run FAILS rather than drifting --
/// see [`confirm_static_world_pause`].
const STATIC_WORLD_PAUSE_TICK: u64 = 120;

/// Where the `--static-world` freeze got to: whether the command has been sent, and the tick the
/// daemon actually reported itself stopped at.
#[derive(Resource, Default)]
pub struct StaticWorldPause {
    /// Mirrors `StaticWorld` so that one resource answers "must the capture wait?". Carried here
    /// rather than read as a second system parameter because `capture_after_frames` sits on
    /// Bevy's 16-parameter ceiling.
    active: bool,
    requested: bool,
    landed_at: Option<u64>,
    waited: Duration,
}

/// How long the capture will hold for a pause that never arrives before failing LOUDLY.
///
/// A capture that silently waits forever is indistinguishable from a hung daemon, and this project
/// has paid for enough instruments that report nothing and look fine.
///
/// WALL CLOCK, not frames. The landing waits on the daemon's 10 Hz tick, ~12 s from a fresh
/// daemon to tick 120, and a frame count only measures that on a slow renderer. It was 600 frames,
/// calibrated on lavapipe; the vehicle's RTX 4080 ran 600 uncapped headless frames by tick 93 and
/// refused every `--static-world` capture there.
const STATIC_WORLD_PAUSE_TIMEOUT: Duration = Duration::from_secs(60);

impl StaticWorldPause {
    pub fn new(active: bool) -> Self {
        Self {
            active,
            ..Default::default()
        }
    }

    /// True once the DAEMON has confirmed it is paused -- not once we asked.
    pub fn landed(&self) -> bool {
        self.landed_at.is_some()
    }

    /// The tick the daemon reported it stopped at, once it has.
    pub fn landed_at(&self) -> Option<u64> {
        self.landed_at
    }

    /// True while a `--static-world` run is still waiting for its pause. `capture.rs` holds its
    /// frame countdown on this so the settle window cannot start over a world still in motion.
    pub fn holds_capture(&self) -> bool {
        self.active && self.landed_at.is_none()
    }
}

/// Asks the daemon to freeze, once the world has reached [`STATIC_WORLD_PAUSE_TICK`].
///
/// `--static-world` is documented as "freeze the sim so two captures differ only by what you
/// changed" (`README.md`), and until this story it did no such thing. It set a flag that silenced
/// the capture's motion assertions and NOTHING else: the daemon kept ticking, the dwarves kept
/// walking, and `capture.rs` printed "the simulation is paused" over a world that was not. Issue
/// #105 measured what that cost -- the camp window's "flicker" floor was mostly moving,
/// lantern-carrying dwarves rather than flicker, and three documents were written from it.
///
/// The pause is not a new mechanism. It is the same `SetSpeed { Paused }` that `toggle_pause` has
/// sent since 10.5, queued for exactly the reason that function's own comment already gives.
///
/// `Update` rather than `Startup`, because the tick it fires at is now the point: a `Startup`
/// command is sent before the client has heard a single delta, so it lands at whatever tick the
/// scheduler allows and the frozen world differs run to run.
pub fn pause_static_world(
    static_world: Res<StaticWorld>,
    mirror: Res<crate::ingest::MirrorResource>,
    mut state: ResMut<StaticWorldPause>,
    mut paused: ResMut<SimPaused>,
    mut pending: ResMut<PendingCommands>,
) {
    if !static_world.0 || state.requested {
        return;
    }
    // Sent as EARLY as possible now, which is the opposite of what this function used to do. The
    // old code waited for the world to reach the target tick and then sent a bare pause, so the
    // freeze point was wherever the round trip landed. Naming the tick inverts that: the command
    // only has to arrive BEFORE tick 120, and the daemon decides where the world stops.
    state.requested = true;
    paused.0 = true;
    pending.push(Command::SetSpeed {
        speed: Speed::Paused,
        at_tick: Some(STATIC_WORLD_PAUSE_TICK),
    });
    eprintln!(
        "sim PAUSE REQUESTED (--static-world) for tick {STATIC_WORLD_PAUSE_TICK}, sent at tick {}",
        mirror.0.tick()
    );
}

/// Records the tick the daemon actually stopped at, from what the DAEMON reports.
///
/// This reads `Mirror::speed()`, which is the daemon's own speed off the wire, NOT the client's
/// `SimPaused` belief about what it asked for. That distinction is the whole lesson of #105: a
/// client-side flag reported a pause that had never happened, and every instrument downstream
/// believed it. `SimPaused`'s own doc comment asserted the wire carried no speed -- it always did
/// (`protocol::Snapshot::speed`, `protocol::Delta::speed`), and nothing had ever looked.
pub fn confirm_static_world_pause(
    static_world: Res<StaticWorld>,
    mirror: Res<crate::ingest::MirrorResource>,
    mut state: ResMut<StaticWorldPause>,
    time: Res<Time<Real>>,
    mut exit: bevy::prelude::MessageWriter<bevy::app::AppExit>,
) {
    if !static_world.0 || state.landed_at.is_some() {
        return;
    }
    if mirror.0.speed() != Speed::Paused {
        let before = state.waited;
        state.waited += time.delta();
        if before < STATIC_WORLD_PAUSE_TIMEOUT && state.waited >= STATIC_WORLD_PAUSE_TIMEOUT {
            eprintln!(
                "--static-world: the daemon never reported itself paused within \
                 {:?} (last reported speed {:?}, tick {}); \
                 refusing to capture a world that may still be moving",
                STATIC_WORLD_PAUSE_TIMEOUT,
                mirror.0.speed(),
                mirror.0.tick()
            );
            exit.write(bevy::app::AppExit::error());
        }
        return;
    }
    let tick = mirror.0.tick();
    state.landed_at = Some(tick);
    // The landing tick, every run, because it is the one number that says whether two captures
    // froze the same world. Silence here is what made the old floor look like noise.
    eprintln!("sim PAUSED (--static-world) at tick {tick}");
    // A landing anywhere but the requested tick means the daemon was already past it when the
    // command arrived and could not rewind. The capture would still succeed and still exit 0 --
    // and would silently be of a DIFFERENT world than its pair, which is the entire defect this
    // scheduling fixes (#111). Refuse it instead: a target that turns out too tight for a loaded
    // machine must surface as a red gate, not as a drifting measurement nobody can attribute.
    if tick != STATIC_WORLD_PAUSE_TICK {
        eprintln!(
            "--static-world: asked to freeze at tick {STATIC_WORLD_PAUSE_TICK} but the daemon \
             stopped at {tick}; it was already past when the command arrived. This capture is \
             NOT comparable with one that froze at {STATIC_WORLD_PAUSE_TICK}, so it is refused \
             rather than measured."
        );
        exit.write(bevy::app::AppExit::error());
    }
}

/// Hands the daemon back at the speed we found it, when a `--static-world` run ends cleanly.
///
/// The daemon's speed is ONE global (`simd/src/main.rs`), shared by every client and outliving the
/// one that set it. Before this, a `--static-world` run left the daemon frozen for good: the next
/// client to connect without the flag rendered a dead world, failed its motion assertions, exited
/// 101 and wrote no PNG at all -- blaming the dwarves for standing still.
///
/// NOTE: a run that dies by PANIC (the capture range check, `save_then_validate`) does not reach
/// this system, so the daemon stays paused in exactly that case. Naming the limitation rather than
/// reaching for a panic hook: the restart is one command, and a hook that runs during unwinding
/// has no socket guarantees worth the complexity.
pub fn restore_speed_on_exit(
    mut exits: bevy::prelude::MessageReader<bevy::app::AppExit>,
    static_world: Res<StaticWorld>,
    state: Res<StaticWorldPause>,
    mut pending: ResMut<PendingCommands>,
) {
    if exits.is_empty() {
        return;
    }
    exits.clear();
    if !static_world.0 || !state.requested {
        return;
    }
    pending.push(Command::SetSpeed {
        speed: Speed::Normal,
        at_tick: None,
    });
    eprintln!("sim RESUMED (--static-world run ending); daemon handed back at Normal");
}

/// Space toggles the simulation between paused and running.
///
/// Added because judging anything in a moving scene is guesswork: the dwarves wander, so two
/// captures of one binary differ by dwarf-sized areas with no code change at all, and a person at
/// the seat cannot hold a frame still to look at it. Story 10.5's AC2 measurement is unreachable
/// without it -- the same-build noise floor swamps the signal it is meant to separate.
///
/// REFUSES the press under `--static-world`. That flag promises the world is frozen for the whole
/// run, and every figure a capture reports is measured against that promise; a Space press used to
/// queue `SetSpeed { Normal }` regardless, silently resuming the daemon and breaking the guarantee
/// with nothing said. It says so now instead of doing it.
pub fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    static_world: Res<StaticWorld>,
    mut paused: ResMut<SimPaused>,
    mut pending: ResMut<PendingCommands>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    if static_world.0 {
        eprintln!("sim stays PAUSED: --static-world holds the world frozen for the whole run");
        return;
    }
    paused.0 = !paused.0;
    pending.push(Command::SetSpeed {
        speed: if paused.0 {
            Speed::Paused
        } else {
            Speed::Normal
        },
        at_tick: None,
    });
    // Say so on stderr: a paused world looks exactly like a stalled one.
    eprintln!("sim {}", if paused.0 { "PAUSED" } else { "running" });
}

/// Sends all commands built by the input systems. Errors deliberately drain the failed queue:
/// reconnect is outside this story and retrying stale designations would surprise the player.
pub fn send_commands(mut pending: ResMut<PendingCommands>, sink: Option<Res<CommandSink>>) {
    let Some(sink) = sink else {
        return;
    };
    let Ok(mut stream) = sink.0.lock() else {
        let lost = pending.queue.len();
        eprintln!("command writer lock poisoned; dropping {lost} queued command(s)");
        pending.queue.clear();
        pending.dropped += lost;
        return;
    };
    while let Some(command) = pending.queue.pop_front() {
        let encoded = match serde_json::to_string(&command) {
            Ok(encoded) => encoded,
            Err(error) => {
                eprintln!("could not encode command: {error}");
                pending.dropped += 1;
                continue;
            }
        };
        if let Err(error) = std::io::Write::write_all(&mut *stream, encoded.as_bytes())
            .and_then(|()| std::io::Write::write_all(&mut *stream, b"\n"))
            .and_then(|()| std::io::Write::flush(&mut *stream))
        {
            // The failed command is already off the queue, and the rest follow it: reconnect is
            // outside this story and replaying stale designations would surprise the player.
            // Dropping stays the decision; going UNCOUNTED does not.
            let lost = pending.queue.len() + 1;
            eprintln!("could not send command: {error}; dropping {lost} command(s)");
            pending.queue.clear();
            pending.dropped += lost;
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader},
        net::{TcpListener, TcpStream},
        sync::Mutex,
        time::Duration,
    };

    use bevy::prelude::{ButtonInput, KeyCode};
    use bevy::{
        MinimalPlugins,
        app::{App, Update},
    };
    use protocol::{Command, DesignationKind, Rect, Speed};

    use super::{
        CommandSink, MAX_PENDING_COMMANDS, PendingCommands, SimPaused, StaticWorld, send_commands,
        toggle_pause,
    };

    /// Space toggles, and the SECOND press matters as much as the first: a pause that cannot be
    /// released is a hang. Asserted on the queued command rather than on the resource flag, because
    /// the flag moving without a command reaching the daemon is the inert-mechanism shape.
    #[test]
    fn space_toggles_the_simulation_between_paused_and_running() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<PendingCommands>()
            .init_resource::<SimPaused>()
            // Default is OFF, which is the seat's case: this test is about Space WORKING. The
            // refusal under `--static-world` is proved separately, with its own control, by
            // `space_cannot_resume_a_static_world_run`.
            .init_resource::<StaticWorld>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, toggle_pause);

        // No key: nothing queued. An input system that fires unprompted is worse than one that
        // never fires, because it fires during someone else's test.
        app.update();
        assert!(app.world().resource::<PendingCommands>().is_empty());

        for (press, expected) in [(1, Speed::Paused), (2, Speed::Normal), (3, Speed::Paused)] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Space);
            app.update();
            // Production releases and clears every frame; `MinimalPlugins` does neither. RELEASE
            // as well as clear: `clear()` drops `just_pressed` but leaves the key held, and
            // `press()` on an already-held key does not re-fire `just_pressed`, so the second
            // press would never be seen.
            {
                let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                input.release(KeyCode::Space);
                input.clear();
            }
            app.update();
            let queued = app.world().resource::<PendingCommands>().commands().clone();
            assert_eq!(
                queued.back(),
                Some(&Command::SetSpeed {
                    speed: expected,
                    at_tick: None
                }),
                "press {press} must queue set_speed {expected:?}"
            );
            assert_eq!(
                queued.len(),
                press,
                "a held or released key must not queue a second command"
            );
        }
    }

    #[test]
    fn concrete_socket_writer_sends_newline_delimited_json() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let stream = TcpStream::connect(address).unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let (server, _) = listener.accept().unwrap();
        server
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(CommandSink(Mutex::new(stream)))
            .init_resource::<PendingCommands>()
            .add_systems(Update, send_commands);
        app.world_mut()
            .resource_mut::<PendingCommands>()
            .push(Command::Designate {
                kind: DesignationKind::Dig,
                rect: Rect {
                    min: [2, 3, 4],
                    max: [5, 6, 4],
                },
            });

        app.update();
        let mut line = String::new();
        BufReader::new(server).read_line(&mut line).unwrap();
        assert_eq!(
            line,
            "{\"type\":\"designate\",\"kind\":\"dig\",\"rect\":{\"min\":[2,3,4],\"max\":[5,6,4]}}\n"
        );
        assert!(app.world().resource::<PendingCommands>().is_empty());
        assert_eq!(
            app.world().resource::<PendingCommands>().dropped(),
            0,
            "a clean send must not report a dropped command"
        );
    }

    fn dig(x: i32) -> Command {
        Command::Designate {
            kind: DesignationKind::Dig,
            rect: Rect {
                min: [x, 0, 0],
                max: [x, 0, 0],
            },
        }
    }

    /// The bound is a SILENT drop: `push` logs to stderr and returns. Nothing in the suite touched
    /// it, so the queue could have been unbounded, or bounded at 1, with every test green.
    #[test]
    fn the_queue_bound_drops_and_counts_rather_than_growing() {
        let mut pending = PendingCommands::default();
        for x in 0..(MAX_PENDING_COMMANDS as i32 + 8) {
            pending.push(dig(x));
        }
        assert_eq!(
            pending.commands().len(),
            MAX_PENDING_COMMANDS,
            "the queue must stop at its bound rather than growing without limit"
        );
        assert_eq!(
            pending.dropped(),
            8,
            "every command past the bound must leave a counted trace, not just an stderr line"
        );
        assert_eq!(
            pending.commands().front().copied(),
            Some(dig(0)),
            "the bound drops the NEWEST command; the queued ones are already the player's"
        );
    }

    /// A designation that never reaches the socket is the failure this client keeps shipping.
    /// Dropping the queue on a dead peer stays the decision — going UNCOUNTED does not.
    #[test]
    fn a_dead_peer_drains_the_queue_and_counts_every_lost_command() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let stream = TcpStream::connect(address).unwrap();
        stream
            .set_write_timeout(Some(Duration::from_millis(250)))
            .unwrap();
        let (server, _) = listener.accept().unwrap();
        // Hang up. The next write to a closed peer fails rather than blocking forever.
        drop(server);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(CommandSink(Mutex::new(stream)))
            .init_resource::<PendingCommands>()
            .add_systems(Update, send_commands);
        {
            let mut pending = app.world_mut().resource_mut::<PendingCommands>();
            for x in 0..4 {
                pending.push(dig(x));
            }
        }

        // A closed peer can take one buffered write before the reset arrives, so drive a few
        // frames rather than assuming the first send fails.
        for _ in 0..8 {
            app.update();
            if app.world().resource::<PendingCommands>().is_empty() {
                break;
            }
        }
        let pending = app.world().resource::<PendingCommands>();
        assert!(
            pending.is_empty(),
            "a failed send must not leave commands queued forever"
        );
        assert!(
            pending.dropped() > 0,
            "commands lost to a dead peer must be COUNTED; stderr is not an observable"
        );
    }

    /// A poisoned writer drops everything queued. Same rule: the drop is allowed, the silence is
    /// not.
    #[test]
    fn a_poisoned_writer_counts_what_it_discards() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (_server, _) = listener.accept().unwrap();
        let sink = CommandSink(Mutex::new(stream));
        // Poison the mutex the way a panicking system would.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = sink.0.lock().unwrap();
            panic!("poison the writer");
        }));
        assert!(
            sink.0.is_poisoned(),
            "the guard must have poisoned the lock"
        );

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(sink)
            .init_resource::<PendingCommands>()
            .add_systems(Update, send_commands);
        {
            let mut pending = app.world_mut().resource_mut::<PendingCommands>();
            for x in 0..3 {
                pending.push(dig(x));
            }
        }
        app.update();

        let pending = app.world().resource::<PendingCommands>();
        assert!(pending.is_empty());
        assert_eq!(
            pending.dropped(),
            3,
            "all three queued commands were discarded and all three must be counted"
        );
    }
}
