#![forbid(unsafe_code)]

use std::{env, fs, path::Path};

use gui::capture::warm_lit_pixels;

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

/// Requires a display-capable machine with a cargo toolchain: run `gui --capture PATH
/// --frames 60` against the same daemon at two different ticks, then set
/// FROSTVEIN_CAPTURE_FIRST/FROSTVEIN_CAPTURE_SECOND to the two paths. As of 5.3's review
/// (2026-08-14) this test has never executed anywhere — the debt is inherited by 5.4.
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
    assert_ne!(
        fs::read(first).unwrap(),
        fs::read(second).unwrap(),
        "world changes must change capture bytes"
    );
    let first_pixels = image::open(first).unwrap().to_rgba8();
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
