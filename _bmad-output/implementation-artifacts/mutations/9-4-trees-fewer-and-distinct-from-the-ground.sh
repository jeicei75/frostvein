# Story 9.4 sabotage table. Run alone: scripts/mutate.sh <this file>

mutation "tree density returns to the old one-in-twelve roll" sim-core tree_density_for_seed_42_is_deterministic_and_in_target_band <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = '            if rng.random_range(0..48) != 0\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if rng.random_range(0..12) != 0\n'))
PY

mutation "foliage returns to the near-camouflage blue" gui appearance_tables_pin_the_cold_boot_palette <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '        Material::TreeFoliage => Color::srgb_u8(44, 100, 58),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        Material::TreeFoliage => Color::srgb_u8(55, 73, 84),\n'))
PY

# NOTE: an earlier row here lowered MIN_MARK_SEPARATION itself and SURVIVED, correctly -- you
# cannot sabotage an assertion and expect that same assertion to catch it. A weakened floor is
# invisible while the colour is good, and a bad colour is already caught by the row above. This
# row replaces it with a PRODUCTION mutation the invariant must catch, guarding W2's ruling.
mutation "foliage goes brown, breaking the blueward-of-red terrain invariant" gui appearance_tables_pin_the_cold_boot_palette <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
# Mutate the PIN TOO. A colour-only change dies at the assert_eq! pin, which proves the pin
# works and never reaches the invariant. A real brown change would move both, so this does.
old = '        Material::TreeFoliage => Color::srgb_u8(44, 100, 58),\n'
assert s.count(old) == 1
s = s.replace(old, '        Material::TreeFoliage => Color::srgb_u8(100, 74, 52),\n')
pin = '            (Material::TreeFoliage, [44, 100, 58]),\n'
assert s.count(pin) == 1
p.write_text(s.replace(pin, '            (Material::TreeFoliage, [100, 74, 52]),\n'))
PY

mutation "the tree count oracle counts trunk cells instead of columns" sim-core tree_density_for_seed_42_is_deterministic_and_in_target_band <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/tests/worldgen.rs'); s = p.read_text()
old = '''fn tree_trunk_columns(world: &World) -> usize {
    (0..world.dims().y as i32)
        .flat_map(|y| (0..world.dims().x as i32).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            (0..world.dims().z as i32)
                .any(|z| world.tile(Pos { x, y, z }) == Some(Tile::Solid(Material::TreeTrunk)))
        })
        .count()
}
'''
assert s.count(old) == 1
new = '''fn tree_trunk_columns(world: &World) -> usize {
    (0..world.dims().y as i32)
        .flat_map(|y| (0..world.dims().x as i32).map(move |x| (x, y)))
        .map(|(x, y)| {
            (0..world.dims().z as i32)
                .filter(|&z| world.tile(Pos { x, y, z }) == Some(Tile::Solid(Material::TreeTrunk)))
                .count()
        })
        .sum()
}
'''
p.write_text(s.replace(old, new))
PY
