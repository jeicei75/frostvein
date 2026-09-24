# Story 11.3 sitting 1: clock, key, sky and haze. Run alone after committing source.

mutation "hour ignores the wire tick" gui tick_clock_moves_fractionally_and_wraps <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/clock.rs'); s = p.read_text()
old = '    (BOOT_HOUR + (tick % TICKS_PER_DAY) as f32 / TICKS_PER_HOUR) % 24.0\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = tick;\n    BOOT_HOUR\n'))
PY

mutation "clock flag parses but never reaches the pin" gui clock_pin_reaches_the_live_app_and_capture_defaults_to_boot <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        args.clock\n            .or(args.capture.as_ref().map(|_| crate::clock::BOOT_HOUR)),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        None.or(args.capture.as_ref().map(|_| crate::clock::BOOT_HOUR)),\n'))
PY

mutation "installed key direction stays at the boot aim" gui clock_drives_the_installed_key_direction_color_and_illuminance <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        *transform = Transform::from_translation(Vec3::ZERO).looking_to(direction, Vec3::Y);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        *transform = sun_light_transform();\n'))
PY

mutation "the key stays lit at the horizon" gui lit_key_never_points_up_or_jumps_in_illuminance <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '    let horizon = (elevation / ramp_degrees).clamp(0.0, 1.0);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let horizon = 1.0;\n'))
PY

mutation "F8 restores the night budget at noon" gui f8_restores_the_clock_key_at_noon <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        light.illuminance = if toggles.enabled(LightSource::Sun) {\n            illuminance\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        light.illuminance = if toggles.enabled(LightSource::Sun) {\n            night_lighting().directional_illuminance\n'))
PY

mutation "stars retain their night colour at noon" gui noon_stars_and_aurora_fade_from_the_live_shared_materials <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '''    let star_color = if weight == 0.0 {
        lighting.star
    } else if weight == 1.0 {
        lighting.sky
    } else {
        mix_color(night_lighting().star, lighting.sky, weight)
    };
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '    let star_color = lighting.star;\n'))
PY

mutation "rim materials keep the night sky target" gui noon_rim_materials_dissolve_toward_the_live_sky <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                material.base_color = rim_dissolved_color_at(slot.base_color(), level, clear.0);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                material.base_color = rim_dissolved_color_at(slot.base_color(), level, crate::appearance::night_lighting().sky);\n'))
PY

mutation "distance fog keeps the night sky" gui noon_sky_and_distance_fog_share_the_day_colour <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            fog.color = lighting.sky;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            fog.color = night_lighting().sky;\n'))
PY

mutation "the boot hour moves from 22" gui tick_clock_moves_fractionally_and_wraps <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/clock.rs'); s = p.read_text()
old = 'pub const BOOT_HOUR: f32 = 22.0;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const BOOT_HOUR: f32 = 21.0;\n'))
PY

mutation "a default capture follows the wire tick" gui clock_pin_reaches_the_live_app_and_capture_defaults_to_boot <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/clock.rs'); s = p.read_text()
old = '    pin.0.unwrap_or_else(|| hour_at(mirror.0.tick()))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = pin;\n    hour_at(mirror.0.tick())\n'))
PY

mutation "hour advances in whole-hour steps" gui tick_clock_moves_fractionally_and_wraps <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/clock.rs'); s = p.read_text()
old = '    (BOOT_HOUR + (tick % TICKS_PER_DAY) as f32 / TICKS_PER_HOUR) % 24.0\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    (BOOT_HOUR + ((tick % TICKS_PER_DAY) / TICKS_PER_HOUR as u64) as f32) % 24.0\n'))
PY

mutation "F4-on reinserts night haze after the clock writer" gui noon_haze_ambient_survives_f4_off_and_on <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '''    app.add_systems(
        PostUpdate,
        (update_clock_sky, crate::project::update_rim_for_sky).chain(),
    );
'''
assert s.count(old) == 1
new = '''    app.add_systems(
        Update,
        (update_clock_sky, crate::project::update_rim_for_sky)
            .chain()
            .before(effect_controls),
    );
'''
p.write_text(s.replace(old, new))
PY

mutation "clock parser admits the excluded upper bound" gui clock_flag_accepts_hours_and_rejects_out_of_range_values <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '                Some(hour) if hour.is_finite() && (0.0..24.0).contains(&hour) => hour,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                Some(hour) if hour.is_finite() && (0.0..=24.0).contains(&hour) => hour,\n'))
PY

mutation "capture range line loses its clock field" gui the_range_check_line_reports_the_frame_shape <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '         resolution={width}x{height}{clock_note}",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '         resolution={width}x{height}",\n'))
PY

mutation "capture clock note reverses its pin source" gui capture_clock_note_names_the_rendered_hour_and_source <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '    let source = if explicit {\n        "--clock"\n    } else {\n        "capture default"\n    };\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let source = if explicit {\n        "capture default"\n    } else {\n        "--clock"\n    };\n'))
PY

mutation "current_hour ignores an unpinned snapshot" gui an_unpinned_seat_follows_two_wire_snapshots_one_hour_apart <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/clock.rs'); s = p.read_text()
old = '    pin.0.unwrap_or_else(|| hour_at(mirror.0.tick()))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = mirror;\n    pin.0.unwrap_or(BOOT_HOUR)\n'))
PY

mutation "hourly table stays night at noon" gui hourly_light_table_keeps_night_exact_and_reaches_the_approved_day <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '    if weight == 1.0 {\n        return day;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if weight == 1.0 {\n        return night;\n    }\n'))
PY

mutation "sky and ambient jump at dawn" gui sky_and_ambient_change_smoothly_over_each_hundredth_hour <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '    linear * linear * (3.0 - 2.0 * linear)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if linear > 0.0 { 1.0 } else { 0.0 }\n'))
PY

mutation "aurora remains opaque at noon" gui noon_stars_and_aurora_fade_from_the_live_shared_materials <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let aurora_color = Color::srgba(1.0, 1.0, 1.0, 1.0 - weight);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let aurora_color = Color::WHITE;\n'))
PY

mutation "night rim target misses the exact sky colour" gui noon_rim_materials_dissolve_toward_the_live_sky <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '    if level >= RIM_LEVELS - 1 {\n        return sky;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if level >= RIM_LEVELS - 1 && sky != night_lighting().sky {\n        return sky;\n    }\n'))
PY
