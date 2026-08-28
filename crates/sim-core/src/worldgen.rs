use rand::RngExt;
use rand_chacha::ChaCha8Rng;

use crate::{Dims, Material, Tile};

const NOISE_SPACING: u32 = 32;
pub(crate) const CAMP_RADIUS: u32 = 3;

pub(crate) fn index(dims: Dims, x: u32, y: u32, z: u32) -> usize {
    // NOTE: widened to usize before multiplying — the u32 product wraps silently in
    // release, which would address the wrong tile rather than fail.
    x as usize + y as usize * dims.x as usize + z as usize * dims.x as usize * dims.y as usize
}

pub(crate) fn height_field(dims: Dims, rng: &mut ChaCha8Rng) -> Vec<u32> {
    let lattice_x = dims.x.div_ceil(NOISE_SPACING) + 1;
    let lattice_y = dims.y.div_ceil(NOISE_SPACING) + 1;
    // NOTE: worldgen determinism rests on f64 here (`random::<f64>`, `lerp`, `smooth`,
    // `.round()` below). This is reproducible — Rust performs no FMA contraction and
    // these are correctly-rounded IEEE ops — but it is the only float in the sim, so
    // any future change here is a determinism change.
    let lattice: Vec<f64> = (0..lattice_x as usize * lattice_y as usize)
        .map(|_| rng.random::<f64>())
        .collect();

    let mut heights = Vec::with_capacity(dims.x as usize * dims.y as usize);
    for y in 0..dims.y {
        for x in 0..dims.x {
            let lattice_pos_x = x / NOISE_SPACING;
            let lattice_pos_y = y / NOISE_SPACING;
            let fraction_x = smooth((x % NOISE_SPACING) as f64 / NOISE_SPACING as f64);
            let fraction_y = smooth((y % NOISE_SPACING) as f64 / NOISE_SPACING as f64);

            let top_left = lattice[(lattice_pos_x + lattice_pos_y * lattice_x) as usize];
            let top_right = lattice[(lattice_pos_x + 1 + lattice_pos_y * lattice_x) as usize];
            let bottom_left = lattice[(lattice_pos_x + (lattice_pos_y + 1) * lattice_x) as usize];
            let bottom_right =
                lattice[(lattice_pos_x + 1 + (lattice_pos_y + 1) * lattice_x) as usize];
            let top = lerp(top_left, top_right, fraction_x);
            let bottom = lerp(bottom_left, bottom_right, fraction_x);
            let noise = lerp(top, bottom, fraction_y);
            let height = (dims.z as f64 / 2.0 + (noise * 2.0 - 1.0) * 12.0).round();
            heights.push(height.clamp(3.0, dims.z.saturating_sub(2) as f64) as u32);
        }
    }

    clamp_steps(dims, &mut heights);
    heights
}

fn smooth(value: f64) -> f64 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(start: f64, end: f64, amount: f64) -> f64 {
    start + (end - start) * amount
}

