# Deferred Work

Items surfaced by review that were real but not actionable at the time. Each entry
names where it came from and what should trigger revisiting it.

## Deferred from: code review of 1-1-a-seeded-frozen-world-exists (2026-08-02)

- **A spawn that ignores the RNG draw is not detected by the test suite**
  (`crates/sim-core/src/lib.rs:166`). Replacing `rng.random_range(0..candidates.len())`
  with a constant index still passes all six tests, including the strengthened
  cross-seed dwarf-position assertion — because the candidate list is itself
  terrain-derived and therefore seed-dependent, so positions still differ between
  seeds. Left open deliberately: it violates no AC (AC7 requires distinct positions,
  valid tiles and allocator-issued ids; AC8/AC9 still hold) and contradicts only the
  task wording "positions chosen from the worldgen stream". Closing it requires
  asserting scan-order properties — either duplicating production candidate logic in
  the test or a brittle clustering heuristic — both worse than the gap under the
  project's simplicity policy. **Revisit if** spawn placement becomes
  gameplay-relevant (embark-site selection), which would give it a real AC to test
  against.

- **Single RNG stream couples dwarf positions to terrain's exact draw count**
  (`crates/sim-core/src/lib.rs:79-91`). One `ChaCha8Rng` is threaded sequentially
  through `height_field` → `layered_terrain` → `spawn_dwarves`. `layered_terrain`
  consumes exactly `dims.x * dims.y` bool draws before the spawn code reads a single
  value, so any later change to surface-material selection (a third material, a
  skipped draw for ramp columns) silently relocates all five dwarves and invalidates
  every recorded scenario baseline and save file — with no test failing to explain
  why. Story 1.1 mandated the single `STREAM_WORLDGEN` stream, so the code is
  compliant. AD-7's "purpose-named streams" is the relevant architectural decision.
  **RESOLVED — scheduled into Story 2.2 (AC2, AC3), Wolf's call 2026-08-03**: spawn draws from
  its own `STREAM_SPAWN`, and the five positions for seed 42 are pinned as literals so a future
  terrain-draw change cannot move them silently. Rationale: without the split, a pinned test
  degrades into pasting over new values whenever terrain changes, which trains the signal away;
  and 2.4's `SaveState` baselines make the fix costlier from then on. Evidence is inverted
  sabotage — an extra `layered_terrain` draw must leave the pinned test green.
  ~~**Revisit at Story 2.4**, when `SaveState` must persist RNG stream state.~~
  ~~**CORRECTED 2026-08-03 (Epic 2 dependency sweep): revisit at Story 2.2, not 2.4.**~~
  AD-7 names two purpose-named streams — worldgen and **wander** — and the wander stream
  is born in 2.2. Further, `World` retains no RNG state at all today: the `ChaCha8Rng` is
  a local inside `generate()` and is dropped when it returns [crates/sim-core/src/lib.rs:95].
  So 2.2 must both split the streams and persist them on `World`; 2.4 only serializes what
  2.2 creates. Acting at 2.4 would mean recording save baselines against a stream layout
  that 2.2 had already invalidated.

- **Spawn distribution is biased toward the map border**
  (`crates/sim-core/src/lib.rs:143-153`). `is_flat` filters out-of-bounds neighbours
  before `.all()`, so a corner column is judged on 2 neighbours while an interior
  column is judged on 4 — border columns are systematically likelier to qualify.
  Observed: seed 0 spawns a dwarf at `Pos { x: 0, y: 26, z: 20 }`, hard against the
  wall. No AC requires interior spawning, so this is an unintended distribution
  rather than a defect. **Revisit if** dwarf starting position becomes
  gameplay-relevant, or when a real embark-site rule replaces the placeholder.

- **Story artifacts reference planning docs that are untracked**
  (`_bmad-output/implementation-artifacts/1-1-a-seeded-frozen-world-exists.md`
  References section). The story cites `_bmad-output/planning-artifacts/epics.md`,
  the architecture spine, the PRD, the implementation-readiness report, and
  `docs/architecture.md` — all currently untracked in git. This branch commits the
  artifact while leaving every dependency uncommitted, so a fresh clone gets a story
  whose evidence chain dangles. Pre-existing repo hygiene, not caused by this story.
  **Revisit when** Wolf decides what of `_bmad-output/` belongs in version control.

## Deferred from: code review of story 1-3-behold-the-frozen-world (2026-08-03)

- **A panic in the interactive loop is invisible.** The panic hook prints to stderr while the
  alternate screen is still active; `TerminalGuard::drop` then issues `LeaveAlternateScreen`, which
  restores the primary buffer and wipes the message. Observed behaviour is "the client vanished, no
  error" [crates/tui/src/main.rs:20-27,82].
- **No SIGTERM/SIGHUP handling.** `TerminalGuard` runs only on return or unwind, so a killed client
  leaves the terminal in raw mode and the alternate screen, requiring `reset`
  [crates/tui/src/main.rs:26-35].
