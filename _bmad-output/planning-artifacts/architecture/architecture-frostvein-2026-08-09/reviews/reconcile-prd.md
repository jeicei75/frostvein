# Reconciliation — PRD (prd-frostvein-2026-08-09 + addendum) vs ARCHITECTURE-SPINE

Method: every FR, NFR, Visual Target bar, counter-metric, gate/parity rule, and
addendum item is walked and classified **DECIDED** (the spine settles it),
**DEFERRED** (explicitly, with a trigger or story-level homing), or **DROPPED**
(silent). Silence is flagged only where it could cause divergence or where the
PRD explicitly made the architecture pass owe something.

Sources:
- PRD: `_bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/prd.md`
- Addendum: `_bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/addendum.md`
- Spine: `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md`

## Functional requirements

| FR | Verdict | Where / note |
| --- | --- | --- |
| FR27 trees | DECIDED | AD-16: `Material` variants, worldgen-seeded, solidity/`set_tile`/AD-8 path named. Density assumption confirmed as story-level. But see Gap 4: the PRD's "visible to every client (the TUI shows them as glyphs)" half has no binding home — the capability map routes F10 to `sim-core` + `protocol` only. |
| FR28 static emitters | DECIDED | AD-16: entities with `light: Some(kind)`; appearance client-side by kind table. |
| FR29 lanterns | DECIDED | AD-16: same concept on a moving entity; no-economy assumption confirmed. Cleanly severable (a field), which keeps the PRD's cut order viable — but the spine never says so (see Gap 3). |
| FR30 protocol vocabulary | DECIDED | AD-16 tail + structural seed note "vocabulary growth only — no shape changes"; AD-6 mechanics (mirrored serde enums, exhaustive bridges) named. |
| FR31 diorama/camera/zoom | DEFERRED appropriately | Bound to `gui` projection via capability map; camera/zoom behavior is story-level, testable headless per AD-17 rung 2. Acceptable spine altitude. |
| FR32 cold/warm live, atmosphere | DECIDED/DEFERRED | AD-15 sanctions client-side atmosphere; AD-16 kind→appearance table. Values/material depth punted to tech-art guidelines — see Gap 2. |
| FR33 z-slice | DEFERRED explicitly | Deferred entry, outcome bound, addendum collision cited. Correct homing. |
| FR34 aliveness from the wire | DECIDED | AD-14 (mirror projection) + AD-15 (interpolate, never extrapolate). Note: FR34 lists "static lights flicker" under wire-driven life; AD-15/AD-16 decide flicker is client-side animation keyed by kind (wire never carries flicker). That is a real decision resolving an ambiguity in FR34, made on the record — no flag, but the epics pass should not re-litigate it. "Idle dwarves wander" rides on M1 FR4 sim behavior (already exists); no new sim work implied, consistent with the baseline rule. |
| FR35 full parity | DECIDED | Capability map: `gui` → existing protocol commands, AD-10 unchanged. No new command surface — correct. |
| FR36 picking | DEFERRED appropriately | Named in AD-17 rung 2 (headless-testable) and structural seed. Mechanism is story-level; fine. |
| FR37 protocol-only consumer, coexistence | DECIDED | AD-13 crate graph (`gui` → `protocol` + `client-core` only), runtime topology shows both clients on one daemon; NFR8 probe enforces the edge. |

## NFRs

| NFR | Verdict | Where / note |
| --- | --- | --- |
| NFR5 no drift + carve-out | DECIDED | AD-15 restates and extends the carve-out to time (no extrapolation). **Half-met on enumeration:** AD-15's sanctioned list is "sky, aurora, snowfall, flicker animation" — it omits the PRD's dig-face **cosmetic chips** (Visual Target "work leaves evidence" bar), which is the riskiest carve-out member because it is world-anchored cosmetics spawned at sim events, exactly the "acquires sim meaning silently" failure NFR5 warns about. See Gap 5. |
| NFR6 measured bar | **MET (owed, delivered)** | PRD made this a blocking architecture-time item; the spine sets it: 60 fps working zoom / ≥30 fps vista, full world + dwarves + lights, WSLg devpod, read from the frame-time overlay (instrument convention also set); ~200 ms ack kept. Nothing owed remains. |
| NFR7 determinism survives F10 | DECIDED (test content story-level) | AD-16 routes trees/lights through worldgen + AD-7; capability map binds F10 to AD-7. The specific "scenario tests cover trees and light emitters" sentence is not restated, but scenario-test content is legitimately story-level and AD-7 + the parent's scenario-test convention carry it. No flag. |
| NFR8 gate probe | **MET+ (owed, delivered)** | Consistency convention adds `cargo tree` probes for **both** `gui` and `client-core` (the PRD asked only for the gui twin). |

