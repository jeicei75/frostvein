//! Proves the *composed* client loop — reader thread, `apply`, `render`, frame write —
//! actually consumes a live delta stream.
//!
//! Every other `tui` test exercises those pieces in isolation: `view`'s pinned assertion
//! proves `render` formats a tick it is handed, and `apply` is tested against a struct.
//! Nothing connected them to a socket, so a client whose loop had stopped consuming
//! deltas would have left `scripts/gate.sh` fully green (AC10 was live-run-only).
//!
//! The stub server here is deliberately not `simd`: `CARGO_BIN_EXE_simd` is not exported
//! to this package, and hand-rolling the wire keeps the test deterministic and fast.

use std::{
    io::{Read, Write},
    net::TcpListener,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

const DIMS: protocol::Dims = protocol::Dims { x: 2, y: 2, z: 2 };
const SNAPSHOT_TICK: u64 = 7;

fn snapshot_line() -> String {
    let snapshot = protocol::Snapshot {
        msg_type: protocol::MessageType::Snapshot,
        dims: DIMS,
        tiles: vec![protocol::Tile::Solid(protocol::Material::Ice); 8],
        entities: Vec::new(),
        designations: Vec::new(),
        zones: Vec::new(),
        speed: protocol::Speed::Normal,
        tick: SNAPSHOT_TICK,
    };
    format!(
        "{}\n",
        serde_json::to_string(&snapshot).expect("encode stub snapshot")
    )
}

fn delta_line(tick: u64) -> String {
    let delta = protocol::Delta {
        msg_type: protocol::MessageType::Delta,
        tick,
        tiles: Vec::new(),
        entities: Vec::new(),
        designations: Vec::new(),
        zones: Vec::new(),
        speed: protocol::Speed::Normal,
    };
    format!(
        "{}\n",
        serde_json::to_string(&delta).expect("encode stub delta")
    )
}

#[test]
fn the_client_loop_renders_a_frame_per_streamed_delta() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind stub daemon");
    let port = listener
        .local_addr()
        .expect("read stub daemon address")
        .port();

    let mut child = Command::new(env!("CARGO_BIN_EXE_tui"))
        .arg(port.to_string())
        .arg("--frames")
        .arg("3")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn tui");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("tui must connect");
        stream
            .write_all(snapshot_line().as_bytes())
            .expect("send stub snapshot");
        for tick in 8..=10 {
            stream
                .write_all(delta_line(tick).as_bytes())
                .expect("send stub delta");
            thread::sleep(Duration::from_millis(20));
        }
        // Hold the socket open until the client has rendered its frames and exited;
        // dropping it early would race an EOF against the third frame.
        thread::sleep(Duration::from_millis(500));
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout pipe")
        .read_to_string(&mut stdout)
        .expect("read tui stdout");
    let status = child.wait().expect("wait for tui");
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    server.join().expect("stub daemon thread panicked");

    assert!(status.success(), "tui exited with {status}: {stderr}");

    for tick in 8..=10 {
        assert!(
            stdout.contains(&format!("tick {tick}")),
            "no frame rendered for streamed tick {tick}; the client loop is not applying deltas"
        );
    }

    // The connect snapshot is consumed before the streaming loop starts, so a frame for
    // its tick would mean frames are being driven by something other than the deltas.
    assert!(
        !stdout.contains(&format!("tick {SNAPSHOT_TICK}")),
        "a frame was rendered for the connect snapshot rather than for a streamed delta"
    );
}
