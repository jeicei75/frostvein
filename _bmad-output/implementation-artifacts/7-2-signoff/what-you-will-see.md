# Story 7.2 sign-off artifact — "here is what you will see" (UX-DR22 opening half)

**Status: APPROVED BY WOLF 2026-08-21. AC1 is MET and the gate is OPEN.** Approved on the written
halves plus the existing before capture; implementation may start.

**Three rulings given at approval, recorded here as the authoritative record:**

1. **(d) The mark presentation is the FLOOR SLAB** — one thin slab on the floor of the marked tile's
   own volume, the same geometry for all three kinds, colour carrying the kind. The vista
   sub-legibility named below is accepted deliberately; the bar is the working zoom.
2. **AC9's evidence recipe is fixed by taking the VISTA CAPTURE AT FULL DEPTH** rather than at
   `--z 10`, so the marks are genuinely on screen *and* the warm/ground band assertions genuinely
   run. The working-zoom capture stays `--z 10 --distance 30`, where its job is legibility and the
   mark counts. The campfire reading blown at full depth is a known carried-open item and **must not
   be re-tuned to make this capture pass**.
3. **The mark colours break with the TUI's deliberately** (item 6 below): cold-or-neutral in `gui`,
   so UX-DR5's warm/cold read survives. The two clients will not agree on colour.

Part (a), the before capture, is **NOT owed and NOT waived — it already exists.** You took it on
gingerspice on 2026-08-20 as `7-1-signoff/7-1-slice.png`, on the shipped 7.1 binary, and it is
reproduced and read below. One gap is named honestly rather than left implicit: it was taken at the
boot distance, not at the working zoom this story is about. That gap is the reason Task 4 adds
`--distance`.

**Three things in here want a ruling now rather than at the viewing**: the mark presentation (d),
the mark colours breaking with the TUI's, and a defect in the story's own AC9 evidence recipe that
would otherwise be discovered on the vehicle. All three are at the end.

## (a) The before capture — ALREADY TAKEN, 2026-08-20

`_bmad-output/implementation-artifacts/7-1-signoff/7-1-slice.png`

```cmd
gui.exe 7451 --capture 7-1-slice.png --frames 1500 --z 9
```

What it shows, read off the image rather than assumed: the cut at z 9 as a **flat, near-featureless
blue-grey field** filling the lower two thirds of the frame, the aurora and starfield above it, the
readout `Slice: z 9/31 — underground` top-left, and the camp as a warm pool roughly **4% of the
frame** near centre — snow-lit tiles, ice-blue rim, five dwarves and their lanterns, a scatter of
stone items. The 6.1 dig is in there and is not separately legible at this framing.

**That is the "before": you have cut into the mountain and you can see the rock, but nothing on
screen tells you what you ORDERED.** The dug tiles are visible as absence. The marks that caused
them are not drawn at all.

The one thing that capture cannot show you is the working zoom, because the shipped binary has no
way to ask for it — `BOOT_DISTANCE = 90.0` is the only distance a capture can take today.

## (b) What this story adds

**The orders you gave from the TUI become visible in the 3D client.**

Designate a dig from a TUI client on the same daemon and the tiles you marked light up in the Bevy
window as marks on the rock. Designate a channel and it reads as a *different* mark. Drop a
stockpile and its tiles read as a third. Nothing you do in the TUI is invisible in the window any
more: you order work in one client and watch it become work in the other.

Then watch them go. A dwarf digs a marked tile and the mark disappears with the rock. Cancel a
designation from the TUI and it is gone on the next tick. Both by the same path — the client is not
tracking your orders, it is showing you the world's current answer, every tick.

The aim, in one sentence: **at the working zoom the window stops being scenery and starts being an
instrument you can read work off.**

## (c) What you will NOT see

Each line is for you to rule on.

