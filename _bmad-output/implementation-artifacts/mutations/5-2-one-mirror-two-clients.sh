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

mutation "mirror entity iteration reverses ids" client-core recorded_wire_messages_build_the_expected_mirror <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '        self.entities.values()\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.entities.values().rev()\n'))
PY
