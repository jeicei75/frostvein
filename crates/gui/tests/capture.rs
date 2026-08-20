#![forbid(unsafe_code)]

use std::{env, fs, path::Path};

use gui::{camera::CameraRig, capture::warm_lit_pixels};

/// Counts pixels differing from the image's dominant colour. Bevy's clear colour is a
/// grey, not black, so "not pure black" would pass an empty scene; the dominant colour
/// is the background whatever the renderer painted it.
fn non_background_pixels(path: &Path) -> usize {
    let image = image::open(path)
        .expect("capture must be a decodable PNG")
        .to_rgba8();
    let mut counts = std::collections::HashMap::new();
    for pixel in image.pixels() {
        *counts.entry(pixel.0).or_insert(0usize) += 1;
    }
    let background = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(pixel, _)| pixel)
        .expect("capture must contain pixels");
    image.pixels().filter(|pixel| pixel.0 != background).count()
}

/// Requires a display-capable machine with a cargo toolchain. The comparison is intentionally
/// restricted to the projected dig-site window: snowfall alone makes full PNG bytes differ.
#[test]
#[ignore = "requires a real render surface; excluded from the headless gate"]
fn capture_exists_is_not_black_and_changes_with_the_world() {
    let first = env::var_os("FROSTVEIN_CAPTURE_FIRST").expect("first capture path is required");
    let second = env::var_os("FROSTVEIN_CAPTURE_SECOND").expect("second capture path is required");
    let first = Path::new(&first);
    let second = Path::new(&second);
    assert!(
        first.is_file() && second.is_file(),
        "both capture files must exist"
    );
    assert!(
        fs::metadata(first).unwrap().len() > 0,
        "first PNG must not be empty"
    );
    assert!(
        fs::metadata(second).unwrap().len() > 0,
        "second PNG must not be empty"
    );
    assert!(
        non_background_pixels(first) > 0,
        "first capture must contain non-background pixels before comparison"
    );
    assert!(
        non_background_pixels(second) > 0,
        "second capture must contain non-background pixels before comparison"
    );
    let first_pixels = image::open(first).unwrap().to_rgba8();
    let second_pixels = image::open(second).unwrap().to_rgba8();
    assert_eq!(first_pixels.dimensions(), second_pixels.dimensions());
    let rig = CameraRig::new([64, 64, 9]);
    let projected = (55..=56)
        .flat_map(|x| (62..=65).map(move |y| [x, y, 9]))
        .map(|point| {
            rig.project_world_point(point)
                .expect("dig site must project")
        })
        .collect::<Vec<_>>();
    let width = first_pixels.width() as f32;
    let height = first_pixels.height() as f32;
    let min_x = projected.iter().map(|p| p.x).fold(1.0, f32::min) - 0.02;
    let max_x = projected.iter().map(|p| p.x).fold(0.0, f32::max) + 0.02;
    let min_y = projected.iter().map(|p| p.y).fold(1.0, f32::min) - 0.02;
    let max_y = projected.iter().map(|p| p.y).fold(0.0, f32::max) + 0.02;
    let changes = first_pixels
        .enumerate_pixels()
        .filter(|(x, y, pixel)| {
            let inside = (*x as f32 / width >= min_x)
                && (*x as f32 / width <= max_x)
                && (*y as f32 / height >= min_y)
                && (*y as f32 / height <= max_y);
            let other = second_pixels.get_pixel(*x, *y);
            let distance = pixel.0[..3]
                .iter()
                .zip(other.0[..3].iter())
                .map(|(a, b)| a.abs_diff(*b) as u16)
                .sum::<u16>();
            inside && distance > 30
        })
        .count();
    // Measured on the 2026-08-17 pair over the ORIGINAL site's window: 1,651 pixels changed
    // inside it against 604 across the whole rest of the frame, i.e. ~5 expected inside from
    // snowfall and aurora alone. A bare `> 0` would therefore pass on atmosphere with ~99.5%
    // probability -- the same vacuity the whole-file byte comparison had, just smaller.
    // NOTE: the floor is carried over to the re-picked site (same tile count, same v-band, a
    // narrower u-span) and is deliberately conservative at ~12% of that measured signal. The
    // count is printed below, so the first vehicle run on the new site re-calibrates it on
    // evidence rather than on this inference.
    const DIG_SITE_CHANGED_PIXEL_FLOOR: usize = 200;
    // Reported before it is asserted, for the same reason the motion line is: a bare pass tells
    // the operator the bar was cleared but not by how much, and the margin is the interesting
    // number when the question is whether the dig reads at all.
    println!(
        "dig-site window: {changes} changed pixels (floor {DIG_SITE_CHANGED_PIXEL_FLOOR}), \
         window u {min_x:.3}-{max_x:.3} v {min_y:.3}-{max_y:.3}"
    );
    assert!(
        changes >= DIG_SITE_CHANGED_PIXEL_FLOOR,
        "the dig-site window must carry real change, not atmosphere: {changes} pixels differ, \
         floor is {DIG_SITE_CHANGED_PIXEL_FLOOR}"
    );
    let pixels = first_pixels
        .pixels()
        .map(|pixel| pixel.0)
        .collect::<Vec<_>>();
    assert!(
        warm_lit_pixels(&pixels) > 0,
        "first capture must contain warm-lit pixels by the named threshold"
    );
}

#[test]
fn warm_pixel_threshold_requires_red_to_exceed_blue_by_the_named_margin() {
    assert_eq!(warm_lit_pixels(&[[220, 120, 150, 255]]), 1);
    assert_eq!(warm_lit_pixels(&[[180, 120, 150, 255]]), 0);
}
