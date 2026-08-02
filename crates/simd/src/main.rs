#![forbid(unsafe_code)]

mod bridge;

use anyhow::Context;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

const SEED: u64 = 0xF005_7E1A;

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
    let world = sim_core::World::generate(SEED, sim_core::Dims::DEFAULT);
    // NOTE: the world is static in this story, so the snapshot line is encoded
    // once and shared. Story 2.1's tick loop re-encodes per connection.
    let line = Arc::new(format!(
        "{}\n",
        serde_json::to_string(&bridge::snapshot(&world))?
    ));
    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("could not bind 127.0.0.1:{port}"))?;
    println!("listening on 127.0.0.1:{}", listener.local_addr()?.port());

    let live = Arc::new(AtomicUsize::new(0));
    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                if live.load(Ordering::Relaxed) >= MAX_CONNECTIONS {
                    eprintln!("connection limit ({MAX_CONNECTIONS}) reached; dropping client");
                    continue;
                }
                live.fetch_add(1, Ordering::Relaxed);
                let guard = ConnectionGuard(Arc::clone(&live));
                let line = Arc::clone(&line);
                // NOTE: Builder::spawn returns Err where thread::spawn panics — and a
                // panic here would unwind main and take every live connection with it.
                if let Err(error) = thread::Builder::new().spawn(move || {
                    let _guard = guard;
                    serve(stream, line.as_str());
                }) {
                    eprintln!("could not spawn connection thread: {error}");
                }
            }
            Err(error) => {
                eprintln!("accept error: {error}");
                thread::sleep(ACCEPT_BACKOFF);
            }
        }
    }

    Ok(())
}

fn serve(mut stream: TcpStream, snapshot_line: &str) {
    // NOTE: the snapshot is ~6.9 MB, far past any socket send buffer, so write_all
    // blocks once a client stops reading. Without this the thread is pinned forever.
    if let Err(error) = stream.set_write_timeout(Some(WRITE_TIMEOUT)) {
        eprintln!("could not set write timeout: {error}");
        return;
    }
    if let Err(error) = stream.write_all(snapshot_line.as_bytes()) {
        eprintln!("client write error: {error}");
        return;
    }

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
                    break;
                }
                // NOTE: lossy, not a UTF-8 error — a single stray byte must not cost the
                // client its connection, since Story 2.1 streams deltas down it.
                let text = String::from_utf8_lossy(&line);
                eprintln!("unrecognized client message: {}", excerpt(text.trim_end()));
            }
            Err(error) => {
                eprintln!("client read error: {error}");
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
