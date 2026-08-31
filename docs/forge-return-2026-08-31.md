# Forge return — 2026-08-31

Frostvein is at **forge-process 1.3.2**, up from 1.2.0, and `check` reports **in sync**. Both FILE
entries pulled clean; the three changed TEMPLATEs hand-merged and `ack`ed; the one new TEMPLATE
adopted on Wolf's ruling, in a scoped form, with the action items migrated to GitHub issues (§4).
Everything below is measured here, in frostvein's own copies, not read off the forge.

## 1. The `gpt-5.6-terra` price row — the same trap as `PRICES`, caught earlier this time

The forge added a `gpt-5.6-terra` row (`$2` in / `$12` out) because Terra has been the actual Codex
dev model since ~2026-08-10 and had no entry — so it fell through to `gpt-5` (`$1.25` / `$10`) and
every Codex row **understates**. Frostvein is squarely affected: **25 rows across 12 story ledgers**
name `gpt-5.6-terra`, from `5-2-one-mirror-two-clients` through `10-1-the-headless-bench`.

Re-priced at the corrected rate (measured, not estimated):

| | recorded | corrected | delta |
|---|---|---|---|
| 25 `gpt-5.6-terra` rows | $56.85 | **$87.49** | **+$30.65 (1.54x)** |

**Not retro-corrected**, following the forge's own convention for the `gpt-5.5` case: the rows carry
`rates 2026-08-01` and therefore say for themselves which table produced them. Read any pre-2026-08-31
Terra figure as a gpt-5-rate equivalent. **Wolf's call if that is not good enough** — a re-price is a
mechanical pass over the 25 rows, and the epic rollups that aggregate them would need the same.

Two things this does NOT touch: every `tool=codex` figure is a comparability benchmark, not money
(Codex is subscription — `quota_pp` is the axis that binds), and the Claude-side rows are unaffected.

## 2. The ledger-insert fix (1.3.1) — frostvein was already bitten, three times

`append_ledger` used to append at EOF. A ledger legitimately grows prose *below* its table, so a
later row lands under that prose. **Seven** frostvein ledgers carry prose below the table, and three
already have rows stranded under it:

- `7-1-slice-into-the-mountain.md` — the `dev` and `review` rows sit below the ATTRIBUTION NOTE.
- `6-2-lanterns-in-the-dark.md` — the `review` row sits between two blockquotes.
- `2-1-the-world-runs-on-its-own-clock.md` — the `dev` and `review` rows sit below the correction note.

**Scope, checked rather than assumed: this is a RENDERING defect here, not data loss.**
`parse_ledger_rows` reads any line starting with `|` regardless of position, and it still returns
every phase for all three files (7-1: create/dev/review; 6-2: create/dev/dev/review; 2-1:
create/dev/review). So the rollups have always been right; what is wrong is that markdown renders
those rows as literal text instead of table rows, and in 6-2 the note's own phrase *"the `review`
row above"* now points the wrong way.

**Not repaired in this pass** — moving rows inside three historical evidence files is a change to the
record, not a sync, and it is Wolf's to authorise. The repair is cheap and verifiable if wanted:
move each stranded row back under the last table line, with `parse_ledger_rows` output identical
before and after as the check.

## 3. The three hand-merged rules — adapted, with frostvein's own evidence substituted

The forge's evidence for all three is Asgard's (vaults, courts, preflight). The rules are universal;
the instances are not, so each was rewritten against frostvein's own scars.

