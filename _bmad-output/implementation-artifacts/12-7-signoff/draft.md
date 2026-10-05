# 12.7 look draft: cut marks, felling and wood (for Wolf's approval, Task 0)

A cheap text mock (standing AC 7). It shows what you will see; nothing here is built yet. Keys,
slots and colours are today's, measured from `crates/gui/src/designate.rs`, `appearance.rs` and
`crates/tui/src/palette.rs` on `fd9ca98`.

## 1. Marking trees: key `5`, drag like a dig (gui)

`5` arms cut mode, next to the four modes you have. The hint bar becomes:

```
1 dig  2 channel  3 stockpile  4 clear  5 cut   Space pause  +/- speed  Ctrl+S save  Ctrl+L load
```

and in cut mode: `cut: drag over the foot of the trees`, then `release to mark  Esc abort`.

Drag across the foot of a stand of pines, like a dig drag: one rectangle on one level. The level is
the one just above the ground you pointed at (pointing at a trunk uses the trunk's own level). Every
tree with a tile in that rectangle is marked. The preview lights the trunk cells it will catch,
before you let go. On a hillside, trees standing two levels or more above where you pointed are not
caught: drag again higher up, as you would for a dig.

A marked tree shows **one mark at the foot of its trunk**: the same flat slab as a dig mark, in a
new cold colour (a cyan-green; the exact literal is whatever passes the mark-colour test, at least
40 away from the dig blue, the channel violet and the zone teal):

```
   ♣♣♣          ♣♣♣              (crowns, seen from the boot camera)
   ♣│♣          ♣│♣
    │            │
  [▬▬▬]          │               [▬▬▬] = cut mark slab around the trunk's foot
 ─────────────────────── ground
   marked       not marked
```

`4` (clear) removes cut marks too: a clear drag over the foot of a marked tree removes its mark, and
a woodcutter on his way to it lets go.

A drag that catches no tree at all is refused in the bottom-left slot, where a refused stockpile
shows today:

```
cut refused: no tree
```

## 2. Felling (gui and tui)

The woodcutter walks to the foot of the tree and works it (10 s at normal speed, like a dig). When he
finishes, **the whole pine is gone on that tick**: trunk and crown together. A pine whose crown
touched it stays standing. No falling animation.

While he works (12.8 makes the real cut clip, so pick one):
- **(a, recommended) he plays the dig swing, facing the trunk**, a placeholder until 12.8. It reads
  as work, and `gui dwarf N clip dig` prints for it;
- (b) he stands still, which reads as idle.

## 3. Wood (gui)

The pine leaves wood at the foot of its trunk: **a log**, a brown box about 0.7 × 0.28 × 0.28 of a
cell, lying on the ground where the trunk stood. A hauler carries it exactly as he carries a stone:
the Carry clip, held at the same offset. On a stockpile it lies like a stone does.

How many logs a pine gives is Task 0's question 2. With one log per pine, nothing stacks. With one
per trunk cell (3 to 5), the logs share the foot cell, and the gui draws them stacked so you can
count them.

## 4. The tui (display only, no new key)

| what | glyph | colour |
| --- | --- | --- |
| cut mark (on the trunk's foot, on the walking level) | `/` | orange-red (226, 96, 64) |
| a log on the ground | `=` | wood brown (164, 116, 66) |
| a log on a stockpile cell | `=` | stockpile green, like a stored stone |

They are ASCII on purpose: `♠`, `♨` and `☺` already draw as wide emoji in Windows Terminal (#152).
The `marks:` line of `tui --frame` counts cut marks with the dig and channel marks.

Before and after, viewed at the trunk's foot level with `--z`:

```
 . . . . .         . . . . .
 . . / . .   -->   . . = . .
 . . . . .         . . . . .
```

## Not in this draft

- No cut animation (12.8) and no falling tree (12.11).
- A dig or channel drag no longer takes tree tiles. That has no look, only a refusal line (Task 0's
  question 3).
- No change to stones.
