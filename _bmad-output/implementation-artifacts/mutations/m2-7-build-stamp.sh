# Mutation set for M2-7 (the gui build stamp). Run alone: scripts/mutate.sh <this file>

# The whole point of the stamp is that it is REAL. A build script that quietly stops resolving the
# commit leaves a placeholder, and a placeholder on the console reads as "this build predates the
# stamp" rather than "your stamp is broken" -- which is the same silent-no-op shape M2-7 exists to
# close. This row proves the shape check actually executes.
mutation "build stamp degrades to a placeholder instead of the real commit" gui build_sha_is_a_real_commit_or_says_it_does_not_know <<'PY'
import pathlib
p = pathlib.Path('crates/gui/build.rs'); s = p.read_text()
old = '    println!("cargo:rustc-env=GUI_BUILD_SHA={}", sha());\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    println!("cargo:rustc-env=GUI_BUILD_SHA=dev");\n'))
PY

# NOTE: the `-dirty` suffix has NO row and no test. A build script is not compiled into any test
# target, so reaching that branch would mean building a seam for the sole purpose of testing it,
# which this repo's YAGNI policy forbids. The limitation is named rather than engineered around:
# the suffix is one expression, visible in `build.rs`, and its failure mode (a dirty build
# stamped as a clean commit) is caught by the row above only if the stamp stops resolving
# entirely. If a dirty-stamp defect ever actually fires, that is the trigger to extract the
# classifier into an `include!`d module both `build.rs` and the lib compile.