## Visual Target

- **The view** (diorama, one continuum/two registers, discrete z-levels): bound
  through FR31/FR33 as above. The "far register is the same view, not a mode"
  sentence has no spine echo; it is a presentation bar Wolf judges at rung 3.
  Acceptable.
- **The light** (cold-against-warm, eye lands on camp, real in-world sources):
  AD-16 makes the sources real world state; contrast quality is rung-3 territory.
  Decided at the right altitude.
- **Reference-bound bars** (sky-as-illuminant, snow-cap read, work-evidence,
  value discipline, blue-ice variation, edge dissolve): the PRD explicitly homes
  their depth — "values, material rules" — in the **tech-art guidelines
  deliverable**. The spine mentions that deliverable only inside the Deferred
  asset-pipeline entry ("Tech-art guidelines deliverable opens with it"), i.e.
  gated on authored assets arriving. But five of the six bars are procedural-era
  bars needed in the milestone's **first third** (value discipline and
  sky-illuminant especially — they gate the boot-frame wow). As homed, the
  values/material depth has no owner until dwarven assets show up. **Gap 2.**
  Edge dissolve itself is properly deferred (world-edge entry). Work-evidence's
  cosmetic-chips half → Gap 5.
- **Two wow beats**: beat 2's substance (alive from real wire state) is decided
  by AD-14/AD-15. Beat 1's *timing* is a counter-metric — see Gap 3.
- **Anti-requirements table**: outcome bars judged by Wolf; AD-17 rung 3 makes
  his eye the structural judge. Correctly not re-architected.

## Sign-off gate

The PRD gate has **two halves**: (a) *before* implementation, Wolf approves a
cheap "here is what you will see" artifact (target frame, mock, sketch,
generated reference); (b) *after*, Wolf views the built result live against the
approved artifact. The spine's AD-17 rung 3 covers (b) well — scripted
`--capture` output "are the artifact for the PRD sign-off gate", Wolf's eye is
the judge — and the assumptions-resolved paragraph confirms per-story
granularity. But AD-17's captures are, by construction, captures of **built**
work; the opening half — pre-approval, the actual FR24-class fix — has no home
anywhere in the spine, and AD-17's phrasing ("the artifact for the PRD sign-off
gate") invites reading the capture as the whole gate. A story pass derived from
this spine can honestly claim gate compliance while skipping pre-approval
entirely — recreating the exact defect class the gate exists to kill. **Gap 1
(half-met).**

## Counter-metrics

