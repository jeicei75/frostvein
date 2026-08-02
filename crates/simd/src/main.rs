#![forbid(unsafe_code)]

mod bridge;

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

const SEED: u64 = 0xF005_7E1A;

fn main() -> anyhow::Result<()> {
    let port: u16 = match std::env::args().nth(1) {
        Some(arg) => arg.parse()?,
        None => protocol::DEFAULT_PORT,
    };
    let world = sim_core::World::generate(SEED, sim_core::Dims::DEFAULT);
    // NOTE: the world is static in this story, so the snapshot line is encoded
    // once and shared. Story 2.1's tick loop re-encodes per connection.
    let line = Arc::new(format!(
        "{}\n",
        serde_json::to_string(&bridge::snapshot(&world))?
    ));
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("listening on 127.0.0.1:{}", listener.local_addr()?.port());

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                let line = Arc::clone(&line);
                thread::spawn(move || serve(stream, line.as_str()));
            }
            Err(error) => eprintln!("accept error: {error}"),
        }
    }

    Ok(())
}

fn serve(mut stream: TcpStream, snapshot_line: &str) {
    if let Err(error) = stream.write_all(snapshot_line.as_bytes()) {
        eprintln!("client write error: {error}");
        return;
    }

    for line in BufReader::new(stream).lines() {
        match line {
            Ok(line) => eprintln!("unrecognized client message: {line}"),
            Err(error) => {
                eprintln!("client read error: {error}");
                break;
            }
        }
    }
}
