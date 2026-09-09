# Story 10.8 — Task 8 vehicle session card

**This sitting owes AC12's closing half and AC15's two remaining defects. Nothing else.**
Everything below is your eye; every metric claim in this story is already measured headlessly.

**Vehicle:** gingerspice only. Branch `10-8-lighting-and-atmosphere-re-judged-under-the-sun`,
tip `990a65a`. **No PR yet, and the tip is one commit ahead of the remote** — say the word and it
gets pushed first, or this card cannot be walked.

Fill every blank; infer nothing.

## Read this first — the reference frame is NOT the one named `approved`

`approved-moonlit-camp-3479a43-{a,b}.png` **predates Ruling 6** (the lanterns went to a third at
`76c7c47`, after that pair was captured). Judging the live client against it will read as a
regression that is not one.

**The frame that matches this build is `candidate-lantern-third-2b9a307.png`** — the Ruling 6
candidate whose values shipped. Measured today against HEAD, it is the same look:

| frame | mean | shade-band 40–89 |
|---|---:|---:|
| `candidate-lantern-third-2b9a307.png` | 64.727 | 55.55 % |
| HEAD `990a65a`, run a (`head-990a65a-a.png`) | 64.993 | 55.44 % |
| HEAD `990a65a`, run b (`head-990a65a-b.png`) | 65.300 | 55.26 % |
| `approved-moonlit-camp-3479a43-a.png` | 65.526 | 55.32 % |
| `approved-moonlit-camp-3479a43-b.png` | **70.967** | **45.91 %** |

**`approved-…-b` is an outlier and is a FINDING, not a caption** — see the story's Change Log for
2026-09-09. Three same-conditions runs at HEAD sit inside mean 0.31; that frame is 5.44 from its
own pair-mate and 9.6 pp off it on the shade band. Do not use it for anything.

## Expect these — they are not faults

- **`exit=0`, not 101.** The ceilings were re-calibrated on the approved frame (AC6), so the
  guard now passes at the shipped look. `exit=101` here IS the finding of the sitting.
- **`touch crates/gui/build.rs` before every build.** Without it the stamp can lag a commit.
- **Do NOT use `launch-gui.ps1 -- <flag>`** — that form is issue #81 and cannot work; `$Checkout`
  takes position 0 and swallows the flag. Name the parameter: `-GuiArgs @('--subdiv','1')`.

## 1. Build and launch

```powershell
git checkout 10-8-lighting-and-atmosphere-re-judged-under-the-sun ; git pull
.\scripts\launch-gui.ps1
```

The script does the SHA check for you and refuses a stale or `-dirty` binary.

`gui build`: ____________  must read `990a65a`, must not say `-dirty`

## 2. Startup lines — measured at HEAD in the devpod, headless

| line | must read | got |
| --- | --- | --- |
| `subdiv 4: projected N terrain cubes at z 31` | `45042`, `entities=2164 chunks=118` | |
| same line, geometry | `faces=1155694 triangles=927622` | |
| `gui trees:` | `meshes=265 scenes_loaded=true source=embedded` | |
| `slice: z 31 …` | `(265 of 265 cut-face tiles at z 31)` | |

**`meshes=0` or `scenes_loaded=false` → STOP.** Nothing below means anything.
`subdiv 4` and not `subdiv 1` is the point of AC13 — if it says 1, stop.

## 3. Capture — expect exit 0

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--capture','10-8-vehicle.png','--frames','20000')
```

Measured at HEAD in the devpod, two consecutive runs, both **exit 0**:

| run | warm-lit | ground median | near-white | blown pool | p99 | exit |
|---|---:|---:|---:|---:|---:|---|
| a | 32,467 | 82 | 0.4733 % | 0.3011 % | 173.5 | 0 |
| b | 34,686 | 83 | 0.6047 % | 0.3418 % | 180.1 | 0 |

Ceilings: near-white **0.946072 %**, blown pool **0.623806 %**. Both runs sit under both.

**Noise floor at HEAD, worst of the two: mean 0.307 · near-white 0.1314 pp · warm-lit 2,219 px.**
This is 3.3x the near-white floor the story quotes from creation (0.040 pp) — the creation figure
was measured on a brighter build and must not be used to judge deltas on this one.

`capture range check:` ______________________________________  exit: ______

## 4. Your eye — AC12's closing half

Open `candidate-lantern-third-2b9a307.png` beside the live client. **Not** the `approved-*` pair.

| # | question | reading |
| --- | --- | --- |
| a | **Issue #75's six words.** Is *"lighting is still way off"* still true? You answered *no* at Ruling 6 on the first vehicle look; this is the confirmation on the built branch, and it is what closes UX-DR22. | |
| b | **Hover slab on a vertical face near the fire** (inherited eye-check). Close or reopen. | |
| c | **The three mark modes, apart at a glance, at working zoom** (inherited eye-check). Close or reopen. Note 10.7's trap: a mark is consumed in ~2.5 s, so pause the clock before looking. | |
| d | Anything new. | |

## 5. AC15's two remaining defects — the only look changes still open

Ruling 3 named four. **(a) fog and (d) rim you DEFERRED** (*"ok ..fine let's not mess with fog
now.."*) and nothing has landed on them. These two are what is left, and both are your call
because neither is a metric:

| # | your words at the sitting | knob | reading |
| --- | --- | --- | --- |
| b | *"snowfall does not start from the top of the screen depending view angle"* | spawn band: height `11.0 + SNOWFLAKE_FALL_SPAN 20.0` over `SNOWFLAKE_DISC_RADIUS 48.0` (`atmosphere.rs:182-196`) | |
| c | *"maybe flakes could be smaller"* | `snowflake_scale` (`atmosphere.rs:214`), today `0.3 + 0.18` | |

For (b), say **at which zoom / view angle** the band runs out — it is a coverage claim and can be
instrumented once you name the framing it must hold at. For (c), a direction is enough (*"half"*,
*"a third"*) and it comes back as candidate frames the way the lanterns did.

## 6. New this leg — the k=1 flank look (AC14)

Ruling 2 reached the path that lost it: at `--subdiv 1` settled snow was a slab with four snow
sides, and it is now the cell's top face only, so the flanks are rock. The **material** claim is
instrumented (`both_terrain_paths_paint_a_capped_cell_snow_on_its_top_face_only`, RED against the
slab first). Whether it READS right is not instrumentable — no statistic separates flank snow from
slightly-less-snow-area.

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','1')
```

k=4 is the shipped default and this path is only the fallback, so this is **for the record, not a
gate**. Look at a trench wall and a snow-capped ridge.

Does the k=1 control path read as stone flanks under snow, not silvered walls? ______________

## 7. Paste back

Build stamp · the four startup figures · the capture line and exit · the four eye readings in §4 ·
the two AC15 answers in §5 · the one k=1 answer in §6.
