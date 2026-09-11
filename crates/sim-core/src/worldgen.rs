use rand::RngExt;
use rand_chacha::ChaCha8Rng;

use crate::{Dims, Material, Tile};

const NOISE_SPACING: u32 = 32;
pub(crate) const CAMP_RADIUS: u32 = 3;
const RIDGE_BAND: u32 = 6;
const RIDGE_RAISE: u32 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Biome {
    Snowfield,
    Lake,
}

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

fn biome_at(dims: Dims, x: u32, y: u32) -> Biome {
    // NOTE: this single low-frequency field is deliberately hardcoded until the world needs a
    // third biome. The threshold shapes the input; materials are never border-blended.
    let centre_x = dims.x as f64 * 0.22;
    let centre_y = dims.y as f64 * 0.72;
    let dx = (x as f64 - centre_x) / (dims.x as f64 * 0.075);
    let dy = (y as f64 - centre_y) / (dims.y as f64 * 0.055);
    let field = dx * dx + dy * dy + 0.12 * dx * dy;

    if field <= 1.0 {
        Biome::Lake
    } else {
        Biome::Snowfield
    }
}

fn local_gradient(dims: Dims, heights: &[u32], x: u32, y: u32) -> u32 {
    let height = heights[(x + y * dims.x) as usize];
    [
        (x as i32 - 1, y as i32),
        (x as i32 + 1, y as i32),
        (x as i32, y as i32 - 1),
        (x as i32, y as i32 + 1),
    ]
    .into_iter()
    .filter(|&(nx, ny)| nx >= 0 && ny >= 0 && nx < dims.x as i32 && ny < dims.y as i32)
    .map(|(nx, ny)| height.abs_diff(heights[(nx as u32 + ny as u32 * dims.x) as usize]))
    .max()
    .unwrap_or(0)
}

fn surface_material(biome: Biome, gradient: u32, x: u32, y: u32) -> Material {
    match biome {
        Biome::Lake => Material::Ice,
        // A coarse field keeps scouring in broad, readable patches instead of making each
        // stepped cell flip independently. Only sloped ground can lose its snow cover.
        //
        // Scoured ground is ROCK, not ice. It emitted ice until 10.9's closing sitting, where the
        // patches read as artificial plates: ice is the one material the client draws dead flat
        // (`material_detail_depth`, relief depth 0 -- which is what a frozen lake wants), so a
        // scour patch was a mirror-smooth slab laid over a slope. Stone takes relief depth 1 and
        // breaks up. It also leaves ice as the LAKE's exclusive material, which is the uniqueness
        // AC9 claimed. Wolf ruled 2026-09-11 that the snow cap stays on stone: `has_snow_cap`
        // excludes ice and soil but not stone, and that rule was left alone.
        Biome::Snowfield if gradient > 0 && (x / 16 + y / 16).is_multiple_of(5) => Material::Stone,
        Biome::Snowfield => Material::Snow,
    }
}

pub(crate) fn apply_lake(dims: Dims, heights: &mut [u32]) {
    let lake_height = (0..dims.y)
        .flat_map(|y| (0..dims.x).map(move |x| (x, y)))
        .filter(|&(x, y)| biome_at(dims, x, y) == Biome::Lake)
        .map(|(x, y)| heights[(x + y * dims.x) as usize])
        .min()
        .expect("lake footprint is non-empty");

    for y in 0..dims.y {
        for x in 0..dims.x {
            if biome_at(dims, x, y) == Biome::Lake {
                heights[(x + y * dims.x) as usize] = lake_height;
            }
        }
    }
    clamp_steps(dims, heights);
}

fn in_ridge_band(dims: Dims, x: u32, y: u32) -> bool {
    x < RIDGE_BAND || y >= dims.y.saturating_sub(RIDGE_BAND)
}

#[cfg(test)]
fn in_ridge_footprint(dims: Dims, x: u32, y: u32) -> bool {
    let ripple = RIDGE_BAND + RIDGE_RAISE;
    x < ripple || y >= dims.y.saturating_sub(ripple)
}

