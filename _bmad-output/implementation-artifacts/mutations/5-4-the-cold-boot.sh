# Story 5.4 sabotage table. Run alone with scripts/mutate.sh.

mutation "snow cap leaves bare top" gui snow_cap_marks_only_solid_tops_in_a_hand_built_toy_world <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    matches!(mirror.tile(position), Some(Tile::Solid(_)))\n        && !matches!(\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    false\n        && !matches!(\n'))
PY

mutation "torch table goes cold" gui appearance_tables_pin_the_cold_boot_palette <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '            color: Color::srgb_u8(255, 140, 62),\n'
assert s.count(old) == 2
p.write_text(s.replace(old, '            color: Color::srgb_u8(62, 140, 255),\n', 1))
PY

mutation "atmosphere loses client local marker" gui atmosphere_entities_are_client_local_and_never_world_projected <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '            ClientLocal,\n'
assert s.count(old) == 3
p.write_text(s.replace(old, '', 1))
PY

mutation "emitters ignore wire light field" gui recorded_camp_snapshot_projects_exactly_five_warm_point_lights <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                if let Some(light) = mirror_entity.light {\n                    entity.insert(point_light(light));\n                }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                let _ = mirror_entity.light;\n'))
PY