1. **No mouse, no picking, no selection, no cursor.** `gui` binds no mouse input of any kind today
   — verified again against source for this artifact, still zero `MouseWheel`, `MouseButton` and
   `CursorMoved` bindings. Picking is Epic 8.
2. **`gui` still issues no commands.** You designate from a TUI client, exactly as at 6.1 and 7.1.
   The window renders; it does not order. AC3 pins this with an empty diff over the five non-`gui`
   crates.
3. **No cut-face styling.** 7.1 left the floor of the cut deliberately unstyled and this story does
   not touch it. If the marks turn out to be what finally makes the cut legible, that is a new
   decision for you, not a licence this story inherits.
4. **No per-designation progress, no job state, no "who is working on this".** The wire carries
   none of it — `world.jobs()`, `claims()` and `carrying()` are never called by the bridge. There is
   nothing to render, and inventing a client-side guess would be game logic in a client.
5. **No stockpile outline, extent or fill level.** Zones cross the wire as independent tiles with no
   rect grouping and no id, so a 2×2 stockpile is two tiles that happen to be adjacent, and the
   client cannot know they were one drag.
6. **The marks will NOT be the colours the TUI uses — and this is a deliberate break. RULE IT.**
   The TUI draws dig as an amber `×` (232,176,72) and channel as a blue `▼` (92,174,224). Amber is
   *warm*, and UX-DR5's whole read is that warm means fire and life while the world is cold. Putting
   the TUI's amber on up to 79 rock tiles would drop a field of false firelight into the middle of
   the frame and compete with the lanterns you signed off at 6.2. So gui's dig mark will be **cold
   or neutral**, and the two clients will not agree on colour. The alternative — consistency across
   clients, at the cost of the warm/cold read — is available if you want it, but you cannot have
   both.
7. **Marks obey the cut, so a mark above the slice is hidden** — same rule as entities, items and
   chips. Note this rides on 7.1's AC10 ruling, which you have **not yet given**: if you decide
   surface dwarves should stay visible while you look underground, the marks follow that decision
   and this line changes with it.
8. **Marks vanish as the work is done, and that is the feature, not a bug.** Measured live at story
   creation: an 8×12 rect at z 9 yields 79 marks, and the dwarves eat it down to 68 / 59 / 51 at
   t+40 / t+60 / t+100 ticks, then hold at a stable **50** as the rest become unreachable. If you
   watch the site for a minute you will see the marks thin out.
9. **Not every tile you drag over becomes a mark.** The sim silently drops what it cannot work: 17
   of those 96 cells were not solid rock, and a 2×2 stockpile drag produced **2** zone tiles, not 4,
   because a zone tile must be standable. A stockpile dragged onto solid rock is a total no-op. You
   will see fewer marks than cells you selected, every time.
10. **A dig mark and a zone mark sit one level apart.** A dig is on the rock (z 9); a zone is the air
    tile you stand in, one above it (z 10). A capture pinned to `--z 9` would hide the zone
    entirely, so the recipe pins `--z 10`.
11. **Stone items are now much smaller than when you last watched them.** Not this story's work —
    `STONE_ITEM_SCALE` went to 0.4 on 2026-08-20, fixing the full-size block that made a dug tile
    look refilled. But items are in AC8's four-noun legibility bar, so you are being asked to judge
    "can I tell an item apart" on a version of the item you have effectively not seen at the working
    zoom yet.

## (d) THE DECISION — how a mark is presented. Rule this now.

The story names three candidates: an **overlay slab** on the tile face, a **tinted replacement
material** on the tile, or a **small floating glyph-analogue**. Checked against the code, the choice
is narrower than it looks, because of one fact:

**A dig mark lands on a tile that HAS a cube. A channel mark and a zone mark land on tiles that have
NONE.** Dig requires `Tile::Solid`; channel and zone require `is_standable`, which means *empty at
that position with something solid beneath*. There is no cube at a channel tile or a zone tile to
tint or to stick a decal on.

