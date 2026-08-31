---
model: claude-fable-5  # session model set by Wolf in the harness (not the Opus default); recorded per the model policy
baseline_commit: d02f9595a9e137c4dac8873b593fdc8a9886c7cf
---

# Story 10.2: The Live Seat — BlenderMCP on Gingerspice (SPIKE)

Status: review

## Story

As the boss,
I want to explore shapes and looks interactively in Blender with Claude driving alongside me,
so that starting points for assets come out of creative sessions, not cold scripts.

## What a spike's "done" means here

**The output is a decision, not a pipeline.** The open question is the handoff — how a look
found live becomes a committed headless script, so nothing the build depends on lives only in a
session. Writing a confident AC over that unknown would be the 4.1a shape (a meetable spec that
isn't what was wanted); the AC is the decision itself, recorded. No standing workflow, no shared
machinery, no client change is built by this story — if the decision is "MCP joins the standing
workflow", the runbook that operationalises it is *named as owed* in the record, not written here.

**The work splits by venue, and most of it is not agent work:**
- **Vehicle-side (gingerspice, Wolf's hands):** Blender GUI + the BlenderMCP addon + Claude in
  the Claude app. No devpod can open a window (measured; NFR6 amendment), so the live seat is
  vehicle-side *by construction*, and no agent can drive, watch, or verify the session itself.
- **Devpod-side (agent work):** the receiving end of the handoff — the committed script, its
  headless run through the 10.1 bench venue, the determinism and range-check evidence, and the
  written decision.

## Premises re-verified at creation — 2026-08-30

The epic orders its planning-time premises re-verified at story creation. Done; one is
unverifiable from here and is story work by design.

1. **"BlenderMCP requires a live GUI Blender session" — HOLDS**, confirmed against the upstream
   repo (github.com/ahujasid/blender-mcp, read 2026-08-30). Two halves: an **addon** (`addon.py`)
   that runs a socket server *inside a running Blender* — started by hand from the sidebar panel
   ("Start MCP Server", N-panel) — and an **MCP server** (`uvx blender-mcp`) that Claude launches
   and which connects to the addon over localhost:9876 (JSON over TCP; `BLENDER_HOST`/
   `BLENDER_PORT` to override). Requirements: Blender 3.0+, Python 3.10+, `uv` installed from its
   official installer (upstream is explicit: not via pip).
2. **Upstream's own security caveat, now on our record:** the `execute_blender_code` tool runs
   **arbitrary Python inside Blender**; upstream says "ALWAYS save your work before using it."
   The session recipe below carries both halves: save first, and the addon's socket stays
   localhost-only.
3. **Windows note that will otherwise burn Wolf's evening:** GUI apps on Windows don't inherit
   the terminal PATH — the Claude config may need the full path to `uvx.exe` or a
   `cmd /c uvx blender-mcp` wrapper. In Claude Code: `claude mcp add blender -- uvx blender-mcp`.
4. **Blender on gingerspice: NOT verifiable from this devpod** — no agent path to that machine
   exists. The epic budgets the install to this story ("gingerspice gets Blender for 10.2
   anyway"); it is Task 0, Wolf-side.
5. **The receiving end is live TODAY — run at creation, not claimed.** Control executed
   2026-08-30 on this devpod, clean tree at `d02f959`:
   `export_world.py` → `valley_bench.py` printed
   `range-check: exposed_cells=44984 non_sky_fraction=0.686815 distinct_colors=58993 terrain_luma=106.260`,
   exit 0, whole process 4.7 s — **reproducing 10.1's recorded range-check line exactly**, every
   figure (deterministic sim, same tick sample). Any future drift in these figures is a real
   change, not sampling noise.

## Acceptance Criteria

1. `scripts/gate.sh` passes with the story's work in place. (No crate change is expected; run it
   anyway — a doc-only story has shipped a red gate before by inheriting one.)
2. **The session happened:** one real exploration session on gingerspice — a tree or dwarf
   blockout, Wolf's pick live — with BlenderMCP connected to Claude, and its output captured
   durably. **Always committed** to `_bmad-output/implementation-artifacts/10-2-signoff/`: one
   viewport image of the found look, and `what-was-found.md` (which names where the Claude
   transcript lives). **Committed only if candidate (b) wins the handoff:** the `.blend`/glTF
   data file. The `.blend` is always *saved on the vehicle* (upstream's own warning) even when
   not committed. *Wolf-side by construction; an agent can verify the captured output exists,
   not the session.*
3. **The handoff proven once, end to end:** one artifact found live is carried into a committed
   headless script under `scripts/bench/` that runs on this devpod as
   `blender --background --python scripts/bench/<script>.py -- <args> <out.png>`, reads nothing
   from any live session (file inputs it names explicitly, only), and follows the 10.1 bench
   conventions: prints a range-check line **then** asserts on it, and exits non-zero when its
   output is empty. *Mechanism is load-bearing: "committed headless script" IS the story's
   subject — reproducibility without the session is the thing being proven.*
4. **The handoff artifact predicts the found look:** the script's rendered PNG is committed to
   `10-2-signoff/` beside the live session's viewport image, run twice with the pixel-diff figure
   recorded (Cycles CPU precedent: 0 of 2,073,600), and Wolf judges the pair — does the committed
   script reproduce what the session found? A handoff that loses the look is a failed handoff,
   recorded as such, and is itself a valid spike result.
5. **The decision is recorded** in this story's Dev Agent Record, all three parts: (a) **what the
   handoff is** — the mechanism that carried the look out of the session, chosen from the
   candidates below or discovered live; (b) **what it costs** — wall time, manual steps, and
   fidelity loss, measured on the one real pass, not estimated; (c) **whether MCP joins the
   standing workflow or stays an exploration tool** — Wolf's ruling, verbatim. If (c) is
   "joins", the record names the owed runbook as follow-up work; it is not written here.

## Tasks / Subtasks

- [ ] **Task 0 — The vehicle recipe (agent writes it into this file's record; Wolf executes on
      gingerspice)** (AC: 2)
  - [x] Install Blender 4.x (any current stable; the devpod's 4.3.2 is the reference point — a
        wildly newer major on the vehicle is a known-difference to write down, not a blocker).
        *Executed as 5.2 by Wolf's ruling (2026-08-30); consequences carried in the recipe's step 1.*
  - [x] Install `uv` via its official installer; download `addon.py` from
        github.com/ahujasid/blender-mcp; Blender → Edit → Preferences → Add-ons → Install →
        enable "Interface: Blender MCP".
  - [x] Connect Claude: `claude mcp add blender -- uvx blender-mcp@1.9.0` (Claude Code; pin
        measured 2026-08-31, see the recipe's step 4), or the
        `claude_desktop_config.json` `mcpServers` entry for the Claude app. **Windows PATH trap:**
        if the tool fails to spawn, use the full path to `uvx.exe` or `cmd /c uvx blender-mcp`.
  - [x] In Blender: N-panel → BlenderMCP → Start MCP Server (localhost:9876). Smoke-test with one
        scene-info request from Claude before doing anything creative.
  - [ ] **Hygiene, both from upstream's own warning:** save the `.blend` before any
        `execute_blender_code` use, and leave the addon socket on localhost.
- [ ] **Task 1 — The session (Wolf + Claude, on gingerspice)** (AC: 2)
  - [ ] One real exploration: a tree or dwarf blockout — whichever Wolf reaches for. One is
        enough; the spike measures the handoff, not the catalogue.
  - [ ] Capture as the session ends, before closing anything: save the `.blend`; export one
        viewport image of the found look; keep the Claude transcript reachable (it is the record
        of what was *asked for*, which the decision's fidelity judgement needs).
  - [ ] **Land the captured files where the agent can commit them** (this repo's working tree on
        either devpod mount, by whatever transfer Wolf prefers) — the devpod has no path to
        gingerspice, so without this step Task 3 cannot start.
- [ ] **Task 2 — The handoff (the open question; test, don't predict)** (AC: 3, 4)
  - [ ] Candidates, named so the session can try the cheap one first — the spike proves **one**
        end to end and records why that one:
        **(a) Claude re-emits the construction as a script** — the session's own
        `execute_blender_code` actions consolidated into one self-contained headless script.
        Cheapest when the whole look was built through code actions; loses any by-hand viewport
        edit silently — check the transcript for hand edits before trusting it.
        **(b) The `.blend`/glTF data file is the artifact** — the committed script only loads and
        renders it. Lossless, and closest to what 10.5's real pipeline does anyway (Blender →
        glTF); costs a binary blob in git that review can't read, so the viewport image beside it
        is the reviewable half.
        **(c) Hand re-expression from parameters** — most portable, most lossy, most expensive;
        the fallback if (a) and (b) both fail.
  - [ ] Whichever carries: the committed script follows the bench conventions it will live
        beside — **stdlib only** (Blender resolves the uv CPython; numpy is invisible — 10.1
        measured this), `bpy` import guarded, `scene.cycles.use_denoising = False` with the
        hard-failure comment, Cycles CPU, range-check-then-assert, non-zero exit on empty output.
        Copy the shape from `scripts/bench/valley_bench.py`; do **not** modify `valley_bench.py`.
- [ ] **Task 3 — The receiving end (agent, devpod)** (AC: 1, 3, 4)
  - [ ] Commit the script under `scripts/bench/` (name it for its content, e.g.
        `spike_tree_blockout.py`); commit the artifacts into `10-2-signoff/` — script's PNG, the
        session viewport image, a `what-was-found.md` naming the known differences and where the
        transcript lives (10-1-signoff precedent), plus the `.blend`/glTF iff candidate (b) won.
  - [ ] Run the script twice; record both range-check lines and the pixel-diff figure in the Dev
        Agent Record.
  - [ ] **No gate wiring and no mutation table for the spike script** — it is spike *output*,
        not machinery, and may be superseded the moment the decision is recorded. This is a
        stated exception to the tested-instrument rule (technical-preferences.md): the script's
        evidence is the executed two-run recipe below, recorded with its figures, in place of a
        standing test. If the decision keeps the script in a standing workflow, hardening it
        (test + sabotage row) is that follow-up's first task — name this in the decision record.
        `// NOTE:` the limitation at the top of the script so review reads it as decided, not
        forgotten. The gate still runs (AC1).
- [ ] **Task 4 — The decision, recorded** (AC: 5)
  - [ ] Write all three parts into the Dev Agent Record, costs measured not estimated, Wolf's
        ruling verbatim; mirror the essentials into the sprint-status note at close.

## Dev Notes

**Scope guardrails — the do-NOT list:**
- Do NOT build a pipeline, a standing workflow doc, an asset-import path, or any client/crate
  change. The Rust workspace should be untouched; if it isn't, something went wrong.
- Do NOT modify `scripts/bench/valley_bench.py` or `scripts/bench/export_world.py`.
- Do NOT pre-decide the handoff in code — no "handoff framework" that generalises the candidates.
  One proven pass, recorded.
- Do NOT write to the forge or assume Asgard context (standing rule); the future gfx-court plan
  is explicitly out of scope (Wolf, 2026-08-28).
- No UX-DR22 opening artifact is proposed: nothing lands in the client, so there is no look to
  approve in advance. Flagged as an interpretation of a process obligation (10.1 set the
  precedent), not settled by an agent.

**What already exists — build on it:**
- The bench venue and its conventions: `scripts/bench/valley_bench.py` (stdlib-only, guarded
  `bpy`, denoising off, range-check-then-assert, exit non-zero) — the shape to copy.
- `scripts/bench/export_world.py` — only needed if the spike script wants real world data;
  a pure asset blockout won't.
- `_bmad-output/implementation-artifacts/10-1-signoff/` — the committed-pair precedent
  (artifact + comparison image + known-differences note).

**Key decisions & traps:**
- **This story is a poor Codex-delegation fit** — it is interactive and Wolf-gated at every step.
  Run it orchestrator-side; there is no long unattended dev pass to hand off.
- **Post-merge branch trap:** PR #40 (10.1) is open and Wolf merges mid-session. This branch
  stacks on the 10.1 tip (`d02f959`); re-check branch and staging before every commit, and if
  #40 has merged, rebase onto main before pushing anything.
- **The spike can fail honestly.** "The handoff loses the look" and "MCP stays an exploration
  tool" are both successful spike outcomes if recorded with evidence. What is NOT acceptable is
  a session that produces nothing committed — then nothing the spike learned survives it.
- **Metrics:** phases are `create` / `dev` / `review`; run `session_tokens.py --phase` at each
  boundary (10.1's review+patch landed as one unrecoverable row — mark the boundary this time).
  A window belonging to no phase gets `--mark`, never a silent skip.

**Project structure — files this story touches:**
- `scripts/bench/<spike-script>.py` — NEW (name for content)
- `_bmad-output/implementation-artifacts/10-2-signoff/` — NEW (PNG, viewport image,
  what-was-found.md, `.blend` or glTF if candidate (b) wins)
- `_bmad-output/implementation-artifacts/10-2-the-live-seat-blendermcp-on-gingerspice-spike.md`
  — UPDATE (record)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — UPDATE (at close)

## Verification

**Runnable-now control — EXECUTED at creation, 2026-08-30** (the receiving venue is alive):

```bash
python3 scripts/bench/export_world.py /tmp/snapshot.json
blender --background --python scripts/bench/valley_bench.py -- /tmp/snapshot.json /tmp/bench.png
# observed: range-check: exposed_cells=44984 non_sky_fraction=0.686815
#           distinct_colors=58993 terrain_luma=106.260 — exit 0, whole process 4.7 s
```

**The recipe that cannot run yet** (the spike script does not exist at authoring time; the dev
pass inherits the obligation — exact command, exact non-zero observation):

```bash
blender --background --python scripts/bench/<spike-script>.py -- <declared-inputs> /tmp/spike.png
# REQUIRED: a range-check line with every figure non-zero, exit 0, PNG written
blender --background --python scripts/bench/<spike-script>.py -- <declared-inputs> /tmp/spike2.png
# REQUIRED: pixel-diff of the two PNGs recorded (expect 0 differing values; Cycles CPU precedent)
# REQUIRED: the failure path is OBSERVED exiting NON-ZERO — via a degenerate input if the script
#           declares file inputs, or via a one-off content sabotage (e.g. empty the geometry
#           before render) if it is self-contained. Exit 0 is not a result.
scripts/gate.sh   # full tier, green, run not claimed
```

Wolf's judgement (AC4) compares `10-2-signoff/` pair by eye — no agent closes it.

### References

- Epic text and premises: `_bmad-output/planning-artifacts/epics.md` (Epic 10 header + Story 10.2)
- Bench conventions and 10.1's measured traps: `_bmad-output/implementation-artifacts/10-1-the-headless-bench.md` (Tasks 2–4, Dev Notes)
- BlenderMCP upstream (read 2026-08-30): github.com/ahujasid/blender-mcp — addon + `uvx blender-mcp`, localhost:9876, GUI required, arbitrary-code caveat
- Signoff-pair precedent: `_bmad-output/implementation-artifacts/10-1-signoff/what-you-will-see.md`
- Venue measurements: sprint-status.yaml `10-1-the-headless-bench` block; NFR6 amendment in epics.md

## Change Log

| date | change |
| --- | --- |
| 2026-08-30 | Dev started (orchestrator-side; no Codex delegation per Dev Notes). Status → in-progress, sprint-status updated. Branch trap checked: PR #41 merged, origin/main is ancestor of HEAD, no rebase needed. Task 0 vehicle recipe written into the Dev Agent Record for Wolf to execute on gingerspice. Tasks 1–3 blocked until the session's captured files land in this tree. |
| 2026-08-30 | Story created. Epic premises re-verified: BlenderMCP's live-GUI requirement CONFIRMED against upstream (addon socket server in a running Blender, localhost:9876; `uvx blender-mcp` on the Claude side; arbitrary-code caveat now on our record; Windows PATH trap noted). Blender-on-gingerspice is unverifiable from the devpod and budgeted as Task 0, Wolf-side. The receiving end was proven live by a control run at creation that reproduced 10.1's recorded range-check line exactly (4.7 s whole process). Handoff candidates named for testing, not decided. No gate wiring and no mutation table for the spike script — a stated exception to the tested-instrument rule, with the two-run recipe standing in and hardening named as the follow-up's first task if the script survives its own decision. Stacked on 10.1's tip `d02f959` (PR #40 open); post-merge rebase noted. Revised after an adversarial fresh-context checklist review found 3 criticals before save: a fabricated explanation for a luma discrepancy that does not exist, an AC2-vs-Task-3 artifact-list contradiction, and a deferral aimed at 10.4 for hardening work 10.4's text never contains — plus a missing Wolf-side transfer step without which Task 3 could not start. |

## Dev Agent Record

### Agent Model Used

claude-fable-5 (orchestrator-side, not delegated — the story names itself a poor Codex fit).

### The Vehicle Recipe (Task 0 — written by the agent, executed by Wolf on gingerspice)

Everything below runs on gingerspice, by hand. The devpod has no path to that machine, so
nothing here is verifiable agent-side until the captured files land in this repo's tree.

**Setup (once):**

1. **Blender version: 5.2 — DECIDED** (Wolf, 2026-08-30: already installed, "not going to
   uninstall and tired of having different versions around"). The receiving end stays the
   devpod's 4.3.2 (the 10.1 bench venue is calibrated there and is not touched by a spike),
   so the major-version gap is now a **named condition of the spike**, with three
   consequences carried below: the data artifact is **glTF, not `.blend`** (`.blend` files
   are not backward-compatible across majors — the `.blend` is still saved on the vehicle,
   never committed); a candidate-(a) re-emitted script may hit 5.x→4.3 `bpy` API drift,
   which the receiving end checks and records rather than predicts; and the BlenderMCP
   addon on a 5.x major is unverified — the smoke test (step 6) is the tripwire. The
   version pair goes in `what-was-found.md` as a known difference.
2. **Install `uv` from its official installer** (upstream is explicit: not via pip).
   - Windows (PowerShell): `powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"`
   - Linux/macOS: `curl -LsSf https://astral.sh/uv/install.sh | sh`
3. **Install the addon:** download `addon.py` from github.com/ahujasid/blender-mcp (repo root).
   Blender → Edit → Preferences → Add-ons → Install → pick `addon.py` → enable
   **"Interface: Blender MCP"**.
4. **Connect Claude** (either seat):
   - Claude Code: `claude mcp add blender -- uvx blender-mcp`
   - Claude app: add to `claude_desktop_config.json`:
     ```json
     { "mcpServers": { "blender": { "command": "uvx", "args": ["blender-mcp"] } } }
     ```
   - **Windows PATH trap (premise 3):** GUI apps don't inherit the terminal PATH. If the tool
     fails to spawn, replace `uvx` with the full path to `uvx.exe`, or use
     `cmd /c uvx blender-mcp`.
   - **PIN: `blender-mcp@1.9.0` — MEASURED 2026-08-31, not guessed.** Read off gingerspice with
     `uvx --from blender-mcp python -c "import importlib.metadata as m; print(m.version('blender-mcp'))"`
     → `1.9.0`. Note that 1.9.0 shipped 2026-08-30, the same evening the seat went live, so the
     smoke test ran against a day-old release; that is an argument for pinning BEFORE the session,
     not after. Use `uvx blender-mcp@1.9.0` in whichever seat you run — Claude Code
     `claude mcp add blender -- uvx blender-mcp@1.9.0`, or `"args": ["blender-mcp@1.9.0"]` in
     `claude_desktop_config.json`. **The addon is numbered separately** (`bl_info` version, shown in
     Blender's Add-ons panel — Wolf read 1.5 there; upstream main is (1, 6) today). Do not pin the
     server to an addon number: `blender-mcp@1.5` would have rolled the server back eight months.
     Sandboxing stays ruled out as disproportionate (Wolf 2026-08-30) — freezes supply-chain exposure to a release already
     run, and makes the session reproducible. No sandbox around uv: the arbitrary-code
     surface is `execute_blender_code` inside Blender itself, which a uv sandbox would not
     contain; revisit only if the AC5 decision is "joins the standing workflow".
5. **Start the socket:** in Blender's 3D viewport press `N` → **BlenderMCP** tab →
   **Start MCP Server** (localhost:9876; leave host/port alone — the socket stays
   localhost-only).
6. **Smoke test before anything creative** — paste this to Claude:
   *"Using the Blender MCP tools, get the current scene info and tell me: the Blender
   version, the scene name, and a list of the objects in it. Read-only — don't execute any
   code and don't modify anything."*
   Success = the reply matches the outliner (fresh file: Cube/Camera/Light or 5.2's default
   equivalent), confirming the data came from this Blender. Failure modes told apart: tool
   never spawns → the PATH trap above; spawns but connection refused → Start MCP Server not
   clicked, or the addon didn't register on 5.2 (that one is a spike finding — report it).

**Hygiene (both from upstream's own warning):**
- `execute_blender_code` runs arbitrary Python inside Blender. **Save the `.blend` before the
  first such call**, and save again at any point you'd mind losing.
- Do not expose the addon socket beyond localhost.

**The session (Task 1):** one real exploration — a tree or dwarf blockout, whichever you reach
for. One is enough; the spike measures the handoff, not the catalogue.

**Capture as the session ends, before closing anything:**
1. Save the `.blend` (always — vehicle-side backup; it stays there, 4.3.2 can't read it).
2. **Export glTF** (File → Export → glTF 2.0, `.glb` is fine) — with the 5.2/4.3.2 gap this
   is the only data artifact the receiving end can open, so capture it even if candidate (a)
   looks likely; it's the fallback if re-emission hits API drift.
3. Export **one viewport image** of the found look (Viewport Render Image is fine).
4. Keep the **Claude transcript reachable** and note where it lives — the decision's fidelity
   judgement needs the record of what was *asked for*. If the look involved any **by-hand
   viewport edits** (not through Claude's code actions), say so: it decides whether handoff
   candidate (a) can be trusted.

**Transfer (without this, Task 3 cannot start):** land the captured files —
viewport image, `.blend` (and/or a glTF export), transcript pointer — anywhere in this repo's
working tree on either devpod mount, e.g. drop them under
`_bmad-output/implementation-artifacts/10-2-signoff/`. Any transfer route you prefer; the agent
takes it from there (commits, handoff script, two-run evidence, decision record).

### Spike output — the two templates (DRAFT, not yet standing machinery)

These are **recorded here as the spike's product**, proven against exactly one asset
(`SM_VoxelPine_Tree02`, 2026-08-31). They are deliberately NOT installed as a workflow, per this
story's scope guardrail: if AC5(c) rules that MCP joins the standing workflow, promoting these into
`docs/` with a runbook is that follow-up's first task. Every figure below was measured, not
proposed.

**BLOCKING BEFORE ASSET #2 — the scale constant is UNSET, and it is not per-asset.** This tree was
built at **0.2 m voxels off a 1.2 m dwarf**, giving a 7.6 m tree, 38 voxels tall. At that same voxel
size the DWARF is 6 voxels tall, and the reference sheet gives him a beard, belt, tunic panel and
lantern — none of which survive 6 voxels. Pick metres-per-voxel from the DWARF's detail needs
(~0.1 m or finer), fix it once, and let every other asset follow. If each asset picks its own
anchor they will not compose in one scene, and the correction is a re-export of everything rather
than a tweak. Related: the client's cell is a unit cube (`Cuboid::default()`) and `worldgen.rs`
grows trees 4–6 cells, so the cells-per-asset conversion is a second decision that belongs with
this one.

#### A. The standing asset contract — does not change per asset

1. **Scale.** Voxel size and the dwarf-height anchor come from the project constant above, never
   from the asset. State metres-per-voxel and the asset's height in voxels AND in world cells.
2. **Geometry and transform.** Transforms applied, identity. Origin at base centre, sitting on
   zero. **Centred in BOTH horizontal axes** — this tree shipped 0.5 voxels off in X while Z was
   exact, so the check is `bbox centre X == 0.000 and Z == 0.000`, not "looks centred".
3. **Material.** One material, one primitive, one draw call. Flat shading, single-sided
   (`doubleSided: false`). Texture filter NEAREST, wrap CLAMP_TO_EDGE. One palette atlas, colours
   inset in their cells so no mip level bleeds. Palette hexes come from the reference sheet and are
   named constants at the top of the generator.
4. **Export.** GLB, `extensionsUsed: []`, plain metal-rough, all UVs inside 0–1.
5. **Topology may be unwelded, but must be declared.** A greedy voxel mesher produces disconnected
   quads — this asset is 2,565 quads, 2,565 × 4 = 10,260 verts, no vertex sharing, V−E+F = 2565.
   That is correct and should not be "fixed" by welding. It does mean smooth normals, subdivision,
   auto-LOD and adjacency-based collision will not work as-is, so the asset notes must say so.
6. **Self-verification, in this order: print the figures, THEN assert on them, then exit non-zero
   on failure.** Minimum figures: verts, tris, bbox size, bbox centre X and Z, signed mesh volume
   against the voxel count. **Exit 0 with no output is not a result** — the 10.1 lesson, paid for
   already. Signed volume is the right closure oracle here precisely because the mesh is
   non-manifold and a conventional manifold check would fail a perfectly good asset.
7. **Deliverables, all three:** the `.blend`, one GLB per variant, and **a standalone headless
   generator script** that reproduces every variant with no MCP, no live session and no manual
   step — `blender --background --python <gen>.py -- <variant> <out.glb>`, stdlib + `bpy` only
   (Blender resolves its own Python; numpy is not visible), deterministic, explicit seed if
   anything is random. The script is the durable record; the session is not.
8. **Known differences.** The asset notes state anything a consumer would otherwise discover the
   hard way: unwelded topology, T-junctions, any axis asymmetry, any deviation from this contract
   and why.

#### B. The per-asset brief — the only part that changes, and it doubles as the prompt

Drop this beside the reference image in the session's `references/` folder and paste it in:

```
Build a game-ready voxel asset from the reference in this folder.

REFERENCE : <file>, <which section / which subject>
ASSET NAME: SM_<Subject>_<Variant>
VARIANTS  : <list, with the parameter that differs — e.g. tier count and radius>
SCALE     : <metres per voxel> from the project constant; <subject> is <N> voxels tall
PALETTE   : the reference sheet's hex values, verbatim, as named constants

Follow the standing asset contract in this folder (contract.md) in full. Where the
reference and the contract disagree, the contract wins on structure and the reference
wins on look — and say which you followed.

Deliver: the .blend, a GLB per variant, and the standalone headless generator script,
and print the generator in full in your reply as well as writing it to disk.
```

**Why the last line is not optional.** This session's transcript lives only inside Claude Code on
gingerspice — one `/clear`, one crash or one retention window from gone, and it is currently the
only record of how the asset was made. The emitted script is what replaces "the transcript is the
record" with "the committed script is the record", which is the exact question this spike exists to
answer.

### Debug Log References

- 2026-08-31, **RESOLVED: the pin is `blender-mcp@1.9.0`** (measured on gingerspice). "1.5" was
  the ADDON's number, as suspected below; the investigation that established this is kept because
  it is the reusable half — the addon and the server are numbered separately, and the panel shows
  the addon.
- 2026-08-31, the version pin is NOT closed — "1.5" does not identify the server that ran.
  Wolf reported the MCP version as **1.5**; checked before writing it into the recipe, because a
  wrong pin downgrades a seat that currently works. Three facts disagree with it. (a) PyPI has no
  `1.5` — the line is `1.5.0` (2026-01-08) through `1.5.6` (2026-03-18), and **`1.9.0` shipped
  2026-08-30**, the same evening the seat went live. (b) `uvx blender-mcp` with no pin resolves to
  the newest release, so a fresh setup that night got 1.9.0, not a January build, unless a stale uv
  cache intervened. (c) Upstream's `addon.py` carries its OWN number — `bl_info["version"] = (1, 6)`
  on main today — so the addon and the PyPI package are numbered separately, and Blender's Add-ons
  panel shows the ADDON's version. "1.5" is most likely read from there.
  **Owed:** the server package version, from the machine that ran it —
  `uvx --from blender-mcp python -c "import importlib.metadata as m; print(m.version('blender-mcp'))"`.
  Pinning `blender-mcp@1.5` on today's evidence would roll the server back eight months against a
  1.6-era addon; not written.

- 2026-08-31 checkpoint (agent-side, nothing to unblock): no `10-2-signoff/` and no untracked
  files anywhere in the tree — the session capture has not landed, so Tasks 1–3 remain blocked
  exactly where they were. Branch trap re-checked: `origin/main` (09f24ae) still an ancestor of
  HEAD, no rebase needed. `scripts/gate.sh` run at this state: **GATE GREEN** (all nine rows,
  including bench tests and the mutation-table probe) — AC1 holds for the doc-only state and is
  owed again once the spike script lands.

- 2026-08-30 evening, Wolf reports from gingerspice: **"MCP setup with Blender works now"** —
  the seat is live on Blender 5.2. That verifies the one premise unverifiable from any devpod:
  the BlenderMCP addon registers and round-trips on a 5.x major. Task 0's setup boxes checked;
  the hygiene subtask stays open (it is session-time behavior, exercised during Task 1). The
  exploration session (Task 1, sample assets) continues tomorrow, 2026-08-31. Blender-mcp
  version pin per the recipe CLOSED 2026-08-31: `blender-mcp@1.9.0`.
- 2026-08-30 dev start: branch `10-2-the-live-seat-blendermcp-on-gingerspice-spike`, clean tree.
  Post-merge branch trap checked: PR #41 (10.1) merged; `origin/main` (09f24ae) verified an
  ancestor of HEAD — no rebase needed. No `10-2-signoff/` exists yet; Tasks 1–3 blocked on the
  Wolf-side session and file transfer.

### Completion Notes List

### The Decision (AC5)

- **The handoff is: candidate (a) — the session re-emits its construction as a standalone headless
  generator — and it carried BIT-EXACTLY, which is stronger than the AC asked for.**
  `voxel_pine.py` was written by the session, committed, and re-run on the devpod: it reproduces
  all four GLBs **byte-identically** to the files exported on gingerspice, from a different
  machine, with no MCP, no live Blender session and no manual step. Candidate (b) (ship the data
  file) is superseded — the GLBs are committed as convenience, but the script is the artifact of
  record. Candidate (c) (hand re-expression) was never needed. The enabling condition was that
  every geometry change went through tool calls rather than the viewport, which is what let the
  construction be re-emitted at all.

- **It costs: effectively nothing to carry, and about ten minutes to produce.**
  Measured, not estimated. Session (Wolf + Claude on gingerspice): reference sheet landed 07:33,
  exploration finished 08:58 — and inside that, the WIP-to-final look pass took **five minutes**
  (08:53 → 08:58). The generator was emitted on request and delivered by 09:30, ~30 minutes,
  including two real defect fixes. Regeneration on the devpod: **1.75 s per variant**. Fidelity
  loss: **zero, bit-exact**. Manual steps in the handoff: **none** — the script takes a type and
  an output path and reads nothing else. Round trip idea → four verified game-ready assets:
  **2 h 10 min**, with the boss doing other work alongside.
  Not free, and worth naming: the handoff only holds while authoring stays inside tool calls. A
  by-hand viewport edit is invisible to the transcript and would break the property silently.

- **MCP's place (Wolf, verbatim, 2026-08-31):** *"AC5 hmm.. yes I think there is great value with
  MCP .. it will be much faster to tweak (actually I did for trunks because at first those were
  too thick) ... so we will keep it...just need to think about handover process at start ..
  templates are first step .. but ..that is not urgent now"*

  **Reading: MCP JOINS the standing workflow as the AUTHORING SEAT; the committed generator is the
  deliverable.** The trunk-thickness tweak is the ruling's own evidence — a look correction that
  would have cost a script edit, a re-render and a re-look offline took one instruction live.

- **OWED, named here and deliberately not built (this story's scope forbids it):**
  1. **The handover runbook** — Wolf: *"just need to think about handover process at start"*, and
     *"templates are first step .. that is not urgent now"*. The two drafts above (the asset
     contract and the per-asset brief) are that runbook's content, proven on one asset. Story 10.3
     is its natural home.
  2. **The scale constant.** Metres-per-voxel is a PROJECT constant, not a per-asset choice, and
     it is currently unset. 0.2 m gives the dwarf six voxels of height. **Blocks asset #2.**
  3. **Hardening `scripts/bench/spike_pine_render.py`** — the stated exception in Task 3 says that
     if the decision keeps the script, a test plus a sabotage row is the follow-up's first task.
     The decision keeps it.

### File List