fn clamp_steps(dims: Dims, heights: &mut [u32]) {
    loop {
        let mut changed = false;
        for y in 0..dims.y {
            for x in 0..dims.x {
                let current = (x + y * dims.x) as usize;
                for (nx, ny) in [(x + 1, y), (x, y + 1)] {
                    if nx >= dims.x || ny >= dims.y {
                        continue;
                    }
                    let neighbour = (nx + ny * dims.x) as usize;
                    if heights[current] > heights[neighbour] + 1 {
                        heights[current] = heights[neighbour] + 1;
                        changed = true;
                    } else if heights[neighbour] > heights[current] + 1 {
                        heights[neighbour] = heights[current] + 1;
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
}

pub(crate) fn layered_terrain(dims: Dims, heights: &[u32], rng: &mut ChaCha8Rng) -> Vec<Tile> {
    let mut tiles = vec![Tile::Empty; dims.x as usize * dims.y as usize * dims.z as usize];
    for y in 0..dims.y {
        for x in 0..dims.x {
            let height = heights[(x + y * dims.x) as usize];
            for z in 0..height {
                let material = if z + 2 < height {
                    Material::Stone
                } else {
                    Material::Soil
                };
                tiles[index(dims, x, y, z)] = Tile::Solid(material);
            }
            let surface = if rng.random::<bool>() {
                Material::Snow
            } else {
                Material::Ice
            };
            tiles[index(dims, x, y, height)] = Tile::Solid(surface);
        }
    }
    tiles
}

pub(crate) fn place_ramps(dims: Dims, heights: &[u32], tiles: &mut [Tile]) {
    for y in 0..dims.y {
        for x in 0..dims.x {
            let height = heights[(x + y * dims.x) as usize];
            let has_higher_neighbour = [
                (x as i32 - 1, y as i32),
                (x as i32 + 1, y as i32),
                (x as i32, y as i32 - 1),
                (x as i32, y as i32 + 1),
            ]
            .into_iter()
            .any(|(nx, ny)| {
                nx >= 0
                    && ny >= 0
                    && nx < dims.x as i32
                    && ny < dims.y as i32
                    && heights[(nx as u32 + ny as u32 * dims.x) as usize] == height + 1
            });

            if has_higher_neighbour {
                let surface = index(dims, x, y, height);
                if let Tile::Solid(material) = tiles[surface] {
                    // NOTE: Ramps only bridge the single-level steps generated in this story.
                    tiles[surface] = Tile::Ramp(material);
                }
            }
        }
    }
}

pub(crate) fn camp_origin(dims: Dims, heights: &[u32]) -> crate::Pos {
    let centre = (dims.x as i64 / 2, dims.y as i64 / 2);
    let radius = CAMP_RADIUS;

    let (x, y, height) = (radius..dims.y - radius)
        .flat_map(|y| (radius..dims.x - radius).map(move |x| (x, y)))
        .filter_map(|(x, y)| {
            let height = heights[(x + y * dims.x) as usize];
            let flat = (y - radius..=y + radius).all(|ny| {
                (x - radius..=x + radius).all(|nx| heights[(nx + ny * dims.x) as usize] == height)
            });
            flat.then_some((x, y, height))
        })
        .min_by_key(|&(x, y, _)| {
            let dx = x as i64 - centre.0;
            let dy = y as i64 - centre.1;
            (dx * dx + dy * dy, y, x)
        })
        .expect("worldgen requires one 7x7 flat camp clearing");

    crate::Pos {
        x: x as i32,
        y: y as i32,
        z: height as i32 + 1,
    }
}

pub(crate) fn place_trees(
    dims: Dims,
    heights: &[u32],
    tiles: &mut [Tile],
    camp: crate::Pos,
    rng: &mut ChaCha8Rng,
) {
    let camp_radius = CAMP_RADIUS as i32;
    let mut trunks = Vec::new();

    for y in 1..dims.y - 1 {
        for x in 1..dims.x - 1 {
            if (x as i32 - camp.x).abs() <= camp_radius + 1
                && (y as i32 - camp.y).abs() <= camp_radius + 1
            {
                continue;
            }
            if rng.random_range(0..48) != 0
                || trunks
                    .iter()
                    .any(|&(tx, ty): &(u32, u32)| tx.abs_diff(x) <= 2 && ty.abs_diff(y) <= 2)
            {
                continue;
            }

            let surface = heights[(x + y * dims.x) as usize];
            let height = rng.random_range(4..=6);
            let crown_top = surface + height;
            if crown_top >= dims.z {
                continue;
            }

            for z in surface + 1..crown_top {
                tiles[index(dims, x, y, z)] = Tile::Solid(Material::TreeTrunk);
            }
            let crown_tip = index(dims, x, y, crown_top);
            if tiles[crown_tip] == Tile::Empty {
                tiles[crown_tip] = Tile::Solid(Material::TreeFoliage);
            }
            for fy in y - 1..=y + 1 {
                for fx in x - 1..=x + 1 {
                    if fx != x || fy != y {
                        let foliage = index(dims, fx, fy, surface + 1);
                        if tiles[foliage] == Tile::Empty {
                            tiles[foliage] = Tile::Solid(Material::TreeFoliage);
                        }
                    }
                }
            }
            for z in crown_top.saturating_sub(2)..crown_top {
                for fy in y - 1..=y + 1 {
                    for fx in x - 1..=x + 1 {
                        if fx == x && fy == y {
                            continue;
                        }
                        let foliage = index(dims, fx, fy, z);
                        if tiles[foliage] == Tile::Empty {
                            tiles[foliage] = Tile::Solid(Material::TreeFoliage);
                        }
                    }
                }
            }
            trunks.push((x, y));
        }
    }
}