That kills **tinted replacement** outright — it can only express one of the three kinds, and the
other two would need a different mechanism, so the "one appearance rule, three colours" that AC4 and
AC5 assert would not exist. It also weakens the **face overlay**: for channel and zone the slab has
to attach to the tile *below* the one that is actually marked, which is a lie about which tile you
ordered, and at the cut it would be indistinguishable from styling the cut face — the thing item 3
above says we are not doing.

**Recommendation: a thin slab resting on the floor of the marked tile's own volume, one shared
geometry for all three kinds, distinguished by colour.** Concretely, the `SnowCap` mesh precedent
(`Cuboid::new(1.02, 0.08, 1.02)`, dropped to the tile floor the way `STONE_ITEM_DROP` drops an item).

Why this one:

- **It is the same object in all three cases** — on top of the rock for a dig, on the floor of the
  air tile for a channel or a zone. One mesh, one rule, three colours. That is exactly the shape
  AC4/AC5 test.
- **It reads from above, which is the direction you are looking** when you have cut into the
  mountain. A slab is nearly all of its own footprint from a top-down-ish angle and nearly nothing
  from the side, so it marks the tile strongly in the view that matters and stays out of the way in
  the vista.
- **It does not occlude the dwarves or the items**, both of which stand in the same air tiles a
  zone occupies. A floating glyph-analogue at tile centre would sit exactly where a dwarf walks.
- **It is the honest referent** — chalk on the ground is what a work order on a floor looks like.
- **It costs one mesh handle and three material handles**, and reuses two mechanisms already in the
  codebase rather than inventing a third.

The main thing given up: a slab is a *flat* mark, so at the full vista it will be nearly invisible
(the same sub-legibility 6.1 measured for the dig face at 0.30% of frame). That is accepted
deliberately — this story's bar is the **working zoom**, and the vista is where item 11's "the camp
takes the eye first" has to keep holding.

If you would rather have a mark that reads at the vista too, say so now: it means a taller or
brighter mark, and it puts AC9's warm/cold read directly at risk.

## The defect in the story's own evidence recipe — found before it cost a vehicle session

**AC9 cannot be proved by either command the story's Verification section prescribes.** Both
captures are pinned `--z 10`, and on 2026-08-20 you ruled the warm-pixel and ground-median band be
**scoped to the world top**, because at a cut the sample window shows interior rock rather than
sky-lit snow (that is the fix that stopped z 9 panicking at 67 against the 70 floor on a picture you
confirmed was fine). The code now skips both assertions whenever `slice.level() < slice.top()`.

So a `--z 10` capture **prints** the warm and ground numbers and **asserts neither**. AC9 says the
floor and the range "still hold with marks on screen"; on the story's own recipe nothing would check
that they do, and the run would exit 0 either way. This is the ninth or tenth time an AC's stated
proof has not proved the AC on this project, and it is the same class every time.

**Proposed fix, for your ruling — it is one line of recipe, no code:** take the *vista* capture at
**full depth** rather than at `--z 10`. The marks at z 9 and z 10 are still drawn there (the filter
is "at or below the cut", and the cut is at the top), so marks are genuinely on screen, the band
assertions genuinely run, and AC9 gets real evidence. The working-zoom capture stays at
`--z 10 --distance 30`, where its job is legibility and the mark counts, not the band.

One consequence to accept with it: at full depth the **campfire still reads blown**, which you
carried open past the 08-20 sign-off with the diagnosis recorded (its amplitude went 0.11 → 0.40 at
`04e6de5` and peaks 40% above the value 5.4 was sized against). The vista capture will show that.
It is a known open item, not something this story caused, and this story must not quietly re-tune it
to make a picture pass.

## One defect found while reading the before capture — not this story's, reported not fixed

