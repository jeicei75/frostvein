# Mutation set for story 5.2. Run:
# scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/5-2-one-mirror-two-clients.sh

mutation "delta absence no longer deletes entities" client-core delta_deletes_entities_and_items_absent_from_authoritative_lists <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '        self.entities = next_entities;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.entities = self.entities.clone();\n'))
PY

mutation "snapshot retains previous-tick state" client-core changes_partition_entities_and_keep_one_previous_generation <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '        *self = Self::replace(snapshot);\n'
new = '''        let previous_entities = std::mem::take(&mut self.previous_entities);
        *self = Self::replace(snapshot);
        self.previous_entities = previous_entities;
'''
assert s.count(old) == 1
p.write_text(s.replace(old, new))
PY

mutation "unchanged entity is reported as changed" client-core changes_partition_entities_and_keep_one_previous_generation <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '                Some(next) if next != entity => {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                Some(next) if next == entity => {\n'))
PY

mutation "daemon accepts inverted rectangles" simd invalid_rects_are_logged_dropped_and_leave_the_client_connected <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''                        if let Some(rect) = command_rect(&command)
                            && !rect_is_valid(rect)
                        {
                            eprintln!("invalid client rect: {}", excerpt(text));
                            continue;
                        }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

# Added at review 2026-08-11. The tui key tests briefly expected `rect_on_level(..)` — the same
# helper `apply_key` calls — so this mutation would have SURVIVED. It is here to prove the
# restored literal oracle in view.rs actually kills a normalization change. Deliberately targets
# `tui`, not `client-core`: client-core's own test already pins the helper against literals, and
# the guard that was lost was the one on the caller's side.
mutation "rect helper stops normalizing corners" tui second_enter_commits_each_single_command_mode_and_stays_in_mode <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '''        min: [a.0.min(b.0), a.1.min(b.1), z],
        max: [a.0.max(b.0), a.1.max(b.1), z],
'''
new = '''        min: [a.0.max(b.0), a.1.max(b.1), z],
        max: [a.0.min(b.0), a.1.min(b.1), z],
'''
assert s.count(old) == 1
p.write_text(s.replace(old, new))
PY

mutation "mirror entity iteration reverses ids" client-core recorded_wire_messages_build_the_expected_mirror <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '        self.entities.values()\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.entities.values().rev()\n'))
PY
