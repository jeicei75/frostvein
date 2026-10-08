# 12.8 look draft: cut mode (#174), for Wolf's approval (Task 0)

**APPROVED by Wolf 2026-10-07** (Task 0.1): §1 and §2 as drafted, §3 option (a), the foot slab kept. This is a cheap text mock (standing AC 7). It covers only #174's look items. The cut
swing is a work animation, so it skips the draft and goes straight to the seat. Nothing here is built
yet. Colours and keys are today's, measured from `crates/gui/src/appearance.rs`, `designate.rs` and
`project.rs` on `486095d`:

| Colour | sRGB | Defined at |
| --- | --- | --- |
| hover | (80, 220, 210) | `appearance.rs:224` |
| cut mark | (30, 190, 150) | `appearance.rs:208` |
| tree foliage, the ghost tree's colour | (44, 100, 58) | `appearance.rs:351` |

## 1. Pointing at a tree in cut mode (the flicker)

**Today.** The pointer ignores crowns: a ray through a crown lands on the ground 8-12 cells BEHIND the
tree. As you sweep across one pine, the highlight hops between the trunk's side and that far ground. Your
video shows it.

**In cut mode, the pointer stops at the tree.** Point at any part of a pine, crown or trunk, and the
highlight sits at that pine's foot, where its cut mark goes. Between trees it sits on the ground you point
at, as now.

```
   ♣♣♣      ←  cursor anywhere on the crown
   ♣│♣
    │
  [▒▒▒]     ←  hover slab at this pine's foot, not on the ground behind it
 ─────────────────────────
```

Dig, channel, stockpile and clear keep today's pointer: you still dig the ground seen through a crown.

## 2. Dragging a cut area: the whole box shows

**Today.** The preview lights only the tree cells the drag catches, so over open ground you see nothing
and cannot tell how big your box is.

**While you drag**, every cell of the box shows a thin hover-coloured slab on its ground. The pines the box
catches are already tinted, as in §3, so you see which trees you are about to mark. **On release**, only
those trees are marked. The box disappears and the tinted trees stay tinted. The sim already marks only
trees, so the order sent does not change.

```
   drag:   [▒][▒][♣][▒][▒]          ▒ = hover slab on each box cell
           [▒][▒][▒][▒][♣]          ♣ = a caught pine, tinted green
   release:        ♣       ♣        only the trees stay marked (tinted)
```

A refused cut, a box over no tree, still shows `cut refused: no tree, or the mark limit is reached`.

## 3. A marked tree is green all over

**Today.** A marked pine looks like every other pine, apart from a small slab at its foot.

**Proposed.** A marked pine is tinted green all over, like the ghost tree in your video. Three ways to do
it:

- **(a, recommended) Repaint the pine model green.** It keeps the pine's shape and voxel texture, but in
  a flat foliage green with no snow, much like the ghost. 12.2 recoloured the dwarves' tunics the same
  way, so this is a known path. Clearing the mark, or felling the tree, returns it to snowy.
- **(b) A green glow over the normal pine.** The snow stays. Glow reads weaker with bloom and haze on
  (11.2-11.3 measured this), so it may be subtle at the seat.
- **(c) Green cubes over the tree's cells, exactly the ghost look.** The pine's shape disappears behind
  blocks.

**The slab at the foot**: keep it (recommended), because it shows where to drag `4` to clear the mark. Or
drop it, since the green tree says enough.

## Not in this draft

- **The cut swing (the animation):** made at the Blender seat; no draft, by the M3 rule.
- **#164 (work shown before the dwarf arrives):** a timing fix with no new look; judged at the seat.
- **#173 (the ghost tree):** a defect, not yet reproduced. The green tint in §3 is deliberate and comes only
  from a cut mark; the ghost was a marked pine drawn green by something that is not this.
- **The tui:** unchanged. It shows the cut mark as `/` and gets no cut key (NFR10).
