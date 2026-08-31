# Action items — the prose archive

**State lives on GitHub issues, not here.** Open action items are issues on `jeicei75/frostvein`
labelled `action-item`, each carrying a `route:` label naming its vehicle. This file is the
**reasoning** archive — the evidence, measurements, rulings and closure notes that are too long to
live in an issue body and too valuable to delete. When the two disagree: **history wins here,
status wins on the issue.** Never read status from this file.

Moved out of `sprint-status.yaml` on 2026-08-31 (forge-process 1.3.2, Wolf's ruling). The
`action_items:` block's absence from that file is CORRECT and is not a missing field to repair.
Every item below is reproduced verbatim from the block as it stood at commit `00d1eb8`; nothing was
summarised or dropped.

**The twelve items that were open at the move were re-verified against the tree first** — three
were already done and had simply never been struck, two were half-done with stale framing. Only the
live remainder became issues. Those verifications are recorded on each item below.

## Epic 1 — 16 items (0 open at the move)

### Retune code-review: Blind Hunter + Edge Case Hunter on Sonnet; Accepta…

**Action.** Retune code-review: Blind Hunter + Edge Case Hunter on Sonnet; Acceptance Auditor, Feature Auditor and triage stay Opus. Control = mutation kill-rate vs 1.3's baseline; revert Lever A if it drops

**Owner.** Amelia

**Status.** done

**Note.** SETTLED by Wolf at 2.2's review: KEEP the Sonnet hunters, and MANDATE live execution in every layer — model tier is no longer the experiment. Vindicated on the first run: the Sonnet Blind Hunter produced this review's single most valuable finding (two dwarves sharing a tile, the higher Id silently erasing the lower) by writing a scratch binary and sweeping 200 seeds x 300 ticks — exactly the unique finding it failed to produce at 2.1, when it only read the diff. Settled configuration now lives in _bmad/custom/bmad-code-review.toml; do not re-ask

### Scope each review hunter to its own diff and the ACs it owns, not the…

**Action.** Scope each review hunter to its own diff and the ACs it owns, not the full story file + spine + PRD (per-turn context was ~123k)

**Owner.** Amelia

**Status.** done

**Note.** Applied at 2.1's review: hunters got the crates/-only diff, the Acceptance Auditor got an extracted AC+guardrail file instead of story+spine+PRD

### Batch mutation testing into one apply/revert script emitting a results…

**Action.** Batch mutation testing into one apply/revert script emitting a results table, replacing ~3 turns per sabotage

**Owner.** Amelia

**Status.** done

**Note.** scripts/mutate.sh + _bmad-output/implementation-artifacts/mutations/<story>.sh; exits non-zero on a survivor. Caught a surviving mutation in 2.1's own review patches

### Add 'verify by sabotage, paste the red output' as a required task chec…

**Action.** Add 'verify by sabotage, paste the red output' as a required task checkbox for every mapping/pinning test in the dev handoff prompt

**Owner.** Amelia

**Status.** done

### Runbook: if a story spans multiple Codex sessions, restate RED evidenc…

**Action.** Runbook: if a story spans multiple Codex sessions, restate RED evidence in the continuation handoff (1.2 lost it across a session boundary)

**Owner.** Amelia

**Status.** done

### Every story ships an observability instrument named in its tasks (the…

**Action.** Every story ships an observability instrument named in its tasks (the --frame lesson)

**Owner.** Amelia

**Status.** done

**Note.** Closed at 2.2: the story named its instrument in a task of its own (Observability instrument, AC 11/12) rather than acquiring one at review. Sting in the tail — 2.2's review found the instrument itself was broken (`stream_frames` recomputed the camera per frame, pinning the dwarf to screen centre and rendering motion as stillness), so the live evidence taken through it was an artefact. Fixed, regression-tested and re-taken at review. Naming the instrument is necessary but not sufficient: the instrument needs a test too. ENCODED 2026-08-03 in two places so 2.3 onward inherits it: the TESTED-INSTRUMENT ADDENDUM on the observability rule in _bmad/custom/bmad-create-story.toml (which story-creation actually loads) and the Story rules section of docs/technical-preferences.md

### Set sandbox_workspace_write.network_access=true in scripts/codex-hando…

**Action.** Set sandbox_workspace_write.network_access=true in scripts/codex-handoff.sh and update its header comment; keep cargo fetch prewarm and --offline builds in the handoff prompt

**Owner.** Amelia

**Status.** done

### BLOCKING 2.1 — resolve the TUI delta-consumption design (event::poll v…

**Action.** BLOCKING 2.1 — resolve the TUI delta-consumption design (event::poll vs reader thread) during 2.1 story-creation; main.rs:121 blocks on event::read()

**Owner.** Winston

**Status.** done

### BLOCKING 2.1 — specify the daemon's client registry and per-tick broad…

**Action.** BLOCKING 2.1 — specify the daemon's client registry and per-tick broadcast in 2.1's story; 2.3's two-client AC depends on it

**Owner.** Winston

**Status.** done

### Make closing the 'tick: 0' mutation an explicit 2.1 task — Epic 1's on…

**Action.** Make closing the 'tick: 0' mutation an explicit 2.1 task — Epic 1's one irreducible surviving mutation

**Owner.** Amelia

**Status.** done

### Fold the 1.2 half-close / dead-peer reclaim deferral into 2.1's scope…

**Action.** Fold the 1.2 half-close / dead-peer reclaim deferral into 2.1's scope (the // NOTE: already hands it to 2.1)

**Owner.** Amelia

**Status.** done

### Carry the RNG-stream-coupling and rand_chacha serde-feature debts into…

**Action.** Carry the RNG-stream-coupling and rand_chacha serde-feature debts into 2.4's story notes

**Owner.** Amelia

**Status.** done

**Note.** Closed at 2.4 story creation (2026-08-04), both re-verified against source rather than inherited. RNG-stream coupling: already RESOLVED at 2.2 — STREAM_WORLDGEN/STREAM_SPAWN/STREAM_WANDER exist at crates/sim-core/src/lib.rs:17-19 and only the wander stream is retained on World, so 2.4 serializes one stream and says so. rand_chacha serde feature: still open and now scheduled — the workspace pins `rand_chacha = "0.10.0"` with no features, the feature is optional (rand_chacha-0.10.0/Cargo.toml:53), and ChaCha8Rng only derives Serialize/Deserialize with it; 2.4's first task turns it on

### Fix AC11's unreachable 100x40 fallback spec text at the next TUI story…

**Action.** Fix AC11's unreachable 100x40 fallback spec text at the next TUI story

**Owner.** Amelia

**Status.** done

**Note.** Closed at 2.2 story creation. 1.3's AC11 now describes what frame_size actually does (renders at the reported size; 100x40 only on error or a zero dimension) and the deferred-work entry is marked resolved. Spec text only — no code changed

### Create frostvein/AGENTS.md with the project's durable rules (mirroring…

**Action.** Create frostvein/AGENTS.md with the project's durable rules (mirroring CLAUDE.md) plus standing dev-agent rules. Codex reads only the forge's generic /workspace/AGENTS.md today, so project rules never survive a session boundary

**Owner.** Amelia

**Status.** done

### Add scripts/gate.sh (fmt, clippy, test, tui dependency-edge probe) exi…

**Action.** Add scripts/gate.sh (fmt, clippy, test, tui dependency-edge probe) exiting non-zero, wired to a pre-commit hook, so a green gate cannot be merely claimed

**Owner.** Amelia

**Status.** done

### Add 'codex review --base main' as a pre-handback self-gate, instructed…

**Action.** Add 'codex review --base main' as a pre-handback self-gate, instructed at self-referential tests and unbounded I/O. Measure: 2.1's Opus patch count below 1.3's ten

**Owner.** Amelia

**Status.** done

**Note.** VERIFIED at 2.2: the self-gate ran for the first time — the writable-CODEX_HOME fix in codex-handoff.sh works. It returned 'No actionable defects were found' on the ~1200-line diff and requested no code changes. Cost: ~15 min of wall-clock after the last commit, producing no commits, and it re-printed the full branch diff each poll turn (185k-line run log). Its nested sandbox denied loopback, so it skipped two simd tests as environmental. MEASURE MET: 2.2's review applied 7 patches against 1.3's ten — but read it carefully, because the self-gate found NOTHING and all 7 came from the review layers, so the drop is not evidence the self-gate works. Its true value is still unmeasured; the honest read is that ~15 min of wall-clock bought a clean bill of health that four later layers contradicted

## Epic 2 — 13 items (0 open at the move)

### Encode a hard per-layer review time-box in bmad-code-review.toml: 20 m…

**Action.** Encode a hard per-layer review time-box in bmad-code-review.toml: 20 min/layer, orchestrator kills and continues, a timed-out layer is reported as a COVERAGE HOLE never a clean result, a growing transcript is explicitly not a progress signal, 60 min whole-review ceiling

**Owner.** Amelia

**Status.** done

**Epic 3 outcome.** READ THIS BEFORE TRUSTING THE `done` ABOVE. The rule was encoded and the rule was WRONG, and this row said `done` for the whole of Epic 3 while it cost coverage on two of three stories. It kills on wall-clock-since-launch; the four layers run concurrently, each is MANDATED to run cargo test, and they starve on one target/ lock -- a lock-blocked layer emits nothing, which the rule cannot distinguish from a hang. 3.1: three of four layers killed with ZERO findings between them, and the epic's most severe defect (a whole-world designate taking a delta from 378 bytes to 16,761,209 bytes at 34.7 MB/s, world simultaneously unsaveable) was found by the orchestrator INLINE after all three had failed; one of the killed layers was literally reporting the cause (`sequential due to target lock`). 3.3: three killed in round one, and the Edge Case Hunter died silent AGAIN on the re-run, so that story's loader/boundary territory is UNREVIEWED by any layer. 3.2, the one story where all four completed, is also the story where review found the most. The Epic 2 retro DIAGNOSED this exactly -- measure silence-since-last-named-step, stop combining mandatory live execution with concurrent layers sharing a build lock -- and wrote it into no config file, which is the identical meta-failure this item was created to fix. THE FIX IS NOW Epic 3 items P1 (silence-based detector) and P2 (per-layer CARGO_TARGET_DIR). STANDING LESSON, now a team agreement: `encoded` and `correct` are different claims and a follow-through table cannot tell them apart

**Note.** Applied 2026-08-05 as the LAYER TIME-BOX persistent fact. Origin: story 2.4, where a hunter layer hung ~2.5h (43% of that story's wall-clock) and was only caught when Wolf returned. Verified at the retro that the in-session fixes had never been written to any config file, so Epic 3 would have inherited the trap unchanged

### Add tokens and wall-clock to session_tokens.py alongside cost: ledger…

**Action.** Add tokens and wall-clock to session_tokens.py alongside cost: ledger `minutes` column, cursor last_ts so each phase's duration is billed over its own delta window, and a rollup Spend-shape table (turns, tokens, cache-read %, output, minutes)

**Owner.** Amelia

**Status.** done

**Note.** Applied 2026-08-05. `minutes` is appended LAST so the 20 historical rows still parse (they read as blank rather than being re-aligned by a column). Also fixed a parsing defect found while doing it: annotation tables inside a ledger were parsed as data rows, inventing phantom rollup phases (`row`, `dev (codex, gpt-5.6-sol)`). First finding off the new table: 96% of every token processed is a cache read, in BOTH epics

### Apply the review restructure: R1 give each hunter a disjoint territory…

**Action.** Apply the review restructure: R1 give each hunter a disjoint territory (Blind Hunter = sim-core, Edge Case Hunter = the shells) while both Opus auditors keep whole-diff scope; R2 one verification pass per review instead of one per patch

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 at the Epic 3 retrospective -- DECIDED, both halves, and the decision is not the one the evidence pointed at. BOTH TAKEN by Wolf, and they are now Epic 3 items P3 and P4; this item closes as decided, they close as encoded. READ THE EVIDENCE HONESTLY BEFORE RE-DERIVING ANYTHING: R1's justification was convergence -- three layers independently finding the same defect at 2.3 and 2.4 -- and Epic 3 was supposed to MEASURE it. It could not, because Pattern 1 killed the layers that would have converged. The ONE clean four-layer story (3.2) produced 8 findings with exactly ONE convergence (AC10, acceptance + feature). 1-in-8, not 3-in-3. I recommended closing R1 UNAPPLIED on that basis; Wolf took it anyway, and the package is coherent because he also took A and B: with the coverage holes closed, R1 partitions WORKING coverage instead of thinning broken coverage. That reasoning is load-bearing -- R1 without A+B would have been a mistake. Epic 2's revert rule is carried verbatim into P3: revert R1 (keep R2) if a defect is later found whose site sat inside a hunter's excluded territory; control = mutation kill-rate against 1.3's baseline. R2's accepted risk, stated so it is not rediscovered as a surprise: a later patch can break an earlier verified one, caught only by the final gate and the full mutation run -- both mandatory, both green on all three Epic 3 stories

**Note.** Proposed at the retro on Wolf's 'we need to try something different'; awaiting his approval of the shape before encoding. Diagnosis: review costs turns x context x rate, is 96% re-read context, and no lever so far touched turns or orchestrator context. Evidence for R1 is convergence -- the stale-speed trap (2.3) and duplicate dwarf ids (2.4) were each found independently by THREE layers. Control = mutation kill-rate against 1.3's baseline; revert R1 (keep R2) if a defect is later found whose site sat inside a hunter's excluded territory. Measure = turns and cache-read tokens per review against Epic 2's 537 turns / 72.7M tokens per story. NOTE the humility clause: Epic 1's retro predicted review $193 -> $39 and delivered a small INCREASE, so this is measured on turns, not promised in dollars

### Record which review layer raised each finding, in every triage…

**Action.** Record which review layer raised each finding, in every triage

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-06 -- encoded, not just agreed. It arrived from the FORGE rather than being written here: the rule already existed upstream as ep-11 retro A3 and had been sitting untaken in bmad-code-review.toml since 2026-08-03, which is exactly the drift forge-process exists to surface. Merged into _bmad/custom/bmad-code-review.toml as the REVIEW-COST DISCIPLINE fact, rule (1): every finding carries its originating LAYER and a SEVERITY, and a TIMED-OUT layer is recorded as a coverage hole with zero findings rather than as a clean result. Rules (2) cap-the-LOW-tail and (3) patch-in-a-fresh-session were taken at the same time on Wolf's yes. R1's territory split is still OPEN and still needs his approval -- this only makes it measurable

**Note.** Cheap, and it is what R1 rests on -- convergence was only visible this epic because three layers happened to be named in two story files. Makes it measured rather than inferred at the Epic 3 retro

### Add an AC-authoring check to story-creation: every AC must name an obs…

**Action.** Add an AC-authoring check to story-creation: every AC must name an observation that can actually occur, and must not restate an earlier story's contract inaccurately

**Owner.** Amelia

**Status.** done

**Baseline.** Spec-text defects are now their own recurring class with no mechanical guard: 1.3's unreachable 100x40 fallback, 2.1's AC2 amended at review, and 2.3 shipping TWO unmeetable ACs (AC9 demanded a speed change in the 'very next delta' when the command crosses TCP and lands on the second; AC2 contradicted itself about oversized lines). Four instances in seven stories, all authored in create-story, all caught by review. Baseline for Epic 3: zero

**Note.** Applied 2026-08-05 at 3.1's story creation, on Wolf's yes. Encoded as a persistent fact in _bmad/custom/bmad-create-story.toml, merging this item with the forge's ep-11 A6 rule that frostvein had never taken (forge-process check reported bmad-create-story.toml UPSTREAM CHANGED). Two checks per AC: (1) CAN IT HAPPEN -- trace the observation through the real code path before writing it; (2) OUTCOME NOT MECHANISM, with the named legitimate exceptions (determinism invariants, byte-exact wire contracts, architectural edges, reuse-don't-rebuild). It fired on its first use: 3.1's draft AC4 said a designate command sent while paused appears in 'the very next delta' -- the identical defect to 2.3's AC9, since the command crosses TCP and lands on a later iteration. Rewritten to 'within a bounded number of deltas, every one carrying the frozen tick' before the story was saved. The forge's MODEL POLICY fact was deliberately NOT taken (frostvein has no Fable history); that side of the reconciliation is still open

### RESOLVED (Wolf, 2026-08-05) -- the AD-10 command consumer is OPTION C:…

**Action.** RESOLVED (Wolf, 2026-08-05) -- the AD-10 command consumer is OPTION C: a plain `&mut self` method on World, called by simd at iteration top BEFORE the conditional world.step(). Carry it into 3.1 story-creation

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-06 -- verified in the shipped story, not assumed. 3.1 carries Option C as its core design (Key decisions: 'Option C is Wolf's ruling, not one of several readings'), the consumer is a plain method on World [3-1-give-the-order.md:379] called by simd at loop-iteration start with the pause NOTE rewritten [main.rs task, line 145: 'pause never blocks command intake -- that placement is the AD-2/AD-10 contract, not a detail'], and the wire change landed in one commit as required. The two things the item demanded 3.1 SETTLE were both settled: the still-applies-while-paused line, and the wire change. NOTE for 3.2: 3.1 found AD-10's prose UNDER-LISTS its commands -- `remove_stockpile` is a fourth world-mutating command added on Wolf's call, so the spine's command table and docs/architecture.md disagree with the code until someone reconciles them

**Note.** THE PROBLEM: simd pauses with `if speed != Speed::Paused { world.step(); }` at crates/simd/src/main.rs:181-183, which skips the ENTIRE schedule (crates/sim-core/src/lib.rs:239). That satisfies AD-2 only by accident today, because every system in the schedule happens to be world-advancing. 3.1 adds the AD-10 queue whose commands MUST apply while paused, or the player pauses, marks a rectangle and sees nothing until unpause. WHY C over the alternatives: (A) a second always-run schedule breaks AD-7's single chained schedule and doubles what 2.4's single assemble() must keep in step -- the exact divergence it was built to prevent; (B) bevy run-conditions on a Paused resource make sim-core learn pause, contradicting 2.3's no-sim-core-change guardrail. C matches the existing precedent: World::set_tile at lib.rs:375 already mutates sim state as a plain method, not a system. One schedule, one assemble, pause stays in simd, and AD-10's `consumed at loop-iteration start` is literally the call site. C's COST, which needs a // NOTE: in 3.1 -- command ordering is explicit by call-site rather than by .chain(). TWO THINGS 3.1 MUST SETTLE ALONGSIDE IT: (1) where the still-applies-while-paused line falls -- a designation appears in the next delta while paused YES, but does designation-to-job conversion run while paused, does the reaction-delay counter tick? Those are world-advancing and should skip; 3.2 adds both, and if 3.1 does not draw the line 3.2 draws it by accident. (2) 3.1 IS A WIRE CHANGE and its own epic text does not say so -- designations and zones are still Vec<()> at crates/protocol/src/lib.rs:100-101 and 115-116, so protocol + the simd bridge + the pinned JSON literals + tui move in ONE commit or the suite is red between them, exactly as 2.2 had to for Entity.state

### Story 3.2 stays ONE story (Wolf, 2026-08-05, declining the proposed 3.…

**Action.** Story 3.2 stays ONE story (Wolf, 2026-08-05, declining the proposed 3.2a/3.2b split). Fold the size mitigations into its story-creation and handoff instead

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 at the Epic 3 retrospective. 3.2 SHIPPED as one story at 17 ACs, 76 mutations all killed, gate green -- the one-story ruling held and was not wrong. But record the full price, because it is what reversed the precedent: 3.2 cost $132.98, was the epic's most expensive story, and EXHAUSTED A FULL WEEK OF CODEX QUOTA in ~3h10m (~51-60pp), which forced story 3.3 onto Claude at $57.78 -- roughly double any Codex dev row. Mitigations 2, 3 and 4 (restate RED evidence across a session boundary; commit per green step; cargo clean -p before the final gate) all applied and all earned their keep. Mitigation 1, the review-side layer time-box, was the one that failed -- see Pattern 1 -- and its real fix is now Epic 3 items P1/P2. THE PRECEDENT IS DELIBERATELY REVERSED AT EPIC 4: Wolf split 4.1 into 4.1a/4.1b on this evidence. Do not read that as overturning this ruling; read it as the same judgement applied to a story that is LARGER than 3.2 was

**Progress.** 2026-08-06 -- the STORY-CREATION half is done; the HANDOFF half is still owed. Mitigations 2, 3 and 4 are written into 3.2's tasks and Dev Notes (restate RED evidence across a session boundary; commit at minimum once per completed task as the recovery mechanism, now also a hard floor in bmad-dev-story.toml; the `cargo clean -p` step before the final gate). Mitigation 1 (the layer time-box) is review-side and already encoded, though 3.1 proved it misfires on target/ lock contention -- that fix is still owed and is one of the two forge-propagation blockers. Mitigation 5 (more ACs, more chances of an unmeetable one) was applied as a checklist pass over every AC, which found and fixed six defects in the draft before it was saved: a cap that used `break` and so could not meet its own overwrite clause; a save point pinned to a magic tick against a 5-30 tick reaction delay; an A* node cap set below a legitimate cross-map path; and -- the one that matters -- an instrument asserting a wall-to-floor glyph change that CANNOT be observed, because render peeks below an Empty tile and redraws the same glyph dimmed, which NO_COLOR then strips. That is the 2.2 false-evidence failure exactly, caught in the spec this time rather than at review. FINAL SHAPE: 17 ACs. Wolf was told the count before the story was written and held the one-story ruling

**Note.** Risk accepted deliberately, not overlooked. 3.2 carries FR5, FR6, FR8, FR11, FR12 plus the AD-12 job market, the SaveState extension, the first-ever production set_tile caller, and the dwarf-collision fix deferred from 2.2. Calibration: 2.4 was the largest story so far (12 ACs, 30 mutations, $83, ~6h) and is the one that HUNG. MITIGATIONS, all belonging to story-creation and the handoff prompt: (1) the review half is already protected by the new layer time-box, which is what actually failed at 2.4; (2) a story this size may span two Codex sessions, so the standing rule -- restate RED evidence in the continuation handoff -- must be explicit in the prompt, since Epic 1 Pattern 4 was TDD discipline lost at exactly that boundary; (3) commit-per-green-step stops being style and becomes the recovery mechanism, letting a stalled run resume from the last green commit; (4) budget a cargo clean -p step for a 40+ mutation set, since mutate.sh is not concurrency-safe and both 2.3 and 2.4 hit stale mutated binaries; (5) more ACs means more chances of an unmeetable one -- 2.3 shipped two on a story a third this size

### Author an explicit dwarf-occupancy / tile-reservation AC into the dig…

**Action.** Author an explicit dwarf-occupancy / tile-reservation AC into the dig story

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-06 at 3.2 story creation. Authored as 3.2 AC14, but deliberately NOT as tile reservation -- Wolf's ruling when the two shapes were put to him: the reproduced defect is that a dwarf silently VANISHES FROM VIEW, which is a rendering fault, so the fix is a distinct crowd glyph drawn where two or more dwarves share a screen cell, and dwarves may still share a tile. Rejected: filtering A* on occupancy, which buys physical correctness at the price of a deadlock class (two dwarves each holding the tile the other needs, deterministic, so the harness would faithfully reproduce the hang) in a story already at capacity. 3.2's scope guardrails name the rejection explicitly so a reviewer does not read the absence of reservation as an oversight

**Note.** Deferred to 3.2 by Wolf at 2.2's review, but epics.md's 3.2 text says nothing about occupancy, so today no AC anywhere covers it. Reproduced at 2.2: seed 133, ids 0 and 1 share (62,36,17) for ticks 133-142; render writes entities in ascending Id so the second erases the first and a dwarf silently vanishes for ten ticks. With A* and jobs, dwarves CONVERGE on dig sites, so the failure gets far likelier

### Schedule the three 3.1-owned deferred items into 3.1's story rather th…

**Action.** Schedule the three 3.1-owned deferred items into 3.1's story rather than re-deferring them

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-06 -- all three landed as ACs in 3.1, verified against the story file rather than inferred from the item being old. (1) Stale-speed = AC12: next speed computed from client-side state, so `+` then `-` inside one round-trip emits Fast then Normal, never Paused. (2) read_inbound = AC13: the overflow log fires only when the line actually reached MAX_LINE_BYTES -- and its mutation (`read_inbound calls every partial line an overflow`) was KILLED, so the fix is tested, not merely present. (3) Command::Quit = AC10, resolved as TEXT not a key: the hint bar names `q quit client` and the scope guardrail keeps 2.4's AC9 standing (a shared daemon must not die from one viewer's keypress). Read that third one carefully -- the affordance gap was closed by making the UI honest about the absence, deliberately, NOT by adding the command

**Note.** The stale-speed compose trap (crates/tui/src/view.rs:180-195), read_inbound's partial-line-as-overflow misreport (crates/simd/src/main.rs:270), and the missing in-UI affordance for Command::Quit (crates/tui/src/view.rs:222)

### Re-check FR23's motion sign-off at Story 3.3, with the full designate…

**Action.** Re-check FR23's motion sign-off at Story 3.3, with the full designate -> dig -> haul loop visible

**Owner.** Wolf

**Status.** done

**Note.** FR23's motion half did NOT close at Epic 2. Wolf's verdict: happy in the early stories, then 'the world didn't change after that, so it was a bit boring'. Predicted verbatim by 2.2's review and deferred: at 80x24 only one of five dwarves is ever visible, Walk is a one-tick pulse in eleven, and the dirty-tile path is inert so every Epic 2 delta carried tiles=[]. Wolf's call at this retro: do NOT spend a TUI story on it -- Epic 3 fixes it by construction (3.1's cursor puts the camera where the work is, 3.2's dig is the first thing that visibly changes the world). Stays OPEN until 3.3. CLOSED at 3.3 (2026-08-07): the check happened and Wolf watched the full loop. Verdict, verbatim: "looks ok for 2d tui game atm ... not sure how much more visually pleased it could be without designing own font or something" and "most likely we need to get to the 3d first to say". So the FEEL FLOOR (NFR2) is met and the boring-world complaint is answered -- the world now visibly changes because the player made it change. But the FR23 icy-grim-identity-in-motion verdict is only PROVISIONAL at 2D: Wolf judges the glyph-and-truecolor client close to its ceiling without a custom font, and defers the real judgement to the depth view (epic 4, 4-1-behold-the-fortress-in-depth). Do not read this item as FR23 fully signed off; read it as signed off for the 2D client with the identity question re-opened at epic 4

### Hand the Epic 2 process changes to Nidavellir via a docs/forge-transfe…

**Action.** Hand the Epic 2 process changes to Nidavellir via a docs/forge-transfer-*.md note

**Owner.** Amelia

**Status.** done

**Note.** docs/forge-transfer-2026-08-05.md, following the 2026-08-03 convention: read-only, split by evidence class, every claim naming the file or command that verifies it. NOTE this is the NOTE only -- the actual forge-process propagation is deliberately held until after story 3.1, see the next item

### AFTER STORY 3.1 IS DONE: propagate the Epic 2 process changes into the…

**Action.** AFTER STORY 3.1 IS DONE: propagate the Epic 2 process changes into the forge via forge-process, then re-install into every consumer

**Owner.** Wolf + Amelia

**Status.** done

**Closed.** 2026-08-08 -- COMPLETE, both directions. The forge merged frostvein's porting spec and released forge-process 1.1.0; frostvein then pulled it and is `in sync` (verified: both FILE entries ok, all four TEMPLATEs adapted). NOTHING WAS DROPPED IN EITHER DIRECTION, checked symbol by symbol before overwriting our copies rather than taken on trust: nested_codex_rollouts, _rollout_meta, sum_codex_session, _span, _parse_ts, _minutes_between, _fmt_pp, _SENTINEL, _merge_preserved, _render_shape, quota_pp, counted_transcripts and --no-nested are all present upstream, and our both-tools extension of the cursor rebase survived (`primary_only` now covers codex as well as claude). WE TOOK BACK MORE THAN WE SENT: the forge's REVIEWS ARE READ-ONLY rule, its better-organised LAYER TIME-BOX, its REVIEW RUNS IN A FRESH CONTEXT fact (and we deleted our duplicate so the two cannot drift), and the ep-06 A2 preflight principle that closed P6. FOUR API BREAKS resolved by the pull alone -- _as_summary's keyword-only span/quota, sum_claude_transcript back to one file, sum_codex_session returning a triple, merge_summaries now private -- because frostvein has NO local python importers, only CLI call sites in the toml files. NOT TAKEN UPSTREAM, deliberately and recorded there: per-layer build isolation (Asgard's layers are read-only and run no build, so the cause does not exist; the coupling rule is recorded so it becomes mandatory the day a layer there compiles) and layer territories (their convergence data reads both ways). Return note: docs/forge-return-2026-08-08.md

