# Mutation set for story 2.4. Run:
# scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh

mutation "from_save drops the wander cooldown" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = "                    cooldown: dwarf.cooldown,\n"
assert old in s
p.write_text(s.replace(old, "                    cooldown: 0,\n"))
PY

mutation "from_save drops the wander home" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = "                    home: dwarf.home,\n"
assert old in s
p.write_text(s.replace(old, "                    home: dwarf.pos,\n"))
PY

mutation "from_save reseeds the wander RNG" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = """            tick,
            wander_rng,
            IdAllocator { next: next_id },
"""
assert old in s
p.write_text(s.replace(old, """            tick,
            ChaCha8Rng::seed_from_u64(seed ^ STREAM_WANDER),
            IdAllocator { next: next_id },
"""))
PY

mutation "loaded world schedule omits wander" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-08-22: `assemble` gained designation and zone arguments, and the save
# destructure gained items and emitters, so this anchor rotted. The seam is unchanged.
old = '''            zones.into_iter().collect(),
        );
        for dwarf in dwarves {
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''            zones.into_iter().collect(),
        );
        world.schedule = Schedule::default();
        world.schedule.add_systems(advance_tick);
        for dwarf in dwarves {
'''))
PY

mutation "from_save regenerates terrain from the seed" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-08-22: `assemble` gained designation and zone arguments, and the save
# destructure gained items and emitters, so this anchor rotted. The seam is unchanged.
old = '''            emitters,
        } = save;
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''            emitters,
        } = save;
        let tiles = World::generate(seed, dims).to_save().tiles;
'''))
PY

mutation "from_save resets the id allocator" sim-core loading_does_not_reuse_entity_ids <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = "            IdAllocator { next: next_id },\n"
assert old in s
p.write_text(s.replace(old, "            IdAllocator::default(),\n"))
PY

mutation "from_save marks every tile dirty" sim-core loading_starts_with_no_dirty_tiles <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-08-22: `assemble` gained designation and zone arguments, and the save
# destructure gained items and emitters, so this anchor rotted. The seam is unchanged.
old = '''            zones.into_iter().collect(),
        );
        for dwarf in dwarves {
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''            zones.into_iter().collect(),
        );
        {
            let mut terrain = world.ecs.resource_mut::<Terrain>();
            let d = terrain.dims;
            for z in 0..d.z as i32 {
                for y in 0..d.y as i32 {
                    for x in 0..d.x as i32 {
                        terrain.dirty.insert(Pos { x, y, z });
                    }
                }
            }
        }
        for dwarf in dwarves {
'''))
PY

mutation "to_save records tick zero" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = "            tick: self.tick(),\n"
assert old in s
p.write_text(s.replace(old, "            tick: 0,\n"))
PY

mutation "to_save leaves dwarves in ECS order" sim-core save_orders_dwarves_by_id <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Narrowed 2026-08-22: the trailing blank line this anchored on is gone.
old = "        dwarves.sort_by_key(|dwarf| dwarf.id);\n"
assert s.count(old) == 1
p.write_text(s.replace(old, ""))
PY

mutation "daemon Save arm parses but writes nothing" simd saved_file_decodes_as_a_save_state <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = "                protocol::Command::Save => save_world(&world),\n"
assert old in s
p.write_text(s.replace(old, "                protocol::Command::Save => {}\n"))
PY

mutation "daemon Load arm skips snapshot broadcast" simd save_then_load_rewinds_every_client <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = "                        broadcast(&mut clients, &line);\n"
assert old in s
p.write_text(s.replace(old, ""))
PY

mutation "daemon Quit arm is ignored" simd quit_exits_the_daemon_cleanly <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = """                protocol::Command::Quit => {
                    eprintln!("shutting down on client quit");
                    return Ok(());
                }
"""
assert old in s
p.write_text(s.replace(old, "                protocol::Command::Quit => {}\n"))
PY

mutation "daemon save path is renamed" simd saved_file_decodes_as_a_save_state <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = 'const SAVE_PATH: &str = "frostvein.save";\n'
assert old in s
p.write_text(s.replace(old, 'const SAVE_PATH: &str = "other.save";\n'))
PY

mutation "daemon save read limit is widened" simd oversized_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
# Re-pointed 2026-08-22: the surrounding code grew (rect validation, haul jobs, a `path`
# argument, a larger save ceiling) and this anchor rotted with it. The seam is unchanged.
old = 'const MAX_SAVE_BYTES: u64 = 64 * 1024 * 1024;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const MAX_SAVE_BYTES: u64 = 1024 * 1024 * 1024;\n'))
PY

