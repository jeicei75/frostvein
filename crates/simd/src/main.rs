#![forbid(unsafe_code)]

mod bridge;

use anyhow::{Context, bail};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread,
    time::{Duration, Instant},
};

const SEED: u64 = 0xF005_7E1A;
const TICK_PERIOD: Duration = Duration::from_millis(100);
const FAST_TICK_PERIOD: Duration = Duration::from_millis(20);
const CLIENT_QUEUE: usize = 16;
const SAVE_PATH: &str = "frostvein.save";
const MAX_SAVE_BYTES: u64 = 16 * 1024 * 1024;

/// Caps on what one misbehaving local client can cost the daemon. Phase one is
/// localhost-only (NFR1), so these bound accidents — a buggy client — not attacks.
const MAX_CONNECTIONS: usize = 256;
const MAX_LINE_BYTES: u64 = 64 * 1024;
const LOG_EXCERPT_CHARS: usize = 200;
const WRITE_TIMEOUT: Duration = Duration::from_secs(30);
/// Sticky `accept` errors (EMFILE/ENFILE) repeat immediately; without a pause the
/// loop spins at 100% CPU emitting log lines until the disk fills.
const ACCEPT_BACKOFF: Duration = Duration::from_millis(100);

/// Decrements the live-connection count when a connection's thread ends.
struct ConnectionGuard(Arc<AtomicUsize>);

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

struct Client {
    tx: SyncSender<Arc<String>>,
    /// Cloned purely so eviction can force the socket shut. Dropping `tx` alone ends
    /// the write thread only once it unblocks, and a client that has stopped reading
    /// leaves `serve` parked in `write_all` for up to two `WRITE_TIMEOUT` windows —
    /// squatting a `MAX_CONNECTIONS` slot for ~60s after the daemon declared it gone.
    stream: TcpStream,
}

fn main() -> anyhow::Result<()> {
    // NOTE: args_os, not args — std::env::args panics on non-UTF-8 argv during
    // iteration, which would bypass anyhow and print a raw backtrace.
    let port: u16 = match std::env::args_os().nth(1) {
        Some(arg) => {
            let text = arg
                .to_str()
                .with_context(|| format!("port argument is not valid UTF-8: {arg:?}"))?;
            text.parse().with_context(|| {
                format!("invalid port argument {text:?}: expected 0-65535 (0 = OS-assigned)")
            })?
        }
        None => protocol::DEFAULT_PORT,
    };
    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("could not bind 127.0.0.1:{port}"))?;
    println!("listening on 127.0.0.1:{}", listener.local_addr()?.port());

    let live = Arc::new(AtomicUsize::new(0));
    let (new_tx, new_rx) = mpsc::channel();
    let accept_live = Arc::clone(&live);
    thread::Builder::new()
        .name("accept".to_string())
        .spawn(move || accept_connections(listener, new_tx, accept_live))
        .context("could not spawn accept thread")?;

    let (command_tx, command_rx) = mpsc::channel();
    let world = sim_core::World::generate(SEED, sim_core::Dims::DEFAULT);
    tick(world, new_rx, live, command_tx, command_rx)
}

fn accept_connections(
    listener: TcpListener,
    new_tx: mpsc::Sender<TcpStream>,
    live: Arc<AtomicUsize>,
) {
    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                if live.load(Ordering::Relaxed) >= MAX_CONNECTIONS {
                    eprintln!("connection limit ({MAX_CONNECTIONS}) reached; dropping client");
                    continue;
                }
                live.fetch_add(1, Ordering::Relaxed);
                if new_tx.send(stream).is_err() {
                    live.fetch_sub(1, Ordering::Relaxed);
                    break;
                }
            }
            Err(error) => {
                eprintln!("accept error: {error}");
                thread::sleep(ACCEPT_BACKOFF);
            }
        }
    }
}