**Blocked on.** Two defects story 3.1 exposed, both must be fixed in frostvein before anything ships to the forge: (1) the layer time-box misfires on target/ lock contention; (2) session_tokens.py does not count subagent transcripts. Detail in the note. (Status was `blocked` until 2026-08-06 -- not a valid action-item status, so it tripped sprint-status validation; the block itself is real and lives here instead.) UNBLOCKED 2026-08-08, LATER THE SAME DAY -- both blockers are now FIXED here, which is what this item was waiting for. TRANSFER NOTE WRITTEN: docs/forge-transfer-2026-08-08.md, and a function-by-function PORTING SPEC: docs/forge-porting-spec-2026-08-08.md. OWNERSHIP SETTLED (Wolf, 2026-08-08): the FORGE SESSION does the merge -- frostvein does NOT write into /workspace. Frostvein's side is committed on branch epic-3-retrospective. THE MERGE IS ADDITIVE, forge as base: it already has _price_bucket/_BUCKETS, by_model, sum_claude_session and the cursor rebase, and none of that is reverted. What goes UP is what the forge lacks -- nested codex rollouts (a DIFFERENT mechanism from its own sub-agent fix, so fixing one did not fix the other), minutes, quota_pp, rollup preservation, --no-nested, plus three net-new review rules (silence-based time-box, build isolation, territories) since the forge's bmad-code-review.toml has none of them. The one structural merge point is _as_summary: union the keys, both suites assert the exact key set so a miss fails loudly. The one real DECISION is the API shape -- forge's sum_claude_session 3-tuple vs frostvein's include_subagents= flag; recommendation is keep the forge's and add --no-nested, with nesting ON by default. AND THE SHAPE OF THE PROPAGATION CHANGED -- it is a TWO-WAY MERGE, not a push. `forge-process.sh check` now reports UPSTREAM CHANGED on BOTH shared script files: the forge independently implemented the same Claude sub-agent fix, down to the function name `subagent_transcripts` (same glob, same answer, arrived at twice). Frostvein-only: nested Codex rollouts, minutes, quota_pp, rollup preservation. Forge-only: _price_bucket/_BUCKETS, _CURSOR_SCHEMA, sum_claude_session's 3-tuple shape. PRICES rows are identical, verified. A `cp` in EITHER direction destroys real work -- reconcile the two, do not ack a FILE. FROSTVEIN ALREADY TOOK THE FORGE'S BETTER IDEA: its cursor-rebase (schema 2) caught a trap T1's first implementation had missed -- a pre-fix cursor diffed against a fan-out-inclusive cumulative dumps every historical sub-agent token into the next row, and frostvein had 42 such cursors. Adopted with identical semantics so the two stay mergeable. THE RUNBOOK'S KNOWN LIMITATION HAS NOW FIRED: the unit that wants sharing is the RULE not the FILE, and the time-box is hand-merge 2 OF 2 against its own stated trigger. SEQUENCING HAZARD: the forge session was LIVE when this was written (session_tokens.py mtime 10:02, note written 10:10), so quiesce or coordinate with it before touching _bmad/scripts/, and re-diff rather than trusting the snapshot in the note. Blocker (1), the layer time-box misfiring on target/ lock contention: closed by items P1 (kill on silence-since-last-named-step, not wall-clock-since-launch) and P2 (per-layer CARGO_TARGET_DIR, which removes the contention rather than detecting it better). Blocker (2), session_tokens.py not counting fan-out: closed by item T1, and verified against an independent oracle -- the tool now reproduces story 3.2's hand-built self-gate table to the token (218 turns / 20,107,290 tokens / $18.29 vs $18.28 hand-derived) having found the rollouts itself by cwd + window overlap. WHAT TO SHIP, and note the first entry CHANGED while this was blocked: (1) session_tokens.py + its tests -- frostvein is still the side that is AHEAD and now further ahead (minutes/cursor work PLUS the fan-out walk); reconcile the FILE, do not ack it. (2) The LAYER TIME-BOX rule -- ship the REWRITTEN silence-based version paired with BUILD ISOLATION, NOT the 2026-08-05 wall-clock version, which Epic 3 proved wrong; docs/forge-transfer-2026-08-05.md now carries a SUPERSEDED banner over that section so the old prescription cannot be shipped by accident. The rule is universal (it is about fan-out orchestration, nothing to do with Rust), so hand-merging it is still MERGE 1 OF 2 against the runbook's known limitation. (3) The forge's bmad-dev-story.toml codex-metric defect, already closed upstream. REMAINING SEQUENCE unchanged and still verified: copy both script files into /workspace/_bmad/scripts/, bump VERSION 1.0.2 -> 1.1.0 in /workspace/forge-process.manifest, re-run `./scripts/forge-process.sh check projects/frostvein`, install into every other consumer, add a History line to /workspace/docs/forge-process-upgrade-runbook.md. ALSO STILL PENDING: check reports UPSTREAM CHANGED on all three _bmad/custom/*.toml files. ORIGINAL BLOCK RECORD FOLLOWS. STILL BLOCKED AT THE EPIC 3 RETROSPECTIVE, 2026-08-08, and both blockers were VERIFIED UNFIXED that day by reading the files rather than inferred from this row: bmad-code-review.toml still says `HARD wall-clock budget of 20 MINUTES` with no silence rule, and session_tokens.py still has no subagent or sibling-rollout walk. Blocker (1) is now SCHEDULED as Epic 3 items P1 + P2 (Wolf approved both). Blocker (2) is now Epic 3 item T1 and is UNTOUCHED -- and it got worse: Epic 3 confirmed the same defect on the DEV side by a second mechanism (nested `codex exec` self-gate rollouts, $18.28 unrecorded on story 3.2 alone), so the fix must walk sibling rollouts AND Claude's subagents/ directory, not just one. Consequence to state plainly: every cost figure in the Epic 3 retrospective is a known undercount, so no cost conclusion is safe until T1 lands

**Outbound queue.** What frostvein owes the FORGE when the two blockers below clear, so it is not rediscovered: (1) session_tokens.py + its tests -- frostvein is AHEAD (the minutes/cursor work), and `check` correctly still reports DIFFERS on both; do NOT ack a FILE, reconcile it. (2) The LAYER TIME-BOX rule, with the contention fix folded in. (3) RAISED AND ALREADY CLOSED THE SAME DAY, 2026-08-06 -- the forge's bmad-dev-story.toml on_complete told agents to SKIP the dev metric because 'Codex records it in its own session'. It does not and cannot. Wolf verified it independently in the forge and it had ALREADY COST A ROW: ep-06-us-04 finished dev, review and review-patch with no codex-dev line at all, silently omitting what the implementation cost. He fixed the forge text and backfilled the row ($9.16 / 105 turns / gpt-5.6-sol). NOTE FOR ANYONE RE-CHECKING THIS: I briefly retracted the finding after seeing that backfilled row and reading it as proof the forge was fine -- the row existed only because the fix had just landed. Do not re-derive the retraction; the defect was real. It then came BACK as an inbound merge, see below. (4) THE bmad-help PYTHON PIN, added 2026-08-09 and NOT yet shipped. .claude/skills/bmad-help/SKILL.md line 26 ships `uv run --python 3.11 _bmad/scripts/resolve_config.py`. frostvein has no pyproject.toml of its own, so uv walks UP the tree and adopts the FORGE's /workspace/pyproject.toml (nidavellir, requires-python >=3.14) -- the pin and the discovered project contradict each other and the command fails outright, every time. Fixed here to --python 3.14 and verified live (resolves 3.14.5 from /workspace/.venv, script runs clean). THIS IS PROPAGATION WORK RATHER THAN A LOCAL EDIT FOR TWO REASONS, both of which mean the local fix cannot be trusted to survive: .claude/skills/ is GITIGNORED here, so the change is unversioned and dies silently at the next BMad install with no diff, no PR and no forge-process notice; and the pin comes from upstream BMad, so every sibling project installed from it carries the same broken command. WORTH KNOWING BEFORE ANYONE RE-DERIVES THIS: it was the ONLY --python pin in the entire install -- the other 22 `uv run` call sites across the skills are bare and resolve correctly through the same forge venv, so DROPPING the pin is the equally-valid fix; Wolf chose the explicit 3.14 so it self-documents and matches the forge's requires-python. The general hazard this exposes is worth carrying separately: frostvein owns its process but lives INSIDE the forge directory, and any tool that does upward discovery (uv here; equally cargo, npm, git) will reach across that boundary and pick up nidavellir's config.

**Note.** DEFERRED BY WOLF 2026-08-05, AFTER story 3.1 exercised both changes. Do NOT propagate yet — 3.1 proved both artefacts DEFECTIVE, and shipping them to every sibling project would spread the defects. TWO BLOCKERS, both must be fixed here first. (1) THE LAYER TIME-BOX MISFIRES ON CONTENTION. It fired five times at 3.1 and was WRONG twice: all four layers were launched concurrently, each mandated to run cargo test across four crates, so they starved on the single shared target/ lock — and a lock-blocked layer emits nothing, which the rule cannot distinguish from a hang. Two working hunters were killed mid-analysis (one was literally diagnosing the contention: `sequential due to target lock`). The other two kills were CORRECT — 17 and 24 minutes of silence with no contention, one running entirely alone — so the rule does catch real hangs. FIX: measure silence-since-last-named-step rather than wall-clock-since-launch, and stop combining mandatory live execution with concurrent layers that share one build lock. (2) session_tokens.py DOES NOT COUNT SUBAGENT TRANSCRIPTS. Review layers write their own agent-*.jsonl under the session subagents/ dir, so the ledger row misses them entirely: 3.1 recorded review=$39.18 / 319 turns / 57.8M tokens while the five layers burned a further ~14.1M tokens and 193 turns unrecorded. Combined ~71.9M is essentially Epic 2 baseline (72.7M) — so the apparent saving is an artefact of the gap, and any cost conclusion drawn from the current ledger is wrong. This matters doubly because three of those five layers produced NOTHING, so a real fraction of review spend bought coverage holes that the ledger cannot show. ONCE BOTH ARE FIXED, the propagation sequence below is unchanged and still verified. ORIGINAL NOTE FOLLOWS. TRIGGER: story 3.1 complete (dev + review). Wolf, 2026-08-05: 'I want to test at least the first story before moving to forge-process' -- the time-box and the new metrics columns should prove themselves on a real story first. WHY THIS IS EASY TO FORGET AND HOW IT FIRES ANYWAY: a STEP C was added to on_complete in _bmad/custom/bmad-code-review.toml that surfaces this at the END of 3.1's review, and instructs its own removal once done. THE EXACT STEPS, verified live at the retro by running `./scripts/forge-process.sh check projects/frostvein` from /workspace: (1) copy _bmad/scripts/session_tokens.py and _bmad/scripts/tests/test_session_tokens.py back into the FORGE at /workspace/_bmad/scripts/ -- both are manifest FILE entries and must be byte-identical, and frostvein is currently the side that is ahead (verified: the ENTIRE diff between the two copies is this retro's work, no pre-existing divergence, so the old PRICES defect really was closed); (2) bump VERSION=1.0.2 to 1.1.0 in /workspace/forge-process.manifest -- new capability, not a fix; (3) re-run check, those two entries should go ok; (4) `scripts/forge-process.sh install projects/<name>` for every other consumer; (5) add a History line to /workspace/docs/forge-process-upgrade-runbook.md. SEPARATE AND HARDER: the LAYER TIME-BOX rule lives in bmad-code-review.toml, a TEMPLATE, so `check` can only ever say `upstream moved, go look`. The runbook's own Known-limitation section already says the unit that wants sharing is the RULE not the FILE, and names the trigger: revisit at a third project, or the first time a shared rule is hand-merged TWICE. The time-box is a universal rule (it is about fan-out orchestration, nothing to do with Rust), so hand-merging it is MERGE 1 OF 2 -- Wolf is on the clock for a decision he already recorded. ALSO PENDING, pre-existing and unrelated to this retro: check reports UPSTREAM CHANGED on all three _bmad/custom/*.toml files -- the forge has improvements frostvein never took, and create-story + dev-story shape how 3.1 and 3.2 get written, so clear them BEFORE 3.1 rather than after

### Wire the metrics-ledger tests into scripts/gate.sh as a fifth check…

**Action.** Wire the metrics-ledger tests into scripts/gate.sh as a fifth check

**Owner.** Amelia

**Status.** done

**Note.** Applied 2026-08-05 on Wolf's yes. `run "metrics ledger tests" python3 -m unittest discover -s _bmad/scripts/tests` -- stdlib unittest deliberately, no pytest and no venv, so the pre-commit hook cannot break on a missing dev dependency. WHY: that suite existed, was thorough, and NOTHING ran it; its own docstring records that it went red after the 2026-08-01 PRICES fix and stayed red unnoticed. Every cost conclusion in the retro comes out of that script and it is a forge-process FILE, so a defect in it propagates to every sibling project. SABOTAGE-VERIFIED before being believed, per the standing rule that a green check is not evidence: (1) PRICES reverted to the retired $15/$75 Opus row, (2) rollup preservation of hand-written analysis disabled, (3) phase duration billed from session start instead of from its cursor -- each turned the gate RED, and restoring turned it green

## Epic 3 — 12 items (0 open at the move)

### P1 (A) -- rewrite the LAYER TIME-BOX rule to kill on SILENCE SINCE THE…

**Action.** P1 (A) -- rewrite the LAYER TIME-BOX rule to kill on SILENCE SINCE THE LAST NAMED METHOD STEP OR FINDING, with a ~45 min absolute ceiling, replacing wall-clock-since-launch

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 -- ENCODED, same day. _bmad/custom/bmad-code-review.toml's LAYER TIME-BOX fact is REPLACED, not tightened: kill on 8 MINUTES OF SILENCE since the last named method step or finding, 45 min absolute ceiling, 90 min whole-review. Also folded in the salvage step that Epic 3 proved matters -- message the layer `your box is up, report what you have NOW` BEFORE killing, because a bare kill returns nothing and that is the opposite of the rule's own stated purpose. Verify: rg 'THE MEASURE IS SILENCE' _bmad/custom/bmad-code-review.toml

**Success criterion.** Encoded in _bmad/custom/bmad-code-review.toml before Epic 4's first review; no layer killed while it is making progress

**Note.** WHY, in one line: a starved layer and a hung layer look identical to the current rule, because both emit nothing. Epic 3 killed three of four layers at 3.1 and three of four at 3.3 round one, and the Edge Case Hunter died silent TWICE on 3.3 -- leaving that story's loader/boundary territory unreviewed. One killed layer at 3.1 was literally reporting the cause: `sequential due to target lock`. Pair with P2: P1 stops the wrong kills, P2 removes the reason they happen. Do NOT ship P1 alone -- a better detector on top of real contention just waits longer before killing the same starved layer

### P2 (B) -- give every review layer its own CARGO_TARGET_DIR under /tmp…

**Action.** P2 (B) -- give every review layer its own CARGO_TARGET_DIR under /tmp in the layer prompt template, and delete the now-obsolete 'cargo serializes on the target lock, this is not a defect' advisory

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 -- ENCODED in the REVIEW LAYER CONFIGURATION fact as BUILD ISOLATION. Each layer exports CARGO_TARGET_DIR=/tmp/review-<layer>/target. The obsolete advisory that told layers target-lock contention was normal and not a defect is DELETED -- it was training them to sit through the exact starvation the time-box then misread as a hang. The still-true half (a sibling may briefly hold the daemon port) is kept, and `cargo clean` stays forbidden

**Result.** MEASURED AT STORY 5.1 (2026-08-10) -- SUCCESS CRITERION MET. 4 of 4 layers completed, zero coverage holes, zero timeouts; all four verified `cargo 1.97.1` and all four executed binaries. This is the FIRST clean four-layer run since 3.2, against Epic 3's record of three-of-four killed at 3.1 and again at 3.3. Each layer built into its own /tmp/review-<layer>/target and none reported lock contention. ONE UNRESOLVED TENSION worth ruling on at the Epic 5 retro: the Blind Hunter ran ~79 minutes, past the 45-minute per-layer ceiling in the LAYER TIME-BOX fact, but was never SILENT -- it was emitting named sweeps throughout (3000 seeds of generation, 80 seeds through the live dig path, 60 seeds x 400 ticks). The silence detector correctly left it alone and it produced a real finding; the hard ceiling would have killed it. The two rules disagree, and on this evidence the silence rule was right. Either raise the ceiling or state explicitly that silence overrides it

**Success criterion.** 4 of 4 layers complete on Epic 4's first review

**Note.** The root cause, not the symptom. Four layers run concurrently and EVERY ONE is mandated by the REVIEW LAYER CONFIGURATION fact to run the binaries -- that mandate is correct and stays, it is what made the Sonnet Blind Hunter valuable. The defect is four mandated cargo runs against ONE target/ lock. Cost: disk, plus a cold build per layer, which build in parallel instead of queueing. NOTE the advisory being deleted is currently telling each layer that contention is normal and expected, which trains them to sit through the exact starvation P1 is trying to detect

### P3 (C/R1) -- encode the disjoint hunter territories: Blind Hunter = si…

**Action.** P3 (C/R1) -- encode the disjoint hunter territories: Blind Hunter = sim-core, Edge Case Hunter = the shells; both Opus auditors keep whole-diff scope. CARRY THE REVERT RULE VERBATIM

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 -- ENCODED as the LAYER TERRITORIES (R1) fact, with the revert rule verbatim, the 1-in-8 convergence evidence stated honestly, and the Epic 1 humility clause. It carries an explicit SEQUENCING guard in its own text: do NOT apply the split unless P1 and P2 are both in place. Both are, as of the same day

**Result.** FIRST CLEAN MEASUREMENT, story 5.1 (2026-08-10) -- and it is the first time R1 has been measured on a review where all four layers actually finished, which is what Epic 3 could not do. REVERT RULE NOT TRIGGERED: no defect was found whose site sat inside a hunter's excluded territory. CONVERGENCE: 2 of 13 findings raised independently by two layers -- the Lantern unreachable!() in bridge.rs (edge + acceptance, and they disagreed on severity) and the emitter occlusion (acceptance + feature). 2-in-13 sits alongside 3.2's 1-in-8 and against Epic 2's inferred 3-in-3, so the convergence premise R1 rested on is still NOT reproduced at the rate that justified it -- but with territories now disjoint, low convergence is the intended outcome rather than evidence against the split, and the honest read is that this metric can no longer test the premise. Judge R1 on coverage and cost from here, not on convergence. COST against Epic 3's 862 turns / $45.52 per story baseline: 415 turns, $50.86, 127 min wall-clock, 40.4M tokens processed. Turns roughly HALVED; dollars up ~12%. Cache reads were 86% of tokens processed, down from the 96% recorded in both Epic 2 and Epic 3. Read the dollar figure carefully -- see the metrics row's 5.1 result for why it is not comparable to any pre-2026-08-08 number

**Revert rule.** Revert R1 (keep R2) if a defect is later found whose site sat inside a hunter's EXCLUDED territory. Control = mutation kill-rate against 1.3's baseline

**Success criterion.** Encoded; turns and cache-read tokens per review measured against Epic 3's 862 turns/story baseline

**Note.** TAKEN BY WOLF AGAINST MY RECOMMENDATION, and the reasoning matters more than the decision. I recommended closing R1 unapplied: its whole justification was convergence, and the one clean four-layer story of Epic 3 (3.2) showed 1-in-8, not the 3-in-3 Epic 2 inferred. Wolf took it -- and it is coherent BECAUSE he took P1 and P2 in the same breath. With the coverage holes closed, R1 partitions working coverage; without them it would thin coverage that is already holed. SEQUENCING IS THEREFORE LOAD-BEARING: P1 and P2 land BEFORE P3, never after or alongside

### P4 (R2) -- one verification pass per review, after all patches land, i…

**Action.** P4 (R2) -- one verification pass per review, after all patches land, instead of one per patch

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 -- ENCODED as rule (4) of REVIEW-COST DISCIPLINE. Apply all patches, THEN one clean-build gate.sh plus one full mutate.sh. Neither may be skipped or sampled, and a red final pass is read, not bisected by re-gating each patch

**Success criterion.** Encoded; re-gate turn count measured against Epic 3

**Note.** Re-gate turns are the highest cost-per-turn work in a review. ACCEPTED RISK, recorded so it is not rediscovered as a surprise: a later patch can break an earlier verified one, caught only by the final gate and the full mutation run. Both are mandatory and both were green on all three Epic 3 stories, so the net is judged safe -- but if a patch-breaks-a-patch defect ever ships, this row is where to look first

### P5 (E) -- make review-cost rule (3) MANDATORY rather than advisory: re…

**Action.** P5 (E) -- make review-cost rule (3) MANDATORY rather than advisory: review and patch never run in the dev session

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 -- ENCODED as rule (3), rewritten as a PRECONDITION rather than advice: if the current session did the dev, stop and start a new one before reviewing. Carries the measured price -- 3.3's review re-read 493k context per turn inheriting the dev session against 3.2's 213k in its own

**Success criterion.** Every Epic 4 phase has its own transcript in the ledger; per-turn context back under ~250k

**Note.** The free lever nobody pulled. The rule already existed and was correct; 3.3 ignored it -- dev and review share transcript 2a9b9908 -- and the ledger prices the miss exactly: 3.2's review re-read 213k context per turn in its own session, 3.3's review re-read 493k per turn inheriting the dev session. 2.3x, for carrying a dev transcript the review had no use for. Same shape as the time-box: a rule that exists, is right, and is not enforced

### P6 -- extend the AC-authoring check to VERIFICATION RECIPES: a recipe…

**Action.** P6 -- extend the AC-authoring check to VERIFICATION RECIPES: a recipe must be executed and shown to produce non-zero evidence before the story is saved, and must range-check its own output rather than trusting exit 0

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 -- CLOSED BY RECONCILIATION, not by writing it ourselves. Syncing to forge-process 1.1.0 surfaced the forge's EVERY LIVE-GATE STORY CARRIES AN EXECUTABLE PREFLIGHT BLOCK (its ep-06 retro A2), which is THE SAME RULE as P6 in another court and states the principle better than our draft did: PROSE PRECONDITIONS DO NOT COUNT AND NEVER DID. Taken as the PRINCIPLE only -- its [preflight] toml block, preflight.sh, court/skill_role/Hermes toolset resolution are Asgard machinery frostvein does not have. Frostvein's form is now a persistent fact in _bmad/custom/bmad-create-story.toml: the Verification recipe must be EXECUTED during story-creation and shown to produce NON-ZERO evidence before the story is saved, with three rules -- (1) range-check the output, never the exit code, EXIT 0 IS NOT A RESULT; (2) pin anything world- or time-dependent (pass tui --z N, never assume a camera/tick/dwarf position); (3) a recipe that cannot yet run must state the exact command and the exact non-zero observation the dev agent owes, so the obligation is inherited rather than lost. Sits next to the OBSERVABILITY INSTRUMENT RULE it extends: that rule made the instrument mandatory and tested; this one makes the RECIPE THAT DRIVES IT reproducible. BASELINE FOR EPIC 4 unchanged: zero irreproducible recipes

**Baseline.** Epic 3's baseline was zero spec-text defects and it was MISSED -- but read how it was missed, because the guard was not useless. The AC check worked where it was aimed: it caught seven draft defects across 3.1 and 3.2 before those stories were saved, including an instrument asserting a glyph change that cannot be observed. The class RELOCATED one step downstream, from the AC to the recipe that proves the AC. Three instances: 3.2's documented Verification recipe designates at the opening view level where the map is air, so it marks nothing and the feature reads as broken (the Feature Auditor's first live run reproduced exactly that); 3.3's live recipe was not reproducible and FAILED SILENTLY WITH EXIT 0, because its leading `<` assumed a fixed opening camera z and the camera follows a dwarf who moves -- it captured zero of every glyph and the auditor read it as `hauling is broken`; and four of 3.2's ACs disagreed with the shipped code, the code being right in each case. The guard checks the AC. Nothing checks the recipe

**Success criterion.** Baseline for Epic 4: ZERO irreproducible recipes

### T1 -- session_tokens.py must walk BOTH Claude subagent transcripts AND…

**Action.** T1 -- session_tokens.py must walk BOTH Claude subagent transcripts AND sibling/nested `codex exec` rollouts. This is Epic 2 forge blocker (2), now confirmed on both sides

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 -- FIXED and VERIFIED AGAINST AN INDEPENDENT ORACLE, which is the part that matters. session_tokens.py now folds in Claude sub-agent transcripts (<session-id>/subagents/agent-*.jsonl) and nested Codex rollouts (same cwd, overlapping window); --no-nested measures one file in isolation. PROOF, not a green test: re-running story 3.2's dev rollout produced 933 turns / 116,108,161 tokens / $79.25 against the row's 715 / 96,000,871 / $60.96 -- a delta of +218 turns / +20,107,290 tokens / +$18.29, reproducing the hand-built table in 3-2-the-dig.md (218 / 20,107,290 / $18.28) TO THE TOKEN by a different code path. The hand table named six rollouts; the tool found them by cwd + window overlap without being told which. Review side: 472 -> 683 turns, 119.7M -> 139.2M, and claude-sonnet-5 appears for the first time -- the Sonnet hunters were invisible to a parent-only sum. Six sabotages all killed (subagents not walked, nested not walked, cwd filter dropped, window filter dropped, quota summed instead of enveloped, .meta.json globbed as transcripts); 33 tests green, wired into gate.sh. Ledger header now states that pre-2026-08-08 rows are undercounts and are deliberately NOT retro-edited. THIS CLEARS FORGE BLOCKER (2)

**Result.** HOLDING AT STORY 5.1 (2026-08-10), and the magnitude is larger than the fix was justified on. 5.1's review recorded 415 turns / 40.4M tokens / $50.86 across claude-opus-5 and claude-sonnet-5, INCLUDING 5 subagent transcripts worth 28.2M tokens -- 69.8% OF THE SESSION. A parent-only sum would have reported roughly a third of the true cost and would have shown no Sonnet line at all, since both hunters are Sonnet and are invisible to the parent transcript. Epic 3 estimated this gap at ~20% of tokens across the project and 50-70% in review-heavy sessions; 5.1 lands at the top of that range and confirms the estimate rather than merely inheriting it. PRACTICAL CONSEQUENCE for the Epic 5 retro: any cross-epic cost comparison that reaches back before 2026-08-08 is comparing a full count against a partial one, and the ledger header already says those rows are deliberately not retro-edited -- so quote no trend line across that boundary

**Success criterion.** Story 3.2 re-measured: the $18.28 self-gate and the ~495k review-layer tokens appear in the rows

**Note.** This is the item that makes every other cost number trustworthy, and it is why the Epic 3 retrospective states plainly that its own figures are an undercount. Measured on 3.2 alone: six `codex review --base main` self-gate cycles cost $18.28 / 218 turns / 20.1M tokens in rollouts that no row records, and four review layers burned ~495k tokens across ~111 tool-uses that no row records. It also blocks forge propagation (F1) and it propagates to every sibling project, because session_tokens.py is a forge-process FILE. SABOTAGE-VERIFY the fix before believing it -- the metrics-ledger suite is wired into gate.sh precisely so this script cannot silently rot

### T2 -- add `--mark` to session_tokens.py for phase boundaries, so a pha…

**Action.** T2 -- add `--mark` to session_tokens.py for phase boundaries, so a phase recorded mid-session bills only its own window

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-24 -- shipped as forge-process 1.2.0 (dbc2fe6, PR #14) and pulled here; check reports in sync. Full record: t2-mark-forge-handover.md

**Success criterion.** A phase recorded mid-session bills only its own delta window

**Success criterion met.** NOT the criterion above -- the delta cursor already met it. Real defect: the cursor only advanced when a row was written. See the record.

**Followthrough.** METRIC RULE hand-merged into the three adapted workflow TEMPLATEs and acked, so the flag is invoked, not just available. Epic 8 records under it.

**Note.** Its absence has now cost an honest number on THREE rows across TWO epics: 2.1's review (mixed session), 3.2's create (opens at session start, so it includes an unrelated forge-process branch push and PR #13), and 3.2's review (opens at session start, so it bills story selection, the Codex handoff, independent verification, the quota_pp tooling AND the four-layer review to `review`). Each was handled by writing a caveat instead of a number, deliberately -- a hand-adjusted figure is less trustworthy than a stated caveat -- but three caveats is the signal that the tool, not the discipline, is what is missing

### T3 -- deterministic opening camera z (e.g. the level with the most sta…

**Action.** T3 -- deterministic opening camera z (e.g. the level with the most standable ground) plus an explicit --z flag for captures. PREREQUISITE TO STORY 4.1a, not deferred work

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 -- FIXED, and live-verified against a real daemon rather than asserted. view::initial no longer derives the opening view from entities.first(); it opens at world centre on the z with the most standable ground (Empty over Solid/Ramp, ties to the LOWEST z so it is reproducible, middle-z fallback when nothing is standable), and tui takes --z N to pin a level for captures. LIVE PROOF: against a real simd, three runs across 24 seconds of dwarf movement all reported `z 17/31` where the old code followed a wandering dwarf; --z 20 and --z 5 aimed exactly, --z 9999 clamped to 31/31, --z with no value bails. Four sabotages all killed, each by a distinct test. Gate green. NOTE the accepted trade-off, recorded in the code: the opening frame no longer centres on a dwarf -- if that annoys in play the answer is a centre-on-dwarf key, NEVER a nondeterministic opening. The reproducible-capture rule is also now in docs/technical-preferences.md beside the observability-instrument rule: exit 0 is not a result

**Success criterion.** Two clients connecting minutes apart open on the same level; a scripted capture is reproducible across runs

**Note.** WHY IT IS A PREREQUISITE AND NOT A NICE-TO-HAVE: `initial` takes z from snapshot.entities.first(), i.e. dwarf 0, who wanders wherever work took him -- so the same key sequence aims at a different z depending on when it runs. Epic 4 is a PURE CAMERA epic whose every AC will be proven by a scripted capture. Shipping 4.1a on a nondeterministic opening camera builds the epic's entire evidence base on the one thing already known to make evidence lie: it produced a false `the feature does not work` verdict at 3.3's review, with exit 0. Recorded in deferred-work.md from 3.3's review as a product decision; this promotes it

### E1 -- update epics.md and the sprint plan to SPLIT story 4.1 into 4.1a…

**Action.** E1 -- update epics.md and the sprint plan to SPLIT story 4.1 into 4.1a (raycast renderer) and 4.1b (sub-voxel creatures), with FR23/FR24 sign-off landing on 4.1b

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-08 -- DONE. epics.md now carries Story 4.1a (renderer: v toggle, camera, DDA, shared palette, framebuffer/100ms, plus a capture AC requiring --z and a range check) and Story 4.1b (sub-voxel models, fine-step sampling, LOD, seed palette swaps, a measured LOD/feel-budget AC, and the FR23/FR24 sign-off). The Epic List entry and the FR coverage map rows for FR23/FR24/FR26 are reconciled, and deferred-work's FR23 pointer now names 4.1b. COUNTER-METRIC CHECKED rather than assumed: the 2026-08-02 readiness report warned a 4.1 split lands at 13 stories and invokes the cut list -- but that assumed 3.2 was split too, and it was not. Actual: 3+4+3+2 = 12, exactly at the 8-12 cap, so the cut list is NOT invoked and FR16 (save/load) is not at risk

**Success criterion.** Both stories present in the epic text and the sprint plan before story-creation runs

**Note.** Wolf's decision, 2026-08-08, DELIBERATELY REVERSING the 3.2 one-story precedent on the evidence 3.2 produced. 4.1a = the `v` toggle and hint bar, camera move/turn, DDA voxel traversal, reuse of the 2D id->RGB table with no second mapping (AD-4), the shared cell framebuffer at the ~100 ms feel budget (NFR2). 4.1b = sub-voxel dwarf models (~10x5x13 boxes-as-code), fine-step sampling inside creature-flagged tiles, distance LOD, seed-derived palette swaps. The seam is natural -- different files, different risk, different failure modes -- not invented to hit a size target. Calibration: 4.1 as written is LARGER than 3.2, which shipped at 17 ACs, cost $132.98 and exhausted a full week of Codex quota in ~3h10m. sprint-status already carries provisional 4-1a/4-1b keys; this item makes epics.md agree with them

### T4 -- adopt a `live-gate` ledger phase once T2 (--mark) lands, so the…

**Action.** T4 -- adopt a `live-gate` ledger phase once T2 (--mark) lands, so the live run that makes a story done is billed to a row of its own

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-23 at the M2 retrospective -- SHIPPED, and it shipped AHEAD OF ITS STATED BLOCKER. This row said it was blocked on T2 (--mark); T2 is still open and the phase exists anyway. Story 7.2's ledger carries a `live-gate` row (484 turns / $151.81 / 1,870 min elapsed) AND a separate `signoff` row (86 turns / $8.26 / 95 min). The live run that made the story done is now billed to rows of its own, which is exactly the success criterion. READ THE MINUTES AS ELAPSED, NOT EFFORT -- 1,870 min spans the human gap between the vehicle capture on 2026-08-22 and Wolf's by-eye pass on 2026-08-23. T2 remains open on its own merits.

**Success criterion.** --rollup stops reporting stories with no live-gate row, because the row exists

**Note.** RAISED BY THE FORGE, and judged rather than dismissed. forge-process 1.1.0 made build_rollup flag stories with no `live-gate` row; the forge asked whether that was noise for frostvein and said NOT to edit session_tokens.py locally (it is a FILE entry and must stay byte-identical). VERDICT: NOT NOISE. It is the second of the two branches its own message names -- the gate ran and its cost was never recorded. Every Epic 3 story had a live run (real simd+tui against a real daemon at 3.1 and 3.2, Wolf watching the full haul loop for 3.3 AC17) and none of it was billed to its own row. BLOCKED ON T2, and that is why this is a separate item: frostvein runs its live checks INSIDE the review session, so they cannot be split into a row until --mark exists. Until then the flag is a standing true statement rather than a false alarm, and the annotate-and-move-on escape in its own wording keeps it from becoming a warning that always fires and gets ignored. ONE LINE OF IT IS ASGARD-SPECIFIC and was fed back rather than changed: `it is among the most expensive rows in this epic` is not true here -- frostvein live checks are minutes inside a larger session. The RECORDING argument carries universally; the COST argument is theirs.

### E2 -- story-creation for 4.1a must SETTLE two architecture questions r…

**Action.** E2 -- story-creation for 4.1a must SETTLE two architecture questions rather than discover them: whether creature-flagged tiles need a wire change in `protocol`, and how the raycast view stays inside the tui-has-no-sim-core-edge gate probe

**Owner.** Winston + Amelia

**Status.** dropped

**Closed.** 2026-08-23 at the M2 retrospective -- OBSOLETE, cannot be done. Epic 4 closed early on 2026-08-08; 4.1a's story was written and shipped before this item could apply to it, 4.1b was dropped and never written, and 3D-in-TUI is abandoned. Neither question has a story left to settle it. Recorded as dropped rather than done because it was never actually performed -- the distinction matters for follow-through honesty.

**Success criterion.** Both answered in the story's Key decisions, with the wire change landing in ONE commit if it is needed

**Note.** The 3.1 lesson, applied before the fact rather than after: 3.1 WAS a wire change and its own epic text did not say so, which meant protocol + the simd bridge + the pinned JSON literals + tui all had to move in one commit or the suite was red between them. Raycasting is pure client work and should fit AD-4 cleanly -- the client already receives entity positions and already owns the id->RGB table -- but `creature-flagged tiles ... sampled fine-step during DDA` is epic text that has never been traced against the actual wire shape. Trace it before writing ACs, per the CAN IT HAPPEN check

## Epic 4 — 1 items (0 open at the move)

### E4-P1

**Action.** Guard the NEW spec-defect subclass 4.1a exposed: an AC that is MEETABLE, IMPLEMENTED and NOT WHAT THE USER WANTED. Fix belongs at requirement + story authoring, NOT at review.

**Status.** done

**Closed.** 2026-08-23 at the M2-to-date retrospective -- MET, and closed on evidence rather than by expiry. Its named venue (an Epic 4 retro) was DELIBERATELY SKIPPED when Epic 4 was ditched, so the item was orphaned by design, not neglect. BOTH substantive candidate fixes were applied anyway and M2 is the proof. (a) OUTCOME-NOT-MECHANISM REACHED THE REQUIREMENTS: FR31 reads 'the isometric orbitable diorama the Visual Target describes: one zoom continuum from working-close to valley-vista, camera always usable, never lost' -- an outcome pointing at reference images -- and FR33 states the rule outright: 'Mechanism per the addendum's open question -- chosen by testing in its story, not here.' Against FR24's 'The RAYCAST 3D view is its own story', the success criterion is satisfied. (b) THE CHEAP ARTEFACT BEFORE THE BUILD is UX-DR22's sign-off gate, near-verbatim; it ran on 5 of 8 M2 stories and FIRED REPEATEDLY -- 6.1's 0.30%-of-frame dig site (2,255 px differ, 1,625 inside the predicted window), 6.1's contiguous ramps standing as a wall (8th instance of the AC-text class and the FIRST ever caught only by a human eye at the vehicle), and 5.4's tree/landform ruling REVERSED the same day once Wolf saw the composed frame. THE CAVEAT THE RECORD MUST KEEP: the gate is only as honest as its artifact. 5.4's approved artifact drew snow-laden spruce sprites -- geometry NOBODY WAS TASKED TO BUILD -- so the AC19 comparison failed on trees by construction. That is the 4.1a class reappearing INSIDE the fix for the 4.1a class. Rule earned and recorded: a Task 0 artifact script must not substitute geometry the renderer is not tasked to produce.

**Success criterion.** Milestone 2's replacement for FR24 states what the boss should SEE and FEEL and names no rendering technique; ruled on at the Epic 4 retrospective before any Bevy story is written

**Note.** READ THIS BEFORE PROPOSING A REVIEW-SIDE FIX -- there isn't one. All four review layers audit 'does the code match the spec?' and 4.1a's code matched it faithfully; even the Feature Auditor, whose whole question is 'would the user get the outcome the story promises?', is defeated when the PROMISE is the wrong thing. The tracked spec-defect class so far is 'AC unmeetable as written' (2.3's AC9, 4.1a's AC3) -- caught by review, reliably, 4 instances in 7 stories. This is the other kind and it is far more expensive: a whole story, gate green, four layers clean, $28 of review, and the feature was not what Wolf asked for. ROOT CAUSE IS VISIBLE IN THE REQUIREMENT: FR24 read 'The RAYCAST 3D view is its own story' -- it named a MECHANISM, not an outcome, so nobody ever had to ask what Wolf wanted to SEE. That is the same failure the AC-authoring rule (adopted at 3.1, _bmad/custom/bmad-create-story.toml, check 2 = OUTCOME NOT MECHANISM) already guards against ONE LEVEL LOWER DOWN -- it governs ACs and never reaches FRs. Wolf's own words: 'I meant actually more like isometric 3d camera view but I didn't manage to clarify that.' CANDIDATE FIXES for the Epic 4 retro to rule on: (a) extend the outcome-not-mechanism check UP to requirement authoring in bmad-prd/bmad-create-epics-and-stories, not only to ACs; (b) for any story whose headline outcome is VISUAL or otherwise subjective, require a cheap human-checkable artefact BEFORE the full build -- a mock frame, a sketch, a one-paragraph 'here is what you will see' -- since the whole cost here was discovering the mismatch only after a complete implementation; (c) accept the class as uncatchable-by-agents and rely on earlier live checks. Worked example to cite: FR24 + story 4.1a, and Milestone 2's re-statement of FR24 as an outcome is the first test of whether the fix holds.

## Epic 5 — 18 items (12 open at the move)

### M2-1

**Action.** Make `run()`'s live App testable. Decide what of run() is assertable without DefaultPlugins, build it, and make 'the live system tuple is exercised by a test' a standing story requirement.

**Owner.** Amelia + Winston

**Status.** done

**Closed.** 2026-08-23 -- DONE AND SABOTAGE-PROVEN, 9 of 9 KILLED. Two functions extracted from run() beside the projection_systems precedent: client_systems() carries the Startup tuple (setup_camera, setup_night_lighting, setup_projection_assets, setup_atmosphere, log_adapter), the PostStartup classify_client_local pass, and the Update tuple (camera_controls, update_fog_from_camera, toggle_overlay, fall_snow); capture_systems() carries the ordered (accumulate_motion, capture_after_frames).chain().after(ProjectionSet) pair. run() now calls both, so the SAME registration the live app uses is reachable from the suite. CaptureDistance was made pub -- it is the --distance seam 7.2 found inert. SEVEN NEW TESTS in crates/gui/tests/headless.rs drive client_systems through a MinimalPlugins app that mirrors run()'s resource set, and EVERY ASSERTION IS AN OBSERVABLE EFFECT, never `is it registered` -- a registration assertion would be precisely the vacuity this project keeps re-finding (see M2-11). They assert: the rig, the directional fill and the full sky spawn; --distance reaches the rig; no entity escapes the AD-14 partition; E zooms; fog tracks rig.distance through fog_falloff; a flake descends; F3 flips FpsOverlayConfig. NEW SABOTAGE TABLE mutations/m2-1-live-app-systems.sh, nine rows, one per system plus the --distance read: ALL KILLED, re-run after cargo fmt reformatted ingest.rs (the M2-8 dry anchor-check ran first and reported every literal still matching). Suite 364 -> 371, gate GREEN. THE CLASS IS CLOSED AT THE ROOT: deleting any system from run()'s tuples now turns the suite RED instead of shipping green, which is what five of eight M2 stories could not say.

**Success criterion.** A sabotage that deletes any system from run()'s registration tuple, or any production drive line feeding it, turns the suite RED. Proven by a mutation row before 8.1 is dev'd.

**Note.** WOLF'S RULING 2026-08-23, choosing the ROOT over catching the symptom a sixth time. THIS IS THE RETRO'S STRONGEST FINDING AND IT HAS A PAPER TRAIL. The cause was identified AT 5.4's REVIEW and filed [feature/MED] and DEFERRED: 'the live App built by run() has no test of any kind -- every headless test builds its own MinimalPlugins app, so nothing catches a system dropped from the registration tuples'. That MED defer then became the TOP-SEVERITY FINDING IN THE NEXT FOUR CONSECUTIVE STORIES. 6.1 run one: deleting BOTH blend_projection and flicker_projection from the live tuple -- the entire headline outcome -- left the suite 54/54 GREEN, because projection_systems was called only by run(). 6.1's review: AC6 proved the system is IN the tuple and said nothing about its INPUTS, so three one-line deletions (observe_tick, delta_secs, elapsed_secs) EACH KILLED WOW BEAT 2 with 57/57 green -- the four seam tests all hand-drove TickClock::advance. 6.2: accumulate_motion, the production code deriving the lit region, had ZERO test callers and had never executed once anywhere. 7.1: the ENTIRE on-screen readout (AC9's mechanism) and the --z pin could each be deleted or made inert, suite green. 7.2: --distance was parsed, validated, then never reached the camera rig, and its only test was NAMED for reaching the camera setup; AC10's restyle was pinned on a bookkeeping component rather than the style, and the mutation row sabotaged the same wrong branch. FIVE OF EIGHT STORIES. THE DEFECT RELOCATES ONE LEVEL DOWN EACH TIME IT IS CLOSED -- close it at the root. THE FIX HAS A WORKING PRECEDENT: 6.1's review closed it for that story with three production-drive tests plus three mutations; M2-1 generalises a fix that already worked. SEVERITY AT DEFER TIME IS A PREDICTION, and this one was wrong by four stories -- that is the meta-lesson.

### M2-2

**Verified 2026-08-31 — OPEN, filed.** No persistent fact in any `_bmad/custom/*.toml` carries Wolf's 2026-08-22 look-tuning rule, and `docs/tech-art-guidelines.md` does not state it either.

**Action.** Encode the look-tuning rule AND preserve the measurement method. Record Wolf's 2026-08-22 rule (a look change needs a concrete defect, not a preference); document the measure-against-the-artifact technique; carry 7.2's measured numbers forward as the gfx pass's inherited targets.

**Owner.** Amelia

**Status.** open

**Success criterion.** The rule is a persistent fact story-creation actually loads; the technique is written down with 5.4's numbers as the worked example; the gfx item inherits measured targets rather than a vibe.

**Note.** WOLF'S RULING 2026-08-23: KEEP THE METHOD, RETIRE THE SPEND. THE METHOD IS A GENUINE ASSET. 5.4 stopped re-deriving predictions from Bevy's shader model -- which mispredicted at round 2, forecasting snow at ~0.36 sRGB when the capture measured 0.09 -- and instead measured every target off the images already in hand, with a pure-Python PNG decoder written for the purpose (no numpy/PIL in the devpod). Result: ground median 21 -> 156 -> 144.6 -> 122.7 against the approved artifact's 123.3, i.e. 0.5% off, with the WHOLE DISTRIBUTION tracking (p90 187.3 vs 183.8), and the in-binary instrument and the offline decoder agreeing to 123 vs 123.4. D1's camp drift was solved OFFLINE TO FIVE DIGITS (0.22709 measured vs 0.227 modelled), locating the cause exactly -- BOOT_COMPOSITION_OFFSET carried a 28.6-unit component along the camera's right vector. D3 was FALSIFIED ON ARITHMETIC before a vehicle round was spent on it. THE SPEND IS WHAT IS RETIRED, not the technique: $274.75 across 8 patch rounds and 6 live viewings, converging PLACEHOLDER material to half a percent of target, after which Wolf ruled the practice out on 2026-08-22. THE GFX PASS INHERITS THESE MEASURED TARGETS: working-zoom ground median 231 on the cut floor with the three mark colours at luminance 120-150, so MARK POLARITY IS INVERTED there (marks darker than the floor, the opposite of the vista); the camp still reads blown at the vista; and AC9's band SKIPS its warm/ground assertions below the world top, which is precisely why no instrument caught the 231.

### M2-3

**Verified 2026-08-31 — OPEN, filed.** `bmad-code-review.toml` still carries "hard ceiling 45 minutes per layer and 90 minutes for the whole review" alongside kill-on-silence; the contradiction this item names is intact.

**Action.** Rewrite the LAYER TIME-BOX fact so SILENCE IS THE SOLE KILL CRITERION; demote the 45-minute per-layer ceiling to advisory with a far higher hard stop.

**Owner.** Amelia

**Status.** open

**Success criterion.** The two rules no longer contradict each other; encoded in _bmad/custom/bmad-code-review.toml before Epic 8's first review.

**Note.** WOLF'S RULING 2026-08-23: SILENCE OVERRIDES THE CEILING, stated explicitly. THE CONTRADICTION HAS BEEN OPEN SINCE 5.1 AND WENT UNRULED THROUGH SEVEN MORE REVIEWS -- it was raised in P2's own result note and simply sat there. The evidence is one-sided: at 5.1 the Blind Hunter ran ~79 MINUTES, past the 45-minute ceiling, but was NEVER SILENT -- it was emitting named sweeps throughout (3,000 seeds of generation, 80 seeds through the live dig path, 60 seeds x 400 ticks) and it produced a real finding. The silence detector correctly left it alone; the hard ceiling would have killed a productive layer. Note this is the SECOND time a time-box rule has been encoded and been WRONG -- Epic 2's original 20-min wall-clock rule said `done` for the whole of Epic 3 while costing coverage on two of three stories. Standing lesson, third confirmation: `encoded` and `correct` are different claims and a follow-through table cannot tell them apart.

### M2-4

**Action.** REDEFINE THE NFR6 BAR against the real vehicle, and correct every AC that names the WSLg devpod, BEFORE Epic 8 story creation runs.

**Owner.** Winston + Amelia

**Status.** done

**Closed.** 2026-08-23 -- DONE, and THE SCOPE WAS BIGGER THAN THIS ITEM ORIGINALLY SAID. I wrote it as 'every Epic 8 AC'; measured, it was NINE WSLg sites in epics.md and FOUR in ARCHITECTURE-SPINE.md -- NFR6's definition in BOTH files plus SIX story ACs (5.3, 5.4, 6.1, 6.2, 7.1, 8.1), FIVE OF THEM IN STORIES ALREADY MARKED done. THE PRD NEEDED NOTHING: prd.md has zero WSLg hits and defines NFR6 machine-agnostically as 'on the dev machine' with an [ASSUMPTION] that the number is set at architecture time, so the chain is SPINE SETS THE BAR -> EPICS RESTATES IT -> PRD DELEGATES. Wolf checked this and was right to ask. WHAT CHANGED: the bar's NUMBERS are untouched (60 fps working zoom, >=30 full vista -- they were met with headroom); only the MACHINE is corrected, to gingerspice / native-Windows gui.exe / NVIDIA Vulkan with simd in WSL over localhost. Clients are protocol-only TCP so the crate graph is unaffected. MEASURED FIGURES NOW CARRIED IN BOTH DOCUMENTS: 146 fps at 5.3 (unlit envelope), 140-146 sustained at 5.4 on the full lit and snowing world (2.3x the bar), >143 at 6.1 at BOTH working zoom and full vista (~2.4x and ~4.8x). METHOD, on Wolf's ruling: the five CLOSED stories' ACs are ANNOTATED, not rewritten -- their ACs were MET and their story files already record which machine each figure came from; only the venue named in the epic was wrong, and silently rewriting a closed story's AC is its own risk. 8.1 is corrected outright because it is not yet written. 5.3's envelope AC is annotated ANSWERED-NO: the window did not open on the devpod on any backend, and that finding IS the story's deliverable -- it is why the vehicle is native Windows. THE CATCH THIS REPRESENTS: had 8.1's WSLg clause survived into story creation it would have been the 4TH CONSECUTIVE EPIC shipping a false technical premise (6.2's wire claim, 7.1's control collision, 7.2's sim-Id requirement were 3 for 3; the project is 5 for 5). Caught at the retrospective instead.

**Success criterion.** The bar names a machine that can actually run the client; no live AC names the WSLg devpod as its venue; settled before 8.1's story is written.

**Note.** WOLF'S RULING 2026-08-23: redefine now, fix all three stories. THIS QUESTION WAS EXPLICITLY OWED TO THIS RETRO SINCE 5.3 AND THEN SAT THROUGH FIVE MORE STORIES. 5.3 walked the whole fallback ladder and proved the envelope does NOT hold on gingerspice on any backend, stock or self-built: the GL rung dies because WSL2 kernel 6.18 exposes no /dev/dri, so Mesa EGL-X11 falls to software presentation whose configs report NATIVE_RENDERABLE=FALSE and wgpu-hal refuses a non-presentable surface below tier 2 -- no container or distro change can fix a missing kernel device node. The Vulkan rung needed Dozen BUILT FROM SOURCE, and then died with a misreported DeviceLost (nvidia-smi watched flat at ~2.7 GiB of 12). The remaining lever was forcing downlevel limits in gui to dodge a non-conformant driver, which AC9 bans -- so it was NOT taken, correctly. CONSEQUENCE: NFR6's bar is defined against a machine that cannot run a hardware wgpu client on any stock-or-buildable path. Every figure since carries a THIRD label. MEASURED FIGURES TO SET THE NEW BAR FROM, all gingerspice / native Windows / NVIDIA Vulkan 591.74: 146 fps at 5.3 (unlit grey boxes), 140-146 sustained at 5.4 on the full lit/snowing world (2.3x the 60-fps bar), and >143 at 6.1 at BOTH working zoom and full vista (~2.4x and ~4.8x). THE FALSIFIED EPIC 8 TEXT, caught at this retro: story 8.1's AC reads 'Given the full world with picking active on the WSLg devpod, Then NFR6 still holds' -- a premise measured false at 5.3. This would have been the FOURTH CONSECUTIVE EPIC whose technical premises were wrong (6.2's wire claim, 7.1's control collision, 7.2's sim-Id requirement were 3 for 3, and 5 for 5 across the project); it is caught here instead of at story creation.

### M2-5

**Verified 2026-08-31 — OPEN, filed.** LAYER TERRITORIES still maps the M1 crates only (Blind Hunter = `crates/sim-core`; Edge Case Hunter = `crates/simd`, `crates/tui`, `crates/protocol`). No `gui` / `client-core` mapping exists.

**Action.** Give R1's territory split a real M2 mapping: Blind Hunter = PURE LOGIC (blend.rs, appearance.rs, project.rs predicates, client-core); Edge Case Hunter = THE SHELL (ingest.rs, capture.rs, main.rs, CLI, tests/). Both Opus auditors keep whole-diff scope. Carry the revert rule verbatim.

**Owner.** Amelia

**Status.** open

**Success criterion.** Encoded in the LAYER TERRITORIES (R1) fact; Epic 8's first review runs it without improvisation.

**Note.** WOLF'S RULING 2026-08-23, formalising what 6.1 improvised and every story since reused. R1 AS ENCODED NAMES sim-core / simd+tui+protocol -- NONE OF WHICH A gui-ONLY DIFF TOUCHES -- so a literal reading leaves BOTH hunters idle, and every M2 story from 6.1 on invented its own split at launch. 6.1's record asked for exactly this ('give R1 a real M2 mapping at the Epic 5/6 retro'). The mapping preserves R1's original principle -- deterministic core vs I/O boundary -- expressed in M2's crates rather than M1's. REVERT RULE, VERBATIM AND STILL LIVE: revert R1 (keep R2) if a defect is later found whose site sat inside a hunter's EXCLUDED territory; control = mutation kill-rate against 1.3's baseline. NOT TRIGGERED in any M2 story. READ THE CONVERGENCE EVIDENCE HONESTLY: R1's justification was convergence and it has never reproduced at the rate that justified it -- 2-in-13 at 5.1, 1-in-8 at 3.2, against Epic 2's inferred 3-in-3. With territories disjoint, low convergence is the INTENDED outcome rather than evidence against the split, so this metric can no longer test the premise. Judge R1 on coverage and cost from here.

### M2-6

**Verified 2026-08-31 — ALREADY DONE 2026-08-28, never struck. Closed on the evidence, not filed.** `bmad-code-review.toml` carries "REAPING IS PART OF THE REVIEW, NOT AN AFTERTHOUGHT (added 2026-08-28)": the orchestrator runs `scripts/reap-build-caches.sh --tmp-only --force` after triage is written and the story file updated, and records the reclaimed figure in the review record. That is what this item asked for.

**Action.** Add a reaper to review teardown -- delete /tmp/review-<layer>/target in the same step that writes the triage.

**Owner.** Amelia

**Status.** open

**Success criterion.** Disk after a review returns to its pre-review level; the ~92GB/review figure stops accumulating.

**Note.** WOLF'S RULING 2026-08-23. Flagged as a deferred item at 6.2's review and never actioned. P2's per-layer build isolation is the fix that made 4-of-4 layers complete on every single M2 review, so NOBODY WANTS TO REMOVE IT -- the cost is simply unbounded because nothing cleans up. A cold build per layer was already an accepted cost of P2; the leftover trees are not. Note `cargo clean` stays forbidden inside a layer -- this is teardown by the orchestrator, not a layer instruction.

### M2-7

**Verified 2026-08-31 — HALF DONE; only the remainder was filed.** The stamp half LANDED: `crates/gui/build.rs` stamps `GUI_BUILD_SHA` (rerunning on `.git/HEAD` and `.git/index`) and `crates/gui/src/lib.rs` exposes `BUILD_SHA` with a test. The automation half did NOT: there is no `scripts/` entry that cross-compiles and re-copies `gui.exe`, and no mandatory first line in the vehicle-session runbook.

**Action.** Kill the stale-binary trap BOTH ways: (1) a scripts/ entry that cross-compiles and re-copies gui.exe, plus a mandatory first line in vehicle-session-runbook.md; (2) stamp the build's git SHA into gui so it prints at startup and in every capture line.

**Owner.** Amelia

**Status.** open

**Success criterion.** A stale binary cannot be used unnoticed -- the runbook prevents it, and the startup line detects it from the evidence itself.

**Note.** WOLF'S RULING 2026-08-23: BOTH, because prevention alone already failed. THE TRAP FIRED THREE TIMES IN 5.4 ALONE and it was expensive. Sighting 1 cost A WHOLE VEHICLE SESSION: the first live check of the gating light patch showed NO CHANGE AT ALL, because the cross-compiled gui.exe was built at 13:24 while the earliest patch commit landed at 13:58 -- the binary predated all 20 patch commits and the frame was byte-identical to the falsification run. Sighting 3 was caught only by CROSS-CHECKING THE INSTRUMENT: a capture printed ground-median-luminance=145 while the PNG itself measured 123 in the exact instrument window, and 145 was precisely boot4's value -- so the --capture run used the round-6 binary while the live viewing used the fresh one. Those printed numbers were disqualified from the record. ROOT CAUSE IS PROCESS, NOT CODE: the cross-compile is a MANUAL step in the vehicle recipe and NOTHING IN THE DELEGATED DEV FLOW TRIGGERS IT. Same family as the exit-0 rule -- an unchanged frame is not evidence about the code until the binary is known to contain it. THE NATIVE WINDOWS BUILD ALSO HAS NO FORMAL HOME: 5.3 recorded it as owed to correct-course or this retro; it exists today only as a reproducible command sequence and a 187 MB untracked gui.exe. This item gives it one.

### M2-8

**Verified 2026-08-31 — ALREADY DONE, never struck. Closed on the evidence, not filed.** `scripts/audit-mutations.py` runs inside `scripts/gate.sh` and checks every row's literal against the source it claims to sabotage, failing the gate on BROKEN — the same guarantee this item asked for, gate-enforced rather than a pre-run step, and it matches ANY identifier bound to a string literal, not just `old`.

**Action.** Make the DRY ANCHOR-CHECK a standing pre-run step for mutate.sh -- grep every `old =` string against the live tree before any run.

**Owner.** Amelia

**Status.** open

**Success criterion.** Zero APPLY-FAILED rows discovered by a full run; stale anchors are caught in seconds instead of a 15-minute cycle.

**Note.** The technique already exists and already works -- it was invented at 5.4 round 6 and at round 7 it caught FIVE anchors staled by that round's own edits (taper, cap colour, crown colour, fog end, ambient brightness), all repaired before any run. It is not yet a rule, and the class hit again at 7.2. STALE-SABOTAGE-LITERAL, 3rd INSTANCE, and the first where the mutation and the code that outdated it were written IN THE SAME SESSION: the agent's own last refactors moved the lines two rows targeted, the table was not re-run after them, and the record asserted 'all eight KILLED' -- true when it ran, false when written. Re-running measured 2 APPLY-FAILED. THE RULE IN ONE LINE: a table is evidence only as of its last run -- RE-RUN AFTER THE LAST REFACTOR, NOT THE LAST FEATURE. Related, from 5.4 round 5: run 1 exposed two mutations whose anchors had gone stale against round-5 source (a cargo fmt reflow, a changed ClientLocal indentation count) and they REPORTED AS SURVIVORS WHILE PINNING NOTHING -- which is exactly the failure mode NO-COMPILE / APPLY-FAILED exists to surface.

### M2-9

**Verified 2026-08-31 — NOW DONE, the last part today. Closed on the evidence, not filed.** `scripts/mutate.sh` captures the exit code before any pipe (`out=$(...); rc=$?`). Commit-before-mutating, and the prohibition on undoing a mutation with `git checkout --` over an uncommitted fix, became a standing fact in `bmad-dev-story.toml` when forge-process 1.3.2 was merged on 2026-08-31.

**Action.** COMMIT BEFORE RUNNING MUTATIONS; never clear a mutation leftover with a tree-wide `git checkout`; capture the exit code BEFORE any pipe.

**Owner.** Amelia

**Status.** open

**Success criterion.** No uncommitted work is destroyed by a mutation cleanup; no runner exit code is masked by a pipe.

**Note.** All three earned by real incidents, all in 5.4. (1) The orchestrator edited the mutation table WHILE mutate.sh was executing it -- bash reads scripts incrementally, so that run's results were VOID -- then ran `git checkout -- crates/` to clear the killed run's leftover sabotage and DESTROYED THE UNCOMMITTED D6 WORK WITH IT. Reapplied and committed, but the rule is cheap and the loss was not. (2) A mutate.sh run piped through `tail` reported TAIL'S EXIT 0 and masked the runner's 1; the APPLY-FAILED was caught by READING THE PRINTED TABLE, not by the status. Same family as the project's standing rule that exit 0 is not a result. (3) Also worth carrying: 5.4 round 5 burned 391 turns over 67 minutes with a substantial number of them POLLING a background mutate.sh run that takes ~15 minutes of wall clock -- the single biggest avoidable cost in that round. Fix is procedural: arm one blocking watcher and stop, or batch the mutation run with other waiting work.

### M2-10

**Verified 2026-08-31 — OPEN, filed.** `bmad-dev-story.toml` carries the SELF-GATE CAP (three `codex review` passes) but says nothing about self-gate findings landing in the Dev Agent Record, which is what this item asks for.

**Action.** Self-gate findings MUST land in the Dev Agent Record, fixed or not. A finding that exists only in a handback message is lost at the session boundary.

**Owner.** Amelia

**Status.** open

**Success criterion.** No self-gate finding reaches the story record only because an orchestrator transcribed it.

**Note.** Earned at 5.4: TWO self-gate P2s were surfaced in the round-4 handback and recorded by NOTHING -- Codex named them 'pre-existing-style' and left them unfixed, and they reached the record only because the orchestrator transcribed them in-tree. The characterisation was also wrong on one of them: the incremental foliage reprojection staleness was NOT pre-existing, it was introduced by that very patch cycle (round 3 widened foliage_scale's read radius to two tiles above without widening reconcile's +/-1 NEIGHBOURS invalidation radius). Same family as the Epic 1 runbook rule about restating RED evidence across a session boundary.

### M2-11

**Verified 2026-08-31 — OPEN, filed.** create-story's only "headline" occurrence is inside the OBSERVABILITY INSTRUMENT RULE (the instrument shows the headline outcome). The recipe rule still requires non-zero evidence in general, not evidence of the story's own headline outcome. Adjacent work landed the same day (the deliberate-red rule (4) merged from forge-process 1.3.2) and does NOT close this.

**Action.** Extend P6 one step: a verification recipe must be shown to produce non-zero evidence OF THE STORY'S OWN HEADLINE OUTCOME, not merely non-zero output.

**Owner.** Amelia

**Status.** open

**Success criterion.** A recipe that cannot observe the story's headline defect fails story-creation, not review.

**Note.** P6 WORKED AND THE CLASS RELOCATED AGAIN -- the same shape as the AC-authoring guard relocating from AC to recipe at Epic 3. P6 fired well at 7.2: the recipe was EXECUTED LIVE AT CREATION and found the real trap, that designations are CONSUMED BY THE DWARVES, measuring the decay curve (79 marks, 68/59/51 at t+40/60/100, plateauing at 50 from t+120 as the rest become unreachable) and pinning --z 10 because zone tiles sit one level above the rock. AND THE RECIPE IT PRODUCED STILL COULD NOT SEE THE STORY'S OWN HEADLINE DEFECT. Two instances: 7.1's capture instrument was LEVEL-BLIND -- `terrain_tiles > 0` cannot see a hollow cut, measuring 209 vs 258 with 49 floor tiles missing -- on the ONE STORY whose named headline trap IS the hollow shell. 7.2's prescribed working-zoom capture PHOTOGRAPHED AN EMPTY SITE AND EXITED 0, because a dig slab sits at z+0.54 while the slice draws every solid tile at the cut as a full cube spanning [z-0.5, z+0.5] regardless of exposure, so buried digs were sealed inside opaque geometry -- and the dwarves dig the REACHABLE tiles first, so the marks surviving a capture window are exactly the buried ones. Measured live: 25 of 79 visible at t+2, 9 at t+46, 2 at t+64, 0 OF 50 FROM t+102 ON. AC13's counter could not catch it because ALL 50 GENUINELY WERE PROJECTED. THE GUARD CHECKS THAT THE RECIPE RUNS. NOTHING CHECKS THAT IT CAN SEE THE THING THE STORY EXISTS TO PROVE.

### M2-12

**Verified 2026-08-31 — OPEN, filed as `route:undecided` and surfaced as a risk.** Still undecided. `scripts/codex-handoff.sh` records the self-gate as never verified: it died with "Read-only file system", the story worked around nothing and simply never ran it, and the "Verify at 2.2" note was never resolved.

**Action.** DECIDE THE SELF-GATE'S FUTURE -- keep, replace, or drop `codex review --base main` as a pre-handback step. Its value is unmeasured after three epics.

**Owner.** Wolf + Amelia

**Status.** open

**Success criterion.** A decision on the record, with the measure that justifies it. Stop paying for it silently.

**Note.** THE EVIDENCE IS NOW THREE EPICS DEEP AND IT IS NOT GOOD. Epic 1's item that created it closed with an honest caveat: at 2.2 it found NOTHING on a ~1200-line diff while four later layers found seven real defects, so 'the honest read is that ~15 min of wall-clock bought a clean bill of health that four later layers contradicted'. M2 is worse. 5.4, the milestone's most expensive story: ZERO USABLE PASSES across the entire patch cycle -- one launch per round, BOTH truncated by the sandbox command-parent timeout before findings. Under the three-pass cap that is permitted, but it means that patch cycle carries NO SELF-GATE EVIDENCE AT ALL. 6.1: recorded verbatim as 'the self-gate is a COVERAGE HOLE for this story -- it produced no conclusion on either run'. AGAINST THAT, THE HONEST OTHER SIDE: 7.2's run two ran exactly 3 self-gate passes and fixed all 5 findings including 2 P1s, and 5.4's round-4 pass produced one legitimate close-zoom visibility finding that was fixed in 2ea6ae3. So it is not worthless -- it is UNRELIABLE, and its cost is real and unrecorded. The decision needs a measure, not a vibe.

### M2-13

**Action.** Split the gate into two tiers: pre-commit runs `scripts/gate.sh --fast` (~5s), pre-push runs the full gate (~67s). The fast tier must NAME what it skipped and print GATE GREEN (FAST), never GATE GREEN.

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-23 -- IMPLEMENTED AND MEASURED IN SESSION. scripts/gate.sh takes --fast; .githooks/pre-commit calls it; a new .githooks/pre-push runs the full gate. WOLF'S WORDS THAT PROMPTED IT: 'it takes a long time to run and it's ran always before commits so it's also a bit clumsy but there is a good point of running it before' -- both halves right. MEASURED WARM, not estimated: crates/simd/tests/serve.rs is 61 tests and 58.9s = 88% OF THE WHOLE GATE; sim-core 102 tests in 4s; gui's 112 tests in 0.14s; client-core 7 tests in 0s; simd unit (--bins) 18 tests in 0.41s; fmt + clippy + three dependency probes + metrics tests + mutation audit ~1s combined; full gate 67s. CORRECTION TO MY OWN FIRST MEASUREMENT, recorded because it would mislead anyone re-deriving this: gui first read as 50s and that was BEVY COMPILING, not tests running. WHY serve.rs MOVES RATHER THAN SHRINKS: it is slow BY NATURE -- real daemon, real socket, real wall-clock (a tick-rate test asserts elapsed within [1200,4500] ms, a 350ms sleep, a 10s IO timeout, 10ms poll loops). Making it fast means making it fake. THE REASONING THAT DECIDED IT: 67s on every commit is clumsy enough to tempt --no-verify, and a gate that gets skipped protects nothing. THE HONESTY CLAUSE IS LOAD-BEARING: the fast tier prints GATE GREEN (FAST), names crates/simd/tests/serve.rs as skipped, and calls it a COVERAGE HOLE -- the same rule this project applies to a timed-out review layer. A fast-tier pass pasted into a story record must be impossible to mistake for the full gate.

**Success criterion.** Pre-commit is ~5s; pre-push runs serve.rs; no fast-tier output can be mistaken for a full green.

### M2-14

**Action.** FULL RE-RUN OF EVERY MUTATION TABLE, once, to establish how many rows still KILL rather than merely apply.

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-23 -- RUN COMPLETE, 15 tables, 324 rows: 311 KILLED, 9 SURVIVED, 4 NO-COMPILE, 0 APPLY-FAILED. THIRTEEN ROWS (4.0%) PROVE NOTHING TODAY. THE HEADLINE: scripts/audit-mutations.py PASSES, EXIT 0, RIGHT NOW -- it scores 0 of the 13, because it catches APPLY-FAILED (of which this run found zero, the 2026-08-22 audit having already repaired the 29 it found) and is blind to both live modes: SURVIVED (applies, compiles, suite does not notice) and NO-COMPILE (literal matches, resulting code does not build). Static `applies` and dynamic `kills` are now MEASURED as different questions rather than argued. THE SPLIT IS BY MILESTONE AND IT HAS A CAUSE: M1 tables (Epics 2-3) 189/201 = 94.0%; M2 tables (Epics 5-7) 122/123 = 99.2%. Twelve of thirteen dead rows are M1, and not because M1 was sloppier -- because M2 CHANGED THE CODE M1 PINNED. 5.2's TUI adoption (AD-13) rebuilt the client loop around client-core's Mirror, so 2-1's `client loop receives deltas but never applies them` now SURVIVES. `frames key path never writes its command` survives in BOTH 2-3 and 2-4 (same sabotage, duplicated) as the tui CLI moved under it. 2-1's `step() clears the dirty set` NO-COMPILEs on `no field dirty on type &mut World`. THE ONE M2 SURVIVOR IS BENIGN VESTIGE: 7-1's `capture accepts an empty requested slice` targets `self.terrain_tiles > 0`, the level-blind check the 7.1 review ALREADY replaced on Wolf's ruling. THE BEST FIND, and it is section 4.2's own pattern caught by measurement: 3-1's `to_save drops designations` AND `from_save discards designations` BOTH SURVIVE, sharing one test (save_load_then_tick_matches_never_saved, crates/sim-core/tests/save_load.rs:9). Its save point is a stepped CONDITION -- the first tick a dwarf holds a stone -- introduced with a comment saying it `moves with the sim instead of going quietly vacuous`. But a dwarf holds a stone only AFTER the dig completes, and a designation is deleted the instant its job completes (lib.rs:883,898), so designations() IS EMPTY AT THE SAVE POINT and save/load of designations is pinned by nothing. THE FIX DESIGNED TO PREVENT VACUITY IS WHAT CAUSED IT, in another dimension. Cheap repair: assert the designation set is non-empty at the save point. FULL TABLE AND EVERY NON-KILL ROW: epic-5-retro-2026-08-23.md section 12.

**Success criterion.** A measured baseline: per-table KILLED / SURVIVED / NO-COMPILE / APPLY-FAILED across all 15 tables, and every non-KILL row named.

**Note.** WOLF'S RULING 2026-08-23, prompted by his own framing: 'nothing is more dangerous than a useless test suite so how can we improve it'. M2 IS THE EVIDENCE FOR THAT WORRY -- five of eight stories shipped a green suite while the headline feature was inert (see M2-1). THE GAP THIS CLOSES: the only instrument that measures suite VALUE is the mutation table, and scripts/audit-mutations.py -- added 2026-08-22, which found 29 of 326 rows across 9 tables could not apply, one dead since Epic 6, with 3-2-the-dig alone carrying 12 -- is a STATIC audit. Its own docstring says so. It proves a sabotage CAN BE INSERTED; it does NOT prove the suite KILLS it. A row can apply and still survive, and nothing re-runs an old table (~11 min each). So before today we knew our sabotages still FIT; we did not know they still BITE. METHOD: all 15 tables run SEQUENTIALLY -- mutate.sh is not concurrency-safe -- from a tree with crates/ clean, parsing the authoritative MUTATION RESULTS block rather than grepping the log (the first grep double-counted, because each verdict appears both inline and in the summary table). mutate.sh restores from a tar backup, NOT git checkout, so uncommitted work is safe -- which is the M2-9 rule already honoured by the tool. RESULTS: recorded in epic-5-retro-2026-08-23.md section 12 when the run completes.

### M2-15

**Verified 2026-08-31 — HALF DONE and STALE AS WRITTEN; the remainder was re-aimed, not filed verbatim.** The tick-timing half LANDED as `--at-tick` (`crates/gui/src/capture.rs`: `at_tick: Option<(u64, u64)>`, with the delivered-tick floor separated so an `--at-tick` capture scales it). The scenario-driven session mode did not. "Fold into Epic 8" is stale — Epic 8 has closed — so the issue aims the remainder at the gfx pass rather than at a closed epic.

**Action.** Fold the GUI-triggered vehicle run into Epic 8, riding on 8.2's command path: add tick-based capture timing (--capture-at-tick) and a scenario-driven session mode to gui.

**Owner.** Amelia + Winston

**Status.** open

**Success criterion.** An Epic 8 vehicle session is one command plus Wolf's eye, with no fps arithmetic and no stopwatch race.

**Note.** WOLF'S IDEA AND HIS RULING 2026-08-23: 'could we trigger vehicle run from GUI?' -- yes, and Epic 8 is the right home. ALREADY DIAGNOSED ONCE, IN A COMMIT MESSAGE: ed6d6e3 (2026-08-22, Task 6's vehicle session) states it outright -- 'RECIPE FINDING: --frames does not control tick count. Ticks observed = frames / framerate * 10, so on a 4080 a light scene rendered 1500 frames in 5.8s and saw 58 ticks, failing the motion floor of 100, while a heavier scene saw 237 from the same command. Use --frames 3000 here.' That is a SHARPER statement of the defect than this item first made, with the conversion formula measured -- and it lived only in a commit message, so it produced a per-story workaround (bump the number) rather than a fix. THIRD SIGHTING of the pattern M2-16 names: diagnosed, written down somewhere unindexed, never an action item. THE SHARP EDGE IS A UNIT ERROR: the capture is specified in --frames, a RENDER-RATE quantity and the only timing control (crates/gui/src/ingest.rs:239), while every assertion it feeds is in TICKS (motion: ticks >= 100). The conversion factor is fps, which changes with machine, zoom and scene, so the arithmetic is redone by hand per story and HAS BEEN WRONG. 6.1: --frames 600 at 10 Hz against >135 fps is ~44 ticks against a >=100 floor, so TASK 6'S FIRST COMMAND WOULD HAVE PANICKED BEFORE WRITING ANY PNG -- caught by Wolf, by hand, not by any test. 7.2: --frames 1500 is ~110 ticks, ten ticks of margin. THE CHANNEL RACE: channels decay to ZERO by t+114 (measured 2026-08-22: 39 marks, 14 by +52, 0 by +114) against a ~110-tick capture window, so the runbook has to say 'start the capture within a few seconds of this line' -- a stopwatch race against a window the same width as the capture. WHY gui CAN DISSOLVE IT: gui already owns the TCP connection to simd, the tick counter (TickClock/observe_tick) and the capture path, so it can designate, wait N TICKS, and capture internally -- no fps dependence, no race, no external command sender, and no blind TUI key path (which silently placed 0 zone tiles on a world that already had the other rects). WHY EPIC 8 RATHER THAN BEFORE IT: 8.2 IS 'designate with the mouse' from gui, so the command-sending path arrives anyway; the driver becomes a by-product of planned work instead of new scope. THE LINE TO HOLD (AD-4 / ground rule 1): gui owns the MECHANISM (--capture-at-tick, a scenario file), the STORY owns the SCENARIO. Sending protocol commands is not game logic -- 8.2 does exactly that -- but rects and orderings baked into the binary would be scenario data living in a client. Ground rule 1's 'no config system before a third concrete use' is satisfied: five vehicle sessions are behind us (5.4, 6.1, 6.2, 7.1, 7.2) and three lie ahead.

### M2-16

**Action.** FIXED IN SESSION -- scripts/mutate.sh poisoned the build cache: restore_all used `tar -xf`, which restores ORIGINAL mtimes, so artifacts built during a run outlive the source restored after them and cargo never rebuilds.

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-23 -- DIAGNOSED 2026-08-05 AND UNFIXED FOR THREE EPICS; re-derived from measurement while validating the new gate tier after M2-14's full re-run, then fixed with `tar -xmf` (--touch), which stamps extraction time as NOW so restored source always postdates the artifacts. MEASURED BOTH WAYS, not reasoned: with crates/ GIT-CLEAN, `cargo test -p simd` failed 1 of 18 -- loading_rejects_static_lantern_emitters_before_the_wire_bridge, panicking `lantern save reached the live world` -- and `touch`ing the sources rebuilt it to 18/18 on the same flags and the same binary name. Source mtimes read 2026-08-20 19:27 and 2026-08-22 11:33 against an artifact stamped 2026-08-23 12:51. Re-verified after the fix: a fresh table run leaves sources stamped at the current time, cargo recompiles, simd passes 18/18, crates/ clean. WHY IT MATTERS MORE THAN IT LOOKS: every story's sequence is RUN MUTATIONS, THEN RUN THE GATE. A poisoned cache makes that gate grade SABOTAGED CODE -- a false RED that reads exactly like a regression, or, if the trailing sabotage is one the suite does not catch, A FALSE GREEN. Strong candidate for the mechanism behind the `5 false failures in one session` already on record against this script. NOTE THE SHAPE: this is the evidence apparatus failing, not the feature -- the same class as 6.2's untested capture instrument and 7.1's level-blind oracle, this time in the tool the whole project's mutation evidence rests on. THE REAL FINDING IS THE DELAY, NOT THE BUG. The mechanism was already recorded, verbatim, in the project memory written at 2.3's review on 2026-08-05: 'mutate.sh restores source TIMESTAMPS too, so Cargo may reuse the mutated build even after the source is correct. That is also how a false GREEN can appear, not just a false RED.' The same note lists FIVE false failures in one session traced to it. It then sat unfixed through Epics 3, 5, 6 and 7 -- because it lived only in memory prose and WAS NEVER AN ACTION ITEM. This is the third instance of the meta-failure the Epic 2 retro named (its time-box rule was 'diagnosed exactly and written into no config file'), and the same shape as R1 and P6. `diagnosed` is a THIRD distinct claim alongside `encoded` and `correct`, and a follow-through table cannot see it either, because there is no row to mark. RULE EARNED: when a root cause is identified, open an item for it the same session or it does not exist. The concurrency hazard in the same memory note is UNCHANGED and still live -- run mutate.sh alone.

**Success criterion.** After a mutation run, the next cargo build recompiles from restored source rather than reusing an artifact built from sabotage.

### M2-17

**Action.** FIXED IN SESSION -- save_load_then_tick_matches_never_saved was VACUOUS for designations; both `to_save drops designations` and `from_save discards designations` survived M2-14's re-run.

**Owner.** Amelia

**Status.** done

**Closed.** 2026-08-23 -- FIXED, and both rows now KILL (3-1's table went from 3 survivors to 1). THE CAUSE IS THE FIX FOR VACUITY CAUSING VACUITY, which is why it is worth the words. The save point is a stepped CONDITION -- the first tick a dwarf is holding a stone -- introduced with a comment saying it `moves with the sim instead of going quietly vacuous`. But a dwarf can only hold a stone AFTER the dig completes, and a designation is deleted the instant its job completes (sim-core/src/lib.rs:883,898 -- the same line for `dug out` as for `cancelled`). So designations() had emptied itself by the save point and the round-trip loop, which does assert loaded.designations() == control.designations(), was comparing two empty vectors forever. THE REPAIR uses the mechanism 7.2's vehicle session found live: a dig buried in the rock mass is unreachable BY CONSTRUCTION, because work_positions demands a dwarf on a standable tile at the same z orthogonally adjacent, and standable means empty with solid below -- wall the four orthogonal neighbours in and no standing position exists, so the job is queued and retried forever (there is no abandon path) and the designation never dies. That is the same permanent field of blue marks Wolf saw at the working zoom. A second designation on such a tile is added before the save, plus AN EXPLICIT GUARD asserting the designation set is non-empty AT the save point -- so if the save point ever drifts past the last surviving designation again, the test says so loudly instead of going quietly vacuous a third time. ASSERT THE STATE THE COVERAGE DEPENDS ON; do not trust that it holds.

**Success criterion.** Both designation sabotage rows in 3-1's table KILL.

### M2-18

**Verified 2026-08-31 — OPEN, filed.** The row is intact in `mutations/3-1-give-the-order.sh` (`apply_command skips bounds clipping`, test `designation_rect_clips_to_world_bounds`) and `audit-mutations.py` passes it, so the anchor still applies to the live tree. Whether the mutation still SURVIVES needs a real run (~11 min per table) and was NOT re-run here; the claim stands as of Epic 5.

**Action.** 3-1's `apply_command skips bounds clipping` still SURVIVES -- the rect clip is redundant for CORRECTNESS and load-bearing for TERMINATION, and the test only covers correctness.

**Owner.** Amelia

**Status.** open

**Success criterion.** A sabotage that removes the rect clip turns the suite RED, without the test relying on a hang to do it.

**Note.** FOUND 2026-08-23 while closing M2-17; the last survivor in 3-1's table and deliberately NOT fixed in that session, to keep the scope Wolf set. THE ANALYSIS, so nobody re-derives it: designation_rect_clips_to_world_bounds designates rect (-1,-1,1)..(1,0,1) and asserts only the two in-bounds positions land. Removing the clip does not change that RESULT, because every out-of-bounds position is dropped downstream anyway -- a Channel needs is_standable, which needs a tile, and tile() is None out of bounds. So the clip is genuinely redundant for correctness. WHAT IT IS NOT REDUNDANT FOR IS TERMINATION: the clip is what bounds the ITERATION, so without it a rect with extreme coordinates iterates an astronomically large range instead of a no-op. The neighbouring test fully_out_of_bounds_rect_is_a_no_op uses coordinates near -3, far too small to expose that. THE DESIGN PROBLEM TO SOLVE, not just the fix: a test that catches this by actually hanging is a bad test -- it fails by timeout rather than by assertion. Prefer asserting the clipped bounds directly, or choosing a rect large enough to be measurably slow yet finite. Related history: Epic 3's most severe defect was a whole-world designate taking a delta from 378 bytes to 16,761,209 bytes, so unbounded rect handling has bitten this project before.

