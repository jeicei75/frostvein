---
model: claude-opus-5[1m]  # policy default (Opus); recorded per the model policy so the ledger row is readable
baseline_commit: cca118a3a6fc9c0fe1676f454f3556ed9c424eab
---

# Story 8.2: Designate with the Mouse

Status: in-progress

## Story

As the boss,
I want to drag out rectangles in the 3D view to designate digs and channels, cancel them, and
place and remove stockpiles,
so that I can run the fortress from the client I actually want to look at.

## No sign-off gate on this story — read before looking for a Task 0

UX-DR22 applies to **8.3 and not to 8.1–8.2**, decided in the epic rather than left to
inference [epics.md:1014]. Drag feedback is **legibility** work on a look 5.4 and 7.2 already
settled, governed by UX-DR17 and UX-DR18. There is **no Task 0 artifact, no `8-2-signoff/`
directory, and no closing-half AC**.

**One look change IS in scope and it is the only one.** The hover highlight moves to the hit
face (AC13). That is not preference tuning — it is 8.1's HIGH deferral, ruled by Wolf on
2026-08-25 with a measured defect behind it, and it lands here because this story designates
by pointing at exactly the vertical faces where the slab is buried. **Do not re-tune any other
look constant.** M2-2 is open and carries the gfx pass's inherited targets.

## The live vehicle — unchanged, do not re-derive

**gingerspice**: cross-compiled `gui.exe` on native Windows, NVIDIA Vulkan, `simd` in WSL over
localhost. **No devpod can open a window** — measured at 5.3, both fallbacks walked to the end,
and re-measured twice at 8.1's review (`bevy_winit`: *neither WAYLAND_DISPLAY nor WAYLAND_SOCKET
nor DISPLAY is set*). Consequence for planning: **every `--capture` AC in this story is
vehicle-bound and will not execute anywhere else.** The headless suite is the only half that
runs in a devpod.