fn tick(
    mut world: sim_core::World,
    new_rx: Receiver<TcpStream>,
    live: Arc<AtomicUsize>,
    command_tx: mpsc::Sender<protocol::Command>,
    command_rx: Receiver<protocol::Command>,
) -> anyhow::Result<()> {
    let mut clients = Vec::new();
    let mut speed = protocol::Speed::Normal;
    loop {
        for command in command_rx.try_iter() {
            match command {
                protocol::Command::SetSpeed { speed: next } => speed = next,
                // NOTE: encoding ~524k tiles stalls this iteration by roughly the same
                // amount as a connect snapshot. Save is an explicit operator action, so
                // no worker thread or async write is warranted yet.
                protocol::Command::Save => save_world(&world),
                protocol::Command::Load => {
                    // NOTE: speed belongs to the daemon, not the saved sim. Loading while
                    // paused therefore leaves the loop paused on the restored tick.
                    if let Some(loaded) = load_world() {
                        world = loaded;
                        let line = Arc::new(format!(
                            "{}\n",
                            serde_json::to_string(&bridge::snapshot(&world, speed))?
                        ));
                        broadcast(&mut clients, &line);
                    }
                }
                protocol::Command::Quit => {
                    eprintln!("shutting down on client quit");
                    return Ok(());
                }
            }
        }
        let deadline = Instant::now() + period(speed);

        // Encode the connect snapshot at most once per iteration and share it — every
        // client admitted here connects at the same tick, so one encoding serves them
        // all. Encoding per stream let a burst of connects stall the timestep (measured
        // 1.4s for 8 simultaneous connects, and `deadline` never catches the loss back).
        // NOTE: stays lazy on purpose. Hoisting this unconditionally would serialize the
        // whole world every tick even with nobody connected.
        let mut snapshot_line: Option<Arc<String>> = None;
        for stream in new_rx.try_iter() {
            let line = match &snapshot_line {
                Some(line) => Arc::clone(line),
                None => {
                    let encoded = Arc::new(format!(
                        "{}\n",
                        serde_json::to_string(&bridge::snapshot(&world, speed))?
                    ));
                    snapshot_line = Some(Arc::clone(&encoded));
                    encoded
                }
            };
            if let Some(client) =
                connect_client(stream, line, Arc::clone(&live), command_tx.clone())
            {
                clients.push(client);
            }
        }

        // NOTE: the whole schedule is world-advancing today, so pausing skips all of it
        // and freezes the tick with it. Story 3.1 adds a command-consuming system that
        // must run while paused; that is when this splits into command consumption and
        // world advancement.
        if speed != protocol::Speed::Paused {
            world.step();
        }
        let delta_line = Arc::new(format!(
            "{}\n",
            serde_json::to_string(&bridge::delta(&mut world, speed))?
        ));
        broadcast(&mut clients, &delta_line);

        thread::sleep(deadline.saturating_duration_since(Instant::now()));
    }
}

fn save_world(world: &sim_core::World) {
    let tick = world.tick();
    let result = (|| -> anyhow::Result<()> {
        let encoded = serde_json::to_vec(&world.to_save()).context("could not encode world")?;
        if encoded.len() as u64 > MAX_SAVE_BYTES {
            bail!(
                "encoded save is {} bytes; limit is {MAX_SAVE_BYTES}",
                encoded.len()
            );
        }
        let temporary = format!("{SAVE_PATH}.tmp");
        fs::write(&temporary, encoded)
            .with_context(|| format!("could not write temporary save {temporary}"))?;
        fs::rename(&temporary, SAVE_PATH)
            .with_context(|| format!("could not rename {temporary} to {SAVE_PATH}"))?;
        Ok(())
    })();

    match result {
        Ok(()) => eprintln!("saved tick {tick} to {SAVE_PATH}"),
        Err(error) => eprintln!("could not save {SAVE_PATH}: {error:#}"),
    }
}