- ~~**AC11's 100×40 fallback is unreachable on Linux.**~~ **RESOLVED 2026-08-03** (at 2.2 story
  creation, per the retro item "fix it at the next TUI story"). crossterm's `terminal::size()`
  shells out to `tput cols`/`tput lines` before returning `Err`, so the fallback cannot fire while
  `tput` is on PATH; a no-TTY frame renders at whatever terminfo guesses (verified: 80×24 = 1920
  cells). Side effect: two forked `tput` processes per `--frame` run. This was a spec-accuracy
  issue in the AC text, not a code defect: story 1.3's AC11 now states that the frame renders at
  the reported size and falls back to 100×40 only on an error or a zero dimension, which is what
  `frame_size` does [crates/tui/src/main.rs:214-219]. No code changed.

## Deferred from: code review of story 2-1-the-world-runs-on-its-own-clock (2026-08-03)

- **The status line outgrows an 80-column terminal as the tick counter gains digits.** `view.rs`
  now prepends `"tick {}  "`, taking the status row from 69 to 78 columns at `tick 87`. Truncation
  is a silent `(0..w).zip(status.chars())`, so at `tick 1000000` (~28 h of uptime at 10 Hz) the row
  reaches 84 columns and pushes `q quit` off screen. Graceful, not a panic. **Revisit when** the
  next TUI story touches the status line. (The AC11 spec-text item it was once paired with is
  closed; 2.2 colors dwarves by state and deliberately leaves the status line alone, so this
  stays open) [crates/tui/src/view.rs:131-147].
- **The dirty-tile path is inert in production.** `set_tile` has zero production callers — every
  call site is a test — so the schedule's one system touches only `Tick` and `Delta.tiles` is `[]`
  on every real tick (confirmed on a live wire dump: four consecutive deltas, `tiles=[]` each
  time). This is the story's explicit, recorded decision: build the AD-8 mechanism now, prove it
  with AC4's direct test, first real producer arrives with the dig. **Revisit at Story 3.2** —
  which is the first time a `TileChange` crosses the wire for real, and the first time this
  plumbing is exercised end to end. Do not read AC6 as evidence that tile streaming has been
  proven [crates/sim-core/src/lib.rs:169-178].

## Deferred from: code review of story 2-2-dwarves-wander-the-frost (2026-08-03)