#[cfg(test)]
fn in_lake_footprint(dims: Dims, x: u32, y: u32) -> bool {
    // The lake's flat ice is its core; the dilation below covers the stepped shore `clamp_steps`
    // leaves around it, which is part of the lake footprint rather than an unrelated terrain
    // change.
    //
    // NOTE: the +/-4 is a GENEROUS BOUND, not a measurement. On `DEFAULT_SEED` the basin's actual
    // cut is ONE level (pre-flatten heights inside the lake biome are 17..=18), so the real shore
    // is one cell and this exempts four. Nothing in `apply_lake` bounds how far `lake_height` can
    // sit below the rim -- that gap is whatever `height_field` produced for the seed -- so the
    // constant cannot be derived the way `in_ridge_footprint`'s `RIDGE_BAND + RIDGE_RAISE` is.
    // Raise it if a seed ever cuts deeper; do not read it as a proven maximum.
    (-4_i32..=4).any(|dy| {
        (-4_i32..=4).any(|dx| {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            nx >= 0
                && ny >= 0
                && nx < dims.x as i32
                && ny < dims.y as i32
                && biome_at(dims, nx as u32, ny as u32) == Biome::Lake
        })
    })
}

pub(crate) fn apply_ridges(dims: Dims, heights: &mut [u32]) {
    for y in 0..dims.y {
        for x in 0..dims.x {
            if in_ridge_band(dims, x, y) {
                let column = (x + y * dims.x) as usize;
                heights[column] = heights[column].saturating_add(RIDGE_RAISE).min(dims.z - 2);
            }
        }
    }
    clamp_steps(dims, heights);
}