- **`bmad-code-review.toml` ← BOTH-SIDES CLOSURE RULE** (forge ep-15 A1 / nidavellir #20). A patch is
  not closed until verified from the direction it was *not* written for, against a pre-existing-state
  fixture; state in one line which side the fix was written for and which side you tested. Frostvein
  instances used instead of the forge's six: "KILLED" names the test and not the assertion you just
  added; a fix to an inert mechanism relocates the defect into what feeds it; client-side green is
  one side only, because the sim accepts-and-discards silently. Cost argument restated in frostvein's
  numbers ($69.25 / 615 turns for 8.2's review + patch pass).
- **`bmad-create-story.toml` ← EVERY AC THAT NAMES AN OBSERVABLE MUST CITE ITS EMITTER** (forge ep-13
  A1 / nidavellir #32). Cite the emitter as `file:line` or make emitting it a task, and ask the second
  question too — would the output DIFFER between the fixed and unfixed code? Frostvein evidence:
  10.1's literal-scraping guard stayed green while the bench camera was rolled 110°, and three more of
  the same shape were found later, one of them pinned by the guard and read by nothing. Added with an
  explicit SCOPE clause saying what it is *narrower* than, so it does not read as a restatement of the
  OBSERVABILITY INSTRUMENT RULE or of CAN IT HAPPEN.
- **`bmad-create-story.toml` ← the deliberate red**, merged as rule (4) *inside* the existing
  "THE VERIFICATION RECIPE MUST BE EXECUTED BEFORE THE STORY IS SAVED" fact rather than as a fourth
  overlapping rule — that fact is already frostvein's form of the forge's preflight rule. It now
  requires the recipe to name the condition to break, the exact expected failure output and the
  restore step. Frostvein evidence: three instruments in one session reported success while capturing
  nothing.
- **`bmad-dev-story.toml` ← MUTATION-VERIFY EVERY NEW TEST** (the dev half of the same rule). This
  file had exactly one fact before today and no mutation rule at all, even though the mutation table
  is frostvein house format and `audit-mutations.py` sits in the gate — so the standing instruction
  never told the dev agent to run the sabotage itself, which is precisely the self-referential-test
  antipattern hit in 1.1, 1.2 and 1.3. Restated in this project's machinery (`mutations/` tables, the
  `assert s.count(old) == N` guard, `mutate.sh`, `audit-mutations.py`) with four traps: KILLED names
  the test; a focused run cannot establish exclusivity; a test can pass vacuously; APPLY-FAILED is not
  noise. Plus the two tooling facts — `mutate.sh` is not concurrency-safe, and commit before mutating.

All three parse (`tomllib`), and the gate's own metrics-tests row covers the FILE half.

## 4. Action items moved to GitHub issues — adopted, scoped (Wolf, 2026-08-31)

1.3.0's `bmad-sprint-status.toml` moves action items out of `sprint-status.yaml` and into GitHub
issues. Frostvein adopts it, in the scoped form: **state on the issue, reasoning in the repo.**

**The efficiency question was measured, not guessed, and it points the other way.** The
`action_items:` block was **27,193 tokens — 48% of a 56,528-token file**; `gh issue list --json`
over the live items is **~330 tokens**. `sprint-status.yaml` is now 226k → 119k chars and still
parses with all 52 stories. The block was also where items went to rot: every open item dated from
Epic 5 and was still open at Epic 10, visible on the board the whole time — which is the forge's
own argument for the routing labels, and it lands here exactly.

**`createIssue` was proven before anything was migrated** (the manifest's own instruction; ep-15-a2,
verify the instrument). Throwaway issue #42 created, read back, deleted. The fine-grained PAT does
carry the Issues permission. Labels created: `action-item`, `route:skill-rule`, `route:story`,
`route:forge-process`, `route:undecided`.

**Twelve open items became nine issues, because they were re-verified against the tree first.**
This is the migration's real finding — filing them verbatim would have fabricated roughly three
items of scope:

| verdict | items | evidence |
|---|---|---|
| already DONE, never struck | M2-6, M2-8, M2-9 | the review-teardown reaper is mandated in `bmad-code-review.toml` (added 2026-08-28); `audit-mutations.py` runs the anchor check in the gate; `mutate.sh` captures `rc` before the pipe, and commit-before-mutating became a standing fact today |
| HALF done, framing stale | M2-7, M2-15 | `build.rs` stamps `GUI_BUILD_SHA` but nothing automates the `gui.exe` copy; `--at-tick` landed in `capture.rs` but the scenario session mode did not, and "fold into Epic 8" aims at a closed epic |
| open as written | M2-2, M2-3, M2-5, M2-10, M2-11, M2-12, M2-18 | filed as #43–#51 |

The two half-done items were filed with **only their remaining half**, and M2-15 re-aimed at the gfx
pass rather than at Epic 8. M2-12 (decide the self-gate's future — the one item Wolf co-owns) is
`route:undecided` on purpose: an item with no vehicle is a risk, not a backlog entry.

`action-items.md` carries all 60 items verbatim — every `action` and `note` string was checked
present after generation — plus the twelve verification notes above. `check` now reports **in sync**.

**One trap found on the way, worth keeping:** `.gitignore` ignores `_bmad/custom/*` with a per-file
allowlist, so the new `bmad-sprint-status.toml` was invisible to git — the adoption would have
worked locally and never travelled. `.gitignore` now allowlists it. Any future TEMPLATE added to
the manifest needs the same line, or `check` on a fresh clone reports MISSING for a file that
exists.

**Two rules were adapted rather than copied**, both where the forge's text is true for Asgard and
false here: frostvein's issues and PRs live in the *same* repo, so `Closes #N` does work (the
forge's cross-repo caveat is inverted for us), and the "re-verify before filing" rule is
frostvein's own, written from today's three-of-twelve.

## 5. Owed back to the forge

Both rules gained frostvein evidence the forge's copies do not carry, and both are portable:
`mutate.sh` is not concurrency-safe (five false failures in one session), and "KILLED names the TEST,
not the assertion" — an earlier assert absorbs the mutation, so a strengthened test still reports
KILLED while the new assertion has never run. Neither is Rust-specific.

## Verification — run here, not claimed

```
bash /workspace/scripts/forge-process.sh check /workspace/projects/frostvein
  target : /workspace/projects/frostvein  @ 1.3.2
  FILE     _bmad/scripts/session_tokens.py                      ok
  FILE     _bmad/scripts/tests/test_session_tokens.py           ok
  TEMPLATE _bmad/custom/bmad-create-story.toml                  adapted
  TEMPLATE _bmad/custom/bmad-dev-story.toml                     adapted
  TEMPLATE _bmad/custom/bmad-code-review.toml                   adapted
  TEMPLATE _bmad/custom/bmad-sprint-status.toml                 adapted
  TEMPLATE scripts/codex-handoff.sh                             adapted
  in sync.            (exit 0)

gh issue list --label action-item --state open   → 9 issues, #43-#51, every one route-labelled
sprint-status.yaml                               → 226,114 -> 118,714 chars, parses, 52 stories

python3 -m unittest discover -s _bmad/scripts/tests   → Ran 62 tests, OK   (frostvein's copy)
PRICES_VERSION 2026-08-31, gpt-5.6-terra {'input': 2.0, 'cache_write': 2.5, 'cache_read': 0.2, 'output': 12.0}
```
