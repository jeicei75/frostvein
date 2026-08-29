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

# Added at review, 2026-08-29, with Wolf's snow-cover fix. The guard being pinned is the
# ground-rest clause, so the row DELETES that clause rather than touching the colour: a colour
# mutation would die at the palette pin and never reach this behaviour.
mutation "snow returns to the skirt by dropping the ground-rest clause" gui foliage_resting_on_the_ground_is_a_skirt_and_never_catches_snow <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '''        )
        && !rests_on_the_ground(mirror, position)
}'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''        )
}''', 1))
PY

# Added 2026-08-29 with Wolf's second vehicle ruling. Restores the ground-level foliage ring, which
# is the production defect: it seals the lower trunk so a third of trees draw none. The row must
# die on the TRUNKLESS assertion, not on the foliage-at-base one -- the trunkless clause is the
# outcome Wolf asked for and the base-level clause is only its mechanism.
mutation "the ground-level foliage ring comes back and seals the trunks" sim-core every_tree_shows_a_trunk_and_no_foliage_sits_at_the_trunk_base <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
anchor = '            // NO FOLIAGE RING AT surface + 1.'
assert s.count(anchor) == 1
ring = '''            for fy in y - 1..=y + 1 {
                for fx in x - 1..=x + 1 {
                    if fx != x || fy != y {
                        let foliage = index(dims, fx, fy, surface + 1);
                        if tiles[foliage] == Tile::Empty {
                            tiles[foliage] = Tile::Solid(Material::TreeFoliage);
                        }
                    }
                }
            }
'''
p.write_text(s.replace(anchor, ring + anchor, 1))
PY

# Headless capture, added 2026-08-29. Row 7 removes the flag so a --headless run silently opens the
# windowed path; row 8 makes setup_camera ignore the request so a headless run renders to a window
# that does not exist -- the silent-empty-frame shape this whole path exists to avoid.
mutation "the --headless flag stops being parsed" gui headless_is_off_by_default_and_on_only_when_asked <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '''        } else if arg == "--headless" {
            headless = true;
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '', 1))
PY

mutation "a headless camera keeps its window target" gui a_headless_camera_draws_into_an_offscreen_target_and_a_windowed_one_does_not <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let headless_target = match (headless.is_some(), images) {'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let headless_target = match (false, images) {', 1))
PY
