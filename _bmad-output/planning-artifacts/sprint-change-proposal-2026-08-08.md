# Sprint Change Proposal — 2026-08-08

**Project:** frostvein
**Trigger story:** 4.1a — Behold the Fortress in Depth
**Raised by:** Wolf, at 4.1a's code review
**Scope classification:** **MAJOR** — fundamental replan (phase boundary moves, client strategy changes)
**Mode:** Batch

---

## 1. Issue Summary

**Story 4.1a shipped a technically sound feature that did not buy what it was for.**

The raycast depth view was implemented, reviewed by four layers with no coverage holes, and passed
an independently re-run gate. Wolf then ran it live and judged it **"quite far from wow effect"**,
adding that he is **"not sure can we achieve wow effect with tui"**.

**A requirements miss sits underneath it.** Wolf wanted an **isometric 3D camera**; the story
specified and shipped a **first-person raycast** view. In his words: *"I meant actually more like
isometric 3d camera view but I didn't manage to clarify that."*

**Why no review layer caught it — and this is the transferable part.** All four layers audit *"does
the code match the spec?"*, and it does, faithfully. Even the Feature Auditor, whose question is
*"would the user get the outcome the story promises?"*, is defeated when the **promise itself** is
the wrong thing. This is a **new subclass of the tracked spec-defect category**:

| Known subclass | Example | Caught by |
| --- | --- | --- |
| AC unmeetable as written | 2.3's AC9, 4.1a's AC3 | Review, reliably (4 instances in 7 stories) |
| **AC meetable, implemented, and not what the user wanted** | **4.1a (this)** | **No layer. By construction.** |

**The root cause is visible in the requirement itself.** FR24 reads *"The **raycast** 3D view is its
own story late in the milestone."* It names a **mechanism, not an outcome** — the exact failure the
AC-authoring rule adopted at story 3.1 exists to prevent, sitting one level above where that rule
reaches. Nobody had to ask what Wolf wanted to *see*, because the FR already said how to *draw* it.

### Evidence

- Live judgement from Wolf (above), after a real interactive session.
- Review outcome: 4/4 layers completed, 3 convergences, gate GREEN independently re-run, dev's
  reported capture numbers reproduced exactly by the Acceptance Auditor (honest reporting confirmed).
  **The code is not the problem.**
- Story 3.3's FR23 verdict was already provisional and already pointed here: *"most likely we need
  to get to the 3d first to say."* The 3D now exists and the answer is no.

---

## 2. Impact Analysis

### Epic impact

| Epic | Status | Impact |
| --- | --- | --- |
| Epics 1–3 | done | **None.** All sim and 2D-client work stands. |
| **Epic 4** | in-progress | **Closed early.** 4.1a done-but-unmerged; **4.1b dropped**. |
| Milestone 1 | done | **Unaffected** — the walking skeleton was complete before Epic 4. |

**Story 4.1b is dropped.** It carried sub-voxel *glyph* creature models (~10×5×13 boxes-as-code,
fine-step DDA sampling, distance LOD, seed-derived palette swaps) built to settle an identity
question that a Bevy client answers far better with a camera that can actually be chosen. Building
it now is money burned.

**Story count: 12 → 11.** Inside the 8–12 counter-metric cap, so **the cut list is not invoked and
FR16 (save/load) is not at risk.** Epic 4's split rationale in `epics.md` becomes moot.

### Requirement impact

- **FR24 (raycast 3D view)** — withdrawn from phase one, re-homed to Milestone 2 **as an outcome**.
  Note the uncomfortable honesty: FR24's *letter* was delivered (4.1a **is** a raycast 3D view,
  gate green) while its *intent* was not. Re-write it to say what the boss should be able to see,
  never how to render it.
- **FR23 (icy-grim identity)** — its phase-one obligation is **met**; its 3D ambition moves to
  Milestone 2. See success criteria below.
