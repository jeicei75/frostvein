# T2 (`--mark`) — the record

**Status: CLOSED 2026-08-24.** Shipped in the forge as forge-process 1.2.0 (`dbc2fe6`, PR #14)
and pulled into frostvein the same day; `forge-process.sh check` reports in sync at 1.2.0.

This file is a RECORD, not a runbook — it answers "why is it this way?", and the handover
prompt below is kept verbatim because it is the evidence, not a summary of it. The
`sprint-status.yaml` entry for T2 points here rather than carrying this text, per the
~150-char cap on a file every workflow run loads.

## Outcome

**The original success criterion was already met before the fix, and is not what closed the
item.** `delta_since_cursor()` plus `.session-cursors.json` had billed per-transcript deltas
since Epic 3. The real defect sat one level down: **the cursor only advanced when a ledger row
was written**, so any window not bookended by two rows was swept into the next row taken — the
leading window before a session's first record (3.2's failure), and the gap between one phase
ending and the next beginning (7-1's and 7-2's). `--mark` advances the cursor without writing a
row, and prints the window it discards *before* moving it, because a silent mark erases that
window from every ledger at once.

**Why it sat open three epics: it was never a frostvein task.** `session_tokens.py` is a
forge-owned `FILE` entry that must stay byte-identical, so the owner recorded against it could
not land it. It needed to be a forge change request and was never filed as one; forge-process
1.1.0 merged frostvein's other Epic 3 findings and T2 did not travel with them. The delay cost
5 caveated rows across 4 epics, and closing T4 made it **worse** — adding `live-gate` as a
phase created more boundaries with nothing to mark them.

**A second staleness gap, found at close.** The fix was in the forge but invisible here:
frostvein's copy was byte-identical to forge commit `6e000bb` (1.1.0), the old 1135-line
version with no `--mark` at all. This is the same failure the forge-process upgrade runbook
opens by citing — the `PRICES` fix that never reached frostvein and left Epic 1's ledger
reading ~3x high. A fix landing upstream is not a fix delivered.

## All three traps held — two guarded harder than asked

1. **Cursor shape** — both the recording path and `--mark` write through one shared
   `cursor_from()` helper, so drift is structurally impossible rather than merely tested. A row
   also *clears* the `marked_at` it supersedes.
2. **`--no-nested`** — refused outright, and the *baseline* direction this prompt only
   half-named is refused too: a cursor previously written with `--no-nested` is caught by a new
   `primary_only` flag, because marking from it would discard a window computed against the
   wrong baseline.
3. **v1 rebase** — refuses to mark a v1 cursor, but guarded on whether fan-out actually exists
   rather than on the schema alone, with the reasoning recorded: 72% of live cursors are v1, so
   a blanket refusal would have forced exactly the row `--mark` exists to avoid.

## Not fixed, deliberately

**7-2's `live-gate` row stays a mixed window** — $151.81 covering review-patch *and* the
vehicle session — and its caveat stands. Splitting it retroactively needs a timestamp bound on
the summarizer that nothing has today. `--mark` is forward-looking; Epic 8 is the first epic to
record under it.

## Verified at close

- 61 tests green in frostvein's own copy of the suite (not just the forge's).
- `--mark` present in `--help` here.
- `forge-process.sh check projects/frostvein` → in sync at 1.2.0, all FILE entries `ok`.
- The METRIC RULE wiring hand-merged into frostvein's three adapted workflow TEMPLATEs and
  `ack`ed, so the flag is *invoked*, not merely available — the T4 lesson inverted. The rule
  carries its own guardrail: a mark used where a ROW was owed converts a visible
  over-attribution into an invisible under-attribution, which is worse.

---

# Appendix — the handover prompt as sent to the forge, verbatim

You are working in the **Nidavellir forge** (`/workspace`). Read `/workspace/CLAUDE.md`
first — command hygiene and YAGNI are policy here, not advice.

## Why this is coming to you and not being done locally

