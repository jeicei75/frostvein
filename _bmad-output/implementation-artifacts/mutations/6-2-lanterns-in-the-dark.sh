# Story 6.2 sabotage table. Run alone with scripts/mutate.sh.

mutation "snapshot dwarf arm drops lanterns" simd every_dwarf_carries_a_lantern_in_snapshot_and_delta_without_duplication <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '                light: Some(light_kind(light)),\n'
assert s.count(old) == 2
p.write_text(s.replace(old, '                light: None,\n', 1))
PY

mutation "delta dwarf arm drops lanterns" simd every_dwarf_carries_a_lantern_in_snapshot_and_delta_without_duplication <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '                light: Some(light_kind(light)),\n'
assert s.count(old) == 2
head, separator, tail = s.rpartition(old)
assert separator
p.write_text(head + '                light: None,\n' + tail)
PY

mutation "static emitter bridge accepts lanterns" simd static_lantern_emitters_remain_rejected_by_the_bridge_guard <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '        sim_core::LightKind::Lantern => unreachable!("lanterns are not live emitters"),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        sim_core::LightKind::Lantern => protocol::EntityKind::Dwarf,\n'))
PY

mutation "saved static lantern emitters load" simd loading_rejects_static_lantern_emitters_before_the_wire_bridge <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            if *light == sim_core::LightKind::Lantern {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false {\n'))
PY

mutation "wire-declared lanterns no longer create lights" gui a_wire_declared_dwarf_lantern_uses_the_shared_appearance_table <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                    if let Some(light) = mirror_entity.light {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    if let Some(light) = None {\n'))
PY

mutation "lantern capture accepts an unmoved region" gui capture::tests::lantern_instrument_requires_a_lit_region_to_move <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '''    fn moved(&self) -> bool {
        self.first_region
            .as_ref()
            .is_some_and(|first| *first != self.last_region)
    }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '    fn moved(&self) -> bool {\n        true\n    }\n'))
PY

mutation "lantern capture loses its lit-terrain count" gui capture::tests::lantern_instrument_requires_a_lit_region_to_move <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '        self.lit_terrain_tiles += region.len();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.lit_terrain_tiles = 0;\n'))
PY
