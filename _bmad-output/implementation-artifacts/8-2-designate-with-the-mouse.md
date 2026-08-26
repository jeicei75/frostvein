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

- [ ] **Task 3 — The rect and the commands (AC: 3, 4, 10, 11, 12)**
  - [x] Build every rect with `client_core::rect_on_level(anchor.xy, release.xy, anchor.z)`.
        The anchor's z is the level; the release tile's z is discarded. Add a `// NOTE:` naming
        that limitation — a drag up a cliff designates on the anchor's level, which is the
        single-z rect rule (AD-18), not a bug.
  - [x] Map mode → command: Dig/Channel → `Designate { kind, rect }`, Stockpile →
        `PlaceStockpile`, Clear → the two-command pair above.
  - [x] Nothing here consults tile contents, reachability or job rules. `gui` holds no game
        logic (AD-4); the sim decides what a rect means.
  - [ ] AC12's test: ingest a delta carrying the designation, run **one** `app.update()`, assert
        the `ProjectedDesignation` entity exists. Two updates passing is not the assertion.
  - [ ] AC11's test runs the whole path at a slice level below the world top.

- [ ] **Task 4 — The hit-face highlight and the drag preview (AC: 13)**
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

- [ ] **Task 5 — The instruments (AC: 15, 16, 17)**
  - [ ] `--at-tick N` in `parse_args_from` (`ingest.rs:288`) and `--drag <mode>,<x0>,<y0>,<x1>,<y1>`.
        Both require `--capture`, matching the `--distance`/`--cursor` shape (`ingest.rs:344-350`).
        **A typo'd flag is silently swallowed as the TCP port** (`ingest.rs:332-333`) — reject an
        unparseable value explicitly, as `parse_cursor` does.
  - [ ] `--drag` writes the real cursor and the real `ButtonInput<MouseButton>` across
        successive frames so the scripted run enters the **same** mode machine a human does.
        A flag that builds a rect directly proves nothing about this story's headline outcome
        (M2-11, and 7.2's `--distance`).
  - [ ] `--at-tick` records the mirror tick at connect and fires when `tick >= start + N`. If
        the frame budget is exhausted first, print what tick was reached and exit non-zero.
  - [ ] Range-check: the existing `marks:` line already prints `designations=A of B zones=C of D`
        (`capture.rs:589`). Assert the expected count is **non-zero** before comparing —
        7.2 photographed an empty site and exited 0 because both sides agreed on zero.
  - [ ] Both instruments get their own tests, driven through `capture_after_frames` under
        `MinimalPlugins` as `crates/gui/tests/capture.rs` already does.
  - [ ] `--frames` stays; `--at-tick` is the tick-anchored alternative, not a replacement.
        **Do not bake any rect into the binary** — the scenario is the caller's (M2-15).

- [ ] **Task 6 — Sabotage table (AC: 20)**
  - [ ] `_bmad-output/implementation-artifacts/mutations/8-2-designate-with-the-mouse.sh` in the
        house format — `assert s.count(old) == 1` guard on every edit.
  - [ ] Minimum rows: `send_commands` dropped from its registration tuple; the built command
        discarded instead of queued; `rect_on_level` replaced by a hand-rolled `min`/`max`;
        the anchor's z replaced by the release tile's z; the `Esc`/right-button abort still
        sending; clear mode sending only one of its two commands; `--drag` parsed but never
        reaching the mouse state; `--at-tick` firing on the frame count instead of the tick;
        the highlight's face offset replaced by the old unconditional `+Y*0.55`.
  - [ ] **Commit before running** (M2-9). Run `scripts/mutate.sh` **alone** — it is not
        concurrency-safe. Capture the exit code before any pipe.
  - [ ] **Dry anchor-check first** (M2-8), and again after `cargo fmt` — formatting moves
        anchors, and 8.1's row 2 went stale mid-round exactly this way.
  - [ ] Widening `PickedTile` reformats `pick.rs` and `project.rs`. Re-run
        `python3 scripts/audit-mutations.py` and repair **8.1's** table if a row stops applying
        — at 8.1 a helper broke a row in *5.4's* table, in a file that story never opened.

- [ ] **Task 7 — VEHICLE-BOUND: the whole feature, by hand (AC: 19)**
  - [ ] **Rebuild and re-copy `gui.exe` first. Record the build wall-clock time and the source
        commit.** 8.1 recorded the commit and not the time; do both.
  - [ ] Drag each of the four modes on the surface and on a sliced level; confirm each takes
        effect, and that the hover highlight is visible on a cliff face.
  - [ ] Read sustained fps at working zoom and at full vista from the F3 overlay. A failed
        reading is the finding and gets reported, not worked around.
  - [ ] Write `8-2-signoff/task-7-vehicle-runbook.md` from the worked example at
        `7-2-signoff/task-6-vehicle-runbook.md` — the commands that actually ran, corrected.

- [ ] **Task 8 — The gate (AC: 1)**
  - [ ] `cargo clean -p gui`, then `scripts/gate.sh` full tier. Paste the tail. A
        `GATE GREEN (FAST)` line is a coverage hole, not a pass.

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

Codex (GPT-5)

### Debug Log References

- RED (Task 1 writer literal): `left: {"type":"designate","kind":"dig",...}` differed from
  the first hand-written, externally shaped enum literal.
- RED (Task 2 harness): before clearing `ButtonInput` transition state, the drag asserted
  `left: Rect { min: [2, 1, 0], max: [2, 1, 0] }` against the anchor-level literal rect.
- RED (Task 3 seam sabotage): replacing the enqueue with a discard made the loopback server read
  fail with `WouldBlock` at `ingest.rs:993`.
- RED (Task 4 face sabotage): replacing the picked normal with `Vec3::Y` produced
  `left: Vec3(3.0, 5.55, -4.0)`, `right: Vec3(3.55, 5.0, -4.0)`.

### Completion Notes List

- Tasks 1–3 provide the committed upstream drag slice: concrete timeout-bound TCP writer,
  digits/mode/hint UI, anchor-level rect construction, and an end-to-end real mouse to real
  loopback-byte test. Clear maps to both existing protocol commands in TUI order.
- Task 4 is partially complete: face-aware pick data, face-normal hover transform and client-local
  drag preview are committed and headlessly pinned. AC11 and AC12's dedicated projection tests
  remain unchecked, so Task 4 itself remains open honestly.
- Repaired the two stale 8.1 mutation anchors caused by the widened pick and preview code; the
  audit reported all 342 existing rows applicable.
- Tasks 5–6 are not started. Task 7 is vehicle-bound and owed to a live gingerspice session; no
  window, capture, or FPS observation was attempted in this devpod. Task 8 and `codex review`
  are deferred until the story is complete.
- `cargo clean -p gui` completed (removed 3,625 files / 14.7 GiB). Three attempts to run the
  full gate reached `cargo test` after fmt and clippy succeeded, but this execution environment
  terminates foreground commands at 30 seconds before the daemon integration suite returns. No
  full-gate result is claimed. The per-commit fast gate and focused GUI tests were green.
- `codex review --base main` was launched once (session `01a03e72-26bb-74c3-9110-903c84111f12`),
  but the same 30-second runner limit ended it during repository inspection before it emitted a
  findings report. No review finding is claimed or silently dropped.

### File List

- crates/gui/src/command.rs (new)
- crates/gui/src/designate.rs (new)
- crates/gui/src/lib.rs
- crates/gui/src/ingest.rs
- crates/gui/src/pick.rs
- crates/gui/src/project.rs
- crates/gui/src/capture.rs
- crates/gui/tests/headless.rs
- _bmad-output/implementation-artifacts/mutations/m2-1-live-app-systems.sh
- _bmad-output/implementation-artifacts/mutations/8-1-point-at-the-world.sh
- _bmad-output/implementation-artifacts/8-2-designate-with-the-mouse.md

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-26 | Story created. Four decisions ruled by Wolf at creation: press-drag-release, digit mode keys, hit-face highlight, `--capture-at-tick` in scope. Epic premises re-verified: the command set and `rect_on_level` hold; `simd`'s rect validation is already built and tested, so the epic's second AC2 clause is inherited rather than owed. |
| 2026-08-26 | Implemented and committed Tasks 1–3 and the completed portions of Task 4; remaining instrument, mutation, vehicle, and final-gate work stays in-progress. |