- **10–14 story cap + cut order** (FR29 first; then FR35/36 → camera+speed):
  DROPPED — the spine never mentions either. The cap is legitimately a
  planning-pass concern. The **cut order**, though, has an architectural
  shadow: the spine should (and implicitly does, via AD-16's field-shaped
  lanterns and AD-10's unchanged command set) keep the cuts severable — but it
  never says so, and nothing warns the epics pass that FR29 and the parity
  tail must not become load-bearing for other stories. Folded into **Gap 3**.
- **First-third wow sequencing**: DROPPED, and here silence bites. AD-13
  *mandates* structural work before rendering: a new `client-core` crate owning
  all snapshot/delta application, plus a `tui` adoption story that retires the
  in-crate state. A story plan derived naturally from the spine's structure
  (client-core → tui adoption → gui envelope → gui rendering) back-loads the
  boot-frame wow — precisely what the PRD calls "wrong, cap or no cap". The
  spine sequences exactly one thing ("First `gui` story proves the envelope")
  and never states that the tui-adoption story, rung-1 harness work, or
  client-core completeness must not gate the first-third payoff. **Gap 3.**
- **No TUI regression / sim changes update the TUI**: half-homed. The light
  table convention ("sibling to `tui`'s color table") and AD-13's shared mirror
  imply it, and FR27's PRD text carries the glyph requirement — but the parity
  rule is never restated as binding, and the capability map's F10 row lists
  only `sim-core` + `protocol`, so a vocabulary story can pass the map without
  touching `tui` rendering. **Gap 4 (divergence risk, minor).**
- **Docs re-readable in one sitting**: the spine is lean; met in practice.

## Parity rule (scope-shape)

Forward half (Bevy catches up; TUI not extended for Bevy-only work): consistent
— no spine element extends the TUI. Backward half (sim/bug changes update the
TUI): see Gap 4 above.

## Addendum items

| Item | Verdict |
| --- | --- |
| Z-level navigation (mousewheel/zoom collision) | DEFERRED explicitly, collision cited, outcome bound. Homed. |
| World-edge treatment | DEFERRED explicitly, candidates cited, no-raw-edge bar bound. Homed. |
| Plateau terrain guidance | Not in spine — correct; the addendum itself homes it to FR27's story orbit as non-binding guidance. |
| Vista mountain silhouette | DEFERRED explicitly ("decide on the record at worldgen tuning"; FR2 assumption flagged for conscious revisit). Homed. |
| Valheim lesson (budget to light/air over geometry) | ABSORBED — stack section: code-built meshes, no voxel crate, no asset pipeline, single dependency. The tone landed. |
| Asset pipeline (.vox / bevy_vox_scene, Bevy churn caveat) | DEFERRED with trigger, path named; the churn caveat is partially answered by the Bevy-versioning convention (one 0.x line workspace-wide). Homed. |

## Owed-obligation scorecard

| Obligation | Status |
| --- | --- |
| NFR6 number set at architecture time | MET |
| NFR8 gate probe | MET (extended to client-core) |
| Evidence discipline (live-gate, exit-0-is-not-a-result) | MET — AD-17 is the strongest section of the spine |
| Z-slice homing | MET (explicit deferral) |
| World-edge homing | MET (explicit deferral) |
| Sign-off gate (both halves) | HALF-MET — Gap 1 |
| Visual-bar depth homing (tech-art guidelines) | HALF-MET — Gap 2 |
| First-third sequencing / cut-order severability | DROPPED — Gap 3 |
| Parity rule backward half | HALF-MET — Gap 4 |
| NFR5 carve-out enumeration (cosmetic chips) | HALF-MET — Gap 5 |

## Gaps (ranked)

1. **Sign-off gate's opening half is unhomed.** AD-17 delivers only the closing
   half (built-result captures, live viewing). The pre-implementation approval
   artifact — the actual structural fix for the FR24 defect class — appears
   nowhere, and AD-17's "captures are the artifact for the PRD sign-off gate"
   phrasing actively invites conflating the two. One sentence in AD-17 (or the
   review-process row) distinguishing the pre-approval artifact from the capture
   closes it.
2. **Tech-art guidelines depth is gated on the wrong trigger.** The PRD homes
   the reference-bound bars' depth (values, material rules) in the tech-art
   guidelines deliverable; the spine defers that deliverable behind the
   asset-pipeline trigger ("dwarves expected first"). Value discipline and
   sky-as-illuminant are procedural-era, first-third bars — as written they
   have no owner until authored assets arrive, which may be never inside M2.
3. **First-third wow sequencing and the cut order are silent, and AD-13's
   structure fights the sequencing rule.** The mandated client-core extraction
   and tui-adoption story, ordered naively from the spine, back-load the
   boot-frame wow the PRD requires in the first third. Nothing marks FR29 /
   the parity tail as severable per the decided cut order either. The spine
   needs one line: the tui-adoption story and full mirror completeness do not
   gate the first-third boot-frame payoff, and no M2 structure may make FR29
   or FR35/36-beyond-camera+speed load-bearing.
4. **Parity rule's backward half has no binding home.** F10's capability row
   maps to `sim-core` + `protocol` only; TUI rendering of trees/light entities
   (glyphs, color-table growth) rides on an aside. A vocabulary story can
   satisfy the spine while the TUI stagnates on sim-level change.
5. **AD-15's sanctioned-atmosphere list omits dig-face cosmetic chips** — the
   one carve-out member the PRD names that is world-anchored (spawned where sim
   events happen) and therefore the likeliest to acquire sim meaning silently.
   Either add it to the sanctioned list with the same never-sim-meaning clause,
   or state that world-anchored cosmetics need explicit sanction per story.

Everything else — all of F10's modeling, the mirror/projection split, the
evidence ladder, NFR6/NFR8, all six addendum items, the deferred stack — is
decided or deferred cleanly at the right altitude. No FR or NFR is dropped
outright; the five gaps are all quiet-requirement or process-half misses of
exactly the kind the AD structure tends to shed.
