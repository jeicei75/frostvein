# Story 12.7: Timber. A `cut` designation marks one whole tree at its trunk's base; a woodcutter
# fells it (trunk column + the TreeFoliage in the 3x3 column around it, base ..= top + 1) and leaves
# one wood item per trunk cell. Dig and channel never take tree tiles; a zero-cell dig, channel or
# cut rect is refused. Items carry a kind on the wire and in the save. Run alone, after commit:
#   RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-7.sh
# Row numbers follow the story's Task 6 list. Row 3's daemon-side RED on the live recipe is a
# scratch-worktree release build, recorded in the story, not a row here.

mutation "1 the cut does nothing" sim-core cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if let Some((base, tiles)) = tree {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if let Some((base, tiles)) = tree.filter(|_| false) {\n'))
PY

mutation "2 the cut removes the trunk column only" sim-core cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                for tile in &tiles {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                for tile in tiles.iter().filter(|t| t.x == base.x && t.y == base.y) {\n'))
PY

mutation "3 the crown box is 5x5 and takes the touching neighbour's crown" sim-core cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        for fy in y - 1..=y + 1 {
            for fx in x - 1..=x + 1 {
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''        for fy in y - 2..=y + 2 {
            for fx in x - 2..=x + 2 {
'''))
PY

mutation "4 a cut is a miner's job" sim-core cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        JobKind::Cut => Profession::Woodcutter,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        JobKind::Cut => Profession::Miner,\n'))
PY

mutation "5 the wood spawns as stone" sim-core cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    ecs.spawn((Item(ItemKind::Wood), item_id, base));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    ecs.spawn((Item(ItemKind::Stone), item_id, base));\n'))
PY

mutation "5 (serve) the wood spawns as stone" simd a_cut_fells_one_whole_tree_into_wood_and_a_cut_over_nothing_is_refused <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    ecs.spawn((Item(ItemKind::Wood), item_id, base));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    ecs.spawn((Item(ItemKind::Stone), item_id, base));\n'))
PY

mutation "5b one log per tree, not per trunk cell" sim-core cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                for _ in 0..trunk_cells {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                for _ in 0..trunk_cells.min(1) {\n'))
PY

mutation "6 the dig filter takes tree tiles again" sim-core a_dig_mark_never_lands_on_a_tree_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                                    matches!(terrain.tile(*pos), Some(Tile::Solid(_)))
                                        && !is_tree(*pos)
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''                                    matches!(terrain.tile(*pos), Some(Tile::Solid(_)))
'''))
PY

mutation "7 the channel filter takes cells standing on trees" sim-core a_channel_mark_never_lands_on_a_cell_standing_on_a_tree_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                                    terrain.is_standable(*pos)
                                        && !is_tree(Pos {
                                            z: pos.z - 1,
                                            ..*pos
                                        })
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''                                    terrain.is_standable(*pos)
'''))
PY

mutation "8 a cut over no tree is not refused" sim-core a_cut_over_no_tree_is_refused_and_a_dig_never_marks_a_tree <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                if applied == 0 {
                    return Some(Refusal::Designate { kind, rect });
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''                if applied == 0 && kind != DesignationKind::Cut {
                    return Some(Refusal::Designate { kind, rect });
'''))
PY

mutation "8 (serve) a cut over no tree is not refused" simd a_cut_fells_one_whole_tree_into_wood_and_a_cut_over_nothing_is_refused <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                if applied == 0 {
                    return Some(Refusal::Designate { kind, rect });
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''                if applied == 0 && kind != DesignationKind::Cut {
                    return Some(Refusal::Designate { kind, rect });
'''))
PY

mutation "9 cancel leaves the cut job" sim-core a_cancel_touching_a_marked_tree_removes_its_mark_and_releases_the_woodcutter <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        matches!(job.kind, JobKind::Dig | JobKind::Channel | JobKind::Cut)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        matches!(job.kind, JobKind::Dig | JobKind::Channel)\n'))
PY

mutation "10 the bridge sends wood as stone" simd a_cut_fells_one_whole_tree_into_wood_and_a_cut_over_nothing_is_refused <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '        sim_core::ItemKind::Wood => protocol::ItemKind::Wood,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        sim_core::ItemKind::Wood => protocol::ItemKind::Stone,\n'))
PY

mutation "11 the save drops the kind (wood reloads as stone)" sim-core a_world_saved_mid_cut_with_wood_on_the_ground_loads_and_steps_identically <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            world.ecs.spawn((Item(kind), Id(id), pos));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let _ = kind;\n            world.ecs.spawn((Item(ItemKind::Stone), Id(id), pos));\n'))
PY

mutation "12 gui cut mode sends dig" gui a_cut_drag_over_a_trunk_writes_one_designate_cut_at_the_trunks_level <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '            kind: DesignationKind::Cut,\n            rect: cut_rect,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            kind: DesignationKind::Dig,\n            rect: cut_rect,\n'))
PY

mutation "13 gui clear omits the cut-target rect" gui a_clear_drag_at_a_trees_foot_writes_a_cancel_that_holds_the_base <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '''            .chain(std::iter::once(Command::CancelDesignation {
                rect: cut_rect,
            }))
'''
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "14 gui wood draws as a stone" gui a_wood_item_projects_as_a_log_and_a_stone_still_as_a_stone <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        ItemKind::Wood => WOOD_ITEM_SCALE,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        ItemKind::Wood => Vec3::splat(STONE_ITEM_SCALE),\n'))
PY

mutation "14b gui items sharing a cell all draw at the same height" gui four_logs_on_one_cell_draw_at_four_distinct_heights <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            (item.id, *below - 1)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            (item.id, 0)\n'))
PY

mutation "15 tui wood uses the stone glyph" tui a_wood_item_draws_as_a_log_and_a_stored_one_in_the_stockpile_colour <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '                (protocol::ItemKind::Wood, false) => wood_item_cell(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                (protocol::ItemKind::Wood, false) => item_cell(),\n'))
PY

# Beyond the story's list: Task 0.1's placeholder clip has a test, so it gets a row.
mutation "16 a woodcutter working a cut walks" gui a_woodcutter_on_a_cut_job_in_work_gets_the_dig_clip <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '''                    | protocol::DwarfJob::Channel { .. }
                    | protocol::DwarfJob::Cut { .. }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''                    | protocol::DwarfJob::Channel { .. }
'''))
PY

# #168, found at the seat (AC12): a felled pine left dig-debris chips hanging where its crown was.
mutation "17 debris spawns over air again" gui a_felled_tree_leaves_debris_at_its_foot_and_none_in_the_air <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                && matches!(mirror.tile(below), Some(Tile::Solid(_) | Tile::Ramp(_)))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                && (below[2] < 0 || true)\n'))
PY

# Review patch pass (2026-10-06). D1: a full mark cap refuses with these texts, so they name it.
mutation "18 the cut refusal forgets the mark cap" client-core every_refusal_has_its_one_text <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '            protocol::DesignationKind::Cut => "cut refused: no tree, or the mark limit is reached",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            protocol::DesignationKind::Cut => "cut refused: no tree",\n'))
PY

# items() drops an item with no Pos while item_kinds() keeps it; a zip shifts every later kind.
mutation "19 the bridge zips items to kinds by position" simd an_item_without_a_position_does_not_shift_later_kinds <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '            kind: item_kind_out(kinds[&id]),\n'
assert s.count(old) == 1
s = s.replace(old, '            kind: item_kind_out(zipped.next().unwrap()),\n')
old = '    let kinds: std::collections::BTreeMap<_, _> = kinds.into_iter().collect();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let mut zipped = kinds.into_iter().map(|(_, kind)| kind);\n'))
PY
