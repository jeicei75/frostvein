use std::{collections::VecDeque, net::TcpStream, sync::Mutex};

use bevy::prelude::{Res, ResMut, Resource};
use protocol::Command;

/// The GUI's upstream half of the daemon connection. `TcpStream` is Send but not Sync, so the
/// mutex is the same resource boundary used by `IngestReceiver`.
#[derive(Resource)]
pub struct CommandSink(pub Mutex<TcpStream>);

/// Bounded commands awaiting the next frame's upstream write.
#[derive(Resource, Default)]
pub struct PendingCommands(VecDeque<Command>);

const MAX_PENDING_COMMANDS: usize = 256;

impl PendingCommands {
    pub fn push(&mut self, command: Command) {
        if self.0.len() == MAX_PENDING_COMMANDS {
            eprintln!("command queue full; dropping command");
            return;
        }
        self.0.push_back(command);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Sends all commands built by the input systems. Errors deliberately drain the failed queue:
/// reconnect is outside this story and retrying stale designations would surprise the player.
pub fn send_commands(mut pending: ResMut<PendingCommands>, sink: Option<Res<CommandSink>>) {
    let Some(sink) = sink else {
        return;
    };
    let Ok(mut stream) = sink.0.lock() else {
        eprintln!("command writer lock poisoned; dropping queued commands");
        pending.0.clear();
        return;
    };
    while let Some(command) = pending.0.pop_front() {
        let encoded = match serde_json::to_string(&command) {
            Ok(encoded) => encoded,
            Err(error) => {
                eprintln!("could not encode command: {error}");
                continue;
            }
        };
        if let Err(error) = std::io::Write::write_all(&mut *stream, encoded.as_bytes())
            .and_then(|()| std::io::Write::write_all(&mut *stream, b"\n"))
            .and_then(|()| std::io::Write::flush(&mut *stream))
        {
            eprintln!("could not send command: {error}");
            pending.0.clear();
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

    use super::{CommandSink, PendingCommands, send_commands};

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
    }
}
