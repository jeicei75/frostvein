# 12.10 look draft: ore in the rock (for Wolf's approval, Task 0)

**APPROVED by Wolf, 2026-10-11, at creation:** §1 option (b)'s colours under Dwarf Fortress names,
tetrahedrite and bloodstone (Wolf: "let's use dwarf fortress ores and gems"); §2-§3 as drafted.
The names are flavour, the colours are (b)'s: real tetrahedrite is steel-grey and bloodstone dark
green, and Wolf kept (b) knowing that.

A cheap text mock (standing AC 7). It shows what you will see; nothing here is built yet. Today's
colours are measured from `crates/gui/src/appearance.rs` and `crates/tui/src/palette.rs` on
`32565fd`. The proposed literals are starting points: the exact value is whatever passes the
palette tests (at least 40 from every terrain colour and every mark colour, `appearance.rs:842`).

## 1. Which ores (both clients)

One mineral and one gem (FR46 allows one or two minerals; one keeps 12.12's "collect N minerals"
to a single kind). Gems are rarer and deeper.

- **Tetrahedrite** (the mineral) is a deep saturated blue; **bloodstone** (the gem) is a pale
  green-teal. Both keep the rule that every terrain colour stays blue at or above red
  (`docs/tech-art-guidelines.md:179`, asserted at `appearance.rs:602-605`).
- Not taken: (a) a warm rust-red iron. It broke the blue-at-or-above-red rule and blurred the
  warm-lit capture check (`capture.rs:583-593`) in torch-lit tunnels.

| | gui colour (rock and item) | tui in the rock | tui item |
|---|---|---|---|
| stone (today) | (60,70,92) | `█` (86,92,104) | `*` (176,172,160) |
| **tetrahedrite** | (40,64,176) | `%` (60,90,210) | `•` (60,90,210) |
| **bloodstone** | (120,196,176) | `$` (130,220,190) | `◊` (130,220,190) |

Stored on a pile, an ore item keeps its glyph in the zone green, as stone and wood do. The glyphs
are narrow on purpose: `♠ ♨ ☺` draw as wide emoji in Windows Terminal (#152). If `◊` draws wide at
the seat, it falls back to `'`. Not `o` for tetrahedrite: the status and hint rows already hold six, so a
glyph count could never fail.

## 2. How ore reads in the rock

Ore is **inside** the rock, never on the surface: on a fresh world you see no ore at all. You find
it three ways:

- **tui:** step `--z` / `<` `>` down into a hill. A level inside the rock shows the cross-section:

```
  █ █ █ █ █ % % █ █ █          █ = stone
  █ █ █ % % % █ █ █ █          % = tetrahedrite vein (deep blue)
  █ █ █ █ % █ █ █ $ █          $ = a bloodstone (pale teal), alone, deeper
  ▓ ▓ █ █ █ █ █ █ █ █          ▓ = soil, near the hillside
```

- **gui slice:** lower the slice into the hill. The cut face shows tetrahedrite veins and lone bloodstones
  in their own colours on the grey-blue stone.
- **gui, digging:** a tunnel's walls show any ore it passes, the same colours on the wall faces.

Tetrahedrite comes in **veins**: winding runs of a few to a dozen cells. Bloodstone comes as **single cells**,
only well below the surface (5 or more levels down), about one for every twenty tetrahedrite cells.

## 3. Digging it (gui and tui)

You dig ore like stone: the same `1` dig mark, the same 10 s at normal speed. When the miner
finishes, **a tetrahedrite item (or a bloodstone) lands on the cell, not a stone**, and a hauler carries it to
the pile like a stone. In the gui an ore item is the stone's small cube in the ore's colour; nothing
glows.

**Hillsides now take a dig mark on their slope cells (the ramps).** Today every hill is ringed by
ramps that refuse a dig, so no tunnel ever gets under a hill and the ore would be out of reach. With
this story, `1` on a slope digs it out like any rock; the preview lights it before you let go. All
diggable ground digs the same way, hillsides included (Wolf, 2026-10-11).

## Not in this draft

- Using ore for anything (M3 out of scope).
- Goals that count ore (12.12).
- A second mineral, ore that glows, ore in soil or on the surface.
- Channelling down a level (#163): its own story, 12.10b, right after this one (Wolf, 2026-10-11).