- **NFR2 (feels alive)** — written TUI-specific (*"The **TUI** keeps pace with the sim at 10
  ticks/sec"*). Phase one's bar is met and stays met. Milestone 2 needs its own client-side bar;
  do not silently stretch NFR2 over a renderer it was never measured against.
- **FR22 (glyph dwarves), FR21 (keymap)** — unaffected, correctly TUI-scoped, still true.

### Success criteria — phase one

| # | Criterion | Verdict |
| --- | --- | --- |
| 1 | Walking skeleton as a headless scenario test (FR26) | **MET** (Milestone 1) |
| 2 | Same scenario live in the TUI, feel floor (NFR2), Wolf signs off icy-grim (FR23) | **MET, qualified** — see below |
| 3 | Quality gate green across the workspace (NFR4) | **MET** — re-verified 2026-08-08 |
| 4 | Planning docs re-readable in one sitting | **MET** |

**Criterion 2 is met and the qualification is recorded rather than hidden.** The criterion asks for
sign-off **in the TUI**, and at 3.3 Wolf gave it: *"looks ok for 2d tui game atm."* He also said
*"not sure how much more visually pleased it could be without designing own font or something…
most likely we need to get to the 3d first to say"* — and **that escalation was self-imposed, not
something the PRD required.** It is what created 4.1b's sign-off obligation. Phase one closes on
its own stated terms; the icy-grim *ambition* carries to Milestone 2 as ambition, not as an unmet
phase-one criterion.

### Architecture impact — **zero changes required**, and that is the headline

A Bevy client is **another `protocol` consumer**. Under AD-1 (clients hold zero game logic) and
AD-4 (clients render a world, never rules), `sim-core`, `simd` and `protocol` need **no changes at
all**. Nine stories of sim work carry over intact. This is the four-crate spine paying off exactly
as designed.

**Deliberately NOT edited now** (YAGNI is policy; Milestone 2 gets its own architecture pass):

- The spine is not amended for a client that has no stories yet.
- Two concrete inputs are **recorded here** for that pass rather than built now:
  1. `scripts/gate.sh`'s `cargo tree -p tui | rg -q sim-core` probe needs a **sibling for the new
     client crate**, or the AD-1 edge goes unguarded for the client that matters most.
  2. **NFR2 needs a client-agnostic restatement plus a Bevy-specific bar.**

### Artifact impact

| Artifact | Change |
| --- | --- |
| `prd.md` | FR24 withdrawn/re-homed; FR23 + NFR2 notes; success criterion 2 verdict; counter-metric FR24 line; Bevy added to Future phases |
| `epics.md` | Epic 4 closed; story 4.1b removed; FR coverage map (FR23, FR24) updated |
| `sprint-status.yaml` | 4.1b dropped; `epic-4: done`; Milestone 1 closed; retro action item added |
| Architecture spine | **No edit.** Inputs recorded above for the Milestone 2 pass. |
| UX spec | N/A — none exists |
| `deferred-work.md` | Already updated at review (5 items) |

### Technical impact

- `main` stays **2D-only**. Branch `4-1a-behold-the-fortress-in-depth` (5 commits, gate green) is
  kept unmerged. This is **consistent with the PRD's own counter-metric**: *"No code exists that
  serves only a future phase (YAGNI is policy)."*
- **The TUI is not retired.** It is demoted from product to **instrument and debug client**, and
  that role is now load-bearing — see the risk below.

### Risk carried forward — the one thing that must not be lost

**This project's evidence discipline rests on deterministic scripted TUI captures**
(`tui --frames N --key …` over a text framebuffer: byte-comparable, assertable in CI). That is what
makes "unit-green is never feature-proof" enforceable, and it is what caught a broken instrument at
2.2 and an irreproducible recipe at 3.3. **A real 3D renderer cannot be asserted on this cheaply.**

**Mitigation, and it is the reason the TUI stays:** both clients speak the same protocol, so sim
behaviour stays provable through the cheap one. Keep `tui` as the deterministic assertion
instrument; let Bevy carry the visual ambition with only sparse screenshot-level checks.

---

## 3. Recommended Approach

**Hybrid: MVP Review (Option 3) + partial rollback of scope, not of code.**

1. **Close phase one now.** Milestone 1 is done and all four success criteria are met.
2. **Close Epic 4 early.** 4.1a `done` and unmerged; 4.1b dropped.
3. **Withdraw FR24 from phase one**, re-home to Milestone 2 rewritten as an **outcome**.
4. **Open Milestone 2 for the Bevy client with its own planning pass** — product brief or PRD
   amendment, then architecture, then epics. **Do not write Bevy stories inside Epic 4.**

### Options considered and rejected

| Option | Verdict |
| --- | --- |
| **Direct adjustment** — rewrite Epic 4 as the Bevy epic | **Not viable.** Plans a whole new client (window, camera, input, render loop, asset strategy) inside an epic scoped for one TUI story, with no architecture pass. Effort High, risk High. |
| **Rollback** — revert 4.1a from history | **Not viable and not needed.** Wolf's branch-not-merged call achieves the same outcome at zero cost and keeps the work legible. Effort Low, value nil. |
| **MVP review** (selected) | **Viable.** Reduces phase-one scope by exactly one withdrawn FR and one dropped story, closes a milestone honestly, and gives the new client the planning it warrants. Effort Low now, risk Low. |

**Rationale.** The pivot is a *strategy* change, not a *correctness* change — nothing built is
wrong. The cheapest honest move is to stop adding to a plan whose remaining item no longer serves
the goal, close the milestone on criteria that are genuinely met, and re-plan the new surface
properly. Bevy is not a story; it is a milestone.

---

## 4. Detailed Change Proposals

### 4.1 — `prd.md` · FR24

> **OLD**
> - **FR24** — The raycast 3D view is its own story late in the milestone.
>   Required for phase one — Wolf's override (2026-08-01) of this FR's earlier
>   may-slip clause; it no longer slips and is off the cut list.
>
> **NEW**
> - **FR24** — ~~The raycast 3D view is its own story late in the milestone.~~
>   **WITHDRAWN FROM PHASE ONE, 2026-08-08, and re-homed to Milestone 2.** Story 4.1a delivered
>   this FR *to the letter* — a raycast 3D view, gate green, four review layers clean — and Wolf
>   judged the live result "quite far from wow effect". He had wanted an **isometric** camera and
>   the FR never said so, **because this FR named a mechanism ("raycast") instead of an outcome.**
>   That is the same defect class the AC-authoring rule guards against one level lower down.
>   The code is kept on branch `4-1a-behold-the-fortress-in-depth` and deliberately **not merged**
>   (see counter-metric: no code serves only a future phase). Milestone 2 re-states this as an
>   outcome — *what the boss should be able to see and feel* — never as a rendering technique.

**Rationale:** withdraws the requirement, records *why* it misfired, and leaves an instruction that
prevents the same shape being written again.

### 4.2 — `prd.md` · FR23 (append)

> **NEW** (appended to FR23)
> **Phase-one obligation MET (2026-08-08).** Success criterion 2 asks for sign-off on the icy-grim
> look *in the live TUI*; Wolf gave it at story 3.3 — "looks ok for 2d tui game atm". His further
> "we need to get to the 3d first to say" was an **escalation beyond this FR's phase-one bar**, and
> created story 4.1b's sign-off obligation. 4.1b is dropped; the icy-grim-in-depth ambition moves
> to Milestone 2's client as **ambition, not as an unmet phase-one criterion**.

### 4.3 — `prd.md` · NFR2 (append)

> **NEW** (appended to NFR2)
> **Scope note (2026-08-08):** this NFR is written TUI-specific and is **met** for phase one. It
> does **not** silently extend to Milestone 2's Bevy client — that client needs its own measured
> bar, set when it is planned. Do not stretch a number over a renderer it was never measured against.

### 4.4 — `prd.md` · Counter-metrics

> **OLD**
>   FR24 (raycast view) was removed from the cut list by Wolf's override,
>   2026-08-01.
>
> **NEW**
>   FR24 (raycast view) was removed from the cut list by Wolf's override,
>   2026-08-01, and then **withdrawn from phase one entirely on 2026-08-08** — see FR24. Final
>   phase-one story count: **11**, inside the 8–12 cap, so the cut list was never invoked and FR16
>   (save/load) was never at risk.

### 4.5 — `prd.md` · Future phases (append)

> **NEW**
> - **A Bevy 3D client (Milestone 2).** Unreal was dropped 2026-08-08 in favour of Bevy: Rust, one
>   workspace, one `cargo`/gate loop, no editor binary and no second toolchain — decisively better
>   for agentic development. It is **another `protocol` consumer**, so AD-1/AD-4 mean `sim-core`,
>   `simd` and `protocol` need no changes. Isometric vs first-person vs orbit stops being an
>   architecture decision there and becomes a camera setting.
>   **The TUI is NOT retired** — it stays as the 2D debug client *and*, load-bearingly, as the
>   deterministic assertion instrument the whole evidence discipline depends on.
>   Needs its own planning pass; no code and no abstraction for it may exist before then.

### 4.6 — `epics.md` · Epic 4 header

Replace the Epic 4 preamble (split rationale, story-count arithmetic, prerequisite note) with a
closure record: closed early 2026-08-08, 4.1a done-but-unmerged, 4.1b dropped, FR24 withdrawn,
final story count 11. **Keep** the T3 deterministic-opening-camera note — it was a real fix that
outlives this epic and still protects the TUI's captures.

### 4.7 — `epics.md` · Story 4.1b

Remove the story body; replace with a **DROPPED** record stating what it was, why it was dropped,
and where FR23's 3D ambition went. Deleting it outright would erase the trail.

### 4.8 — `epics.md` · FR Coverage Map

> **OLD**
> FR23: Epic 1 - Icy-grim visual identity (… PROVISIONAL at 3.3 …, deferred to Story 4.1b)
> FR24: Epic 4 - Raycast 3D view (firm scope …; split … into 4.1a renderer + 4.1b sub-voxel dwarves)
>
> **NEW**
> FR23: Epic 1 — Icy-grim visual identity. Phase-one obligation **MET** at 3.3 (Wolf's live TUI
>   sign-off). The icy-grim-in-depth ambition moved to Milestone 2 when 4.1b was dropped, 2026-08-08.
> FR24: **WITHDRAWN from phase one 2026-08-08** and re-homed to Milestone 2 as an outcome. Epic 4
>   delivered 4.1a (kept unmerged); 4.1b dropped.

### 4.9 — `sprint-status.yaml`

- `4-1b-dwarves-in-depth` → **`dropped`**, with a comment recording why (`dropped` is a new value;
  the STATUS DEFINITIONS block gains a line so the file stays self-describing).
- `epic-4` → `done`, commented as **closed early**.
- Milestone 1 recorded as closed with all four success criteria met.
- New action item for the Epic 4 retro: **the requirements-miss class** — an AC that is meetable,
  implemented and unwanted; no review layer can catch it; any fix belongs at story creation and
  requirement authoring, and FR24 is the worked example.

---

## 5. Implementation Handoff

**Scope: MAJOR.** A phase boundary moves, a requirement is withdrawn and the client strategy changes.

| Step | Owner | Deliverable |
| --- | --- | --- |
| 1. Apply edits 4.1–4.9 | Developer (this session, on approval) | Updated PRD, epics, sprint-status |
| 2. **Epic 4 / Milestone 1 retrospective** | `bmad-retrospective` | The requirements-miss lesson, the first fan-out-accounted review cost, R1's 3-convergence result |
| 3. Milestone 2 planning — product first | PM (`bmad-product-brief` / `bmad-prd`) | What the Bevy client is *for*, as outcomes. **FR24's replacement must not name a technique.** |
| 4. Milestone 2 architecture | Architect (`bmad-architecture`) | Client-side design; the AD-1 gate probe sibling; NFR2's Bevy bar; how deterministic evidence works for a real renderer |
| 5. Milestone 2 epics/stories | `bmad-create-epics-and-stories` | Only after 3 and 4 |

**Do NOT** write Bevy stories before steps 3 and 4. The whole point of closing the milestone is to
give the new client a real planning pass instead of an epic bolted onto a TUI plan.

### Success criteria for this change

1. `main` remains 2D-only; 4.1a's branch survives unmerged.
2. No phase-one requirement is left silently unmet — FR24 withdrawn *explicitly*, FR23 closed
   *with its qualification recorded*.
3. The story-count counter-metric holds at 11, cut list un-invoked.
4. Milestone 2 starts from a product statement, not from a rendering technique.
5. The TUI's role as deterministic assertion instrument is written down somewhere durable —
   otherwise it is lost the first time someone asks why the terminal client still exists.

---

## 6. Epic 4 closure record — the measurements Epic 3's retro asked for

Recorded here **in lieu of a full retrospective** (Wolf's call, 2026-08-08: "either have a very
short one or just skip it"). Epic 4 ran one story, so a full retro would re-derive what is already
written down. These four numbers are the ones Epic 3 explicitly committed to measuring, and they
would otherwise dangle unanswered.

**1. R1 — the layer territory split: SUPPORTED, keep it.**
Epic 3 approved R1 (Blind Hunter = `sim-core`, Edge Case Hunter = the shells) while recommending it
be closed unapplied, because Epic 3 could not measure convergence — its layers kept dying. 4.1a is
the first clean measurement. **3 convergences across ~13 findings**, against Epic 3's single clean
story (3.2) at 1-in-8:
- the `Hit.distance` comment defect — Blind Hunter **and** Acceptance Auditor, independently;
- the stale `NO_COLOR` warning — Acceptance **and** Feature Auditor;
- the unpinned fourth band glyph — Acceptance **and** Feature Auditor.
**The revert rule did not fire:** it says revert R1 if a defect is later found whose site sat inside
a hunter's excluded territory. No such defect was found. *Caveat kept honest:* 4.1a's diff was
client-only, so `sim-core`'s owner had an empty territory and was reassigned to `raycast.rs`. R1 was
therefore exercised in spirit, not literally, and one story is not a trend.

**2. Layer reliability — FIXED. First clean four-layer run since story 3.2.**
**4 of 4 layers completed, zero kills, zero coverage holes.** Epic 3 lost three of four layers at
both 3.1 and 3.3. The two fixes shipped together as required — the **silence-based time-box** (P1)
and **per-layer `CARGO_TARGET_DIR`** (P2) — and nothing starved: four layers each ran `cargo` builds
and live daemons concurrently without contending. **This is the coupling rule vindicated**: the
time-box was never the whole fix, the build isolation was.

**3. Review cost — the first honest figure this project has ever recorded.**
**$28.24 / 306 turns / 70 min**, of which **4 subagent transcripts = 15.2M tokens = 56.5% of the
session.** That last number is the point: before Epic 3's item T1 landed, *over half of this review
was invisible to the ledger*. **Do not compare this to any pre-2026-08-08 review row** — every one
of them is a known undercount, so the falsified "~$22/story floor" cannot be re-derived from them
either. This is the new baseline; the next review is the first legitimate comparison.

**4. A new spec-defect subclass, and the honest verdict that review cannot catch it.**
See action item **E4-P1**. The tracked class was "AC unmeetable as written" — reliably caught by
review, four times in seven stories. 4.1a exposed the other kind: **meetable, implemented, and not
what the user wanted.** Every layer audits the code against the spec; when the *spec* is the defect,
they are all blind by construction, including the Feature Auditor. The root cause is one level up,
in an FR that named a mechanism ("raycast") instead of an outcome. **Any fix belongs at requirement
authoring, not in the review workflow** — and one candidate the retro should weigh is whether a
visually-subjective story owes a cheap human-checkable artefact (a mock frame, a sketch, a
"here is what you will see" paragraph) *before* a full implementation is built.
