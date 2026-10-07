# Story 12.8: Dwarves at work, the cut. #164: a hauler's item is lifted only once his drawn body
# reaches its cell, and released only once he is drawn at the cell the wire drops it on; the drawn
# walker keeps pace with the sim speed. Run alone, after commit:
#   RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-8.sh
# Row names follow the story's Task 6 list.

mutation "pick-up ungated" gui a_hauler_does_not_pick_up_his_stone_until_he_is_drawn_at_its_cell <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                    && !walking_in.contains_key(&dwarf) =>\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    && (true || !walking_in.contains_key(&dwarf)) =>\n'))
PY

mutation "Fast walker unscaled" gui a_dwarf_walking_in_at_fast_reaches_his_work_clip_within_the_run <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        protocol::Speed::Fast => 5.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        protocol::Speed::Fast => 1.0,\n'))
PY