**REBUILD `gui.exe` BEFORE THE SESSION AND RECORD THE BUILD TIME AND SOURCE COMMIT.** The
stale-binary trap has now fired five times (three in 5.4; a 216-minute-old binary at 8.1's
review; and 8.1's Task 6 shipped with **no wall-clock build time captured at all**). **M2-7 is
still open and unstarted** — verified 2026-08-26: `scripts/` holds only `audit-mutations.py`,
`codex-handoff.sh`, `gate.sh`, `mutate.sh`, `task6-designate.py`, and
`rg 'GIT_SHA|git_sha|build_sha|vergen' crates/gui/src/` returns nothing. Nothing automates this.
This is the **fifth** occurrence; say so in the record.

## Wolf's rulings, taken at story creation 2026-08-26

| # | Question | Ruling |
| --- | --- | --- |
| W1 | Drag vs click-anchor-click-commit (epic AC3 left it to "testing in this story", which no devpod can do) | **Press-drag-release.** Left button down anchors, drag previews, release commits. Right button or `Esc` aborts. |
| W2 | Mode keys — the TUI's `d` collides with camera yaw-right (`ingest.rs:519`) | **Digits `1`/`2`/`3`/`4`.** Zero collisions; every letter stays free. The hint bar makes the divergence from the TUI's letters moot. |
| W3 | Which fix for 8.1's buried hover slab | **Slab on the hit face.** The DDA already knows which axis it crossed. |
| W4 | Does M2-15's `--capture-at-tick` ship here | **Yes.** The ack AC needs tick-anchored evidence and `--frames` cannot give it. |

## Acceptance Criteria

### The gate

1. `scripts/gate.sh` (the **full** tier, not `--fast`) is green on a cold rebuild, and the diff
   is confined to this story's own commit range from `baseline_commit`.

### The commands

2. Dig, channel, stockpile placement, stockpile removal and designation cancellation are each
   issued from `gui` as the **existing** `protocol::Command` variant. `crates/protocol/` is
   unmodified: no variant added, no field changed, no ack or acknowledgement message.
   *Mechanism is load-bearing: AD-6 makes `protocol` the single home of message shapes and
   AD-10 fixes this command set, so "some other equivalent wire shape" is a violation, not an
   alternative implementation.*
3. Every rect `gui` puts on the wire is produced by `client_core::rect_on_level`. `crates/gui/`
   contains no second rect normalization — no local min/max sorting of corners and no local
   z-flattening. *Mechanism is load-bearing: AD-18 gives `client-core` the one helper both
   clients use.*
4. A rectangle dragged between two tiles at different heights yields a single-z rect at the
   **anchor** tile's z, with inclusive corners and `min ≤ max` on every axis.
5. **Seam exercised.** The built command reaches the socket: a test drives the real
   press→drag→release path over a real loopback `TcpStream` and asserts the **bytes the daemon
   would read**. Discarding the command after building it, or dropping the send system from its
   registration, turns that test RED. Asserting that a command was constructed, or that a
   system is registered, does not satisfy this.

### The interaction

6. With a mode active, pressing the left button over a picked tile anchors a rectangle,
   dragging updates it, and releasing commits it and clears the anchor. A press or a release
   where nothing is picked commits nothing and leaves no dangling anchor.
7. Pressing the right button, or `Esc`, during a drag abandons it: no command is sent and the
   preview disappears.
8. `1`/`2`/`3`/`4` select dig, channel, stockpile and clear; `Esc` with no drag in progress
   leaves the mode. No new binding collides with `W`/`A`/`S`/`D`, `Q`/`E`, `,`/`.` or `F3`.
9. A hint bar is visible in every frame. It names the active mode and the key that leaves it,
   and with no mode active it names the keys that enter one. Its text is ASCII-only.
   *(UX-DR18; ASCII because the shipped font draws a replacement box for anything else —
   `slice.rs:95`, the em-dash that shipped in every capture since 7.1.)*

### It reaches the world and comes back

10. A dig, a channel and a stockpile issued from the Bevy client appear in that client's own
    view, and a designation cancelled from the Bevy client disappears from it — through
    `client-core`'s absence-is-deletion, the same path 7.2 proved for a TUI-issued cancel.
11. Designating on a sliced underground level produces the same result as designating on the
    surface: the rect lands on the tiles pointed at, not on the world top.
12. The mark for a command issued in frame *N* is projected in the first frame after the delta
    carrying it is ingested — no second frame of latency added by this story's code. Combined
    with the 10 Hz tick that is the ~200 ms of the epic's ack bar; no ack message exists.

### The highlight, from 8.1's deferral

13. The hover highlight is drawn on the face of the picked cell that the ray entered, so it is
    visible on a cliff face, a corridor wall and a shaft side — not enclosed by the cube above
    it. It stays distinct from the designation, channel and zone marks and clear of the
    near-white reserved for stars and emitter faces.

### Headless (AD-17 rung 2)

14. Under `MinimalPlugins` in `cargo test`, driving real `ButtonInput<MouseButton>`,
    `ButtonInput<KeyCode>` and cursor state through the shared registration point: the mode
    machine's transitions, the abort paths, the anchor-z rule of AC4, and the underground case
    of AC11 are all asserted without a GPU.

### The instruments

15. `gui --capture <path> --at-tick N --z N --drag <mode>,<x0>,<y0>,<x1>,<y1>` scripts a drag
    in viewport pixels **through the real press/drag/release path** — not by constructing a
    rect directly — and the capture range-checks the marks it created against a non-zero
    expected count before drawing any conclusion.
16. `--at-tick N` triggers the capture on the mirror's tick rather than on a frame count, and a
    run that never reaches tick N says so and exits non-zero rather than capturing early or
    hanging silently. *(M2-15: `--frames` is a render-rate quantity and every assertion it
    feeds is in ticks; the conversion factor is fps, which changed the answer by 4x between
    6.1 and 7.2 on this same vehicle.)*
17. The instruments have their own tests: a different `--drag` produces a different rect on the
    wire, and `--at-tick` fires at the tick it names rather than at a frame count.
18. A `tui` client on the same daemon independently confirms the sim received exactly the
    intended designations — a count of its own mark glyphs, range-checked, not assumed
    (AD-17 rung 1).

### Measured on the vehicle

19. On the live vehicle, Wolf drags out a dig, a channel, a stockpile and a clear with the
    mouse, on the surface and on a sliced underground level, and each takes effect in the same
    client. NFR6 still holds with the input path live: sustained **60 fps at working zoom** and
    **≥30 fps at full vista**, read from the F3 overlay.

### Evidence

20. A sabotage table at
    `_bmad-output/implementation-artifacts/mutations/8-2-designate-with-the-mouse.sh` covers
    every seam AC above; every mutation is KILLED and the RED evidence is recorded per row with
    the assertion that went red.

## Tasks / Subtasks

- [x] **Task 1 — The upstream path (AC: 2, 5)**
  - [x] `connect_to_daemon` clones a write handle **before** `BufReader::new(stream)` consumes
        the stream, exactly as `tui::command_writer` does (`tui/src/main.rs:312-319`), and sets
        a 30 s write timeout. It now returns the writer alongside the mirror and receiver.
  - [x] New `crates/gui/src/command.rs`: `CommandSink(Mutex<TcpStream>)` as a `Resource`
        (`TcpStream` is `Send` but not `Sync`; `IngestReceiver` (`ingest.rs:66`) is the
        in-repo precedent for the `Mutex` wrapper), a `PendingCommands` queue resource, and one
        `send_commands` system that drains it, writes newline-delimited JSON and flushes.
  - [x] **No trait, no `Box<dyn Write>`.** A test makes a real socket pair with
        `TcpListener::bind("127.0.0.1:0")` + `TcpStream::connect`, so the production type is the
        tested type and there is no single-implementation abstraction.
  - [x] A write error is logged and the queue drained; it must not panic the client.
  - [x] `crates/protocol/` is not opened.

- [x] **Task 2 — Mode machine, hint bar and the drag (AC: 6, 7, 8, 9, 14)**
  - [x] New `crates/gui/src/designate.rs`: `DesignateMode { None, Dig, Channel, Stockpile, Clear }`
        and `DragAnchor(Option<[i32; 3]>)`, both client-local resources.
  - [x] `1`/`2`/`3`/`4` set the mode; `Esc` aborts a drag if one is in progress, otherwise
        clears the mode. Right button aborts a drag.
  - [x] Left button `just_pressed` with a mode active and `PickedTile` non-empty sets the
        anchor. `just_released` builds the rect and pushes the command(s), then clears the
        anchor **unconditionally** — including on the paths that send nothing.
  - [x] **Clear mode sends BOTH `CancelDesignation` and `RemoveStockpile`**, in that order, for
        the one rect — the TUI's `Mode::Remove` does exactly this (`view.rs:421-424`) and
        parity means the same pair, not a choice.
  - [x] Hint bar: a `Text` + `Node` UI entity following `setup_slice_readout`
        (`ingest.rs:450-473`) — absolute position, **explicit `GlobalZIndex`** (without one it
        renders under the F3 overlay), `ClientLocal` at spawn. Put it at the bottom of the
        window so it does not collide with the slice readout at `top: 44`.
  - [x] An ASCII-only test over every `(mode, dragging)` pair, following
        `the_readout_stays_inside_the_shipped_fonts_glyph_range` (`slice.rs:99`).

- [x] **Task 3 — The rect and the commands (AC: 3, 4, 10, 11, 12)**
  - [x] Build every rect with `client_core::rect_on_level(anchor.xy, release.xy, anchor.z)`.
        The anchor's z is the level; the release tile's z is discarded. Add a `// NOTE:` naming
        that limitation — a drag up a cliff designates on the anchor's level, which is the
        single-z rect rule (AD-18), not a bug.
  - [x] Map mode → command: Dig/Channel → `Designate { kind, rect }`, Stockpile →
        `PlaceStockpile`, Clear → the two-command pair above.
  - [x] Nothing here consults tile contents, reachability or job rules. `gui` holds no game
        logic (AD-4); the sim decides what a rect means.
  - [x] AC12's test: ingest a delta carrying the designation, run **one** `app.update()`, assert
        the `ProjectedDesignation` entity exists. Two updates passing is not the assertion.
  - [x] AC11's test runs the whole path at a slice level below the world top.

- [x] **Task 4 — The hit-face highlight and the drag preview (AC: 13)**
  - [x] `first_visible_hit` returns the crossed axis with the cell. Widen `PickedTile` to
        `Option<PickedCell { tile: [i32; 3], face: Face }>` and add
        `PickedTile::tile() -> Option<[i32; 3]>` so 8.1's existing call sites and tests change by
        one method call rather than being rewritten.
  - [x] **The camera can start inside the world box**, and a ray that hits on its very first
        cell has crossed no axis. Default that case to the top face — today's behaviour — and
        say so in a `// NOTE:`.
  - [x] `sync_hover_highlight` translates by `face_normal * 0.55` and rotates `mark_mesh`'s thin
        Y axis onto the face normal. Reuse the existing mesh; do not add a second one.
  - [x] Test it on a **vertical** face: a tile with a drawn cube directly above it, picked from
        a legal pitch, must place the highlight outside that cube's render-space span
        (`z+0.5..z+1.5`) rather than inside it. This is the exact arithmetic of 8.1's deferral —
        assert the geometry, since no test can see the pixel.
  - [x] Drag preview: slabs over the pending rect while the button is held, despawned on commit
        and on abort. Hoist them with `dig_mark_level` (`project.rs:597`) so the preview sits
        where the committed marks will sit.

- [x] **Task 5 — The instruments (AC: 15, 16, 17)**
  - [x] `--at-tick N` in `parse_args_from` (`ingest.rs:288`) and `--drag <mode>,<x0>,<y0>,<x1>,<y1>`.
        Both require `--capture`, matching the `--distance`/`--cursor` shape (`ingest.rs:344-350`).
        **A typo'd flag is silently swallowed as the TCP port** (`ingest.rs:332-333`) — reject an
        unparseable value explicitly, as `parse_cursor` does.
  - [x] `--drag` writes the real cursor and the real `ButtonInput<MouseButton>` across
        successive frames so the scripted run enters the **same** mode machine a human does.
        A flag that builds a rect directly proves nothing about this story's headline outcome
        (M2-11, and 7.2's `--distance`).
  - [x] `--at-tick` records the mirror tick at connect and fires when `tick >= start + N`. If
        the frame budget is exhausted first, print what tick was reached and exit non-zero.
  - [x] Range-check: the existing `marks:` line already prints `designations=A of B zones=C of D`
        (`capture.rs:589`). Assert the expected count is **non-zero** before comparing —
        7.2 photographed an empty site and exited 0 because both sides agreed on zero.
  - [x] Both instruments get their own tests, driven through `capture_after_frames` under
        `MinimalPlugins` as `crates/gui/tests/capture.rs` already does.
  - [x] `--frames` stays; `--at-tick` is the tick-anchored alternative, not a replacement.
        **Do not bake any rect into the binary** — the scenario is the caller's (M2-15).

- [x] **Task 6 — Sabotage table (AC: 20)**
  - [x] `_bmad-output/implementation-artifacts/mutations/8-2-designate-with-the-mouse.sh` in the
        house format — `assert s.count(old) == 1` guard on every edit.
  - [x] Minimum rows: `send_commands` dropped from its registration tuple; the built command
        discarded instead of queued; `rect_on_level` replaced by a hand-rolled `min`/`max`;
        the anchor's z replaced by the release tile's z; the `Esc`/right-button abort still
        sending; clear mode sending only one of its two commands; `--drag` parsed but never
        reaching the mouse state; `--at-tick` firing on the frame count instead of the tick;
        the highlight's face offset replaced by the old unconditional `+Y*0.55`.
  - [x] **Commit before running** (M2-9). Run `scripts/mutate.sh` **alone** — it is not
        concurrency-safe. Capture the exit code before any pipe.
  - [x] **Dry anchor-check first** (M2-8), and again after `cargo fmt` — formatting moves
        anchors, and 8.1's row 2 went stale mid-round exactly this way.
  - [x] Widening `PickedTile` reformats `pick.rs` and `project.rs`. Re-run
        `python3 scripts/audit-mutations.py` and repair **8.1's** table if a row stops applying
        — at 8.1 a helper broke a row in *5.4's* table, in a file that story never opened.

- [ ] **Task 7 — VEHICLE-BOUND: the whole feature, by hand (AC: 19)**
  - [ ] **Rebuild and re-copy `gui.exe` first. Record the build wall-clock time and the source
        commit.** 8.1 recorded the commit and not the time; do both.
  - [ ] Drag each of the four modes on the surface and on a sliced level; confirm each takes
        effect, and that the hover highlight is visible on a cliff face.
  - [ ] Read sustained fps at working zoom and at full vista from the F3 overlay. A failed
        reading is the finding and gets reported, not worked around.
  - [x] Write `8-2-signoff/task-7-vehicle-runbook.md` from the worked example at
        `7-2-signoff/task-6-vehicle-runbook.md` — the commands that actually ran, corrected.
        **Written 2026-08-27, BEFORE the session rather than after it, which is the one way it
        differs from its model.** 7.2's runbook recorded commands that had run; this one cannot,
        because none of it can execute in a devpod. Every flag, threshold, glyph and colour in it
        was read off the source at `aca07be` rather than carried forward — `--drag` and `--at-tick`
        did not exist when 7.2's was written, and three of its flag rules were only added by the
        2026-08-26 review. It is a recipe to be corrected by the session, not a record of one.

- [x] **Task 8 — The gate (AC: 1)**
  - [x] `cargo clean -p gui`, then `scripts/gate.sh` full tier. Paste the tail. A
        `GATE GREEN (FAST)` line is a coverage hole, not a pass.

### Review Findings

Code review 2026-08-26 — four layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor,
Feature Auditor), fresh context, **no coverage hole: all four ran `cargo` and reported**.
Baseline `cca118a`, which is also `main`'s tip — this story is NOT stacked.
Gate re-run green independently: fmt ok, clippy `-D warnings` ok, `cargo test` 403/0/1-ignored,
three `cargo tree` probes show no `sim-core` edge.

**The headline: the feature is genuinely wired.** All 15 hops trace to real live callers and the
daemon end was proved with the real binaries (a dig rect landed as 16 designations, read back over
an independent connection). This story does NOT carry the inert-seam defect that beat 7.2 and 8.1.
What it carries instead is that same defect displaced one level out — into the instruments meant to
detect it, and into the tests meant to pin it.

- [x] [Review][Defer] Blocking socket write runs inside the Bevy `Update` schedule — `send_commands` does a blocking `write_all` under a 30 s write timeout (`SNAPSHOT_READ_TIMEOUT` reused), registered at `ingest.rs:260`. A back-pressured daemon freezes the render loop for up to 30 s. **RESOLVED 2026-08-26 (Wolf): accepted as-is** — the daemon is localhost, so back-pressure long enough to matter is not a real shape today, and a writer thread is exactly the speculative machinery YAGNI forbids. REOPEN TRIGGER: if `simd` ever runs off-box, or the client is ever pointed at a non-loopback daemon, this becomes live and needs the background writer. [crates/gui/src/command.rs:37]
- [x] [Review][Patch] `audit-mutations.py` cannot see 7 mutation rows, and one has already rotted — the script only audits literals bound to a named variable, so rows passing literals inline to `s.replace(...)` are skipped entirely. `2-1-…:20` searches for a `tui` arm refactored long ago; it writes the file back byte-identical, the test passes, and the runner reports SURVIVED — printing "the test is not pinning what it claims" when the truth is the sabotage is broken. The script exits 0 and prints an all-clear. **RESOLVED 2026-08-26 (Wolf): patch now**, out-of-scope notwithstanding — a broken observability instrument is the standing exception. [scripts/audit-mutations.py]
- [x] [Review][Patch] Drag preview shares the hover material and the dig offset for every mode — previews cyan then commits blue; a channel previews at `0.54` but commits at `-0.46`. Task 4 says "the preview sits where the committed marks will sit". **RESOLVED 2026-08-26 (Wolf): fix now** — preview takes the committing mode's own offset AND material, so it sits and reads as what it will become. This is a look change made on a concrete Task 4 defect, not tuning; it still wants your eye at the vehicle session. [crates/gui/src/project.rs:290]
- [x] [Review][Patch] Mode key can be switched mid-drag; release uses the *current* mode, not the one the drag began in. **RESOLVED 2026-08-26 (Wolf): lock the mode at anchor time** — the drag commits in the mode it began in. [crates/gui/src/designate.rs:89]
- [x] [Review][Patch] `--at-tick` exhaustion writes `AppExit::error()` but the process exits 0 — `app.run()`'s return is discarded and `main` returns `Ok(())`. Confirmed: `bevy_winit-0.19.0` has no `process::exit`, and `#[must_use]` is on `App`/`AppExit`'s methods but not the `AppExit` enum, so clippy stays green. Violates AC16's "exits non-zero". [crates/gui/src/ingest.rs:95]
- [x] [Review][Patch] A `--drag` capture that designates nothing prints a pass and exits 0 — AC15's non-zero range check is gated behind `--expect-work`, which the story's own recipe omits and which *cannot* be used for a dig-only drag because it also asserts `expected_zones > 0`. The 7.2 empty-site false pass, reproduced. [crates/gui/src/capture.rs:122]
- [x] [Review][Patch] The DDA march's hit face is entirely unpinned — inverting the X and Y face assignments leaves 149/149 green. That inversion buries the highlight in the neighbouring cube, which is exactly 8.1's deferred defect this story exists to fix. [crates/gui/src/pick.rs:151]
- [x] [Review][Patch] The slab's rotation onto the face normal is untested — deleting `.with_rotation(...)` from both call sites leaves 149/149 green; the one geometry test asserts `translation` only and never reads rotation. Without it the slab on a cliff face is an edge-on wafer. [crates/gui/src/project.rs:233]
- [x] [Review][Patch] Channel, stockpile and clear are unreachable by any test — collapsing `Digit2/3/4` to Dig is green; making Channel emit Dig and Stockpile emit nothing is green. Three of the four modes in the story title have no coverage on any path. [crates/gui/src/designate.rs:85]
- [x] [Review][Patch] `entry_face` returns `Face::Top` unconditionally when the camera sits inside solid geometry — reachable in normal play (camera has no terrain collision, min zoom 4.0, and designation is a close-zoom interaction). Confirmed by an executed standalone reproducer. Consequence is a wrong-oriented hover slab, i.e. AC13. [crates/gui/src/pick.rs:177]
- [x] [Review][Patch] A write error silently clears the entire pending queue — `pending.0.clear(); return;` behind nothing but an `eprintln`. A designation the boss dragged vanishes with no on-screen trace, and a transient stall is not distinguished from a dead peer. [crates/gui/src/command.rs:59]
- [x] [Review][Patch] The scripted drag fires on frames 1–3 unconditionally, with no retry and no success check — each stage advances whether or not `update_pick` resolved a tile, at the coldest moment in the app's life. 8.1's `--cursor` rewrote every frame and self-healed; this has three shots. Stacked on the range-check gap, a `--drag` capture can be wholly inert and still exit 0 with a PNG on disk. [crates/gui/src/ingest.rs:501]
- [x] [Review][Patch] `--cursor` is silently ignored when `--drag` is present, yet the capture still asserts the pick against it — a guaranteed spurious failure with a misleading message, where every other bad flag combination in this parser `bail!`s. [crates/gui/src/ingest.rs:496]
- [x] [Review][Patch] `Esc` is dead to every test — removing it from the abort condition leaves 149/149 green. Neither AC7's abort-during-drag nor AC8's leave-the-mode is asserted; the one abort test presses `MouseButton::Right` only, so mutation row 5's KILLED verdict rests entirely on the right-button half. [crates/gui/src/designate.rs:93]
- [x] [Review][Patch] The hint bar never updates and nothing notices — neutering `update_designate_hint` or dropping it from its registration tuple is green. AC9's load-bearing clause ("names the active mode") has no test; the bar would read the no-mode string forever. [crates/gui/src/designate.rs:43]
- [x] [Review][Patch] AC10 is asserted for dig only — no round trip for channel, none for stockpile (`ProjectedZone` is never checked after a client-issued command), and none for a client-issued cancel. `rg 'CancelDesignation|RemoveStockpile|PlaceStockpile' crates/gui/tests/` returns nothing. [crates/gui/tests/headless.rs:2520]
- [x] [Review][Patch] The abort test bypasses the shared registration point — `run_system_once(designation_input)` with hand-inserted `DragAnchor(Some(..))` *and* `PickedTile(Some(..))`. That is the D6-forbidden shape, and exactly what hid 8.1's `--cursor` bug through a whole mutation round. AC14 requires the abort paths driven through the shared point. [crates/gui/src/designate.rs:209]
- [x] [Review][Patch] Add sabotage rows for the seams this round left uncovered — the 9 rows match Task 6's stated minimum and stop there; the demonstrated holes are the march face, the slab rotation, the three non-dig modes, `Esc`, and the hint bar. AC20 asks for every seam AC. The identical critique was raised and patched at 8.1. [_bmad-output/implementation-artifacts/mutations/8-2-designate-with-the-mouse.sh]
- [x] [Review][Patch] `--at-tick` silently disables the motion instrument — the guard drops `position_changes > 0` and `mid_blend_frames > 0`, two live-client health checks unrelated to tick count, on precisely the new path the vehicle recipe will use. Nothing in the story asks for this. [crates/gui/src/capture.rs:676]
- [x] [Review][Patch] `command.rs` has no test for any failure path — zero references anywhere to `MAX_PENDING_COMMANDS`, the queue-full string, the send-failure string, or lock poisoning. Only the happy path is exercised, in a file this story created. [crates/gui/src/command.rs:65]
- [x] [Review][Defer] Paired `Clear` commands can split at the 256 bound — deferred, low reachability (needs 256 queued designations); `CancelDesignation` could send while `RemoveStockpile` is dropped. [crates/gui/src/command.rs:18]
- [x] [Review][Defer] `--at-tick 0` boundary is untested — deferred, plausible-but-untested; `target_tick == start_tick` should fire on the first frame. [crates/gui/tests/capture.rs:933]
- [x] [Review][Defer] Test-harness writers set no write timeout, unlike production — deferred, test-only; a harness bug would hang the process instead of failing fast. [crates/gui/src/command.rs:84]
- [x] [Review][Defer] `SNAPSHOT_READ_TIMEOUT` now names both the read and write timeout — deferred, cosmetic; the value is right, the name covers two unrelated things. [crates/gui/src/ingest.rs:57]
- [x] [Review][Defer] `.codex/` is untracked and not git-ignored — deferred, will attach itself to the next `git add -A`. [.gitignore]
- [x] [Review][Defer] M2-7's build stamp is missing for the fifth time — deferred, no automation exists in `scripts/`; `rg 'GIT_SHA|vergen' crates/gui/src/` returns nothing. [scripts/]

**Two absences, recorded as coverage holes rather than clean passes**, per the story's own record:
`codex review --base main` NEVER RAN (killed twice, quota-blocked once), and no vehicle session has
happened. **AC13's rendered half, AC15/16 end-to-end, AC18 and AC19 remain OPEN and unobserved** —
none of them is inferred green from the 403 passing tests. Findings on the march face and the slab
rotation raise the stakes: the live session is currently the only thing standing between a wrong
face or a missing rotation and a shipped defect.

## Dev Notes

### The epic's premises, verified against source 2026-08-26

Every M2 epic premise checked before a story has been wrong at least once, so all of 8.2's were
re-verified against the tree. **Two hold, one is stale, one needs a correction.**

- **The command set exists and needs no addition** — `protocol::Command` already carries
  `Designate { kind, rect }`, `CancelDesignation { rect }`, `PlaceStockpile { rect }` and
  `RemoveStockpile { rect }` (`protocol/src/lib.rs:82-91`), and `simd` dispatches all four
  (`simd/src/main.rs:151-171`). **Holds.**
- **`client-core`'s rect helper exists** — `rect_on_level(a: (i32, i32), b: (i32, i32), z: i32)`
  (`client-core/src/lib.rs:188`). Note the shape: it takes **2D corners plus a z**, so the
  caller chooses the level. That is what makes AC4 a decision this story must make rather than
  something the helper decides. **Holds.**
- **STALE — the epic's "`simd` validates the incoming rect and logs-and-drops a violation
  without crashing the sim" is ALREADY BUILT AND TESTED.** `rect_is_valid`
  (`simd/src/main.rs:714`) is applied at `:677-681` and pinned by
  `invalid_rects_are_logged_dropped_and_leave_the_client_connected` (`simd/tests/serve.rs:1322`),
  which covers both the inverted-corner and the two-z cases. The epic reads as though 8.2 must
  build it. **It must not.** This story's obligation is the client half: AC3 and AC4 make a
  malformed rect unconstructible in `gui`. Do not open `crates/simd/`.
- **CORRECTION — absence-is-deletion is `client-core`'s, not something this story wires.**
  `Mirror::apply_delta` replaces the whole designation list per delta
  (`client-core/src/lib.rs:99`), and `gui`'s `reconcile` despawns the marks the mirror no longer
  wants (`project.rs:500-570`). AC10's cancel clause therefore tests a path 7.2 already proved,
  from the other direction. Assert the round trip; do not rebuild the mechanism.
- **Reported separately, not fixed here:** `docs/architecture.md:32` and `:127-129` still say
  `gui` runs "via WSLg" and state NFR6 against "the WSLg devpod". M2-4 corrected `epics.md` and
  the spine and missed the companion doc; 8.1 reported it and it is still there. Outside this
  story's diff.

### Key decisions & traps

**D1 — `gui` is receive-only today, and opening the write half is this story's one structural
change.** `connect_to_daemon` (`ingest.rs:97`) hands the `TcpStream` to `BufReader::new` at
`:104` and moves the reader into a thread at `:110`; **no write handle survives.** 8.1's
guardrails forbade touching this and named it 8.2's work. Clone before the `BufReader`, as the
TUI does. Everything else about the connection stays as it is.

**D2 — the send seam is the one this project keeps getting wrong, and it now has THREE past
instances.** 7.2's `--distance` parsed, validated and never reached the camera. 8.1's `--cursor`
did the same and **survived mutation round 1 with the whole suite green**, because the only test
inserted the resource by hand. 8.1's review then found the *call to the extracted wiring* was
itself untested. Read the shape: **a test that starts downstream of the production drive line
pins nothing about the drive line.** So AC5's test must begin at a mouse press and end at bytes
on a socket. `configure_client_app` (`ingest.rs:125`) is where the wiring goes, because it is the
one function a test can enter on a real parsed `Args`.

**D3 — `PickedTile`'s shape changes, and 8.1's tests are the blast radius.** The hit face has to
come out of `first_visible_hit` because nothing else knows it. Widening the resource touches
`update_pick`, `sync_hover_highlight`, `capture_after_frames` (`capture.rs:541,561`) and roughly
a dozen assertions in `tests/headless.rs` and `pick.rs`'s own test module. Add
`PickedTile::tile()` so the mechanical churn is one method call per site. **8.1's picking
behaviour must not change** — its 81-case matrix, its occlusion pin and its
128×128×32 tracer test all keep passing untouched in substance.

**D4 — `MinimalPlugins` gives you no camera, no transforms and no input.** 8.1 built the harness
this story extends: `live_app` (`headless.rs:1996`) writes `Camera.computed.target_info`,
`clip_from_view` and a hand-made `GlobalTransform`, because `camera_system` and `TransformPlugin`
do not run. `ButtonInput<MouseButton>` is a plain resource and must be `init_resource`d and
written by hand, the way `ButtonInput<KeyCode>` already is (`ingest.rs:813`). `just_pressed` is
cleared by Bevy's own `ButtonInput::clear` between frames, which does not run here — call
`clear()` yourself between simulated frames or a press will look held forever.

**D5 — register in `client_systems` (`ingest.rs:196`) and nowhere else,** for the same reason
8.1 did: it is the shared registration point the live app and the headless harness both drive.
M2-1 closed this class at the root. Order matters — the drag reads `PickedTile`, which
`update_pick` writes in `PostUpdate` after `TransformSystems::Propagate`, so the drag systems
chain **after** `update_pick` (`ingest.rs:223-229`) and `send_commands` runs last.

**D6 — assert observable effects, never registration; never insert the resource the production
path is supposed to write.** D7/D8 of 8.1, unchanged, and the reason its inert-seam bug was
caught at all. Expected rects are hand-written literals.

**D7 — `--frames` is not ticks, which is why AC16 exists.** `capture.rs:533-547` counts `Update`
runs. Measured on this vehicle: the same `--frames 1500` gave 58 ticks on a light scene and 237
on a heavy one; 6.1's `--frames 600` was ~44 ticks against a `>= 100` floor and **would have
panicked before writing any PNG**. Do not copy a frame count from an older runbook. Once
`--at-tick` exists, use it.

**D8 — channels decay to zero and digs plateau.** Measured 2026-08-22: an 8×8 channel rect gave
39 marks, 14 by +52 ticks, **0 by +114** — dwarves consume every one, because a channel only ever
targets standable ground. Digs settle at a stable floor (79 → ~50 from +120) because the
remainder becomes unreachable. **A scripted capture that designates a channel and then waits
photographs nothing.** Keep `--at-tick` small for channels, or designate them last. This is
exactly the race M2-15 exists to kill.

**D9 — check the window is 16:9 before believing a vehicle mismatch.** `project_render_point`
hardcodes `BOOT_ASPECT_RATIO = 16.0/9.0` (`camera.rs:30`) while the live camera derives aspect
from the real viewport, and the capture oracle multiplies a 16:9-derived normalized coordinate by
the actual window size (`capture.rs:635`). Deferred at 8.1, still open, and `--drag` takes
viewport pixels, so it inherits the exposure.

### Scope guardrails — do NOT build these here

- **No speed, save, load or quit.** FR35's control half is **8.3**. This story is the
  world-mutating set only.
- **Do not open `crates/protocol/` or `crates/simd/`.** The commands exist; the validation
  exists and is tested. A change in either is a scope escape, and AC2 is partly a structural
  claim about the diff.
- **Do not touch `client-core`.** `rect_on_level` is already there and already shared.
- **No wheel binding.** Still unclaimed in code and **still unruled by Wolf from 7.1**; UX-DR2's
  wheel zoom will want it. Leave the decision where 7.1 left it.
- **Do not enable the `bevy_picking` / `mesh_picking` Cargo features.** Considered and rejected
  at 8.1; the crate is in the lockfile via `bevy_dev_tools` but is not reachable through the
  facade.
- **No look tuning beyond AC13's face move** (M2-2 open).
- **No camera-control changes.** W2 chose digits precisely so nothing that ships today moves.

### What already exists (build on it, do not re-derive)

- **The whole picking path** — `PickedTile` / `update_pick` / `first_visible_hit`
  (`pick.rs:16,20,49`), `sync_hover_highlight` (`project.rs:214`), and 8.1's harness and
  81-case matrix in `tests/headless.rs`.
- **The TUI's mode machine, as the parity reference** — `Mode` (`view.rs:37`), the anchor/commit
  handler (`view.rs:400-428`), the hint strings (`view.rs:319-336`). Read it for *which
  commands each mode sends*, not for its key bindings.
- **The TUI's writer** — `command_writer` (`tui/src/main.rs:312`) and `send_command` (`:301`):
  clone the socket, set a write timeout, `writeln!` the JSON, flush.
- **A `Mutex`-wrapped non-`Sync` resource** — `IngestReceiver` (`ingest.rs:66`).
- **A UI text entity done right** — `setup_slice_readout` / `update_slice_readout`
  (`ingest.rs:450-490`): absolute `Node`, explicit `GlobalZIndex`, `ClientLocal`, change-detected
  update, ASCII-only test.
- **Capture plumbing** — `CaptureState` (`capture.rs:203`), `capture_after_frames` (`:533`),
  the `marks:` range-check line (`:584`), `insert_capture_resources` (`ingest.rs:394`),
  `parse_cursor` (`:362`), `TickClock`/`observe_tick` (`ingest.rs:615`).
- **The scripted TUI** — `tui --key <seq> --frames N --z N`, key names at `tui/src/main.rs:344`.
  Mark glyphs: dig `×`, channel `≡` (`view.rs:969-970`).

### Project Structure (files to touch)

| File | NEW/UPDATE | What |
| --- | --- | --- |
| `crates/gui/src/command.rs` | NEW | `CommandSink`, `PendingCommands`, `send_commands` |
| `crates/gui/src/designate.rs` | NEW | Mode machine, drag anchor, rect build, hint-bar text |
| `crates/gui/src/lib.rs` | UPDATE | `mod command; mod designate;` |
| `crates/gui/src/ingest.rs` | UPDATE | Writer out of `connect_to_daemon`; register the drag + send systems in `client_systems`; `--drag` and `--at-tick` in `parse_args_from`; hint-bar spawn |
| `crates/gui/src/pick.rs` | UPDATE | `first_visible_hit` returns the crossed face; `PickedCell` |
| `crates/gui/src/project.rs` | UPDATE | Hit-face hover transform; drag-preview spawn/despawn |
| `crates/gui/src/capture.rs` | UPDATE | `--at-tick` trigger, scripted drag, mark range-check |
| `crates/gui/tests/headless.rs` | UPDATE | Mode machine, drag, socket-byte seam test, AC11/AC12/AC13 |
| `crates/gui/tests/capture.rs` | UPDATE | Instrument tests for `--drag` and `--at-tick` |
| `_bmad-output/implementation-artifacts/mutations/8-2-designate-with-the-mouse.sh` | NEW | Sabotage table |
| `_bmad-output/implementation-artifacts/8-2-signoff/task-7-vehicle-runbook.md` | NEW | The commands that actually ran on gingerspice |
| `_bmad-output/implementation-artifacts/metrics/8-2-designate-with-the-mouse.md` | NEW | Ledger rows (written by the workflow, not by hand) |

### Previous story intelligence (deltas that change THIS story)

- **Branch from `main`.** 8.1 merged (PR #34, `f9df762`); `cca118a` is HEAD, tree clean, gate
  green (run 2026-08-26, output in Verification). The stacked-branch rule still applies to AC1:
  prove the diff scope against **this story's own commit range**, never against `main` or a
  branch tip.
- **8.1's mutation round 1 caught its own headline seam SURVIVING with the suite green.** Budget
  for that outcome: commit before the run, expect a survivor, and treat it as the story's
  finding rather than a setback.
- **`cargo clean -p gui` after a mutation round** was mandated at 7.1/7.2 for cache poisoning;
  M2-16 fixed the root cause and 8.1 saw no evidence it was still needed, but never tried
  skipping it. Keep it, and say in the record whether it mattered.

### Verification

**Executed at story creation, 2026-08-26** — the full gate on `cca118a`, clean tree:

```
frostvein gate
  cargo fmt --check           ok
  cargo clippy -D warnings    ok
  cargo test                  ok
  tui has no sim-core edge                ok
  client-core has no sim-core edge        ok
  gui has no sim-core edge                ok
  metrics ledger tests        ok
  mutation tables still apply ok
GATE GREEN
```

**Not executable at story creation — the feature does not exist yet.** The obligation is
inherited: run each of these and paste the non-zero observation named beside it.

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# 1. Headless (AC 5, 6, 7, 8, 9, 11, 12, 13, 14) — name the modes and levels covered, not just "passed"
cargo test -p gui designate
cargo test -p gui pick          # 8.1's matrix must still be green after PickedTile widens

# 2. Sabotage table (AC 20) — commit first; run alone; exit code before any pipe
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/8-2-designate-with-the-mouse.sh
python3 scripts/audit-mutations.py    # 8.1's table too — this story reformats the files it pins
cargo clean -p gui

# 3. The gate (AC 1) — full tier
scripts/gate.sh
```

Vehicle side (Task 7), after the mandatory rebuild:

```bash
# WSL
cargo build -p simd -p tui
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
./target/debug/simd 7451          # port is positional; seed is fixed in the binary

# Windows, after copying target/x86_64-pc-windows-gnu/release/gui.exe across
gui.exe 7451 --capture 8-2-dig-working.png --at-tick 20 --z 10 --drag dig,820,470,1100,610

# WSL, against the SAME daemon — the rung-1 cross-check (AC18)
./target/debug/tui 7451 --frames 3 --z 10 | rg -o '×' | wc -l
```

**Required observations, not exit 0.**

1. The capture prints a `marks:` line whose `designations=A of B` has **B > 0 and A == B**, and
   the PNG shows the marks. Match by **prefix** — 7.1 changed the oracle's line shape and older
   recipes quoting whole lines stopped matching.
2. The `tui` glyph count is **non-zero** and consistent with the rect dragged. Zero is the 7.2
   failure mode, not a pass.
3. Press **F3** and read sustained fps at working zoom and at full vista.
4. Drag all four modes by hand, on the surface and at a slice below the top, and confirm the
   hover highlight is visible **on a cliff face** — AC13's rendered half, which no test can see.

If `--at-tick` is not yet implemented when the session runs, `--frames` still works — but state
the frame count used and why, and read D7 first.

### Branch and commits

Branch `8-2-designate-with-the-mouse`, cut from `main`. Author every commit
`Völundr <jeicei75@gmail.com>`. **Commit at minimum once per completed task, ideally on each
green** — never one squashed commit; the pre-commit hook runs `scripts/gate.sh --fast` and the
pre-push hook runs the full gate. Review-gated: **no push, no PR** until Wolf says so.

### If this overruns one session

It may — 8.1 was seven tasks and still needed a twelve-patch review round. **Split at Task 5.**
Tasks 1–4 (the upstream path, the mode machine, the rect, the hit-face highlight) are a complete
vertical slice with observable behaviour: a mouse drag in the Bevy client changes the sim.
Tasks 5–8 (the scripted instruments, the sabotage table, the vehicle session, the gate) become
the continuation. **Restate the RED evidence in the continuation handoff** — 1.2 lost it across
a session boundary.

**Self-gate findings land in the Dev Agent Record, fixed or not** (M2-10). A finding that exists
only in a handback message is lost at the session boundary.

### References

- Story text and Epic 8 framing — `_bmad-output/planning-artifacts/epics.md:1004-1090`
- FR35–FR37 — `epics.md:79-84`; NFR5–NFR8 — `epics.md:95-119`; UX-DR17/18/21/22 — `epics.md:204-215`
- AD-4, AD-10, AD-13…AD-18 —
  `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:78-190`;
  AD-17's three rungs at `:146-166`; AD-18's rect contract at `:168-185`; the one-transform
  convention at `:194`
- M2 retrospective (M2-1, M2-2, M2-7, M2-8, M2-9, M2-10, M2-11, M2-15, M2-16) —
  `_bmad-output/implementation-artifacts/epic-5-retro-2026-08-23.md`; M2-15's reasoning at `:373-392`
- 8.1's deferrals, including the buried-highlight ruling —
  `_bmad-output/implementation-artifacts/deferred-work.md:813-884`
- Vehicle procedure — `_bmad-output/implementation-artifacts/vehicle-session-runbook.md`;
  worked example with the raw-wire-command recipes `7-2-signoff/task-6-vehicle-runbook.md`
- Story rules and anti-overengineering policy — `docs/technical-preferences.md`

## Dev Agent Record

### Agent Model Used

Codex `gpt-5.6-terra`, reasoning effort high, across **three** delegated sessions (banner
verified each launch; no model/effort drift). Orchestration, independent gate runs, the mutation
round and this record: Claude Opus 5 `claude-opus-5[1m]`.

**SPLIT OF HANDS — the review must know this.** Sessions 1-3 (Codex) wrote all production code
and all tests. The ORCHESTRATOR (Claude), not Codex, did the following, because Codex ran out of
quota before it could: completed the interrupted commit of the AC11/AC12 tests; ran `cargo fmt`
on unformatted work; repaired 8.1's stale mutation anchor; committed Codex's uncommitted
entry-face work; **changed one order-dependent assertion** in
`the_production_wiring_runs_every_call_run_makes_after_its_plugins` from an ordered `Vec` to a
sorted comparison; and executed the mutation round and every gate. So the reviewer is reviewing
one small assertion change authored by the same model that reviews it. That is a narrower
conflict than authorship of the feature, but it is real and is named here rather than left to be
discovered.

### Debug Log References

- RED (Task 1 writer literal): `left: {"type":"designate","kind":"dig",...}` differed from
  the first hand-written, externally shaped enum literal.
- RED (Task 2 harness): before clearing `ButtonInput` transition state, the drag asserted
  `left: Rect { min: [2, 1, 0], max: [2, 1, 0] }` against the anchor-level literal rect.
- RED (Task 3 seam sabotage): replacing the enqueue with a discard made the loopback server read
  fail with `WouldBlock` at `ingest.rs:993`.
- RED (Task 4 face sabotage): replacing the picked normal with `Vec3::Y` produced
  `left: Vec3(3.0, 5.55, -4.0)`, `right: Vec3(3.55, 5.0, -4.0)`.

**MUTATION ROUND (AC20) — run by the orchestrator 2026-08-26, `scripts/mutate.sh` ALONE, exit
code captured before any pipe. `MUTATE EXIT: 0`. 9 rows, 9 KILLED, NO SURVIVOR.** Dry
anchor-check ran before the round and again after `cargo fmt` (351 rows, every literal matching).
`crates/` verified clean afterwards — every sabotage restored. Per-row RED, with the assertion
that went red:

| # | Row | Test that went RED |
| --- | --- | --- |
| 1 | command writer dropped from the live input schedule | `configured_app_sends_a_real_mouse_drags_command_to_the_daemon_socket` |
| 2 | built designation command discarded instead of queued | `configured_app_sends_a_real_mouse_drags_command_to_the_daemon_socket` (panic at `ingest.rs:1223`) |
| 3 | shared rect helper replaced by local min/max | `designation_input_uses_the_shared_rect_helper_not_local_normalization` |
| 4 | release height replaces the anchor level | `mouse_drag_uses_the_anchor_level_and_clears_its_anchor_on_release` — *"a cross-height drag is one inclusive rectangle on the literal anchor level"* |
| 5 | abort no longer wins over a concurrent release | `abort_wins_over_a_same_frame_release_and_sends_nothing` |
| 6 | clear mode omits stockpile removal | `clear_issues_both_existing_commands_in_tui_order` |
| 7 | parsed drag never reaches the scripted mouse state | `parsed_capture_drags_send_their_own_rectangles_to_the_daemon_socket` |
| 8 | at-tick capture fires on frame count | `at_tick_capture_waits_for_the_mirror_tick_and_reports_an_exhausted_budget` |
| 9 | hover slab returns to the unconditional top-face offset | `a_vertical_hit_face_places_the_hover_slab_outside_the_cell_side` |

**No survivor this round.** 8.1's round 1 caught its own headline seam surviving with the suite
green, and this story's D2 names three past inert seams, so a survivor was the expected outcome
and its absence is worth stating plainly rather than glossing: rows 1, 2 and 7 are precisely the
inert-seam shape, and all three went red.

**`cargo clean -p gui` before the final gate:** kept per the story's instruction. It has been
mandated since 7.1/7.2 for cache poisoning, M2-16 fixed the root cause, and — as at 8.1 — there
was **no evidence it was needed**. It was still not tried without.


### Completion Notes List

**Delivered: the whole story except the vehicle-bound Task 7.** A mouse drag in the Bevy client
now changes the sim. 14 commits, every one authored `Völundr <jeicei75@gmail.com>`, no squash.

- **Tasks 1-3 — the upstream slice.** `gui` was receive-only; `connect_to_daemon` now clones the
  write handle before `BufReader` consumes the stream (the TUI's shape), feeding a concrete
  `CommandSink(Mutex<TcpStream>)` and a bounded `PendingCommands` queue. No trait, no
  `Box<dyn Write>` — the production type is the tested type. Anchor-level rects come from
  `client_core::rect_on_level`; clear sends `CancelDesignation` **and** `RemoveStockpile` in TUI
  order. `crates/protocol/`, `crates/simd/` and `crates/client-core/` were never opened, so AC2's
  structural claim about the diff holds.
- **AC5's seam is real, and this is the AC that mattered most.** D2 records three past inert
  seams — 7.2's `--distance`, 8.1's `--cursor` twice, one of which survived a whole mutation round
  with the suite green. `configured_app_sends_a_real_mouse_drags_command_to_the_daemon_socket`
  enters through the production `configure_client_app` on real parsed `Args`, drives real
  `ButtonInput`, and asserts the bytes read off a real loopback socket. Mutation rows 1, 2 and 7
  attack exactly that seam and all three were KILLED.
- **Task 4 — the hit-face highlight**, 8.1's HIGH deferral, ruled by Wolf. The DDA now returns the
  crossed axis; `PickedTile` widened to `PickedCell { tile, face }` with `PickedTile::tile()` so
  8.1's call sites changed by one method call. 8.1's picking behaviour is unchanged in substance.
  A late refinement computes the true entry face for a ray starting OUTSIDE the world box rather
  than defaulting to Top; rays starting inside still default to Top with a `// NOTE:`, as the task
  specified. **That refinement was Codex's last uncommitted work and it carries no separate
  record from Codex — flagging it for the review's attention.**
- **Task 5 — instruments, both tested.** `--at-tick N` fires on the mirror's tick and exits
  non-zero when the frame budget runs out first; `--drag` writes the real cursor and real
  `ButtonInput<MouseButton>` across frames, so a scripted run enters the same mode machine a human
  does. The non-zero mark range-check (7.2's empty-site false pass) is in force.
- **Task 6 — 9 rows, 9 KILLED, no survivor.** Full per-row evidence in Debug Log References.
- **Task 8 — full gate green on a cold rebuild**, run by the orchestrator, `cargo clean -p gui`
  first. Tail pasted below.

```
frostvein gate
  cargo fmt --check           ok
  cargo clippy -D warnings    ok
  cargo test                  ok
  tui has no sim-core edge                ok
  client-core has no sim-core edge        ok
  gui has no sim-core edge                ok
  metrics ledger tests        ok
  mutation tables still apply ok
GATE GREEN
```

**OWED, and not closable by any agent here:**

- **Task 7 / AC19 — NOTHING HAS BEEN OBSERVED ON THE VEHICLE.** No devpod can open a window
  (`bevy_winit`: neither `WAYLAND_DISPLAY` nor `WAYLAND_SOCKET` nor `DISPLAY`). The four hand
  drags on surface and slice, the cliff-face highlight check, and the sustained-fps readings at
  both zooms are owed to a live gingerspice session. **No fps figure was fabricated.** AC13's
  rendered half and AC19 are both unmet until that session runs.
- **M2-7's build stamp is missing for the FIFTH time.** `scripts/` still holds no build-stamp
  automation and `rg 'GIT_SHA|git_sha|build_sha|vergen' crates/gui/src/` still returns nothing.
  The story asked that the fifth occurrence be said out loud; this is it.
- **`codex review --base main` NEVER RAN for this story.** Three attempts, three distinct
  failures: session 1's died to the 30-second foreground-command limit during repository
  inspection; session 2 was killed by the harness before reaching it; session 3 hit the Codex
  usage limit (`try again at 7:00 PM`). **No self-gate finding exists — this is an absence, not a
  clean pass**, and the review should weigh it accordingly. The three-pass cap was never
  approached, so no quota was spent on it.

**THIS STORY CONSUMED THE ENTIRE WEEKLY CODEX QUOTA: 0% → 100%.** Four dev rows,
98pp total (32 + 6 + 6 + 54), $6.37 by the dollar benchmark — and the dollars are the column that
cannot see what happened. **Codex is now unavailable to the next story until the 7-day window
resets**, exactly the 3.2 failure repeating, and this time to exhaustion rather than to a third.
Two facts the retrospective should hold onto: **12pp bought nothing at all** (the two
harness-killed sessions, ~3 minutes each, no usable output), and **the self-gate never ran**, so
the three-pass cap was never even approached — the quota went entirely on dev turns and their
nested rollouts. The quota is also account-wide and shared with nidavellir's court brain, so this
starves more than frostvein.

**Session history, because it explains the commit shape.** Session 1 (23 min) built Tasks 1-4 and
stopped at the 30-second command limit, correctly refusing to claim a gate it could not run.
Sessions 2 and 3 were killed by the harness at ~3 minutes each for reasons never identified — no
auth failure, no OOM, RAM free. Session 4 hit the usage limit mid-fix. The orchestrator recovered
each interruption by hand (see the split-of-hands note under Agent Model Used); nothing was lost,
but three of the four sessions ended involuntarily.

### Vehicle Session Record — 2026-08-27 (IN PROGRESS)

**Build stamp (M2-7's fifth occurrence, worked around by hand for the fifth time).**

| | |
| --- | --- |
| Binary | `target/x86_64-pc-windows-gnu/release/gui.exe`, 188,932,160 bytes |
| Built | **2026-08-27T05:13:38Z** — artifact mtime; `gui.exe`, its `deps/gui-9fadb9141a8d5463.exe` hardlink and `gui.d` all agree |
| Source commit | **`fb61f3f`** — the last commit touching `crates/`. `612cbdf` was HEAD at build time, but `aca07be`, `bfac2c4` and `612cbdf` are `_bmad-output/`, `.gitignore` and `scripts/audit-mutations.py` only |
| Tree at build | clean; HEAD committed 05:06:10Z, 7m28s before the build |
| Wall-clock | 9.62s, **incremental** — only `gui` recompiled. A freshness stamp, NOT a build-cost figure |

**The stamp step failed on its first use, in exactly the way M2-7 predicts.** The runbook as
written asked for `date -u` around the build; it was run **~114 minutes after** it, and would have
recorded `2026-08-27T07:07:17Z` for a binary written at `05:13:38Z`. That is the same shape as
8.1's 216-minute-old binary, arrived at by a different route — not a stale binary this time, but a
stale *reading* of a current one. **A hand-typed clock reading is not a build stamp.** The runbook
was corrected the same day to take the stamp from the artifact's mtime and the last `crates/`
commit, neither of which can drift or be pasted from scrollback. This is the sixth time M2-7 has
cost something and the first time the cost was caught before it entered the record.

The binary is confirmed current: mtime 2026-08-27T05:13:38Z is later than `fb61f3f`, the tree is
clean, so every 2026-08-26 review patch is in it.

**WHAT THE HANDS FOUND, 2026-08-27 — two of the four modes had never worked at all.**

Wolf drove it on the vehicle and reported it as "a bit fragile and confusing... sometimes it loses
dragged tile color... sometimes not colorize it at all... channelling, what should it do". Four
defects behind that, none of them look-tuning, none visible to any instrument:

1. **Channel and stockpile were COMPLETELY INERT.** Picking only ever resolves a `Solid` or `Ramp`
   cell (`is_visible_at_slice`), and `sim-core` filters both of those commands on standability —
   `Tile::Empty` with support beneath — dropping the remainder in **silence**: no error, no ack,
   no log. The client sent well-formed commands the daemon accepted and discarded. **Proven
   against the real binary, not inferred:** a channel rect at the picked cell yields 0
   designations, the same rect one cell up yields 9; stockpile behaves identically. **AC10 was
   false for two of its three clauses**, and AC19's four hand drags were never possible.
2. **Clear was half-working** — it removed digs, which do live at the picked cell, and could never
   reach a channel or a stockpile, which live one cell across the entered face.
3. **The preview died on any frame whose ray missed terrain** (`project.rs`, `sync_drag_preview`)
   while the drag stayed live and still committed on release. That is "loses dragged tile color",
   and it means dragging blind rather than merely a flicker.
4. **Channel marks never got 7.2's buried-mark fix.** Dig climbs onto the top face of covering
   rock via `dig_mark_level`; channel was an unconditional `slab_transform(position, -0.46)` at
   both the committed mark and the preview, so it sat sealed inside anything drawn above it —
   7.2's measured "0 of 50 marks visible while the count read 50", reproduced for channel.

**The hole underneath all four: no test ever asked the daemon whether it KEPT anything.** Every
`gui` test asserts what the client queues. The review's "the feature is genuinely wired, all 15
hops reach real live callers" was true about the client's hops and structurally blind to the sim's
filters — and the instruments are blind the same way, because `marks: designations=D of X`
compares projected entities to mirror entities, so a rect the sim discarded reads `0 of 0` and a
slab spawned inside rock is projected and counted like any other. **Every §6 capture could have
gone green while showing nothing.** This is the inert-seam defect displaced one level further out
again: wiring → instruments → the sim's own acceptance.

**RULED 2026-08-27 (Wolf): the target is the neighbour across the face the ray entered.** A top
face channels the air directly above; a cliff face targets the cell you are looking into, which is
standable exactly when it borders a ledge. The face was already computed for AC13's highlight and
is now behavioural rather than decorative. Dig is unchanged.

**Fixed and pinned** (`8ccb569`): `client-core` gained `is_standable`, the client previews only
cells the sim will keep, and `crates/simd/tests/serve.rs` now runs the round trip with
`client_core::is_standable` as the oracle and the **real daemon as the judge** — so the rule the
client uses and the rule the sim enforces are pinned against each other rather than free to drift
apart again. Suite 165 → 175; mutation table 27 → 33 rows.

**Sabotage round 2 — 6 rows, 6 KILLED, no survivor.**

| # | Sabotage | Test that went red |
| --- | --- | --- |
| 10 | channel and stockpile revert to the picked solid cell | `each_mode_key_sends_its_own_command_at_the_cell_the_sim_accepts` |
| 11 | clear stops reaching the cell the ray hit | `each_mode_key_sends_its_own_command_at_the_cell_the_sim_accepts` |
| 12 | standability drops its support check | `the_daemon_keeps_channels_and_stockpiles_only_at_standable_cells` |
| 13 | a missed ray erases the live preview | `the_drag_preview_survives_a_frame_whose_ray_misses_terrain` |
| 14 | preview promises marks the sim discards | `the_preview_covers_only_the_cells_the_sim_will_keep` |
| 15 | buried channel mark stays sealed in the rock | `a_buried_channel_mark_climbs_onto_the_rock_covering_it` |

Three rows in the round-1 table were re-pointed on the way — two anchors the fix reformatted and
one renamed test — all three caught by `audit-mutations.py` **before** the run, which is the guard
`17b4e94` built doing its job.

**ROUND 3 — the fixed build was driven and was STILL wrong, in two ways only measurement settled.**
Wolf: "still not there... dragging might skip 2 first blocks... stockpiling does pretty much
nothing usually". Both real, both quantified against the generated world rather than argued:

- **The face-neighbour rule lands 100% of TOP-face hits and 8.5% / 11.0% / 9.6% / 11.8% of
  East / West / North / South hits** (16,367 surface blocks sampled). On flat ground the cell
  beside a block is another block. Pointing at a block's *front edge* rather than its top
  designated nothing — that is "skip 2 first blocks". **The 2026-08-27 ruling did not survive
  contact**, and the measurement is the reason, not taste. **RE-RULED (Wolf): fall back to the
  cell directly above the block**, standable for 100% of surface blocks, while keeping the face
  neighbour where it *is* standable so a wall bordering a ledge still targets that ledge.
- **AC4's single-z rect keeps a MEDIAN 19.4% of a 6x6 stockpile footprint and 14.0% of a 10x10**;
  60% of 6x6 drags keep under a quarter. Standable cells exist only where the surface *is* the
  anchor's height, and a fixed z crosses a hillside in a thin band. **That is "stockpiling does
  pretty much nothing usually", and it had been true since the AC was written — a spec defect,
  not a code defect.** **RULED (Wolf): the standable modes follow the ground**, one cell per
  column, chosen nearest the height the drag began at so a ledge drag stays on its ledge.
  **Dig keeps the single-z rect** — cutting one level into a slope is what dig is for
  (dig keeps 88.9% / 58.3% / 51.0% at 3x3 / 6x6 / 10x10, which is the intended shape).

**Said plainly because it matters more than the fixes: part of what Wolf saw was the preview
finally telling the truth.** Before the round-2 fix it drew the full rect and the sim discarded
most of it in silence. The cells "missing" from his drags had always been thrown away. The fix
did not create that loss; it made it visible, and what it revealed was AC3/AC4 being wrong for
two of the four modes.

**AC3 and AC4 are AMENDED for the standable modes only** — they still hold verbatim for dig. The
followed surface is sent as **exact merged runs, never a bounding box**: a box would also cover
cells the drag never chose and the sim would silently keep any cave floor among them, which is
the same silent-wrong-cell class as the inert modes. `surface_targets` and `rects_for_cells` live
in `client-core` so the daemon's own test proves the coverage claim with the real binary as judge,
and so the preview is built from the very functions the release path sends.

**Sabotage round 3 — 7 rows, 7 KILLED, no survivor.** Eight round-1/2 rows had to be re-pointed
first, all eight caught by `audit-mutations.py` before the run. Table 33 → 40 rows, suite 175 →
180.

| # | Sabotage | Test that went red |
| --- | --- | --- |
| 16 | a side-face hit stops falling back to the cell above | `a_side_face_hit_on_flat_ground_falls_back_to_the_cell_above` |
| 17 | the fallback wins over a standable ledge | `a_side_face_hit_on_flat_ground_falls_back_to_the_cell_above` |
| 18 | the standable modes flatten back to the anchor level | `a_channel_drag_across_a_step_follows_the_ground_while_dig_stays_on_one_level` |
| 19 | dig follows the surface instead of cutting one level | `a_channel_drag_across_a_step_follows_the_ground_while_dig_stays_on_one_level` |
| 20 | the column scan stops one short of the cut surface | `each_mode_key_sends_its_own_command_at_the_cell_the_sim_accepts` |
| 21 | the followed surface is sent as one bounding box | `a_surface_following_drag_lands_its_whole_footprint_and_nothing_else` |
| 22 | the preview stops following the ground with the send path | `a_channel_drag_across_a_step_follows_the_ground_while_dig_stays_on_one_level` |

**WHAT CHANNELLING IS FOR**, asked twice and answered from the spec rather than the code: **FR9 —
dig is same-level, channel is dig DOWN, leaving a ramp.** It is the down-staircase. Stand on a
floor, channel it, and the block beneath becomes a walkable ramp so dwarves can descend; FR2 gives
the terrain rolling height specifically "to exercise climb pathfinding and channel digging". On a
flat floor, which is where it is actually used, the single-z rect was always right for it.

**WOLF'S CALL, 2026-08-27, closing the round:** *"ok well .. better .. so maybe it's ok at this
point.. it will get clearer with only real gfx.. now it's too confusing still to understand what
happens."* That is [[art-gates-visual-judgement]] applying to this story: placeholder cubes cap
what visual judgement is worth making, and he is declining to spend more on it. **Recorded as a
deferral, not a pass** — the difference matters and is spelled out below.

**What the round actually settled, and what it did not:**

| | State |
| --- | --- |
| The mechanism — four modes reach the sim at cells it keeps | **PROVEN**, real daemon as judge, 40/40 mutations killed |
| AC15/16/17 instruments, AC14 headless | **MET** |
| **AC13's rendered half** — is the hit-face highlight legible, distinct, clear of the reserved near-white | **DEFERRED to the gfx pass.** Not observed, not inferred |
| **AC19** — each of the four drags "takes effect in the same client", by eye | **PARTIAL.** "Better" is not four confirmed drags. The *takes-effect* half is answerable without art; the *reads-clearly* half is not |
| **AC18** — `tui` cross-check | **STILL OPEN**, and it is the one owed item that does not need art at all |
| NFR6 fps at both zooms | **NOT READ** |

**No fps figure was fabricated and no AC is being marked met on "better".** The honest summary is
that 8.2's mechanism is done and its *observation* is blocked on two different things: real art
for the look, and a short vehicle session for the readings.

**ORIENTATION, 2026-08-27 — a correct result that read as a bug.** Wolf, running the readout
pass: *"do we have coordinates wrong? I think I dig on north but got `*` in west."* Then, having
checked: *"yes I counted and checked the form .. not sure the direction though."*

**The coordinates were not wrong, and the count and footprint he checked were right.** Measured by
projecting known world offsets through the real boot camera:

| World direction | On the Bevy screen | In the TUI |
| --- | --- | --- |
| `+y` | up-right | down (south) |
| `-y` (north) | **down-left** | up (north) |
| `+x` | down-right | right (east) |
| `-x` | up-left | left (west) |

Solving for "straight up the Bevy screen" gives world **`-x, +y`** — a diagonal, because
`BOOT_YAW = 0.7` rad ≈ 40 degrees and the camera orbits freely with `A`/`D`. The TUI's screen axes
**are** the world axes. **World north lands DOWN-LEFT in the Bevy client at boot**, which nobody
would guess, and neither client said which way it faced. The chain that would have to be broken
for this to be a real defect was checked and is intact: `world_to_render`/`render_to_world`
round-trip under a hand-written handedness test, 8.1's review proved the DDA against an
independent oracle, the picked cell is what gets designated, the daemon keeps it, and the dug
stone spawns at `job.target` — so his comparison was fair. **The frames differ, not the data.**

**RULED (Wolf): a compass in both clients.** The TUI's is fixed (`N up`) because its axes cannot
move. The Bevy one is computed by projecting a north probe through the **same projection the
picking ray uses**, so a compass that disagrees with what is drawn is not possible, and it reports
`?` rather than inventing a bearing when no camera resolves. The Bevy readout also names the cell
under the pointer, and the `tui --frame` readout prints the world span of the marks at the cut —
between them, a cross-client check is a comparison of **numbers** rather than a reconciliation of
two orientations by eye, which is what produced this false alarm.

**Sabotage — 6 rows, 6 KILLED, no survivor.** Table 384 → 390 rows, suite 180 → 186.

**Recorded because it nearly went in the other direction:** the first run of this round reported
all six KILLED from a harness that passed a bogus flag to the test binary and **failed on a clean
tree** — six false kills. Caught by running the control before believing the result. The lesson is
the one `mutate.sh` already encodes and this hand-rolled loop did not: **verify the harness against
an unmutated tree before trusting a single verdict.**

**One flake, stated rather than swallowed:** `a_mid_haul_save_loads_and_the_daemon_keeps_ticking`
failed once in a full-gate run and passed alone and on the next full gate. It is a pre-existing
daemon test that this round did not touch, but this round did add two more daemon-spawning tests
to the serialized `serve.rs` set, which lengthens the run. Filed as deferred; a gate that goes red
one run in N is a gate nobody trusts.

**CONFIRMED ON THE VEHICLE, 2026-08-27** (Wolf): *"ok yes ..it's correct.. gui has just wider
perspective than tui so it was confusing but it's ok now with compass."* **The orientation
question is CLOSED as observed, not inferred** — a live confirmation that the coordinates agree
across both clients, which is the first thing on this story to be settled by eye rather than by
test.

His second clause is a finding in its own right and belongs with the instrument design: **the two
clients cover very different amounts of world.** The TUI draws one screenful of tiles — on the
order of 80x22 of a 128x128 world, roughly a tenth of it — while the Bevy client at
`BOOT_DISTANCE` frames the whole valley. So a mark plainly visible in one can be legitimately
off-screen in the other, with neither client at fault. **That is the second independent reason a
cross-client check must not be done by comparing pictures**, after the ~40-degree yaw, and it is
why the readout leads with `of X` — the mirror-wide count, which no viewport can clip — and prints
a world span rather than asking anyone to match two views by eye.

**THE READOUT REPORTED ZERO WHILE THE MARKS WERE VISIBLE, 2026-08-27.** Wolf: *"that `--frame`
gives me 0 but I can see in tui."* Not a defect in the marks — he was reading a different cut than
they were on. **No single `--z` can show all four modes**: a dig sits at the cell the ray hit while
a channel or a stockpile sits one level up, and an interactive `tui` opens at `opening_z` rather
than at whatever was passed to `--frame`. `0 of 0` was, once again, indistinguishable from "the sim
kept nothing" — **the exact failure this readout was built to remove, reappearing one level along**.
It now names the levels that DO hold marks and says what to do:

```
marks: z 20 designations=0 of 0 zones=0 of 0
       marks at OTHER levels -- z 8: 9 designations, 0 zones  z 9: 0 designations, 9 zones
       nothing at z 20. A dig sits at the cell the ray hit; a channel or a stockpile sits
       ONE LEVEL UP. Re-read with --z set to one of the levels above.
```

2 further sabotage rows, both KILLED; table 390 → 392, suite 186 → 187. The runbook card now says
to expect two reads per drag pair.

**THE FLAKY DAEMON TEST IS CLOSED, on the second sighting this story's own deferral asked for.**
`read_snapshot_after_load` budgeted **four lines** for the load snapshot — a timing assumption
wearing a budget's clothes, since the daemon ticks at 10 Hz throughout and how many deltas arrive
first is set by machine load. **The same unit error as M2-15.** Now a deadline with a runaway
backstop; five consecutive runs green, then a full gate green. Worth noting against M2-15's own
record: that action item named `--frames` in `gui`, and the identical mistake was sitting
unnoticed in the daemon's test harness the whole time.

## RESUME HERE — 2026-08-28

**Session paused 2026-08-27 by Wolf; re-checked 2026-08-28.** Tree clean, **38 commits on
`8-2-designate-with-the-mouse`, none pushed, no PR** (21 on 08-26, 17 on 08-27 — the "16" written
here on the 27th counted that day only). Status stays `in-progress`.

**Binaries are current — do not rebuild blind, check the stamps:**

| Binary | Built | From |
| --- | --- | --- |
| `target/x86_64-pc-windows-gnu/release/gui.exe` | 2026-08-27T16:54:53Z | `8ee683c`, the last commit touching `gui`/`client-core`/`protocol` |
| `target/debug/tui` | 2026-08-27T17:26:07Z | `5880e51` |
| `target/debug/simd` | 2026-08-27T15:41:19Z | unchanged since; `5880e51` touched only its *tests* |

**Copy `gui.exe` Windows-side and check the copy's mtime** — the stale-binary trap has fired five
times and the copy is the file that has been stale every time. The runbook card named a *sixth*
variant of it on 2026-08-28: its Binaries paragraph still pointed at the `e01e7ff` build from
15:05:25Z, one commit behind the `gui.exe` on disk and without the compass or cursor readout.
Corrected in place — the card now carries the same three-row table as above.

**The only work left is the readout pass** — `8-2-signoff/task-7-vehicle-runbook.md`, the card at
the top. It asks no visual judgement. Owed:

- [ ] Four `marks:` lines, one per mode. **Two reads per drag pair**: a dig sits at the cell the
      ray hit, a channel or stockpile ONE LEVEL UP, so no single `--z` shows all four. Write the
      footprint down BEFORE reading; a count with no expectation is not a check.
- [ ] Two fps readings, `F3`, working zoom and full vista. **A failed reading is the result.**

Dig was already confirmed by Wolf on 2026-08-27 — he counted it and checked the form — and the
orientation question is closed as observed. Channel, stockpile and clear plus the two fps numbers
are the remainder. When they land: AC18, AC19's takes-effect half and NFR6 close on evidence, and
**8.2 goes to `done` with AC13's rendered half and AC19's reads-clearly half filed against the gfx
pass**, which is where Wolf ruled them.

**Do not** re-open the look questions, re-tune a colour, or claim an AC on "better".

---

**gui.exe MUST BE REBUILT before the session resumes.** The 05:13:38Z binary predates all of this
and has two dead modes in it.

**Owed by the rest of this session:** AC19's four hand drags on surface and slice, AC13's rendered
half on a cliff face / corridor wall / shaft side, AC15 and AC16 end to end, AC18's `tui`
cross-check, and the two fps readings. None observed yet.

### File List

- _bmad-output/implementation-artifacts/8-2-signoff/task-7-vehicle-runbook.md (new)
- crates/gui/src/command.rs (new)
- crates/gui/src/designate.rs (new)
- crates/gui/src/lib.rs
- crates/gui/src/ingest.rs
- crates/gui/src/pick.rs
- crates/gui/src/project.rs
- crates/gui/src/capture.rs
- crates/gui/tests/headless.rs
- crates/gui/tests/capture.rs
- _bmad-output/implementation-artifacts/mutations/8-2-designate-with-the-mouse.sh (new)
- _bmad-output/implementation-artifacts/mutations/8-1-point-at-the-world.sh
- _bmad-output/implementation-artifacts/mutations/m2-1-live-app-systems.sh
- _bmad-output/implementation-artifacts/8-2-designate-with-the-mouse.md

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-26 | Story created. Four decisions ruled by Wolf at creation: press-drag-release, digit mode keys, hit-face highlight, `--capture-at-tick` in scope. Epic premises re-verified: the command set and `rect_on_level` hold; `simd`'s rect validation is already built and tested, so the epic's second AC2 clause is inherited rather than owed. |
| 2026-08-26 | Implemented and committed Tasks 1–3 and the completed portions of Task 4; remaining instrument, mutation, vehicle, and final-gate work stays in-progress. |
| 2026-08-26 | Tasks 3-6 and 8 completed across three further delegated sessions. `--drag` and `--at-tick` instruments built and tested; 9-row sabotage table run ALONE, **9/9 KILLED, no survivor**; full gate GREEN on a cold rebuild. Status → review. |
| 2026-08-27 | Task 7's runbook written ahead of the session (`8-2-signoff/task-7-vehicle-runbook.md`), sourced from `aca07be` rather than carried forward from 7.2. Separately, the review's `.codex/` deferral was reopened and closed: the directory holds a live `auth.json` and the `.gitignore` secret patterns do not match that name. |
| 2026-08-26 | Owed and stated rather than glossed: Task 7/AC19 unobserved (no devpod can open a window), M2-7's build stamp missing for the **fifth** time, and `codex review --base main` **never ran** (killed twice, quota-blocked once). |
