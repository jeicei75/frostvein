# Reconciliation review — M2 spine vs parent spine + sprint change proposal

**Reviewed:** 2026-08-09
**Spine under review:** `architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md` (draft)
**Input 1 (binding):** `architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md` (parent, final)
**Input 2:** `sprint-change-proposal-2026-08-08.md` (§2 architecture impact, §5 step 4, success criteria)
**Also consulted:** `prds/prd-frostvein-2026-08-09/prd.md` (NFR5–NFR8, TUI-role line), `scripts/gate.sh` (current probe shape)

---

## A. Inheritance integrity

### A1. AD-13's tui surgery vs the parent's tui description — **CONTRADICTION, unsurfaced** ⚠

Parent AD-6's rule text is literal and load-bearing:

> "`simd` and `tui` both depend on it; **`tui` depends on nothing else in the
> workspace**."

AD-13 gives `tui` a second workspace dependency (`client-core`) and retires its
in-crate client state. That is almost certainly the *right* call — AD-13's
prevents-clause (two drifting implementations of AD-8's client-side semantics)
is exactly the failure AD-6 exists to prevent, one level up — but the M2 spine
simultaneously asserts:

> "AD-1 through AD-12 … **bind this milestone unchanged**. … No AD below may
> weaken one of these."

Both sentences cannot be true. AD-6's "nothing else" clause is being amended,
and the spine's own parent has a precedent for how to do this honestly: AD-6
and AD-10 both carry explicit "Amends technical-preferences.md … that doc is
updated to match" records, and AD-10 carries a dated amendment block for the
`remove_stockpile` case ("only the enumeration was stale"). The M2 spine's
**Parent updates owed** section is the designed home for exactly this and lists
only the two Deferred entries. AD-6's sentence belongs on that list, with the
same framing AD-10 used: the *rule* (wire shapes only in `protocol`, no client
links the sim) survives intact; the *enumeration* of tui's dependency set is
what went stale.

**Verdict: fix required — one line in "Parent updates owed" + a parenthetical
in AD-13 ("amends AD-6's 'nothing else' enumeration; the no-sim-edge rule is
unchanged and now gate-probed for all three client-side crates").**

### A2. The dependency graph vs the parent's "no edge may be added" rule — **not explicit enough** ⚠

The parent frames its 3-edge graph as an invariant: "Dependency direction is a
rule — **no edge may be added to this graph**." The M2 spine presents a 7-edge
graph under the same "no edge may be added" formula. The extension is
legitimate (the proposal's §2 sanctions a new protocol consumer; the parent's
rule was always about direction and sim-isolation, not a frozen crate count),
but the spine never *says* it is superseding the parent's graph. A reader
holding both documents finds two graphs, both claiming closure, with no bridge
sentence. Because the graph rule is neither an AD, a Consistency Convention,
nor the stack, the M2 spine's inheritance preamble ("AD-1 through AD-12, all
Consistency Conventions, and the closed stack … bind unchanged") does not even
clearly *cover* it — it falls through the enumeration.

**Verdict: fix required — one sentence above the M2 graph: this graph
supersedes the parent's for M2; every parent edge is preserved, the additions
are the two client crates and `tui`'s adoption of `client-core` (AD-13); the
direction rule (nothing client-side may reach `sim-core`) is unchanged and
newly probed (NFR8).**

### A3. AD-15's previous-tick retention vs AD-8's delta semantics — **compatible, one silent edge**

No contradiction in the core: AD-8 governs how the *current* authoritative set
is computed (full-resend = exact replacement, absence is deletion), and AD-15's
previous-tick copy is a read-only history of wire-delivered truth, snapshotted
*before* the delta is applied. Retention does not weaken
absence-is-deletion — the deleted entity is absent from the current set; the
mirror's current state is exactly the list sent. "The mirror holds only states
the wire delivered" is a faithful extension of AD-4/NFR5 into time, and the
never-extrapolate clause keeps AD-4 intact.

**One silent edge:** AD-11 sanctions `snapshot` as a wholesale reset, and its
own text warns "between snapshots ticks never decrease" — i.e. *across* a
snapshot they may. AD-15 does not say what happens to the previous-tick buffer
on a snapshot reset. If it survives, the projection layer could blend between a
pre-load world and a post-load world — motion invented across a world
replacement, exactly the class AD-15 exists to prevent. AD-13 owns
"ALL snapshot/delta application", so the fix is one clause: a snapshot clears
interpolation history; the first post-snapshot frame projects a single state.

**Verdict: minor fix — one clause in AD-15 (or AD-13).**

### A4. AD-16's light field vs AD-9 / AD-6 — **clean**

- AD-9: light sources are entities → they get global-allocator `u32` ids for
  free; the moving-lantern-dwarf case is the same entity, so no second id
  space appears. Consistent.
- AD-6: AD-16 explicitly routes the vocabulary through the AD-6 pipeline
  (sim-core source of truth → mirrored serde enums in `protocol` → exhaustive
  `match` bridges) and forbids appearance on the wire, so `light` is an enum
  kind, not a string or an RGB. Consistent, and the M2 light/appearance
  convention (data table in `gui`, sibling of `tui`'s color table) is a
  faithful sibling of the parent's Color convention.
- AD-8: trees mutate via `set_tile` (dirty set); light entities ride the
  entities full-resend. Both delta paths are the parent's, unextended. Clean.
- Nit, not a gap: AD-16 attributes the color-as-data rule to "(AD-4)"; the
  rule actually lives in the parent's Consistency Conventions (Color row) —
  AD-4 is the wire-side half. Cite both or cite the convention.

### A5. Other inheritance checks — **clean**

- **AD-14 vs AD-8/AD-9:** reconciliation keyed by sim `Id` (AD-9), render
  state derivable-from-mirror keeps the mirror (hence the wire) authoritative
  — reinforces AD-4 rather than weakening it.
- **AD-17 vs AD-1/AD-7:** rung 1 asserts world-correctness on `client-core`
  headless, byte-exact — this *depends on* AD-7's determinism and adds no new
  demand on `sim-core`. No-GPU-in-CI keeps the parent's test topology
  (scenario tests → direct lib calls) intact.
- **Stack:** the parent's closed-list rule is restated, `bevy` 0.19.0 arrives
  with justification and a version-lockstep convention against `bevy_ecs`
  0.19.0. Consistent.
- **AD-10:** F12 maps `gui` input onto *existing* protocol commands,
  "AD-10 (unchanged)" — correct; picking is a client-side act that ends in a
  command, not a new mechanism.

---

## B. The change proposal's recorded demands on this pass

| # | Demand (proposal §2 / §5 step 4 / success criteria) | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | Gate probe **sibling for the new client crate** (else AD-1's edge unguarded for the client that matters most) | **MET** | M2 Conventions "Gate probes" row: `gui` **and** `client-core` each probed for no `sim-core` edge via `cargo tree`, explicitly "siblings of the existing `tui` probe (NFR8)". Goes one better than asked — the proposal named one crate, the spine covers both new crates. Implementation is correctly left to a story; the architecture record is what the proposal demanded. |
| 2 | **NFR2 client-agnostic restatement plus a Bevy-specific bar** | **HALF-MET** | The Bevy-specific bar is fully set: NFR6 section (60 fps working zoom / ≥30 fps full vista, full world, all lights, on the named hardware, read from the overlay — a measured bar, not a felt one) + "Command acknowledgement keeps NFR2's ~200 ms". But the **client-agnostic restatement** never happened anywhere in the pass: the M2 PRD's NFR6 is Bevy-scoped and phrases the ack bar as "matches the TUI's (~200 ms)", and the old PRD's NFR2 scope-note only says NFR2 does *not* stretch. The proposal asked for two artifacts and got one. What's missing is small: a client-agnostic sentence — "any client acknowledges via next-delta within ~200 ms and keeps pace with the 10 ticks/s stream; each client's feel bar is set when that client is planned" — either in the spine (natural home: next to the NFR6 section, since the spine already restates the ack rule) or flagged as a PRD-side owed edit. Without it, a third client (the deferred Asgard adapter is already on the parent's radar) re-derives the bar from TUI-specific prose again. |
| 3 | **"How deterministic evidence works for a real renderer"** | **MET** | AD-17 is a direct, complete answer: byte-exact assertion migrates to `client-core` headless (rung 1), gui logic headless under minimal plugins (rung 2), visual truth via a scripted capture instrument with its own honesty tests ("exit 0 is not a result") but never golden-imaged in CI, judged by Wolf's eye "structurally" (rung 3). This preserves the proposal's risk-section insight (a real renderer can't be asserted cheaply) *and* the live-gate rule, and the golden-image trap is explicitly deferred with a trigger. Binds "the review process" — the strongest AD in the draft. |
| 4 | **TUI's instrument role written down durably** (success criterion 5) | **MET, thinly in the spine itself** | The durable statement landed primarily in the M2 PRD ("the TUI's load-bearing role as deterministic assertion instrument … stand[s] unchanged", lines 11–12) plus the parity rule. In the spine, the role appears only obliquely: AD-17 rung 1 ("`tui` as the live cross-check on a shared daemon"), the CLI-discipline convention defined by reference to `tui`'s, and the Structural Seed. The spine never states *why the terminal client still exists* — the exact question criterion 5 was written against. Note also a quiet role shift: the proposal said "keep `tui` as **the** deterministic assertion instrument"; AD-17 moves byte-exact assertion to `client-core` and recasts `tui` as live cross-check. Defensible — AD-13 means `tui`'s rendering *is* downstream of the same asserted mirror — but the shift deserves one sentence of acknowledgement so the proposal's risk mitigation visibly maps onto the ladder. |
| 5 | **Milestone 2 starting from outcomes, not techniques** (success criterion 4) | **MET** | The spine binds FR27–FR37 and its capability map speaks outcomes ("diorama, light, aliveness"; "input parity + picking"). The FR24 failure shape does not recur: camera style is nowhere decided (matching the proposal's "becomes a camera setting"), z-slice mechanism and world-edge treatment are explicitly deferred to story-level design binding only the outcome/bar. Techniques the spine *does* name (mirror, interpolation-as-presentation, trees-as-tiles/lights-as-entities, procedural-first meshes) are wire-modeling and evidence decisions — the altitude where an architecture pass is supposed to name mechanisms. "Bevy" in the scope line is the proposal's own decided input, not a regression. |

---

## C. Stale-parent flagging

The spine's "Parent updates owed" flags two items: Deferred "Raycast 3D view"
and the "Unreal client" mention. **The mechanism is right** (surface, don't
silently contradict; the parent is final and read-only for this pass) — the
raycast Deferred flag also implicitly covers that entry's embedded 4.1b
creature-rendering approach (sub-voxel DDA models, "never sprites or
per-creature assets"), which died with 4.1b and which M2's deferred asset
pipeline (`.vox`, authored dwarves first) would otherwise contradict.

**But the sweep was incomplete.** Also stale in the parent, unflagged:

1. **Consistency Conventions → Color row:** "…the id → RGB mapping … is a data
   table in `tui`, **shared by the 2D view and the future raycast view**".
   The raycast view is withdrawn; the clause dangles. Worse, the M2 spine's
   own light/appearance convention describes itself as "sibling to `tui`'s
   color table" — it actively builds on the very row whose second half it
   knows to be dead. One line in Parent updates owed.
2. **Deferred → "Mouse/touch input — phase 2, confined to `tui`'s input
   layer"**: F12 gives `gui` picking — mouse input's real home is now the
   Bevy client, and "confined to `tui`'s input layer" is flatly wrong for M2.
   This one can actively mislead a story author (it reads as a rule, not a
   note). One line.
3. *(Minor, metadata)* parent frontmatter `binds: [FR1-FR26, …]` still counts
   the withdrawn FR24; and the parent's "Raycast 3D view" flag could name the
   creature-approach clause explicitly rather than by inclusion. Worth folding
   into whichever parent-errata edit eventually lands; not worth its own flag.

Checked and genuinely still valid (no flag needed): LLM sidecar entry (AD-10
queue boundary unchanged by M2), tokio/async, TUI-framework trigger,
hierarchical pathfinding, binary protocol, protocol generalization,
parallel scheduling, save-format stability and multi-machine (M2's
portability convention is consistent with "nothing may preclude").

---

## Summary of required edits (all small)

1. Add AD-6's "tui depends on nothing else" sentence to **Parent updates
   owed**; note the amendment in AD-13 (A1).
2. One bridge sentence above the M2 dependency graph declaring it the
   superseding graph and restating the preserved direction rule (A2).
3. One clause: snapshot reset clears the previous-tick/interpolation buffer
   (A3).
4. One client-agnostic ack/keeps-pace sentence near the NFR6 section, or an
   explicit owed-PRD-edit flag (B2).
5. One sentence in or near AD-17 stating the TUI's instrument/debug role and
   acknowledging the byte-exact duty's migration to `client-core` (B4).
6. Two lines added to **Parent updates owed**: the Color row's raycast clause;
   the mouse-input-confined-to-tui Deferred entry (C).

Nothing found rises to a structural defect: no new AD weakens an inherited
invariant in substance, AD-16/AD-14/AD-17 actively reinforce AD-4/AD-6/AD-7,
and the proposal's five demands are 3 met / 2 half-met with one-sentence fixes.
The draft's failures are all failures of *explicitness* — ironic but cheap to
fix, given the parent's own house style (AD-10's amendment block) shows exactly
how.
