#![forbid(unsafe_code)]

mod frame;
mod palette;
mod view;

fn main() {
    println!("tui will connect to port {}", protocol::DEFAULT_PORT);
}
