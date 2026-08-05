# Mutation set for story 3.1. Run:
# scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/3-1-give-the-order.sh

mutation "apply_command skips rectangle normalization" sim-core reversed_rect_designates_the_normalized_inclusive_tiles <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        let min = Pos {
            x: rect.min.x.min(rect.max.x),
            y: rect.min.y.min(rect.max.y),
            z: rect.min.z.min(rect.max.z),
        };
        let max = Pos {
            x: rect.min.x.max(rect.max.x),
            y: rect.min.y.max(rect.max.y),
            z: rect.min.z.max(rect.max.z),
        };
'''
assert old in s
p.write_text(s.replace(old, '        let min = rect.min;\n        let max = rect.max;\n'))
PY

mutation "apply_command skips bounds clipping" sim-core designation_rect_clips_to_world_bounds <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        let min = Pos {
            x: min.x.max(0),
            y: min.y.max(0),
            z: min.z.max(0),
        };
        let max = Pos {
            x: max.x.min(dims.x as i32 - 1),
            y: max.y.min(dims.y as i32 - 1),
            z: max.z.min(dims.z as i32 - 1),
        };
'''
assert old in s
p.write_text(s.replace(old, '        let min = min;\n        let max = max;\n'))
PY

mutation "PlaceStockpile ignores is_standable" sim-core stockpile_keeps_exactly_the_standable_tiles <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                    positions()
                        .filter(|pos| terrain.is_standable(*pos))
                        .collect()
'''
assert old in s
p.write_text(s.replace(old, '                    positions().collect()\n'))
PY

mutation "CancelDesignation also clears zones" sim-core each_eraser_leaves_the_other_mark_kind_untouched <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                for pos in positions() {
                    designations.0.remove(&pos);
                }
            }
'''
new = '''                for pos in positions() {
                    designations.0.remove(&pos);
                }
                drop(designations);
                self.ecs.resource_mut::<Zones>().0.clear();
            }
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "RemoveStockpile also clears designations" sim-core each_eraser_leaves_the_other_mark_kind_untouched <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                for pos in positions() {
                    zones.0.remove(&pos);
                }
            }
'''
new = '''                for pos in positions() {
                    zones.0.remove(&pos);
                }
                drop(zones);
                self.ecs.resource_mut::<Designations>().0.clear();
            }
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "RemoveStockpile is a no-op" sim-core each_eraser_leaves_the_other_mark_kind_untouched <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''            SimCommand::RemoveStockpile { .. } => {
                let mut zones = self.ecs.resource_mut::<Zones>();
                for pos in positions() {
                    zones.0.remove(&pos);
                }
            }
'''
assert old in s
p.write_text(s.replace(old, '            SimCommand::RemoveStockpile { .. } => {}\n'))
PY

mutation "Designate refuses to overwrite an existing kind" sim-core designate_overwrites_the_existing_kind <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    designations.0.insert(pos, kind);\n'
assert old in s
p.write_text(s.replace(old, '                    designations.0.entry(pos).or_insert(kind);\n'))
PY

mutation "to_save drops designations" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            designations: self.designations(),\n'
assert old in s
p.write_text(s.replace(old, '            designations: Vec::new(),\n'))
PY

mutation "to_save drops zones" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            zones: self.zones(),\n'
assert old in s
p.write_text(s.replace(old, '            zones: Vec::new(),\n'))
PY

mutation "from_save discards designations" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            designations.into_iter().collect(),\n'
assert old in s
p.write_text(s.replace(old, '            BTreeMap::new(),\n'))
PY

mutation "from_save discards zones" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            zones.into_iter().collect(),\n'
assert old in s
p.write_text(s.replace(old, '            BTreeSet::new(),\n'))
PY

