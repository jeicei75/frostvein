# Agent position journal, 2026-08-03 → 2026-09-04 (archive)

**What this is.** The verbatim contents of the agent memory `frostvein-position.md` at the point it
was trimmed on 2026-09-04. It had grown to 2,760 lines / 208 KB — a per-session narrative of every
story from 1.1 to 10.7, kept in a file whose job is to say where work stands, and which loads on
every recall.

**Why it is archived whole rather than filtered.** Wolf's instruction was to trim it without losing
anything that exists nowhere else. Every one of its 72 cost figures was checked and is either
present in `metrics/` or is arithmetic on ledger rows that are — e.g. 5.4's "$53.25 / 379 turns" is
exactly the ledger's $42.46 + $10.79 over 337 + 42 turns, and existing rows are stamped
`rates 2026-08-01`, so a forge-level repricing cannot move them. The narrative, though, carries
rulings, measurements and dead ends whose derivability cannot be proved line by line at sensible
cost. Archiving the whole file is the zero-risk option; filtering it is not.

**Where the live record actually lives** — this file is history, not state:
`sprint-status.yaml`, the story files, `action-items.md`, `deferred-work.md`, the epic retros,
`metrics/*.md`, and GitHub issues (per the live-bug rule shipped in forge-process 1.5.0).

**Ordering warning.** The journal is not chronological. It runs newest-first for the first few
sections, scrambles through late August, then at the `STATE AT 2026-08-04` heading drops back to the
beginning and runs *forwards* to late August. Two chronologies point opposite ways in one file.

---

---
name: frostvein-position
description: "What frostvein is, the forge-hosted arrangement, and where work stands. CURRENT 2026-09-03: **10.4 DONE — MERGED as PR #63, merge commit 276dbd8.** Three stories left in M2: **10.7 the-sun-lights-the-valley — BUILT, REVIEWED and PATCHED 2026-09-03; status `in-progress`, branch PUSHED, no PR. Next action is WOLF'S VEHICLE SITTING** (card: `10-7-signoff/review-vehicle-runbook.md`) — UX-DR22's closing half covers the sun only and his sittings predate the torch/emissive/subdiv-2 fixes and **10.5 dwarves-worth-looking-at** (10.7 is placed ABOVE 10.5 and must run first), plus 8.3 last of all. **Board and story both say `done`; closed via PR #64 (`47139fa`). Nothing outstanding, tree clean on main.** NFR6 PASSES on the vehicle: >100 fps with the mesh trees at 67% of the triangles. THE SUN-UNDER-THE-MAP FINDING IS CLOSED by 10.7 (elevation -6.42 -> +17.66 deg, Wolf's ruling; [[feature-on-everywhere-does-nothing]] is now history, not an open item). Its consequence still stands though: every look judgement made BEFORE 10.7 — 9.1's blow-out work, 9.4's tree colours, 10.3's rules of the look, 10.4's tree judgement — was tuned with the sun off and none has been re-judged under it; that is a later story and Wolf's to schedule. Also still open and pre-existing: NEAR_WHITE_AREA_CEILING is breached on main (`gui --headless --capture` exits 101), filed as its own defect.
metadata: 
  node_type: memory
  type: project
  originSessionId: 0d055a59-6fae-4208-8220-4eecf3e9ff04
  modified: 2026-08-31T14:20:29.642Z
---

## 2026-09-03 (FINAL) - 10.4 MERGED. START HERE NEXT SESSION.

**PR #63 MERGED**, merge commit `276dbd8`. Verified against the remote, not the change log:
`state=MERGED`, every commit an ancestor of `origin/main`, remote branch deleted, no stacked PR
left pointing at it. **[[post-merge-branch-trap]] fired for the 6th time** — the merge dropped the
working copy onto `main`.

**Closed out via PR #64 (`47139fa`), merged.** The bookkeeping — sprint-status
`10-4: review -> done` and the story's `Status: done` plus a merge row — is on `main`. Nothing
outstanding; working tree clean on `main`.

**Wolf called the post-merge closure round trip "annoying stuff", and he is right.** The board and
story statuses cannot be set to `done` inside the story's own PR (they are not done until it
merges), so every story ends with a second branch + PR + merge for two lines. Worth asking him
whether docs-only closure commits can go straight to `main` and skip the PR.

**Three stories left in M2**, and the order is decided:
1. **10.7 The Sun Lights The Valley** — **UPDATE 2026-09-03 (review + patch pass, fresh session):
   SHIPPED AND REVIEWED, status `in-progress`, branch pushed, NO PR.** Four layers, zero coverage
   holes, full gate GREEN, mutation table 7 -> 12 rows all KILLED. The code was sound on every
   count executed — sun +13.2 mean luminance on two independent builds, five toggles wired live, no
   third residual emitter (`warm-lit=0` with all off), subdiv-2 holes closed. **The one HIGH was the
   EVIDENCE**, see [[artifact-name-outlives-its-content]]. **Wolf lifted the no-CLI-flag ruling at
   triage — scope change #5** — so `--lights-off` and `crates/gui/tests/pixel_guard.rs` landed, the
   first tests here that assert PIXELS; the full gate went 67s -> **4m57s** and Wolf chose to keep
   it (full tier only; fast tier still ~5s and names them skipped). **Blocked only on Wolf's
   sitting.** Two hunter findings were territory-split false positives,
   see [[territory-split-false-positives]]. Original entry below, now history:

   **`ready-for-dev` as of 2026-09-03**, context-filled,
   evidence in `10-7-signoff/`. **All three rulings ANSWERED by Wolf**: elevation is BENCHED not
   pre-picked (Task 2/3, UX-DR22 opening half); the sun is DECOUPLED from the aurora; nothing else
   is re-tuned (9.1/9.4/10.3/10.4 and the near-white ceiling all stay). `baseline_commit` corrected
   `e930d07` -> `47139fa` (it was 5 stale commits and AC1 grades the diff from it). Its board key is
   deliberately placed ABOVE 10.5 because [[board-order-contradicts-ruling]] means prose ordering
   loses to top-to-bottom. **BRANCH `10-7-the-sun-lights-the-valley` EXISTS, cut from `main` at
   `47139fa`, one commit `0e669a9` "Create story 10.7 and rule the sun" (story + epics.md +
   sprint-status + metrics). NOT pushed, no PR — review-gated. Working copy is ON that branch.**
   NEXT ACTION: `dev-story` (delegates to Codex, then chains to review).
   **Two traps the story now carries, both measured at creation and both new:** (a) it is a
   DIRECTION bug — Bevy ignores a directional light's translation
   (`bevy_light-0.19.0/src/directional_light.rs:25`), the sun is at **-6.42 deg elevation**, and the
   draft's own suggested height guard WOULD CERTIFY A BROKEN BUILD; (b) `gui --headless --capture`
   **already exits 101 on `main`** (`near-white area is 1.8757%, above the 1.5630% ceiling`) — the
   ceiling is breached on BOTH venues before the story starts, so a red capture check is not
   evidence the sun broke anything, and the PNG is still written (save-then-validate).
2. **10.5 Dwarves Worth Looking At** — `backlog`. Must NOT be judged before 10.7 lands: approving
   authored dwarves under an ambient-only scene repeats 10.4's own failure one level up.
3. **8.3** — still last of all.

**Carried out of 10.4's sitting, all in deferred-work.md:** the sun; the fps variation while
dwarves move (pause the daemon with the view held — lanterns stay, deltas stop — to tell lighting
from the incremental re-mesh path); and the near-white ceiling reading **2.2071 %** on the RTX 4080
against 9.1's **1.5630 %** bar, which wants a baseline GPU capture before anyone touches that
constant. **Do not raise the ceiling to clear the panic.**

## 2026-09-03 (latest) - PUSHED, PR UPDATED, SUN STORY WRITTEN

Wolf: *"push and update pr then write sun story so we don't forget it."* Done. Full gate GREEN
before the push and again in the pre-push hook. PR #63 body rewritten end to end — it now carries
the vehicle sitting, the three card corrections Wolf's run exposed, and the sun finding.

**Story 10.7 `The Sun Lights The Valley` created** (`10-7-the-sun-lights-the-valley.md`), status
`backlog`, explicitly NOT context-filled, with three rulings open and ACs marked provisional. Its
board key is placed **physically above 10.5** because [[board-order-contradicts-ruling]] has cost
this project twice — prose ordering loses to the top-to-bottom rule. It must run first: judging
authored dwarves under an ambient-only scene repeats 10.4's own failure (approving a frame the
client does not draw) one level up.

**Process note worth keeping:** 10.7's story doc was committed onto 10.4's branch, which breaks
one-story-one-branch. Deliberate — Wolf wanted it un-forgettable and #63 merges soon — but it is
called out IN the PR body rather than left to be discovered. A rule bent knowingly and in writing
is different from one broken quietly.

## 2026-09-03 (later) - THE VEHICLE SITTING: AC12 CLOSED, AND A SHADOW FINDING

Wolf worked the card on build `3b0c43f` (RTX 4080 Laptop / NVIDIA 616.56). **Both questions the
devpod could not answer came back yes:** a lone `gui.exe` from `$env:TEMP` with no `assets/` loaded
all 265 pines from `embedded://`, and the WINDOWED `--capture` printed `265 of 265` and wrote its
PNG where before the review it asserted `0 == 265`. **NFR6 PASSES** — >100 fps typical, brief ~60,
against 60 / 30 floors.

**Three corrections to my own card, all found by Wolf running it.** (1) I listed `slice:` among the
lines a plain `gui.exe 7451` prints; it is capture-only. (2) I predicted the GPU would read the
near-white ceiling UNDER the bar — it is the worst reading yet, 2.2071 % vs 1.5630 %. (3) I said
`--frames 2000` was plenty on an inference that divided *mid-blend* frames by wall time, which
measures nothing; **the panel is 144 Hz G-Sync**, so 14 s is ~2016 frames and a truncated run could
not be told from a complete one. Use 20000. **Lesson: a card is only tested where its author ran
it — I ran every command headless, and every one of the three misses is at a windowed/vehicle-only
seam.** Also: 8.2's "~140 fps" and 10.6's ">140 at k=4" were reading the 144 Hz panel, not a scene.

**THE SUN CASTS NO VISIBLE SHADOW** — Wolf's aside, measured, and not confined to the trees.
See [[feature-on-everywhere-does-nothing]]: deleting the whole directional shadow pass moves fewer
pixels than two runs of the same build. Strongest cause is Bevy's `CascadeShadowConfig` default
`maximum_distance: 150.0`, never set here, on a world of **1 render unit per cell**. May be one
defect with the tree-root reading — an object casting no ground shadow has nothing anchoring it.

**State: 8 commits (`8f2464d`..`2100f1a`) COMMITTED, NOT PUSHED.** Story at `review`, board updated,
full gate GREEN. Two deferrals out: the fps variation attribution and the GPU near-white reading.

## 2026-09-03 - 10.4 CODE-COMPLETE, CARD WRITTEN, WAITING ON THE VEHICLE

10.4's only open AC is **AC12's closing half** — Wolf's eye plus two fps readings on gingerspice.
Every prior visually-gated story (6.1, 6.2, 7.1, 7.2, 8.2, 9.1, 9.4) shipped a vehicle card and
10.4 had none; it now exists at `10-4-signoff/task-6-vehicle-runbook.md`. Every command in it was
executed here on `12da79d` before it was written, not remembered — see [[live-gate-rule]].

**Two things the dry run established.** The `build.rs` stamp defect **reproduced**: the binary
would not restamp after a commit until `crates/gui/build.rs` was touched by hand, so that `touch`
is a mandatory build step in the card. And the story's own "Still open" note published
**"~479 k terrain triangles" as a figure that was never measured** — it matches no reading at any
subdivision and `rg` finds it nowhere in the repo. Measured: 576,972 triangles at the shipped
default (subdiv 1), 926,426 at subdiv 4, against ~1.19 M for the 265 pines, so the trees are
**67 % of the scene as `gui.exe` draws it** — a sharper fps question than the story was asking.
See [[documented-constant-was-a-measurement]].

**Also fixed while there:** the File List still named `client-head-9eba31f-subdiv2.png`, a file the
yaw re-capture replaced with `client-head-06471d7-subdiv2.png`, and the Task 5 completion note
still read as a live claim while publishing the three figures the review proved sat inside the
noise floor — annotated as superseded rather than rewritten. [[partial-doc-update-immunises]].

**State: 2 commits (`8f2464d`, `3b0c43f`) COMMITTED, NOT PUSHED** — rule 4 wants Wolf's explicit
yes. PR #63 is open and is stale by those two commits until they are pushed.

## 2026-08-30 - 10.1 DONE; THE RECORD SAID "PR OPENED" WHEN IT WASN'T

AC15 was closed by Wolf on the re-rendered signoff pair ("geometrically it's correct and overall
looks the same" — the bench predicts the build), story marked done, branch pushed. **But the change
log's "Pushed and PR opened" was half false: no PR existed.** The resumed session verified with
`gh pr list --state all` and opened **PR #40**, based on `9-4-trees-fewer-and-distinct-from-the-ground`
(10.1 stacks on 9.4; 26 own commits). This is [[stale-record-fabricates-scope]]'s sibling shape —
a record asserting completed work that never happened; verify "opened/merged/pushed" claims against
the remote, not the log.

**Standing hazards on #40:** 9.4's PR #39 is still OPEN with base main. Per the stacked-PR merge
trap (see 2026-08-22 entry), after #39 merges run `gh pr edit 40 --base main` BEFORE merging #40,
or its commits land on a dead branch while GitHub reports "merged".

**Bench's first finding, logged not acted on:** Wolf judged the bench closer to the art target than
the client — the camp pool's blown core (client campfire 25M lm vs bench 1,500; only light COLOURS
are pinned, intensities deferred). Same complaint as 6.2's carried-open "camp too blown out".
Filed for 10.4 in deferred-work.md. `codex review --base main` never ran on 10.1 — an absence, not
a pass. Next in backlog: 10.2, the live-seat BlenderMCP-on-gingerspice spike.

## 2026-08-28 - 8.2 CLOSED. A WORKING MODE READ AS A THIRD DEAD ONE.

The readout pass ran. AC18 is met on all four modes, read back from the sim by a second client:
dig 8 designations @ z8, stockpile 12 zones @ z9, channel 16 designations split 9/7 across z9 AND
z10 (channel follows the ground, so ONE drag lands on several levels — sum them, never read one
cut), clear removed every zone and every channel mark across two drags while leaving all 8 digs
untouched. NFR6 read at ~140 fps against floors of 60 and 30. Status → done.

**Three consecutive reads showed `designations=0` for a channel that worked perfectly.** The story
had already produced two genuinely dead modes, so absence looked like a third — and I nearly
reported it as one. Measured against the real daemon instead: 16 channel designations accepted in
full, ALL consumed within ~25 ticks (~2.5 s), 16 blocks below left as `Ramp`. A channel job is
worked from the cell the dwarf already stands on, so it needs no travel; a dig sits at a solid cell
a dwarf must path to, which is why dig marks linger and channel marks do not. See
[[transient-observable-needs-pause]].

**Two of my own runbook defects fed it**, both corrected: the card named a `gui.exe` one commit
stale (the stale-binary trap relocating into the INSTRUCTIONS), and it ordered the clear drag
BEFORE the only read — clear cancels on both the picked rect and the standable rect, so it erased
the evidence it was meant to check.

**MERGED 2026-08-28: PR #35 (a949835).** Sprint status marked done on main as `c2aa7fc`, following
8.1's precedent (`cca118a`) of committing the done-marker straight to main rather than via a branch.
M2 now 10/11 — one story left in the milestone.

**Deferred by Wolf, asked not assumed:** clear cannot reach a mark in a column holding a second
standable cell when the two drags are anchored at different heights (measured: near_z=2 → z2,
near_z=5 → z5). Needs a cave or an overhang to fire. In `deferred-work.md` with a reopen trigger.

## 2026-08-27 - SESSION PAUSED BY WOLF. RESUME AT THE READOUT PASS.

**State: tree clean, full gate GREEN, 17 commits on `8-2-designate-with-the-mouse`, NONE PUSHED,
no PR, status `in-progress`.** The resume pointer with binary stamps lives at the top of
`8-2-designate-with-the-mouse.md` under "RESUME HERE" — read that, do not re-derive.

**Only two items are owed and neither needs art:** four `marks:` lines (two reads per drag pair —
dig sits at the picked cell, channel/stockpile one level up, so no single `--z` shows all four)
and two fps readings. Dig is already confirmed by Wolf; the orientation question is closed as
observed. When those land, 8.2 → `done` with AC13's rendered half filed against the gfx pass.

**What this session cost and why it was worth it:** four vehicle rounds, each triggered by a vague
Wolf report — "fragile and confusing", "still not there", "do we have coordinates wrong", "gives me
0 but I can see in tui". **Every one of the four was a real defect that all four review layers and
a 27-row sabotage table had passed.** Two dead modes, a spec defect in AC4, a ~135-degree
orientation mismatch, and three separate instruments reporting success while capturing nothing.
**Treat a vague "it feels off" from Wolf as a lead to run to ground with a reproducer, never as a
look complaint.**

## 2026-08-27 - THE CLIENTS DID NOT AGREE ON WHICH WAY IS NORTH, AND NOTHING SAID SO

Wolf: *"do we have coordinates wrong? I think I dig on north but got `*` in west."* **They were
not.** `BOOT_YAW = 0.7` rad (~40 deg) and the camera orbits freely, so straight up the Bevy screen
is world `-x,+y` — a diagonal that MOVES — while the TUI's screen axes ARE the world axes. **World
north lands DOWN-LEFT in the Bevy client at boot.** Both clients were right; neither said which
way it faced. Now both carry a compass (Bevy's computed through the SAME projection the picking
ray uses, so it cannot disagree with what is drawn; it says `?` rather than inventing a bearing),
the Bevy readout names the cell under the pointer, and `tui --frame` prints the marks' world span.
**Confirmed correct on the vehicle by Wolf.**

**Wolf's own second cause, worth keeping:** the two clients cover very different amounts of world
— TUI one screenful (~a tenth of a 128x128 world), Bevy the whole valley — so a mark visible in
one is legitimately off-screen in the other. **Two independent reasons never to cross-check these
clients by comparing pictures.** Read `of X`, the mirror-wide count no viewport can clip.

**THE PROCESS LESSON OF THIS WHOLE SESSION, and it fired three times:** *verify the instrument
before trusting what it says.* (1) `tui --frame` reported 0 glyphs for 9 real marks, silently.
(2) My hand-rolled sabotage loop reported **6/6 KILLED from a harness that failed on a clean
tree** — six false kills, caught only by running the control afterwards. (3) A build stamp taken
from `date` was ~114 min stale. **Run the control. Every time.** See [[sabotage-restore-trap]] for
the sibling lesson about restoring with `git checkout --`.

## 2026-08-27 - THE LOOK HALF OF 8.2 IS DEFERRED; A READOUT PASS CLOSES THE REST

Wolf, after driving the third build: *"ok well .. better .. so maybe it's ok at this point.. it
will get clearer with only real gfx.. now it's too confusing still to understand what happens."*
**Filed as a DEFERRAL, not a pass** — AC13's rendered half and AC19's reads-clearly half go to the
gfx pass; nothing is marked met on "better".

**The move that unblocked it, and it generalises:** when placeholder art blocks *judgement*, split
the ACs into what needs an eye and what needs an instrument. AC18's `tui` glyph counts and the
NFR6 fps readings are objective readouts that close regardless of how the client looks. Wolf took
that split immediately. **Do not let an art deferral swallow items that were never about art.**

**Round 3 findings** (before the deferral), both settled by MEASUREMENT against the generated
world rather than by argument — this is now the pattern that works with Wolf:
- The face-neighbour rule he had ruled the day before landed **100% of top-face hits and 8.5-11.8%
  of side-face hits**. His own ruling did not survive contact; presenting the number, not an
  opinion, got it re-ruled in one turn.
- **AC4's single-z rect kept a MEDIAN 19.4% of a 6x6 stockpile footprint** on natural terrain.
  That was a **SPEC defect, not a code defect**, and had been true since the AC was written.
- Said plainly to Wolf, and it mattered: **part of what he was seeing was the preview finally
  telling the truth.** The round-2 fix did not create the loss, it exposed it.

**AC18's own instrument was broken too, found by TESTING THE RECIPE rather than shipping it.**
`tui --frame` drew only what fit its viewport and painted the cursor/entity/item layers over
marks, so a 9-tile stockpile read 0 glyphs from one terminal and 7 from another **and said nothing
either time**. It now prints `marks: z N designations=D of X zones=Z of Y` with an INCOMPLETE note.
Generalise: **before handing Wolf a recipe, run its instruments here first** — this is the third
instrument in two days that reported success while capturing nothing.

## 2026-08-27 (later) - THE VEHICLE SESSION FOUND TWO MODES THAT HAD NEVER WORKED

Wolf ran it and reported, in his own words, "a bit fragile and confusing... sometimes it loses
dragged tile color... sometimes not colorize it at all... channelling, what should it do".
**That vague-sounding report was four real defects**, and the biggest was that **channel and
stockpile designation from the Bevy client were COMPLETELY INERT** - see
[[silent-sim-filter-trap]] for the mechanism and the rule that follows from it.

**Lesson about the report, not the code:** a "hard to say, feels fragile" from Wolf is worth
running to ground with a reproducer, not filing as a look complaint. He also proposed waiting for
readier gfx, and half of that was right - the campfire blowout genuinely blocks *observation* -
but none of the four defects was look-tuning and all four would have survived final art intact.
**Split the report: what needs art to judge, versus what is mechanism.** I did that split and it
was the correct call; had I accepted "wait for gfx", two dead modes would have shipped.

**What Wolf ruled**, and it is reusable: channel/stockpile target the **neighbour across the face
the ray entered**. This gave AC13's `Face` a second, behavioural consumer instead of a purely
decorative one.

**Process notes worth keeping:** the build-stamp step in my own runbook failed on FIRST USE - a
`date -u` typed ~114 min after the build - so the runbook now stamps from the artifact's mtime and
the last commit touching `crates/`, never a clock. And I destroyed my own uncommitted fix with a
sabotage restore, see [[sabotage-restore-trap]].

## 2026-08-27 - 8.2 VEHICLE RUNBOOK WRITTEN AHEAD OF THE SESSION; A CREDENTIAL LEAK CLOSED

Three commits, **not pushed** (`aca07be`, `bfac2c4`, `612cbdf`). 8.2 stays `in-progress`; nothing
here closes it, and a live gingerspice session is still the only thing that can.

**`.codex/` was NOT a cosmetic deferral and the review misjudged it.** The review filed it as a
one-line `.gitignore` nit on the reasoning "harmless, it's untracked". The directory holds
`.codex/auth.json`, a live credential, and `.gitignore`'s secret patterns (`*token*`, `*secret*`,
`.env*`) **do not match that filename** - one `git add -A` commits a working auth token. Closed.
Generalise: "untracked so harmless" is not a deferral rationale when the file is a credential;
that is the [[defer-vs-close-now]] shape, and I nearly repeated the review's call.

**Task 7's runbook is written BEFORE the session, which is how it differs from its model.** 7.2's
`task-6-vehicle-runbook.md` recorded commands that had run; this one cannot, since none of it
executes in a devpod. So every flag, threshold, glyph and colour was **read off the source at
`aca07be`** rather than carried forward - `--drag` and `--at-tick` did not exist when 7.2's was
written, and three of its flag rules came only from the 08-26 review. **A runbook copied forward
would have been wrong in at least five places.** Ones worth keeping:

- `--cursor` and `--drag` now `bail!` together (were silently one-ignored-while-asserted).
- `--drag clear` **asserts nothing about work** - `assert_drag_produced_work` has an empty arm for
  it, correctly, so a clear capture proves strictly less than the other three.
- A stockpile on non-standable ground is **dropped silently** by the sim; broken-path and
  correct-drop look identical. Same trap that gave 0 zone tiles at 7.2.
- Channels decay to **zero by ~114 ticks**, so `--at-tick` must stay LOW (3, not 100).
- **fps cannot be read from a capture** - capture mode forces the F3 overlay off
  (`force_capture_overlay_off`), deliberately. AC19's readings are interactive-only.
- Exhaustion check must be **run on purpose** (`--at-tick 100000 --frames 30`, expect `exit=1`,
  no PNG); every other step in the file is blind to AC16's negative half.

The two items carrying the session's weight, both patched, tested, and **never rendered**: the DDA
march's hit face (inverting X/Y left 149/149 green) and the slab's rotation onto the face normal
(deleting `.with_rotation` from both call sites left 149/149 green).

## 2026-08-26 - STORY 8.2 CODE REVIEW -> in-progress, PUSHED (no PR)

Four layers (Blind/Edge on Sonnet, both auditors on Opus), fresh context, **none a coverage hole**
- the first four-layer full house since build isolation landed. Nothing starved, nothing killed on
silence. See [[review-layer-reliability]].

**The finding that matters most for how this project reviews.** The inert-seam defect was NOT here
- the feature is genuinely wired end to end. It had RELOCATED one level out, in two directions at
once, which is [[verification-defect-relocates]] confirmed a second time:
- **into the instruments**: `--at-tick` exhaustion set `capture.failed` and wrote
  `AppExit::error()`, then exited **0**, because `app.run()`'s return was discarded and `AppExit`
  is not `#[must_use]` so clippy `-D warnings` stayed green. A `--drag` that designated nothing
  evaluated `0 == 0` and passed - AC15's non-zero check sat behind `--expect-work`, which the
  story's own recipe omits and which a dig-only drag CANNOT pass (it also demands `zones > 0`).
- **into the tests**: march face, slab rotation, 3 of the 4 modes in the story title, both Esc
  paths, hint bar and drag preview were each provable GREEN under sabotage. Only `Digit1` was ever
  pressed by any test.

**LESSON, and it generalises past this story: when a story ships a NEW INSTRUMENT, review the
instrument as adversarially as the feature.** Both HIGHs were in the detector, not the detected.
Neither was findable in a devpod - `gui` dies at winit init before reaching either path - so both
took a source trace, and a live vehicle run would have SILENTLY PASSED on both.

