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
        !self.moved_ids.is_empty()
    }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '    fn moved(&self) -> bool {\n        true\n    }\n'))
PY

mutation "lantern capture loses its lit-terrain count" gui capture::tests::lantern_instrument_requires_a_lit_region_to_move <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '        self.lit_tiles.extend(region.iter().copied());\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "lantern capture accepts an empty final region" gui capture::tests::a_final_observation_that_lit_nothing_fails_even_after_a_dwarf_moved <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '            !self.last_region.is_empty(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            true,\n'))
PY

mutation "reconciliation lights a dwarf the wire left unlit" gui an_unlit_dwarf_gets_no_point_light <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            if let Some(light) = mirror_entity.and_then(|entity| entity.light) {'
assert s.count(old) == 1
new = ('            if let Some(light) = mirror_entity.and_then(|entity| entity.light.or_else(|| '
       '(entity.kind == protocol::EntityKind::Dwarf).then_some(protocol::LightKind::Lantern))) {')
p.write_text(s.replace(old, new, 1))
PY

# --- Added by the 2026-08-19 code review. Each one reverts a patch that review applied. ---

mutation "lantern capture latches an empty first region" gui capture::tests::an_empty_first_observation_cannot_stand_in_for_movement <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '            if lit.is_empty() {\n                continue;\n            }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "lantern movement forgets a dwarf that wandered back" gui capture::tests::a_dwarf_that_returns_to_where_it_started_still_counts_as_having_moved <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '        !self.moved_ids.is_empty()\n'
assert s.count(old) == 1
new = ('        self.first_regions\n'
       '            .values()\n'
       '            .next()\n'
       '            .is_some_and(|first| *first != self.last_region)\n')
p.write_text(s.replace(old, new, 1))
PY

mutation "the capture stops deriving lit regions from mirror states" gui accumulate_motion_derives_a_moving_lit_region_from_mirror_states <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '    if capture.lantern.needs_observation(&positions) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false {\n', 1))
PY

mutation "the lantern goes dark but stays present" gui a_wire_declared_dwarf_lantern_uses_the_shared_appearance_table <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '            intensity: 11_000_000.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            intensity: 0.0,\n', 1))
PY
