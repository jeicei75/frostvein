#![forbid(unsafe_code)]

use std::{env, fs, path::Path};

fn non_background_pixels(path: &Path) -> usize {
    image::open(path)
        .expect("capture must be a decodable PNG")
        .to_rgba8()
        .pixels()
        .filter(|pixel| pixel.0 != [0, 0, 0, 255])
        .count()
}

/// Requires the rebelspice display/daemon harness. The harness sets the two paths
/// after capturing different daemon ticks with `gui --capture PATH --frames 60`.
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
}
