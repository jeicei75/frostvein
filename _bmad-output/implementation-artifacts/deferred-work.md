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
  **CLOSED in Story 5.1:** the real embark-site rule now selects the nearest qualifying
  central 7x7 flat clearing, and all five seeded spawn draws are restricted to it.

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
  **SCHEDULED into Story 3.2 (2026-08-06).** Its dig and channel jobs are the first production
  callers of `set_tile`, and AC12's task carries the live-daemon test that finally reads a delta with
  a non-empty `tiles` off a real socket. Until that test is green, this item stays open.
  **CLOSED by Story 3.2 (2026-08-06):** dig/channel now call `set_tile`, and the bounded live-daemon
  test observes a single real delta carrying both non-empty `tiles` and non-empty `items`.

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

  **UPDATE — this fired at Story 3.1 (2026-08-05), by the other route.** Not a dims change: 3.1
  added *state*. Designations clip to the whole world, so a legal command produced a 23.2 MB save
  against the 16 MB cap and the world became unsaveable. The cap was raised to 64 MB (matching the
  client's `MAX_SNAPSHOT_BYTES`) at 3.1's review. **The underlying defect is unchanged and still
  open:** the cap is a hand-picked constant, not derived from the largest legal world, so the next
  story that adds per-tile state can silently re-break it. The prediction above was right about the
  mechanism and wrong only about which input would grow.

## Deferred from: code review of 3-1-give-the-order (2026-08-05)

- **AD-8's full-resend makes designation volume a wire amplifier.** Every delta carries *every*
  designation, so cost scales with total marks rather than with what changed. Measured live at
  3.1's review: one designate command clipping to the whole world (128x128x32 = 524,288 marks) takes
  a delta from 378 bytes to **16,761,209 bytes**, sustained at **34.7 MB/s** (11 deltas in 5.3 s) to
  every connected client, with no recovery short of a daemon restart. Reachable from the shipped
  client, not only a hostile one: the TUI clamps a rect to a single z-level (16,384 marks, ~5 MB/s),
  so 32 ordinary designate commands reach the full volume. Deliberately not fixed at 3.1 — every
  candidate fix either changes AD-8's full-resend contract (absence means deletion) or invents a
  game rule bounding how much may be designated, and both are architecture calls rather than
  patches. **Owner: Story 3.2**, which adds the job market on top of designations and must revisit
  this area regardless; it will make marks more numerous, not fewer. Note the interaction already
  closed: the save half of this same root cause hard-failed and was fixed by raising the cap above
  [crates/simd/src/bridge.rs:74-92, crates/sim-core/src/lib.rs:445-470].

  **SCHEDULED into Story 3.2 (Wolf's ruling, 2026-08-06), not re-deferred.** The fix is a
  deterministic `MAX_DESIGNATIONS = 4096` cap inside `apply_command`, which bounds the worst-case
  delta at roughly 131 KB rather than 16.8 MB. Chosen over delta-encoding designations, which would
  have broken AD-8's "absence is deletion" full-resend contract for one section and made a client
  resync bug silent; and over a third deferral. Two things to read carefully. (1) The cap is a *game
  rule* about how much may be marked, and it lives in `sim-core` precisely so it is deterministic —
  a cap applied in `simd` would not survive the scenario harness. (2) 3.2's diggability filter does
  most of the work independently: a `Dig` mark is recorded only on `Solid` tiles and a `Channel` mark
  only where `is_standable` holds, so a surface rect that used to mark every tile now marks almost
  none. The cap is what bounds the remaining case, a rect through solid rock.
  **CLOSED by Story 3.2 (2026-08-06):** `apply_command` enforces the deterministic 4,096-mark cap
  and the kind-specific diggability filter described above.

## Deferred from: code review of 3-2-the-dig (2026-08-06)

Four layers ran, all completed — no coverage holes. Each item carries its originating LAYER and
SEVERITY. Every item below is LOW; HIGH/MED went to patch or to Wolf as a decision.

- **A Channel job is permanently orphaned when an independent Dig removes its support**
  [`crates/sim-core/src/lib.rs:463-469`] — LAYER: edge-case-hunter. SEVERITY: med.
  **Wolf's ruling, 2026-08-06: leave as specified. No code change.** Designate a channel at `P`
  (needs `is_standable(P)`, so `P.z-1` is `Solid`), then separately designate a dig at `P.z-1` —
  independently valid, it is solid. If the dig lands first, `is_standable(P)` is false forever,
  `work_positions` returns the empty set, and no dwarf can ever take the job. Live-verified over
  500 ticks: `jobs().len()` pinned at 1, `retry_after` climbing to 602, never converting to `Ramp`.
  WHY THIS IS NOT A DEFECT: AC11 deliberately specifies "the job stays in the market and is retried,
  never dropped (FR8)". The retry is nearly free — `astar_with_budget` early-returns on an empty
  goal set *before* spending any node budget — so there is no starvation and no performance cost.
  The designation mark stays visible on screen and `x` cancels it, so the player is not without a
  signal. REVISIT only if it actually bites in play. If it ever is addressed, note that "permanently
  unwinnable" is not trivially decidable: a channel target can become standable again if the tile
  below is refilled, and nothing in phase one refills terrain — but that is an assumption about
  today's rules, not an invariant.

- **`jobs.next_id += 1` is a plain add** [`crates/sim-core/src/lib.rs:176`] — LAYER:
  edge-case-hunter. Every neighbouring tick/id computation added by the same story uses
  `saturating_add` (`created_tick.saturating_add`, `tick.saturating_add(RETRY_COOLDOWN)`), which
  makes this one look like an oversight rather than a choice. In debug it panics on overflow; in
  release it wraps to 0, and because `Jobs::insert`'s uniqueness check only looks at *live* jobs, a
  wrapped id could silently reuse a long-completed `JobId`. Deferred because it needs ~4 billion
  lifetime job creations to reach — but note `simd`'s `load_world` already rejects an exhausted
  `next_job_id` on load, so the guard exists on one side only. One-line fix when next in this
  function.
  **CLOSED at Story 3.3 (2026-08-07).** The second allocation site arrived, so the increment became
  one `Jobs::next_job_id()` using `saturating_add`, shared by `create_jobs` and `create_haul_jobs`.
  Pinned by `next_job_id_counts_up_and_saturates_at_the_maximum` and by a `wrapping_add` mutation.

- **The AD-12 seam promised to Story 3.3 is heavier than planned** — LAYER: acceptance-auditor.
  The guardrail says 3.3 adds `Haul` "as a variant plus its execution system and **must not touch
  claiming logic**". Because `claim_jobs` now computes work positions, runs A* and spends a shared
  per-tick node budget, a `Haul` variant will require both `work_positions` and the claim-time
  search to understand hauling. Not an AC violation and not a defect — but the seam is no longer
  purely `JobKind`, and 3.3's planning should budget for it rather than discover it.

- **Crowd counting and crowd drawing use different entity filters** [`crates/tui/src/view.rs`] —
  LAYER: acceptance-auditor. `dwarf_counts` counts only `EntityKind::Dwarf`, but the draw loop
  applies `crowd_cell()` to *any* entity that lands on a crowded screen index. Harmless today
  because dwarf is the only `EntityKind`, and AC14 only speaks about dwarves. It becomes a silent
  visual defect the moment a second entity kind reaches the wire — which is the same failure shape
  as the 2.2 defect AC14 exists to fix.
  **CLOSED at Story 3.3 (2026-08-07).** The cell contention rule was rewritten for the carrier
  glyph, and both loops now filter on `EntityKind::Dwarf`; the stone count is built in the very pass
  that draws the stones, so count and draw cannot disagree about screen position or level either. A
  `// NOTE:` records that a second entity kind must decide its own contention rule. Pinned by a
  wrong-z stone case and by a "counting and drawing use different filters" mutation.

- **The daemon test lock lengthens the gate** [`crates/simd/tests/serve.rs`] — LAYER:
  acceptance-auditor. A new `static DAEMON_TEST_LOCK` serializes *every* daemon test in the file and
  `read_delta_with_speed`'s retry budget went 5 to 50. It was added for a real reason — dozens of
  concurrent daemon processes competing on timing assertions — and no guardrail forbids it. But it
  serializes tests this story never touched and changes their failure mode, so a future flaky-gate
  investigation should know it is here. Revisit if gate wall-clock becomes a problem.

## Deferred from: code review of 3-3-the-haul-and-the-skeleton-walks (2026-08-07)

Only ONE of four review layers reported: the Acceptance Auditor. Blind Hunter, Edge Case Hunter and
Feature Auditor were killed at the 20-minute time-box having produced nothing, so the adversarial,
edge-case and does-it-actually-work territories of this story are UNREVIEWED. Read the short list
below as "what one layer found", not as "what is wrong with 3.3".

- **Haul jobs are no longer bounded by `MAX_DESIGNATIONS`** [`crates/simd/src/main.rs:317-329`] —
  LAYER: acceptance-auditor. Before 3.3, `save.jobs.len() > MAX_DESIGNATIONS` rejected the whole
  save; now that cap applies to tile jobs only, and haul jobs are bounded by `save.items.len()`,
  which nothing caps but `MAX_SAVE_BYTES` (64 MB). WHY IT IS NOT A DEFECT: capping haul jobs at 4096
  would refuse a *legitimate* late-game save — designations are consumed as tiles are dug while
  stones accumulate, so a long game can hold far more than 4096 stones, and AC12 deliberately
  specifies the item-count bound instead. The exposure is a local, operator-written file bounded at
  64 MB. **Revisit if** a save ever carries enough items that `from_save` or a tick becomes slow
  enough to measure, or if saves ever arrive from anywhere but the local operator.

- **The entity draw loop skips any non-`Dwarf` `EntityKind`** [`crates/tui/src/view.rs:193`] —
  LAYER: acceptance-auditor. Story 3.3 closed the older "counting and drawing use different filters"
  item by making BOTH filter on `EntityKind::Dwarf`, which is strictly better today (they can no
  longer disagree) but converts the old defect into a narrower one: a second entity kind would not be
  drawn at all rather than being drawn with the wrong glyph. Behaviour-identical while
  `protocol::EntityKind` has one variant, and the code carries a `// NOTE:` saying a second kind must
  decide its own contention rule. **Revisit when** a second `EntityKind` reaches the wire — that
  story owns the rule, and this is where it lands.

- **`glyph_positions` records only the first occurrence of a glyph per line**
  [`crates/tui/tests/client.rs:2154`] — LAYER: acceptance-auditor. Sound for the one-stone haul stub,
  where "all early `*` are at the source column" is a valid derivation. It would not notice a SECOND
  stone glyph appearing on the same row. **Revisit when** a capture stub grows a second item, which
  is likely the first multi-stone TUI story.

- **Placing a stockpile on solid rock is a silent no-op** [`crates/sim-core/src/lib.rs`,
  `SimCommand::PlaceStockpile`] — LAYER: feature-auditor (story 3.3 review). The command filters to
  standable tiles, so a rect entirely in rock adds zero zones and the player is told nothing — no
  mark, no message, no refusal. The auditor hit this for real: aiming one z level low produced a
  capture with zero of every glyph and exit 0, which is indistinguishable from "hauling is broken".
  Pre-existing since 3.1 (the same is true of a dig rect that hits nothing diggable), so not caused
  by this story. **Revisit when** a story touches command feedback or the status line — the cheap fix
  is telling the player how many tiles a command actually took.

- **The client's opening camera z is nondeterministic** [`crates/tui/src/view.rs`, `initial`] —
  LAYER: feature-auditor (story 3.3 review). `initial` takes z from `snapshot.entities.first()`, i.e.
  dwarf 0, who wanders and settles wherever work took it. Two clients connecting to the same daemon
  minutes apart therefore open on different levels, which makes every `--key` capture recipe in this
  project fragile: the same key sequence aims at a different z depending on when it is run. This cost
  a false "the feature does not work" reading during 3.3's review. **Revisit when** the next TUI story
  touches the camera; the options are an explicit `--z` flag, opening on the level with the most
  standable ground, or documenting that every scripted capture must range-check its own glyphs first.

- **Two dig designations never completed across ~38k ticks** — LAYER: feature-auditor (story 3.3
  review). Observed in a live run: `×` flat at 2 marks while everything else progressed. Most likely
  the known unreachable-target class (a tile with no standable work position), which 3.2 ruled is
  retried forever rather than dropped, but it was not chased. **Revisit if** a player ever reports
  designations that never clear, or alongside the channel-orphan item above.

- **The glyph client may be near its visual ceiling** — LAYER: Wolf, at 3.3's AC17 sign-off
  (2026-08-07). Verdict on the finished haul loop: "looks ok for 2d tui game atm ... not sure how much
  more visually pleased it could be without designing own font or something". This is not a defect and
  not a request: it is the standing judgement that further investment in the 2D presentation has low
  expected return, and that FR23's icy-grim-identity-in-motion question is better answered by the depth
  view than by more work on glyphs and truecolor. **Revisit at epic 4, specifically story 4.1b** (`4-1b-dwarves-in-depth` — the 4.1 split of 2026-08-08 put the FR23 verdict on the creatures story, not the renderer)
  — and note the implication for scope: a custom font or a tileset is the alternative lever, and
  neither is in milestone 1. Treat "make the TUI prettier" stories as needing an explicit case against
  this entry.

## Deferred from: code review of 4-1a-behold-the-fortress-in-depth (2026-08-08)

- **Camera inside solid rock renders a featureless full-screen `█` with no cue why**
  [crates/tui/src/raycast.rs:174-207]. The camera's own voxel is tested on iteration 0, so a solid
  camera tile returns `distance 0.0` → band 0 for every ray: `tui --z 5 --frames 2 --key v` gives a
  map-area histogram of exactly `[('█', 1760)]`. The spec pre-declares this "a legitimate picture,
  not a bug", and for captures AC12's two-distinct-bands check is the guard. But it is reachable in
  ordinary play — `<`, `<`, `v` — and a player then sees a screen indistinguishable from a crashed
  renderer, with neither the status line nor the hint saying "you are inside rock". Raised by the
  Feature Auditor.
- **`shade()` has no direct test and `percent = 0` is never exercised anywhere**
  [crates/tui/src/palette.rs:151-159]. The new public cross-module home of the "scale colour toward
  black" formula is reached only via `dim()` (which early-returns at depth 0, never touching
  `shade`) and via `raycast.rs`'s tests at 100/80/62/46. Its lower boundary and identity case are
  unverified in the module that owns it. Raised by the Edge Case Hunter.
- **Partial-clamp corner untested** [crates/tui/src/view.rs:563-567]. The only new edge-clamp test
  sits the camera in a full corner of a 4×4 world so both axes clamp together. No test covers one
  axis already at its bound while the other moves freely (e.g. `dims 4×4`, heading `se`, start
  `(3,1)` → expect `(3,2)`). Correct by construction — the two `.clamp()` calls are independent —
  so this is a coverage gap, not a defect. Raised by the Edge Case Hunter.
- **`simd` serve suite flakes under heavy concurrent load** [crates/simd/tests/serve.rs:148].
  `two_haul_jobs_on_one_item_save_is_logged_and_the_daemon_keeps_ticking` failed twice with
  `snapshot line must match the protocol: Error("EOF while parsing a value")` while four review
  layers were running cargo concurrently, and passed 3/3 in isolation (0.71s each); the full gate is
  GREEN on a quiet machine. EOF means the daemon *closed the socket*, not that it was slow, so this
  is not a plain timeout. Pre-existing and entirely outside 4.1a's diff (which never leaves
  `crates/tui`), but it matters to process: the review workflow mandates four concurrent
  cargo-running layers, so this will keep firing and will keep looking like a story defect.
  Not port contention — `Daemon::spawn` passes `0` and lets the OS assign. Raised by the orchestrator.
- **SPEC premise false: "this devpod sets `NO_COLOR=1`"** (story 4.1a Key decisions). `NO_COLOR` is
  unset here (`COLORTERM=truecolor`); both hunters independently found this and had to force it to
  test the glyph-ramp claim. The design conclusion survives — the ramp does carry geometry with
  colour stripped, verified live both ways — but two things follow: the reasoning rests on a false
  premise, and the stale `NO_COLOR` warning at `main.rs:382-388` never fires in this devpod, which is
  why nobody noticed it had gone out of date. Raised by the Edge Case and Feature Auditors.
