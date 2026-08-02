#![forbid(unsafe_code)]

mod bridge;

fn main() {
    let world = sim_core::World::generate(0xF005_7E1A, sim_core::Dims::DEFAULT);
    let dims = world.dims();
    println!(
        "generated {}x{}x{} world with {} dwarves on port {}",
        dims.x,
        dims.y,
        dims.z,
        world.dwarves().len(),
        protocol::DEFAULT_PORT
    );
}
