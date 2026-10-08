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
old = '    depth.retain(|cell, _| is_walkable(terrain, blocked, *cell));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    depth.retain(|cell, _| terrain.is_standable(*cell));\n'))
PY

mutation "daemon discards stockpile refusal" simd the_daemon_keeps_channels_and_stockpiles_only_at_standable_cells <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                    refusals.extend(world.place_stockpile(&rects).map(bridge::refusal_out));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    refusals.extend(world.place_stockpile(&rects).map(bridge::refusal_out).filter(|_| false));\n'))
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
# Re-pointed 2026-10-05 (12.7): the item layer now picks the cell by kind and stored-ness in one
# `match`, so the seam is the stone-on-a-stockpile arm.
old = '                (protocol::ItemKind::Stone, true) => stored_item_cell(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                (protocol::ItemKind::Stone, true) => item_cell(),\n'))
PY

# Review patch pass (2026-09-29).

mutation "a drag is refused when any rect is" sim-core a_drag_is_refused_only_when_none_of_its_rects_has_a_valid_cell <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        (refused == rects.len()).then(|| Refusal::PlaceStockpile {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        (refused > 0).then(|| Refusal::PlaceStockpile {\n'))
PY

mutation "gui sends one stockpile command per rect" gui a_stockpile_drag_over_several_rects_is_one_command <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '        DesignateMode::Stockpile => vec![Command::PlaceStockpile {\n            rects: surface.to_vec(),\n        }],\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        DesignateMode::Stockpile => surface.iter().map(|rect| Command::PlaceStockpile { rects: vec![*rect] }).collect(),\n'))
PY

mutation "drop search walks through rock" sim-core release_claim_drops_where_the_carrier_can_walk <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
check = '                        if !occupied.contains(&cell) {\n'
walk = '                        for candidate in astar_neighbours(terrain, &blocked, cell) {\n'
assert s.count(check) == 1 and s.count(walk) == 1
s = s.replace(check, '                        if is_walkable(terrain, &blocked, cell) && !occupied.contains(&cell) {\n')
p.write_text(s.replace(walk, '                        for candidate in [(-1, 0, 0), (1, 0, 0), (0, -1, 0), (0, 1, 0), (0, 0, -1), (0, 0, 1)].map(|(dx, dy, dz)| Pos { x: cell.x + dx, y: cell.y + dy, z: cell.z + dz }).into_iter().filter(|candidate| terrain.tile(*candidate).is_some()) {\n'))
PY

mutation "campfire pile never fills" sim-core a_stockpile_around_the_campfire_never_zones_or_receives_the_fire <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    depth.retain(|cell, _| is_walkable(terrain, blocked, *cell));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    depth.retain(|_, _| false);\n'))
PY

mutation "tui refusal never clears" tui a_refusal_stays_until_this_client_sends_a_world_command <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '        state.refusal = None;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
