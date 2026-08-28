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
- Point lights cast no shadows (`crates/gui/src/project.rs:118`, `..Default::default()` →
  `shadow_maps_enabled: false`); AC4's "shadow" term is carried entirely by the single
  250-lux directional. Perf-vs-look tradeoff for a vehicle session, not a headless call.
  `[auditor/LOW]`
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
- **Process, not code — the per-layer build isolation has no reaper.** This review's four
  `CARGO_TARGET_DIR`s cost **~92 GB**, and `/tmp` already held ~25 GB of orphans from earlier reviews
  (`review-accept`, `review-orchestrator`, `review-sim-core`, `review-tui`, `review-protocol`).
  Headroom was fine (439 GB free) and nothing was deleted during the review, but the P2 isolation rule
  shipped without a cleanup step and the cost accumulates one review at a time. `[orchestrator/LOW]`

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
- **The hover highlight is invisible on every tile with a drawn tile above it.** DEFERRED TO 8.2 by
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
  Almost certainly downstream of the campfire's already-open blown-emitter item rather than a new
  hover defect: `04e6de5` raised the campfire amplitude 0.11→0.40, peaking at 44.8M, ~40% above the
  value 5.4 was sized against, and a cyan slab at `(80,220,210)` will not survive that exposure.
  Recorded against that open item; no look-tuning now, per the art rule. `[wolf/MED]`

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
- **The campfire is still blown out and is now measurably obstructing observation.**
  Carried open since 6.2 and re-confirmed by Wolf on 2026-08-27 as a reason the client is hard to
  read: *"campfire is still overblown so it hides stuff"*. This is no longer only a look
  complaint — it is degrading the vehicle as an instrument, which is what makes it worth ranking
  above ordinary look items when the gfx pass is planned. `[feature/MED]`
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

