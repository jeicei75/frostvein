# Story 12.1: emitter filtering and refusal visibility. Run alone after commit.

mutation "stockpile keeps emitter zones" sim-core a_stockpile_around_the_campfire_never_zones_or_receives_the_fire <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        .filter(|pos| is_walkable(terrain, &blocked, *pos))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        .filter(|pos| terrain.is_standable(*pos))\n'))
PY

mutation "old save haul goals include emitters" sim-core an_old_save_zone_on_an_emitter_is_no_haul_goal <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                .filter(|pos| is_walkable(terrain, blocked, *pos) && !stored.contains(pos))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .filter(|pos| terrain.is_standable(*pos) && !stored.contains(pos))\n'))
PY

mutation "daemon discards stockpile refusal" simd the_daemon_keeps_channels_and_stockpiles_only_at_standable_cells <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                            .map(bridge::refusal_out),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                            .map(bridge::refusal_out).filter(|_| false),\n'))
PY

mutation "tui omits refusal status" tui streamed_refusal_stays_on_the_status_row_across_plain_deltas <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '        if let Some(refusal) = &state.refusal {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if let Some(refusal) = &Option::<protocol::Refusal>::None {\n'))
PY

mutation "gui drops last refusal" gui refusal_hud_follows_wire_and_clears_on_the_next_world_command <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '                    last_refusal.0 = Some(*refusal);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    last_refusal.0 = None;\n'))
PY

mutation "retry drop stacks a full stockpile" sim-core a_full_stockpile_never_stacks_uncarried_stones <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            let drop_pos = if occupied.contains(&pos) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let drop_pos = if false {\n'))
PY

mutation "release drop stacks an occupied stockpile" sim-core release_claim_avoids_an_occupied_stockpile_cell <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            let drop_pos = if occupied.contains(&pos) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let drop_pos = if false {\n'))
PY

mutation "tui draws stored stones grey" tui a_stone_on_a_stockpile_cell_draws_in_the_stockpile_colour <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '                stored_item_cell()\n            } else {\n                item_cell()\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                item_cell()\n            } else {\n                item_cell()\n'))
PY