mutation "failed load panics the daemon" simd undecodable_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
# Re-pointed 2026-08-22: the surrounding code grew (rect validation, haul jobs, a `path`
# argument, a larger save ceiling) and this anchor rotted with it. The seam is unchanged.
old = '''        Err(error) => {
            eprintln!("could not load {path}: {error:#}");
            None
        }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '        Err(error) => panic!("could not load {path}: {error:#}"),\n'))
PY

mutation "load skips tile-count validation" simd inconsistent_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''        let tile_count = u64::from(save.dims.x)
            .checked_mul(u64::from(save.dims.y))
            .and_then(|area| area.checked_mul(u64::from(save.dims.z)))
            .context("save dimensions overflow the tile count")?;
        if save.tiles.len() as u64 != tile_count {
            bail!(
                "save has {} tiles but dims {}x{}x{} need {tile_count}",
                save.tiles.len(),
                save.dims.x,
                save.dims.y,
                save.dims.z
            );
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "load widens the supported tick range" simd boundary_tick_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = "const MAX_LOAD_TICK: u64 = u64::MAX / 2;\n"
assert old in s
p.write_text(s.replace(old, "const MAX_LOAD_TICK: u64 = u64::MAX;\n"))
PY

mutation "load accepts an out-of-bounds dwarf" simd out_of_bounds_dwarf_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = "            if !in_bounds(dwarf.pos) {\n"
assert old in s
p.write_text(s.replace(old, "            if false {\n"))
PY

mutation "load accepts an out-of-bounds dwarf home" simd out_of_bounds_dwarf_home_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = "            if !in_bounds(dwarf.home) {\n"
assert old in s
p.write_text(s.replace(old, "            if false {\n"))
PY

mutation "S maps to Load" tui speed_keys_follow_the_pinned_step_table_and_clamp <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = "        KeyCode::Char('S') => Action::Command(Command::Save),\n"
assert old in s
p.write_text(s.replace(old, "        KeyCode::Char('S') => Action::Command(Command::Load),\n"))
PY

mutation "L maps to Save" tui speed_keys_follow_the_pinned_step_table_and_clamp <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = "        KeyCode::Char('L') => Action::Command(Command::Load),\n"
assert old in s
p.write_text(s.replace(old, "        KeyCode::Char('L') => Action::Command(Command::Save),\n"))
PY

mutation "frames key path never writes its command" tui key_l_rewinds_captured_ticks_then_they_climb_from_the_saved_tick <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/main.rs'); s = p.read_text()
old = "            send_command(&mut writer, command)?;\n"
assert old in s
p.write_text(s.replace(old, "", 1))
PY

mutation "frames instrument ignores load snapshots" tui key_l_rewinds_captured_ticks_then_they_climb_from_the_saved_tick <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/main.rs'); s = p.read_text()
# Re-pointed 2026-08-22: 5.2 moved `tui` onto the shared client-core mirror, which rotted this
# anchor. The seam itself is unchanged.
old = '''            Ok(Ok(Msg::Snapshot(next))) => {
                mirror
                    .apply_snapshot(*next)
                    .context("could not update client mirror")?;
                state.speed = mirror.speed();
            }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '            Ok(Ok(Msg::Snapshot(_))) => {}\n'))
PY

mutation "frames instrument ignores deltas" tui load_capable_stub_climbs_monotonically_when_no_key_is_sent <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/main.rs'); s = p.read_text()
# Re-pointed 2026-08-22: 5.2 moved `tui` onto the shared client-core mirror, which rotted this
# anchor. The seam itself is unchanged.
old = '''            Ok(Ok(Msg::Delta(delta))) => {
                mirror.apply_delta(*delta);
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''            Ok(Ok(Msg::Delta(delta))) => {
                let _ = delta;
'''))
PY

mutation "save discriminator is renamed" protocol decodes_and_reencodes_the_documented_command_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = "    Save,\n"
assert old in s
p.write_text(s.replace(old, '    #[serde(rename = "store")]\n    Save,\n'))
PY

mutation "load discriminator is renamed" protocol decodes_and_reencodes_the_documented_command_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = "    Load,\n"
assert old in s
p.write_text(s.replace(old, '    #[serde(rename = "restore")]\n    Load,\n'))
PY

mutation "quit discriminator is renamed" protocol decodes_and_reencodes_the_documented_command_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = "    Quit,\n"
assert old in s
p.write_text(s.replace(old, '    #[serde(rename = "exit")]\n    Quit,\n'))
PY

mutation "load skips the dwarf-id uniqueness check" simd duplicate_dwarf_id_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = """            if !seen_ids.insert(dwarf.id) {
                bail!("save reuses dwarf id {}", dwarf.id);
            }
"""
assert old in s
p.write_text(s.replace(old, "            seen_ids.insert(dwarf.id);\n"))
PY

mutation "apply_key rejects SHIFT" tui save_and_load_keys_still_map_when_shift_is_held <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = "    if !key.modifiers.difference(KeyModifiers::SHIFT).is_empty() {\n"
assert old in s
p.write_text(s.replace(old, "    if !key.modifiers.is_empty() {\n"))
PY

mutation "the frames key path sends save as load" tui key_s_sends_save_and_leaves_the_streamed_ticks_climbing <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/main.rs'); s = p.read_text()
# Re-pointed 2026-08-22: 5.2 moved `tui` onto the shared client-core mirror, which rotted this
# anchor. The seam itself is unchanged.
old = '        "S" => Some(KeyCode::Char(\'S\')),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        "S" => Some(KeyCode::Char(\'L\')),\n'))
PY
