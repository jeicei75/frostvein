# Story 12.2: five dwarves you can name. Run alone after commit.
# Row 9 is killed by the UNIT test, not the pixel guard: the widened aperture keeps a feet-focused
# dwarf sharp too (ratio 1.000). Row 10 is the one the pixel guard owns (ratio 0.468 unscaled).

mutation "identity drawn from spawn_rng" sim-core spawn_positions_for_seed_42_are_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        names.shuffle(&mut identity_rng);\n        colours.shuffle(&mut identity_rng);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        names.shuffle(&mut spawn_rng);\n        colours.shuffle(&mut spawn_rng);\n'))
PY

mutation "from_save ignores the saved identity" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-10-02 (12.4): `dwarf.profession,` now follows the identity in the spawn tuple.
old = '                    dwarf.identity,\n                    dwarf.profession,\n                ))\n'
assert s.count(old) == 1
new = '                    Identity {\n                        name: DwarfName::Durin,\n                        colour: DwarfColour::Red,\n                    },\n                    dwarf.profession,\n                ))\n'
p.write_text(s.replace(old, new))
PY

mutation "all five dwarves get one colour" sim-core five_dwarves_on_walkable_surface <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            colour: colours[i],\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            colour: colours[0],\n'))
PY

mutation "bridge sends identity none" simd save_then_load_rewinds_every_client <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '                .map(|(_, identity)| protocol::Identity {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .filter(|_| false)\n                .map(|(_, identity)| protocol::Identity {\n'))
PY

mutation "tui never draws the roster" tui the_roster_row_names_each_dwarf_in_his_colour_and_follows_an_identity_swap <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
# Re-pointed 2026-10-02 (12.4): the roster now pairs each identity with the dwarf's profession.
old = '                .identity\n                .map(|identity| (identity, entity.profession))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .identity\n                .filter(|_| false)\n                .map(|identity| (identity, entity.profession))\n'))
PY

mutation "one material for all dwarves" gui a_dwarfs_tunic_material_follows_his_identity_and_a_swap_swaps_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            .position(|c| *c == colour)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            .position(|_| true)\n'))
PY

mutation "reconcile ignores an identity change" gui a_dwarfs_tunic_material_follows_his_identity_and_a_swap_swaps_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                && projected_tunic.is_none_or(|existing| existing.0 != identity.colour)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                && projected_tunic.is_none()\n'))
PY

mutation "hud never shows the name" gui the_name_hud_shows_the_selected_dwarfs_name_in_his_colour_and_clears <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            client_core::dwarf_name_text(identity.name),\n            crate::appearance::dwarf_tunic_color(identity.colour),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            "",\n            crate::appearance::dwarf_tunic_color(identity.colour),\n'))
PY

mutation "dof focus reverted to translation (feet)" gui depth_of_field_focuses_the_selected_dwarf_not_the_rigs_aim_point <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '.map(|(_, transform)| transform.translation + Vec3::Y * (DWARF_HEIGHT_CELLS / 2.0))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '.map(|(_, transform)| transform.translation)\n'))
PY

mutation "selected aperture not scaled" gui a_selected_dwarf_at_the_closest_zoom_is_sharp_with_dof_on ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    DOF_APERTURE_F_STOPS * (boot / focal_distance).powi(2).max(1.0)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = (boot, focal_distance);\n    DOF_APERTURE_F_STOPS\n'))
PY

mutation "tui dwarf glyph ignores his tunic colour" tui a_named_dwarf_is_drawn_in_his_tunic_colour_and_a_nameless_one_in_his_job_colour <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '                match entity.identity {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                match None::<protocol::Identity> {\n'))
PY

mutation "named tui dwarf drawn as the emoji-capable smiley" tui a_named_dwarf_is_drawn_in_his_tunic_colour_and_a_nameless_one_in_his_job_colour <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = "                        glyph: '☻',\n                        fg: dwarf_colour(identity.colour),\n"
assert s.count(old) == 1
p.write_text(s.replace(old, "                        glyph: '☺',\n                        fg: dwarf_colour(identity.colour),\n"))
PY

mutation "deselect keeps the selected focal distance" gui depth_of_field_focuses_the_selected_dwarf_not_the_rigs_aim_point <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '                dof.focal_distance = dof_focal_distance(transform.translation(), rig);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                let _ = rig;\n'))
PY

mutation "deselect keeps the widened aperture" gui depth_of_field_focuses_the_selected_dwarf_not_the_rigs_aim_point <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '                dof.aperture_f_stops = DOF_APERTURE_F_STOPS;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "--select of a missing dwarf runs on" gui select_of_a_missing_or_non_dwarf_id_fails_the_run <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        eprintln!("--select {id}: no dwarf has that id in the daemon\'s snapshot");\n        exit.write(AppExit::error());\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        eprintln!("--select {id}: no dwarf has that id in the daemon\'s snapshot");\n'))
PY

mutation "--select accepts a campfire id" gui select_of_a_missing_or_non_dwarf_id_fails_the_run <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        .any(|entity| entity.id == id && entity.kind == protocol::EntityKind::Dwarf)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .any(|entity| entity.id == id)\n'))
PY

mutation "same seed draws different identities per world" sim-core same_seed_and_commands_remain_deterministic <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        let mut identity_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_IDENTITY);\n'
assert s.count(old) == 1
new = ('        static WORLDS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);\n'
       '        let mut identity_rng = ChaCha8Rng::seed_from_u64(\n'
       '            seed ^ STREAM_IDENTITY ^ WORLDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed),\n'
       '        );\n')
p.write_text(s.replace(old, new))
PY
