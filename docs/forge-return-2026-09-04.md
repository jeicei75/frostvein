# Forge return — 2026-09-04

Frostvein is at **forge-process 1.5.0**, up from 1.3.2, and `check` reports **in sync**. Two FILE
entries pulled clean, three new FILE scripts installed, two adapted TEMPLATEs hand-merged and
`ack`ed, one new TEMPLATE taken but **inert** (§4). Everything below is measured here, in
frostvein's own copies and its own rollouts, not read off the forge.

## 1. The quota-window fix — frostvein WAS bitten, and the numbers are wrong by ~6.5x

The forge fixed `quota_pp` (`a4b717d`): it read `rate_limits.primary.used_percent` and labelled the
result "the 7-day window". On 2026-08-06 that was correct — a real rollout carried
`primary.window_minutes == 10080` with `secondary: null`. **By 2026-08-31 Codex had added a 5-HOUR
window as `primary` and moved the weekly to `secondary`**, so unchanged code began recording 5h
points under a weekly heading. The fix selects the window by `window_minutes` nearest seven days,
never by key name.

**Frostvein's two affected rows are both in `10-6-how-fine-can-we-go`, recorded 2026-08-31.** Read
straight out of the rollouts they name:

| rollout | `primary` (5 h) | `secondary` (7 d) | recorded as | the weekly delta the fixed code returns |
|---|---:|---:|---:|---:|
| `…T14-30-51…` | `window_minutes=300`, 52.0% | `window_minutes=10080`, 23.0% | **52pp** | **8pp** |
| `…T15-17-02…` | `window_minutes=300`, 86.0% | `window_minutes=10080`, 28.0% | **33pp** | **5pp** |

**Both rows overstate weekly quota consumption by roughly 6.5x.** The 5-hour window self-clears, so
the alarming figures were largely transient pressure, not weekly headroom spent. This matters here
specifically: frostvein already carries "one story burned the whole weekly quota 0→100%" from 8.2
and a standing rule to check headroom before delegating — a 6.5x overstatement pushes exactly that
call in the wrong direction.

**Not retro-corrected**, following the convention established for `PRICES` and `gpt-5.6-terra`: the
rows say for themselves when they were recorded. Read any `quota_pp` recorded **on or after
2026-08-31 and before today** as a 5-hour figure. Only those two rows qualify — every earlier
`quota_pp` predates the Codex change and its `primary` genuinely was the weekly.

**The header prose in every metrics file is now stale too**: it says `quota_pp` is "read from
`rate_limits.primary.used_percent`". That sentence is what the fix deletes. Not rewritten across 12
ledgers here — flagged, because a rewrite touches files whose figures must not move.

## 2. The phase-metric safety net (1.4.0) — taken, and it is INERT until something declares a phase

`phase_state.py` + `record_phase_on_exit.py` + `test_phase_metric_hook.py` close a real hole:
`session_tokens.py` writes its row from a skill's `on_complete`, so a session that dies first —
context exhausted, interrupted, harness-killed — drops its whole spend from the ledger silently.

**Frostvein has paid for exactly this.** Its metrics record carries "a whole session ABSENT from the
cursors — no row, no mark, so nothing looks anomalous; 10.1 dev read $2.41 when it was $24.77", and
"4 of 6 delegated runs harness-killed". This is the fix for that class.

**But taking the scripts does not deliver the fix**, and this is the part to be careful about:

- Nothing in frostvein declares a phase. The only `phase_state.py set` call in the forge lives in
  `.claude/skills/bmad-review-patch/steps/step-01-load-findings.md` — **a skill frostvein does not
  have**. Installed as-is, the SessionEnd hook fires, finds no declaration, and records nothing.
- **The hook cannot be committed.** `.claude/` is gitignored here, so wiring
  `record_phase_on_exit.py` into `.claude/settings.json` is a per-machine change that will not
  propagate — and work moves between two devpods, so the metric would exist on one and not the other.

**Owed, and it is Wolf's call:** add an `activation_steps_prepend` phase declaration to frostvein's
own `bmad-create-story` / `bmad-dev-story` / `bmad-code-review` TEMPLATEs, and decide whether
`.claude/settings.json` becomes tracked. Until then this is scaffolding, and it is recorded as
scaffolding rather than reported as a delivered fix.

## 3. The live-bug rule (1.5.0) — hand-merged into both adapted TEMPLATEs

*"A bug found by RUNNING the system gets an issue when found"* was Wolf's ruling of 2026-09-03 and
was written down nowhere; it lived in one session's memory. Merged into `bmad-dev-story.toml` and
`bmad-code-review.toml`, then `ack`ed.

**Adapted, not copied.** The forge's text labels a ruling-needed issue `needs:wolf`; frostvein has no
such label, so the merged rule maps it to **`route:undecided`**, which this board already surfaces as
a RISK. One `gh label create` changes that if Wolf prefers the distinct name. The forge's worked
example (its own gh #70/#74) is replaced by frostvein's: **the subdiv-2 holes Wolf saw from the seat
on 2026-09-04 became issue #65 only because he said to file one** — the story's four-layer review had
already passed, and the finding would otherwise have gone to `deferred-work.md` and stopped there.

`deferred-work.md` holds **143** items here. The rule's claim that it is a record and never a
substitute is the reason the rule exists.

## 4. `bmad-review-patch.toml` — taken, inert, and a decision is owed

`install` copied it because it was MISSING. **Frostvein does not have the `bmad-review-patch` skill**,
so no workflow ever loads this customisation. It is tracked so `check` stays clean — a permanently
red check is the failure mode this whole mechanism exists to prevent — but it does nothing today.

The real question is whether frostvein wants the *skill*. It ran a review-patch pass by hand on 10.7
(seven findings, one pass, full gate green), so the capability is being exercised without it.
**Wolf's call. Recorded rather than quietly adopted.**

## 5. What was verified here

- `python3 -m unittest discover -s _bmad/scripts/tests` — **78 tests OK**, including the 11 new
  phase-hook tests.
- `scripts/forge-process.sh check /workspace/projects/frostvein` — **in sync** at 1.5.0.
- `scripts/gate.sh` — **GATE GREEN**, all ten lines. The upgrade runbook's step 5 asks for the
  project's own gate as well as `check`, because installing a new `session_tokens.py` must not break
  the metric recording the gate exercises; re-run after the devpod crash, on this branch, at 1.5.0.
- `.gitignore` gained four negations. Without them all four new files were installed to disk and
  invisible to git, so a fresh clone would have reported MISSING and the sync would have been real
  on one machine only.

## 6. Owed back to the forge

Nothing found broken upstream. One observation worth returning: **the 1.4.0 phase declaration exists
only in `bmad-review-patch`**, so any project without that skill takes the hook and the scripts and
gets an inert safety net. The forge may want the declaration in `bmad-dev-story` and
`bmad-code-review` too, which are the phases every consumer runs.

The forge's own upgrade runbook asks for a History line recording which projects were updated. **Not
written — that is a commit in the forge repo, and it is left for Wolf** rather than taken
unilaterally from inside frostvein.
