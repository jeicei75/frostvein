#![forbid(unsafe_code)]

use std::{env, fs, path::Path};

/// Requires the rebelspice display/daemon harness. The harness sets the two paths
/// after capturing different daemon ticks with `gui --capture PATH --frames 60`.
/// Pixel range checking awaits the display-capable runner, where the screenshot
/// observer has produced real PNGs to inspect.
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
    assert_ne!(
        fs::read(first).unwrap(),
        fs::read(second).unwrap(),
        "world changes must change capture bytes"
    );
}
