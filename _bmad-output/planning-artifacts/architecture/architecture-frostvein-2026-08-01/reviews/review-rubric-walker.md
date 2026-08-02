# Review — rubric walker (good-spine checklist)

Run inline (sequential fallback — subagent quota exhausted mid-gate),
cross-checked against the two independent reconciliation subagents' reports
(PRD/addendum and tech-prefs/brief), which did run with fresh context.

## Checklist verdicts

- **Fixes the real divergence points, misses none** — mostly; the gate found
  three real holes (load-desync, vocabulary double-ownership, pause-ack),
  closed under the adversary review. Post-fix: pass.
- **Every AD enforceable and preventing its divergence** — pass. AD-7 and
  AD-8 phrase rules mechanically (chained schedule, single mutation path);
  AD-11 gained the missing I/O-owner clause.
- **Nothing under Deferred lets two units diverge** — two findings:
  the raycast view is *in-milestone by default* but its Deferred entry
  didn't carry the addendum's decided sub-voxel/no-sprites constraints
  (fixed); the LLM entry said "logged inputs" while AD-10 defers the log
  (clause added).
- **Named tech verified-current** — pass (see review-reality-check.md; all
  eight crates checked against crates.io on 2026-08-01; rand_chacha `serde`
  feature confirmed and noted).
- **Covers the driving inputs' capabilities** — F1–F9 all mapped; the PRD's
  explicit confirm-or-override obligation on its six [ASSUMPTION]s was
  performed in coaching (all confirmed by Wolf, in the memlog) but the spine
  didn't record it — one line added.
- **Ratifies rather than contradicts existing reality** — the two intended
  amendments (fourth `protocol` crate vs "three crates"; no phase-one
  command log vs ADR2's "command log" wording) must actually land in
  docs/technical-preferences.md and CLAUDE.md ground rule 2, or agents that
  read those first (they claim to win) will build the old layout. Scheduled
  as part of finalize; AD-10 now carries the same amendment annotation as
  AD-6.
- **Every owned dimension decided/deferred/open** — operational envelope is
  thin but *decided*: phase one is explicitly dev-only (WSL2 devpod,
  `cargo run`, localhost TCP, local save file) and NFR1 forbids building
  beyond it; correct at hobby stakes. Tech-prefs rules with architectural
  force that hadn't landed (single-impl abstraction ban, third-use rule for
  config/plugin/event systems, closed dependency list, unsafe ban,
  once-per-frame flush) are added as conventions.

## Residual findings

- **[medium→user]** Pause-ack semantics (adversary #3) — genuine fork,
  touches adopted ADR2; goes to Wolf.
- **[low]** NFR2's ~100 ms full-z-level frame budget relies on untested
  assumptions about JSON decode + framebuffer flush cost; no spine change
  (measurement infrastructure is out of scope), noted for the first TUI
  story to eyeball.

Verdict: sound after fixes; one decision escalated to the user, everything
else closed in place.
