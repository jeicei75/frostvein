#![forbid(unsafe_code)]

mod palette;
mod view;

fn main() {
    println!("tui will connect to port {}", protocol::DEFAULT_PORT);
}
