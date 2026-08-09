# Rubric Review — Architecture Spine, frostvein M2 (Bevy client)

- **Artifact:** `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md` (draft, 2026-08-09)
- **Parent:** `architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md` (final)
- **Driving PRD:** `prd-frostvein-2026-08-09/prd.md` + addendum
- **Reviewer:** rubric walker (BMad bmad-architecture Reviewer Gate)
- **Date:** 2026-08-09

## Verdict

**CONDITIONALLY APPROVED.** The spine is sound: the mirror-then-project
paradigm plus AD-13..AD-17 fix the genuinely dangerous divergence points for
M2 (duplicated delta semantics, ECS-scattered ingestion, client-invented
state, incompatible world-content modeling, evidence collapse for a real
renderer), and it ratifies the brownfield codebase accurately. Two amendments
are required before story work: surface the AD-6 contradiction it currently
makes silently (RR-1), and add the sim↔Bevy coordinate-mapping convention
(RR-2). Both are one-paragraph fixes; neither reopens a decision.

## Findings

### RR-1 (MAJOR) — The new graph silently contradicts inherited AD-6

Parent AD-6's rule states: *"`simd` and `tui` both depend on it; `tui`
depends on nothing else in the workspace."* The M2 dependency graph adds
`tui --> client-core`, and AD-13 mandates it. The change is right — but the
spine's own opening section sets the standard: parent contradictions are
*"surfaced, not silently contradicted."* It surfaces two stale parent
Deferred entries (Raycast, Unreal) yet omits this one, which is a *rule*
clause, not a Deferred note. Fix: add AD-6's tui-dependency clause to the
"Parent updates owed" list (AD-13 amends it: clients depend on `protocol` +
`client-core`, nothing else). Note the same stale wording lives in repo docs
(`CLAUDE.md` ground rule 2: "`tui` (client, depends on `protocol` only)";
`docs/technical-preferences.md` per AD-6's own amendment history) and in the
gate's inverted probe rationale comment — the AD-13 adoption story must
carry those doc/probe updates or the gate will honestly report a graph the
docs say is illegal.

### RR-2 (MAJOR) — Coordinate mapping sim↔render is a whole dimension left silent

Parent conventions fix sim geometry hard (z vertical, 0 = lowest; row-major
index; inclusive rects). Bevy is Y-up right-handed. The transform between
them — axis mapping, voxel size, world origin — is needed identically by at
least three M2 stories: projection (AD-14 reconciliation), picking (FR36,
which the PRD names M2's hardest input work, and which needs the *inverse*
of the same transform, including on sliced z-levels), and the capture
instrument (AD-17 rung 3, whose "changes when the world changes" checks
assume a stable framing). Two stories choosing independently is exactly the
divergence class this spine exists to prevent, and checklist dimension 8's
"a whole dimension left SILENT is a finding" applies. Fix: one Consistency
Conventions row — e.g. "sim (x, y, z) ↦ Bevy (x, z_up=sim z, y or −y),
1 voxel = 1.0 world unit, defined once in `gui` (single transform pair,
forward + inverse); no other site converts coordinates." The specific
mapping matters less than there being exactly one.

### RR-3 (MINOR) — "No shape changes, only vocabulary" vs AD-16's new `light` field

The structural seed repeats FR30's "vocabulary growth only — no shape
changes" for `protocol`, but AD-16 requires every light source to be "an
entity carrying a `light` field naming a kind" — on the common reading,
adding a field to the entity wire struct *is* a shape change. The intent is
clearly "no new message types or mechanisms," and the addition is additive
and typed, so the design is fine — but a DEV agent taking the seed's line
literally could contort FR29's carried-light into an existing field or a
separate entity kind to avoid "changing a shape." Fix: one clarifying
clause defining what "shape" means here (message set and delta/snapshot
mechanics fixed; additive typed fields on existing structs are vocabulary).

### RR-4 (MINOR) — Crate-enumerated parent conventions not extended to the two new crates

Parent conventions that enumerate crates go stale at five/six crates and the
M2 additions table does not re-point them: `#![forbid(unsafe_code)]` "in all
four crates"; errors "`thiserror` in `sim-core`/`protocol`, `anyhow` in
`simd`/`tui`" (where do `client-core` and `gui` sit? — presumably
`thiserror` for the lib, `anyhow` for the bin, but it is currently silent).
Also stale in the parent for the same reason: the color-table convention
still says the `tui` table is "shared by the 2D view and the future raycast
view" (dead since the 2026-08-08 pivot) — belongs on the "Parent updates
owed" list beside the two entries already there. One conventions row plus
one owed-update line closes all of this.

### RR-5 (INFO) — AD-13's "neither reimplements any of it" is review-enforced, not probe-enforced

