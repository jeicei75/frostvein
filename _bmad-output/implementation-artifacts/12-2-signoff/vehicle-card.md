# Story 12.2 seat card — five dwarves you can name

Launch the daemon in WSL with `simd 7451`. In PowerShell from the Windows checkout, launch the client with `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`. Attach a second client in WSL with `tui 7451`.

**Old saves will not load.** Dwarves now carry a name and colour, so a `frostvein.save` from before this story is refused. Press Ctrl+S in the new build first if you want one.

On the default seed the five are (id, name, tunic):
Nain purple, Ori green, Bifur red, Frar gold, Dori blue. Hexes: red `#B23A34`, gold `#D6A42C`, green `#3E924C`, blue `#3C62BA`, purple `#804CA8`.

(a) At your usual working zoom, name each dwarf by his tunic colour, without clicking. Then click each one: the HUD line (top-right, just under the clock) shows his name in his tunic colour (`Nain` purple, `Ori` green, `Bifur` red, `Frar` gold, `Dori` blue). Does the colour you guessed match the name shown?

(b) Select one dwarf, then wheel in to the closest zoom. He should stay sharp the whole way, head to boots (#136).

(c) Press Escape. The name line empties and the depth of field goes back to the camp-focused look (sharp camp, soft far ridge).

(d) Look at the attached tui. The row above `tick` lists `Nain  Ori  Bifur  Frar  Dori`, each in the same colour as his gui tunic, ordered by id. Each named dwarf is a `☻` on the map in his tunic colour, whether or not he carries a stone (Task 9, Wolf's rulings of 2026-09-30 and 2026-10-01: Windows Terminal paints `☺` as a yellow emoji); a `⚇` crowd keeps its own colour.

Wolf: can you tell the five apart at a glance, and is he sharp at the closest zoom?
