# 12.6 look draft: roster, selection and changing a trade (for Wolf's approval, Task 0)

**APPROVED by Wolf as drafted, 2026-10-05.** Task 0.2 ruled that an emptied trade is allowed, so the
last-of-trade line in §4 is VOID: only `trade refused: no such dwarf` exists.

A cheap text mock (standing AC 7). It shows what you will see. Nothing here is built yet. The
positions are today's HUD slots, measured from `crates/gui/src/ingest.rs` on `98149e1`.

## 1. Nothing selected: the crew roster (top-right, under the clock)

Today this slot (`NameReadout`, top 48 px, right 16 px, 22 px text) is empty when nothing is
selected. It becomes the roster: one dwarf per line, his name in his tunic colour and his trade
in the grey the tui uses (150,160,170), in id order, like the tui roster row.

```
                                        06:12   elapsed 0d 02:03   speed normal
                                                            Nain  woodcutter
                                                            Ori   hauler
                                                            Bifur miner
                                                            Frar  hauler
                                                            Dori  miner
```

(DEFAULT_SEED's crew. Nain purple, Ori green, Bifur red, Frar gold, Dori blue.)

## 2. A dwarf selected: his name, his trade, and the key

Click a dwarf (or `--select 2`). The roster gives way to the selected dwarf. Line one is his name
in his tunic colour, as today. Line two is his trade and the key, in grey.

```
                                        06:12   elapsed 0d 02:03   speed normal
                                                       Bifur
                                                       miner   T: change trade
```

## 3. Changing the trade

`T` sends one command: the next trade in the order miner -> hauler -> woodcutter -> miner. The
line changes when the daemon's next delta says so, not when the key is pressed, so what you read
is the world, not the request. An attached tui roster changes on the same delta.

```
                                                       Bifur
                                                       hauler   T: change trade
```

`T` is free today (`the_client_keymap_avoids_keys_other_plugins_have_claimed`). Letter keys per
trade do not work, because `H` is the HUD toggle. Escape still deselects and brings the roster
back.

## 4. A refused change (bottom-left, where a refused stockpile shows today)

Same slot, colour and clearing rule as 12.1's stockpile refusal: it stays until your next world
command.

```
trade refused: last of his trade
```

(A fixed line, like the stockpile's: `client_core::refusal_text` returns a `&'static str`, so it
names neither the dwarf nor the trade. Naming them would mean changing that function for both
clients. Say so if you want it.)

That line exists only if you pick option 1 or 2 for the last-of-a-trade question in Task 0. The
other refusal, for an unknown dwarf id, cannot be produced from the gui (you can only select a
dwarf that exists). Only a test client produces it:

```
trade refused: no such dwarf
```

## Not in this draft

- No buttons or mouse menu: one key.
- No change to the tui (NFR10: no new tui input). Its roster already shows trades, and it follows
  a change on the next delta.
- The capture HUD is still hidden for every `--capture`.