pub(crate) fn layered_terrain(dims: Dims, heights: &[u32]) -> Vec<Tile> {
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
            let surface = surface_material(
                biome_at(dims, x, y),
                local_gradient(dims, heights, x, y),
                x,
                y,
            );
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
            if biome_at(dims, x, y) == Biome::Lake {
                continue;
            }
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
            // NO FOLIAGE RING AT surface + 1. Removed 2026-08-29 on Wolf's ruling at 9.4's
            // review, after he saw it on the vehicle as "green boxes on ground level next to some
            // full trees". A ring of foliage cubes sitting ON the ground did three wrong things at
            // once: it read as foliage lying on the terrain rather than as a tree; it enclosed the
            // lower trunk on all four sides, so 86 of 265 trees (every height-4 tree) had no
            // exposed trunk cell and drew no trunk at all; and it broke on slopes, because it was
            // stamped at the TRUNK's surface across all eight neighbours regardless of their own
            // terrain -- 285 cubes hung in the air over lower ground and 296 were dropped where
            // higher ground was in the way. Trees now taper from a bare trunk into the crown rings
            // below, which is also the shape the approved artifact shows.
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

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;
    use crate::{DEFAULT_SEED, STREAM_TREES, STREAM_WORLDGEN};

    #[test]
    fn height_field_for_the_default_seed_stays_pinned_before_post_passes() {
        let mut rng = ChaCha8Rng::seed_from_u64(DEFAULT_SEED ^ STREAM_WORLDGEN);
        let heights = height_field(Dims::DEFAULT, &mut rng);

        let samples = [(0, 0), (64, 64), (127, 127), (20, 92)];
        let actual: Vec<_> = samples
            .into_iter()
            .map(|(x, y)| heights[(x + y * Dims::DEFAULT.x) as usize])
            .collect();
        assert_eq!(actual, vec![15, 8, 5, 18]);
    }

    #[test]
    fn ridges_only_change_the_far_edge_footprint_and_lake() {
        let mut rng = ChaCha8Rng::seed_from_u64(DEFAULT_SEED ^ STREAM_WORLDGEN);
        let before = height_field(Dims::DEFAULT, &mut rng);
        let mut after = before.clone();
        apply_lake(Dims::DEFAULT, &mut after);
        apply_ridges(Dims::DEFAULT, &mut after);

        for y in 0..Dims::DEFAULT.y {
            for x in 0..Dims::DEFAULT.x {
                if !in_ridge_footprint(Dims::DEFAULT, x, y)
                    && !in_lake_footprint(Dims::DEFAULT, x, y)
                {
                    assert_eq!(
                        after[(x + y * Dims::DEFAULT.x) as usize],
                        before[(x + y * Dims::DEFAULT.x) as usize],
                        "post-passes reached ({x},{y}) outside their footprints"
                    );
                }
            }
        }

        assert_eq!(after[0], before[0] + RIDGE_RAISE);
        let upper_right = (Dims::DEFAULT.x - 1) as usize
            + (Dims::DEFAULT.y - 1) as usize * Dims::DEFAULT.x as usize;
        assert_eq!(after[upper_right], before[upper_right] + RIDGE_RAISE);
    }

    #[test]
    fn camps_stay_outside_the_lake_for_the_default_seed_and_fifty_more() {
        for seed in DEFAULT_SEED..DEFAULT_SEED + 51 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_WORLDGEN);
            let mut heights = height_field(Dims::DEFAULT, &mut rng);
            apply_lake(Dims::DEFAULT, &mut heights);
            apply_ridges(Dims::DEFAULT, &mut heights);
            let camp = camp_origin(Dims::DEFAULT, &heights);
            assert_ne!(
                biome_at(Dims::DEFAULT, camp.x as u32, camp.y as u32),
                Biome::Lake
            );
            if seed == DEFAULT_SEED {
                assert_eq!(camp, crate::Pos { x: 64, y: 64, z: 9 });
            }
        }
    }

    #[test]
    fn biome_decision_is_consumed_by_the_surface_material_rule() {
        let height = 7;
        let tiles = layered_terrain(
            Dims::DEFAULT,
            &vec![height; (Dims::DEFAULT.x * Dims::DEFAULT.y) as usize],
        );

        for ((x, y), expected) in [((64, 64), Material::Snow), ((28, 92), Material::Ice)] {
            assert_eq!(
                tiles[index(Dims::DEFAULT, x, y, height)],
                Tile::Solid(expected),
                "the biome lookup must select {expected:?} at ({x},{y})",
            );
        }
    }

    #[test]
    fn lake_post_pass_flattens_its_entire_ice_core() {
        let mut rng = ChaCha8Rng::seed_from_u64(DEFAULT_SEED ^ STREAM_WORLDGEN);
        let mut heights = height_field(Dims::DEFAULT, &mut rng);
        apply_lake(Dims::DEFAULT, &mut heights);

        let lake_heights: std::collections::BTreeSet<_> = (0..Dims::DEFAULT.y)
            .flat_map(|y| (0..Dims::DEFAULT.x).map(move |x| (x, y)))
            .filter(|&(x, y)| biome_at(Dims::DEFAULT, x, y) == Biome::Lake)
            .map(|(x, y)| heights[(x + y * Dims::DEFAULT.x) as usize])
            .collect();
        assert_eq!(
            lake_heights.len(),
            1,
            "lake core heights were {lake_heights:?}"
        );
    }

    #[test]
    fn trees_do_not_grow_out_of_the_lake() {
        let mut terrain_rng = ChaCha8Rng::seed_from_u64(DEFAULT_SEED ^ STREAM_WORLDGEN);
        let mut heights = height_field(Dims::DEFAULT, &mut terrain_rng);
        apply_lake(Dims::DEFAULT, &mut heights);
        apply_ridges(Dims::DEFAULT, &mut heights);
        let camp = camp_origin(Dims::DEFAULT, &heights);
        let mut tiles = layered_terrain(Dims::DEFAULT, &heights);
        let mut tree_rng = ChaCha8Rng::seed_from_u64(DEFAULT_SEED ^ STREAM_TREES);
        place_trees(Dims::DEFAULT, &heights, &mut tiles, camp, &mut tree_rng);

        for y in 0..Dims::DEFAULT.y {
            for x in 0..Dims::DEFAULT.x {
                if biome_at(Dims::DEFAULT, x, y) != Biome::Lake {
                    continue;
                }
                let height = heights[(x + y * Dims::DEFAULT.x) as usize];
                assert!(
                    (height + 1..Dims::DEFAULT.z).all(|z| !matches!(
                        tiles[index(Dims::DEFAULT, x, y, z)],
                        Tile::Solid(Material::TreeTrunk)
                    )),
                    "lake column ({x},{y}) grew a tree"
                );
            }
        }
    }
}