The new gate probes (no `sim-core` edge for `gui`/`client-core`) guard the
dependency direction but cannot detect a client quietly re-growing its own
delta application beside `client-core`. Acceptable: the `tui` retirement
story removes the only existing second implementation, making any
reappearance a reviewable diff rather than a latent fork. No change
required; flagged so the retirement story is understood as load-bearing for
AD-13's enforceability, not cleanup — it should not be a cut-list casualty
if the 10–14 story cap bites (the PRD's decided cut order doesn't touch it,
which is correct).

## Checklist walk

**1. Real divergence points fixed, none missed — PASS with RR-2.**
The five ADs map cleanly onto the real M2 forks: AD-13 (two delta
implementations — real: `tui` currently owns exactly that state in
`crates/tui/src/view.rs`), AD-14 (ingestion scattered across Bevy systems;
the delete-all-and-reproject reproducibility property is a genuinely sharp
test oracle), AD-15 (prediction creep), AD-16 (worldgen story vs client
story modeling trees/lights incompatibly; appearance leaking into the wire),
AD-17 (evidence collapse when frames stop being byte-comparable — the
correct generalization of the live-gate lesson). NFR6's number is set, as
the PRD required at this altitude. The one missed fork is the coordinate
transform (RR-2). Z-slice and world-edge are single-story questions with
outcomes bound — correctly not fixed here.

**2. Rules enforceable — PASS.** AD-13/AD-16 are compiler- and
probe-enforced (crate graph, exhaustive `match`); AD-14's rule carries its
own test oracle; AD-17 is process-enforced with each rung's judge named,
including the honest structural admission that rung 3's judge is Wolf's eye.
AD-15's "never extrapolates" is the softest (review-enforced), but the
"mirror holds only wire-delivered states" clause makes violation structural
rather than a matter of care — consistent with AD-7's philosophy. RR-5
notes AD-13's one review-enforced clause.

**3. Deferred entries can't cause interim divergence — PASS.** Each
deferred item is either single-story (z-slice, world-edge, silhouette),
trigger-gated with the line held by a live convention (native Windows via
the portability row; asset pipeline via the closed-list rule), or already
covered by a standing AD in the meantime (golden-image CI by AD-17). No
entry leaves two concurrent units free to choose differently.

**4. Tech verified-current — PASS on claim hygiene.** The stack claim is
dated and sourced ("crates.io / bevy.org, 2026-08-09"); Bevy 0.19.0 matches
the workspace's existing `bevy_ecs = "0.19.0"` (root `Cargo.toml`), and the
same-0.x-line convention makes the pairing an invariant rather than a
coincidence. The WSLg envelope claim is evidence-based (D3D12 passthrough,
Mesa 25.3.5, `glxinfo`), honestly bounded ("wgpu prefers Vulkan via Dozen —
unproven until run"), and — best practice — the first `gui` story is
required to prove the envelope before anything builds on it. Fact
re-verification is the other reviewer's lane.

**5. Ratifies the brownfield — PASS.** Verified against code: the four
crates exist as described (`crates/sim-core`, `protocol`, `simd`, `tui`);
`protocol` holds the enum vocabulary AD-16 extends (`Material` with
stone/soil/ice/snow); `scripts/gate.sh` already carries the inverted `tui`
probe the new probes are siblings of; `tui`'s in-crate client state that
AD-13 retires actually exists. The seed is the current tree plus exactly
the two new crates. One doc-level contradiction is RR-1's, in the spine's
favor on substance.

**6. Covers the driving PRD — PASS.** F10→AD-16, F11→AD-14/15/16+NFR6 bar,
F12→AD-10/AD-14 (picking thin, but its architectural residue is RR-2),
F13→AD-13; NFR5→AD-15 (with the carve-out carried over verbatim), NFR6 set
with two numbers and an instrument, NFR7→AD-16 routing through AD-7/AD-8,
NFR8→the gate-probes row. All four PRD `[ASSUMPTION]`s are dispositioned on
the record. The sign-off gate (the FR24 structural fix) is anchored in
AD-17 rung 3 rather than left as PRD prose — good.

**7. No new AD weakens an inherited one — PASS on substance, RR-1 on
process.** AD-14/15 tighten AD-4; AD-16 extends AD-4's color convention and
lands vocabulary per AD-6's mechanism; AD-13 centralizes AD-8's client-side
semantics without touching its sim side; the Bevy-versioning row protects
AD-7's headless-`bevy_ecs` arrangement. The single inherited-text conflict
(AD-6's tui clause) is a deliberate, justified amendment made without being
surfaced — process defect, not design defect.

**8. Every owned dimension decided/deferred/questioned — PASS with RR-2,
RR-4.** The operational envelope the checklist singles out is present, not
gestured at: runtime topology named (WSL2 devpod, WSLg, localhost TCP),
GPU path evidence given, the unproven part named as unproven and assigned
to the first story, no-GPU-in-CI decided (AD-17 rung 2), portability
decided as a convention, Windows deferred with trigger. Silent dimensions
found: the coordinate transform (RR-2) and the new crates' place in the
crate-enumerated conventions (RR-4). Terse-but-not-silent is otherwise
achieved; the document honors the one-sitting counter-metric.