The slice readout renders as `Slice: z 9/31 ⍰ underground` on the vehicle: the em-dash in
`slice.rs:59` has **no glyph in the loaded font** and draws as an empty box. It is in
`7-1-slice.png`, it will be in every 7.2 capture, and it is a one-character fix (`—` → `-`). It is
not mapped to any task in this story, so it is being reported rather than folded in. Say the word
and it is a separate one-line commit.

## The keys, so you are not told them out of band

In the TUI, from `view.rs:387-424` — `d` dig, `c` channel, `p` stockpile, `x` clear, `hjkl` to move,
`Enter` to anchor a corner and `Enter` again to commit, `<` / `>` to change level. **Note `x` sends
two commands** — cancel designation *and* remove stockpile; there is no cancel-only key.

In the Bevy window, `,` steps the cut down and `.` steps it up, and `Q`/`E` zoom — still
**provisional**, still unruled from 7.1.


---

## AMENDMENT — 2026-08-21, added at code review. Read this before the live viewing.

The approved text above still describes the presentation as it was DESIGNED. Two things about
where a dig slab actually sits changed after approval, and the closing sign-off (AC17) compares the
built result against this document, so they are recorded here rather than left to be discovered at
the viewing. **Neither changes a ruling** — both were made to keep Ruling 1 (the floor slab) and
Ruling 2 (`--z 10 --distance 30` for the working zoom) working as intended.

**1. A dig slab sits on its tile's TOP face, not on its floor.** §(d)'s recommendation says "a thin
slab resting on the floor of the marked tile's own volume"; the bullet under it already says "on
top of the rock for a dig", and that is what shipped — dig `+0.54`, channel and zone `-0.46`. Found
during implementation: a dig tile is SOLID, so a slab on its floor is inside opaque rock. Channel
and zone tiles are air and keep the floor placement exactly as described.

**2. A dig with rock above it is drawn on the top face of the rock that covers it.** This one was
found at code review, by measurement, and it is the difference between the working-zoom capture
showing your dig orders and showing none of them.

The slice draws every solid tile *at the cut* as a full cube, whether or not it is exposed. That
cube spans the half-tile above and below its own level — so a dig slab one level beneath it was
sealed inside opaque geometry. It is not a rare case: the dwarves dig the REACHABLE tiles first,
and reachable means open sky above, so the marks that survive long enough to be photographed are
exactly the buried ones. Measured on this story's own recipe, against a live daemon:

| tick | marks on the wire | dig slabs actually visible |
|---|---|---|
| t+2 | 79 | 25 |
| t+46 | 63 | 9 |
| t+64 | 54 | 2 |
| t+102 | 50 | **0** |
| t+165 | 50 | **0** |

The instrument printed `designations=50` and exited 0 throughout, correctly — all 50 *were*
projected. You would have been asked to rule on AC8 ("can I tell a designation apart") using a
frame with no designations in it.

**What you will see instead:** a dig under rock is drawn as a slab on the surface above it, in dig
blue. A dig with open sky above it is unchanged, on its own tile — it is already visible, and
hoisting it would put the mark on rock it does not mark.

**What to check at the viewing, because of this:** that the mark field reads as *orders on the
mountainside* and not as a blue sheet laid over the terrain. This is the one change here that could
plausibly be wrong in a way only your eye will catch.

**3. The mark colours were retuned** (all three still cold-or-neutral, so Ruling 3 and UX-DR5 are
untouched). Dig had shipped BYTE-IDENTICAL to the TUI's CHANNEL blue — Ruling 3 says the two
clients break with each other deliberately, but landing exactly on a *different* order's colour in
the other window was not deliberate. Dig and channel were also two blues separated almost entirely
on the green axis, which the cool directional light compresses. They now separate on red and sit
103 units apart unlit, against 51 before; every mark is now at least 40 from every terrain colour
AND from every TUI mark colour.

**These colours have never been seen rendered.** No devpod here can open a window. Everything
above is computed geometry and colour arithmetic, and the whole point of your viewing is that it
is not evidence.