fn load_world() -> Option<sim_core::World> {
    let result = (|| -> anyhow::Result<sim_core::World> {
        let file =
            fs::File::open(SAVE_PATH).with_context(|| format!("could not open {SAVE_PATH}"))?;
        let mut encoded = Vec::new();
        file.take(MAX_SAVE_BYTES + 1)
            .read_to_end(&mut encoded)
            .with_context(|| format!("could not read {SAVE_PATH}"))?;
        if encoded.len() as u64 > MAX_SAVE_BYTES {
            bail!("save exceeds {MAX_SAVE_BYTES}-byte limit");
        }
        let save: sim_core::SaveState = serde_json::from_slice(&encoded)
            .with_context(|| format!("could not decode {SAVE_PATH}"))?;
        let tile_count = u64::from(save.dims.x)
            .checked_mul(u64::from(save.dims.y))
            .and_then(|area| area.checked_mul(u64::from(save.dims.z)))
            .context("save dimensions overflow the tile count")?;
        if save.tiles.len() as u64 != tile_count {
            bail!(
                "save has {} tiles but dims {}x{}x{} need {tile_count}",
                save.tiles.len(),
                save.dims.x,
                save.dims.y,
                save.dims.z
            );
        }
        if save.tick == u64::MAX {
            bail!("save tick {} cannot advance", save.tick);
        }
        let in_bounds = |pos: sim_core::Pos| {
            pos.x >= 0
                && pos.y >= 0
                && pos.z >= 0
                && pos.x < i32::MAX
                && pos.y < i32::MAX
                && (pos.x as u32) < save.dims.x
                && (pos.y as u32) < save.dims.y
                && (pos.z as u32) < save.dims.z
        };
        for dwarf in &save.dwarves {
            if !in_bounds(dwarf.pos) {
                bail!(
                    "save dwarf {} position {},{},{} is outside dims {}x{}x{}",
                    dwarf.id,
                    dwarf.pos.x,
                    dwarf.pos.y,
                    dwarf.pos.z,
                    save.dims.x,
                    save.dims.y,
                    save.dims.z
                );
            }
            if !in_bounds(dwarf.home) {
                bail!(
                    "save dwarf {} home {},{},{} is outside dims {}x{}x{}",
                    dwarf.id,
                    dwarf.home.x,
                    dwarf.home.y,
                    dwarf.home.z,
                    save.dims.x,
                    save.dims.y,
                    save.dims.z
                );
            }
        }
        Ok(sim_core::World::from_save(save))
    })();

    match result {
        Ok(world) => Some(world),
        Err(error) => {
            eprintln!("could not load {SAVE_PATH}: {error:#}");
            None
        }
    }
}

fn period(speed: protocol::Speed) -> Duration {
    match speed {
        protocol::Speed::Paused | protocol::Speed::Normal => TICK_PERIOD,
        protocol::Speed::Fast => FAST_TICK_PERIOD,
    }
}

fn broadcast(clients: &mut Vec<Client>, line: &Arc<String>) {
    clients.retain(|client| match client.tx.try_send(Arc::clone(line)) {
        Ok(()) => true,
        Err(TrySendError::Full(_)) => {
            eprintln!("client delta queue full; disconnecting client");
            // AC9 says disconnected AND removed. Removing it from the registry is the
            // cheap half; without this shutdown the socket lingers ESTAB until
            // `write_all` times out, so force it now.
            let _ = client.stream.shutdown(Shutdown::Both);
            false
        }
        // Its write thread has already ended, so the socket is closed with it.
        Err(TrySendError::Disconnected(_)) => false,
    });
}

fn connect_client(
    stream: TcpStream,
    snapshot_line: Arc<String>,
    live: Arc<AtomicUsize>,
    command_tx: mpsc::Sender<protocol::Command>,
) -> Option<Client> {
    // Built first so every early return below still releases the connection slot that
    // `accept_connections` reserved.
    let guard = ConnectionGuard(live);
    let evict_handle = match stream.try_clone() {
        Ok(handle) => handle,
        Err(error) => {
            eprintln!("could not clone client socket for eviction: {error}");
            return None;
        }
    };
    let (tx, rx) = mpsc::sync_channel(CLIENT_QUEUE);
    tx.try_send(snapshot_line)
        .expect("a new client queue must have room for its snapshot");
    if let Err(error) = thread::Builder::new()
        .name("client-write".to_string())
        .spawn(move || serve(stream, rx, guard, command_tx))
    {
        eprintln!("could not spawn connection thread: {error}");
        return None;
    }
    Some(Client {
        tx,
        stream: evict_handle,
    })
}

fn serve(
    mut stream: TcpStream,
    lines: Receiver<Arc<String>>,
    _guard: ConnectionGuard,
    command_tx: mpsc::Sender<protocol::Command>,
) {
    if let Err(error) = stream.set_write_timeout(Some(WRITE_TIMEOUT)) {
        eprintln!("could not set write timeout: {error}");
        return;
    }

    let read_stream = match stream.try_clone() {
        Ok(read_stream) => read_stream,
        Err(error) => {
            eprintln!("could not clone client socket for reads: {error}");
            return;
        }
    };
    if let Err(error) = thread::Builder::new()
        .name("client-read".to_string())
        .spawn(move || read_inbound(read_stream, command_tx))
    {
        eprintln!("could not spawn client reader thread: {error}");
        return;
    }

    for line in lines {
        if let Err(error) = stream.write_all(line.as_bytes()) {
            eprintln!("client write error: {error}");
            break;
        }
    }
    let _ = stream.shutdown(Shutdown::Both);
}

