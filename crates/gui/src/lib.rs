#![forbid(unsafe_code)]

pub mod appearance;
pub mod atmosphere;
pub mod blend;
pub mod camera;
pub mod capture;
pub mod command;
pub mod designate;
pub mod ingest;
pub mod pick;
pub mod project;
pub mod slice;
pub mod transform;

/// The commit this binary was built from, stamped in by `build.rs`.
///
/// M2-7. `gui.exe` is built in WSL and copied to the Windows vehicle, so the running copy can be
/// any number of commits behind the tree a session is reasoning about — it has been stale six
/// times, and on the sixth the RUNBOOK telling you to check the mtime named the wrong build too.
/// A procedure cannot close this; a value compiled into the binary can, because it is whatever the
/// binary actually is. Printed at startup by `ingest::run`, so a vehicle session reads the commit
/// off its own console instead of inferring it from a file timestamp.
///
/// `-dirty` means the build had uncommitted changes and the SHA does NOT describe what is running.
pub const BUILD_SHA: &str = env!("GUI_BUILD_SHA");

#[cfg(test)]
mod build_stamp {
    /// The stamp is only worth anything if it is REAL. A build script that silently failed would
    /// leave an empty string, and an empty stamp reads on the console as "no stamp feature yet"
    /// rather than "your build is broken" — so this pins the shape, not merely the presence.
    #[test]
    fn build_sha_is_a_real_commit_or_says_it_does_not_know() {
        let sha = super::BUILD_SHA;
        assert!(!sha.is_empty(), "the build stamp must never be empty");
        if sha == "unknown" {
            return;
        }
        let core = sha
            .strip_suffix("-dirty")
            .or_else(|| sha.strip_suffix("-unknown-dirtiness"))
            .unwrap_or(sha);
        assert!(
            core.len() >= 7 && core.chars().all(|c| c.is_ascii_hexdigit()),
            "the build stamp must be a short SHA, optionally suffixed: got {sha:?}"
        );
    }
}
