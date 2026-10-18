# 12.12 look draft: the goal display and the chain announcement (for Wolf's approval, Task 0)

**APPROVED by Wolf as drafted, option (A), 2026-10-18** ("2 ok"). Targets ruled 10 / 10 / 1 (Q1). No banner.

A cheap text mock (standing AC 7). Nothing here is built yet. Slots are today's HUD slots, measured
from `crates/gui/src/ingest.rs`, `crates/gui/src/designate.rs` and `crates/tui/src/view.rs` on
`60955ef`. The targets shown are the recommended ones (Task 0 Q1: 10 wood, 10 tetrahedrite, 1
bloodstone); whatever Q1 rules is what the lines print.

## 1. gui: a goal list, bottom-right (a free slot today)

Bottom 16 px, right 16 px, the HUD's 22 px text (the default font is monospaced, so the columns
line up). It carries `Hud`, so `H` hides it and every `--capture` hides it, as it does the other
HUD text. Always three goal lines, in chain order, under a title line:

```
                                                    goals
                                                    [x] wood          10 / 10
                                                    [ ] tetrahedrite   3 / 10
                                                    [ ] bloodstone     0 / 1
```

- A **completed** goal: `[x]`, in the stockpile green the tui uses (88,190,118).
- The **current** goal: `[ ]`, in white.
- A **later** goal: `[ ]`, in the trade grey (150,160,170). Its count is live too: stock that is
  already on the pile counts when that goal comes up, so a later goal can read `1 / 1` in grey
  before it is its turn. It ticks when the goal ahead of it is done.
- The count is the daemon's `progress`, the items of that kind lying on a pile right now. If
  stock leaves the pile after a goal is done, the count drops and the `[x]` stays.

When a goal completes, its line turns green and gets its `[x]` on the next delta. That change is
how you see a goal complete.

## 2. gui: the chain announcement

When the last goal completes, the title line changes and turns gold (the Gold tunic, #D6A42C). The
list stays, play goes on, and nothing else changes:

```
                                                    all goals done - play on
                                                    [x] wood          12 / 10
                                                    [x] tetrahedrite  10 / 10
                                                    [x] bloodstone     1 / 1
```

## 3. tui: one new text row, above the roster

The tui has three rows below the map today (roster, status, hint), and the status row is already
near full when a refusal shows (`client-core/src/lib.rs:14-16`). So the goal gets a fourth row,
above the roster, and the map loses one row. Grey text, with the done part in the pile green:

```
goal 2 of 3: tetrahedrite 3/10   done: wood 10/10
Nain woodcutter  Ori hauler  Bifur miner  Frar hauler  Dori miner
tick 4211  fast4x  z 9/31  dwarves 5  N up
d dig  c channel  p stockpile  x clear  <> z  hjkl move  q quit client
```

The first goal, before anything is done:

```
goal 1 of 3: wood 4/10
```

After the chain (at most 72 columns, so it fits an 80-column terminal):

```
all goals done - play on: wood 12/10  tetrahedrite 10/10  bloodstone 1/1
```

No new tui key (NFR10: the tui gains no input in M3).

## The options

- **(A) Recommended: as drafted above.** The display is the state, so there is nothing to time
  and nothing to dismiss. "You see a goal complete" is the line turning green with its `[x]`, and
  "the game says so" is the gold title.
- **(B) As (A), plus a banner.** On each completion, a centred gui line such as
  `goal done: 10 wood` for about 5 seconds, and `all goals done - play on` for the last one. It is
  timed in the client (presentation only, no game logic), and it is more noticeable. The cost is a
  client-local timer, plus one more HUD text to keep out of captures.

## Not in this draft

- No sound, no pop-up and no pause: play never stops.
- No goal UI input of any kind. The goals cannot be skipped, reset or reordered.
- No reward. Resources are only collected and counted (PRD non-goal).
