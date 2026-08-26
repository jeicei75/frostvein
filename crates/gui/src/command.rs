use std::{collections::VecDeque, net::TcpStream, sync::Mutex};

use bevy::prelude::{Res, ResMut, Resource};
use protocol::Command;

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

    use bevy::{
        MinimalPlugins,
        app::{App, Update},
    };
    use protocol::{Command, DesignationKind, Rect};

    use super::{CommandSink, MAX_PENDING_COMMANDS, PendingCommands, send_commands};

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
