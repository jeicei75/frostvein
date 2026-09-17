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
    // Re-run on EVERY build. Cargo does that for a `rerun-if-changed` path that does not exist,
    // and the cost is two `git` calls per build against a stamp whose whole value is that it
    // never lies.
    //
    // It used to watch `../../.git/HEAD` and `../../.git/index`, and both are blind to a commit.
    // Measured 2026-09-17: the pre-commit hook's own gate build runs this script while the tree is
    // mid-commit — files staged, HEAD still at the PARENT — so it bakes `<parent>-dirty`. The
    // commit then moves `.git/refs/heads/<branch>` (08:14:12) while `.git/index` only matches the
    // script's own run (08:13:16) and `.git/HEAD` never moves at all on a same-branch commit. So
    // neither watched path is newer afterwards, the cached output is reused, and every binary
    // built through the normal commit flow carries the WRONG commit and a FALSE `-dirty`.
    // Reproduced twice; `touch crates/gui/build.rs` was the only cure.
    //
    // Watching the resolved ref as well would fix the commit case and leave the worse one open:
    // working-tree dirtiness flips without any ref moving, which is the half this module's doc
    // calls the more dangerous one.
    println!("cargo:rerun-if-changed=.git-stamp-recomputed-every-build");

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
