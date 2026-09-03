//! Stamps the commit the binary was built from into the binary itself.
//!
//! M2-7, open since M2 began and re-noted in five stories without ever being automated. The trap
//! it closes: `gui.exe` is built in WSL and COPIED to the Windows vehicle, so the copy can be an
//! arbitrary number of commits behind the tree the session is reasoning about. It has been stale
//! six times. Every previous guard was a procedure — "check the mtime", "rebuild first" — and a
//! procedure is exactly what a stale binary defeats, because the runbook telling you to check can
//! itself name the wrong build (it did, on 2026-08-28).
//!
//! A stamp compiled INTO the binary cannot go stale: it is whatever the binary actually is.

use std::process::Command;

fn main() {
    // Rerun when HEAD moves. Without these the stamp is frozen at whatever the first build saw,
    // which would make this worse than nothing — a stamp that lies is trusted, a missing one is not.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/index");

    println!("cargo:rustc-env=GUI_BUILD_SHA={}", sha());
    // NOTE: `GUI_WORKSPACE_ROOT` used to be stamped here too. Its only consumer was
    // `resolve_asset_root`, deleted when the pines were embedded -- and it stamped THIS machine's
    // absolute Linux path into a binary that gets copied to Windows, which is the exact artefact
    // the embedding fix existed to remove. Removed with its consumer.
}

/// The short SHA, suffixed `-dirty` when the working tree has uncommitted changes.
///
/// NOTE: untested, and deliberately so — a build script is not compiled into any test target,
/// so covering this would mean extracting a seam for the sole purpose of testing it. One
/// expression, in plain sight, is the trade. See the note in the mutation table.
///
/// A bare SHA on a dirty tree is the more dangerous half of this problem: it names a commit whose
/// content is NOT what is running. `unknown` when git cannot answer — a source tarball, or a build
/// outside a repo — because a fabricated stamp is worse than an absent one.
fn sha() -> String {
    let Some(sha) = git(&["rev-parse", "--short", "HEAD"]) else {
        return "unknown".to_string();
    };
    match git(&["status", "--porcelain"]) {
        Some(status) if !status.is_empty() => format!("{sha}-dirty"),
        Some(_) => sha,
        None => format!("{sha}-unknown-dirtiness"),
    }
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}