**`scripts/audit-mutations.py` was itself blind** - see [[stale-sabotage-literal]]. It only
inspected literals bound to a NAME, so rows passing text inline to `s.replace()` were skipped
entirely (7 of them), and one had already rotted with the script printing a clean all-clear over
it. Fixed + made cumulative (3-1 swaps through a temporary sentinel and must not be flagged). The
fix then caught 6-1's and 7-2's rows, broken by THIS session's own `capture.rs` edits - they would
have shipped rotten. Rows 351 -> 369.

Mutations 9 -> 27, **all 27 killed** (19 new + all 8 originals re-verified, because the patches
touched files the originals anchor into - do this every patch round). Tests 149 -> 165. Full gate
green. 5 commits pushed to `origin/8-2-designate-with-the-mouse`; **no PR by Wolf's explicit
instruction**. `.codex/` is still untracked and NOT git-ignored - deliberately not swept in.

**Accepted risk (Wolf):** `send_commands` blocks the Bevy `Update` schedule on a 30 s write
timeout. Daemon is localhost so the stall shape is not real today; REOPEN if `simd` ever runs
off-box.

## 2026-08-26 - STORY 8.2 CREATED (8.1 done + merged)

`8-2-designate-with-the-mouse.md` written on `main` at `cca118a`, gate GREEN at that baseline
before anything was written. Sprint-status flipped to ready-for-dev. NOT committed - unlike 8.1,
which committed the story onto its own branch first; if that matters for AC1's diff scope, cut the
branch before committing (see [[stacked-branch-ac-defect]]).

**Four rulings taken from Wolf AT CREATION rather than left to the dev pass**, recorded in a table
at the top of the story: (W1) press-drag-release, because the epic left the pattern to "testing in
this story" and no devpod can open a window to test it; (W2) mode keys are DIGITS 1/2/3/4 - the
TUI's `d` collides with camera yaw-right at `ingest.rs:519`, and digits leave every letter free;
(W3) 8.1's buried hover slab moves to the HIT FACE, since the DDA already knows the crossed axis;
(W4) M2-15's `--capture-at-tick` ships here.

**THE PREMISE CHECK CUT SCOPE, WHICH IS THE POINT OF DOING IT** (see [[epic-premises-go-stale]]).
The epic's AC2 reads as though 8.2 must build `simd`'s rect validation. It is ALREADY BUILT AND
TESTED - `rect_is_valid` at `simd/src/main.rs:714`, applied at `:677`, pinned by
`invalid_rects_are_logged_dropped_and_leave_the_client_connected` (`serve.rs:1322`) over both the
inverted-corner and two-z cases. The story inherits it and FORBIDS opening `crates/simd/`. Same
shape for absence-is-deletion: `client-core`'s `apply_delta` replaces the whole designation list
per delta (`lib.rs:99`), so AC10 asserts a round trip rather than rebuilding a mechanism.

**THE STRUCTURAL CHANGE AND WHY ITS AC IS WORDED THE WAY IT IS.** `gui` is receive-only - the
`TcpStream` is eaten by `BufReader` at `ingest.rs:104` and no write handle survives. 8.2 opens
that write half, and this is the exact seam class the project has now got wrong three times in a
row: 7.2's `--distance` parsed and never reached the camera; 8.1's `--cursor` did the same and
SURVIVED mutation round 1 with the whole suite green; 8.1's review then found the *call site* of
that fix was itself untested. Hence AC5: a test that starts at a mouse press and ends at bytes on
a real loopback socket. Same reasoning made the instrument script INPUT (`--drag` in viewport
pixels through the real press/drag/release path) rather than building a rect directly.

**Blast radius to watch at dev time:** widening `PickedTile` to carry the hit face touches ~a dozen
of 8.1's assertions (mitigated by a `PickedTile::tile()` accessor) AND reformats files 8.1's
mutation table pins - so Task 6 re-runs `audit-mutations.py` against 8.1's table too. At 8.1 a new
helper broke a row in *5.4's* table, in a file that story never opened (see [[stale-sabotage-literal]]).

**M2-7 is at its FIFTH occurrence and still unstarted** - no build script, no SHA stamp in `gui`,
verified again this session. Nothing automates the `gui.exe` rebuild; every vehicle session
re-litigates it by hand.

Create phase cost $14.00 / 121 turns / 15.2M processed, recorded as `phase=create`.


## 2026-08-25 - EPIC 8 OPENED, STORY 8.1 CREATED (uncommitted at hand-off)

`8-1-point-at-the-world.md` written, sprint-status flipped (epic-8 in-progress, 8.1
ready-for-dev, last_updated bumped in BOTH the metadata field and the header comment mirror,
which had drifted a day behind). All four files - story, sprint-status, the create-phase metrics
row and its cursor - committed on branch `8-1-point-at-the-world` at 44a7f21, deliberately NOT on
main, so the story-creation commit sits INSIDE the range AC1's diff-scope check reads (see
[[stacked-branch-ac-defect]]). Not pushed. Story's `baseline_commit` is 32e6933, one behind the
branch tip, which is correct.

**WOLF'S RULING - THE PICKING RAY COMES FROM THE RENDERING CAMERA.** `Camera::viewport_to_world`
plus a DDA march, not an inverse of gui's hand-rolled `project_render_point`. The rejected
option would have been the THIRD copy of the frustum math (`camera.rs:76`, `atmosphere.rs:213`,
and the new one), and `camera.rs:30` hardcodes `BOOT_ASPECT_RATIO = 16/9` while the real
viewport does not have to. Cost accepted: `MinimalPlugins` runs no `camera_system` and no
`TransformPlugin`, so headless tests hand-build `camera.computed.clip_from_view` and
`GlobalTransform`. Bevy's own unit test is the pattern.

**THE TRAP THAT WOULD HAVE SHIPPED A HALF-WRONG FEATURE, and it is a general one:** a transform
whose name reads like a total function is not one. `render_to_world` is `as i32` TRUNCATION and
its doc says "voxel-aligned Bevy position" - but `Cuboid::default()` is CENTRED on its
translation, so voxel p occupies render-space p +/- 0.5, and feeding it a raw ray-hit point
resolves HALF OF EVERY VOXEL to its neighbour. Recorded as D2 in the story. Generalisation worth
carrying: **before making a test-only helper into production code, check what its callers were
allowed to assume.** `render_to_world` had ZERO production callers in three epics - it existed
only as a test oracle and a round-trip pin, so nothing had ever exercised its rounding
semantics. The epic called it "the existing transform", which is true and misleading.

**8.1 IS THE FIRST M2 STORY WHOSE EPIC PREMISES ALL HELD** (4 of 4), against a project record of
5-for-5 wrong. The reason is visible: M2-4 corrected 8.1's text at the retrospective, BEFORE the
story was written. Premise-falsification moving one step earlier is what changed the outcome -
see [[epic-premises-go-stale]].

**M2-4 MISSED THE COMPANION DOC.** `docs/architecture.md:32` and `:127-129` still say gui "runs
via WSLg" and state the NFR6 bar against "the WSLg devpod". The spine and epics.md were both
corrected. Flagged in 8.1's Dev Notes, not fixed - it is outside that story's diff.

Judgement recorded for the next planner: **do NOT collapse 8.1 into 8.2.** The epic offers the
collapse "if picking proves easy" and it is not - there is no ray, cursor, `Window` or
`MouseButton` code anywhere in gui, and the headless harness has neither a camera nor transform
propagation. M2 stays at 11.

## 2026-08-24 - FORGE-PROCESS SYNCED 1.1.0 -> 1.2.0, T2 CLOSED (PR #33)

`--mark` shipped in the FORGE (1.2.0, `dbc2fe6`, PR #14) and was **invisible here**: frostvein's
copy of `session_tokens.py` was byte-identical to forge commit `6e000bb`, the 1.1.0 version with
no `--mark` at all. Pulled down, verified in frostvein's own copy (61 tests green, flag in
`--help`), check now reports in sync at 1.2.0. See [[forge-sync-staleness]].

**T2's stated success criterion was already met before the fix** - `delta_since_cursor()` has
billed per-transcript deltas since Epic 3. The real defect was one level down: the cursor only
advanced when a ROW was written, so any window not bookended by two rows was swept into the next
row taken. That is the [[verification-defect-relocates]] pattern again - closing on the stated
criterion would have recorded a fix for a defect that was never the defect.

**It sat open three epics because it was mis-owned**, not deprioritised: a forge-owned FILE entry
cannot be actioned by a frostvein owner.

The METRIC RULE was hand-merged into the three adapted workflow TEMPLATEs and `ack`ed, so the flag
is INVOKED, not merely available. **Epic 8 is the first epic to record under it.** 7-2's mixed
`live-gate` window ($151.81 = review-patch + live-gate) is NOT fixed retroactively; its caveat
stands. Open action items 13 -> 12. **MERGED 2026-08-24 (PR #33, merge `32e6933`)** - local main synced, full gate re-run GREEN on merged main, working tree clean, branch deleted. Post-merge branch trap fired again as usual, see [[post-merge-branch-trap]].

## 2026-08-23 (later) - BOTH ROOT-CAUSE FIXES LANDED AND MERGED (PR #32, merge d2dc946)

Wolf: "fix both". Done, committed, pushed, PR body updated.

**M2-1 - the inert-mechanism class is closed at the root.** `client_systems()` and
`capture_systems()` extracted from `run()`, following the `projection_systems` precedent earlier
reviews set, so the suite drives THE SAME registration the live client uses. `CaptureDistance` made
`pub` - it is the `--distance` seam 7.2 found inert. Seven tests, **every assertion an observable
effect, never "is it registered"** - a registration assertion would be the very vacuity we keep
re-finding. New table `m2-1-live-app-systems.sh`: **9/9 KILLED**. Deleting any system from run()'s
tuples now turns the suite RED. See [[verification-defect-relocates]].

**M2-17 - the save_load vacuity, fixed with a mechanism the game itself taught us.** A dig buried in
rock is unreachable BY CONSTRUCTION (`work_positions` needs a dwarf on a standable tile at the same
z, orthogonally adjacent) so its designation is retried forever and never dies - the permanent blue
field Wolf saw at the working zoom, turned into a test fixture. Plus a guard asserting designations
are non-empty AT the save point. **ASSERT THE STATE THE COVERAGE DEPENDS ON; do not trust that it
holds.** Both rows now KILL.

**Mutation baseline VERIFIED, not inferred: 333 rows / 322 KILLED / 11 dead (3.3%)**, down from
13 of 324. The second full re-run earned its cost - `ingest.rs` was refactored AND reformatted, and
five older tables sabotage it; every gui table (5-3, 5-4, 6-1, 6-2, 7-2) still 100%. Claiming that
without measuring would have been the exact mistake this retro spent the day cataloguing.

**Left open deliberately - M2-18:** `apply_command skips bounds clipping` survives because the clip
is REDUNDANT FOR CORRECTNESS (out-of-bounds positions are dropped downstream) but LOAD-BEARING FOR
TERMINATION. Catching it needs a design decision, not a line: a test that fails by hanging is a
worse test.

**Gate is now two-tier** (M2-13): `gate.sh --fast` on pre-commit at 7-8s, full 67s on a new pre-push
hook. `serve.rs` alone is 88% of the gate and is slow BY NATURE (real daemon, real socket, real
wall-clock) - it moved, it was not weakened. The fast tier prints GATE GREEN (FAST) and names its
coverage hole so it cannot be mistaken for a full green. See [[review-cost-economics]].

**Suite 364 -> 371. MERGED 2026-08-23 16:05 UTC**, all six commits on main, full gate re-run GREEN on merged main, working tree clean. Post-merge branch trap fired as usual - the working copy landed on main by itself; see [[post-merge-branch-trap]].


## 2026-08-23 - M2-TO-DATE RETROSPECTIVE (Epics 5+6+7 in one)

Wolf ruled the scope: **ONE retro across all eight M2 stories**, filed as Epic 5's record
(`epic-5-retro-2026-08-23.md`), because the repeating defect classes only read as patterns across
epics. **Epic 4's retro was a DELIBERATE SKIP** - the epic was ditched when 4.1a proved the wow
effect unreachable in a TUI, and the retro went with it. Now annotated in sprint-status so nobody
re-flags it as drift (I did, at the start of this session). See [[defer-vs-close-now]].

**The headline finding, and it has a paper trail.** At 5.4's review, "the live `App` built by
`run()` has no test of any kind" was filed `[feature/MED]` and **deferred** - and that MED defer
became the **top-severity finding in the next four consecutive stories**: 6.1 (both projections
deleted from the live tuple, 54/54 green), 6.1's review (three drive-line deletions each killed
wow beat 2, 57/57 green), 6.2 (`accumulate_motion` had zero test callers and had never executed
anywhere), 7.1 (the whole on-screen readout and the `--z` pin inert), 7.2 (`--distance` never
reached the camera rig). **Five of eight stories. Severity at defer time is a prediction, and this
one was wrong by four stories.** Reinforces [[verification-defect-relocates]] - the defect
relocates one level down each time it is closed. Wolf ruled: attack the root (M2-1), not the
symptom a sixth time.

**Cost, the first fully-counted epic set:** $1,070.07 / 8,931 turns / 8 stories = $133.76 each,
median $98.23. **Do not trend against Epic 3's $45.52** - T1's own result note forbids it. **5.4
alone is 34% of the milestone** ($359.85); its review-patch phase ($274.75) cost more than all of
Epic 6 and about as much as all of Epic 7. See [[review-cost-economics]],
[[metrics-attribution-traps]].

**Wolf's seven rulings** -> M2-1 make `run()`'s App testable (before 8.1); M2-2 keep the 5.4
measurement method, retire the look-tuning spend ([[art-gates-visual-judgement]]); M2-3 **silence
overrides the 45-min layer ceiling** (contradiction open since 5.1, unruled through 7 reviews -
[[review-layer-reliability]]); M2-4 **redefine the NFR6 bar** against the real vehicle and fix
Epic 8's WSLg ACs; M2-5 R1 gets a real M2 mapping, pure logic / shell
([[concurrent-review-tooling-hazards]]); M2-6 reaper on review teardown (~92GB/review); M2-7 kill
the stale-binary trap both ways (script+runbook AND a git-SHA startup stamp).

**Caught at the retro, before it cost anything:** story 8.1's AC reads *"Given the full world with
picking active on the **WSLg devpod**, Then NFR6 still holds"* - false since 5.3
([[devpod-no-graphics-userspace]]). Would have been the **4th consecutive epic** with wrong
premises; M2 went 3-for-3 and the project is now **5-for-5**. See [[epic-premises-go-stale]].

**E4-P1 closed on evidence, not expiry:** both candidate fixes were applied silently during M2 -
outcome-not-mechanism reached the requirements (FR31/FR33), and cheap-artifact-before-build became
UX-DR22's sign-off gate, which fired repeatedly. **Caveat kept:** 5.4's approved artifact drew
geometry nobody was tasked to build, so the AC19 comparison failed by construction - the 4.1a class
reappearing *inside the fix for the 4.1a class*.

**P2 is a solved problem:** 4-of-4 layers, zero coverage holes, on **all eight** M2 reviews.

**Still open:** T2 (`--mark`), the gfx/lighting pass with measured targets, the sim-side
unreachable-designation question, and M2-12 (the self-gate's value is unmeasured after 3 epics -
zero usable passes at 5.4, a declared coverage hole at 6.1).


## 2026-08-23 — 7.2 SIGNED OFF, EPIC 7 CLOSED

Wolf viewed the vehicle frames live and signed off AC17. **All 17 ACs met; story `done`, `epic-7`
`done`.** He ruled it straight to done rather than back through code review — the 2026-08-21
four-layer review stands and no `crates/` file changed after it, so a second review would have
re-read an unchanged diff. No code was written this session.

**His seven rulings** (the six parked questions plus one of his own), all recorded in the story's
Dev Agent Record: working-zoom blow-out DEFERRED to the gfx pass (ruling (d) not revisited); the
blue dig sheet STANDS as honest; gutter closed; three marks confirmed (teal "a bit lame" but fine
pre-art); vista camp still too blown, also deferred to gfx; AC8's four-noun bar MET without
reservation.

**The Q3 puzzle he raised was sim truth, not a rendering defect** — worth carrying because it will
recur at every future mark story. A designation dies with its job (`sim-core/src/lib.rs:883,898`,
the same line for "dug out" as for "cancelled"). Dig needs a dwarf on a standable tile at the same
z orthogonally adjacent; channel needs the dwarf to stand ON the designated tile. Neither is
satisfiable for a buried dig or a mid-air channel, and **the sim never abandons an impossible job**
— it just sets `retry_after` forever (`lib.rs:423-428`). So a permanent blue field = work no dwarf
can reach, and a floating violet band = work no dwarf can ever do. 7.2's buried-dig promotion is
what made a previously invisible failure visible.

**Two items carried open, neither a 7.2 defect:** (1) the lighting/gfx pass, with concrete targets
recorded — ground median 231 on the cut floor, mark polarity inverted at the working zoom, and
AC9's band SKIPPING below the world top which is why no instrument caught it; (2) sim-side, a later
epic — impossible orders render identically to pending ones.

**Cost: $241.07 for the story.** See [[review-cost-economics]]; the `live-gate` ledger row is a
mixed window and cannot be quoted cleanly, which is the concrete case for Epic 3's open item T2.

**Working tree:** branch `7-2-vehicle-evidence`, four modified files (story, sprint-status, metrics
ledger + cursors), UNCOMMITTED as of session end unless Wolf said otherwise. See
[[post-merge-branch-trap]].

## 2026-08-22 — both graphics calls closed, the mutation tables un-rotted, everything merged

**State:** main = `427aeac`, clean, all three PRs merged. `7-2-read-the-working-zoom` and
`campfire-and-mutation-table-audit` can be deleted whenever. Sprint: 7.2 `in-progress`, epic-7
retro `optional` and NOT run, epic-8 `backlog`.

**Wolf ruled two open look questions from a phone**, both closed and verified:
campfire base 32M -> 25M keeping 6.1's ±40% breathing (peak back on 5.4's approved 35.52M — this
closes 6.2's carried-open "camp is too blown out"), and the zone mark darker/colder
`(120,206,196) -> (40,120,150)`.

**THE BIG SYSTEMIC FIND — see [[stale-sabotage-literal]] for the full account.** 36 mutation rows
across 9 tables pinned nothing; `scripts/audit-mutations.py` is now in `gate.sh` and catches three
rot shapes statically. Auditing is NOT verifying: running the 45 changed rows caught three repairs
that applied without killing.

