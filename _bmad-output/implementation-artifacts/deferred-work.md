# Deferred Work

Items surfaced by review that were real but not actionable at the time. Each entry
names where it came from and what should trigger revisiting it.

## Deferred from: code review of 5-2-one-mirror-two-clients (2026-08-11)

- **Duplicate entity/item ids silently collapse to last-one-wins** (`client-core/src/lib.rs:75-78`,
  `:101`, `:151-162`). Proven with a probe crate: a snapshot carrying two `Entity{id:5}` yields one
  entity, the first gone with no diagnostic; same for items, and for duplicates inside one delta.
  The mirror validates tile count and nothing else, so an id-reuse bug in `simd` would make a dwarf
  vanish from every client silently. **Why deferred, and this is the interesting part:** the obvious
  fix — a `DuplicateId` variant — is forbidden by AC9, which says the tile-count error is "the only
  error the crate defines". Not reachable today: sim ids are unique by construction. **Revisit
  trigger:** any story that makes the daemon allocate or recycle entity ids, or the first time a
  second producer feeds the mirror. Raised by the Blind Hunter.
- **`rect_is_valid` is a structural gate, not a dims gate** (`simd/src/main.rs:714-716`). A rect
  spanning the full `i32` range passes validation: a live probe sent
  `min:[-2147483648,0,3] max:[2147483647,0,3]` and designated an entire 128-tile map row in one
  command — no log, no rejection. Harmless today because `sim-core`'s unchanged clamp bounds the
  work, and this is AC11 exactly as written (it names `min.z != max.z` and `min > max`, nothing
  about dims). Recorded because "the daemon now validates rects" is easy to misread as bounds
  safety that in fact still lives entirely in `sim-core`, outside this story's diff. **Revisit
  trigger:** any story that removes or weakens `World::apply_command`'s clipping. Raised by the
  Edge Case Hunter.
- **Ascending `items()` order has no test that would fail if it flipped**
  (`client-core/tests/mirror.rs:82-85`). The only ordering assertion over items is a single-element
  list, so adding `.rev()` to `items()` would be killed by nothing — unlike the entity counterpart,
  which the mutation table covers. AC4 names items explicitly. Structurally safe today because
  `BTreeMap`. One-line fixture change plus a mutation entry when next in the file. Raised by the
  Acceptance Auditor.
- **A NOTE documenting a still-live limitation was deleted** (`tui/src/main.rs:255-258`). The removed
  comment said dims are assumed never to change between snapshots. `Mirror::apply_snapshot` does now
  replace dims, but `ViewState.camera` and `.z` are still not re-clamped against the new ones — the
  limitation survived, only its documentation went. Behaviour is identical to `main` and the render
  path bounds-checks everything, so a shrunken world renders blank rather than panicking.
  Comment-only regression. Raised by the Acceptance Auditor.