`_bmad/scripts/session_tokens.py` is **forge-owned and the source of truth**. frostvein
was explicitly told not to edit it locally ("it is a FILE entry and must stay
byte-identical"), so this request has to land here or not at all.

frostvein has carried this as open action item **T2** since Epic 3. It was filed against
frostvein's own owner, who structurally cannot land it — which is why it has sat open
across three epics rather than being deprioritised. `forge-process 1.1.0` merged
frostvein's Epic 3 findings but T2 did not travel with them: `rg -- '--mark' /workspace/_bmad`
returns nothing today.

## The gap — narrower than T2's original wording

T2 was written as "a phase recorded mid-session bills only its own delta window." **That
criterion is already met.** `delta_since_cursor()` (`_bmad/scripts/session_tokens.py:574`)
plus `.session-cursors.json` already bill per-transcript deltas.

The real defect: **the cursor only advances when a ledger row is written**
(`session_tokens.py:1085` returns early unless both `--story` and `--phase` are passed;
the cursor write is at `:1118`). Any window not bookended by two rows is swept into the
next row that happens to be taken. Two shapes of loss:

1. the leading window before a session's first record;
2. the gap between one phase ending and the next beginning.

## Evidence this is real and getting worse

Readable on disk (frostvein is git-ignored at forge root but present — use `fd`/`rg --files`,
and read these known paths directly):

- `/workspace/projects/frostvein/_bmad-output/implementation-artifacts/metrics/7-2-read-the-working-zoom.md`
  (lines ~20-30) — the `live-gate` row bills a 14-patch verification pass **and** a
  separate vehicle session as one $151.81 / 1870-minute delta on transcript `99c8862c`.
  The ledger states: *"There is no way to split them: that is precisely what open action
  item T2 (`--mark` for phase boundaries) exists to fix."* Written 2026-08-23.
- `/workspace/projects/frostvein/_bmad-output/implementation-artifacts/metrics/7-1-slice-into-the-mountain.md`
  (lines ~15-22) — story-creation over-attributes: a prior review-patch round, two live
  vehicle sessions, a dig-site re-pick and a snow-cap fix all billed to `create`, because
  no row was taken when the previous phase ended.

That is 5 caveated rows across 4 epics. Note the direction of travel: **closing frostvein's
T4 made this worse.** Adding `live-gate` as a phase means more phases per session, so more
boundaries with nothing to mark them.

## The ask

Add a `--mark` mode: **advance a transcript's cursor to "now" without writing a ledger row**,
so a phase boundary can be set at the moment a phase ends.

## Three traps that will silently corrupt the ledger if missed

1. **Cursor shape.** `--mark` must write the exact v2 cursor payload the recording path
   writes at `:1118` — all `_CURSOR_BUCKETS`, `schema`, `by_model`, `last_ts`, `quota_last`.
   A partial cursor makes every *later* row on that transcript wrong, which is worse than
   the problem being fixed because it looks precise.
2. **`--no-nested` interaction.** The cursor stores the full-session (fan-out inclusive)
   cumulative. If `--mark` ever stamps a primary-only cumulative, later deltas inflate by
   the whole fan-out. Decide deliberately whether `--mark` accepts `--no-nested` at all.
3. **The v1→v2 rebase path** (`:1093-1103`) exists for pre-fix cursors. Decide whether
   `--mark` participates in the rebase or refuses on a v1 cursor.

## Decide, don't assume — surface these rather than picking silently

- **Does a mark leave a trace?** Silently advancing means the discarded window vanishes
  from all accounting. Printing what it discarded (turns / est_usd) is probably the
  minimum. Whether an unattributed window should be *recoverable* is a real question.
- **Codex rollouts** (`--tool codex`) — in scope for marking, or Claude transcripts only?
- **Retroactive splitting is a SEPARATE question — do not fold it in.** Splitting 7-2's
  existing mixed window would need a timestamp bound on the summarizer. Rows do carry
  timestamps (`:249`, `:296`, `first_ts`/`last_ts` at `:372`), but nothing filters by
  them today. Treat `--mark` as forward-looking; 7-2's caveat stands either way. If you
  think the timestamp bound is cheap, say so and let Wolf decide — don't build it uninvited.

## Verification

- Add tests to the existing `DeltaCursorTests` class
  (`_bmad/scripts/tests/test_session_tokens.py:86`): a mark followed by a recorded row
  bills only the post-mark window; a mark writes a schema-2-complete cursor; the tokens
  before the mark appear in no row.
- Run the full suite — `PreservationTests`, `RollupTests` and `LedgerWidthGuardTests` are
  the ones most likely to catch collateral damage.

## Success criterion

A window between two phases can be closed without writing a ledger row, and the next row
on that transcript bills only its own work. (Replaces T2's original criterion, which the
delta cursor already satisfies.)

## Out of scope

Ledger format changes, rollup changes, re-pricing, backfilling old rows, and any
frostvein-side edit. If this lands, frostvein needs only to be told the flag exists so it
can close T2 and start using it in Epic 8.

## Why the timing matters

frostvein's Epic 8 is vehicle-heavy — action item M2-15 folds GUI-triggered vehicle runs
into story 8.2. Unfixed, every 8.x `live-gate` row inherits the same mixed-window defect
that made 7-2's headline figure unquotable, in the epic where those figures matter most.