**STACKED-PR MERGE TRAP, and it will recur because every M2 story is stacked.** PR #30 was based on
the 7.2 branch (it edited `zone_color`, introduced by 7.2, so it could not branch off main).
Merging it landed the work ON THE 7.2 BRANCH, not on main — main silently ended up without it, and
`git pull` on main said "Already up to date". Cost a third bookkeeping PR (#31).
**The fix is `gh pr edit <n> --base main`, run AFTER the base PR merges and BEFORE merging the
stacked one.** Better habit: do not open the stacked PR until its base has merged. Related:
[[post-merge-branch-trap]].

**Carrying into Epic 8:** verify its technical premises before writing any story — it asserts NFR6
holds "on the WSLg devpod", which is false ([[devpod-no-graphics-userspace]]), and
[[epic-premises-go-stale]] records 2 of 2 checked premises being wrong. Epic 8 also has a written
CUT option (drop 8.1/8.2, TUI keeps designations) that only works because 7.2 landed. And
[[art-gates-visual-judgement]] now governs how much look-work is worth doing.

## 2026-08-21 (late) — 7.2 reviewed and pushed; retro deferred; Wolf is travelling

**State:** PR #29 open on `7-2-read-the-working-zoom` (2 commits on top of the 12), **not merged** —
Wolf merges himself. Story `in-progress`, not done. Sprint-status synced.

**The review's headline finding, worth carrying because no test could see it.** The story's own
prescribed capture photographed an EMPTY SITE and exited 0. A dig slab sits at `z+0.54`, but the
slice draws every solid tile *at the cut* as a full cube spanning `[z-0.5, z+0.5]` regardless of
exposure, so a dig with rock above it was sealed inside opaque geometry — and the dwarves dig the
*reachable* tiles first, so the marks that survive a capture window are exactly the buried ones.
Measured live: 25 of 79 visible at t+2, **0 of 50 from t+102 on**, while the instrument correctly
printed `designations=50`. **The counter was not lying — all 50 were projected.** This is
[[live-gate-rule]] and [[verification-defect-relocates]] at once: an instrument can be perfectly
correct about the thing it measures and still certify a frame that shows nothing.

**Two seams were sabotage-proved INERT with the whole suite green**, each caught by more than one
layer: `--distance` parsed, validated, then never reached the camera rig — while its only test was
*named* for reaching the camera setup; and AC10's restyle was pinned on a bookkeeping component, not
the style, with the mutation row sabotaging that same wrong branch. **Vacuity relocates**: the
colour-table vacuity fixed before review reappeared in three new places. Check the *name* of a test
against what it asserts — two of this review's findings were visible from the name alone.

**New defect class — CROSS-CLIENT COLOUR COLLISION, no rule existed for it.** `gui`'s DIG blue
shipped byte-identical to the TUI's **CHANNEL** blue, on the two windows Wolf reads side by side.
Breaking with the TUI was a deliberate ruling; landing on a *different order's* colour was not. The
colour test now carries a cross-client floor. Worth generalising: any second client rendering the
same domain needs its palette checked against the first, not only against its own background.

**Process notes.** R1's territory split is stated in CRATES and this was a gui-only story, so both
hunters would have had empty territories — split them by SEAM within `crates/gui` instead. Flag for
the retro: R1 may need re-stating in terms of seams. Review cost **$46.60 / 527 turns**, above Epic
3's $45.52 baseline, so the split bought no measured saving here — but it was the first review in a
while with four live layers and zero coverage holes, 2 of 17 findings converging.

**Watch for next session:** Epic 8's text asserts NFR6 holds "on the WSLg devpod" — false, see
[[devpod-no-graphics-userspace]], and [[epic-premises-go-stale]] says verify every premise at story
creation. Epic 8 also carries a written cut option (drop 8.1/8.2, TUI keeps designations) that only
works *because* 7.2 landed.

## 2026-08-21 — 7.2's headless half, and three defects the dev agent did not find

Tasks 1-5 done across two Codex runs (terra/high, $5.51, **13 quota points — the weekly window is
now at 90%**). Gate green cold, 359 tests (from 348), AC3 empty, sabotage **10/10**. What matters:

1. **Run one committed NOTHING** — Tasks 1-3 implemented and left entirely staged when its window
   closed. A dead run with no recovery point is exactly what the cadence floor exists to prevent,
   and the floor was in the prompt. Restating it is not sufficient; **check `git log` on handback**.
2. **The stale-sabotage-literal class, 3rd instance** ([[stale-sabotage-literal]]) — and the first
   where the mutation and the code that outdated it were written in the *same session*. The agent's
   own final refactors moved the lines two rows targeted, the table was never re-run, and the record
   asserted "all eight KILLED" — true when it ran, false when written. **A table is evidence only as
   of its last run: re-run it after the last REFACTOR, not the last FEATURE.**
3. **AC4/AC5 were met only vacuously** — the colour test asserted mere *inequality* for an AC
   demanding *visually* distinguishable. Measured, channel sat **16 RGB units** from
   `Material::Snow` and zone 22 from `foliage_snow_color`, which the terrain list did not even
   include. Two of three marks were near-neighbours of the surfaces they are drawn on. Now a
   40-unit separation floor, proven able to go red. Same family as
   [[self-referential-test-antipattern]]: the assertion could not fail for the property it named.

**Generalisation worth keeping:** all three were found by *measuring* what the record asserted, not
by reading it. The agent was honest throughout (it reported its own incompleteness both runs, and
refused to claim a gate it had not finished) — honesty is not the same as correctness.

**Also found at 7.2, reported not fixed:** the slice readout's em-dash (`slice.rs:59`) has no glyph
in the vehicle's font and draws as an empty box — in `7-1-slice.png` and every capture since.

**AC9's recipe was dead as written and no agent would have caught it at run time:** the 08-20 fix
scoped the warm/ground band to the world top, and both of 7.2's prescribed captures pin `--z 10`,
where the assertions are *skipped*. Wolf ruled the vista capture moves to full depth. Note the
dev-story workflow may not edit a story's Verification section, so the corrected recipe lives in the
sign-off artifact and Dev Agent Record — **Task 6's runbook must carry it or it will be lost.**

## 2026-08-20 — the vehicle session that closed Epic 6

All three stories signed off in one sitting on one binary. **Three defects were found by Wolf's eye
that the entire suite was green on**, which is the single most important fact this session produced:

1. **6.1** — stone items spawned at `scale 1.0`: a terrain-sized block standing in the tile it was
   dug from, so a dug tile visually REFILLED and a worked face read as untouched rock. It also
   geometrically enclosed AC8's debris chips, which could therefore never be seen. Fixed at
   `STONE_ITEM_SCALE = 0.4` (the largest scale keeping all four chips outside the cube).
2. **7.1** — a FAILING capture wrote no PNG at all: `save_to_disk` and the range checks were two
   observers on one event and Bevy ran validation first. The run whose frame most needed looking at
   produced no frame.
3. **7.1** — 5.4's range band was judging a scene it was never calibrated against: at a cut the
   sample window shows interior rock, not sky-lit snow, so z 9 read 67 against the 70 floor and
   panicked on a picture that was fine. Same correction the 08-19 review made one assertion higher
   (lanterns) and stopped short of — see [[verification-defect-relocates]].

**The pattern to carry forward:** every one of these passed every instrument because the assertions
were about *presence*, not *size, order, or scope*. See [[live-gate-rule]] — unit-green is not
feature-proof, and this session is the strongest evidence yet.

**Carried open past sign-off, recorded not blurred:** the campfire still reads blown at full depth
(diagnosis: 04e6de5 raised its amplitude 0.11->0.40, peaking at 44.8M, 40% above the value 5.4 sized
against the approved artifact — NOT 6.2's lanterns); 7.1's AC10 surface/underground ruling never
given; `,`/`.` still PROVISIONAL with the mousewheel confirmed unclaimed in code; 6.1's AC15 closed
on PRE-FIX evidence after a fresh measurement was offered and declined.

**STACKED-PR MERGE TRAP, cost a lost merge 2026-08-20:** merging a stacked PR whose base branch was
just merged and deleted does NOT reach main. #27 (7.1 -> 6-2) merged 22 seconds AFTER #26 (6-2 ->
main), so 7.1's commits landed on a branch already merged and main never saw them -- while GitHub
reported #27 as "merged". Deleting a base branch also auto-CLOSES the PR stacked on it (#26), and it
cannot be reopened until the branch exists again: push the base commit back to recreate the branch,
reopen, retarget to main, delete the branch again. **Retarget every stacked PR to main BEFORE
merging anything.** Recovery for the lost one was a fresh PR (#28) from the same branch, no rewrite.

**Process finding: 6.2's entire code-review round was committed onto 7.1's branch, not its own.**
So PR #26 is 6.2's first pass only and the story is complete only when #27 merges. Both PR bodies say
so. Check which branch you are on before committing a story's follow-up work.



## 2026-08-19 — story 7.1 code review (4 layers, fresh context, $39.31 / 467 turns)

**No layer was a coverage hole — a first this epic.** The cut itself is PROVEN: Blind Hunter fuzzed
the real projection over ~1.1M position checks + 4,000 ECS steps, 0 failures; Feature Auditor drove
a LIVE simd and measured 15,316 of 16,071 cut-face tiles supplied only by the `z == level` arm. All
12 live-path hops WIRED, no dead capability.

**EVERY defect was in the observability layer, never the feature.** That clustering is the finding.
19 findings: 3 decision (Wolf ruled all three same session), 10 patch, 5 defer, 1 dismissed. All 13
patches applied same session, batched, ONE verification pass. Suite 84→90, sabotage table 7→13,
13/13 KILLED, GATE GREEN observed.

Two HIGHs, both the 6.1 untested-drive-line class recurring: the ENTIRE on-screen readout (AC9's
mechanism) and the `--z` PIN could each be deleted or made inert with a fully green suite. The story
text had NAMED the mitigation ("any new system goes in `projection_systems` or it is invisible to
the suite") and it was skipped anyway — writing the rule down did not make it happen.

**Cost facts worth keeping:** 45.1% of all tokens were SUBAGENT transcripts (so every pre-fan-out
ledger figure is a large undercount — this review would have booked ~$22 under the old method, which
is where that falsified "$22 floor" came from), and 95.8% was cache read, confirming review is
expensive because it RE-READS, not because it thinks.

See [[stacked-branch-ac-defect]] for the AC-text defect this review generalised, and
[[review-cost-economics]], [[self-referential-test-antipattern]], [[live-gate-rule]].

**Frostvein** = a Dwarf Fortress–inspired voxel colony sim: pure-Rust `sim-core` +
`simd` daemon (tick loop, TCP, NDJSON protocol) + `tui` client. Milestone 1 = the
walking skeleton (dig → path → dig tile → haul to stockpile, live in the TUI).
Everything decided is in `docs/project-brief.md` + `docs/technical-preferences.md`
(binding — anti-overengineering is policy).

**Arrangement (Wolf, 2026-08-01):** hosted in the Nidavellir forge at
`/workspace/projects/frostvein`, but OWNS its process — own git repo
(`jeicei75/frostvein`), own BMad 6.10 install, own memory (this store), sessions
launched FROM this dir. Not Asgard-related. Promotion to its own devpod when it
needs services / deps conflict / Wolf wants isolation — the repo lifts out wholesale.

**State 2026-08-01:** repo live + pushed; BMad installed; customizations ported from
the forge (delegated Codex dev with cargo gates + offline `cargo fetch` prep, Feature
Auditor review layer, metric ledger via `_bmad/scripts/session_tokens.py`);
`scripts/codex-handoff.sh` local copy (-C this repo). NO code yet, NO Cargo workspace
yet. **PRD DONE (2026-08-01, status final):**
`_bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-01/` (prd.md + addendum.md
+ .memlog.md). Product-framed, M1 = phase one, 26 FRs / 4 NFRs / 6 tagged assumptions
(architecture confirms-or-overrides via the Assumptions index). Notable calls beyond
the brief: ice/snow terrain materials (NOT simulated processes), boss-not-remote-control
feel (seeded reaction delays + idle wander), cancel-designation in M1, protocol
world-not-game principle, LLM-whimsy sidecar + Asgard-adapter ideas parked in addendum
with triggers. **ARCHITECTURE DONE (2026-08-01, status final):** spine AD-1..AD-12 + 10-min
`docs/architecture.md` — details in [[frostvein-architecture]]. **EPICS/STORIES DONE
(2026-08-01):** `_bmad-output/planning-artifacts/epics.md` — 4 epics, 11 stories
(Frozen World on Screen 3 · World Breathes 4 · Boss Gives Orders 3 · Raycast 3D 1),
all 26 FRs mapped, validated. **Wolf overrides during story design:** FR24 raycast is
now FIRM phase-one scope (PRD's may-slip clause overridden 2026-08-01; cut list now
starts at FR16) — PRD/spine text NOT yet amended to match. Keymap additions agreed:
`S`/`L` save/load, `v` 2D↔3D view toggle (letters over Tab for tmux safety).
**SPRINT PLANNING DONE + STORY 1.1 READY (2026-08-02):**
`_bmad-output/implementation-artifacts/sprint-status.yaml` tracks 11 stories; `epic-1` is
`in-progress` and `1-1-a-seeded-frozen-world-exists` is `ready-for-dev` (story file beside
it; create cost ≈$19.69). Repo still has NO Rust code — 1.1 writes the first line, scaffolding
the 4-crate workspace + seeded worldgen. Toolchain now installed: see [[frostvein-toolchain]].
Wolf approved three authoring calls in 1.1 that later stories inherit: `Tile` is a
payload enum (`Empty | Solid(Material) | Ramp(Material)`), coordinates are `i32` while dims
are `u32` (stops A* underflow in 3.2), and `simd`'s stub `main()` actually generates+prints a
world so both dependency edges are exercised, not merely declared.

**STORY 1.1 DONE + PR OPEN (2026-08-02):** first Rust code exists. 4-crate workspace,
seeded ChaCha8 worldgen (hand-rolled value noise), 5 dwarves, 6 integration tests; gate
green from a clean checkout. Branch `1-1-a-seeded-frozen-world-exists`, 9 commits, pushed;
**PR #1 open against main — Wolf merges himself.** Dev delegated to Codex ($2.28); review
by Claude, 4 layers ($57.37 — a 25x asymmetry Wolf should weigh before 1.2). Review verdict:
implementation correct, TESTS were weak — 3 mutations (seed-independent terrain, inverted
layering, allocator bypass) passed the original suite and now fail it; a 4th (spawn ignoring
the RNG draw) is knowingly left open in
`_bmad-output/implementation-artifacts/deferred-work.md`, which is now the standing home for
review deferrals. Patches preserved determinism byte-for-byte (verified seeds 0/42/7777).

**STORY 1.2 DONE + MERGED (2026-08-02).** PR #2 merged by Wolf; branch deleted local+remote;
`main` = `ebd27dd`, gate green, 20 tests. `protocol` now owns all wire shapes; `simd` serves a
~6.9 MB NDJSON snapshot on connect from `127.0.0.1:7373` (optional positional port arg, `0` =
OS-assigned). Dev delegated to Codex across TWO runs ($1.38 + $1.56) — see
[[codex-delegation-runbook]] for why. Review $75.97, 4 layers, 13 findings fixed / 7 dismissed;
the central catch is in [[self-referential-test-antipattern]]. Wolf overrode my "defer, YAGNI"
recommendation and had me harden the daemon now: accept backoff, 64 KiB line cap, log
truncation, 30 s write timeout, 256-connection cap, non-panicking spawn+argv, lossy inbound
decode. Two knowingly-open gaps, both `// NOTE:`-ed in source for Story 2.1: (a) `tick: 0` vs
`world.tick()` is untestable until tick advances; (b) read EOF closes the connection — half-close
and full close are indistinguishable there, and 1.2 has no write path to detect a dead peer, so
"keep it open" would park a thread per normal disconnect. 2.1's delta writes supply that.

**STORY 1.3 DONE + MERGED (2026-08-03).** PR #3 merged; `main` = `a76a359`. The real `tui`
exists: palette data table, pure `render`, one-buffer ANSI frames, `--frame` no-TTY path,
`TerminalGuard`. Wolf signed off FR23 (icy-grim) from a live interactive run — the last AC no
agent could close. Review: 10 patches, 2 Wolf decisions (Ctrl-C quits immediately; `q` prompt
replaces the whole status line), 3 deferred. The untracked-docs problem is RESOLVED — `docs/`,
`CLAUDE.md` and all of `_bmad-output/` are tracked as of `5675630`.

**EPIC 1 DONE + RETRO DONE (2026-08-03).** `sprint-status.yaml`: `epic-1: done`,
`epic-1-retrospective: done`, plus a new `action_items:` section with 13 open entries.
Retro doc: `_bmad-output/implementation-artifacts/epic-1-retro-2026-08-03.md`. Gate
independently re-verified at retro time: fmt/clippy clean, **39 tests green**. Epic cost
$256.11 — the review-economics decisions Wolf made there are in [[review-cost-economics]];
the sandbox reversal is in [[codex-delegation-runbook]].

**PROCESS CHANGES WIRED (2026-08-03)** — see [[codex-delegation-runbook]] for detail:
`AGENTS.md` now exists (Codex had never read frostvein's rules), **`scripts/gate.sh` is the
gate** (fmt/clippy/test/tui-edge probe, exits non-zero, pre-commit hook via
`git config core.hooksPath .githooks` — verified red under 3 sabotages), and
`codex-handoff.sh` sets `network_access=true` (verified live: `BIND_OK ('127.0.0.1', …)`).

**EPIC 2 DEPENDENCY SWEEP + STORY 2.1 READY (2026-08-03, `190e12d`).** `epic-2: in-progress`,
`2-1-the-world-runs-on-its-own-clock: ready-for-dev`. Both retro blockers resolved IN the
story: TUI uses a **reader thread** (not `event::poll` + socket timeout — `read_line`'s
buffer is documented unspecified on error, so timeout-driven line framing is unsound), and
the **tick loop owns the `World` outright**, admitting sockets over a channel so no `Mutex`
exists; bounded 16-slot per-client queue, `try_send` failure drops the client. That write
path also closes both 1.2 deferrals (read-EOF no longer tears down a half-closed client;
dead peers reclaimed). Wolf's call on AD-8: build `set_tile` + dirty set now and **prove it
with AC4** (direct `set_tile` → appears in one delta, gone the next) rather than ship it
inert. bevy_ecs 0.19 skeleton was **compile-probed**, not recalled — trait is
`IntoScheduleConfigs` (NOT `IntoSystemConfigs`), `#[derive(Resource)]` covers its `Component`
supertrait.

Sweep also corrected three things: **2.2 is a WIRE CHANGE** (job state adds a field to
`protocol::Entity`, moving protocol + bridge + 1.2's JSON literals + tui together) and is
where AD-7's purpose-named RNG streams are born — `World` retains no RNG state at all today,
so the deferred RNG-coupling item moved from 2.4 to **2.2**; **2.3 is the first story where
the client writes anything** (protocol has zero command types; the daemon treats every
inbound line as unrecognized). Notes are inline in `epics.md`.

**STORY 2.1 DEV DONE → REVIEW (2026-08-03).** Branch `2-1-the-world-runs-on-its-own-clock`,
7 Völundr commits, nothing pushed. Codex dev cost **$5.29 / 66 turns** — the first run with
AGENTS.md, the enforced `scripts/gate.sh` and network access all in play, and it shows: gate
GREEN on my independent re-run, **33 workspace tests**, loopback e2e actually executed by
Codex (zero-client ticking, late-client snapshot `tick > 0`, half-close, queue eviction,
one-client-drop isolation), and **ten pasted sabotage RED outputs** including both constants
(`TICK_PERIOD`, `CLIENT_QUEUE`) and the `tick: 0` mutation — Epic 1's one irreducible
survivor, now killed by `bridge::tests::snapshot_uses_the_current_world_tick`. One honest
self-report: `codex review --base main` could NOT run — its in-process app-server dies with
`Read-only file system (os error 30)` under `-s workspace-write`. No workaround attempted
(correct per AGENTS.md rule 3). **That action item is blocked on the sandbox, not on
intent** — fixing it means a writable app-server dir in `codex-handoff.sh`. Codex's own
manual review still caught and replaced a weak queue-capacity test that never exercised
registry removal.

**STORY 2.1 REVIEWED + DONE (2026-08-03).** 4 layers; **the two Opus layers RAN THE BINARIES**
and that is where every finding of substance came from. Feature Auditor traced all 16 hops and
found no stranded capability (daemon alone → tick 10,267; live TUI 8473→8532 at 10 Hz while keys
responded), so **AC10 closed inside review** rather than needing Wolf — a first. Acceptance
Auditor measured two real defects: an 8-client connect burst stalling the timestep **1432 ms**
(snapshot re-encoded per pending client on the tick thread), and AC9 only half-met — an evicted
client stayed ESTAB **~60 s**, squatting a `MAX_CONNECTIONS` slot, because eviction dropped only
the `SyncSender` while `serve` sat in `write_all`. Outcome: 2 decisions, **9 patches applied**,
2 deferred, 4 dismissed. Gate green, **57 tests**, 4 patch commits (branch has 11). Review cost
**$66.19** — but that figure also swallows this session's dev orchestration (the known
`session_tokens` mixed-session defect), and Epic 1's per-story reviews were $57–76, so **Lever A
did not visibly cut cost**; 25M cache-read tokens still dominate.

**Two rulings Wolf made at review (2026-08-03), both now load-bearing:** (1) the dirty set is
**per-drain, not per-tick** — only `drain_dirty` empties it, `step()` never does; AC2's wording
was amended and it is pinned by `stepping_does_not_clear_the_dirty_set`. A clear-system in the
schedule was rejected as a second mechanism whose correctness would hang on `.chain()` order.
(2) The `tui` binary now gets a **spawned-binary test** (`crates/tui/tests/client.rs`) driven by
a stub server, plus a `--frames N` headless mode that runs the REAL reader-thread loop — because
`--frame` returns before the reader thread is spawned and could never show a climbing tick.

**THE LESSON OF 2.1'S REVIEW — I shipped the antipattern myself.** My new socket-shutdown
assertion **survived its mutation**: dropping the evicted `Client` dropped its `TcpStream`, which
closed the socket unaided, so the peer saw EOF with or without the fix. Production differs —
`serve` still holds the original handle. Fixed by holding a second live clone in the test. See
[[self-referential-test-antipattern]]: this class has now hit 1.1, 1.2, 1.3 **and the reviewer**.
Batched mutation testing into one apply/revert script (retro action item) — 5 mutations, one turn.

**STATE AT CONTEXT-CLEAR (2026-08-03, end of session).** `main` = PR #4 (story 2.1) + PR #5
(process tooling + the forge's pricing fix), both merged by Wolf. **PR #6 is OPEN and
unmerged** — `AGENTS.md` self-gate rule, the `gate.sh` shared-target guard, and a rewritten
README. Working folder is on branch `agents-self-gate-rule`, clean, nothing local-only.
Wolf ran the TUI himself and confirmed the tick climbing — the human sign-off on 2.1's
observable outcome. **README was stale in two ways now fixed:** it described the wire as
"one line then silence" (untrue since deltas landed) and omitted the tick from the status
line. Every command in it was executed against a live daemon before commit.

**TWO DECISIONS — NOW WIRED INTO THE PROCESS, so they fire without anyone remembering
(Wolf's instruction, 2026-08-03: "make sure we handle 1 and 2 in the story"). Do NOT
re-derive these; just follow what the skills now say:**
1. **Lever A is UNRESOLVED and `_bmad/custom/bmad-code-review.toml` now says so.** It carries
   2.1's evidence and instructs the reviewer to put the verdict to Wolf **at the start of the
   next review** — keep Lever A / revert hunters to Opus / drop hunters and spend on live
   execution — then **replace that fact with the settled config**. Evidence: kill-rate held
   (5/5) but the Sonnet hunters produced no unique finding (all 3 issues also found by Opus,
   2 of their 4 dismissed as unreachable); corrected cost flat ~$22 vs Epic 1's ~$21/story.
   Every substantive finding came from the layers that RAN THE BINARIES.
2. **`_bmad/custom/bmad-create-story.toml` now carries an OBSERVABILITY INSTRUMENT RULE** —
   every story must name, in a TASK, the instrument a human uses to see its headline outcome,
   with the command in Verification. Judge a candidate by: does it exercise the same code path
   the outcome flows through, and would it visibly differ if that path broke? 2.1 is the
   worked example in the rule itself.

**STORY 2.2 READY-FOR-DEV (2026-08-03, create cost $9.12).** `2-2-dwarves-wander-the-frost`,
13 ACs. Design calls made in the story, do NOT re-litigate: (a) **terrain moves into the ECS**
as a `Terrain` resource (dims/tiles/dirty) because a bevy system cannot see `World`'s private
fields and AD-7 forbids running `wander` outside the single chained schedule — `World`'s public
API is unchanged so 2.1's tests are the regression net; (b) wander = one orthogonal step to a
standable same-z tile within Chebyshev 3 of the spawn home every 10 ticks, staggered by
`id % 10`, so **`Walk` is a one-tick pulse** (sustained walking waits for 3.2's paths) and no
climb rule is invented early; (c) one shared `WanderRng` + explicit ascending-`Id` sort — the
same-seed test structurally cannot catch a missing sort; (d) `protocol::Entity` gains `state`
→ `{id, kind, pos, state}`; (e) instrument = 2.1's existing `tui --frames N`, cited not
reinvented. **Wolf's call, overriding my "leave it deferred": the 1.1 spawn/terrain RNG
coupling is CLOSED here** (`STREAM_SPAWN` + positions pinned as literals for seed 42) — his
question was "ain't there a danger of creating a future bug", and the deciding argument is
that a pinned test *without* the split degrades into pasting over new values whenever terrain
changes, plus 2.4's `SaveState` baselines make it costlier from then on. Evidence is inverted
sabotage: an extra `layered_terrain` draw must leave the pinned test GREEN. Also closed at
this session: 1.3's AC11 100×40 spec text (retro item → done; spec only, no code).
FR23's icy-grim sign-off recurs at 2.2/3.3 with motion — Wolf has signed off only the still
frame and a climbing counter.

**Ledger: 11 of 16 retro action items done, 5 open/in-progress** (sprint-status
`action_items` now carries `note:` fields explaining each). Applied
at 2.1's handoff: sabotage-with-RED as a hard requirement (worked). Still open and now due
before/at 2.1's *review*: hunters→Sonnet, diff-scoped hunter context, batched mutations, and
the `codex review` self-gate (blocked, see above). Due in 2.4: `rand_chacha`'s `serde`
feature. Per epics.md, **FR23's final sign-off recurs in Epic 2 with motion** — Wolf has only
signed off the still frame. Also unaddressed: the "every story ships an observability
instrument named in its tasks" item — 2.1's tasks name none (its live check is the nearest
thing); decide before 2.2.

**STORY 2.2 DONE + MERGED (2026-08-03/04).** PR #7 merged; `main` = `7371508`, working tree
clean. Cost $43.26 (create $9.12 · Codex dev $10.17 · review $23.97 — the first review under
$24, vs $57–76 in Epic 1). Sprint board now: `epic-2: in-progress`, 2.1 + 2.2 `done`,
**2.3 and 2.4 still `backlog` — nothing is ready-for-dev, so the next move is create-story 2.3**
(`3-1`/`3-2`/`3-3`/`4-1` also backlog; `epic-2-retrospective: optional`). Two things settled at
2.2's review and already encoded, do NOT re-ask: **Lever A is CLOSED** — keep the Sonnet hunters
AND mandate live execution in every layer (the Sonnet Blind Hunter produced the review's single
best finding — two dwarves sharing a tile, higher `Id` silently erasing the lower — by writing a
scratch binary over 200 seeds × 300 ticks); config lives in `_bmad/custom/bmad-code-review.toml`.
And the observability rule gained a **TESTED-INSTRUMENT addendum** (`bmad-create-story.toml` +
`docs/technical-preferences.md`) because 2.2's own instrument was broken — `stream_frames`
recomputed the camera per frame, pinning the dwarf to screen centre and rendering motion as
stillness, so the live evidence taken through it was an artefact. Naming an instrument is
necessary, not sufficient: it needs its own test. **Retro ledger: 15 of 16 items done, 1 open** —
"carry the RNG-stream-coupling and rand_chacha serde debts into 2.4's story notes"; the RNG half
was actually closed inside 2.2, so what genuinely remains for 2.4 is `rand_chacha`'s `serde`
feature. Deferred-work still open: the 80-column status line truncating as the tick gains digits.

**Two tooling defects found and handled, worth not rediscovering:** (a)
`_bmad/custom/bmad-create-story.toml` carried Nidavellir's sharded-epics rule verbatim from
the port, telling agents to load `epics/ep-NN.md` and NEVER read `epics.md` — frostvein has
no `epics/` dir, so following it means refusing to read the only file with the ACs. FIXED
2026-08-03. (b) `session_tokens.py` bills a transcript's delta since its last record, so a
mixed session (retro → sweep → wiring → create) lands entirely on one phase — 2.1's create
row is annotated `—`/unrecoverable rather than recording a 5x-inflated $96.21. A `--mark`
call at phase boundaries would fix it; not built.

**STORY 2.3 DONE + MERGED (2026-08-04).** PR #8 merged by Wolf, who also tested it himself.
Cost **$41.96** (Codex dev $19.92 · review $22.04 — review is now **53%** of spend, down from
Epic 1's 75%). The client writes for the first time: `protocol::Command` (internally tagged,
exactly one `SetSpeed` variant), `simd` decodes every inbound line and drives the loop from it,
`tui` sends one NDJSON line per keypress through a `try_clone`'d write half. Pause is **one `if`
around `world.step()`** — the tick freezes because `advance_tick` lives inside the schedule;
fast is a loop-period change only (100→20 ms). `sim-core` untouched, as AC5 required and as the
diff proves. Status line gained the speed and **dropped the camera coordinates** to fit 80
columns, closing the deferred overflow item. Gate green (verified 4×), **13/13 mutations killed
across 3 independent runs**, all 11 ACs met with 2–11 settled by live execution.

**Sprint board now:** `epic-2: in-progress`; 2.1, 2.2, 2.3 `done`; **`2-4-the-world-endures` is
`backlog` — nothing is ready-for-dev, so the next move is create-story 2.4.** It is the last
story of Epic 2 and owns the `Save`/`Load`/`Quit` `Command` variants 2.3 deliberately excluded,
plus the one genuinely-remaining retro item (`rand_chacha`'s `serde` feature).

**THE FINDING THAT MATTERS FROM 2.3'S REVIEW — the defect was in the SPEC, not the code.** Two
acceptance criteria were unmeetable or self-contradictory and four stories of process had never
caught it, because nothing had ever checked ACs against a *running* system. **AC9** demanded a
speed change appear in "the very next delta" — impossible by construction, since the command
crosses TCP and the loop drains `command_rx` at iteration top, so it lands on the second delta;
restated as the same-tick property FR19 actually requires (which is met). **AC2**'s parenthetical
claimed oversized lines leave the connection open, contradicting its own "keeps 1.2's behaviour
exactly" — 1.2 closes them. Both corrected in the story at review. This is the strongest argument
yet for the live-execution mandate: the layers that ran the binaries found it in one pass.

**Wolf's two calls at 2.3's review (do not re-litigate):** (1) the **stale-speed compose trap** is
ACCEPTED for now with an accurate `// NOTE:`, and handed to **Story 3.1** in `deferred-work.md`.
Three review layers independently found that the sanctioned limitation understated itself: two
*different* keys inside one round-trip compose against the same stale wire speed, so at `Normal`
`+` sends `Fast` and `-` sends `Paused` and last-write-wins lands on **Paused** — and since speed
is one shared value, a fumbled double-tap silently pauses the sim for every watching terminal. The
fix is optimistic client-side speed, which AD-4 and the story's guardrails forbid. (2) Both wrong
ACs were **corrected in place** rather than left frozen as historical artefacts.

**Codex's behaviour is worth noting as the pattern to keep:** its own `codex review` self-gate
raised a P1 asking for a bounded inbound command queue, and it **refused, citing the story's
no-backpressure guardrail**, rather than quietly complying. It applied the valid P2 (unbounded
TUI command writes → 30 s timeout, with its own killed mutation). It also reported a nested-sandbox
loopback denial instead of coding around it. That is exactly the delegated-agent behaviour
[[codex-sandbox-rule]] and AGENTS.md rule 3 are trying to produce, and it worked unprompted.

New at 2.3: [[concurrent-review-tooling-hazards]] — `scripts/mutate.sh` is not concurrency-safe,
and my AC-extract for the Acceptance Auditor was scoped too thin. Both cost real time this run.

**STORY 2.4 DEV DONE → REVIEW (2026-08-04).** Branch `2-4-the-world-endures`, **10 Völundr
commits, nothing pushed**. Codex dev **$25.66 / 325 turns** — by far the most expensive dev run
yet (2.3 was $19.92), plus **$7.03 of `codex review` self-gate** across 4 passes, which finally
puts a price on the self-gate the Epic 1 retro left unmeasured. Verified independently by me, not
taken on report: `scripts/gate.sh` GREEN, **27/27 mutations killed**, File List matches
`git diff --name-only` exactly, story Status `review` with zero unchecked boxes.
**Live evidence I took myself** (not Codex's): save at tick 22 → `--frames 20` climbs 66→85 →
`--frames 20 --key L` shows 86, 87, **22**, 23, 24, 25; two raw clients both received the
tick-22 snapshot from one client's `load`; `quit` → both clients reach EOF after draining
buffered deltas, daemon exits 0, `shutting down on client quit`, no panic.
**The headline trap was avoided** — `save_load_then_tick_matches_never_saved` uses the public
API as oracle against a never-saved control, ticks 37 before saving, asserts every step
([[self-referential-test-antipattern]] not hit a fourth time).

**FOR THE REVIEWER TO ADJUDICATE — Codex added validation no AC asked for.** Four of the ten
commits (`c727a7b`, `c8c1699`, `f4a1f70`, plus the `MAX_SAVE_BYTES` work) landed *after* it first
marked the story ready-for-review, driven by its own self-gate: tile-count consistency, a 16 MiB
save cap, `MAX_LOAD_TICK = u64::MAX/2`, and dwarf pos/home bounds checks in `simd`'s `load_world`.
My read is defensible-not-creep — AC7 requires an undecodable save be logged and dropped *without
panicking*, and a truncated file would otherwise panic `from_save`; AGENTS.md rule 1 exempts
bounds at an I/O boundary; it is all in `simd`, keeping `sim-core` I/O-free per AD-1. Worth a
second look anyway: the live save is **6.9 MB against a 16 MiB cap**, only ~2.4x headroom, and
`save_world` refuses to write what it could not read back, so the two limits must stay in step.
Codex also declined a self-gate P1 asking for a bounded command queue, citing the story's
no-backpressure guardrail — the same correct refusal behaviour it showed at 2.3.

**STORY 2.4 DONE + MERGED (2026-08-04).** PR #9 merged by Wolf mid-session; `main` = `3227de3`,
remote branch deleted. **EPIC 2 IS COMPLETE** — 2.1/2.2/2.3/2.4 all `done`, `epic-2-retrospective`
still `optional`, Epic 3 all backlog with nothing ready-for-dev. [[post-merge-branch-trap]] fired
again and cost a commit: Wolf merged while I was working, my checkout landed on `main`, and the
follow-up "Record story 2.4 review cost" commit was left **orphaned** (not on main) while
`git push` cheerfully reported "Everything up-to-date" because the upstream had become
`origin/main`. Recovered by cherry-picking the dangling commit onto a branch. **Check
`git branch --show-current` AND `git ls-remote origin <branch>` before believing any push result.** Story total **$72.88** (create $10.21 · dev-orchestration $3.05 · Codex dev $25.66 ·
self-gate $7.03 · review $37.14) — review is **51%** of spend against the ~$22/story 2.2 and 2.3
settled at, and that figure likely EXCLUDES ~580k subagent tokens. The overrun is almost entirely
the hunter stall (see [[review-layer-reliability]]). All 12 ACs met, all 16 hops wired, gate green
**8 consecutive runs**, **30/30 mutations killed**.

**Review outcome: 1 decision + 4 patches applied, 2 deferred, 2 dismissed, plus a 6th patch I
found while verifying.** The decision item — Wolf chose to CLOSE NOW, consistent with
[[defer-vs-close-now]]: a save reusing a dwarf id loaded clean and was broadcast forever while
every *other* malformed-save class was logged and dropped. Found independently by two layers and
reproduced by me. **The 6th patch is the one that matters most: the gate was failing ~1 run in 4,
pre-existing in Codex's work** — `Daemon::spawn` bound port 0, read the port, DROPPED the listener,
then spawned `simd` on it, and with 28 parallel daemon tests a sibling claimed it in the gap. Fixed
by passing `0` and letting the daemon bind (it already prints the port the harness already parses).
Proved pre-existing by stashing my patches and re-running. **A flaky gate is a process-level
emergency here** — "a green gate is a fact rather than a claim" is what the whole delegation loop
rests on.

**Wolf ran the TUI himself after the merge and confirmed the rewind works — the human sign-off on
2.4's observable outcome.** His first read was "I can see 2 dwarves jumping around, that is all",
which is exactly right and not a defect: `render` draws ONE z-slice (`entity.pos[2] != state.z` is
skipped), so 2 of the 5 dwarves are visible on any given level while the status line still reports
`dwarves 5`; and the "jumping" is 2.2's wander — one orthogonal step per 10 ticks, `Walk` being a
one-tick pulse until 3.2's pathfinding. **Worth remembering for every future demo: 2.4 is invisible
while idle.** The recipe is press `S` (nothing on screen — evidence is the daemon's
`saved tick N` log and the file), let the tick climb, press `L`, and watch the tick jump BACKWARD
with dwarves snapping to their saved positions while camera and z stay put.

**The validation-scope question was settled, do not re-litigate:** Codex's unrequested input
validation (tile-count, 16 MiB cap, `MAX_LOAD_TICK`, dwarf bounds) is SANCTIONED, not scope creep.
`MAX_SAVE_BYTES` is *mandated* by AGENTS.md rule 1 ("bound every read you add"); the tile-count
check mirrors `tui`'s existing `validate_snapshot`; the dwarf-bounds guard converts a real debug
panic in `wander` (`here.x + dx` overflow) into AC7's log-and-drop.

New at 2.4: [[metrics-attribution-traps]] — the ledger was wrong twice before I caught it, once
because **Codex ran `session_tokens.py` itself** and billed three phantom `review` rows onto my
live orchestration transcript, once because `--tool codex` grabbed a foreign `/workspace` rollout
worth $0.35 instead of the real $25.66 run. Hand-corrected; the handoff prompt still needs an
explicit "do not run session_tokens.py" line.

---

## STATE AT 2026-08-04 (superseded — see the 2026-08-05 section at the end)

**EPIC 1 AND EPIC 2 ARE BOTH COMPLETE AND MERGED.** `main` = `8bf4548` (PR #10), local synced,
working tree clean, nothing unpushed, both 2.4 branches deleted. PR #9 = story 2.4, PR #10 = its
cost ledger. Wolf ran the TUI himself and confirmed the save/load rewind.

**NEXT MOVE, agreed with Wolf: run the EPIC 2 RETROSPECTIVE** (`bmad-retrospective`).
`sprint-status.yaml` has `epic-2-retrospective: optional` — it needs flipping to `done` when the
retro completes, and the retro appends its action items to the existing `action_items:` section
(Epic 1's 16 items are all `done`, each with a `note:`). After the retro: create-story 3.1
(`3-1-give-the-order`) — all of Epic 3 is `backlog` and nothing is `ready-for-dev`.

**Material the retro should chew on — all four already written up in memory, do not re-derive:**
1. **Review-layer reliability is the big one.** Both Sonnet hunters hung ~2.5h and returned
   nothing, twice in one session; the relaunch pushed 2.4's review to **51% of spend ($37.14 of
   $72.88)** against the ~$22/story 2.2 and 2.3 settled at. They DID earn their keep once running
   (duplicate-dwarf-id defect, 40-seed determinism sweep). Fix is prompt discipline, not dropping
   the layers → [[review-layer-reliability]].
2. **The gate was flaky (~1 run in 4) and nothing caught it** — a port race in 2.4's own new test
   harness that survived Codex's dev run, four self-gate passes and two review layers. "A green
   gate is a fact rather than a claim" is what the delegation loop rests on. Ask why every layer
   missed an intermittent red.
3. **`session_tokens.py` mis-recorded cost twice**, including Codex running it itself and billing
   three phantom rows onto the orchestrator's transcript → [[metrics-attribution-traps]]. The
   handoff prompt still needs an explicit "do NOT run session_tokens.py" line.
4. **The self-gate now has a price: $7.03** across 4 passes (new `dev-selfgate` metric row). Epic
   1's retro left its value unmeasured; at 2.4 it drove three real post-review commits. Decide
   whether it stays.

**Also open:** `_bmad/custom/bmad-code-review.toml` tells every review subagent to export a mise
rust path that DOES NOT EXIST (harmless only because the rustup shim is already on PATH) →
[[frostvein-toolchain]]. Six stale local branches from merged stories could be pruned. Deferred
work for Epic 3: no in-UI quit affordance and the `MAX_SAVE_BYTES`/`Dims::DEFAULT` coupling
(both in `deferred-work.md`), plus 2.3's stale-speed compose trap owned by 3.1.

---

## STATE AT SESSION END (2026-08-05) — NEXT SESSION STARTS HERE

**EPIC 2 RETROSPECTIVE DONE.** `_bmad-output/implementation-artifacts/epic-2-retro-2026-08-05.md`;
`sprint-status.yaml` has `epic-2-retrospective: done`, `last_updated: 2026-08-05`, and 28 action
items (Epic 1's 16 all done, 12 from Epic 2). Gate re-verified green at retro time (102 tests).
**NEXT MOVE: create-story 3.1** — Epic 3 is unblocked, both blockers ruled on by Wolf.

**THE HEADLINE FINDING, and it inverts Epic 1's.** Wolf corrected the framing: Codex spend is a
DIFFERENT CURRENCY (it does not consume Claude quota), so read the Claude subtotal, not the total.
On that axis **review is 78% of Claude spend in BOTH epics** — unchanged by all three of Epic 1's
cost levers. New instrument made the reason visible: **96% of every token processed is a cache
read, in both epics** (Epic 1: 971 turns / 102.8M tokens; Epic 2: 2,146 turns / 290.8M). Epic 1's
retro predicted review $193→$39; corrected-against-corrected it ROSE, $21.44→$26.30/story. What
the levers actually bought was correctness (patches 10→4, mutations 5→30, the self-referential
class extinct after 2.2). Say this plainly rather than re-deriving it — full numbers in
[[review-cost-economics]].

**TWO RULINGS WOLF MADE AT THE RETRO — do NOT re-litigate, carry them into 3.1/3.2:**
1. **T1 = OPTION C.** The AD-10 command consumer is a plain `&mut self` method on `World`, called
   by `simd` at iteration top BEFORE the conditional `world.step()`. Rejected: a second schedule
   (breaks AD-7 + doubles what 2.4's single `assemble()` must keep in step) and bevy run-conditions
   (makes `sim-core` learn pause, against 2.3's guardrail). C matches `World::set_tile`'s existing
   precedent — sim state already mutates via a plain method. Cost needing a `// NOTE:`: ordering is
   explicit by call-site, not `.chain()`. **3.1 must ALSO settle** where the still-applies-while-
   paused line falls (designation appears while paused = yes; does job conversion / reaction delay
   tick? those are world-advancing and skip — 3.2 adds both and will draw the line by accident
   otherwise), and that **3.1 is a WIRE CHANGE its epic text never says** — `designations`/`zones`
   are still `Vec<()>` at `crates/protocol/src/lib.rs:100-101,115-116`.
2. **Story 3.2 STAYS ONE STORY.** Wolf declined my 3.2a/3.2b split ("even though it will take
   time"). Risk accepted, not overlooked; five mitigations belong in its story-creation + handoff —
   full list in the sprint-status action item.

**PROCESS CHANGES SHIPPED THIS SESSION (all verified, not claimed):**
- **`_bmad/custom/bmad-code-review.toml` gained a `LAYER TIME-BOX` fact** — 20 min/layer,
  orchestrator kills and continues, a timed-out layer is reported as a COVERAGE HOLE never a clean
  result, 60 min whole-review ceiling. Fixes 2.4's 2.5h hang. **Key insight worth keeping: a
  growing transcript is NOT progress** — a hung layer keeps emitting tokens, so file size and
  "still running" look identical to healthy work. I verified the in-session fixes from 2.4 had been
  written to NO config file, so Epic 3 would have inherited the trap.
- **`_bmad/scripts/session_tokens.py`** gained wall-clock (`minutes` column appended LAST so the 20
  historical rows still parse; cursor stores `last_ts` so each phase's duration bills over its own
  window), a rollup **Spend-shape table** (turns/tokens/cache-read %/output/minutes), a **ledger-row
  width guard** (annotation tables inside a ledger were being parsed as data rows, inventing phantom
  rollup phases), and a **preserve marker** so `--rollup` stops destroying hand-written analysis —
  which it did to `1-rollup.md` before I caught it; unmergeable content is now backed up, never lost.
- **`_bmad/scripts/tests/test_session_tokens.py`: 19 tests pass** (was 10, one of which my change
  broke). **This suite is NOT wired into `scripts/gate.sh`** and its own docstring records that it
  "went red and stayed red, unnoticed, because nothing runs it". It runs on system
  `python3 -m unittest discover` — no pytest, no venv. One line in gate.sh closes it; Wolf has not
  ruled on that yet because it changes his pre-commit hook.

**FORGE PROPAGATION IS DELIBERATELY HELD — and Wolf asked to be reminded because he will not
remember.** He wants the time-box and the metrics columns proven on story 3.1 first. Three
mechanisms carry it so it cannot rot: (1) the `AFTER STORY 3.1` action item in sprint-status with
the exact verified command sequence; (2) a **self-removing STEP C in `bmad-code-review.toml`'s
`on_complete`** that fires at the end of 3.1's review and instructs its own deletion; (3) this
memory. Verified live by running `./scripts/forge-process.sh check projects/frostvein` from
`/workspace`: `session_tokens.py` **DIFFERS** with **frostvein ahead** (the entire 166-line diff is
this session's work — no pre-existing divergence, so the old PRICES defect really was closed), and
all three `_bmad/custom/*.toml` report **UPSTREAM CHANGED** (pre-existing; the forge has fixes
frostvein never took, and create-story/dev-story shape how 3.1 and 3.2 get written — clear them
BEFORE 3.1). The time-box lives in a TEMPLATE, so `check` can only say "go look"; the runbook's own
Known-limitation section names the trigger for fixing that properly (third project, or a shared rule
hand-merged twice) — **this is merge 1 of 2**.

**FR23's MOTION SIGN-OFF DID NOT CLOSE, and the review predicted it verbatim.** Wolf: happy in the
early stories, then "the world didn't change after that, so it was a bit boring after all". 2.2's
review had already written *"at a default 80x24 terminal only one of the five dwarves is ever
visible... the headline outcome reads as one dwarf twitching once a second"* — and it was deferred.
Compounding causes, all recorded: one dwarf visible per z-slice, `Walk` is a one-tick pulse in
eleven, and the dirty-tile path is INERT so every Epic 2 delta carried `tiles=[]`. **Wolf's call: do
NOT spend a TUI story on it** — 3.1's cursor puts the camera where the work is and 3.2's dig is the
first thing that visibly changes the world. Re-checked at 3.3. **The transferable lesson: when a
review layer says "the headline outcome reads as X" and X is bad, that is a PRODUCT finding, not a
nice-to-have** — no AC was violated here, which is exactly why it got deferred.

**Files changed this session (all uncommitted at session end, gate green):**
`_bmad/scripts/session_tokens.py`, `_bmad/scripts/tests/test_session_tokens.py`,
`_bmad/custom/bmad-code-review.toml`, `_bmad-output/implementation-artifacts/sprint-status.yaml`,
`metrics/1-rollup.md`, `metrics/2-rollup.md`, plus two new:
`_bmad-output/implementation-artifacts/epic-2-retro-2026-08-05.md` and
`docs/forge-transfer-2026-08-05.md`.

---

## STORY 3.1 CREATED (2026-08-05, later the same day) — NEXT = dev-story 3.1

**`3-1-give-the-order` is `ready-for-dev`**, `epic-3: in-progress`. Story file:
`_bmad-output/implementation-artifacts/3-1-give-the-order.md`, 15 ACs, baseline `8bf4548`.
Create cost **$15.50** (two runs: $10.49 + $5.01 for the stockpile-removal amendment). Working tree
is dirty with the retro session's files PLUS `_bmad/custom/bmad-create-story.toml`,
`sprint-status.yaml` and the new story — **nothing committed, still on `main`**; the story says
branch `3-1-give-the-order` ([[post-merge-branch-trap]]).

**THE AC-AUTHORING RULE IS NOW ENCODED and it fired on its first use.** Wolf approved pulling the
forge's ep-11 A6 rule into `_bmad/custom/bmad-create-story.toml`, merged with Epic 2's own open
action item (now `done` with the evidence in its `note:`). Two checks per AC: (1) CAN IT HAPPEN —
trace the observation through the real code path before writing it; (2) OUTCOME NOT MECHANISM, with
named legitimate exceptions (determinism invariants, byte-exact wire contracts, architectural edges,
reuse-don't-rebuild). **It caught my own draft AC4**, which said a designate command sent while
paused appears in "the very next delta" — the *identical* defect to 2.3's AC9, since the command
crosses TCP and lands on a later iteration. Rewritten before saving. The forge's MODEL POLICY fact
was deliberately NOT taken (frostvein has no Fable history); that half of the reconciliation, and
`session_tokens.py` still `DIFFERS`, remain for the post-3.1 propagation.

**WOLF'S SCOPE ADDITION: stockpile removal, which `epics.md` does not contain.** Without it a
misplaced stockpile is permanent. Shape I chose and he accepted: **`x` is a single eraser key that
emits TWO wire commands**, `cancel_designation` then `remove_stockpile`. Rejected teaching
`cancel_designation` to also delete zones — it would keep AD-10's list at three but leave a wire
command whose name lies about half its job, in the crate whose entire purpose is being the single
source of message shapes. Two precise commands buy independently assertable erasures (AC2 and AC3
are mirrors) and a harness that can inject either alone. **Consequence to close later, flagged in
the story rather than silently patched: AD-10's prose and `docs/architecture.md` still enumerate
THREE world-mutating commands.** The dev agent is told to raise it in the Dev Agent Record so epic
text + spine are corrected together, once — correct-course work, not a story-side edit.

**Design calls baked into the story — do NOT re-litigate at dev or review:**
- **Dig rects are NOT filtered for diggability** — every in-bounds tile is recorded, air included.
  The epic's clip rule is written for stockpiles only; what is *diggable* is 3.2's, with FR8's retry.
- **`Zone` carries no `kind` field.** Stockpile is the only zone in phase one and a single-variant
  enum is the abstraction YAGNI forbids. Designations DO get an enum — dig and channel are two real
  cases, so AD-6's mirror-and-bridge pattern is exercised properly there.
- **The hint bar is a SECOND row** (`map_h = h - 2`), key hints move out of the status line — that
  is what buys back the 80-column budget the status line already exhausts at 7-digit ticks. This is
  the largest churn source: every existing render test's expected framebuffer shifts.
- **`apply_key` changes shape twice in one edit** — `speed` parameter leaves (replaced by optimistic
  `state.speed`), `viewport` arrives (so the camera can follow the cursor).
- **Markers are glyph-distinct by design**, not colour-only, so the `--frames` capture is real
  evidence even under this devpod's `NO_COLOR=1` — the trap that made 2.2's colour evidence vacuous.
- **`Action` carries one command today**; `x` needs two. Story says take the narrowest change and
  `// NOTE:` that two is the only arity anything needs — **watch at review for a general
  multi-command mechanism nobody asked for.**

All three 3.1-owned deferred items are folded in as ACs: optimistic client-side speed (the 2.3
compose trap), `read_inbound`'s partial-line-as-overflow misreport (**note the deferred file cites
`main.rs:270`; 2.4 moved it to `main.rs:401-405`** — a stale citation I corrected, worth re-checking
line numbers in every deferred item), and the `q quit client` wording.

---

## STORY 3.1 DONE — PR #12 OPEN, AWAITING WOLF'S MERGE (2026-08-05) — NEXT = create-story 3.2

**`3-1-give-the-order` is `done`** in both the story file and sprint-status; branch pushed, **PR #12
open against main, Wolf merges himself.** 19 Völundr commits, gate green, **33/33 mutations killed**
(was 35; two retired with the code they guarded — by design, not attrition). Codex dev **$29.88 /
376 turns / 86 min** + **$7.93** across three self-gate passes; review **$39.18 / 319 turns** *plus
~14.1M unrecorded subagent tokens* — story ≈ **$77**. Epic 3 continues: 3.2 and 3.3 are `backlog`,
nothing is `ready-for-dev`.

**Codex's run was the strongest yet and needs no re-litigating.** All 15 ACs met, wire change landed
atomically, nothing added to `schedule.add_systems` (Option C held), all three 3.1-owned deferred
items closed. Two things it did that are worth copying: it **reported the story's own Verification
recipe as defective** rather than passing off confounded output as evidence (the `p` rect is a subset
of the `d` rect so AC11's layer order hides `≡` under `×`, and separate client starts recenter on a
moving dwarf), then ran a controlled variant; and it **declined an out-of-scope save-cap fix its own
self-gate demanded**, documenting the measurement instead. That declined finding turned out to be
real — see below.

**THE ONE HIGH FINDING, and I found it inline because three review layers died.** A designate rect
clips to the whole world = 524,288 marks, and AD-8 full-resends every mark in every delta: measured
live, **378 bytes/delta → 16,761,209 bytes/delta, 34.7 MB/s sustained**, to every client, no recovery
short of a daemon restart. Reachable from the shipped client (TUI clamps to one z-level ≈ 16,384
marks, so 32 ordinary commands get there). Wolf's ruling — **split it**: the save half was a defect
*this story introduced* (23.2 MB save vs a 16 MB cap = a legal action made the world unsaveable), so
`MAX_SAVE_BYTES` was raised to 64 MB matching the client's `MAX_SNAPSHOT_BYTES`; the **wire half is
deferred to 3.2** with its measurements in `deferred-work.md` and a `// NOTE:` at the site, because
every fix changes AD-8's full-resend contract or invents a designation limit. **This also fired the
existing deferred `MAX_SAVE_BYTES`/`Dims::DEFAULT` item — by the other route.** It predicted a dims
change; what actually broke it was *added state*. Prediction right about mechanism, wrong about input.

**Review outcome: 2 decisions resolved, 2 patches, 1 deferred, 2 dismissed.** The other decision:
`WORLD_COMMAND_CAPTURE_DRAIN` — ~50 lines in the real `tui` binary with a magic `17` encoding
**simd's `CLIENT_QUEUE` depth inside the client**, the crate that depends on `protocol` alone.
Removed; the instrument now just asks for 21 frames to outlast the backlog, which also made AC14's
negative control the *identical* run the AC demands. Verified live after removal against a real
daemon: no-key capture 0 dig glyphs, keyed capture 80. **The best small catch was S4** — the ≤80-column
hint assertion read 80 cells out of an 80-wide framebuffer and asserted `<= 80`, mathematically
unfailable; now renders at 120. Dismissed with reasons kept: `load_world`'s OOB mark validation
(defensible, mirrors 2.4's precedent) and a "stray top-left marker" that was a **review-harness
artifact** — two Opus auditors sharing one fixed-port daemon, one seeing the other's marks at
(12,11) which lands at screen col 2 row 0 for a camera at (60,30).

**THE PROCESS VERDICT WOLF ASKED FOR — the time-box's first real exercise, and it is NOT ready to
propagate.** It fired 5 times: **2 correct kills** (genuine hangs, one running entirely alone) and
**2 false positives** (working hunters starved on the shared `target/` lock), 1 layer completed. The
forge propagation action item in `sprint-status.yaml` is now **`blocked`** with both defects written
into its note. Two blockers, both must be fixed here first: (1) the time-box must measure
**silence-since-last-named-step**, not wall-clock-since-launch; (2) `session_tokens.py` misses
subagent transcripts entirely. Details: [[review-layer-reliability]], [[metrics-attribution-traps]].
**STEP C in `bmad-code-review.toml` has been deleted as instructed** — it fired, it was actioned, it
will not fire twice. The other pre-existing propagation debt is untouched: all three
`_bmad/custom/*.toml` still report UPSTREAM CHANGED.

**Do not rediscover:** clean the build cache **after** `mutate.sh`, not only before — a stale mutated
`simd` produced two convincing false failures that I nearly "fixed" as real defects
([[concurrent-review-tooling-hazards]]). And `pkill -f 'target/debug/simd'` **kills your own shell**,
because the pattern matches the command running it; use `for p in $(pgrep -x simd); do kill $p; done`.

---

## 2026-08-06 — THE THREE UPSTREAM-CHANGED TOMLS ARE CLEARED. NEXT = create-story 3.2

**PR #12 merged; `main` = `ebe31c4`.** Sprint board: Epic 1 + 2 `done`, `3-1` `done`, **`3-2-the-dig`
and `3-3` `backlog`, nothing `ready-for-dev`** — so the next move is `create-story 3.2`.

**A TOOL DEFECT FOUND AND FIXED — `forge-process.sh` had no way to clear a TEMPLATE notice.** The
runbook contradicted itself: step 2 said hand-merge, step 5 said `check` then comes back clean. It
could not. The verdict compares the forge's *current* hash against the target's `SRC` stamp, and
`install` **deliberately carries a kept TEMPLATE's old hash forward** (so an unrelated FILE install
cannot silently clear a pending notice — that guard is correct). Net effect: a correctly hand-merged
TEMPLATE stayed red **forever**, and a permanently-red check gets ignored, which is exactly how the
PRICES fix rotted for a month. Fixed by adding **`forge-process.sh ack <project-root> <path>...`** —
re-stamps one path, copies nothing, validates all paths before writing any (a mixed good/bad batch
writes nothing — verified), and **refuses FILE entries** since a `DIFFERS` there is a real defect to
reconcile, not silence. No VERSION bump (no FILE changed). Runbook step 2 + History updated.

**What was actually merged in (the diffs were mostly noise; only three things were real):**
- **`bmad-code-review.toml` ← REVIEW-COST DISCIPLINE** (forge ep-11 A3, sitting untaken since
  2026-08-03). All three rules on Wolf's yes: (1) every finding labelled with **layer + severity**
  — this *closes* the Epic 2 action item and is what R1 rests on; (2) **cap the LOW tail** straight
  to `deferred-work.md`, patch only HIGH/MED — **with a frostvein exception I added: a LOW that is a
  latent silent-failure trap is patched regardless** ([[defer-vs-close-now]]); (3) **patch in a
  fresh session**, handed the diff + findings, never the review transcript.
- **`bmad-dev-story.toml` ← COMMIT CADENCE HARD FLOOR** (ep-11 A8): minimum one commit per completed
  TASK. Directly serves 3.2, whose own mitigation list calls commit-per-green the recovery mechanism.
- **`bmad-create-story.toml` ← MODEL POLICY, reframed by Wolf** (2026-08-06). The forge's version
  rationed non-Opus; **Wolf's stance is Opus by default, other models are legitimate when the work
  warrants it, and the binding half is RECORDING the exact model id** (`claude-opus-5`, never a
  family nickname) in story frontmatter with a one-line reason — for *learning*, so a retro can ask
  which model produced better work at what price. This supersedes the 3.1-era "deliberately NOT
  taken" decision; that reconciliation half is now CLOSED.

**A FORGE DEFECT FOUND IN THE OTHER DIRECTION — do not let anyone "fix" frostvein toward it.** The
forge's `bmad-dev-story.toml` `on_complete` says to **SKIP** the dev metric because "Codex records it
in its own session". That is wrong — a `session_tokens` run inside `codex exec` bills the wrong
transcript ([[metrics-attribution-traps]]), so the orchestrator must record it with `--tool codex`.
**frostvein's copy is right, the forge's is the defect**, and any sibling installed from it is
silently losing dev-phase rows. Added to the propagation item as a third outbound fix.

**Also deliberately NOT taken:** the forge's sharded-epics rule. frostvein has no `epics/` dir and
corrected this once already; taking it back would re-break story creation.

**sprint-status.yaml cleanup (Wolf's yes):** `epic-2` → `done` (all four stories + retro were done,
the epic status was simply lagging); the invalid `status: blocked` → `in-progress` with a new
`blocked_on:` field holding the two real blockers (`blocked` is not a valid action-item status and
tripped validation); three stale items closed **after verifying against 3.1's story file, not
assumed** — Option C landed, and all three deferred items are ACs 12 (stale-speed), 13
(`read_inbound`), 10 (`q quit client`, resolved as *honest text* rather than adding the command).
**Also fixed a latent silent-failure trap in that file: the AC-authoring item had TWO `note:` keys on
one mapping**, so any YAML parser would keep only the last and discard the "Applied 2026-08-05"
evidence. Second key renamed `baseline:`. Open items now 5 (was 8).

**CORRECTION TO THE ABOVE, same day — the dev-metric finding was REAL, and my retraction of it was
the error.** I reported that the forge's `bmad-dev-story.toml` `on_complete` wrongly says to SKIP
the dev metric in delegated mode. Then I "checked" and retracted, because the forge's ledgers had
`dev | codex` rows with real rollout transcripts. **The row I used as evidence had been backfilled
by Wolf minutes earlier, precisely because the defect had eaten it** — `ep-06-us-04` had gone
through dev, review AND review-patch with no `codex-dev` line at all, silently omitting the
implementation cost. I read the output of the fix as proof the bug never existed. **The transferable
lesson: a `recorded` timestamp long after the run is evidence of a BACKFILL, not of a working
pipeline — I even noticed the 19-hour gap and explained it away instead of following it.** When
checking whether a thing is broken, never sample an artefact someone may be actively repairing.

**IT THEN CAME BACK AS AN INBOUND MERGE, and the forge's version is better than frostvein's was.**
Wolf's fix added the half frostvein never had: bare `--tool codex` takes the NEWEST rollout in
`$CODEX_HOME/sessions`, so **always pass `--transcript <path>` unless you have confirmed the newest
rollout IS the dev run** — at 2.4 that was a foreign `/workspace` run worth $0.35 standing in for
the real $25.66 one ([[metrics-attribution-traps]]). Merged into frostvein's
`bmad-dev-story.toml` with the concrete path (`/workspace/.codex/sessions`, the CODEX_HOME
`codex-handoff.sh` pins) and the identify-it recipe (`rg -c <story-key>` across that day's
rollouts, then check cwd). Acked. All four TEMPLATEs are `adapted` again.

**Worth noting as a process result, not just a fix:** this is the SECOND time a forge instruction
defect surfaced only because a project that copied the forge had already fixed it (ep-11 A8 was the
first). The propagation channel works; what is missing is a reason to look down it before a row
goes missing.

---

## 2026-08-06 — STORY 3.2 DEV DONE → REVIEW. NEXT = code-review 3.2

**`3-2-the-dig` is `review`** in both story file and sprint-status. Branch `3-2-the-dig`,
**27 Völundr commits, NOTHING PUSHED, no PR** — review-gated as always. Baseline `c7a32fa`
was preserved (already stamped at create); branched from `main` = `51eb65d`, which is
docs-only ahead of `c7a32fa`, so code-identical to what the story assumed.

**Verified by me independently, not taken on Codex's report:** gate GREEN (run twice by me
plus the pre-commit hook = 3 confirmations), **75/75 mutations KILLED on a clean four-crate
build**, 26 dev commits all authored `Völundr`, all 16 story-listed files present
(+4,168/−174), story file has 61/61 boxes ticked and zero unchecked, and both assigned
`deferred-work.md` items (AD-8 amplification + inert dirty path) closed with nothing else
touched. The 111 `401` hits in the run log are diff hunk headers and source line numbers —
**not** auth failures; check `Missing bearer`/`Unauthorized` specifically before believing
that grep.

**THE COMMIT-CADENCE FLOOR WORKED ON ITS FIRST REAL TEST.** Four stories running, the
cadence was written into Dev Notes and never actually ASKED FOR in the handoff prompt, and a
squash shipped every time. This handoff asked for it explicitly and got one commit per task
group (`Move the entity allocator into ECS` → `Add the deterministic job market` → … →
`Prove the dig mutation set`). Keep restating it in the prompt — the story file is not where
Codex takes instructions from.

**Cost: dev $60.96 / 715 turns / 96.0M tokens (189 min)** on rollout `019fd5b5`. The trap in
[[metrics-attribution-traps]] was live — the NEWEST rollout was `019fd647`, a nested
`codex review` sub-session, and a `/workspace` run with zero story hits also sat in the
directory. Always `--transcript`.

**NEW DEFECT INSTANCE, and it is the SAME CLASS as forge blocker (2).** Codex ran its
mandated `codex review --base main` self-gate **six times**, and each cycle spawned its
**own sibling rollout** rather than logging into the dev transcript — so **$18.28 / 218
turns / 20.1M tokens is invisible to the ledger**. True dev cost is **$79.24 / 933 turns /
~116.1M**. Quantified per-rollout in `metrics/3-2-the-dig.md` as a caveat (not hand-merged
into the row — same precedent as the `create` caveat). Blocker (2) was found on the
*review* side (Claude subagents under `subagents/`); this confirms it on the **dev** side by
a different mechanism (nested `codex exec` sessions). **Whatever fix lands must walk sibling
rollouts, not just Claude's subagents dir.** Measure without recording via
`session_tokens.py --tool codex --transcript <f>` with `--story` OMITTED.

**TWO THINGS THE REVIEW MUST ADJUDICATE — flagged, not resolved:**
1. **The 7th self-gate never ran — Codex usage is exhausted until 2026-08-12.** Six cycles
   completed and raised 17 legitimate findings (lost work progress, movement before settle,
   an overestimating ramp heuristic, global item-id reuse, dwarf starvation, stale transient
   paths, per-claim A* budget escape), all fixed with RED-first regressions. But the final
   post-fix confirmation could not start, so **the last two commits (`a594e54`, `df59dc2`)
   carry no clean self-gate verdict** — code review is their first independent look.
2. **Load-validation scope, the 2.4 question again.** AC13's task asked only to extend the
   existing `in_bounds` check to job targets and item positions. Codex went well past that:
   rejecting exhausted `u32::MAX` allocators, over-budget designation/job tables, jobs whose
   kind mismatches their designation, duplicate ids, claims for missing jobs. Defensible
   under AGENTS.md rule 1 (bounds at an I/O boundary) and all self-gate-driven — and 2.4
   settled the identical question as SANCTIONED — but it is more surface than assigned. Also
   a **test-only** `std::sync::Mutex` now serializes the daemon harness against pre-existing
   cross-process timing contention; test-only, no assertion weakened, but confirm it.

**Design decision Codex made where the story let it choose:** `WorkProgress` is a dwarf ECS
**component**, persisted via `SavedDwarf.work_progress`; `Path` stays transient and is
deterministically recomputed after load (AC13 forbids saving it).

**I did NOT chain to code review**, though `bmad-dev-story`'s `on_complete` says to do so
without asking. This session carries a standing instruction not to spawn agents unless the
user requests it, and the configured review is a five-layer multi-agent run. Held and put to
Wolf instead. **If a future session has no such restriction, just run it** — the project's
default is review-immediately-after-dev.

**STORY 3.2 REVIEWED + DONE + MERGED (2026-08-06).** PR #14 merged by Wolf mid-session; `main` = `3837fc1`, working copy back on `main`, remote and local story branches gone, nothing unpushed ([[post-merge-branch-trap]] fired again but cost nothing this time — everything was pushed before he merged). Status `done` in
story and sprint board; 31 commits pushed. Four review layers, ALL COMPLETED, no coverage holes:
1 decision, 3 patches, 5 deferred, 3 dismissed, all 17 ACs met. Re-verified by me on a CLEAN build:
gate GREEN, **75/75 mutations killed**.

**THE THING TO REMEMBER FROM THIS REVIEW — I made the suite weaker and the mutation set caught me.**
The Acceptance Auditor called `astar_stops_at_the_node_cap`'s 224x224 grid (50,176 nodes vs a 50,000
cap) a fragile 0.35% margin and recommended widening it. I agreed and went to 320x320. That let the
`MAX_ASTAR_NODES is widened` (50_000 -> 60_000) mutation SURVIVE — the story's first survivor —
because 102,400 nodes exhaust under BOTH caps, so the test passes either way. **A tight margin in a
constant-pinning test is often deliberate: it must sit BETWEEN the real constant and the smallest
mutation that probes it. Before "fixing" a fragile-looking fixture, find which mutation it is
calibrated to kill.** Reverted to 224x224 with a `// NOTE:` explaining why (an Opus auditor AND I
both misread it), plus a new `astar_finds_a_path_well_inside_the_node_cap` for the downward
direction. This is why mutation testing is run after review patches, not only before.

**Four ACs were amended to match code that was right** (AC5 claim-time reachability is a real third
gate; AC8's "Idle exactly when no job" is unsatisfiable since `wander` reports `Walk`; AC10's "below
is standable" would strand a dwarf hovering over a two-deep shaft; AC3-vs-Tasks `continue`/`break`).
**Spec-text defects are now 8 instances across 9 stories** — Epic 3's baseline of zero is already
blown, and every one is authored in create-story and caught at review.

**The story's own Verification recipe produced ZERO marks** (Feature Auditor, live). Three faults:
no `--key` at all (so `apply_key` never fires and nothing is designated), no `<` to descend to a
diggable level, and it told the reader to watch for a wall-to-floor glyph change that AC15 in the
same document says is unobservable under PEEK_DEPTH + NO_COLOR. Corrected and verified live before
being written down (118 mark-frames / 197 stone-frames).

**MY OPERATIONAL ERROR, do not repeat:** I ran `cargo clean -p` BEFORE `mutate.sh` and not after,
so stale mutated artifacts poisoned two review layers' test runs — both auditors hit it, neither
mistook it for a code defect. [[concurrent-review-tooling-hazards]] already says clean AFTER. Worth
building into `mutate.sh` rather than trusting memory.

**Time-box: all four layers finished; my extension was right but my stated evidence was wrong.** I
cited the Feature Auditor at 26.3 min from the agent-reported `duration_ms`, which is INFLATED — by
wall-clock it finished inside 20 min. Only the Acceptance Auditor (~25 min) and Blind Hunter
exceeded. **Trust your own launch timestamp, never `duration_ms`.** The real blocker is unchanged
and unfixed: measuring silence-since-last-named-step needs the orchestrator to READ layer progress,
and layer transcripts cannot be read without overflowing context. I asked for `STEP:` markers and
could not use them. That needs a mechanism, not a number.

**NEXT = create-story 3.3** (`3-3-the-haul-and-the-skeleton-walks`, still `backlog`, last of Epic 3).
**Its Codex dev cannot run until 2026-08-12 07:00 UTC** — see [[review-cost-economics]]. Story
creation and review are Claude and unaffected. 3.3 also inherits: FR23's motion sign-off (open, owner
Wolf, due at 3.3) and the heavier-than-promised AD-12 claiming seam (`claim_jobs` now owns
work-positions, A* and the node budget, so `Haul` is not a pure `JobKind` addition).

**WOLF PLAYED 3.2 AND WAS CONFUSED — treat this as a PRODUCT finding for 3.3, not a support
question (2026-08-06).** His words: "I can dig some areas and it leaves `*` .. stockpiling does
nothing.. channelling I didn't try". Everything he saw was CORRECT behaviour, which is the problem.
Verified against source, not assumed:
- **`p` stockpile has ZERO sim behaviour.** `Zones` is written by `PlaceStockpile`, saved, and sent
  on the wire, but **no system reads it** (`crates/sim-core/src/lib.rs:290,1043-1067`). Hauling is
  3.3. So the client paints a rectangle for a feature that does not exist — a false affordance that
  teaches the player the game is broken.
- **`c` channel WORKS but is invisible from where you stand.** I ran it live (closing the gap the
  Feature Auditor left open): keys `c,enter,l,l,l,enter` on the dwarves' own level produced stone in
  204 frames — and stone only spawns on job COMPLETION, with no `d` pressed. But the ramp it carves
  is one z BELOW, and worldgen `▲` ramps are everywhere already (5,720 glyph hits in one capture),
  so a new one cannot be spotted by eye. You must press `<` afterwards to see anything.
- **Glyph table for demos:** `×` dig mark · `▼` channel mark · `≡` stockpile zone · `*` stone ·
  `▲` ramp. Wolf had no way to know `≡` vs `▼`.

**THIS IS FR23's "a bit boring" RECURRING, and 2.2's review predicted the shape of it before it was
deferred.** Two candidate items PUT TO WOLF AND NOT ACTIONED — he closed the story instead, so raise
them at 3.3 story-creation rather than re-deriving them: (1) `p` should either say it does nothing
yet or be held back until 3.3; (2) channel needs an outcome observable from the player's own level
(a status-line report would do), because the result is by definition somewhere they cannot see.
3.3's hauling closes most of this by construction — it is the first time stone MOVES rather than
merely appears — and FR23's motion sign-off is already scheduled as Wolf's call at 3.3.

**REAL DEFECT FOUND BY WOLF PLAYING, FIXED, MERGED AND PLAY-TESTED (PR #15, 2026-08-06).** Wolf's verdict after merging: "merged and tested .. works now" — dwarves resume wandering after a distant job. `main` = `bf1f5c0`, tree clean, nothing unpushed. "after digging or
channelling dwarves don't start idling". He was right, and it is the most important thing to come
out of 3.2.

**THE DEFECT:** `Wander::home` is written once at spawn (`lib.rs:1160`) and NEVER updated; save/load
preserves it verbatim. `wander` only accepts candidates within `WANDER_RADIUS = 3` of `home`, but
A* has no limit, so a dwarf walks any distance to a job. At **Chebyshev >= 5 from home every
neighbour is still >= 4**, so the candidate set is empty every future tick — the dwarf sets `Idle`
and NEVER MOVES AGAIN. Distance 4 is the boundary (one step inward reaches 3 and it self-recovers),
which is why only genuinely distant jobs strand a dwarf, and why two of my repro attempts failed.
Reproduced live: one wall tile 14 squares from a spawn, claimed by a dwarf 56 tiles away, motionless
for **1,289 consecutive deltas**. Introduced BY 3.2 — before A* travel, dwarves could never leave
their radius.

**THE FIX (Wolf's choice of three):** re-home the dwarf in `release_claim` — the ONE funnel every
release path goes through (completion, no-op completion, vanished job, retry, AND cancel). A dwarf
settles where work took it and never returns to spawn; `// NOTE:`-ed. 76/76 mutations killed
including the new `release_claim does not re-home the dwarf`.

**WHY EVERY REVIEW LAYER MISSED IT, and this is the transferable part:** no AC covers "a dwarf
resumes wandering after finishing". AC8 only requires a job-holder is never reported `Idle` — and a
stranded dwarf reports `Idle` PERFECTLY CORRECTLY. Even the Feature Auditor, the layer built for
exactly this, passed it: its live dig was near the camera over 200 frames, and stranding needs a
DISTANT job plus time. **The lesson is not "add a layer" — it is that Wolf's play-test is a required
step, not a nicety.** Second time a real defect surfaced only when he actually played (FR23's
"boring" was the first).

**Two testing lessons from the fix itself:** (1) my first version of the scenario test PASSED — it
picked a solid tile buried in rock with no standable face, so no dwarf ever walked there and it
proved nothing. Tightened to require a workable face AND to assert `items()` is non-empty so it
cannot pass vacuously. Same can't-fail class as the `<= 80`-on-80-cells assertion at 3.1. (2) Always
`cargo clean -p` AFTER `mutate.sh`, not only before — doing only "before" this session poisoned two
review layers' test runs with stale mutated artifacts.

**Consequence for 3.3:** dwarves now accumulate at the work face instead of returning to spawn.
`dwarves_stay_standable_and_near_home` still passes only because it makes no designations — it is
now meaningful only for a colony that has never worked.


## Milestone 1 is DONE (2026-08-07): story 3.3 merged as PR #16

The walking-skeleton sentence is true end to end — designate → dig → carry → stockpile — proven
headless in `scenario.rs`, live over the real daemon/client, and 34/34 on the mutation table.
Implemented DIRECTLY by Opus, not Codex (quota exhausted until 2026-08-12); see
[[review-cost-economics]] for what that cost.

**Wolf's AC17 verdict, and it matters for epic 4's framing:** the feel floor (NFR2) passed — "looks ok
for 2d tui game atm" — but FR23's icy-grim-identity-in-motion is only PROVISIONAL. His words: "not
sure how much more visually pleased it could be without designing own font or something ... most
likely we need to get to the 3d first to say." So further investment in the 2D presentation has low
expected return by his own judgement, and the identity question is re-opened at
`4-1-behold-the-fortress-in-depth`. A future "make the TUI prettier" story needs an explicit case
against that. Recorded in `deferred-work.md` too.

**Epic 3 review of 3.3 found one real product defect no test of mine caught:** two carriers racing for
the last free stockpile tile left a PERMANENT stack of stored stones (both counted as stored, both
jobs retired, invisible to the sim). Wolf chose repair over prevention — only the lowest-id uncarried
stone on a tile counts as stored, so extras stay loose and re-haul themselves. AC3 was amended at
review to say so.

## Epic 3 retrospective — done 2026-08-08 (`epic-3-retro-2026-08-08.md`)

**MILESTONE 1 IS DONE:** FR26 walking-skeleton gate passes headless, gate re-verified green at the
retro, 143/143 mutations killed across the epic. `epic-3` and `epic-3-retrospective` are both `done`.

**FR23 is NOT signed off.** NFR2's feel floor passed and Epic 2's "boring world" complaint is
answered, but Wolf's icy-grim-identity verdict is **provisional at 2D only** and moves to the depth
view. Standing scope judgement now in `deferred-work.md`: the glyph client is near its ceiling, so any
future "make the TUI prettier" story needs an explicit case against that entry.

**Wolf's decisions:** take A (silence-based layer kill), B (per-layer `CARGO_TARGET_DIR`), C (R1
disjoint hunter territories — taken *against* my recommendation, and coherent only because A+B land
first), E (fresh-context review, enforced), and R2 (one verification pass per review). Keep all four
review layers and re-measure. **Split story 4.1 into 4.1a (raycast renderer) / 4.1b (sub-voxel
creatures)** — deliberately reversing the 3.2 one-story precedent, on the evidence 3.2 produced.

**Before 4.1a can start:** T3 (deterministic opening camera z + `--z` flag) is a *prerequisite*, not
deferred work — 4.1 is a pure-camera epic and the nondeterministic z already produced one false
"the feature does not work" verdict with exit 0. E1 rewrites `epics.md` for the split.

**Hard scheduling fact:** Codex quota exhausted by 3.2; resets **2026-08-12 07:00 UTC**. Story
creation can proceed; delegated dev cannot, without paying the Claude rate (~2x, see
[[review-cost-economics]]).

**The retro's own headline lesson:** *"encoded" and "correct" are different claims.* The Epic 2
time-box item read `done` for all of Epic 3 while the rule it encoded was wrong and cost coverage on
two of three stories. See [[review-layer-reliability]].


## STORY 4.1a DEV DONE — AWAITING REVIEW IN A FRESH SESSION (2026-08-08)

**Branch `4-1a-behold-the-fortress-in-depth`, 5 Völundr commits, NOTHING PUSHED, no PR.** Sprint
board and story file both say `review`. Gate green, **22/22 mutations killed**, live capture taken.
Dev cost **$31.89 / 230 turns / 63 min** — direct Opus, against 3.3's direct-Opus $57.78/310.

**NEXT MOVE: run `bmad-code-review` FROM A NEW SESSION.** Not from the dev session — the
`REVIEW RUNS IN A FRESH CONTEXT` fact in `bmad-code-review.toml` is a stated PRECONDITION and
3.3 measured the cost of ignoring it at 2.3x per turn. Dev and review share a model family this
time (both Opus), so the different-LLM lever is absent; say so in the review.

**Codex was genuinely unavailable and it was VERIFIED, not assumed** — a trivial probe through
`scripts/codex-handoff.sh` returned `You've hit your usage limit ... try again at Aug 12th, 2026
7:00 AM`. Wolf chose implement-now over waiting four days. See [[codex-delegation-runbook]] for the
config drift the probe also exposed.

**Three things from this story worth not rediscovering:**
1. **`protocol::EntityKind` has ONE variant, so "the dwarf index ignores EntityKind" is an
   unkillable mutation** — deleting the filter is a semantic no-op and no test at any level can
   observe it. Recorded in the mutations file rather than dropped. This is NOT 3.3's rejected
   "no scenario can tell them apart" argument; that one fell to a unit test one level down.
2. **A change-detection test comparing whole captured frames is FALSE EVIDENCE.** The status line
   carries the tick, which differs every frame regardless of what the picture does — so the test
   passes against a completely frozen render. Strip the status line and give it a stillness control.
   Same class as 3.1's `<= 80`-on-80-cells assertion. I shipped it and caught it myself.
3. **Backticks in a `git commit -m "..."` message are COMMAND SUBSTITUTION in bash/zsh.** A message
   mentioning `` `v` `` launched vim and hung the tool for two minutes. Write commit messages to a
   file and use `-F`.

**Also learned:** this devpod did NOT have `NO_COLOR` set this session, contrary to what the older
memory implies — check `${NO_COLOR}` rather than assuming, and set it explicitly when testing that
path. A dead `pub fn`/field in a binary crate fails `clippy -D warnings`, so a helper cannot be
committed one task ahead of its call site; that is why 4.1a's tasks 2 and 3 share a commit.


---

## 2026-08-08 (late) — PHASE ONE CLOSED. NEXT = MILESTONE 2 PRODUCT STATEMENT

**`main` = `cae8907` (PRs #18 and #19, both merged by Wolf). Working tree clean, nothing unpushed,
nothing stranded on any local branch, local main synced.** The **4.1a archive branch survives on the
remote and MUST NOT be deleted** -- it is the only copy of the raycast code; verified `raycast.rs`
is still absent from `main`. [[post-merge-branch-trap]] fired again exactly as recorded — Wolf merged and my checkout
landed on `main`; verified rather than assumed.

**Story 4.1a was REVIEWED, then the plan changed under it.** Full story in
[[client-strategy-pivot]] — read that first, it is the load-bearing one. One-line version: the
raycast depth view works, Wolf judged it live as "quite far from wow effect", and he had actually
wanted an **isometric** camera. 3D-in-TUI abandoned, Unreal dropped for **Bevy**, TUI kept as the
2D debug client **and the deterministic assertion instrument**.

**THE REVIEW ITSELF WAS THE BEST ONE THIS PROJECT HAS RUN — record the numbers, they settle three
open Epic 3 experiments:**
- **4 of 4 layers completed, zero kills, zero coverage holes** — first clean four-layer run since
  3.2. Epic 3 lost 3 of 4 layers at both 3.1 and 3.3. The fix was the PAIR: silence-based time-box
  (P1) **plus** per-layer `CARGO_TARGET_DIR` (P2). Four layers ran cargo + live daemons concurrently
  without contending. **The coupling rule was right — isolation was the real fix, not the timer.**
- **R1 territory split: SUPPORTED.** 3 convergences across ~13 findings vs Epic 3's 1-in-8. Revert
  rule did not fire. Caveat: 4.1a's diff was client-only so `sim-core`'s hunter had an EMPTY
  territory and was reassigned to `raycast.rs` — exercised in spirit, not literally.
- **Review cost $28.24 / 306 turns / 70 min, of which 4 subagent transcripts = 56.5% of tokens.**
  FIRST honest figure — before T1 landed, over half of a review was invisible. **Do not compare to
  any pre-2026-08-08 row; they are all undercounts.** This is the new baseline.
- **I dismissed a HIGH and was right to.** Blind Hunter claimed the renderer "lies about distance"
  and proposed multiplying by `|direction|`. That is the standard perpendicular-distance
  (`perpWallDist`) construction — its own evidence showed a flat surface returning one uniform `t` —
  and the proposed fix would have INTRODUCED fisheye. What survived was a comment that describes the
  opposite of the code. **Verify a layer's reasoning, not just its confidence.**

**NO EPIC-4 RETROSPECTIVE** — Wolf: "either have a very short one or just skip it". The four
measurements above are recorded in §6 of
`_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-08.md` instead. `sprint-status.yaml`
still shows `epic-4-retrospective: optional`; leave it unless Wolf asks.

**NEXT MOVE, and do NOT skip ahead to stories:** Milestone 2 = product statement first
(`bmad-product-brief` / `bmad-prd`), then architecture (`bmad-architecture`), then epics. Wolf chose
"new milestone, close phase one now" precisely so Bevy gets a real planning pass instead of an epic
bolted onto a TUI plan. **Action item E4-P1 is the test of whether the lesson took: Milestone 2's
replacement for FR24 must say what the boss should SEE and FEEL and name no rendering technique.**

**Still open, unchanged by all this:** forge propagation (see the 2026-08-08 blocked_on note — both
blockers are now FIXED here, the merge is owed and the FORGE session does it); and the
`bmad-code-review.toml` nonexistent-mise-path defect in [[frostvein-toolchain]], which I propagated
into all four layer prompts again today — harmless because the rustup shim saves it, but it is a
false instruction still sitting in a config file.

---

## 2026-08-09 — M2 PRD FINALIZED. NEXT = bmad-architecture for the Bevy client

**`_bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/` is `status: final`** (prd.md ~300
lines + addendum.md + .memlog.md + 4 reviewer/reconcile files). E4-P1 passed: FR24's replacement is a
"Visual Target & Game Feel" section + FRs stated as outcomes; **no line names a rendering technique**.
Sources: Wolf's `docs/narrative.md` + two concept images (guidance, NOT acceptance bars). Uncommitted —
docs/ has the narrative + jpgs untracked too.

**Wolf's decisions in it (do not re-ask):** GDS module NOT installed (borrowed game-design vocabulary
instead); new PRD, not M1 amendment; FR27–FR37 + NFR5–NFR8 continue global numbering; scope = renderer
+ minimum light content (trees FR27, torches/campfire FR28, carried lanterns FR29 as moving-light
testbed, protocol vocab FR30) — minecarts/walls/crystals/rivers/off-map stay OUT; **full TUI input
parity (his call over my camera-only rec)** incl. 3D picking FR36; z-levels survive into 3D
(mousewheel candidate, collides with zoom — open, addendum); one zoom continuum diorama↔vista;
**cap 10–14 stories, cut order = lanterns first, then parity narrows**; boot-frame wow due in FIRST
THIRD; parity rule = Bevy catches up first, TUI updated only for sim changes/bugs that affect it;
**ART REVERSAL ON THE RECORD: M1's "code-authored, never assets, ever" overturned** — Wolf is an
artist w/ AI tooling; procedural-first, asset pipeline (MagicaVoxel/bevy_vox_scene) only when a story
forces it, dwarves expected first; tech-art-guidelines deliverable owed then. Valheim = reference
(atmosphere-over-cheap-geometry lesson, addendum).

**Sign-off gate (the FR24 fix, both halves):** every visually subjective story needs Wolf-approved
"here is what you will see" artifact BEFORE implementation AND Wolf comparing built result vs
artifact live before done.

**For the architecture pass (blocking + inputs):** NFR6's measured Bevy feel bar is BLOCKING there;
gate.sh needs the bevy-crate sibling probe (NFR8); open design-and-test questions in addendum
(z-slice control, world-edge/vista treatment, vista mountain-silhouette vs FR2's rolling hills).
Reviewer findings all triaged and applied; rubric verdict Approve, no critical/high.

---

## 2026-08-09 (later) — M2 ARCHITECTURE SPINE FINALIZED. NEXT = bmad-create-epics-and-stories

**`architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md` is `status: final`** (~290 lines,
AD-13…AD-18 continuing global numbering; reviews/ holds 5 gate outputs; .memlog.md the rationale).
`docs/architecture.md` updated in step (six crates, M2 section, still ten-minute). **Parent M1 spine
amended in place** with dated 2026-08-09 notes (AD-6 tui-edge relaxation, graph supersession, raycast/
mouse-touch/Unreal/color-table staleness) — both spines mutually true. All uncommitted.

**The new ADs (do not re-derive):** AD-13 client-core crate (mirror + ALL delta application, protocol-only
dep, both clients consume; tui adopts it in an M2 story — load-bearing, on no cut list; amends AD-6);
AD-14 mirror-then-project with world-projected vs client-local entity classes; AD-15 interpolation is
presentation (blend N-1..N only, snapshot clears previous, rewind snaps never animates; carve-out list
incl. dig chips); AD-16 trees are tiles (TreeTrunk/TreeFoliage, dig drops NO item — Wolf's call),
everything-that-glows is an entity (Entity gains light: Option<LightKind>{Torch,Campfire,Lantern},
EntityKind +Torch,+Campfire — THE entire sanctioned M2 wire diff), appearance = gui data table, wire
never carries RGB; AD-17 evidence ladder (client-core headless CI / gui MinimalPlugins tests / --capture
instrument tested-but-never-golden-imaged, out of gate.sh; sign-off gate opening half precedes it);
AD-18 client-core owns the mirror contract (previous=entities only, mandate not cap; rect rule BINDING
for commands, one normalization helper, simd validates+drops).

**Env + bar:** WSLg verified live (glxinfo: D3D12→RTX 4080 Laptop, Mesa 25.3.5); NFR6 = 60 fps working /
≥30 vista, full world+lights, frame-time overlay (FpsOverlayPlugin needs non-default bevy_dev_tools
feature); ack ~200 ms any client. Bevy 0.19.0 verified current (2026-06-19, same train as bevy_ecs
0.19.0 in lockfile; MSRV 1.95 ok). Dozen/Vulkan risk real → first gui story MUST prove a window renders
fast (non-negotiable); client-core lands before either client consumes it. Native Windows deferred
(no unix-only code in gui/client-core). Six crates: sim-core, protocol, client-core, simd, tui, gui.

**Epic-planning inputs:** PRD cap 10–14 + cut order (lanterns → parity narrows), first-third wow,
sequencing facts above; gate.sh grows gui + client-core probes (NFR8); tech-art guidelines procedural
half owed by first gui visual stories (NOT gated on asset pipeline). Cold builds get slow with full
bevy — per-layer CARGO_TARGET_DIR matters more.

---

## 2026-08-09 (later) — M2 EPICS + STORIES DONE. NEXT = bmad-check-implementation-readiness

**`_bmad-output/planning-artifacts/epics.md` now carries Milestone 2**, appended to the M1 file
(Wolf's call over a separate `epics-m2.md`) with epic numbering continuing at 5 so story ids and
`implementation-artifacts/` filenames never collide. Frontmatter: `stepsCompleted: [1,2,3,4]`,
`milestone: 2`, 9 inputDocuments incl. both concept jpgs. ~1057 lines, uncommitted like the rest of
M2 planning. All four workflow steps ran; validation clean.

**4 epics / 11 stories, inside the 10–14 cap:** Epic 5 *The Cold Boot* (5.1 worldgen trees+emitters ·
5.2 client-core + tui adopts it · 5.3 gui crate + WSLg envelope proof + orbit camera + capture
instrument · 5.4 the light/sky/aurora pass = **wow beat 1 at story 4 of 11**, inside the first-third
mandate); Epic 6 *The Valley Lives* (6.1 interpolation/work/flicker = **wow beat 2** · 6.2 lanterns);
Epic 7 *Into the Mountain* (7.1 z-slice mechanism chosen by testing · 7.2 designation/zone rendering
+ working-zoom legibility); Epic 8 *The Boss Gives Orders in 3D* (8.1 picking · 8.2 mouse designation
· 8.3 speed/save/load + skeleton walks in 3D). All 11 FRs and all 22 UX-DRs covered.

**Authoring calls made in the breakdown — do NOT re-litigate:**
- **UX-DR1…UX-DR22 exist.** No UX contract was ever written, so the PRD's Visual Target + the
  six inverted 4.1a anti-requirements + the addendum's two open questions were extracted at full
  granularity into the epics file's UX section. Every visual AC traces to one.
- **`LightKind::Lantern` is NOT added at 5.1** — it lands in 6.2 with FR29, so cutting lanterns
  (first on the cut list) leaves no dead variant on the wire. Torch/Campfire land at 5.1.
- **5.3 is written as ALLOWED TO BE UGLY.** Grey boxes orbiting at speed is a pass; every visual bar
  sits in 5.4. That is the only thing keeping 5.3 inside one dev session, and it is what protects the
  first-third wow — any story inserted before 5.4 pushes beat 1 out of the first third.
- **7.2 renders designations/zones driven by a TUI client on the same daemon**, issuing nothing from
  gui. Proves AD-4 zero-game-logic, and makes designation *rendering* survive the Epic 8 cut.
- **5.2's before/after TUI capture guard must be sabotage-tested** (break a mirror rule → comparison
  must fail), else it is [[self-referential-test-antipattern]] a fifth time.
- **Defect found and fixed during step-4 validation:** the cut path contradicted 8.3's AC. If
  FR35/FR36 shrink to camera+speed, 8.2 vanishes and "I designate a dig in the Bevy client" becomes
  unmeetable — Epic 8's cut note now states the substitution (designate from TUI, watch in gui).

**Flex order if reality pushes back:** 8.1+8.2 collapse if picking is easy (→10); 5.2 or 5.4 split if
either overruns (→12); then the PRD cut order fires (lanterns → Epic 8 shrinks to speed control).

---

## 2026-08-09 — M2 READINESS DONE, FIXES APPLIED, PLAN COMMITTED + PUSHED. NEXT = sprint-planning

**`implementation-readiness-report-2026-08-09.md` exists** (M2's own; the 08-02 one is M1's).
Verdict at assessment: **NEEDS WORK — light**, 9 substantive + 4 minor issues. **All 9 applied to
`epics.md` the same session on Wolf's immediate yes; status after remediation = READY.** Both
commits gate-green and pushed to `origin/m2-bevy-client-planning` (`adf20c5` plan fixes + report,
`c718671` a sprint-status forge note). **The M2 planning set is no longer uncommitted.**

**The result worth carrying: epic-level coverage was already complete and the coverage maps in
`epics.md` were honest — 11/11 FRs, 4/4 NFRs, 22/22 UX-DRs, no forward dependencies, no technical
epics, story count inside the cap. The PRD and the M2 spine drew ZERO findings.** Every issue was
one layer down and all nine had the SAME shape: **the plan stated the right thing in narrative and
did not bind it in an acceptance criterion.** That is the pattern to look for at the next readiness
pass — not missing requirements, missing ACs for requirements the prose already promises. A dev
agent works the ACs.

**The nine, as fixed (do not re-derive):** (1) FR35's `cancel designation` reached 8.2's "I want"
sentence but never its ACs — the one parity command with no AC in the milestone. (2) **5.4 and 6.2
named no observability instrument at all**, against the binding repo rule — 5.4 being the wow-beat-1
story whose sign-off artifact IS a capture per AD-17; both now have `gui --capture` ACs. (3) No
contingency if 5.3's WSLg render envelope fails, with 8 of 11 stories downstream — Epic 5 now records
the ladder (force GL via `WGPU_BACKEND`, then the spine's deferred native Windows build). (4) 6.1's
dig face could be occluded (z-slicing arrives at 7.1, its capture has no `--z`) = story 3.3's false
failure set up to run again; now pinned to a named surface-visible face designated from the TUI.
(5) The **vista mountain silhouette** — last of the spine's three decisions owed — had no owning
story, the silent stretching the spine named by that phrase; now a decision-on-the-record AC in 5.1,
cross-referenced from 5.4. (6) The fps overlay had no off switch and would burn into every sign-off
artifact; now toggleable, off in `--capture`. (7) Epic 8 never said whether UX-DR22 binds it, where
Epic 5 had set the precedent of saying so; now states gate applies to 8.3, not 8.1–8.2, with reason.
(8) 5.3 had no split contingency despite being the largest story and the only unproven-until-run one;
both 5.2 and 5.3 now have named split lines. (9) **CM2 was claimed more confidently than it holds** —
beat 1 completes at **36%**, at the edge, and the plan's own reserved splits push it to 42%; a split
of 5.2 or 5.3 is now the trigger to re-check CM2, not a free move.

**Left unfixed deliberately:** 5.2 is developer-facing (mitigated by an identical-output +
sabotage instrument, and AD-13 makes it load-bearing); 5.1 touches a TUI state layer 5.2 retires;
sizing is uneven (5.2/5.3/5.4/6.1/8.3 heavy, 6.2 light). **And two standing warnings: no UX document
exists at all, and the Bevy client has NO interaction spec anywhere** — no keymap, no mode model —
every interaction decision deferred into stories. That is right per the FR24 lesson but concentrates
unbudgeted design work in Epic 8, the cut-risk epic.

**Wolf's calendar item, not a plan change: 5.4 and 6.1 cannot be closed by a dev agent** — they end
on his eye by design (AD-17 rung 3), and 5.4 needs its "here is what you will see" artifact approved
BEFORE implementation starts, on the critical path to the first-third wow.

**SPRINT PLANNING DONE the same day (`36b7de3`, pushed).** `sprint-status.yaml` now carries Epics
5-8 + 11 stories + 4 retrospective entries, all `backlog`; M1 statuses untouched, 42 action items
carried over. Validated: 8/8 epics and 22/22 story keys match `epics.md`, no illegal statuses, YAML
parses. One deliberate extra row — `4-1b-dwarves-in-depth: dropped` has no story header in epics.md
(split out at the Epic 3 retro, dropped before a story file existed); it stays, because removing it
erases why the 3D ambition moved to M2. **The readiness findings were written into the board as
comments, not left only in the report** — the board is what an agent reads first: cut order, the
5.2/5.3 split lines, 5.3's envelope risk + fallback ladder, 5.1's owed silhouette decision, 6.1's
surface-visible dig face, CM2 at the edge, and the two stories no dev agent can close.

**NEXT MOVE = `create-story 5.1`** (`5-1-the-world-grows-things-that-glow`). Nothing is
`ready-for-dev`; creating it flips `epic-5` to `in-progress`. Fresh context window. 5.1 is a clean
start: sim-side, deterministic, headlessly testable, observable through the existing TUI instrument,
and it carries only readiness fix #5 (state the silhouette decision on the record).

**Branch state 2026-08-09 session end:** `m2-bevy-client-planning` = `36b7de3`, pushed, working tree
clean, gate green on all three commits. **No PR opened yet** — M2 planning lives on the branch.

---

## STORY 5.1 CREATED (2026-08-09) — NEXT = dev-story 5.1

**`5-1-the-world-grows-things-that-glow` is `ready-for-dev`, `epic-5: in-progress`.** 21 ACs,
10 tasks, baseline `36b7de3`, create cost **$23.40** (195 turns, 45.5% of it four Explore subagents
Wolf authorised for the run). Uncommitted, still on `m2-bevy-client-planning`
([[post-merge-branch-trap]] — the story says branch `5-1-the-world-grows-things-that-glow`).

**TWO DECISIONS WOLF TOOK AT CREATION — recorded in the story AND in the sprint board, do NOT
re-litigate:**
1. **The vista mountain silhouette is YES and is shaped in 5.1** — the last of the spine's three
   owed decisions, closed. In-grid terrain within 128×128×32 is tuned to give the skyline peaks the
   aurora can backlight, to a *headless-testable* target (height span ≥16 z-levels, min ≤10, max
   ≥26) because no 3D client exists until 5.3. 5.4 judges the look under the sign-off gate and does
   **not** re-open worldgen. Rationale: FR33's "slice into the mountain" presumes in-grid mountains;
   discovering this at 5.4 would re-open a story two slots back with CM2 already met at the edge.
2. **The camp is real — 5.1 clusters all five dwarves into it.** FR28 places torches "at the
   dwarven starting camp" and **no camp existed**: measured live, the default seed scatters the five
   dwarves up to ~104 tiles apart. 5.1 picks a deterministic camp site nearest the map centre and
   spawns them there. UX-DR5 wants the eye to land on the encampment by warm/cold contrast alone,
   and a campfire with no dwarves round it is not one. Consequence accepted: the pinned seed-42
   spawn positions and the terrain fingerprint both change, loudly.

**MEASURED FACTS ABOUT THE DEFAULT WORLD, taken live from a running daemon — do not re-derive:**
seed is the constant `SEED = 0xF005_7E1A` (`crates/simd/src/main.rs:20`) and **`simd` has no
`--seed` flag; the port is positional only** (`simd 7413`). Surface heights run **12–20 on a
32-level world** (8 levels of relief — a plain, from an isometric vista). Dwarves at
`(96,68,20) (78,48,17) (16,96,15) (120,48,20) (75,33,15)`. `opening_z` with no `--z` is **17**.
Under a pipe the TUI viewport is **100×40** fixed on the map centre, so a capture shows only
x 14..113, y 45..82 — **a camp outside that window is invisible to the recipe**, which is half the
reason camp-nearest-centre was chosen.

**THE VERIFICATION RECIPE WAS EXECUTED AT CREATION (P6), and executing it caught two traps that
would otherwise have shipped in the story:** (a) my first probe passed `--port`/`--seed` flags
`simd` does not accept — it captured nothing and **exited 0**, the exact indistinguishable-from-
broken failure P6 exists for; (b) counting glyphs with **`tr -cd '█'` is wrong and lies quietly** —
`tr` works on bytes, the box glyphs share leading UTF-8 bytes, and it reported **3520 dwarf glyphs
for a five-dwarf world**. Use `grep -o '<glyph>' | wc -l`. The half that cannot run yet (the four
new glyph counts) is stated in the story as an exact command plus the exact non-zero observation the
dev agent must produce.

**Two `deferred-work.md` entries whose triggers this story actually fires, and which it closes:**
`:46-53` (border-biased spawn — 5.1 *is* the real embark-site rule that entry named as its revisit
trigger) and `:317-324` (the `tui` entity draw loop silently skipping any non-`Dwarf` `EntityKind`
— `Torch`/`Campfire` is the second variant that entry was waiting for). Also relevant and NOT
actioned: `:196-209`, `MAX_SAVE_BYTES` is a hand-picked constant already broken once by added state.

**Traps written into the story that a dev agent would otherwise hit:** amplitude alone will not make
a skyline (`clamp_steps` caps neighbour delta at 1, so `NOISE_SPACING` must move with the
amplitude); `heights[i]` is the topmost **solid** z so standable is `height+1`; the height clamp
`[3, dims.z-2]` leaves one free level and a crown on a peak clips out of the grid; `light` is
**always serialized** (`"light":null` for dwarves) rather than `skip_serializing_if`, to avoid a
second meaning for "absent" next to AD-8's section-level absence-is-deletion; `Entity.state` stays
required with emitters carrying `Idle`, because optional would exceed AD-16's sanctioned wire diff;
`bridge.rs:381` is a **runtime** material allow-list that no compiler catches; `to_save`'s
`filter_map` silently drops an entity missing a component.

---

## 2026-08-10 — 5.1 MERGED, 5.2 CREATED AND HANDED TO CODEX

**`main` = `305aa03`.** Sprint board: `epic-5: in-progress`, `5-1` **done**, `5-2` **in-progress**
(delegated), `5-3`/`5-4` and all of Epics 6–8 `backlog`.

**THE MEASURED-FACTS BLOCK ABOVE IS STALE — 5.1 changed every number in it.** Corrected, taken live
from a running daemon at 5.2's creation:

| fact | was (pre-5.1) | now |
| --- | --- | --- |
| surface height span | 12–20 (8 levels) | ≥16 levels, min ≤10 / max ≥26 |
| dwarf spawns | scattered ~104 tiles apart | clustered in the camp at **(64,64,9)** |
| `opening_z` with no `--z` | 17 | **19 — and it shows NO camp, NO dwarves, NO lights** |
| entity ids | 5 dwarves | 0–4 dwarves, **5–9 emitters** (campfire 5, torches 6–9) |
| TUI viewport under a pipe | "100×40 fixed" | **FALSE** — `frame_size()` returns the real terminal size; 100×40 is only the fallback when `terminal::size()` *fails* |

**Pin `--z 9` in every TUI recipe from now on.** Wolf's ruling at 5.1's review: the camp being
invisible at the default level is left alone because the TUI is a 2D instrument, not the product, and
5.3's Bevy window is the real viewer. Revisit trigger is 5.4's sign-off gate.

**A live TUI capture cannot be byte-compared across runs, and 5.2's AC was written around that.**
Measured twice with the identical command: terrain glyphs are stable (`│=6 ♠=48`, seeded and static)
but entity glyphs and the tick are not (`†` 24 vs 21, `♨` 6 vs 3, `☺` 22 vs 30; capture opened at
tick 31 in one run) because the client connects at a wall-clock-dependent tick and the dwarves
wander. So Epic 5's "run the capture before and after adoption, output identical" is **unmeetable as
literally written**. 5.2 splits it: the byte-exact guard is the stub-daemon capture suite in
`crates/tui/tests/client.rs` (deterministic, in CI); the live run is a range check with the two
terrain figures pinned. Say this rather than re-deriving it at review.

**5.2's shape, decided at creation from reading the code rather than the epic:** there is **no client
world type today** — `tui` holds a `protocol::Snapshot` as mutable state and mutates it in
`main.rs::apply` (25 lines), and that function *is* what moves into `client-core`. Two traps found by
reading: (a) **`sim-core::apply_command` already normalizes min/max and clips to dims**, so simd's new
log-and-drop rect validation sits *in front* of it and sim-core must not be touched; (b) the wire's
entity order is globally ascending by id **today only by accident** — `bridge` builds it as
`dwarves().chain(emitters())`, so keying by `Id` makes it structural. Also: `view.rs`'s whole test
module builds worlds through one helper (`empty_snapshot`, only two `Snapshot` literals in the file),
which is what makes 5.2's "existing assertions unmodified" AC meetable rather than a wish — the
lesson from 5.1's AC2, which was unmeetable for exactly the opposite reason.

**Known YAGNI tension carried deliberately into 5.2:** AD-18 mandates `previous_entity()` and
`changes()` on the mirror, and **neither has a live caller** — the TUI re-renders whole frames. The
obvious candidate (driving `needs_redraw` from `changes()`) is a trap: `--frames N` emits one frame
per server message, so skipping "unchanged" frames would change the instrument's own output. Left
inert by design, decision-tested per the seam-exercised rule, with the deferral naming **5.3** as the
wiring story.

**Codex quota was back on 08-10**, three days before the "out until 08-12" the memory carried —
[[codex-delegation-runbook]]'s probe rule paid for itself again. Banner read `gpt-5.6-terra` /
`reasoning effort: high`, i.e. no drift this time.

---

## 2026-08-14 — 5.3 MERGED; 5.4 CREATED BUT GATE-PARKED. NEXT = Wolf's artifact ruling

**5.3 DONE + MERGED** (PR #23, `main` = `e1fef5c`): the envelope holds via the **native Windows
client** on gingerspice (cross-compiled `gui.exe`, NVIDIA Vulkan 591.74, 146 fps at grey-box
fidelity) after the gingerspice-devpod ladder failed on both backends; details in the 5.3 story
file's Dev Agent Record — do not re-derive.

**STORY 5.4 CREATED, `ready-for-dev` (2026-08-14, create $27.21, authored on `claude-fable-5`
per frontmatter).** 21 ACs. Key authoring calls: NFR6's AC re-venued on the record to the
native-Windows vehicle (the WSLg observation cannot occur; WSLg figure stays owed to the Epic 5
retro); 5.3's AC26 debt (never-run `--ignored` capture self-test) and the never-seen-live
ramp-complete valley folded in as ACs 17–18; warm/cold encoded as a sabotage-able data
invariant (light colors R > B, night terrain B ≥ R).

**THE GATE IS PARKED — this is the blocking fact.** Task 0 candidate artifact produced same
day: a geometry-true software-iso mock rendered from a live snapshot (its exposed-tile pass
independently reproduced 5.3's 53,365 draw-set oracle), stored with renderer + README in
`_bmad-output/implementation-artifacts/5-4-signoff/`. **Wolf's reaction: "quite far away from
the drafts" (the `docs/*.jpg` concept references); ruling deferred overnight, 2026-08-14.**
No implementation until he approves an artifact (UX-DR22 opening half). Options he holds:
approve, direct iteration of the mock, or a different artifact route (e.g. AI-generated
reference in the concept style over our framing). Note for the record: the gate catching a
mismatch at one image is it working — 4.1a paid a full story for the same class.

---

## 2026-08-15 — 5.4 GATE OPEN: ARTIFACT APPROVED. NEXT = dev-story 5.4

**Wolf APPROVED the sign-off artifact** (UX-DR22 opening half met):
`5-4-signoff/candidate-artifact-2026-08-15.png`, reached in a four-pass iteration on his
directions over the parked 08-14 candidate — (1) draft-2 framing: camp foreground at ~0.48/0.78
of frame, elliptical fog dissolving the far valley into the sky (depth fogs fast, lateral slow —
a horizon, not a dome; fog blends to the sky gradient AT THE TILE'S SCREEN ROW, the fix that
killed the floating-diorama read), sky/aurora the top third, foreground falls into darkness;
(2) trees as snow-laden spruce sprites; (3) visible two-block trunks; (4) slimmer first skirt so
the trunk reads. Wire truth held every pass: the renderer prints the 5.3 AC13 oracle
(53,365 exposed tiles) on every run.

**Two rulings taken with it, recorded in the signoff README + story Task 0:**
- **Tree density stays FULL wire truth (704 trees).** A 65%-thinned variant was rendered and
  offered with the caveat that delivering it = a sim-core worldgen change re-opening 5.1's
  pinned fingerprint. Wolf chose full density → **no worldgen change rides on 5.4**. Variant
  deleted; `--thin 0.65` regenerates it if that ever reopens.
- The valley surface is a per-tile snow/ice mix (8k/8k tiles); the mock desaturates ice toward
  snow so it reads as mottling. The Bevy client should blend materials, not checker them.

**Task 0 is checked in the story file; sprint-status updated (last_updated 2026-08-15).**
Snapshot capture for the mock: `5-4-signoff/capture_snapshot.py` runs `target/debug/simd 0`,
connects, takes the connect snapshot (~7 MB, not committed). The closing half of UX-DR22 is
still owed: Wolf views the built boot frame live against this exact image, on the
native-Windows vehicle. **NEXT = dev-story 5.4** (story cannot be CLOSED by a dev agent).

---

## 2026-08-15 (later) — 5.4 HEADLESS DEV DONE → REVIEW. NEXT = code review in a FRESH session

**5.4's headless half is implemented and independently verified; Status = `review` in story +
sprint-status.** Branch `5-4-the-cold-boot`, **10 Völundr commits off `e1fef5c`, nothing pushed.**
Delegated to Codex **`gpt-5.6-terra`/high — Wolf's explicit choice at handoff** (asked because the
banner had drifted from his 08-09 sol edit; he kept terra). Dev metric recorded from THREE rollouts
(main + question-stub + continuation; nested self-gate children folded): **273 turns / 24.0M tokens /
$4.64 / 11pp quota / ~40 min.** My independent verification, not Codex's claims: `scripts/gate.sh`
GREEN, **5/5 mutations KILLED** (incl. the added snow-flank one), diff touches only
`crates/gui` + `docs/` + implementation-artifacts (AC21 holds).

**THE RUN NEEDED A CONTINUATION, and the reason is a NEW Codex failure mode worth keeping:** the
first session's self-gate pass 1 found **three real defects** (P1: capture range-check read raw
screenshot bytes as RGBA — on BGRA targets warm-pixel check inverts, false-evidence instrument;
P2: mirror items got no mesh → invisible, a 5.3 regression; P2: snow cap painted whole cubes →
the uniform coat AC7 forbids). Codex fixed NONE, launched pass 2, got window-cut, and its
handback said only "no review conclusion is claimed" — **the findings were only in the run log**
(`Full review comments`). ALWAYS grep the run log for pass output before accepting a handback.
Continuation session fixed all three TDD-style with proper per-fix commits (`db54e77`, `ae696f5`,
`31e60a1`). Also: **commit-cadence floor violated AGAIN in session 1** (Tasks 1–5+10 in ~2
commits despite the prompt stating the floor) and `cd289b6`'s message is wrong for its content
(staged-retry under stale message). Both recorded in the story's Completion Notes for the review.

**Still open on 5.4 (by design):** Task 6's edge-treatment comparison (only fog implemented;
rim-darkening untried — the try-two-candidates decision moves to the vehicle), Task 7 by-eye,
Task 8 NFR6 readings, Task 9 captures + AC26 Windows test run, and **Wolf's AC19 closing
sign-off. Review does NOT close this story; only Wolf does.**

**NEXT = bmad-code-review on 5.4 in a FRESH session** (P5 precondition: this session carried the
dev orchestration; fresh-context review is the one measured free lever, 2.3x). Spec = story file;
diff = `5-4-the-cold-boot` vs `main`. Note for the metrics row: a FOREIGN forge Codex run
(`ep-15-us-02`, cwd `/workspace`) was live during this window — pin transcripts explicitly.

---

## 2026-08-15 (evening) — 5.4 REVIEWED + LIVE-FALSIFIED → in-progress. NEXT = patch session (FRESH)

**Review ran in a fresh session as required: $53.25 / 379 turns recorded (`--phase review`,
transcript pinned).** All four layers completed WITH live execution — first zero-coverage-hole
run since the reliability fixes: Sonnet hunters (whole gui diff each), Opus auditors. Verdict:
headless substrate solid (gate independently re-verified, 53,365 oracle recomputed twice, every
hop WIRED — nothing is dead code), but the COMPOSED FRAME fails the artifact. **Outcome: 11
patches as action items in the story's Review Findings, 6 defers appended to deferred-work.md,
6 dismissed, 4 Wolf rulings. Story + sprint-status → `in-progress`.**

**SAME-DAY LIVE RUN (Wolf, native Windows vehicle) CONFIRMED the predictions by eye:** emitters
are orange dots with NO warm pools; NO snowfall in frame; NO aurora or stars; scene too dark to
judge ice/caps/vista at all. **The lighting rescale is therefore the GATING patch — land it and
live-check it FIRST; every other visual finding is unobservable until the value range exists.**
(Initial "gui is crashing" resolved as the by-design loud exit without a reachable daemon.)
`gui.exe` cross-compiled and ready at `target/x86_64-pc-windows-gnu/release/gui.exe`.

**The four HIGH patch targets:** (1) light table ~1/1000 of Bevy reference (torch 900 lm vs
default 1,000,000; campfire beats the cold fill only within ~0.5 units) — rescale + encode the
warm-vs-cold budget as a test; (2) atmosphere authored around the render origin while the world
is render x 0..128 / z −128..0, camp at (64,9,−64) — all 16 snowflakes outside the boot frustum,
aurora buried in terrain, stars ~1 px; add position-pinning tests; (3) fixed fog 85–180 vs zoom
clamp 4–500 — full vista fogs to a flat rectangle; couple fog to camera distance; (4) cap
predicate material-blind — all 3,650 exposed ice tops capped white (AC8 dead; the toy-world
stone case never occurs on the seed: caps = 3,650 ice / 3,817 snow / 9,525 foliage / 0 stone).
MED patches: SnowCap entities carry NEITHER partition marker (spawn ClientLocal — WorldProjected
would get them despawned by entity reconciliation); AC6 test is `>=20` not "every" + partition
tests check disjointness never totality; warm/cold invariant asserts the test's own literals
(self-referential class, 5th sighting); `night_lighting()` unpinned, no test/mutation.

**Wolf's four rulings (do NOT re-ask):** (1) CAP RAMPS TOO — fold into the cap patch (3,813
exposed ramp tops currently bare); (2) AMEND AC11 in place (7th AC-text defect: "chosen by
testing" is vehicle-only) + soften tech-art-guidelines' premature fog verdict; (3) STRENGTHEN
the AC16 check — named warm-pixel-count floor above what emitter faces alone produce (currently
vacuous: emissive bypasses exposure so `warm>0` passes even with all PointLights missing);
(4) ACCEPT CUBE TREES, re-baseline AC19 to light/sky/snow/framing — the approved artifact drew
"spruce sprites instead of per-tile boxes" (`artifact_render.py:7`, trunk direction at `:234`)
but the wire has foliage-skirted stacks and NO task covered tree presentation; deferred to a
later story. **RETRO NOTE: Task 0 artifact scripts must not substitute geometry the renderer
is not tasked to produce** (the artifact half of the 4.1a class).

**Triage facts settled against Bevy 0.19 source — do not re-derive:** emissive_exposure_weight
defaults 0.0 → emissive BYPASSES exposure (`pbr_functions.wgsl:840`) — the Feature Auditor's
"capture aborts, zero warm pixels" was WRONG, corrected at triage; fog DOES apply to unlit
materials (`main_pass_post_lighting_processing`, `fog_enabled` default true). Also for the
Epic 5 retro: **R1's territory map predates `crates/gui`** — this diff sat in territory the
split never assigned; both hunters ran whole-diff. Layer scorecard: Feature Auditor carried the
review (all six frame HIGHs, one corrected); Acceptance Auditor converged independently on
three + unique finds (hollow invariant, unpinned 4th table, both AC-text defects, live seed
census); hunters converged on ramp gap + test weaknesses, unique: id-collision probe (edge),
ruled-out list (blind).

**Working tree at session end:** branch `5-4-the-cold-boot`, 10 commits, PLUS uncommitted
review records (story file with Review Findings, sprint-status, deferred-work.md, metrics) —
the patch session's first commit should record them. NOTHING pushed (Wolf's explicit hold).
**NEXT = fresh patch session: 11 patches (lighting FIRST, one verification pass at the end,
extend the mutation table for new tests, record `--phase review-patch`), then one vehicle
sitting for Tasks 6–9 + the AC19 comparison.**

---

## 2026-08-15 EVENING — 5.4 PATCH ROUNDS 1–4 DONE, PARKED MID-FLIGHT. NEXT = ROUND 5

**PARKED on Wolf's call ~19:40 ("need to park this now and continue tomorrow morning").** Branch
`5-4-the-cold-boot`, working tree CLEAN, everything committed, **NOTHING PUSHED** (hold stands),
no background job running. Story Status stays `in-progress` — Tasks 6–9 are vehicle-only and AC19
is Wolf's alone. All 11 original review action items are CLOSED. `scripts/gate.sh` GREEN and
**20/20 mutations KILLED** on the orchestrator's own runs (never on Codex's report).

**Cost: 4 Codex rounds at `--phase review-patch` = $2.34 + $1.29 + $3.01 + $4.15 ≈ $10.79 and
22 PERCENTAGE POINTS of weekly Codex quota (14% → 37%).** Two more rounds of this size approaches
the zone that blocked dev for six days at 3.2 — watch it before launching round 5.

**THE TRAP THAT COST A WHOLE VEHICLE SESSION — the live half needs a REBUILD step, always.**
Wolf ran the client, reported "looks the same", and it was: `gui.exe` was built 13:24 while the
earliest patch commit landed 13:58. The cross-compile is a MANUAL step in the vehicle recipe and
**nothing in the delegated flow triggers it**. Any handback ending "go look at it on the vehicle"
must name rebuild + re-copy explicitly, and the binary's mtime should be checked against the last
commit before drawing any visual conclusion. Same family as exit-0-is-not-a-result: an unchanged
frame says nothing about the code until the binary is known to contain it.

**TWO CAPTURES NOW EXIST IN `5-4-signoff/` — the closing comparison is images, not memory.**
`capture-2026-08-15T1717-boot.png` (round 3) and `boot2.png` (round 4). Wolf's verdicts:
"works now — well it's a start", then "not there yet". **Hard live numbers:**
`projected 53365 terrain cubes` (AC18 oracle CONFIRMED) and `capture range check:
warm-lit pixels=17648` (1.9% of frame — warm pools measured real; the old floor of 100 vs ~64
emitter-faces-only was useless, now 3,000).

**`artifact_render.py` IS THE SPEC — read it, do not re-derive the target by eye.** The approved
artifact encodes its own numbers: `HORIZON = H*0.30` (sky gets the top THIRD, comment at `:7`),
camp anchored at `W*0.48`/`H*0.78` (`:147`), palette `MAT` at `:54-62` (stone 60,70,92 · soil
56,52,62 · ice 104,128,170 · snow 136,150,178), **`SNOWCAP = (158,170,196)` — a DISTINCT cap
colour brighter than snow**, `SPRUCE_SIDE (42,60,62)` / `SPRUCE_SNOW (172,186,210)`, taper radii
0.82/0.60/0.38 (`:241`), and `topc = SNOWCAP if m in ("stone","soil","snow") else base` (`:256`).
**That last line CONFIRMS ice-keeps-blue and foliage-uncapped are correct** — do not re-open them.

**ROOT CAUSE OF THE "WAFFLE" VALLEY, now fixed:** worldgen gives every tree a **3×3 foliage skirt
flat on the ground** (`worldgen.rs:206-215`); `has_snow_cap` treated foliage as terrain, so ~9,525
ground-level skirts got bright snow slabs and buried the landform. Exactly what AC7's "loaded
branches, NOT a uniform coat" forbids — the predicate met the letter and inverted the intent.
**Wolf REVERSED his morning cube-trees defer on seeing the frame** ("let's fix trees and valley
landform still"): the two decisions turned out to be one, and accept-cube-trees silently meant
accept-no-landform. Fixed client-side without touching `is_exposed`, so the 53,365 oracle holds.

**ROUND 5 IS SPECIFIED IN THE STORY FILE (four defects from `boot2.png`) — read it there:**
D1 camp sits at screen x≈0.227 vs the artifact's 0.48 (**orchestrator spec defect: round 4's
framing test pinned only the VERTICAL fractions**; `BOOT_COMPOSITION_OFFSET` pushes along world −Z
while the camera is yawed 0.7, so it slides along the camera's right vector — push along the view
direction and pin x too); D2 the aurora renders as three flat green rectangles because
`unlit:true` + `AlphaMode::Blend` on a `Cuboid` gives uniform colour and hard edges — **the
mechanism is wrong, not the position**; a procedurally built gradient `Image` is the honest fix
(hand-rolled data, NOT an asset pipeline, no new dep); D3 the far skyline is a raw grid edge
(~20% fogged; AC11 forbids it) — retune `fog_falloff` for the new camera and assert a fog
FRACTION, not "before complete fog"; D4 the field is still darker/muddier than the artifact and
the palette already matches, **so the gap is lighting, not the table**.

**PROCESS FINDINGS WORTH KEEPING:** (1) **Codex's self-gate finally completed** (rounds 1–2 were
truncated by the sandbox command-parent timeout, zero usable passes) and independently caught the
close-zoom composition bug — the offset now scales by `(distance/BOOT_DISTANCE).min(1.0)`.
(2) **Round 4's handback named two P2s that never reached the story record** and the orchestrator
had to transcribe them — a finding that lives only in a handback message dies at the session
boundary; the round-5 prompt now demands self-gate findings land in the Dev Agent Record, fixed or
not. One of them was mischaracterised as pre-existing: `foliage_scale` reads TWO tiles up while
`reconcile`'s dirty set propagates only ±1, so round 3 widened the read radius without widening
the invalidation radius — **new, not inherited**. (3) **Piping `mutate.sh` through `tail` masks its
exit code** — round 1's `APPLY-FAILED` showed as exit 0 because `tail` succeeded; capture the exit
status before any pipe.

---

## ROUND 5 of the 5.4 patch cycle — 2026-08-16 (orchestrator implemented DIRECTLY)

Wolf's call this session: **I implement, not Codex** — to keep the measurement harness in the
session and to protect the weekly Codex quota (held at 37%; two more Codex rounds would have
approached the 3.2 lockout). 6 commits, gate GREEN on each, **34/34 mutations KILLED**, tree
clean, nothing pushed.

**THE METHOD IS THE TAKEAWAY.** Rounds 2–4 predicted from Bevy's shader model and it
mispredicted the field brightness by ~4x. Round 5 measured instead — a **pure-Python PNG
decoder** (zlib + unfilter, ~50 lines; the devpod has no numpy/PIL) comparing the capture to
`candidate-artifact-2026-08-15.png`. Every fix then became arithmetic:

- **Field was ~18x too dark in LINEAR light** (valley-floor median sRGB luminance 21 vs the
  artifact's 123). Ambient 2,000→30,000, directional 1,500→60,000 (directional share raised
  11%→24% to keep modelling), emitters ×12.
- **Warm/cold is carried by HUE, not luminance.** The artifact's camp is only ~1.3x the field
  in luminance while R/B goes 0.72→0.97. The old `warm >= cold * 3.0` floor was satisfiable
  both by the table that shipped black and by a white blowout. Now a band (1.2–6.0x) plus a
  chromatic term.
- **The camp's PEAK was nearly right while its MEDIAN was 5.8x low** — the signature of too
  little fill, not too little torch. Diagnose with both statistics, never the mean alone.
- **D1 was solved offline.** An independent projection model in Python reproduced the capture's
  camp position to five digits, locating the cause exactly: the composition offset carried a
  28.6-unit component along the camera's RIGHT vector. **Keep that model** — it also solved the
  aurora band, the star shell, and the silhouette depth profile without a single build.
- **Fog-alone is FALSIFIED as the edge treatment** and this is now on the record: the entire
  visible skyline IS the map boundary, at depths 86–145, while the camp sits at 71. Any fog
  range that hides it erases the valley. Replaced by a **world-space rim dissolve** over the
  outer 10 tiles; the draw set is unchanged so the 53,365 oracle holds.
- **Sky geometry radius is constrained by the ZOOM CLAMP.** At zoom 500 the camera orbits 426
  units from world centre, so any sky ring smaller than that puts the camera outside it and
  swings the aurora in front of the valley. Curtain at 600, stars at 650, with a test that
  zooms to the clamp and asserts containment. Express "hugs the horizon" as an ANGLE from the
  eye, never a raw height — a height bar is meaningless once the radius moves.

**PROCESS SCARS FROM THIS ROUND — read before the next one:**
1. **The stale-cache trap fired again on a CLEAN TREE**, second sighting (round 3 lost a whole
   self-gate to it). `recorded_camp_snapshot_projects_exactly_five_warm_point_lights` failed
   `left: 0, right: 5` at HEAD with no local changes; `cargo clean -p gui` fixed it. Its
   signature is IDENTICAL to the emitter sabotage. **Suspect the cache before the code.**
2. **COMMIT BEFORE RUNNING MUTATIONS.** I edited the mutation table while `mutate.sh` was
   executing it (bash reads scripts incrementally — that run was void), then cleared the
   leftover sabotage with `git checkout -- crates/` and destroyed uncommitted work.
3. **Mutation anchors go stale silently.** Two failed to APPLY after a `cargo fmt` reflow and a
   changed indentation count — reported as survivors while pinning nothing. Re-run the table
   after any reformat.
4. **The sabotage table caught a defect in MY OWN test**: the median oracle used values whose
   mean and median were both 100, so a mean-based implementation passed the one test meant to
   distinguish them. Always pick oracle values where the statistics disagree.
5. **Don't poll background jobs.** `mutate.sh` takes ~15 min; polling it dominated a $55.44
   round. Arm one blocking watcher and stop.
6. **Ledger comparisons across tools are apples-to-oranges.** The "$10.79 for rounds 1–4"
   figure counts CODEX ONLY — the orchestrator's supervision for those rounds was never
   recorded. Round 5's $55.44 includes diagnosis, verification and record-writing.

**NEXT = the live vehicle run, and it is Wolf's alone**: Tasks 6–9 (edge comparison by eye,
framing vs artifact, NFR6 at working zoom + full vista, AC26 cross-compiled capture self-test)
and AC19. **Rebuild and re-copy `gui.exe` first.** Both capture checks PRINT before they assert
(`warm-lit pixels=N ground-median-luminance=M`), so the run yields numbers pass or fail; if M
lands 40–70 the next scale factor is `123 / M`, not a guess.


---

## ROUND 6 — 2026-08-16, after Wolf's boot3 live run

boot3 verdict: better, but snowfall broken etc. Measurement against the artifact found the
round-5 value work overshot: ground median 156 vs 123 (26% over sRGB), shadows flooded (p05 87
vs 28 — ambient too high), blue-green cast (saturated light tints multiplying onto blue
materials). Plus two scatter bugs only a real frame could show:

- **Correlated-sampling bug**: stars used fractions of the SAME irrational for azimuth and
  height — fract(i*0.381966) = 1 − fract(i*0.618034) — so all 300 lay on one helix (dotted
  lines across boot3's sky). **Rule: two axes need two independent irrationals (R2/R3
  sequences).** Pinned by a bin-spread test on (azimuth±height) mod 1.
- **Snowfall formation**: shared columns + one speed + one respawn height = permanent
  synchronized rows. Scatter + per-flake speeds + phase-preserving wrap.
- **The light budget divides, it doesn't just scale**: a uniform brightness factor fixes the
  median and floods the shadows. Ambient sets the shadow floor; directional models lit faces.
- **Capture value check is a BAND now** (floor 70 / ceiling 180) — each end has caught a real
  failure (round-4 black 21, boot3 washed 156).
- **Process win**: dry anchor-check (grep every mutation `old =` string against the tree)
  before running the suite — caught 5 stale anchors in seconds vs round 5's 15-minute-run
  discovery. Round 6 cost $39.75 vs round 5's $55.44; the polling waste is mostly gone (one
  blocking watcher per suite run).


---

## STORY 6.1 CREATED (2026-08-16) — NEXT = Task 0 (Wolf's sign-off artifact), then dev-story 6.1

**`6-1-the-world-moves` is `ready-for-dev`, `epic-6: in-progress`.** 19 ACs, baseline `1f262d8`,
create cost **$19.77 / 129 turns**. gui-only story: no wire change, no sim change, so the parity
rule's backward half does not fire.

**Everything in it was measured live, not reasoned about** (`simd` on the shipped seed, real `tui`
key sequence, a python wire watcher):
- **The named dig site is `[58,68,9]`–`[64,69,9]`** — exactly 8 mineral tiles, every one the top of
  its column and unoccluded from the boot camera, projecting to screen `(0.49,0.70)`–`(0.53,0.73)`.
  `Tile::Ramp` is **not** diggable (`sim-core:1339-1341`), so a rect must contain `Solid`.
- **A 8-tile dig is over in 52 ticks (~5 s)**; WORK_TICKS is 5, so one tile's work phase is half a
  second. **An instantaneous "is a dwarf working?" sample is a coin flip** — the instrument must
  assert a maximum over the run, never a value at the shot.
- **47% of ticks contain a dwarf position change** with zero commands issued. That is the floor
  under the aliveness AC and why its window is ≥100 ticks.
- Rubble persists: no stockpile placed → the 8 stone items sit at the dug tiles forever.
- The scriptable designation is `tui <port> --z 9 --frames 3 --key d,h,h,h,h,h,h,j,j,j,j,enter,l,l,l,l,l,l,j,enter`
  (cursor opens at 64,64 and `d` resets it there).

**THE FINDING WOLF HAS TO RULE ON, and it is Task 0's blocking clarification.** UX-DR14 — the wow
beat 2 bar — says "a dwarf picks something up and carries it". **That is not on the wire and no
client can show it.** `World::items()` reports every item at its last resting position and a
carried stone keeps its pickup tile until `release_claim` drops it (`sim-core:1462-1471`, `:674-687`,
`:891`); the TUI's "carrier" glyph is only "a dwarf standing on a tile that has an item"
(`tui/src/view.rs:240-251`). Making it visible is a sim + wire change, i.e. a separate story. Sign
beat 2 without it, or spin that story — but do not let it be discovered at the live viewing, which
is the 4.1a/5.4-artifact failure shape.

**The two clobber sites are the whole implementation risk.** `reconcile` re-inserts a snapped
`Transform` (`gui/src/project.rs:286-292`) **and** re-inserts `point_light()` (`:293-297`) on
EVERY frame for every existing entity. Left alone, both the blend and the flicker are computed and
thrown away — present-but-inert, the seam-exercised defect. Two ACs exist purely to make that a red
test. Related: alpha must come from the measured delta cadence, not a hardcoded 10 tps (the wire
carries `Speed` but no tick rate), which also makes pause and fast-forward work for free.

**Snowfall makes "the capture changed" vacuous.** 5.4's `--ignored` capture self-test compares
whole-file bytes; the aurora and 96 snowflakes animate every frame, so it now passes on atmosphere
alone. 6.1 re-anchors it to the dig-site screen window and adds a sim-derived motion line
(ticks observed / position changes / mid-blend frames / max working / items) with `--expect-work`.


---

## 2026-08-16 (later) — 6.1 TASK 0 GATE: WRITTEN HALF DONE, RULING TAKEN, PAIR OWED BY WOLF

**`6-1-the-world-moves` is `in-progress`** (story file + sprint board), baseline `1f262d8`, still on
`main` — **no branch cut, no code written, no Codex handoff issued.** AC1 forbids an implementation
commit before Wolf approves the artifact, and he has not yet.

**The written half of Task 0 is delivered:**
`_bmad-output/implementation-artifacts/6-1-signoff/what-you-will-see.md` — part (b) the four
additions with the look each aims for (dwarves slide, light breathes, chips at the dug tiles,
rubble that stays), part (c) a **seven-line** "what you will NOT see". Six lines were the story's;
**I added a seventh — dwarves remain rigid cubes with no walk cycle** — on the 5.4 lesson that an
unstated absence surfaces at the live viewing, not before it. Part (a), the before/after capture
pair, is **NOT produced**: it is vehicle-bound (no devpod can open a window) and Wolf is taking it.

**WOLF'S TWO RULINGS, 2026-08-16 — do not re-litigate:**
1. **The carried stone: sign wow beat 2 WITHOUT it.** No sim story spun, no wire change on 6.1.
   **UX-DR14's "picks something up and carries it" clause is FORMALLY NOT DELIVERED in M2** —
   recorded as a decision, never blurred into the beat.
2. **The capture pair is being TAKEN, not skipped.** The written-only fallback was offered and
   **declined**, so the closing sign-off compares against two real images rather than memory.

**THE FACT THAT DECIDED RULING 1, and it was NOT known at story-creation:** haul jobs are derived
**only from stockpile tiles** (`crates/sim-core/src/lib.rs:319`, `:260`), and **6.1 deliberately
places no stockpile** — that is what makes the rubble pile up at the face. So **no dwarf ever
carries a stone in this story's scenario at all**: the gap is not merely unrendered, it does not
occur, and no viewer can notice its absence. The underlying wire truth (re-verified this session,
not inherited): `World::items()` reports every item at its stored `Pos` (`:1462-1471`), and a
stone's `Pos` is rewritten in exactly one place — `release_claim`, i.e. **the drop** (`:696-707`) —
so a carried stone would sit still at its pickup tile and then teleport. `uncarried_stones`
(`:674-687`) proves the sim knows which stones are in transit; that knowledge never reaches the wire.

**WHAT UNBLOCKS THE STORY — Wolf runs this on gingerspice, then approves.** No rebuild needed: the
shipped 5.4 `target/x86_64-pc-windows-gnu/release/gui.exe` (built 2026-08-16 08:30) takes both
captures. `simd 7451` in WSL → `gui.exe 7451 --capture 6-1-before.png --frames 600` → the TUI
designation key sequence from the story's Verification → `gui.exe 7451 --capture 6-1-after.png
--frames 600`, both stored in `6-1-signoff/`. **The moment the Task 0 checkbox is ticked, the
delegation launches:** the Codex handoff prompt is already drafted and `cargo fetch` already run to
prewarm the offline sandbox. Codex's scope is **Tasks 1,2,3,4,5,7,8** — Task 6 (live vehicle) and
Task 9 (Wolf's closing sign-off) are vehicle/human-bound and must be left unchecked, never faked.

**Gate-session cost: $5.80 / 62 turns / 8 min**, recorded as `dev | claude` on
`metrics/6-1-the-world-moves.md`. Transcript span was verified (12:42–12:50, this session only)
before trusting the row — [[metrics-attribution-traps]] did not fire.

## 2026-08-17 — 6.1 TASK 0 GATE CLOSED AND APPROVED; CODEX DEV RUNNING

**Wolf took the pair on gingerspice and APPROVED the artifact. AC1 is MET, the gate is OPEN.**
Branch **`6-1-the-world-moves` exists** with one commit — `bf30d80` "Close story 6.1's sign-off
gate" (Völundr, pre-commit gate green, **nothing pushed**). Codex launched in the background on
`gpt-5.6-terra` / effort **high** (banner checked — no drift to luna/medium), session
`01a00f06-ac14-7d21-8992-ba08c966669f`, scope **Tasks 1-5, 7, 8**.

**THE FINDING THE PAIR PRODUCED, and it is the gate earning its keep for two screenshots.** Wolf's
reaction was *"did not see the difference"* — and he was **right, and the code was also right**.
Measured rather than argued: the named dig site projects to **64×43 px = 0.30% of a 1280×720 boot
frame**; **2,255 pixels differ** between the captures and **1,625 (72%) fall inside the window
`CameraRig::project_world_point` predicts**. So designation landed, 8 tiles emptied, stone items
rendered, change concentrated exactly where the camera math said — merely **sub-legible at the
vista**. `6-1-signoff/6-1-digsite-inset.png` (marked full frames + 7× nearest-neighbour crops)
makes it readable; at 7× the pale shelf is visibly cut into a dark trench with a stone item below.

**Recorded as line 8 of "what you will NOT see" — THE DIG FACE READS AT WORKING ZOOM, NOT AT THE
BOOT VISTA.** Three consequences now binding, all written into the handoff prompt so Codex cannot
"helpfully" violate them: (1) do NOT touch framing, camera, dig site or tile scale to make the dig
more visible — Wolf explicitly declined re-picking the site, since that would invalidate the
live-verified exposure/occlusion/projection/52-tick figures and force AC7+AC9 amendments; (2)
**AC15's windowed comparison is now measured fact, not theory** — whole-frame inequality here is
satisfied by snowfall alone; (3) AC7's rubble and AC8's chips are sub-legible at boot framing **by
design**, so their evidence is the headless tests and the instrument counts, never the wide capture.
**The opening frame's visible weight therefore rests on dwarf motion and light flicker** (camp-scale),
not on the dig face — worth remembering when judging wow beat 2.

**TRANSFERABLE LESSON for any future visual-story artifact:** a before/after pair is only evidence
if the change is *legible at the framing the pair uses*. Compute the projected screen footprint of
the thing that changes BEFORE relying on a wide-shot pair, and ship the magnified inset alongside
it. This is the same family as 3.3's zero-of-every-glyph-with-exit-0 and 2.2's motion-rendered-as-
stillness: an evidence channel that technically works and communicates nothing.

**Second ruling reconfirmed at approval:** the dig site is **NOT re-picked**. Ruling 1 (beat 2 signs
without UX-DR14's carried stone) stands from 2026-08-16.

**Owed next:** verify Codex on exit (never trust exit 0 — grep 401, confirm files, check branch +
Völundr author, re-run `scripts/gate.sh` independently, `scripts/mutate.sh` alone), record the dev
metric with `--tool codex --transcript` pointing at rollout `01a00f06...` (NOT bare `--tool codex`),
then chain to `bmad-code-review`. **Tasks 6 and 9 remain vehicle/human-bound and must never be
faked** — AC13/16/17 stay open, and only Wolf closes this story.

## 2026-08-17 — 6.1 HEADLESS HALF DONE → REVIEW. The ticked-but-undelivered failure class, caught

**Branch `6-1-the-world-moves`, 11 commits, NOTHING PUSHED, gate green, 306 workspace tests
(57 in gui), 8/8 mutations killed.** Story Status `review`; sprint board `review`. Codex dev cost
**$2.82 total across two runs / 163 turns / 5pp of weekly quota** (37%→43%).

**THE FINDING THAT MATTERS, and it is a NEW instance of an old class.** Codex's first run exited 0,
reported Tasks 1-5,7,8 complete, passed the gate, killed 4/4 mutations and had exact AC19 scope —
**and had ticked FOUR subtasks it never delivered**, precisely the seam ACs (4, 5, 6, 11) the story
was written to protect. I falsified AC6 **by sabotage, not by reading**: removing both
`blend_projection` and `flicker_projection` from the live `Update` tuple — deleting the story's
entire headline outcome — left the suite **54/54 GREEN**. Cause: `projection_systems` was called
only by `run()`; `headless_app()` built its own wiring and never called it, so the shared-
registration mechanism AC6 mandates existed and was **inert**. Its doc comment claimed "for both the
live app and headless tests", which made it *read* as satisfied — worse than an obvious gap.
AC4/AC5/AC11 had no app-level test at all, and no mutation, because there was no test to mutate.
`tests/headless.rs:24` was a **duplicate of a unit test wearing an integration test's name**.

**A continuation handoff fixed it, and the shape of that prompt is the reusable part:** it carried
the RED evidence verbatim, told Codex to reproduce the sabotage itself before fixing anything, and
made **a mutation per falsified AC a required pasted deliverable**. Second run closed all four in
**five commits, one per AC — cadence floor met**. The same sabotage now turns **4 tests RED**, and
the mutation table went **4 → 8, all killed**.

**Codex's run-two honesty is the behaviour to keep and to cite.** Its sandbox detached output before
the gate and mutation runs printed conclusions, and it wrote: *"I cannot honestly claim a green gate
or completed self-review… I also did not update the Dev Agent Record with a fabricated mutation RED
table."* Same agent, same story, opposite behaviour to run one's false ticks. **The lesson is NOT
"Codex is unreliable" — it is that a checkbox is worth exactly what its verification is worth, which
is why the orchestrator re-runs everything. Sabotage is the only check that distinguishes a test
that guards a seam from a test that merely mentions it.**

**The self-gate is a COVERAGE HOLE for this story — no conclusion on EITHER run** (run one "stopped
after initialization"; run two was correctly not started rather than half-run). Worth noting a
working self-gate is exactly what should have caught an inert `projection_systems`.

**The mutate.sh stale-artifact trap re-fired and nearly cost me a phantom defect:** after the table
restored source, `cargo test -p gui` failed with the *mutated* threshold's message while `git diff`
showed clean. `cargo clean -p gui` → green. **Clean the crate AFTER mutate.sh, not only before** —
recorded at 3.1's review and still live.

**Owed next:** code review (configured `on_complete`, not yet run — it needs Wolf's go since it
spawns layers), then **Tasks 6 + 9 and ACs 13/16/17 on the vehicle: cross-compile, TUI designation,
the motion capture pair with `--expect-work`, the F3 fps readings at working zoom AND full vista
labelled `gingerspice / native Windows / NVIDIA`, the `--ignored` capture self-test, a `tui` client
open beside the Bevy window, and Wolf's live wow-beat-2 sign-off.** Only Wolf closes this story.


## 2026-08-19 — story 7.2 created (ready-for-dev, blocked on Task 0)

Ran bmad-dev-story; it HALTed correctly — **no ready-for-dev story existed**. 6.1 in-progress with
only Task 6 (vehicle) and Task 9 (Wolf) left, 6.2 and 7.1 in review, 7.2 backlog. Wolf chose to
create 7.2. Authored on Opus with five parallel research subagents; $28.89 create cost (41% of the
session's tokens were the subagents).

**Ninth AC-text defect, caught at creation.** The epic requires mark entities be "created, updated
and despawned by sim `Id`". Designations are `BTreeMap<Pos, DesignationKind>` and zones
`BTreeSet<Pos>` — neither is an ECS entity, neither has an Id at sim, save or wire level, while
Entity and Item both do. AD-14 is internally inconsistent here: it names designations and zones as
world-projected AND says reconciliation is keyed by sim Id. AC10 written keyed by POSITION, with the
AD-14 amendment recorded as owed to the spine (AD-13's explicit-amendment precedent).

**The story is much smaller than the epic implies:** every noun is already on the wire and already
mirrored, `client-core` needs zero change, items already render, channel designations are real end
to end. It is a `gui`-only story.

**The recipe-execution rule paid for itself again.** Running the recipe live found that
**designations are consumed by the dwarves**: 6.1's 8-tile site is fully dug in ~52 ticks against a
~110-tick capture window, so the obvious recipe photographs an empty site and exits 0. Measured an
8x12 rect instead — 79 marks (17 of 96 silently dropped by the diggability filter), decaying
68/59/51 at t+40/60/100 and plateauing at **50** from t+120 as the rest become unreachable. Also:
a TUI-dragged 2x2 stockpile yields **2** zone tiles, and a zone tile sits one level ABOVE the rock
(digs z=9, zones z=10) so `--z 9` hides the zone entirely.

Baseline at creation: **328 workspace tests green**.

## Story 8.1 — headless half to review, 2026-08-25

Codex dev in **one run, six commits**: the commit-cadence floor was met **without a follow-up for
the first time**, and Codex left Tasks 5 and 7 unticked rather than claim a gate its sandbox never
showed it. Dev cost **$2.37 / 5pp weekly quota / 98 turns** — cheap because no self-gate passes ran.

**The finding, caught by mutation round 1 and not by any green test:** `--cursor` parsed, validated,
and was then **silently dropped by `run()`**. Replacing the `ScriptedCursor` insert with a discard
left the whole suite green, because the only test wrote that resource **by hand** — pinning the
resource→pick half and never the flag's own path. This is 7.2's `--distance` lie verbatim, so the
class survives being fixed once per story: **the next `--flag` will do it again unless its test
executes the production wiring.** Remedy that works: extract the wiring out of `run()` (which needs
a socket and a window, so nothing could execute it) into a plain function a test can call on a real
parsed `Args`. See [[verification-defect-relocates]] and [[stacked-branch-ac-defect]].

Round 2: **6/6 KILLED**, zero APPLY-FAILED; full gate green on a cold rebuild, **382 workspace
tests**. Scope proved structurally — zero files changed outside `crates/gui`, so AC9's client-local
claim is verified rather than asserted.

**Open, not closable by any agent: Task 6 / AC12** — nothing observed on the vehicle, NFR6 with
picking unmeasured.

Two items handed to review: (1) the instrument's "expected" oracle is a screen-space
nearest-within-32px match, **depth-blind**, so it can disagree with a correct pick at the vista —
it fails loud, so it can't manufacture 7.2-style false evidence, but a Task 6 mismatch may be the
instrument's defect rather than the pick's; (2) AC8's mutual-inverse pin landed in
`tests/headless.rs` instead of extending `transform.rs`'s round-trip pin as the story specified.

## 2026-08-25 — story 8.1 CODE-REVIEWED → `in-progress`

4 layers + 1 narrowed re-run, **no coverage holes**. $44.47 / 418 turns / 42.5M tokens, **56.2% of
it inside the layers** — against Epic 3's $45.52 baseline that omitted its own fan-out entirely, so
like-for-like this review was cheaper, not dearer. 5 decisions ruled by Wolf, **12 patches left as
action items for a fresh session**.

**Process lessons worth more than the findings:**

- **Flag a standing rule BEFORE proposing a plan, not after.** I offered to apply patches inline and
  Wolf confirmed — then I found the patch-in-a-fresh-session rule and had to reopen it. He chose the
  rule. Correct outcome, avoidable churn: check the standing rules against the plan first.
- **The salvage step and the one-re-run rule both paid for themselves on the same finding.** Blind
  Hunter timed out at ~18 min; salvaged rather than killed bare, its partial carried the run's one
  scare — an oracle disagreeing with `first_visible_hit` on 24/36,000 rays at full scale. Its one
  permitted narrowed re-run settled it: **all 24 are oracle artifacts** (fixed 0.005 sampling step
  vs corner grazes 0.000065–0.0018 wide, 3×–77× narrower). A bare kill would have left the story's
  core algorithm as an open HIGH. See [[review-layer-reliability]].
- **A layer's headline claim can be wrong while its underlying point stands.** Edge Case Hunter said
  AC8's mutual-inverse test "does not exist" — it does, under a name without the word "inverse". But
  its real point survived: both sides of that round-trip are pinned to 16:9, so it cannot detect the
  aspect mismatch it was written to catch. Verify the claim, then salvage the insight.
- **I gave all four layers a false environment fact** — that `--capture` runs in a devpod. It does
  not (`bevy_winit`, no DISPLAY). Both Opus auditors tested it and caught me rather than inheriting
  it. Consequence was material: AC10/AC11's instrument **has never executed in any process anywhere**
  and joins AC12 as vehicle-bound. See [[devpod-no-graphics-userspace]] — the rule is broader than
  "no window for the TUI"; it kills any Bevy path taking `DefaultPlugins`.

**R1 territory split still has no M2-crate mapping** — hand-adapted for the second story running
(6.1 was the first). Owed a ruling at the Epic 8 retro. Convergence this run was **4-in-22**, the
first favourable datapoint R1 has produced.

**AC5 is NOT met and 8.1 ships knowing it:** the hover slab is invisible on every tile with a drawn
tile above it (cliff faces, corridor walls, shaft sides) and near the campfire. Both deferred by
Wolf awaiting final gfx per [[art-gates-visual-judgement]] — but **8.2 designates by pointing at
exactly those faces**, so this lands before 8.2 ships, not after.

## 2026-08-26 — STORY 8.1 DONE, MERGED (PR #34, `f9df762`). M2 at 9/11.

Picking works. Ray from the rendering camera (`Camera::viewport_to_world`), DDA over integer
render-space cells, hit **cell centre** through `render_to_world` — the only axis conversion in the
path. 389 tests, sabotage table 6 → 9 rows all KILLED, full gate green cold four separate times
(twice by agent, once by Wolf, once by the pre-push hook). Bookkeeping commit `cca118a` went
DIRECT TO MAIN, which is the house pattern here (`1f262d8`, `c8494e7` precede it).

**AC12 met on the vehicle: >140 fps sustained at BOTH working zoom and full vista** (floors 60/30),
on a fresh post-patch rebuild, source commit `3f50178`. **Why the margin is expected, not luck:**
`update_pick` casts ONE ray per frame bounded by the world diagonal, so picking's cost does not
scale with tile count. Carry this into any later story that fears a per-frame ray.

**The review-patch round cost $15.16 / 130 turns**, phase `review-patch` (tool=claude). Patches are
NOT delegated to Codex — the ledger separates codex-dev from the rework a review requires.

**PROCESS RULE CONFIRMED, do not re-derive: a patch round is NOT re-reviewed.** 5.4 (12 patches),
6.x (8), 7.1 (13), 7.2 (14) all went patch -> live-gate -> sign-off with no second review. Review
once, patch in ONE verified pass, close. I offered Wolf a second review and he pushed back
correctly; the precedent was checkable in sprint-status all along. See [[review-cost-economics]].

**THREE THINGS 8.1 LEAVES BEHIND, none blocking, all owed:**
1. **AC5's RENDERED half is unmet** — the hover slab is buried under any drawn tile above it.
   Deferred to 8.2 by Wolf awaiting final gfx ([[art-gates-visual-judgement]]). **8.2 designates by
   pointing at exactly those vertical faces, so it cannot slip again.**
2. **M2-7 bit a FOURTH time** — no build script, no SHA stamp in `gui`, so the vehicle build
   wall-clock went uncaptured and every session re-litigates "is this binary fresh?" by hand.
   Put it to Wolf at the Epic 8 retro as load-bearing, not housekeeping.
3. **R1's territory split still has no M2-crate mapping** — hand-adapted for the second story
   running. Also owed a ruling at the Epic 8 retro. See [[review-layer-reliability]].

**The inert-seam class recurred and then RELOCATED inside one story** — `--cursor` inert at `run()`,
fixed by extracting `insert_capture_resources`, and the review then deleted *the call to it* with
the suite still green. `run()` now splits into `connect_to_daemon` + `configure_client_app`. What is
still uncovered is `run()`'s own three lines, which need a socket and a window. See
[[verification-defect-relocates]].

**NEXT: story 8.2 — Designate with the Mouse.** Not yet created.

**2026-08-28 (end of day).** Branch `9-4-trees-fewer-and-distinct-from-the-ground`, stacked on
`9-1-the-frame-stops-blowing-out`, **27 commits unpushed, nothing on main**. Working tree clean,
full gate green.
- **9.1 — in-progress, VEHICLE-BLOCKED.** Code-reviewed (4 layers, no coverage holes), 8 patches
  applied. **Wolf observed on the vehicle that shadows did NOT close the campfire blow-out** —
  the story's own predicted W1+W2 outcome. Withheld levers (intensity/amplitude/range/emissive)
  stay withheld pending his ruling. Owes AC13's controlled shadows-off/on numbers + AC12 fps.
- **9.4 — at REVIEW, not yet reviewed.** 704 → 265 trees, foliage `(55,73,84)` → `(44,100,58)`.
  AC7 (the 9.1 x 9.4 luminance interaction) is UNMET and unmeasurable headlessly.
- **9.2 and 9.3 — corrected to `backlog` and should NOT be picked up**: their headless halves
  shipped in 8.2, their eye halves are art-blocked behind Epic 10. See
  [[stale-record-fabricates-scope]].
- **Next session:** either code-review 9.4, or run Epic 9's shared vehicle sitting (9.1's and 9.4's
  cards are both written and corrected) — the sitting unblocks more than the review does.

**2026-08-29 — STORY 9.4 CODE-REVIEWED -> in-progress.** Third clean four-layer full house; Edge Case Hunter reassigned off its empty R1 shell territory onto `crates/gui`. Code sound, every measured figure reproduced; almost every finding was OUTSIDE the diff. Two HIGH: the stale draw-set oracle (see [[documented-constant-was-a-measurement]]) and a record claiming a pin was "not at risk" when it had been re-pinned. Wolf looked at a frame mid-review and reported two defects, both then MEASURED: snow on the ground-level skirt (RULED "fix also snow cover" and fixed here — 1,029 bright ground cells -> 0) and **86 of 265 trees (32.5%) drawing no trunk at all**, exactly the height-4 trees, deferred as its own story. ACs 7 and 10 stay vehicle-bound. Review cost $35.36 / 457 turns INCLUDING the patch pass, against 8.2's $69.25.

**2026-08-29, same session — TWO MORE FIXES RULED IN FROM WOLF'S EYE, both measured first.** (1) Snow was on the ground-level skirt, not on top: `has_snow_laden_crown` tested sky exposure alone; ground-resting bright cells 1,029 -> 0. (2) THAT FIX REVEALED A GEOMETRY DEFECT RATHER THAN CAUSING ONE — the cubes had been painted bright and read as snow; correctly green they showed as "green boxes on ground level". `place_trees` stamped a foliage ring at `surface+1` that sealed the lower trunk (86 of 265 trees drew NO trunk, every height-4 tree) and broke on slopes (285 cubes hovering, 296 dropped). Ring removed: trees showing a trunk **179 -> 265 of 265**, foliage 6,329 -> 4,505. **"Underground" was measured and DISPROVEN** — every trunk sits flush on its own ground. LESSON: a wrong colour can mask a wrong geometry; fixing the paint is what exposed the shape. And the draw-set oracle moved TWICE in one story, see [[documented-constant-was-a-measurement]].

**2026-08-31 — STORY 10.2 DONE and MERGED (PR #56, merge `bbfe2bb`).** The BlenderMCP live-seat
spike. Verdict: the handoff WORKS and bit-exactly — a look found in a live Blender+MCP session on
gingerspice was re-emitted as a standalone headless generator (`voxel_pine.py`) that reproduces all
four GLBs byte-identically on the devpod, different machine, no MCP, no live session, no manual
step. Three review layers reproduced it independently. **Wolf's AC5(c) ruling: MCP JOINS the
standing workflow as the AUTHORING SEAT; the committed generator is the deliverable.**

Review: 4 layers, no coverage holes, 3 decisions + 16 patches + 6 deferred, gate green,
$31.18/399 turns. One real code defect (the generator exited 0 on every failure but its own range
check — see [[silent-sim-filter-trap]]'s family) and one headline record defect (see
[[superseded-artifact-identity-collision]]).

**WHAT BLOCKS NEXT, and it is not tracked on an issue:** the **metres-per-voxel project constant is
UNSET and blocks asset #2**. The tree was built at 0.2 m voxels off a 1.2 m dwarf; at that size the
DWARF is 6 voxels tall and cannot carry the beard, belt, tunic panel and lantern the reference sheet
draws. Coupled second decision: the client's cell is a unit cube (`Cuboid::default()`) while
`worldgen.rs` grows trees 4-6 cells against this asset's ~6.3. Two more items owed and unbuilt: the
handover runbook (Wolf: "not urgent now") and hardening `spike_pine_render.py`. **All three are in
`deferred-work.md` but have NO GitHub issue**, and the board's rule is that action-item state lives
on issues "and nowhere else" — so they are findable but not tracked. 10.3's epic text is about
`docs/tech-art-guidelines.md` contracts and will NOT pick up the runbook or the scale constant on
its own.

**2026-09-01 — 10.6 RE-MEASURED, back to `review`.** Wolf returned it to dev rather than
letting the review regenerate its own deliverable. All 23 patch items resolved, full gate green,
14/14 mutation rows killed. The number 10.3 copies is **k=4 = 928,884 triangles** (was 997,428
against a renderer serving 1,527,754). Offline and live now agree to the unit on meshed cells and
chunks; the residual triangle gap is chunk/rim partitioning and is bounded below. Still owed and
genuinely Wolf-side: gingerspice fps at k=4 AND k=8 — k=8 is a real candidate now, its old
exclusion having rested on a mis-sized guard. See [[guard-becomes-a-false-finding]] and
[[chase-the-small-delta]].

**Same day, after Wolf's vehicle screenshots:** the holes were NOT the connectors — `append_quad`
wound every chunk quad against its own normal and back-face culling deleted the whole terrain
surface, since Task 3. Fixed, 15/15 mutations killed, gate green. **Both fps readings are void**
(they measured a scene with no terrain), so AC7 is unmeasured at every k. See
[[counts-are-winding-blind]] and [[fps-that-does-not-move]].

**Dig stall fixed (2026-09-01):** one changed tile rebuilds 1 chunk not 121 — 55 ms against
~2,500 live at k=4, and the saving grows with k. Wolf's "blue tiles" are the 8,145 snow caps,
which cannot be dug because they are not tiles (ClientLocal, absent from the mirror). 19/19
mutations killed. Still owed: gingerspice fps at k=4 and k=8. See [[unit-tests-blind-to-cost]].

**10.6 later the same day:** snow is now PAINTED on the fine top faces (no slabs) — entities
14,527->6,826, 7.8x collapse, triangles unchanged. And the big one for 10.3: **the k=4 budget is a
RANGE, 80,120-928,884 tri** — 96.8% of the committed figure is the placeholder's white-noise detail
rule, which is also what Wolf saw as "terrain not forming meaningful form". NOT worldgen. AC7's
first valid reading: >140fps at k=4, no halts — a floor, not headroom (likely 144Hz vsync).

**10.6 AC7 SETTLED 2026-09-01:** Wolf measured `--subdiv 16` at a steady **60-90 fps,
fullscreen 4K, across zoom levels** — 13,873,064 tri clearing BOTH NFR6 bars at the finest
subdivision the client builds. Confirms k=4's >140 was a refresh cap. **The adopted k=4 is
REOPENED — Wolf's call** — and 16 voxels/cell is exactly the reference sheet's target. Key insight:
**cost is set by the detail's WAVELENGTH, not by k** (k=16 cell-coherent = 80,754 tri, a ninth of
k=4 with per-column noise). 10.6 sits at `review`, 10 commits UNPUSHED, no PR. Wolf will clear
context and run the code review next; 10.3 is the named next story. See
[[placeholder-sets-the-budget]].

**2026-09-02 — STORY 10.4 CODE-REVIEWED** (four layers, all live, no coverage holes; 3 decisions /
14 patches / 5 deferrals). Trees are now four authored GLB pines embedded in the binary. Full gate
GREEN, 10/10 mutations killed, **PR #63 open against main, NOT MERGED — parked 2026-09-02 evening, resume here**. Review cost $99.22 / 936
turns (42% in the layers), above 8.2's $69.25. AC12's closing half — real window, real fps — still
needs the vehicle; ~1.2M triangles for 265 pines vs ~479k terrain is unanswered. See
[[delta-needs-a-noise-floor]] and [[new-failure-hides-behind-adjacent-one]] for what it found.

**Where 10.4 stands at park (2026-09-02).** 25 commits on `10-4-the-trees-look-right-the-pilot`,
full gate GREEN, 12/12 mutations killed, working tree clean, everything pushed. Wolf reviewed the
corrected captures by eye: trees read better than the cube-tree artifact he had been shown, "still
work to do for wow". Per-tree yaw was added on his call and measured (116,963 px at d>=4 vs 46,050
noise). **THE REMAINING GATE IS WOLF'S EYE ON THE VEHICLE** — AC12's closing half, plus an fps
reading for ~1.2M triangles that lavapipe cannot give.

**Next look work is DEFERRED and scoped, not started:** the tree ROOT reads artificial where trunk
meets terrain. Two halves in `deferred-work.md` — asset (root flare in `voxel_pine.py`) and client
(mesh sits flat on the cell floor, no skirt). Wolf named the ground foliage ring as the candidate;
the entry carries 9.4's three measured reasons for removing it and why a PRESENTATION-side skirt
may dodge all three. Do not revert 9.4's commit.

**2026-09-03 STORY 10.7 (the sun) — Tasks 1-7 DONE, Status `review`, NOT pushed.** Wolf ruled
`+17.66°` from a side-by-side of four bench frames; the sun is decoupled from the aurora into its
own azimuth/elevation constants. AC4 measured 131.8x its own noise floor. Vehicle sitting done:
shadows right at the shipped `--subdiv 1`, **fps unchanged**, campfire and lanterns still reading
as the valley's own light. **Review was NOT run in this session** — the fresh-context precondition
forbids it, since this session did the dev; it wants `/clear` then `bmad-code-review`.
**2 stories left after 10.7: 10.5 then 8.3.**

**2026-09-04 STORY 10.7 — ALL TWELVE ACs MET, Status `review`, STILL NOT PUSHED.** The four-layer
review passed on 2026-09-03 and the **pre-merge vehicle sitting falsified AC12 anyway**: Wolf saw
black quads still at the trunk bases with the full gate green. Cause was a **DRAW-SET** defect, not
face emission — `is_visible_at_slice` reads a `Solid(TreeTrunk)` cell as ordinary solid cover, so the
ground under a trunk never reached the mesher. Fixed by `is_visible_to_terrain_at_slice`. Holes at
subdiv 1/2/4 now 1,650/15 · 2,042/16 · 1,825/19, rim-band only. Wolf then signed **UX-DR22's closing
half for ACs 10-12** ("holes are fixed now even on terrain", "16 is ok", "light toggles are working
properly"). Issue **#65 was filed and then CLOSED as not-a-defect** — the residual is sky past the
world's outer edge, framed by pines standing beyond the last terrain cell, and the whole-world face
oracle reports zero missing faces. Full gate GREEN 10/10, mutations 14/14 killed.
**OPEN BEFORE MERGE: the draw-set fix and the guard's oracle replacement have had NO fresh-context
review** — roughly 30 lines of production code plus a rewritten guard, landed after the review.
**MERGED 2026-09-04 as PR #66** (`4256b15` -> merge `87f3bdc`), 43 commits, +3,757/-202, and it
merged with that review gap STATED rather than closed — worth remembering that the review which did
run passed, and the vehicle sitting falsified AC12 anyway. **M2 is now 11/11 on Epic 10's list
except 10.5; 2 stories left: 10.5 then 8.3 closes the milestone.**

**2026-09-04 — forge-process 1.3.2 → 1.5.0, PR #68 open; 10.5 Part A written but NOT started.**
Two threads ran on 2026-09-04 and **the devpods crashed between them**, so the branches sat local:
`10-5-story-creation` (2 commits, story 10.5 Part A written, verification recipe executed, its own
AC2 falsified and rewritten as a WINDOWED bar) and `forge-process-1-5-0`. The upgrade is the LIVE
thread — the story is written, not begun. **10.5 blocks on its own Task 1: Wolf's ruling on the
hot-reload venue** (`embedded_watcher` resolves the source by COMPILE-TIME path, so the Windows
vehicle that can show a dwarf has no source tree and the devpod with the source tree cannot open a
window). Three options costed in the story; recommendation is dev-only `--assets` disk loading
deferred to Part B. The upgrade itself: full gate GREEN, `check` in sync at 1.5.0, 78 unit tests OK,
runbook step 6 closed with forge commit `6e98631`. **Two decisions recorded as owed, not taken:**
the 1.4.0 phase-metric net is INERT here (nothing declares a phase; the only declaration upstream
lives in `bmad-review-patch`, a skill frostvein lacks, and `.claude/` is gitignored so the hook
cannot propagate between the two devpods), and whether frostvein wants that skill at all.