- **Pre-existing `simd` test flakes under parallel execution**
  (`serve.rs::more_haul_jobs_than_items_save_is_logged_and_the_daemon_keeps_ticking`). Failed with
  "daemon stderr closed unexpectedly" at default parallelism, passed in isolation. No rect
  involvement — unrelated to this diff. **Two candidate causes, and they are not distinguishable
  from here:** the project's known target-lock/resource contention trap, or contamination from a
  sibling review layer that ran `pkill -x simd` (self-disclosed by the Feature Auditor, and it
  signals by process name, so it could have killed another layer's daemon). Recorded so a future
  green-gate claim showing red here is not mistaken for a 5.2 regression. **Revisit trigger:** if it
  reproduces outside a concurrent review. Raised by the Edge Case Hunter.
- **AC17's "equals a hand-written expected mirror" is unmeetable as worded**
  (`client-core/tests/mirror.rs:45-143`). `Mirror` derives `PartialEq` but all fields are private, so
  an integration test cannot construct an expected value; the AC is met by 34 field-by-field accessor
  assertions instead. The evidence is real — the ordering mutant died via this file. A literal
  `Mirror == Mirror` would also compare the transient `changes` field, which is probably not the
  intent. AC phrasing to correct at the next story, not a code defect. Raised by the Feature Auditor.

## Deferred from: story 5.2

- `client-core::Mirror::previous_entity()` and `changes()` have no live caller yet. Their
  decision surface is headlessly tested; Story 5.3 is the wiring story for gui reconciliation
  and AD-15 interpolation.
- **`changes()` IS DELTA-ONLY, AND 5.3 MUST READ THIS BEFORE WRITING A RECONCILER.** Wolf's ruling
  at 5.2's review (2026-08-11): keep the current shape, document the contract. After
  `apply_snapshot`, `changes()` reports **empty** — no spawned, no despawned, no changed, no dirty
  tiles — even for entities the snapshot carries (pinned by `client-core/tests/mirror.rs:139-142`).
  A gui reconciler driven naively by `changes()` will therefore create **nothing** for a world it
  just received in full: an empty Bevy window beside a correctly-rendering TUI, on every reconnect
  and every `Load`. **A snapshot means "rebuild everything from `entities()`/`items()`/`tile()`",
  and the only signal that one arrived is that `previous_entity()` returns `None` for every id** —
  AD-15's "absence is the signal", which costs a full scan to detect. Second half of the same
  contract: an out-of-bounds dirty tile in a delta is skipped *and* omitted from `changes().tiles`
  (`client-core/src/lib.rs:68-73`), so a client repainting only `changes().tiles` diverges silently
  from one repainting everything. The skip is deliberate and specified (Task 2, "skipped, not an
  error"); the silent omission from `changes()` is what 5.3 must not build on. Rejected at review:
  populating `spawned` with every id on a snapshot (asymmetric with tiles), and populating
  `tiles` with every position (a ~6MB `Vec<[i32;3]>` per snapshot on the shipped 128×128×32 world,
  for a consumer that does not exist yet). Raised by the Feature Auditor and the Acceptance Auditor.

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
  **CLOSED in Story 5.1:** fixed emitters draw in their own pass above items and below dwarves;
  only dwarves participate in crowd/carrier contention and the status count.

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

## Deferred from: code review of story-5.1 (2026-08-10)

- **`bridge::entity_kind` panics on `LightKind::Lantern`, and the only guard lives in another crate**
  [crates/simd/src/bridge.rs:144-157]. `entity_kind` runs on the daemon's main thread inside the
  tick loop (`simd/src/main.rs:142,189,209`) with no `catch_unwind` anywhere in `simd`, so the panic
  would take down the whole process and disconnect every client, not fail one request. Today it is
  genuinely unreachable and correctly guarded — but the guard is `simd::load_world_from`
  (`main.rs:501-504`), which only covers the save-load path, while `World::emitters()` and
  `World::from_save` in `sim-core` accept `Lantern` without complaint. The invariant is enforced by
  convention across a crate boundary, not by a type. Note `light_kind()` maps `Lantern` correctly;
  only the `kind` field panics, which is easy to miss on inspection. NOT patched because there is no
  correct wire mapping to give it: `protocol` has no `EntityKind::Lantern`, and adding one would
  breach AC12's "exactly this and nothing else". **Revisit trigger: FR29 (moving lights), which is
  first on the M2 cut list and is the story that makes `Lantern` live.** Raised independently by the
  Edge Case Hunter and the Acceptance Auditor.
- **The emitter test oracle copies production's `unreachable!()` arms instead of decoding
  independently** [crates/simd/src/bridge.rs:399-409]. `snapshot_and_delta_carry_the_same_emitters`
  builds its "expected" values with `LightKind::Lantern => unreachable!(...)` twice — textually the
  same arms as production `entity_kind`/`light_kind`. This same file documents the opposite
  discipline for another bridge: `expected_job_state` (`bridge.rs:232-239`) hand-decodes wire
  strings "rather than repeating the production match — a second copy of the same arms would pass
  even when the author got a mapping wrong." For the `Lantern` case the test can therefore never
  produce a useful assertion diff; it would crash with production's own message. Raised by the Edge
  Case Hunter.
- **Two emitters on one cell render with silent last-write-wins** [crates/tui/src/view.rs:218-227].
  The emitter draw pass has no per-cell counting comparable to `item_counts` (`view.rs:210-216`) or
  `dwarf_counts` (`view.rs:231-238`); a second emitter on the same tile overwrites the first with no
  visual signal that anything is hidden. Unreachable from worldgen today — `camp_emitters` places
  all five at distinct positions — so this is reasoned from the code, not demonstrated. Revisit if
  emitter placement ever becomes data-driven or a hand-crafted save can reach the renderer. Raised
  by the Edge Case Hunter.
- **`opening_z` counts tree foliage as standable ground** [crates/tui/src/view.rs:111-113]. The
  condition matches `Tile::Solid(_)` uniformly, so a tree canopy is indistinguishable from a rock
  floor for the opening-level heuristic. Measured against the live daemon: at the winning level
  (z 19) 684 of 1638 tiles counted as standable are canopy, not ground. Excluding tree materials
  still picks z 19 on the shipped seed (954 vs 908 next-best), so there is no behaviour change
  today. **Coupled to the decision item in story 5.1's Review Findings** — if `opening_z` is
  reworked to find the camp, this belongs in the same edit. Raised by the Edge Case Hunter.
- **The camp/tree code carries an undocumented `dims.x, dims.y >= 2*CAMP_RADIUS+1` precondition**
  [crates/sim-core/src/worldgen.rs:144,177; crates/sim-core/src/lib.rs:1516-1517]. `dims.y - radius`
  on a map smaller than 7 wraps in release (overflow-checks off) and the subsequent `heights[...]`
  index panics with `index out of bounds: the len is 4 but the index is 9`, not the intended
  `.expect("worldgen requires one 7x7 flat camp clearing")` at `worldgen.rs:158`; a debug build
  panics earlier with "attempt to subtract with overflow". Reproduced with a scratch harness calling
  `World::generate(1, Dims { x: 2, y: 2, z: 10 })`. The old `spawn_dwarves` walked the full grid with
  no radius subtraction and tolerated arbitrarily small maps, so this is a precondition **this story
  introduced**. Every call site in the repo uses `Dims::DEFAULT`, so it is dead code today. **Revisit
  trigger: the first story that generates a world at a non-default `Dims`** — a small test map or a
  scenario-test helper — where the panic message would give no hint the real cause is camp sizing.
  Raised by the Blind Hunter.
- **AC16's glyph-distinctness assertion proves less than the AC claims**
  [crates/tui/src/palette.rs:323-345]. The four new glyphs were added to `existing_glyphs`, and the
  only new assertion checks the four are distinct **from each other**. Nothing asserts they differ
  from `█ ▓ ▒ ░ ▲ ␠ ☺`. Setting `Tile::Solid(Material::TreeTrunk)`'s glyph to `▲` would leave
  `every_look_is_pinned` green while the distinctness claim silently became false. The claim is true
  today — every palette glyph was enumerated and confirmed — but the test named as its guard does not
  enforce it. Raised by the Acceptance Auditor.
- **AC5's "spawn position is standable" is only half-asserted**
  [crates/sim-core/tests/worldgen.rs:113]. `all_dwarves_spawn_inside_the_camp_with_room_to_move`
  defines a correct `is_standable` helper (`worldgen.rs:21-30`) and uses it for the *neighbour*
  clause, but for the spawn tile itself asserts only `world.tile(pos) == Some(Tile::Empty)` — the
  "solid or ramp below" half is skipped, so a spawn floating one level above the ground would pass.
  The production code does call `terrain.is_standable` (`lib.rs:1519`) and a 3000-seed sweep found no
  violation, so the behaviour is right; the assertion is just weaker than the AC. Raised by the
  Acceptance Auditor.
- **AC18's "at generation" half is not asserted for emitters**
  [crates/sim-core/tests/worldgen.rs:33-39]. `same_seed_produces_identical_worlds` still compares
  `tiles()` and `dwarves()` only. Emitters are compared solely inside
  `same_seed_and_commands_remain_deterministic`'s loop, i.e. from tick 1 onward. Emitters never move,
  so a generation-time divergence would surface one tick later — the gap is narrow, but the AC names
  both moments and only one is covered. Raised by the Acceptance Auditor.
- **Clustering the dwarves makes the crowd glyph the common case** [crates/tui/src/view.rs:239-249].
  Measured on 300 live frames: 115 (38%) had two dwarves on one cell, so a capture routinely shows
  `☺=3` where five dwarves exist. The status line correctly still reads `dwarves 5`, so AC17 holds
  and this is per spec. The rule is pre-existing and deliberate (3.2 AC14, Wolf's ruling that the fix
  is a crowd glyph rather than tile reservation), but it was near-unreachable when dwarves spawned
  ~104 tiles apart and is now routine in a 49-cell clearing holding 5 dwarves and 5 emitters.
  Recorded so a later reader does not read a low `☺` count as a missing dwarf. Raised by the Feature
  Auditor.
- **The default TUI view opens ten levels above the camp** [crates/tui/src/view.rs:97-125].
  `tui <port>` with no `--z` opens at z 19 while the camp sits at z 9 on the shipped seed. Measured
  live: `†=0 ♨=0 ☺=0` on screen while the status line simultaneously reads `dwarves 5` — a user who
  runs the client the obvious way sees a forest and none of story 5.1's headline content.
  `opening_z` picks the level with the most standable ground; the behaviour is pre-existing and
  deliberate (it carries a `// NOTE:` that the answer is a centre-on-dwarf key, never a
  nondeterministic opening), but 5.1 makes it categorically worse — dwarves used to be scattered
  over z 15–20 near the auto-picked level, and are now all concentrated ten levels below it. **No AC
  covers the default opening level, so nothing is violated**; this is the 4.1a class (meetable,
  implemented, not what the user wanted to see) and only a live run finds it. **Wolf's ruling at
  5.1's review (2026-08-10): leave it.** The TUI is a 2D instrument, not the product; 5.3's Bevy
  window is the real viewer, and the instrument recipe pins `--z 9`. Rejected: reworking `opening_z`
  to prefer the level where the dwarves are (`Snapshot.entities` already carries them, so it is a
  `tui`-only change with no wire change — this is the cheap fix if the ruling is ever revisited);
  putting the camp z on the wire (breaches AC12). **Revisit trigger: 5.4, the wow gate** — if that
  sign-off is ever taken through the TUI rather than the Bevy client, the default invocation shows
  none of what 5.4 is judging. Raised by the Feature Auditor and the Edge Case Hunter.

## Deferred from: story 5.3

- `Mirror::previous_entity()` remains without a live caller. AD-15 interpolation is deliberately
  deferred to story 6.1; reconciliation lands in 5.3, but blending does not.

## Deferred from: code review of 5-3-a-window-onto-the-valley (2026-08-14)

- Camera focus is hardcoded `[64, 64, 9]` (`crates/gui/src/ingest.rs:167`), ignoring the wire's
  `dims`; any world other than the shipped 128×128×32 spawns the camera looking outside the
  terrain with no correcting path. AC21's `zoom_never_moves_the_focus` asserts a constant that
  no code can change. Cheap fix when it fires: read `MirrorResource` in `setup_camera` (the
  mirror exists before the App is built). `[edge+auditor/LOW]`
- CLI accepts multiple positional port arguments and the last silently wins
  (`crates/gui/src/ingest.rs:150`); `tui` bails on extras with `port_was_set`. Live-verified:
  `gui 9999 7522` connects to 7522 with no warning. `[edge/LOW]`
- `--frames N` without `--capture` is accepted and silently discarded
  (`crates/gui/src/ingest.rs:132-164`); validation is one-directional. `[edge/LOW]`
- The `--capture` path spawns a `Screenshot` entity during `Update`
  (`crates/gui/src/capture.rs`) after `classify_client_local` has run at `PostStartup`, so it
  carries neither partition marker; the AC12 structural test only inspects already-marked
  entities and cannot see a third class. `[auditor/LOW]`
- `toggle_overlay` stays registered during `--capture` runs (`crates/gui/src/ingest.rs:105`);
  an F3 keypress during the N capture frames re-enables the overlay the startup forcing turned
  off, defeating AC23 on an interactive display. `[blind+auditor/LOW]`
- `ProjectedItem` (`crates/gui/src/project.rs:33`, inserted at `:155`) is never queried by any
  system or test — dead code against the YAGNI policy. `[blind/LOW]`
- Dead condition in the stale-entity despawn loop (`crates/gui/src/project.rs:137`):
  `terrain.get(bevy_entity)` is structurally always `Err` because the query carries
  `Without<TerrainTile>`; the clause reads as if it does real work. `[blind/LOW]`
- `scripts/gate.sh` label columns diverged: `run()` uses `printf '  %-28s'` (line 47) while the
  probe loop uses `%-40s`, so outputs misalign; the header comment still describes "first four
  checks" and a single `tui` probe. Task 1 asked for the widening once. `[auditor/LOW]`
- `ingest_messages` and `reconcile_projection` are registered without `.chain()`
  (`crates/gui/src/ingest.rs:99-107`); both take `ResMut<ProjectionWork>` so Bevy serialises
  them, but the order is incidental, and it is load-bearing for AC16's same-frame dirty-tile
  path. Related: a small `--frames` value screenshots before the first reconcile's queued
  spawns apply (recipe uses 60, floor is >0). `[feature/LOW]`

## Deferred from: code review of 5-4-the-cold-boot (2026-08-15)

- Entity/item id collision in `reconcile`'s `wanted` map silently erases the entity's kind,
  light, and appearance (`crates/gui/src/project.rs:219`) — probed: a campfire sharing an
  item's id renders as a bare stone cube with no light. Unreachable today because
  `sim-core`'s single `IdAllocator` never collides entity and item ids, but nothing in
  `gui`/`client-core`/`protocol` documents or asserts that invariant, and 5.4 widened the
  blast radius from position-only to full identity. `[edge/LOW]`
- ~~Point lights cast no shadows~~ **CLOSED 2026-08-28 by Story 9.1:** the campfire's projected
  `PointLight` now sets `shadow_maps_enabled: true`, pinned through a later reconciliation pass.
  Torches and lanterns deliberately remain unshadowed: each extra point light costs six cube-map
  faces, and the measured defect is the campfire. Vehicle performance remains Task 6 evidence.
- Degenerate captures produce misleading diagnostics (`crates/gui/src/capture.rs:76`): an
  empty pixel buffer asserts "capture is black", a 1-pixel capture always asserts "capture
  is uniform". Unreachable through a real primary-window screenshot. `[edge/LOW]`
- The live `App` built by `run()` (`crates/gui/src/ingest.rs:81`) has no test of any kind —
  every headless test assembles its own `MinimalPlugins` app, so a system dropped from the
  registration tuples, a mis-ordered resource insert, or changed fog/framing constants would
  pass the whole suite. Needs a test-architecture decision (what of `run()` is assertable
  without `DefaultPlugins`). `[feature/MED]`
- NFR6 headroom note for Task 8: 5.4 adds 16,992 cap-slab entities (+32% over the 53,365
  terrain cubes) plus a 2048² shadow cascade over ~70k meshes; the 146 fps baseline predates
  all of it. If AC14's reading fails, this is the measured cause to check first.
  `[feature+auditor/LOW]`
- Tree presentation: the wire's trees are foliage-skirted cube stacks, while 5.4's approved
  sign-off artifact drew "snow-laden spruce sprites instead of per-tile boxes" with visible
  trunks (`_bmad-output/implementation-artifacts/5-4-signoff/artifact_render.py:7,234`).
  Wolf ruled 2026-08-15: accept cube trees for 5.4, AC19 judged on light/sky/snow/framing;
  spruce-like presentation (expose occluded trunks in the draw set + taper foliage scale —
  changes the exposed-predicate and the 53,365 oracle) is a candidate later story. RETRO
  NOTE: Task 0 artifact scripts must not substitute geometry the renderer is not tasked to
  produce. `[wolf-live+orchestrator/HIGH-deferred]`

## Deferred from: code review of 6-1-the-world-moves (2026-08-18)

Four layers, none a coverage hole. R1's territory split has **no mapping for the M2 crates** — it
names `sim-core` / `simd`+`tui`+`protocol`, none of which a gui-only diff touches — so it was
adapted for this story (Blind Hunter → `blend.rs`+`appearance.rs`, Edge Case Hunter → `project.rs`,
`ingest.rs`, `capture.rs`, `tests/`). **Give R1 a real M2 mapping at the Epic 5/6 retro.**

- NaN passes through `f32::clamp`, so `TickClock::factor` and `blended_translation`'s "clamped to
  [0,1]" guarantee has a hole: a NaN `elapsed` is *stored* at `crates/gui/src/blend.rs:35` and does
  not self-heal until the next successful `observe_tick`, and `Vec3::lerp` with a NaN `t` yields a
  garbage screen position. Unreachable — Bevy never emits a NaN `delta_secs()` — and guarding it is
  error handling for an impossible scenario, which ground rule 1 makes a defect in itself.
  `crates/gui/src/blend.rs:47`, `:55`. `[blind/LOW]`
- `flicker_scale`'s per-id phase collides *exactly* for ids ≥ 2^24: `id as f32` stops distinguishing
  consecutive integers, so two emitters of the same kind pulse in perfect sync for all time —
  an unconditional violation of AC10's distinctness clause, not a near-miss. Needs ~16.7M
  `IdAllocator` allocations (entities and dug-stone items share one monotonic `u32`), so it is out
  of reach this milestone. `crates/gui/src/appearance.rs:84`. `[blind/LOW]`
- The flicker aliases from ~3.2 days of client uptime and freezes entirely at ~11.6 days: at 60 fps
  the f32 ULP of `Time::elapsed_secs()` exceeds one frame delta once elapsed > 2^24/60 ≈ 279,620 s,
  after which successive frames feed `flicker_scale` bit-identical seconds. Measured, not argued.
  `crates/gui/src/appearance.rs:85-88`. `[blind/LOW]`
- `TickClock::observe_tick(0)` is a silent no-op — the guard is `tick > self.last_tick` and
  `Default` sets `last_tick: 0`. Not reachable today (a snapshot's `reset` always precedes the first
  delta, and world ticks start at 0), but a future protocol delivering a `Delta` before a `Snapshot`
  would swallow tick 0 rather than be defended against. `crates/gui/src/blend.rs:33`. `[blind/LOW]`
- `TickClock::reset` zeroes `elapsed` and `last_tick` but leaves `interval` at whatever was last
  measured, so the first frames after a reconnect blend against a stale cadence. Never extrapolates
  (worst case it snaps early), so recorded rather than fixed. `crates/gui/src/blend.rs:41-44`.
  `[blind/LOW]`
- `DigChip` entities have no lifetime cap: a tile's chips are removed only when that exact position
  is dirtied again or a full snapshot rebuild occurs, so a long session digging many distinct,
  never-revisited tiles accumulates chip entities without bound.
  `crates/gui/src/project.rs:294-318`. `[edge/LOW]`
- A tile emptied by a delta that lands in the same frame as, and after, a snapshot permanently loses
  its debris chips: `reconcile_projection` takes `rebuild = true` and `mem::take`s `dirty_tiles`,
  while the whole dig-chip block sits behind `if !rebuild_terrain`. No test covers snapshot-then-
  delta in one frame. Cosmetic — chips are `ClientLocal` eye candy with no sim meaning.
  `crates/gui/src/ingest.rs:398-401`, `crates/gui/src/project.rs:294`. `[edge/LOW]`
- The dig-chip test asserts count and markers only: never that the chips are *at* the tile, never
  AC8's "identical for the same position on every run" determinism, and neither of Task 4's two
  negative cases (a tile changing to something solid spawns none; the same position twice does not
  double). The de-dup loop is unguarded by any assertion. Separately `chip_offsets()` is a **fixed**
  array, not "deterministic offsets derived from the tile position" as Task 4 specifies, so every
  dug tile gets an identical four-chip arrangement — satisfies AC8's wording, deviates from the task
  text, and nothing tests either reading. `crates/gui/tests/headless.rs:684-722`,
  `crates/gui/src/project.rs:382-389`. `[auditor/LOW]`
- AC3's mutation removes the clamp in `blended_translation` (`crates/gui/src/blend.rs:55`), but that
  clamp is dead defence — the only production caller passes an already-clamped `clock.factor()`. The
  load-bearing clamp in `TickClock::factor` (`:47`) has a test but no mutation, so the sabotage table
  proves the belt and not the braces. `mutations/6-1-the-world-moves.sh:3-9`. `[auditor/LOW]`
- Making `--expect-work` a no-op leaves 57/57 green. The flag is reachable and its rejection without
  `--capture` is real, but neither is defended by a test; the sibling `--capture`-requires-`--frames`
  guard *is* tested. `crates/gui/src/ingest.rs:189`, `:201`. `[auditor+feature/LOW]`
- `MIN_TICK_INTERVAL` is never exercised: the only clock test walks the high end via `advance(10.0)`.
  Zeroing the floor would make `factor()` divide by ~0, kept benign only by the outer clamp — half
  of Task 1's clamp subtask is unproved and unmutated. `crates/gui/src/blend.rs:35`. `[auditor/LOW]`
- Reconcile no longer refreshes an existing entity's scale (a consequence of the AC5 sole-writer
  edit, not an AC violation): scale is set at spawn only and only when `ProjectionAssets` exists, so
  an entity spawned in a frame without assets keeps scale 1.0 permanently and a wire kind change no
  longer restyles it. Not live under `run()`, where assets exist from the first `Update`. No test
  covers either path. `crates/gui/src/project.rs:335-350`. `[auditor/LOW]`
- Speculative public surface: `MotionStats`'s four counter fields and both `TickClock` interval
  constants are `pub` with no reader outside their own module. `crates/gui/src/capture.rs:31-34`,
  `crates/gui/src/blend.rs:7-8`. `[auditor/LOW]`
- Deleting `clock.reset(...)` on snapshot leaves 57/57 green — harmless today only because
  `Mirror::apply_snapshot` clears `previous_entities` so the snap still happens, i.e. the reset is
  belt over an existing brace and nothing would catch its loss.
  `crates/gui/src/ingest.rs:344`. `[feature/LOW]`
- Three `// NOTE:`s the story's own tasks asked for are absent: the per-frame id map and the
  two-deltas-in-one-frame limitation (Task 1), and point-light-not-emitter-material (Task 3). Only
  the AC14 global-counts NOTE exists. `[feature/LOW]`
- AC18's "RED output pasted into the Dev Agent Record" is half-satisfied: the 8-row KILLED table and
  the before/after suite counts are pasted, but no RED assertion text for the four mutations added by
  the continuation run. The table is what `mutate.sh` prints, so this is a format gap rather than a
  fabrication. `[auditor/LOW]`
- **This register was not updated by story 6.1, so entries the story closed still read as open.**
  Closed by 6.1: 5.3's "`Mirror::previous_entity()` remains without a live caller" — the blend is now
  that caller; and 5.3's review's "`ingest_messages` and `reconcile_projection` are registered
  without `.chain()` … the order is incidental" — the order is now `.chain()`ed and load-bearing.
  Partly closed: 5.4's "the live `App` built by `run()` has no test of any kind" `[feature/MED]` —
  AC6's shared `projection_systems` now gives the live tuple app-level coverage, so the entry needs
  **narrowing to the parts still untested** (fog/framing constants, resource-insert order) rather
  than deletion. `[orchestrator/LOW]`

## Deferred from: code review of 6-2-lanterns-in-the-dark (2026-08-19)

Four layers, none a coverage hole. Only the LOW tail is deferred here; both HIGHs, all five MEDs and
four LOWs (three of them silent-failure/record traps) went to the story's Review Findings as patch
items. Context that outlives the story: **the wire half of 6.2 is proven on a live daemon, the
rendered half has no evidence of any kind, and the instrument built to supply that evidence has never
executed a single line of its production path.**

- Two of AC11's three assertions are tautological — `assert_eq!(translation, projected_translation(…))`
  compares a value to itself, because the `PointLight` sits on the same entity as the `WorldProjected`
  /`Transform` it is compared against, so only one `Transform` exists. AC11 as written is unfalsifiable
  given the chosen architecture. Left standing because the third assertion (translation strictly
  between the two endpoint x's) genuinely dies if the blend is deleted, so AC11 keeps real coverage.
  AC-text defect, not an implementation one. `[auditor/LOW]`
- AC4's and AC5's lantern assertions cannot fail, and AC5's scenario test does not exist. `dwarves()`
  appends the compile-time `DWARF_LIGHT` to every tuple, so the round-trip comparison can never
  disagree on the light and the determinism comparison has no random input to vary; `scenario.rs`
  gained **no new test function**, only mechanical `(_, _, _, _)` destructuring updates, while AC5 names
  a scenario test. The oracles that do carry weight are the literal `*light == LightKind::Lantern`
  check and the diff proving `SavedDwarf` gained no field. Satisfied-by-construction is the honest,
  YAGNI-correct outcome of the story's own "simplest encoding" decision — **re-word the ACs, do not
  change the code.** 10th instance of the AC-text-defect class. `[auditor+feature/LOW]`
- The mutation table covers both bridge arms, both guards, both reconcile light arms and three
  `LanternStats` assertions, but nothing sabotages AC11's blend, the `DWARF_LIGHT` constant, or the
  save round-trip. AC11 is the story's headline interaction and its one falsifiable assertion is
  unsabotaged. This is the residue once the extraction-block mutation named in the story's second HIGH
  patch item is added. `[auditor/LOW]`
- ~~**Process, not code — the per-layer build isolation has no reaper.**~~ **CLOSED 2026-08-28**,
  and the entry under-estimated it by an order of magnitude. Wolf noticed the symptom from the
  outside — `frostvein/target` at 62 GB with 38.5 GB written in 24 h — and the target tree turned out
  to be the *smaller* half: **`/tmp` held 295 GB, of which 277 GB was stale layer caches**, the oldest
  dated 2026-08-09, none in use, nothing running against any of them. The review scaffolding
  outweighed the project's entire build tree more than fourfold. The 62 GB itself is not a leak:
  Bevy links 1.2–1.5 GB test binaries with full debuginfo and cargo never GCs stale hashes, so each
  gate round that touches `gui` relinks ~6.3 GB — six rounds in 24 h is exactly the 38.5 GB observed.
  **276 GB reclaimed; free space 304 GB → 581 GB, 69 % → 40 % used.** The fix is a command, not a
  reminder: `scripts/reap-build-caches.sh`, which the code-review config now requires the
  orchestrator to run after triage (`--tmp-only --force`), with the reclaimed figure recorded in the
  review. It touches only *directories* under `/tmp`, never the repo, never a file (so
  `/tmp/review-findings.md` and the two stray `.diff`s survived), never a symlink, and it refuses
  anything touched within the hour unless forced.

  **SECOND PASS THE SAME DAY, because the first fix answered the smaller half.** Asked whether it
  would hold for 9.1, the honest answer was no: `mutate.sh` and the gate build into the repo's own
  `target/`, which the reaper is forbidden to touch. Measured there: **49 GB of the 62 was
  `debug/deps`, and 29 GB of THAT was 60 stale hash-copies of our own `gui` binary plus its
  `headless` and `capture` test binaries** at 1.2–1.4 GB each. Bevy's rlibs — the expensive half to
  rebuild — are the *small* half and are never touched. So the script gained a second zone and a
  name that no longer lies. **35.2 GB reclaimed in the second pass** (28.8 GB from `target/`, 6.4 GB
  from `/tmp` directories that matched no name pattern), and the trigger moved from a person to
  `scripts/gate.sh`'s full tier — the gate is what *creates* the garbage.

  **Three defects were caught by testing it before trusting it, and two were serious:**
  (1) the symlink guard tested the *leaf*, so a second-level glob walked through a symlinked `/tmp`
  entry and queued **8.6 GB inside the repository** for deletion — the one thing the header promised
  could never happen; now a `realpath` containment test. (2) `CACHEDIR.TAG` as the proof-of-target-dir
  was a **silent no-op on the only tree that mattered**: cargo writes that marker only when *cargo*
  creates the directory, and `gate.sh` does `mkdir -p target` first, so the whole 62 GB was skipped
  while the script reported success — this project's signature failure shape, caught only because the
  dry run's number looked too small. (3) "Keep the newest N sets" was **wrong on its own** and the
  clock said so: one gate round keeps ~11 `gui` hashes alive at once, so N=2 deleted live artifacts
  and made the next gate cost 28–43 s against 9 s warm. The rule is now an age window (7 days) with a
  floor (newest 4), which makes the steady state a genuine no-op — nothing deleted, nothing relinked,
  ~0.3 s — and is why it sits in the full tier and never in the pre-commit one.

  **Standing lesson, the same one M2-7 taught:** every previous guard here was a procedure, and a
  procedure is exactly what an accumulating cache defeats. **Second lesson, from the second pass:**
  a fix aimed at the half you happened to measure is not a fix — the question "will it work for the
  next story?" is what turned a closed entry back into 35 GB. `[orchestrator/LOW → closed]`

## Deferred from: code review of 7-1-slice-into-the-mountain (2026-08-19)

- **`SliceLevel::rebind` is untested and speculative.** Replacing it with `false` leaves 84/84
  green. It keeps a retained client-local level valid if a later snapshot changes world dimensions,
  which cannot happen while `Dims::DEFAULT` is a constant — an untested branch defending an
  impossible case, against this repo's YAGNI-is-policy rule. `crates/gui/src/slice.rs:44-47`
  `[auditor/LOW]`
- **The capture's slice line is untested, and its format disagrees with the startup oracle's.**
  Deleting the `println!` entirely leaves 84/84 green; only `DrawStats::assert_valid` is covered.
  The two lines carry the same number in two shapes — `slice: z 9 projected 36788 terrain cubes`
  vs `projected 36788 terrain cubes at z 9` — so a recipe grepping for one misses the other.
  `crates/gui/src/capture.rs:395-398` vs `crates/gui/src/project.rs:236-240` `[auditor/LOW]`
- **An out-of-range `--z` clamps silently.** `--z 999` becomes z 31 and `--z -5` becomes z 0 with
  no diagnostic. Not false evidence — the printed line names the level actually used — but an
  operator scripting a capture at a level the world does not have gets a silent success.
  `crates/gui/src/slice.rs:17-21` `[auditor/LOW]`
- **`update_slice_readout` rebuilds the whole `Text` every frame** with no `is_changed()` guard,
  re-triggering text layout and glyph work for the life of the process. One small node, so almost
  certainly invisible against AC14's 60 fps floor. `crates/gui/src/ingest.rs:313-317`
  `[feature/LOW]`
- ~~**6.1's vehicle runbook quotes the pre-slice oracle string exactly.**~~ **CLOSED 2026-08-19**,
  same session — fixed while writing the vehicle runbooks rather than left to rot, since the runbook
  is handed to Wolf. The line now names the `at z 31` suffix and says to match the prefix.
  `_bmad-output/implementation-artifacts/6-1-signoff/task-6-vehicle-runbook.md` `[auditor/LOW]`

## Deferred from: code review of 7-2-read-the-working-zoom (2026-08-21)

- **Duplicate-position designations in one payload silently resolve last-write-wins.** The
  `wanted_designations` `BTreeMap` collect drops the earlier entry with no log or assert. Not
  currently reachable — no sim/wire path permits two designations at one position — so this rests
  on an invariant enforced in `sim-core`. Blind Hunter proved the behaviour by execution: Dig then
  Channel at `[1,1,1]` yields one mark, kind Channel, no crash, no orphan.
  `crates/gui/src/project.rs:439-444` `[blind/LOW]`
- **Marks re-insert `Transform` and `MeshMaterial3d` every reconcile tick regardless of change.**
  Efficiency only; no `Changed<T>`/`Added<T>` filter exists anywhere in `gui` today, so nothing
  observable breaks. But the adjacent `WorldProjected` light path gates its insert on
  `existing.0 != light` (`project.rs:396-406`) and the new mark code does not follow that
  established local pattern, while zones are uncapped and full-resent every tick. NOTE the tension
  with the review's Patch 1: the unconditional insert is *why* a kind change restyles today, so
  gating it and asserting the restyle must land together.
  `crates/gui/src/project.rs:466-480, :505-518` `[blind/LOW]`
- **Zone slabs hang in mid-air once the rock supporting them is dug out.** Not `gui`'s defect — the
  sim keeps the zone when the tile below becomes empty. But the story's own recipe places the
  stockpile inside the dig rect, so after ~60 ticks `[56,64,10]` and `[56,65,10]` have empty below
  them (verified live against a real daemon) and two teal slabs float over the pit. Expect it to be
  the first thing Wolf asks about at the viewing. `crates/sim-core` zone lifetime `[feature/LOW]`
- **The "hollow shell" doc comment is attached to the wrong function.** The diff inserted the two
  new mark getters between the comment and its original target, so the rationale for why
  `terrain_tiles > 0` alone is insufficient now documents `pub fn designations()` instead of
  `pub fn assert_valid()`. Cosmetic. `crates/gui/src/capture.rs:55-68` `[edge/LOW]`

## Deferred from: code review of 8-1-point-at-the-world (2026-08-25)

- **`project_world_point` hardcodes `BOOT_ASPECT_RATIO` 16:9 while the pick derives aspect from the
  live viewport.** `project_render_point` divides by the literal `16.0/9.0` (`camera.rs:30,86`),
  while `Camera::viewport_to_world` uses the real render target. The AC8 round-trip cannot detect
  the mismatch because `PICK_VIEWPORT` is pinned to 1920x1080 — exactly 16:9 — so both sides agree
  by construction. Mostly pre-existing (the constant also drives `atmosphere.rs:222`), but 8.1 is
  what makes it load-bearing: the capture oracle multiplies a 16:9-derived normalized coordinate by
  the *actual* `window.resolution.size()` (`capture.rs:635`), so a non-16:9 vehicle window
  desynchronises oracle from pick. **Check the window is 16:9 before believing a Task 6 mismatch.**
  `crates/gui/src/camera.rs:30`, `crates/gui/src/capture.rs:635` `[edge+feature/MED]`
- **`mirror.tile(world).is_some() &&` is redundant.** `is_visible_at_slice` already requires
  `Some(Tile::Solid(_) | Tile::Ramp(_))` on both of its returning branches, so no path can reach
  `true` with the tile absent. Harmless dead redundancy. `crates/gui/src/pick.rs:100` `[blind/LOW]`
- **AC8's mutual-inverse pin landed in the wrong file.** The story's Project Structure table
  specified extending `transform.rs`'s round-trip pin; `git diff main...HEAD -- crates/gui/src/transform.rs`
  is empty. The property IS genuinely proven — `a_cursor_at_a_visible_tiles_independent_projection_picks_that_tile`
  (`headless.rs:2069`) takes its cursor from the independent forward projection and asserts the pick
  returns the literal tile — so substance is met and only discoverability is lost. Recorded because
  the structure table now points at a file that does not contain the pin. `[acceptance/LOW]`
- **AC13's "the RED output is pasted" is discharged by a reference table.** The round-2 table gives
  `file:line — assertion message` per row rather than pasted RED output; only Task 3's Debug Log
  entry pastes a verbatim block. Every cited location was verified to resolve to the named
  assertion, so the evidence is real and checkable — it is simply not the form the AC asks for.
  `[acceptance/LOW]`
- **AC3 is unmeetable as written, and AC7 silently drops its pitch clause.** AC3 quantifies over
  "any orbit yaw, any pitch, any distance in 4.0..=500.0, and any slice level" — an unbounded
  universal with no finite obligation. AC7, the checkable proxy, enumerates orbit angles, distances
  and slice levels and does **not** mention pitch; Task 3 likewise. So AC3's pitch clause had no
  owner, which is precisely how the unreachable-pitch coverage hole opened. This is the project's
  recurring spec-defect class — "AC unmeetable as written" — at instance five-plus. Belongs to the
  Epic 8 retro, not to a patch. `[acceptance/LOW]`
- **Task 1 contradicts Task 4/D8.** Task 1: "One public entry point; no other module gains screen or
  axis math." Task 4 + D8: the instrument's expected tile "comes from the independent forward
  projection". `expected_pick` (`capture.rs:626-640`) is exactly screen math in another module, in
  production code. The dev followed the later instruction, which was the right call; the story asked
  for both. Spec-defect class. `[acceptance/LOW]`
- **`min_by` tie-break depends on undocumented ECS iteration order.** If two visible tiles ever
  project to exactly equal screen distance from the cursor, `Iterator::min_by` returns first-of-ties
  and the winner is decided by `Query<&TerrainTile>` archetype order, which nothing pins. No test
  constructs the tie (every test cursor sits at zero distance from its target).
  `crates/gui/src/capture.rs:638` `[edge/LOW]`
- **AC6's "cursor outside the window" asserts Bevy's own bounds check, not this story's code.**
  `Window::physical_cursor_position` already returns `None` for any out-of-bounds coordinate, so
  `cursor_position()` is `None` before `pick.rs` is reached. Separately, the pick's own
  `viewport_to_world(...).ok()` error branch has no test at all.
  `crates/gui/src/pick.rs:35` `[feature/LOW]`
- **The highlight trails the pick by one rendered frame, by construction.** `sync_hover_highlight`
  runs after `TransformSystems::Propagate`, so the `Transform` it inserts is not propagated to
  `GlobalTransform` until the next frame's `PostUpdate`, and a newly spawned highlight has default
  `ViewVisibility` on its spawn frame. Not observable at 60 fps, but worth knowing before anyone
  chases a "one tile behind" report from the vehicle.
  `crates/gui/src/project.rs:214-240` `[feature/LOW]`
- ~~**The hover highlight is invisible on every tile with a drawn tile above it.**~~ **CLOSED
  2026-08-26 by commit `8782a0d` "Preview mouse designations on picked faces"** — 8.2 took the
  first of the three candidate fixes below, the hit-face slab: `sync_hover_highlight` now offsets
  along the picked face normal and rotates to it (`project.rs:236-238`), guarded by
  `a_vertical_hit_face_places_the_hover_slab_outside_the_cell_side` and four sabotage rows in
  8.2's table. **This entry was left unstruck for two days and Epic 9's story 9.2 was then written
  from it on 2026-08-28, copying the defect description and its three candidate fixes verbatim —
  fabricating a story's worth of scope for work already shipped.** Only the LOOK question remains
  and it is filed separately under 8.2's vehicle deferrals. Struck 2026-08-28. ORIGINAL ENTRY
  BELOW, kept for the record: DEFERRED TO 8.2 by
  Wolf, 2026-08-25 — reason: waiting on final gfx, and the standing art rule (2026-08-22) is that a
  look change needs a concrete defect and the art pass is owed first. The defect is concrete and
  measured: `sync_hover_highlight` places the slab at `world_to_render(pos) + Y*0.55`, the cube above
  tile *z* spans render y `z+0.5..z+1.5`, so the 0.08-thick slab at `z+0.51..z+0.59` is wholly
  enclosed on every cliff face, corridor wall and shaft side. Measured live at production-legal
  pitches: picks correct, highlight buried, only the top row visible. `dig_mark_level`
  (`project.rs:597-604`) solves this for dig marks by hoisting to the top of the contiguous drawn
  column, but **hoisting is the wrong fix here** — the picked tile is by construction visible, so
  moving its marker up the column would highlight a different tile than the one under the cursor.
  Candidate fixes when the art lands: slab on the hit face (the DDA already knows which axis it
  crossed), an outline box around the cell, or a slightly-inflated cell-sized cube. **8.2 designates
  by pointing at exactly these vertical faces, so this lands before 8.2 ships, not after.**
  `crates/gui/src/project.rs:227,230` `[feature/HIGH]`
- **The hover slab is not visible near the campfire.** Observed by Wolf at the 8.1 review, 2026-08-25.
  Story 9.1 enabled campfire shadows without changing intensity, range, flicker, or emissive; its
  vehicle card requires a re-check at Epic 9's shared sitting to determine whether the campfire
  was the cause. The rendered hover fix, if still needed, remains Story 9.2. `[wolf/MED]`
  **RESTATED 2026-08-29 (9.1 AC15). CAUSATION ANSWERED: YES, and it is structural rather than
  incidental.** The hover slab is `(80,220,210)`, luminance **189.5** — a LIGHT-ON-DARK marker,
  +40.5 against night snow (149.0), which is why it reads everywhere else. It sits **10.5 BELOW
  the 200 near-white threshold** that defines the campfire's blown pool, and that pool covers
  0.65-1.0 % of the frame and saturates toward 255. Inside it the background is BRIGHTER than the
  marker, so the contrast inverts and the slab disappears. Measured on 9.1's controlled pair: the
  pool persists at every threshold in BOTH shadow states, so shadows do not recover it.
  **WHAT 9.2 INHERITS:** a brighter slab cannot fix this — nothing beats a background that reaches
  255 on luminance alone. The marker needs hue/saturation separation or an outline. **STILL OPEN:**
  the rendered judgement, which is 9.2's, and which needs a window (the hover path reads
  `windows.single()`, so it cannot be exercised headlessly).

## Deferred from: code review of 8-2-designate-with-the-mouse (2026-08-26)

- **Paired `Clear` commands can split at the 256-command bound** [crates/gui/src/command.rs:18] —
  `PendingCommands::push` is called in a loop across the bound check, so a Clear designation can
  send `CancelDesignation` while `RemoveStockpile` is dropped, leaving the daemon with an
  inconsistent designate/stockpile pair. Deferred: reaching the bound needs 256 queued
  designations. Fix shape is a single atomic `push_all(Vec<Command>)`.
- **`--at-tick 0` boundary untested** [crates/gui/tests/capture.rs:933] — `target_tick ==
  start_tick` means the first `capture_after_frames` call satisfies the tick test trivially.
  Tested value is 3; 0 is a distinct literal boundary. Deferred: plausible-but-untested only.
- **Test-harness writers set no write timeout, unlike production** [crates/gui/src/command.rs:84] —
  production sets one in `connect_to_daemon`; no test harness does, so a harness bug (e.g. a wrong
  `read_line` count) hangs the test process indefinitely instead of failing fast. Deferred:
  test-only, no production consequence.
- **`SNAPSHOT_READ_TIMEOUT` names two unrelated things** [crates/gui/src/ingest.rs:57] — 8.2 reuses
  it as the command *write* timeout. The 30 s value is right for both; the name is now misleading.
  Deferred: cosmetic. (Note: if the blocking-write decision changes the write timeout, this stops
  being cosmetic and should be split then.)
- ~~**`.codex/` is untracked and not git-ignored** [.gitignore]~~ — **CLOSED 2026-08-27.** Not
  deferred after all: the tree holds `.codex/auth.json`, a live credential, and the `.gitignore`
  secret patterns (`*token*`, `*secret*`, `.env*`) do not match that name. One `git add -A` away
  from committing an auth token is a latent trap, not housekeeping. `.codex/` added to
  `.gitignore`; `git check-ignore -v .codex/auth.json` now resolves to `.gitignore:16`.
- ~~**M2-7's build stamp is missing for the FIFTH time**~~ **CLOSED 2026-08-28**, on its SIXTH
  firing and after being re-noted in five stories without ever being automated. The sixth was the
  one that settled it: the trap left the binary and moved into the *instructions* — 8.2's vehicle
  runbook card named a `gui.exe` one commit stale, so the procedure telling you to check the mtime
  was itself checking against the wrong build. Every previous guard was a procedure, and a
  procedure is what a stale binary defeats. Now `crates/gui/build.rs` stamps the short SHA into the
  binary (`-dirty` when the tree had uncommitted changes, `unknown` when git cannot answer — never
  a fabricated value), `gui::BUILD_SHA` exposes it, and `ingest::run` prints `gui build <sha>` as
  its first line, before the connect can fail, so a session that cannot even reach the daemon still
  learns which binary it is holding. A value compiled into the binary cannot go stale: it is
  whatever the binary actually is. Mutation row added and KILLED. The runbook now compares that
  line against `git rev-parse --short HEAD` instead of doing timestamp arithmetic.
  ORIGINAL ENTRY: `rg 'GIT_SHA|git_sha|vergen' crates/gui/src/` returns nothing and `scripts/`
  holds no build-stamp automation. Recurring because it is re-noted per story and never automated.
  The stale-binary trap it guards has fired five times. Deferred here only because it is a
  process/tooling item, not part of this diff — but it is now the longest-running open item in M2.
- **Blocking socket write inside the Bevy `Update` schedule** [crates/gui/src/command.rs:37] —
  `send_commands` blocks on `write_all` under a 30 s write timeout, in the frame loop; an executed
  reproducer confirms a back-pressured peer stalls it for the full timeout. **ACCEPTED, not
  unnoticed** (Wolf, 2026-08-26): the daemon is localhost, so the stall shape is not real today,
  and a background writer thread is speculative machinery YAGNI forbids. REOPEN TRIGGER: `simd`
  running off-box, or the client pointed at any non-loopback daemon. If that happens this is a
  render-loop freeze, not a latency nit.

## Deferred from: 8.2 vehicle session (2026-08-27)

- **AC13's rendered half — the hit-face hover highlight has never been judged by eye.**
  [crates/gui/src/project.rs] The geometry is proven (the march face and the slab rotation both
  have tests and sabotage rows now), but whether the slab READS on a cliff face, stays distinct
  from the dig/channel/zone marks and clear of the near-white reserved for stars and emitter
  faces is a look question. **DEFERRED by Wolf 2026-08-27**: *"it will get clearer with only real
  gfx... now it's too confusing still to understand what happens."* This is the standing art rule
  ([[art-gates-visual-judgement]]) applied to 8.2. REOPEN TRIGGER: real game art lands.
  `[wolf/MED]`
- **The campfire's vehicle outcome remains open.** Story 9.1 added the discriminating blown-pool
  instrument (boot7 ceiling 0.6651% at threshold 200) and enabled campfire-only shadow maps, while
  preserving every withheld look lever. The required controlled shadows-off/on capture and Wolf's
  judgement have not run outside gingerspice; record those numbers before closing or escalating
  this item. `[feature/MED]`
- **AC19's fps readings and AC18's `tui` cross-check are owed but NOT art-blocked.**
  Both are objective readouts that a short vehicle session closes regardless of how the client
  looks. Listed here so they are not swept up in the art deferral above. `[wolf/HIGH]`
- ~~**`a_mid_haul_save_loads_and_the_daemon_keeps_ticking` flaked once**~~ **CLOSED 2026-08-27 on
  its second sighting**, which is the trigger this entry set for itself. Root cause was not
  contention: `read_snapshot_after_load` budgeted **four lines** for the load snapshot to arrive,
  which is a timing assumption wearing a budget's clothes — the daemon ticks at 10 Hz and keeps
  broadcasting while the load command is in flight, so how many deltas arrive first is set by
  machine load. **The same unit error as M2-15** (`--frames`, a render-rate quantity, feeding
  assertions denominated in ticks). Now bounded by a deadline, with a 1,000-line runaway backstop,
  and the panic reports how many deltas it saw. Five consecutive runs green, then a full gate
  green. ORIGINAL ENTRY: `a_mid_haul_save_loads_and_the_daemon_keeps_ticking` flaked once
  — failed in one full-gate run on 2026-08-27, passed alone immediately after and on the next full
  gate. Pre-existing and untouched by that round, but the round added two daemon-spawning tests to
  the serialized `serve.rs` set, which lengthens the run and may have made an existing timing
  sensitivity more likely to fire. Deferred: one observation, not yet a pattern. **Reopen on the
  second sighting** — a gate that goes red one run in N is a gate nobody trusts, and this project
  relies on the gate being believed. `[feature/MED]`

## Deferred from: 8.2 vehicle session (2026-08-28)

- **Clear cannot reach a mark whose column holds a SECOND standable cell, if the clear drag is
  anchored at a different height than the drag that made it.** [crates/client-core/src/lib.rs
  `standable_in_column`] Each column resolves to the standable cell nearest the *drag anchor's*
  height, so a channel anchored on a cave floor and a clear anchored on the surface above it
  target different cells in the same column. The mark survives, silently, with no way to remove
  it but re-dragging from the original height. MEASURED 2026-08-28 on a synthetic column with
  standable cells at z 2 and z 5: `near_z=2 -> [[0,0,2]]`, `near_z=5 -> [[0,0,5]]`.

  **NOT what the vehicle session hit** — the two leftover marks there were at z 9 and z 10, which
  cannot both be standable in one column, so they were separate columns outside the clear drag's
  footprint. This was found while checking that, and is a distinct latent case.

  **DEFERRED by Wolf 2026-08-28**, asked and answered explicitly rather than filed and forgotten:
  it needs a cave or an overhang to fire, and open ground has one standable cell per column. It is
  the same silent-no-op shape as the two dead modes this story already produced
  ([[silent-sim-filter-trap]]), so it is logged rather than dropped. REOPEN TRIGGER: caves,
  overhangs or multi-level interiors become reachable by a drag. `[feature/MED]`

## Deferred from: code review of 9-1-the-frame-stops-blowing-out (2026-08-28)

Four layers, all live, fresh context, **no coverage holes**. Six deferrals; the review's HIGH and
MED findings were routed to patch, not here.

- **AC5's "before ANY assertion" is only half-guarded** [crates/gui/src/capture.rs:1091-1103].
  The source ordering is correct — the metrics line precedes `capture is black` and `capture is
  uniform` — but `capture_range_report_is_emitted_before_a_blown_pool_panic` uses a frame that is
  neither black nor uniform, so moving `report(...)` back below those two guards leaves every test
  green and sabotage row (e) untouched. Closing it needs a new mutation row paired with a
  black-or-uniform test frame. Raised by the acceptance layer.
- **No sabotage row exercises AC7's second clause** [mutations/9-1-the-frame-stops-blowing-out.sh].
  Row (d) kills the discrimination test through the pool clause only. Nothing proves the
  `median_ground_luminance == 123` assertions at tests/capture.rs:193-200 are load-bearing — and
  that clause is the entire reason AC7 is non-tautological rather than self-referential. The honest
  guard is a row that moves the ground window so the median DOES separate the two frames. Raised by
  the acceptance layer.
- **The blown-pool ceiling carries ~1 ulp of headroom over boot7's own measurement**
  [crates/gui/src/capture.rs:442]. Constant as f32 is `0.0066514760`; boot7's `6130/921600` is
  `0.0066514756`. Difference ≈4.7e-10. This is deliberate — AC6 says "no larger than in boot7.png"
  — but it means the vehicle frame must come in at or below boot7 to the pixel, with no tolerance.
  Stated here so a one-pixel overshoot at the sitting is read as the intended bar and not as noise.
  Raised by the acceptance layer.
- **The cut-level skip line bypasses the injected reporter** [crates/gui/src/capture.rs:1107-1110].
  It prints via `println!` rather than through `report`, so it is invisible to any
  report-capturing test. Not an AC violation; the reporting seam is simply half-injected. Raised by
  the acceptance layer.
- **Panic-hook contamination between concurrent tests** [crates/gui/tests/capture.rs:216-221].
  `std::panic::set_hook`/`take_hook` are process-global while `cargo test` runs this binary across
  up to 32 worker threads, so a sibling test panicking inside the window loses its diagnostic
  stderr. Diagnostics-only — the affected tests assert on the panic payload rather than stderr, and
  the hook is restored before the test's own assertion can fail, so nothing leaks into later tests.
  Three default-threaded runs showed no observed message loss. Raised by the edge layer.
- **Story 9.1 carries ZERO self-gate coverage.** Codex's single `codex review --base main` pass was
  harness-killed before producing any findings, against a cap of three. Disclosed honestly in the
  story's own Dev Agent Record. Recorded here so that this code review is not later mistaken for
  having backfilled that hole — four review layers are not a self-gate, and the two are counted
  separately on this project. Raised by the acceptance layer.

## Deferred from: code review of 9-4-trees-fewer-and-distinct-from-the-ground (2026-08-29)

- **Ice and Snow are excluded from the foliage separation guard**
  [crates/gui/src/appearance.rs:296]. The new loop checks foliage against stone and soil only. Of
  the five other `Material` variants, three are silent implicit branches; margins are currently
  large (Ice 130.1, Snow 159.3) so there is no live risk. Trunk was the one that mattered and is
  being patched now. Raised by the edge layer.
- **`MIN_MARK_SEPARATION` now backs two orthogonal constraints with no isolation between them**
  [crates/gui/src/appearance.rs:530]. It was authored as a mark-vs-palette floor — its own docstring
  says to raise it "if a mark is ever tuned toward the palette" — and this story added a second,
  unrelated terrain-vs-terrain use with only ~8-9.6 units of headroom. Raising the constant for mark
  reasons would break the terrain check as an unrelated side effect. Deferred because it fails
  LOUDLY (a red test), not silently, which is the opposite of the class this project patches on
  sight. Nothing documents that the two purposes now share one knob. Raised by the edge layer.
- **No cross-client consistency check exists for TERRAIN colours, only for marks**
  [crates/tui/src/palette.rs:198]. The gui↔tui cross-check at `appearance.rs:441-471` covers
  designation marks and carries an explicit docstring on why marks may diverge. Terrain has no
  equivalent and no documented ruling: the tui's `TreeTrunk (105,76,48)` is warm brown, red over
  blue, which would violate the gui's own terrain invariant, while the tui's foliage `(54,106,78)`
  converges with the gui's new green only by coincidence. 9.4's claim that "the two clients now
  agree" is true of foliage's hue direction and of nothing else — the two foliage values are 23.2
  apart. Leaving the tui alone was still correct. Raised by the edge and feature layers.
- **The density band only discriminates roll denominators outside roughly `36..52`**
  [crates/sim-core/tests/worldgen.rs:190]. Measured by sweeping the denominator on an out-of-repo
  copy: `0..36` → 310 (fails high), `0..40` → 282, `0..44` → 265, `0..48` → 242 (shipped), `0..56` →
  214 (fails low). So a future tuning nudge from 48 to 44 would pass unnoticed. Inherent to one-seed
  banded testing rather than a defect in this diff, and mitigated: the re-pinned terrain fingerprint
  in the same file catches ANY perturbation of the tree stream. Related: `DEFAULT_SEED`'s 265 — the
  number the band was chosen from — is the one figure with no direct guard. Raised by the blind and
  acceptance layers.
- **Two shipped comments assert opposite things about the cool directional**
  [crates/gui/src/appearance.rs:117]. `:117-119` says the directional "compresses exactly that axis"
  (green) and gives that as the reason dig and channel were moved onto RED; 9.4's new comment at
  `:214` says foliage separates on "GREEN, the axis the cool directional does not compress". One of
  two load-bearing rationales is false and both are quoted by later stories. The feature layer
  measured the pure green-axis pair from `:117-119` under the shipped directional and found it does
  NOT collapse (30.7 / 41.0 / 54.7 / 64.7 at gains 0.5-3.0), which favours 9.4's reading — but
  re-ruling another story's stated rationale on one layer's measurement is not this story's to take.
  9.4's own comment is being narrowed to what it actually measured. Raised by the feature layer.
- **AC9's third sabotage row as specified is unkillable and had to be substituted**
  [_bmad-output/implementation-artifacts/mutations/9-4-trees-fewer-and-distinct-from-the-ground.sh].
  The AC demands a row that lowers the separation floor so the old colour would pass; with foliage at
  48.1 a floor lowered to 5.0 still passes, so the row can never go red. The dev found this by running
  the table, replaced it with a brown-foliage production mutation, and documented why — the right
  call, and the replacement genuinely kills at W2's invariant rather than at the equality pin.
  Recorded because the AC text remains a trap for anyone reusing this story as a template: you cannot
  sabotage an assertion and expect that same assertion to catch it. Raised by the acceptance and
  feature layers.

## Deferred from: 9.4 review — Wolf's vehicle observations, measured (2026-08-29)

Both were reported by Wolf from a rendered frame during 9.4's review and then measured against the
shipped world by direct computation. **Neither is caused by story 9.4** — both are pre-existing
worldgen/projection geometry that the density cut merely made easier to see. Recorded here as
concrete measured defects, which is what the standing art rule (2026-08-22) requires before any
look change becomes a story.

- ~~**A THIRD OF ALL TREES HAVE NO VISIBLE TRUNK, and it is deterministic rather than
  incidental.**~~ **RESOLVED THE SAME DAY, 2026-08-29, commit `465f967`** — Wolf ruled it in rather
  than deferring it, after a second look showed the same ring as "green boxes on ground level".
  Removing the `surface + 1` foliage ring fixed both: trees showing a trunk **179 -> 265 of 265**,
  foliage cells 6,329 -> 4,505, draw-set oracle 45,261 -> 44,984, trunk columns unchanged at 265.
  "Underground" was measured and disproven — every trunk sits flush on its own ground. Pinned by
  `every_tree_shows_a_trunk_and_no_foliage_sits_at_the_trunk_base` plus a sabotage row that restores
  the ring. The finding as originally recorded follows, unedited:
  **86 of 265 trees (32.5%) draw zero trunk cells** — and it is exactly the height-4 trees, 100% of
  them; height 5 leaves one bare trunk level, height 6 leaves two.
  [crates/sim-core/src/worldgen.rs:196-227]. **Cause, from the source:** the trunk spans
  `surface+1 ..= surface+height-1`, while foliage rings are stamped at `surface+1` (the skirt),
  `crown_top-2` and `crown_top-1`. `height` is `rng.random_range(4..=6)`. At height 4 those three
  rings cover *every* trunk level, so each trunk cell has all six neighbours solid, `is_exposed`
  returns false and the cell is never drawn. Wolf: *"some trees look like they don't have trunk at
  all (are trunks under ground level) .. not too many of those"* — the trunks are not underground;
  they are enclosed by their own foliage. Cheapest fix to weigh: raise the minimum height to 5, or
  stop stamping the skirt ring at `surface+1`. Either changes every seeded world and needs its own
  before/after numbers.
- ~~**Most of the "snow-laden crown" is not on top — 68% of the ground-level skirt is bright.**~~
  **RESOLVED THE SAME DAY, 2026-08-29, commit `a634235`** — Wolf ruled "fix also snow cover" at the
  review rather than deferring it. `has_snow_laden_crown` now additionally requires the cell not to
  rest on terrain. Measured on the shipped world: ground-resting bright cells **1,029 -> 0**, crown
  cells 3,631 -> 2,602, and 9.4's new green reaches 3,727 of 6,329 foliage cells (58.9%) rather than
  2,698 (42.6%). Pinned by `foliage_resting_on_the_ground_is_a_skirt_and_never_catches_snow`
  [crates/gui/src/project.rs:1424] and by a sabotage row that deletes the clause. **The first
  fixture was VACUOUS and the row caught it** — it stacked foliage above the skirt cell so the
  pre-existing sky-exposure clause fired first and the new clause was never reached. The finding as
  originally recorded follows, unedited:
  [crates/gui/src/project.rs:933]. `has_snow_laden_crown` fires for ANY foliage cell with nothing
  solid directly above it, which is the whole outward-facing surface, not the apex. Measured on the
  shipped world: the tip ring is 265 cells, 100% crown-coloured (correct); the upper shoulder rings
  are 4,240 cells, 50% crown; and **the skirt ring at ground level is 1,824 cells of which 1,246
  (68%) take the bright `(156,170,196)` crown colour**. That is a bright ring sitting on the ground
  around each trunk. The predicate's own comment at `project.rs:931` says the material-swap design
  was chosen precisely so that capping foliage would not "put a bright slab on every ground-level
  skirt tile and bury the landform" — **it does exactly that anyway.** Wolf asked at review whether
  snow-on-top would look better if fixed; the measurement says most of the snow is not on top today,
  so there is a real defect to fix rather than a taste question. Restricting the crown swap to the
  tip and upper rings (or requiring the cell to be the topmost foliage in its column) would remove
  the ground ring. Interacts with 9.4: only 42.6% of foliage takes 9.4's new green precisely because
  the crown swap claims the other 57.4%.

## Carried out of Epic 9's shared sitting (2026-08-29) — the next story candidate

- **The withheld levers, plus AC13's ceiling reading.** Story 9.1 proved with a controlled pair that
  campfire shadows are insufficient to close the blow-out — they help (warm-lit pixels −15.7 %,
  near-white area −10 %) but nowhere near enough, and Wolf's eye agrees. Its standing rule then
  stopped it rather than reaching further, which is what the rule is for. **Opening intensity,
  amplitude, range or emissive is a new ruling from Wolf and belongs in its own story with its own
  before/after numbers.** **AC13's ceiling reading is DONE** — taken on the vehicle
  2026-08-29: `near-white-area=2.0316%` against the 1.5630426 % ceiling, **over by 30 %**. The
  ceiling is CONFIRMED, not corrected, and the frame fails it — consistent with Wolf's eye and with
  9.1's conclusion that shadows are insufficient. The follow-up story inherits a measured bar rather
  than an open question. `[wolf/MED]`
- **9.2 inherits a measured constraint, not a preference.** The hover slab is `(80,220,210)`,
  luminance **189.5**; the campfire's near-white pool exceeds **200** and saturates toward **255**,
  in both shadow states and at every threshold swept. **A brighter slab cannot fix this** — nothing
  beats a background reaching 255 on luminance alone. 9.2's fix needs hue/saturation separation or
  an outline. `[measured 2026-08-29]`

## Found while closing Epic 9 (2026-08-29)

- **`--frames` is a wall-clock budget expressed in frames, and it inverts with machine speed.**
  [crates/gui/src/ingest.rs:71] `DEFAULT_AT_TICK_FRAME_BUDGET = 1_500`. On the headless software
  renderer (~2 fps) that is ~12 minutes and hundreds of daemon ticks; on the vehicle's RTX 4080
  (~140 fps) it is ~11 seconds and **8 ticks**, which fails an `--at-tick 20` floor before any range
  check prints. **The faster the machine, the shorter the budget in real time** — the opposite of
  what an operator expects, and it cost the first vehicle attempt at 9.1's AC13 reading. This only
  became a trap once headless rendering existed and the same recipe started running on machines two
  orders of magnitude apart in frame rate. Fix shape: express the budget in ticks or seconds, or
  derive the frame budget from the requested tick count and the observed tick rate. Vehicle cards
  now carry an explicit `--frames 200000`, which is a workaround, not the fix.
  **Measured on the vehicle, 2026-08-29:** the daemon ticks every 100 ms (`TICK_PERIOD`), so
  `--at-tick 20` needs 2 SECONDS. The RTX 4080 ran 1,500 frames in under a second (8 ticks) and
  6,000 in about a second (11 ticks) — the update loop is unthrottled, so the cap expires long
  before the sim advances. Note the cap is only ever a cut-off: with `--at-tick` the run ends when
  the tick arrives, so a huge cap costs nothing and a small one silently truncates. An operator has
  no way to convert "frames" into "will this reach tick 20 on my machine" without knowing their own
  frame rate, which is exactly the calculation a recipe should not require. `[measured]`

- **`--at-tick`'s floor counts SAMPLED ticks, not ticks the world advanced — and a startup burst
  breaks it on any fast machine.** [crates/gui/src/capture.rs:302, crates/gui/src/ingest.rs:911]
  `ingest_messages` drains the socket in a `loop`, applying every queued delta in ONE frame, while
  `accumulate_motion` runs once per frame and records at most one tick per frame. During startup
  (window creation, ~45k cubes) the client stalls for a second or two; the daemon keeps ticking at
  10/s (`TICK_PERIOD` 100 ms), those deltas queue, and the first drain leaps the mirror ~15 ticks in
  a single frame. **Measured on the vehicle 2026-08-29, three runs: the mirror reached the target
  tick — the capture fired, and no budget message printed — while only 8, 11 and 11 distinct ticks
  had been sampled, against a floor of 20.** `--at-tick 20` is therefore unusable on the RTX 4080
  vehicle and cost three attempts at 9.1's AC13 ceiling reading.
  **Why it never showed up before:** on the headless software renderer (~2 fps) the client is always
  the slower party, so no backlog forms and sampled ticks track real ones — the bug needs a machine
  fast enough to stall at startup and then outrun the daemon.
  **Fix shape:** assert on `mirror.tick() - start_tick` (what the world actually advanced), which is
  what the AC means, and keep the sampled count as a separate diagnostic. The plain `--frames` path
  is unaffected in kind but shares the sampling weakness. `[measured]`

- **llvmpipe UNDER-READS near-white area by ~16 %, and that is the metric now used as the gate.**
  Measured 2026-08-29 by running the same tree headlessly and on the vehicle. Four of five metrics
  agree closely — ground median **exact** (117 vs 117), warm-lit pixels, p99 and the blown-pool
  diagnostic all inside the headless range — so the software renderer is a faithful proxy for
  luminance-distribution work. Near-white area is the exception: headless 1.738-1.753 % against the
  vehicle's 2.0316 %. **A headless area figure is therefore OPTIMISTIC and must never be judged
  against `NEAR_WHITE_AREA_CEILING`, which is calibrated on a GPU frame.** Read headless area only
  as a delta between two headless runs — which is how AC7 and AC13's controlled pair were read, so
  neither conclusion is affected. If headless area is ever wanted as a gate, it needs its own
  renderer-specific ceiling calibrated the same way boot7's was. `[measured]`

## Deferred from: code review of 10-1-the-headless-bench (2026-08-29)

Four layers, zero coverage holes. Three items deferred as real-but-not-now; the two LOW
latent-silent-failure items (`audit-mutations.py`'s Rust-only orphan wording and `mutate.sh`'s
backup set missing `_bmad/scripts/*.py`) were NOT deferred — they were patched under the standing
frostvein exception for instruments that report a wrong value nobody will ever see.

- **No `timeout=` anywhere in the new Blender-spawning chain.**
  `scripts/tests/test_valley_bench.py:81-101` calls `subprocess.run([...blender...])` with no
  `timeout=`; `scripts/mutate.sh:69-70` and `scripts/gate.sh:117` both call into it unwrapped.
  These are the first subprocesses in this repo that can hang the gate or a mutation run
  indefinitely with no recovery. `mutate.sh`'s own comments treat exactly this class as
  first-class for cargo (the NO-COMPILE false-green history) but nothing covers the new tier.
  No hang has been observed; deferred on that basis.

- **`export_world.py` ignores `CARGO_TARGET_DIR`.**
  `scripts/bench/export_world.py:18` derives `<repo>/target/debug/simd` from `__file__`, which is
  exactly what AC5 required (runs from either devpod mount) and is met. But a user who builds with
  a target-dir override gets a stale or missing binary — the "stale-binary trap" the story records
  as having fired six times, in a seventh shape. Deferred: correct for both real mount paths today.

- **THE BENCH'S FIRST FINDING, and it points at the CLIENT, not the bench — Wolf, 2026-08-30.**
  Shown the pair, Wolf's verdict was that **the bench looks MORE like what we are targeting than
  `gui-capture.png` does** (noise aside). This inverts the story's premise, so it is worth stating
  precisely. It is NOT the terrain: that is calibrated to the client (mean band luma 103.6 vs
  105.7). The difference he is responding to is the **camp pool** — the client's campfire runs at
  25,000,000 lm and blows its centre to flat white, where the bench's Cycles 1,500 keeps detail.
  **This is the same complaint as 6.2's carried-open "camp is too blown out", and the 2026-08-22
  ruling that closed it explicitly did NOT treat this case.** That ruling's own comment
  [`appearance.rs:66-76`] says the blow-out "is in the PEAK", drops the base so the peak lands on
  5.4's approved ceiling, and records that "this still frame never moved". `gui-capture.png` is a
  still frame. So the still-frame blow-out was ruled out of scope, not ruled acceptable.
  **For the art pass (10.4) to decide, not this story:** whether the client's still-frame camp
  should come down toward what the bench shows. Do not change `light_properties()` on the strength
  of one observation — but this is the first time the bench has been used for the thing it was
  built for, judging a look before anyone builds it, and the answer it gave was about the client.

- **AC9 guards the light table's colours but not its intensities.**
  `crates/gui/src/appearance.rs:45,48` carries `ambient_brightness: 4_500.0` and
  `directional_illuminance: 22_000.0`; the bench uses `sun_data.energy = 3.0`
  (`scripts/bench/valley_bench.py:352`) and point energies `750/1500/300` against the client's
  `14M/25M/5M` (`appearance.rs:52-86`). Blender and Bevy units genuinely differ, so the AC's
  "literal-equal" cannot apply to intensities and the delivered guard silently covers only the
  colour half. An interpretation question for 10.3's contract work, not a defect.

- **Ramp tiles render as full cubes; the bench has no sloped geometry.**
  `scripts/bench/valley_bench.py:215-218` treats a `{"ramp": ...}` tile as solid for exposure
  (correct — it matches `project.rs:429-442`, and `Ramp(_)` must occlude), but `FACE_CORNERS`
  defines only axis-aligned cube faces, so a ramp draws as an ordinary cube. The script names its
  other known divergence in a `// NOTE:` (the client's top-slice layer) and does not name this one.
  Deferred as a simplification; folded into `what-you-will-see.md` so Wolf is not asked to
  rediscover it.

## Deferred from: code review of story 10-2-the-live-seat-blendermcp-on-gingerspice-spike (2026-08-31)

Four-layer fresh-context review, no coverage holes; baseline `311e169..HEAD` (the story's own commit
range — the frontmatter `baseline_commit` was stale by three merged PRs). The spike's bit-exact
claim was independently reproduced by three layers and holds.

- **`MIN_SUBJECT_LUMA` does not catch a total lighting failure.**
  `scripts/bench/spike_pine_render.py:45`. Both suns at energy 0 still renders
  `subject_luma=34.578` against the 20.0 floor and exits 0, because Cycles treats the world
  backdrop colour as an environment light — the subject stays lit with no key and no fill. Only
  ~1.7x headroom, and it is a side effect of Cycles physics rather than a designed margin. The
  floor's comment ("the asset renders too dark") implies it guards a lighting regression; it does
  not. Belongs to the already-owed "harden `spike_pine_render.py`" item (story 10.2's decision
  record, owed item 3), which currently has no home — see the action-item finding in that story.

- **The other three floors in the spike render bench are decoration for anything short of
  "nothing rendered".** `scripts/bench/spike_pine_render.py:43-46`. Measured headroom against the
  four real assets: `MIN_SUBJECT_FRACTION` (0.02) 3.9-7.8x; `MIN_DISTINCT_COLORS` (32) 325-353x,
  and still 74x with both lights off. `MAX_SUBJECT_FRACTION` (0.90) is untestable by any normal
  content change (real fractions sit at 0.08-0.16) and exists purely as the sRGB/linear trip-wire —
  which it does correctly: reintroducing the documented bug drives it to 0.999770 and exits 1. Not
  a defect, but the guard set should be recalibrated when the script is hardened.

- **Binary data artifacts committed against AC2's own condition.** AC2 and Task 3 commit the
  `.blend`/glTF **iff** handoff candidate (b) wins. Candidate (a) won, and the record calls the
  GLBs "committed as convenience" — the exact thing the condition excludes. The four
  `export/*.glb` are pure redundancy (byte-reproducible from `voxel_pine.py` in ~2.5 s each);
  `tree.glb` earns its place on other grounds, as the evidence for the AC4 revision-mismatch
  finding. Deferred rather than patched: labelling the stale artifact addresses the real harm, and
  deleting committed assets is a call for Wolf, not a review.

- **3.7 MB of next-story assets committed into 10.2's signoff folder.**
  `10-2-signoff/dwarf.mp4` (3.4 MB), `dwarf-animation-reference.jpg`, `dwarf-contact-sheet.jpg`.
  `what-was-found.md:12` states plainly: "Input for a later story; nothing in this one consumes
  it." Outside the story's declared file list and git-permanent. Relevant to 10.5 (dwarves), which
  will want them — so the question is placement, not whether to keep them.

- **The `*:Zone.Identifier` gitignore rule leaves two already-tracked files behind.**
  `.gitignore:39-41`. `_bmad-output/implementation-artifacts/6-1-signoff/6-1-motion-after.png:Zone.Identifier`
  and `...-before.png:Zone.Identifier` are tracked; gitignore never untracks. Pre-existing from 6.1,
  not 10.2's mess, and explicitly left alone per CLAUDE.md ss3. The new rule itself was verified
  correct at every depth and does not disturb the `!_bmad/scripts/session_tokens.py` re-include
  immediately above it.

- **`ASSET_NOTES.md`'s "Generating" block uses relative paths without stating a working
  directory.** `10-2-signoff/ASSET_NOTES.md:10-17` gives `blender --background --python
  voxel_pine.py -- <type> <out.glb>` and `export/SM_VoxelPine_Tree01.glb` with no cwd stated. Folded
  into the `tree.glb` labelling patch if that is taken; recorded here in case it is not.

### Story 10.2's three OWED items — recorded here because they had no durable home

Found at review: `sprint-status.yaml` rules that action-item state lives on GitHub issues labelled
`action-item` "**and nowhere else**". Verified with `gh issue list --label action-item --state all`
— 10 issues, newest #53, **none** of them 10.2's. `action-items.md` has no 10.2 entry either. All
three items below lived only in one story's prose, which is how a deferral gets lost.
**RESOLVED 2026-08-31: issues opened — M2-20 (#57, route:story, the scale constant),
M2-21 (#58, route:undecided, the handover runbook), M2-22 (#59, route:story, hardening the
spike bench). STATUS NOW LIVES ON THOSE ISSUES; the prose below is reasoning only.**
Numbered M2-, not M3-: epics 9 and 10 are ADDED SCOPE INSIDE M2 (Wolf, 2026-08-28), and 8.3
is still the milestone gate. Wolf's ruling
at review (2026-08-31): the handover process and mechanism are real work but **not 10.2's scope**,
so they are recorded, not built. **Issues are NOT opened — that is Wolf's call to make.**

1. **The handover runbook.** Wolf: *"just need to think about handover process at start"* /
   *"templates are first step .. that is not urgent now"*. The content already exists, proven on one
   asset: the standing asset contract and the per-asset brief, both drafted in 10.2's Dev Agent
   Record and applied in `10-2-signoff/ASSET_NOTES.md`. **Caveat found at review:** the story names
   10.3 as its "natural home", but 10.3's epic text is about `docs/tech-art-guidelines.md` contracts,
   not a session handover procedure — so 10.3 will not pick this up on its own.

2. **The metres-per-voxel project constant. BLOCKS ASSET #2.** Currently unset. The tree was built
   at 0.2 m voxels off a 1.2 m dwarf; at that same voxel size the DWARF is 6 voxels tall, which
   cannot carry the beard, belt, tunic panel and lantern the reference sheet draws. Pick it from the
   dwarf's detail needs (~0.1 m or finer), fix it once, let every other asset follow. Second,
   coupled decision: the client's cell is a unit cube (`Cuboid::default()`) while `worldgen.rs`
   grows trees 4–6 cells, against this asset's ~6.3 — so cells-per-asset belongs with it.
   **Caveat found at review:** `epics.md:1505-1506` does name grid scale as blocking 10.4/10.5, but
   that text predates the spike and contains none of the measured finding above. The sprint board's
   claim that 10.3's blocker text "is already right" overstates what 10.3 will actually read.

3. **Hardening `scripts/bench/spike_pine_render.py`** — a test plus a sabotage row, per Task 3's
   stated exception ("if the decision keeps the script, hardening it is the follow-up's first
   task"). The decision keeps it. **This one had no home at all** — no issue, no deferred entry, and
   10.3 is a docs story. Two concrete inputs from this review, both already deferred above: the
   `MIN_SUBJECT_LUMA` floor does not catch a total lighting failure (34.578 against a floor of 20.0,
   ~1.7x and coincidental), and the other three floors are decoration for anything short of "nothing
   rendered".

## Deferred from: code review of 10-6-how-fine-can-we-go (2026-08-31)

- **`--subdiv` is discoverable nowhere and `gui --help` fails** [crates/gui/src/ingest.rs:509].
  Unknown arguments fall through to the port parse, so `gui --help` exits with "invalid digit
  found in string" rather than printing usage. Nothing in `docs/` or `README.md` mentions
  `--subdiv`, and the vehicle card says "read the frame-time overlay" without saying the overlay
  is off by default and toggled with F3 (`ingest.rs:912`). The missing `--help` is pre-existing
  and larger than this story; only the undocumented new flag belongs to 10.6. Raised by the
  Feature Auditor.
- **No test covers `--subdiv 0` or `--subdiv` with a missing value.** The parser rejects both
  correctly at runtime (`ingest.rs:466-478`), so this is a test-coverage gap in CLI parsing
  rather than a defect in this story's code, and it matches the existing coverage level of the
  sibling flags. Raised by the Acceptance Auditor.

## Deferred from: code review of 10-6-how-fine-can-we-go (2026-09-01)

- **Malformed or truncated snapshot raises a raw traceback instead of the bench's own diagnostic** [scripts/bench/resolution_bench.py:66-68, :105-112, :115-133, :531]. `KeyError`/`IndexError` are not in the `except (OSError, ValueError, subprocess.CalledProcessError)` tuple, so a snapshot missing `dims` or `tiles`, or carrying a `tiles` array shorter than `dims` implies, exits 1 with an unfiltered stack trace rather than `resolution bench failed: …`. Verified by running on three malformed fixtures. Deferred because it fails LOUDLY and produces no wrong number — it is not a silent-failure trap. [edge, round 2]
- **AC6's A\* rows come from an `#[ignore]`d test** [crates/sim-core/src/lib.rs]. `resolution_bench_times_existing_astar_on_subdivided_flat_grids` never runs on the gate, so the axis-b path-found column — the reproducible half of that table, and its real finding — has no regression protection. Deferred: acceptable for a measurement instrument, and story 10.6's "costed, not built" guardrail argues against touching `sim-core` further. Revisit if a later story makes the sim grid finer for real. [feature, round 2]
- **Vestigial `SnowCap` match arm in the incremental rebuild branch** [crates/gui/src/project.rs:1173-1183]. `spawn_snow_cap` is called only from the two subdiv ≤ 1 branches, so under `subdiv > 1` — the only condition the incremental loop runs under — the `cap` arm is permanently `None`. Harmless; falls through correctly to the `TerrainChunk` match. Left from before `bace455` moved snow from entities to paint. [blind, round 2]
- **`terrain_positions_near` inherited `terrain_positions_at`'s doc comment** [crates/gui/src/project.rs:1755-1763]. The "client-local draw set at a slice: retain full-depth exposure…" paragraph now documents the restricted-scan function, and `terrain_positions_at` at :1791 has none. [acceptance, round 2]

## Deferred from: 10.6 vehicle observation (2026-09-01)

- **Re-benchmark the fine terrain path UNDER LOAD.** Every figure in 10.6 — fps, per-dig mesh build, boot build — was measured on a nearly-idle world: tick 21, ten entities, five dwarves, and digs issued one at a time by hand. A populated fortress adds dwarves, items, point lights, designations and zones, and digs far more often and in bursts. So 10.6's fps numbers are an UPPER BOUND and its dig frequency an under-count, and the adopted k=4 is chosen with that stated rather than hidden. Wolf, 2026-09-01: "might be of course when we have more things going on it starts to degrade performance so need to benchmark this later on." Owed once there is a fortress worth loading — likely after the M2 gameplay stories, not before. [vehicle, round 2]

## Deferred from: code review of 10-3-the-rules-of-the-look (2026-09-01)

- **Nothing auto-checks any asset that does not already exist.**
  `scripts/tests/test_check_asset.py:52` hardcodes the five committed `.glb` paths. The gate does
  re-check those five on every run — better than assumed — but there is no glob, no discovery and
  no CI (`.github/` does not exist in this repo). `assets/gltf/`, the runtime home the contract
  names at `docs/tech-art-guidelines.md:272`, is created by story 10.5. **10.5 inherits this**: an
  asset landing there is checked by nothing.
- **Nothing hands the asset producer the contract.** Hops 1-2 of the feature path are unwired: no
  generator, hand workflow or MCP session is given the rules at production time.
  `voxel_pine.py:714` cites "the asset contract's clause 6", which resolves to nothing in `docs/`.
  The MCP runbook is explicitly out of scope (issue #58).
- **Batch runs abort at the first bad file.** `scripts/bench/check_asset.py:289` returns on the
  first failure, so a user with ten assets fixes them one round-trip at a time. Spec-consistent
  with AC6's "the first violated clause". Related: a structurally invalid GLB emits no `FIGURES`
  line at all, though AC6 promises one per file.
- **"Three divergent implied values" is two consistent halves plus one stale constant.**
  `docs/tech-art-guidelines.md:181`. The sheet's dwarf reading (12 voxels = 1.20 m = 0.75 cells)
  and its tree reading (measured in 1.6 m cells) are two halves of ONE self-consistent derivation,
  not rival values; the only real divergence is `gui`'s stale `scale: 0.65`. The Dev Notes admit
  this ("it is one stale constant") but the shipped contract text keeps the three-way framing.
  AC3's letter is met. This is the project's recorded "check the upstream reference" pattern.
- **The adopted terrain `k = 4` has no constant and no owner.** `docs/tech-art-guidelines.md`
  records 0.4 m terrain visual voxels as an ADOPTED DECISION, but the shipped default is `k = 1`
  — `TerrainSubdivision` is inserted only under `--subdiv` [`crates/gui/src/ingest.rs:203`] and
  consumers fall back via `subdivision.map_or(1, ..)` [`crates/gui/src/project.rs:1108`, `:1184`,
  `:1196`]. Reaching the adopted resolution today requires `--subdiv 4`. Making k=4 the default —
  putting the adopted `k` in one constant so a future evidence-led revision changes one constant
  rather than a grid convention — is owed by whichever story next
  takes terrain rendering; 10.4 (trees) and 10.5 (dwarves) are not natural owners.
  `docs/tech-art-guidelines.md` now carries only a three-line normative note and points here. The
  sentence it used to carry, verbatim, and the reason this entry exists: **"Do not read this
  paragraph as a description of what the client does."** Wolf's ruling
  at 10.6 fixed the adopted value at 4 (k=8 is servable at 100–140 fps but every dig hitches
  38–78 ms against k=4's 5–13 ms). Related: `--subdiv` "is discoverable nowhere and `gui --help`
  fails" (see the earlier entry in this file).
- **10.2's standing-contract clauses 6 and 8 were not ported into the durable contract.**
  Clause 6 (self-verification order; "Exit 0 with no output is not a result"; signed volume vs
  voxel count) and clause 8 (declare known deviations) remain only in
  `10-2-the-live-seat-blendermcp-on-gingerspice-spike.md:570-597`. Clause 7 WAS ported by the
  2026-09-01 review. Consequence to close with them:
  `_bmad-output/implementation-artifacts/10-2-signoff/voxel_pine.py:714` cites "the asset
  contract's clause 6" and that reference now resolves to nothing under `docs/`.