- **`NO_COLOR` silently deletes the entire visual feature this story shipped.** crossterm gates
  every colour sequence on the `NO_COLOR` env var, and colour is 2.2's *only* signal for job
  state — there is no glyph, brightness or marker fallback. With it set, an idle dwarf and a
  walking dwarf render as the identical uncolourised `☺` and nothing in the client says the
  distinction has been dropped. This devpod sets `NO_COLOR=1` by default, which already made one
  round of the dev agent's own colour evidence vacuous until it was rerun with the var unset —
  so the trap has bitten once. **Not** patched here: a fallback glyph is a design change beyond
  this story, and "24-bit truecolor from the start, colour as data" is a recorded stack decision
  in `docs/technical-preferences.md` rather than an oversight. **Revisit when** a story adds a
  second state signal or the first accessibility pass.
  **Partly closed the same day, before story 2.3.** The *evidence* half is fixed: `tui --frames N`
  now checks `NO_COLOR` itself (mirroring crossterm's rule — set and non-empty) and warns on
  stderr that the capture cannot evidence job-state colours, so a colourless capture can no longer
  be mistaken for proof that the colours work. Two tests drive the real binary — one asserts a
  walking dwarf reaches the capture wearing `38;2;214;154;78`, the other asserts the warning
  appears when colour is suppressed — and both have mutations in the 2.2 set. What remains
  deferred is the *product* half: a player who runs with `NO_COLOR` set still gets no state signal
  at all, because there is no glyph or brightness fallback. That is a design change and still
  belongs to a later story [crates/tui/src/main.rs, crates/tui/src/frame.rs:50-60].

## Deferred from: code review of story 2.3 (2026-08-04)

- ~~**Partial line at EOF is misreported as a 64 KB overflow.**~~ **RESOLVED in Story 3.1
  (2026-08-05).** `read_inbound` now distinguishes a short EOF-terminated partial line from a
  line that reaches `MAX_LINE_BYTES`; the former is reported as an unterminated partial line and
  only the latter as an overflow. A focused daemon test pins the distinction. Previously,
  `read_inbound` treated *any*
  `read_until` result not ending in `\n` as having hit `MAX_LINE_BYTES`, so a client that sends
  a truncated command and closes is logged as `client line exceeded 65536 bytes; closing
  connection`. Proved live with 24 bytes + FIN. A real flood and a benign mid-command
  disconnect are therefore indistinguishable in the logs — an observability defect in a project
  whose recurring failure class is false evidence. The correct pattern already exists in this
  repo: `crates/tui/src/main.rs:366` splits the two cases on `bytes as u64 >= MAX_*`. **Not**
  patched here: it is pre-existing 1.2 code and CLAUDE.md rule 3 forbids improving adjacent code
  that this story did not break. **Revisit when** a story next touches `read_inbound` — 3.1 adds
  world-mutating commands and will [crates/simd/src/main.rs:270].

- **A terminal persistently reporting 0×0 renders nothing forever, with no diagnostic.** The
  startup-0×0 case is already handled and documented — the render guard re-queries the size
  rather than trusting the startup reading. What remains is that nothing ever *reports* the
  condition: keypresses still reach the daemon and the daemon still obeys them, but the screen
  stays blank, which a user reports as "the client is broken". Hit live by a review layer
  driving the TUI under `script` with redirected output. **Revisit when** the first
  accessibility/robustness pass lands, or alongside the still-open `NO_COLOR` product-half item
  above — both are the same shape: the client silently drops its only output channel
  [crates/tui/src/main.rs:240-251].

- **Fast mode shrinks the client-queue slack fivefold.** `CLIENT_QUEUE` is 16 messages, which is
  ~1.6 s of buffer at the 100 ms tick but only ~320 ms at the 20 ms fast tick. A terminal that
  reads slowly is markedly likelier to hit the bounded-queue eviction path while the session is
  fast-forwarding — precisely when the operator is least likely to be watching that terminal.
  Not a defect in 2.3: the eviction path is 1.2's and is working as designed. **Revisit when**
  eviction is observed in practice, or when a story adds reconnect (deliberately out of scope
  through Epic 2) [crates/simd/src/main.rs:22].

- ~~**Stale-speed compose trap in the TUI keymap (owner: Story 3.1).**~~ **RESOLVED in Story 3.1
  (2026-08-05).** The client now updates its local speed optimistically when it emits a speed
  command, so consecutive keys compose from the intended local value; every authoritative wire
  update overwrites that prediction. A mapping test and mutation pin both halves. Previously,
  speed for the next command
  is read from the last delta, never from an optimistic local value. Two *different* keys pressed
  inside one 100 ms round-trip therefore compose against the same stale baseline and settle on a
  speed neither implies: at `Normal`, `+` sends `SetSpeed{Fast}` and `-` sends `SetSpeed{Paused}`,
  and the daemon's last-write-wins drain lands on **Paused**. Because speed is a single shared
  value broadcast to every client, one operator's fumbled double-tap silently pauses the session
  for every watching terminal, with no error and no indication why. Found independently by three
  review layers at 2.3 and reproduced both in isolation and against a live daemon. **Not** patched
  at 2.3: the fix is optimistic client-side speed, which 2.3's scope guardrails and AD-4
  explicitly forbid; Wolf's call at review was to accept it now with an accurate `// NOTE:`.
  **Revisit at Story 3.1**, which already owns the pause/command-consumption split and is the
  natural home for client-side command state [crates/tui/src/view.rs:180-195].

## Deferred from: code review of 2-4-the-world-endures (2026-08-04)

- ~~**No in-UI affordance for `Command::Quit` (owner: Story 3.1).**~~ **RESOLVED in Story 3.1
  (2026-08-05).** The normal-mode hint now says `q quit client`, making the shared-daemon lifetime
  explicit without adding the deliberately forbidden daemon-shutdown key. The wire command, the daemon arm
  and the clean shutdown all work — `quit` logs `shutting down on client quit`, exits 0, and every
  connected client sees EOF. But nothing in the shipped client can send it: AC9 deliberately forbids
  a client quit key, because a shared daemon must not die from one viewer's keypress. The result is
  that the status line advertises only `q quit`, `q` closes the client while the daemon keeps
  ticking, and the only way to stop the daemon is a raw TCP client (the story's own Verification
  section uses `nc`). Spec-sanctioned, so not patched at 2.4. **Revisit at Story 3.1**, which owns
  FR21's hint bar and is the natural place to either surface a shutdown affordance or state plainly
  that the daemon outlives the client [crates/tui/src/view.rs:222].

- **`MAX_SAVE_BYTES` and `Dims::DEFAULT` are not tied together.** The read cap and the write refusal
  share one constant, so those two cannot diverge by edit — but neither is connected to world size.
  The live save is 6,910,452 bytes against a 16,777,216-byte cap: 2.4x headroom. Grow the default
  world past roughly that factor and `save_world` starts refusing every save while still logging,
  so the fortress silently stops being savable. The suite does catch it —
  `saved_file_decodes_as_a_save_state` fails with `saved file must exist` — but that message points
  at the wrong thing entirely, so whoever bumps `Dims::DEFAULT` will not learn why. **Revisit when a
  story changes world dimensions**; the cheap fix is one assertion tying the default world's encoded
  size to the cap [crates/simd/src/main.rs:24].