mutation "load accepts an out-of-bounds designation" simd out_of_bounds_designation_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''        for (pos, _) in &save.designations {
            if !in_bounds(*pos) {
                bail!(
                    "save designation position {},{},{} is outside dims {}x{}x{}",
                    pos.x,
                    pos.y,
                    pos.z,
                    save.dims.x,
                    save.dims.y,
                    save.dims.z
                );
            }
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "load accepts an out-of-bounds zone" simd out_of_bounds_zone_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''        for pos in &save.zones {
            if !in_bounds(*pos) {
                bail!(
                    "save zone position {},{},{} is outside dims {}x{}x{}",
                    pos.x,
                    pos.y,
                    pos.z,
                    save.dims.x,
                    save.dims.y,
                    save.dims.z
                );
            }
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "designate discriminator is renamed" protocol decodes_and_reencodes_the_documented_command_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = '    Designate { kind: DesignationKind, rect: Rect },\n'
assert old in s
p.write_text(s.replace(old, '    #[serde(rename = "mark")]\n    Designate { kind: DesignationKind, rect: Rect },\n'))
PY

mutation "cancel_designation discriminator is renamed" protocol decodes_and_reencodes_the_documented_command_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = '    CancelDesignation { rect: Rect },\n'
assert old in s
p.write_text(s.replace(old, '    #[serde(rename = "erase")]\n    CancelDesignation { rect: Rect },\n'))
PY

mutation "place_stockpile discriminator is renamed" protocol decodes_and_reencodes_the_documented_command_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = '    PlaceStockpile { rect: Rect },\n'
assert old in s
p.write_text(s.replace(old, '    #[serde(rename = "store")]\n    PlaceStockpile { rect: Rect },\n'))
PY

mutation "remove_stockpile discriminator is renamed" protocol decodes_and_reencodes_the_documented_command_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = '    RemoveStockpile { rect: Rect },\n'
assert old in s
p.write_text(s.replace(old, '    #[serde(rename = "unstore")]\n    RemoveStockpile { rect: Rect },\n'))
PY

mutation "designate kind field is renamed" protocol decodes_and_reencodes_the_documented_command_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = '    Designate { kind: DesignationKind, rect: Rect },\n'
new = '''    Designate {
        #[serde(rename = "mode")]
        kind: DesignationKind,
        rect: Rect,
    },
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "bridge swaps dig and channel outbound" simd every_designation_kind_maps_to_its_named_wire_variant <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '''        sim_core::DesignationKind::Dig => protocol::DesignationKind::Dig,
        sim_core::DesignationKind::Channel => protocol::DesignationKind::Channel,
'''
new = '''        sim_core::DesignationKind::Dig => protocol::DesignationKind::Channel,
        sim_core::DesignationKind::Channel => protocol::DesignationKind::Dig,
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "bridge delta drops designations" simd snapshot_and_delta_carry_the_worlds_real_marks <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '''        designations: world
            .designations()
            .into_iter()
            .map(|(pos, kind)| protocol::Designation {
                pos: pos_out(pos),
                kind: designation_kind_out(kind),
            })
            .collect(),
'''
assert s.count(old) == 2
cut = s.rfind(old)
p.write_text(s[:cut] + '        designations: Vec::new(),\n' + s[cut + len(old):])
PY

mutation "bridge delta drops zones" simd snapshot_and_delta_carry_the_worlds_real_marks <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '''        zones: world
            .zones()
            .into_iter()
            .map(|pos| protocol::Zone { pos: pos_out(pos) })
            .collect(),
'''
assert s.count(old) == 2
cut = s.rfind(old)
p.write_text(s[:cut] + '        zones: Vec::new(),\n' + s[cut + len(old):])
PY

mutation "daemon designate arm decodes but discards" simd designation_and_stockpile_changes_reach_both_clients <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''                protocol::Command::Designate { kind, rect } => {
                    world.apply_command(sim_core::SimCommand::Designate {
                        kind: bridge::designation_kind_in(kind),
                        rect: bridge::rect_in(rect),
                    });
                }
'''
assert old in s
p.write_text(s.replace(old, '                protocol::Command::Designate { .. } => {}\n'))
PY

mutation "daemon remove-stockpile arm decodes but discards" simd designation_and_stockpile_changes_reach_both_clients <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''                protocol::Command::RemoveStockpile { rect } => {
                    world.apply_command(sim_core::SimCommand::RemoveStockpile {
                        rect: bridge::rect_in(rect),
                    });
                }
'''
assert old in s
p.write_text(s.replace(old, '                protocol::Command::RemoveStockpile { .. } => {}\n'))
PY

mutation "daemon designate intake is blocked while paused" simd designation_is_applied_while_tick_is_paused <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''                protocol::Command::Designate { kind, rect } => {
                    world.apply_command(sim_core::SimCommand::Designate {
                        kind: bridge::designation_kind_in(kind),
                        rect: bridge::rect_in(rect),
                    });
                }
'''
new = '''                protocol::Command::Designate { kind, rect } => {
                    if speed != protocol::Speed::Paused {
                        world.apply_command(sim_core::SimCommand::Designate {
                            kind: bridge::designation_kind_in(kind),
                            rect: bridge::rect_in(rect),
                        });
                    }
                }
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "daemon swaps place and remove stockpile" simd designation_and_stockpile_changes_reach_both_clients <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
s = s.replace('world.apply_command(sim_core::SimCommand::PlaceStockpile {', 'world.apply_command(sim_core::SimCommand::SWAP {', 1)
s = s.replace('world.apply_command(sim_core::SimCommand::RemoveStockpile {', 'world.apply_command(sim_core::SimCommand::PlaceStockpile {', 1)
s = s.replace('world.apply_command(sim_core::SimCommand::SWAP {', 'world.apply_command(sim_core::SimCommand::RemoveStockpile {', 1)
assert 'SimCommand::SWAP' not in s
p.write_text(s)
PY

mutation "x commits only CancelDesignation" tui remove_mode_commits_cancel_then_remove_stockpile_for_the_same_rect <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''                    Mode::Remove => Action::Commands([
                        Command::CancelDesignation { rect },
                        Command::RemoveStockpile { rect },
                    ]),
'''
assert old in s
p.write_text(s.replace(old, '                    Mode::Remove => Action::Command(Command::CancelDesignation { rect }),\n'))
PY

mutation "second Enter clears anchor without emitting" tui second_enter_commits_each_single_command_mode_and_stays_in_mode <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''                    Mode::Dig => Action::Command(Command::Designate {
                        kind: DesignationKind::Dig,
                        rect,
                    }),
'''
assert old in s
p.write_text(s.replace(old, '                    Mode::Dig => Action::Redraw,\n'))
PY

mutation "Esc exits mode instead of releasing anchor" tui mode_keys_enter_only_from_normal_and_escape_backs_out_one_level <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''        KeyCode::Esc if state.anchor.is_some() => {
            state.anchor = None;
            Action::Redraw
        }
'''
new = '''        KeyCode::Esc if state.anchor.is_some() => {
            state.anchor = None;
            state.mode = Mode::Normal;
            Action::Redraw
        }
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "cursor movement stops following with the camera" tui cursor_moves_clamps_and_pans_camera_only_after_crossing_the_window_edge <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '        state.camera.0 += sx - (i64::from(w) - 1);\n'
assert old in s
p.write_text(s.replace(old, '        state.camera.0 += 0;\n'))
PY

mutation "optimistic speed stops updating local state" tui optimistic_speed_keys_compose_before_a_wire_update <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '        state.speed = speed;\n        Action::Command(Command::SetSpeed { speed })\n'
assert old in s
p.write_text(s.replace(old, '        Action::Command(Command::SetSpeed { speed })\n'))
PY

mutation "hint bar is dropped" tui status_and_hint_occupy_the_bottom_two_rows <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''    let hint = hint(state);
    let hint_y = h - 1;
    for (x, glyph) in (0..w).zip(hint.chars()) {
        framebuffer.cells[usize::from(x) + usize::from(hint_y) * usize::from(w)] = Cell {
            glyph,
            fg: STATUS_TEXT,
        };
    }

'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "designation layer is covered by terrain" tui marker_layers_follow_terrain_zone_designation_entity_pending_cursor_order <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''    for designation in &snapshot.designations {
        if let Some(index) = screen_index(designation.pos) {
            framebuffer.cells[index] = designation_cell(designation.kind);
        }
    }
'''
new = old + '''    for designation in &snapshot.designations {
        if let Some(index) = screen_index(designation.pos) {
            let [x, y, z] = designation.pos;
            framebuffer.cells[index] = tile_cell(snapshot.tiles[tile_index(
                snapshot.dims,
                x as u32,
                y as u32,
                z as u32,
            )]);
        }
    }
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "read_inbound calls every partial line an overflow" simd unterminated_partial_line_is_not_reported_as_overflow <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''                if !line.ends_with(b"\\n") {
                    if line.len() as u64 >= MAX_LINE_BYTES {
                        eprintln!(
                            "client line exceeded {MAX_LINE_BYTES} bytes; closing connection"
                        );
                        let _ = reader.get_ref().shutdown(Shutdown::Both);
                    }
                    break;
                }
'''
new = '''                if !line.ends_with(b"\\n") {
                    eprintln!("client line exceeded {MAX_LINE_BYTES} bytes; closing connection");
                    let _ = reader.get_ref().shutdown(Shutdown::Both);
                    break;
                }
'''
assert old in s
p.write_text(s.replace(old, new))
PY

# NOTE: the two "keyed capture drain" mutations that stood here were removed at 3.1's code
# review together with the production drain they protected (a client-side constant encoding
# simd's CLIENT_QUEUE depth inside `tui`, which depends on `protocol` alone). The instrument
# now simply requests more frames than the delta backlog, so there is no drain to sabotage.
# Mutation count is 33 by design, not by attrition.
