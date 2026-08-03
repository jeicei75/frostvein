---
baseline_commit: ebd27dd4e4474cfe8567bf3222aa857e6e374b88
---

# Story 1.3: Behold the Frozen World

Status: done

## Story

As the boss,
I want to see the icy world in my terminal in truecolor and walk its z-levels,
so that I can behold the fortress site and judge the icy-grim look.

## Acceptance Criteria

1. `tui` depends only on `protocol`, `crossterm` 0.29.0, `serde_json`, and `anyhow`; `cargo tree -p tui | rg sim-core` returns nothing and the client holds no game rule (no terrain semantics, no derived world logic) — it renders what the wire says.
2. Launching `tui` connects to `127.0.0.1:protocol::DEFAULT_PORT`, reads exactly one `\n`-terminated line, and decodes it into `protocol::Snapshot`. An optional single positional arg overrides the port. A line that is not a decodable snapshot exits with an `anyhow` error naming the cause — never a panic, never a hang.
3. The id → look mapping is one data table in `crates/tui/src/palette.rs`: each of `Tile::Solid(m)` and `Tile::Ramp(m)` for all four materials maps to a glyph plus a 24-bit RGB, `Tile::Empty` maps to blank, and `EntityKind::Dwarf` maps to `☺` plus its own RGB. No glyph or RGB literal exists at any draw site.
4. Rendering a snapshot at level `z` fills each viewport cell from `tiles[x + y*dims.x + z*dims.x*dims.y]` through the palette table; world positions outside `dims` render as background blank.
5. Entities with `pos[2] == z` overdraw their terrain cell with the dwarf glyph and color; entities on any other level are not drawn.
6. Where the viewed tile is `Empty`, the first non-empty tile within `PEEK_DEPTH` (3) levels below is drawn with its palette color scaled by that depth's dim factor; nothing within 3 levels renders as background blank.
7. `<` / `>` move the view one z-level down / up, clamped to `[0, dims.z - 1]`; arrows and `hjkl` pan the camera, clamped to `[0, dims.x-1] × [0, dims.y-1]`; `q` shows `quit? (y/n)` in the status line, `y` exits cleanly and any other key cancels. The terminal is restored (raw mode off, alternate screen left, cursor shown) on every exit path including an error return.
8. The initial view is centered on the first entity's `(x, y)` at that entity's `z`; with no entities it centers on the world's middle column at `dims.z / 2`.
9. The bottom row is a status line showing the current z (`z 14/31`), the camera position, the dwarf count, and the active keys. The map occupies every row above it.
10. A frame reaches the terminal as one buffered write per frame — one function serializes the whole framebuffer into a writer and the caller flushes once. A 2×1 framebuffer emits exactly the byte sequence pinned in its test; no per-cell terminal write exists anywhere.
11. `tui --frame` renders one frame to stdout with no raw mode and no alternate screen, then exits 0. It renders at the size crossterm reports, falling back to 100×40 only when that call errors or reports a zero dimension (amended 2026-08-03: the original text promised the fallback for a missing TTY, which does not happen on Linux — crossterm shells out to `tput` and gets terminfo's 80×24 guess, so a headless frame is 80×24, not 100×40). This is how the view is checked without a TTY.
12. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` pass. Wolf's live sign-off on the icy-grim look (FR23) is a separate manual gate — the dev agent reports the live output, it does not claim the sign-off.

## Tasks / Subtasks

- [x] **Dependencies** (AC: 1)
  - [x] Root `Cargo.toml` `[workspace.dependencies]`: add `crossterm = "0.29.0"` (already on the closed stack — no new-dependency justification needed; latest stable, verified crates.io 2026-08-03).
  - [x] `crates/tui/Cargo.toml`: add `crossterm.workspace = true`, `serde_json.workspace = true`, `anyhow.workspace = true` alongside the existing `protocol` path dep. Nothing else.
  - [x] **Run `cargo fetch` with network first** — crossterm and its unix deps (`rustix`/`signal-hook`/`mio`/`parking_lot`) are in neither `Cargo.lock` nor the local registry cache. Then build `--offline`.

- [x] **`crates/tui/src/palette.rs`** — the one data table (AC: 3, 6)
  - [x] `pub struct Cell { pub glyph: char, pub fg: Rgb }`, `pub type Rgb = (u8, u8, u8)`, `pub const BACKGROUND: Rgb`, `pub const BLANK: Cell` (space on background).
  - [x] `pub fn tile_cell(tile: protocol::Tile) -> Cell` and `pub fn entity_cell(kind: protocol::EntityKind) -> Cell`, both exhaustive `match`, no wildcard arm — a new material or entity kind must be a compile error here.
  - [x] `pub fn dim(fg: Rgb, depth: u8) -> Rgb` — integer percentage scale per depth from `const DIM_PERCENT: [u16; PEEK_DEPTH] = [55, 35, 22]`; `depth` 0 = undimmed.
  - [x] Values (tune live for FR23, keep them cold and desaturated): `Solid(Stone)` `█` (86,92,104) · `Solid(Soil)` `▓` (72,66,58) · `Solid(Ice)` `▒` (126,174,196) · `Solid(Snow)` `░` (206,218,228) · `Ramp(m)` `▲` in that material's RGB · `Empty` blank · `Dwarf` `☺` (214,154,78) · `BACKGROUND` (8,10,14) · status text (150,160,170).
  - [x] Test `every_look_is_pinned` — a hand-written literal table of `(value, glyph, rgb)` for all 4 materials × {Solid, Ramp}, `Empty`, and `Dwarf`, compared against the functions. Independent oracle, not a round-trip: swapping `Ice`↔`Snow` must fail it.
  - [x] Test `dim_darkens_monotonically` — `dim(fg, 1..3)` each channel strictly below the previous depth and below `fg`, pinned against the literal expected triples for one input.

- [x] **`crates/tui/src/view.rs`** — pure render + input state (AC: 4, 5, 6, 7, 8, 9)
  - [x] `pub struct Framebuffer { pub w: u16, pub h: u16, pub cells: Vec<Cell> }` (row-major, `w*h` long) with `pub fn cell(&self, x: u16, y: u16) -> Cell`.
  - [x] `pub struct ViewState { pub camera: (i64, i64), pub z: i32, pub confirming_quit: bool }` + `pub fn initial(snapshot: &Snapshot) -> ViewState` implementing AC8.
  - [x] `pub fn render(snapshot: &Snapshot, state: &ViewState, w: u16, h: u16) -> Framebuffer` — bottom row is the status line, map viewport is `h - 1` rows; `screen_x = wx - camera.0 + vw/2` (integer division), out-of-world → `BLANK`; terrain via `tile_cell` + peek-below, then entities overdrawn.
  - [x] `pub fn apply_key(state: &mut ViewState, key: KeyCode, dims: Dims) -> Action` with `pub enum Action { Redraw, Quit, Ignore }`; handles `<`/`>`, arrows + `hjkl`, `q`/`y`/anything-else per AC7. Only these keys — see scope guardrails.
  - [x] Test `renders_the_viewed_level` — hand-build a 5×3×3 snapshot (deliberately non-square so an x/y transposition fails) with a known tile mix; assert the **full expected grid** of `(glyph, fg)` written as a literal, covering a solid cell, a ramp cell, a peeked cell, and an out-of-depth blank.
  - [x] Test `entities_draw_only_on_the_viewed_level` — two entities at different z; rendering the lower z shows exactly one `☺` at the expected screen cell, and the other entity's screen cell holds its terrain glyph, not `☺`. The negative half is the point.
  - [x] Test `out_of_world_cells_are_blank` — camera at a corner; assert cells beyond `dims` are `BLANK`.
  - [x] Test `keys_move_and_clamp` — `>` at `dims.z-1` stays, `<` at 0 stays, pan clamps at both bounds on both axes; `q` sets `confirming_quit` and returns `Redraw`, then `y` returns `Quit` and any other key clears the flag and returns `Redraw`.

- [x] **`crates/tui/src/frame.rs`** — one write per frame (AC: 10)
  - [x] `pub fn write_frame(out: &mut dyn Write, fb: &Framebuffer) -> io::Result<()>` — crossterm `queue!` of `MoveTo`, `SetBackgroundColor`, `SetForegroundColor(Color::Rgb{..})` (emit a color only when it changes from the previous cell) and `Print`; the caller flushes. No `execute!` in the render path.
  - [x] Test `frame_bytes_are_pinned` — a 2×1 framebuffer with two different colors written into a `Vec<u8>`; assert the **exact** byte string (hand-written, including `\x1b[38;2;R;G;Bm`). A changed escape format or a dropped color must fail it.

- [x] **`crates/tui/src/main.rs`** — connect, terminal, event loop (AC: 1, 2, 7, 11)
  - [x] Args: iterate `std::env::args_os()`; `--frame` sets the flag, any other arg parses as the port with `anyhow` context naming the bad value and the valid range (mirror `crates/simd/src/main.rs:41-51`).
  - [x] `fn read_snapshot(reader: &mut dyn BufRead) -> anyhow::Result<Snapshot>` — `read_line`, error if empty (server closed), `serde_json::from_str` with context. Tests feed it a `std::io::Cursor`: one valid line, one garbage line asserting `Err`, one empty reader asserting `Err`.
  - [x] `TcpStream::connect(("127.0.0.1", port))` with context naming the address; wrap in `BufReader` and call `read_snapshot`. **Do not `shutdown`, do not close the write half** — the daemon tears the connection down at read EOF (`crates/simd/src/main.rs:113-118`), which Story 2.1 needs intact.
  - [x] `--frame` path: `terminal::size().unwrap_or((100, 40))`, `render`, `write_frame` to a `BufWriter<Stdout>`, flush, return.
  - [x] Interactive path: a `TerminalGuard` struct whose `Drop` disables raw mode, leaves the alternate screen, and shows the cursor — constructed after `enable_raw_mode` + `EnterAlternateScreen` + `Hide` so every exit path (error, panic unwind, `q`) restores the terminal.
  - [x] Loop: draw, then `event::read()`; on `Event::Key` with `kind == KeyEventKind::Press` call `apply_key` and act on the `Action`; on `Event::Resize` redraw at the new size; ignore everything else.

- [x] **Green gate** (AC: 12) — run the four commands under Verification, then the live check, and report what the live check printed.

### Review Findings

Code review 2026-08-03 — four layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor, Feature
Auditor). Feature Auditor traced every hop argv → glyph → exit under a real pty: **no capability is
unwired**, and no scope guardrail was violated. All findings below are at the last hop (bytes → the
boss's terminal) or in test coverage.

- [x] [Review][Patch] Ctrl-C cannot quit the interactive client — raw mode clears `ISIG`, so Ctrl-C arrives as `KeyCode::Char('c')` and falls into `_ => Action::Ignore` [crates/tui/src/main.rs:98, crates/tui/src/view.rs:143-183] — **Wolf's decision 2026-08-03: bind Ctrl-C to quit immediately**, no y/n confirmation; the keymap widens by one binding because "unkillable app" is the worse trap
- [x] [Review][Decision] `q` confirmation replaces the entire status line rather than overlaying — **Wolf's decision 2026-08-03: leave it**, full replacement is intended; the prompt is transient and the status returns on cancel
- [x] [Review][Patch] No `ResetColor` and no trailing newline — every exit leaves the shell painted black-on-black [crates/tui/src/frame.rs:11-45, crates/tui/src/main.rs:28-35]
- [x] [Review][Patch] Frame has no per-row positioning or row terminators — the story's own `--frame | head -45` check is a no-op (0 newlines in 12 816 bytes) and any width mismatch shears the map [crates/tui/src/frame.rs:16-42]
- [x] [Review][Patch] `PEEK_DEPTH` cap is pinned by no test — raising it 3→6 leaves all 13 tests green, so the checked "out-of-depth blank" subtask is not actually covered [crates/tui/src/view.rs:214-260]
- [x] [Review][Patch] Decoded snapshots are never validated against `dims` — a decodable-but-short `tiles` array panics with index-out-of-bounds, and `tile_index` multiplies untrusted u32 dims before widening (verified: both panic) [crates/tui/src/main.rs:116-128, crates/tui/src/view.rs:79,186-188]
- [x] [Review][Patch] No socket read timeout and unbounded `read_line` — a peer that accepts and stays silent hangs forever; one that streams without a newline drove RSS to 5.4 GB in 4 s [crates/tui/src/main.rs:60-63,117-120]
- [x] [Review][Patch] Status-line content asserted only 7 characters deep — camera coords, dwarf count and key hints are pinned by nothing [crates/tui/src/view.rs:115-138]
- [x] [Review][Patch] Key modifiers are discarded — Ctrl-H/J/K/L pan the camera and Ctrl-Q opens the quit prompt [crates/tui/src/main.rs:98, crates/tui/src/view.rs:143-183]
- [x] [Review][Patch] `BufWriter::new` default 8 KiB splits a frame across 2–4 `write` syscalls, so AC10's "one buffered write per frame" is literally false [crates/tui/src/main.rs:69,83]
- [x] [Review][Patch] Port error text "0 = OS-assigned" is a `bind` semantic copied from the daemon; for a `connect` client 0 is never valid [crates/tui/src/main.rs:54]
- [x] [Review][Defer] A panic in the interactive loop is invisible — the message prints to the alternate screen, which `TerminalGuard` then discards [crates/tui/src/main.rs:20-27,82] — deferred
- [x] [Review][Defer] No SIGTERM/SIGHUP handling — a killed client leaves the terminal in raw mode [crates/tui/src/main.rs:26-35] — deferred
- [x] [Review][Defer] AC11's documented 100×40 fallback is unreachable on Linux — crossterm shells out to `tput` before returning `Err`, so a no-TTY frame renders at 80×24 (verified: 1920 cells). The AC text describes something unobservable [crates/tui/src/main.rs:67] — deferred, spec-accuracy issue

**Post-patch verification (2026-08-03).** All 10 patches applied and re-verified live, not just by
suite: `--frame` now emits 24 newline-terminated rows ending in `\x1b[0m` and `--frame | head -45`
renders the map (it printed one 12 816-byte line before); a snapshot whose `tiles` disagree with
`dims` exits 1 with `snapshot has 2 tiles but dims 4x4x4 need 64` instead of panicking; a peer that
accepts and stays silent now fails at 31 s instead of hanging forever; and under a real pty Ctrl-C
exits 0 with the alternate screen left, cursor shown and colour reset. Sabotage re-checked: widening
`PEEK_DEPTH` 3→6 now fails `peek_below_stops_at_three_levels` (it left all 13 tests green before),
and the Ice↔Snow swap fails 3 tests. Gate: fmt clean, clippy clean, **39 tests** (was 33).

**FR23 — SIGNED OFF by Wolf 2026-08-03**, after running the interactive client in a real terminal.
The icy-grim look passes; the palette table ships as tuned. This closes the last AC that no agent
could close, and it was the only gate standing between `done` and the PR.

**Still unexercised live** (unit-tested only, stated rather than inferred green): `initial()`'s
no-entity branch — the daemon always spawns 5 dwarves — and the `w == 0` / `h == 0` resize guards,
which `TIOCSWINSZ` will not accept.

Dismissed as noise (6): `dim()` "panics at depth ≥ 4" (the `[u16; PEEK_DEPTH]` type makes widening a
compile error, and the sole caller passes 1..=`PEEK_DEPTH`); status-line truncation on narrow
terminals (cosmetic); uncapped framebuffer allocation on an extreme resize (not reachable);
`NO_COLOR` stripping truecolor (correct crossterm behaviour — but **Wolf must not have `NO_COLOR`
set when judging FR23**; note that Codex's sandbox did export it, so its own frame was colourless —
the orchestrator's live check outside the sandbox did carry full RGB); and a suspected
self-referential `index` helper in the render test, which Blind Hunter sabotage-tested and cleared.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No commands upstream.** The client sends zero bytes in this story (AD-10). No `designate`, `set_speed`, `save`, `load`, `quit` message — and none of the keys `d c p x Space + - S L v`, which belong to Epic 2/3.
- **No delta handling, no reconnect, no re-render loop on a timer.** One snapshot, then redraw only on key/resize. Story 2.1 brings deltas.
- **No hint bar, no modal input, no cursor** (FR21, Story 3.1). One static status line is the whole chrome.
- **No raycast/3D view** (Epic 4), no mouse capture (phase 2), no TUI framework (ratatui/tui-rs) — crossterm alone, per the spine's deferred list.
- **No changes to `sim-core`, `simd`, or `protocol`.** This story is `crates/tui/` + two manifests only. If you believe a wire shape is missing, stop and say so rather than adding one.
- **No color config file or runtime palette switching** — a hardcoded table is the decided design (AD conventions, "hardcoded constants at use site").

### What already exists (build on it, do not re-derive)

- `crates/tui/src/main.rs` is a 4-line port print with `#![forbid(unsafe_code)]`; the attribute stays and covers the new modules. Everything else is replaced.
- `protocol::{Snapshot, Dims, Tile, Material, Entity, EntityKind, Speed, MessageType, DEFAULT_PORT}` are final for this story [crates/protocol/src/lib.rs]. `Snapshot.designations`/`zones` are `Vec<()>` and decode from `[]`.
- The daemon serves one snapshot line (~6.9 MB, 524 288 tiles) on connect and nothing more, then reads-and-drops inbound lines [crates/simd/src/main.rs:93-135]. It sets a 30 s write timeout, so read the line promptly — do not stall between connect and read.
- Terrain shape, so you know what a correct screen looks like: surface height is `dims.z/2 ± 4` clamped to `[3, 30]`, so heights land in 12–20. At one z you see `Solid(Snow|Ice)` where `height == z`, `Solid(Soil)` one or two levels under a surface, `Solid(Stone)` deeper, `Ramp` on columns beside a one-level step, and `Empty` above [crates/sim-core/src/worldgen.rs:85-137]. Dwarves stand at `height + 1` — the empty tile above the surface [crates/sim-core/src/lib.rs:178-182], which is why AC8 starts at the dwarf's own level and AC6 draws the ground below it.
- `simd`'s arg parsing, error-context style, and `// NOTE:` convention are the house style to match [crates/simd/src/main.rs:38-61].

### Key decisions & traps

- **Color by `EntityKind`, not profession — deliberate.** The epic AC says "colored by profession/job state", but job state is FR4 and does not reach the wire until Story 2.1. `entity_cell(kind)` is the single site that gains a job/profession arm then; do not invent a client-side profession concept now (that would be a game rule in the client, violating AD-1).
- **Peek-below is in scope and capped.** A strict one-z render leaves dwarves floating over blank air (they stand at `height + 1`), which makes the FR23 look impossible to judge. `PEEK_DEPTH = 3` with a fixed dim table is the whole feature — no fog, no lighting model, no per-level configurability.
- **Panning is in scope for the same reason:** 128×128 does not fit a terminal, so without a camera most of the world is unviewable and `<`/`>` is the only navigation. Epic 3's cursor will drive this same camera.
- **`--frame` exists because the dev agent has no TTY.** It is the story's observability instrument, not a feature: one flag, no config, no output file.
- **Never `unwrap()` a terminal call or a socket read.** `anyhow` with context in `tui` (spine convention); a panic in raw mode leaves the terminal wrecked, which is why `TerminalGuard` is a `Drop` type rather than cleanup at the end of `main`.
- **Emit a color only when it changes cell-to-cell.** A full-screen frame otherwise carries ~20 bytes of SGR per cell; runs of identical terrain are the common case.
- **`h - 1` map rows.** Guard `h == 0` and `w == 0` (a resize to zero) by skipping the draw rather than panicking on the subtraction.
- **`i64` camera arithmetic.** `wx = camera.0 + sx as i64 - vw as i64 / 2` goes negative at the map edge by design; convert to a tile index only after the bounds check.

### Project Structure (files to touch)

```
Cargo.toml                              # UPDATE — crossterm 0.29.0 in [workspace.dependencies]
crates/tui/Cargo.toml                   # UPDATE — crossterm, serde_json, anyhow
crates/tui/src/main.rs                  # UPDATE — args, connect, terminal guard, event loop (+ read_snapshot tests)
crates/tui/src/palette.rs               # NEW    — glyph/RGB data table + dim + tests
crates/tui/src/view.rs                  # NEW    — Framebuffer, ViewState, render, apply_key + tests
crates/tui/src/frame.rs                 # NEW    — write_frame + pinned-bytes test
```

### Previous story intelligence (1.1, 1.2)

- Self-referential assertions have slipped through twice. Both review passes killed tests that ran the oracle and the implementation through the same function. Every mapping test here compares against a **hand-written literal**, and the dev agent verifies by sabotage: swap `Ice`↔`Snow` in the palette and confirm a test goes red before calling the task done.
- The daemon closes the connection at read EOF. Do not half-close or drop the read half to signal "no commands" — 2.1's delta stream depends on the socket staying whole [crates/simd/src/main.rs:113-118].
- The Codex sandbox is offline: `cargo fetch` while online, then build and test `--offline`.

### Verification

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo tree -p tui | rg sim-core   # must return nothing (AC1)
```

Live check (AC2, AC11 — the observable outcome, not just green tests):

```bash
cargo run -p simd &                       # prints: listening on 127.0.0.1:7373
cargo run -p tui -- --frame | head -45    # one colored frame, no TTY needed
cargo run -p tui                          # interactive: < > z-levels, arrows/hjkl pan, q → y
```

Branch: `1-3-behold-the-frozen-world`. Commit as `Völundr <jeicei75@gmail.com>`, one commit per green step, imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.3] — user story and the four source ACs
- [Source: _bmad-output/planning-artifacts/epics.md#Requirements Inventory] — FR20 (single z-level view, z-navigation), FR22 (glyphs, truecolor, color-as-data), FR23 (icy-grim, Wolf's sign-off), NFR2 (~100 ms frame budget)
- [Source: _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md#Consistency Conventions] — color table lives in `tui` and carries no RGB on the wire; framebuffer flushed once per frame; `anyhow` in the binaries; row-major index formula; `#![forbid(unsafe_code)]`
- [Source: _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md#Stack] — crossterm 0.29.0, closed dependency list
- [Source: _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-01/addendum.md#Keymap sketch] — `<`/`>` z-level, `q` quit with confirm; every other key belongs to a later story
- [Source: docs/technical-preferences.md#Anti-overengineering rules] — YAGNI as policy, `// NOTE:` convention, no single-implementation abstractions
- [Source: crates/protocol/src/lib.rs] — the exact wire types this client decodes
- [Source: crates/simd/src/main.rs:93-135] — daemon serve loop, write timeout, EOF-closes-connection NOTE
- [Source: crates/sim-core/src/worldgen.rs:85-137] and [crates/sim-core/src/lib.rs:156-193] — terrain layering and dwarf spawn height, which set the expected on-screen picture

## Dev Agent Record

### Agent Model Used

OpenAI GPT-5 (Codex)

### Debug Log References

- TDD red: palette tests failed with literal `Cell` mismatches before the mapping and dimming implementation.
- TDD red: all six view tests failed against the blank/ignore stubs before rendering and input-state implementation.
- TDD red: `frame_bytes_are_pinned` emitted `[]` before frame serialization; the test explicitly enables ANSI because the sandbox exports `NO_COLOR=1`.
- TDD red: the valid `read_snapshot` test failed with `snapshot reading is not implemented` before the reader implementation.
- Green: `cargo test --offline -p tui` — 13 passed.
- Gate: `cargo fmt --check` and `cargo clippy --offline --all-targets -- -D warnings` passed; `cargo tree --offline -p tui | rg sim-core` printed nothing.
- Sandbox blocker: `cargo test --offline` reaches `crates/simd/tests/serve.rs`, where all six existing loopback tests fail with `simd never printed its listening line: Disconnected` because bind/connect is denied.

### Completion Notes List

- Added only the closed-stack terminal dependencies and resolved them from the orchestrator-prewarmed cache with an offline build.
- Implemented the exhaustive frozen-world palette, three-level dimming, pure snapshot rendering, entity overdraw, camera/z input state, status line, and initialization behavior.
- Implemented one-buffer ANSI frame serialization with color-run suppression and a hand-written exact byte oracle.
- Implemented one-line snapshot decoding, optional port/`--frame` argument handling, zero-byte-upstream TCP use, the no-TTY frame path, and guarded interactive terminal restoration.
- Verified the palette oracle by temporarily swapping Ice and Snow. `every_look_is_pinned` failed with actual `Cell { glyph: '░', fg: (206, 218, 228) }` versus expected `Cell { glyph: '▒', fg: (126, 174, 196) }`; the swap was reverted and the test returned green.
- Did not run the prohibited live socket check and did not claim Wolf's FR23 look sign-off.
- The Green gate remains incomplete only because the sandbox denies the existing `simd` loopback integration tests; no production or test workaround was made.

**Orchestrator verification (Claude, outside the Codex sandbox, 2026-08-03)** — the sandbox-blocked
half of the gate, re-run independently:

- `cargo fmt --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` — 33 passed, 0 failed across all crates, including the six `crates/simd/tests/serve.rs`
  loopback tests Codex could not run. The sandbox failure was environmental, as reported.
- `cargo tree -p tui | rg sim-core` — no output (AC1). `tui`'s direct deps are exactly `anyhow`,
  `crossterm 0.29.0`, `protocol`, `serde_json`.
- Live check (AC2, AC11): `simd` printed `listening on 127.0.0.1:7373`; `tui --frame` exited 0 and
  emitted a single 12 816-byte frame with no TTY. The frame shows the snow/ice surface in `░`/`▒`,
  ramps in `▲`, one `☺` at (214,154,78) on the viewed level, peeked-below terrain in dimmed variants
  (e.g. `(69,95,107)` = ice at depth 1, `(44,60,68)` at depth 2), background `(8,10,14)`, and the
  status line `z 19/31  camera 34,89  dwarves 5  keys: <> z  arrows/hjkl pan  q quit`.
- FR23 (icy-grim look) is Wolf's manual sign-off and is NOT claimed here. The interactive path
  (`<`/`>`, panning, `q`→`y`) needs a TTY and was not driven by either agent — it is covered by unit
  tests over `apply_key`, not by observation.

### File List

- `_bmad-output/implementation-artifacts/1-3-behold-the-frozen-world.md`
- `Cargo.toml`
- `Cargo.lock`
- `crates/tui/Cargo.toml`
- `crates/tui/src/main.rs`
- `crates/tui/src/palette.rs`
- `crates/tui/src/view.rs`
- `crates/tui/src/frame.rs`

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-03 | Story created |
| 2026-08-03 | Implemented the frozen-world terminal client; final workspace tests remain blocked by sandbox-denied loopback sockets. |
| 2026-08-03 | Orchestrator re-ran the full gate outside the sandbox (33 tests green) and the live `--frame` check; Green gate checked, Status → review. |
| 2026-08-03 | Code review (4 layers): 10 patches applied, 2 decisions resolved by Wolf, 3 deferred, 6 dismissed. 39 tests green; live re-verified. |
| 2026-08-03 | Wolf signed off FR23 (icy-grim look) from a live interactive run. All 12 ACs closed. |
| 2026-08-03 | AC11's fallback wording corrected to what the code and Linux actually do (deferred spec-accuracy item, closed at 2.2 story creation). No code change. |