fn read_inbound(stream: TcpStream, command_tx: mpsc::Sender<protocol::Command>) {
    let mut reader = BufReader::new(stream);
    let mut line = Vec::new();
    loop {
        line.clear();
        match (&mut reader)
            .take(MAX_LINE_BYTES)
            .read_until(b'\n', &mut line)
        {
            Ok(0) => break,
            Ok(_) => {
                if !line.ends_with(b"\n") {
                    eprintln!("client line exceeded {MAX_LINE_BYTES} bytes; closing connection");
                    let _ = reader.get_ref().shutdown(Shutdown::Both);
                    break;
                }
                // NOTE: lossy, not a UTF-8 error — a single stray byte must not cost the
                // client its connection, since Story 2.1 streams deltas down it.
                let text = String::from_utf8_lossy(&line);
                let text = text.trim_end();
                match serde_json::from_str::<protocol::Command>(text) {
                    Ok(command) => {
                        if command_tx.send(command).is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        eprintln!("unrecognized client message: {}", excerpt(text));
                    }
                }
            }
            Err(error) => {
                eprintln!("client read error: {error}");
                let _ = reader.get_ref().shutdown(Shutdown::Both);
                break;
            }
        }
    }
}

/// Client bytes are echoed to the log; without a cap the daemon amplifies whatever
/// a client sends into stderr roughly 1:1.
fn excerpt(line: &str) -> String {
    match line.char_indices().nth(LOG_EXCERPT_CHARS) {
        Some((cut, _)) => format!("{}… ({} bytes total)", &line[..cut], line.len()),
        None => line.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_period_is_exactly_ten_hertz() {
        assert_eq!(TICK_PERIOD, Duration::from_millis(100));
    }

    #[test]
    fn speed_periods_are_pinned() {
        assert_eq!(period(protocol::Speed::Paused), Duration::from_millis(100));
        assert_eq!(period(protocol::Speed::Normal), Duration::from_millis(100));
        assert_eq!(period(protocol::Speed::Fast), Duration::from_millis(20));
    }

    /// A real loopback pair: the daemon's end goes into `Client`, the peer end stands in
    /// for the client program and observes whether eviction really shut the socket.
    fn socket_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback listener");
        let addr = listener.local_addr().expect("read loopback address");
        let peer = TcpStream::connect(addr).expect("connect to loopback listener");
        let (daemon_side, _) = listener.accept().expect("accept loopback connection");
        (daemon_side, peer)
    }

    #[test]
    fn full_client_queue_is_removed_from_the_registry_at_sixteen_lines() {
        let (daemon_side, mut peer) = socket_pair();
        // Stands in for the handle `serve` owns while parked in `write_all`. Without it
        // this test is a false positive: dropping the evicted `Client` would close the
        // socket by itself and the peer would see EOF with or without the shutdown.
        let held_by_write_thread = daemon_side.try_clone().expect("clone the daemon side");
        let (sender, _receiver) = std::sync::mpsc::sync_channel(CLIENT_QUEUE);

        // Hardcoded 16, deliberately not CLIENT_QUEUE: widening the constant must leave
        // room here and turn this test red rather than silently still filling the queue.
        for line in 0..16 {
            assert!(sender.try_send(Arc::new(line.to_string())).is_ok());
        }
        let mut clients = vec![Client {
            tx: sender,
            stream: daemon_side,
        }];

        broadcast(&mut clients, &Arc::new("overflow".to_string()));

        assert!(clients.is_empty());

        // AC9's other half: deregistering is not disconnecting. Without the shutdown the
        // peer stays ESTAB until `write_all` times out ~60s later, holding a slot.
        peer.set_read_timeout(Some(Duration::from_secs(5)))
            .expect("set peer read timeout");
        let mut buf = [0u8; 1];
        assert_eq!(
            peer.read(&mut buf).expect("peer read after eviction"),
            0,
            "evicting a client must shut its socket, not leave it open until write timeout"
        );
        drop(held_by_write_thread);
    }

    #[test]
    fn disconnected_client_queue_is_removed_from_the_registry() {
        let (daemon_side, _peer) = socket_pair();
        let (sender, receiver) = std::sync::mpsc::sync_channel(CLIENT_QUEUE);
        drop(receiver);
        let mut clients = vec![Client {
            tx: sender,
            stream: daemon_side,
        }];

        broadcast(&mut clients, &Arc::new("delta".to_string()));

        assert!(clients.is_empty());
    }
}
