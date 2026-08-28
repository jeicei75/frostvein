use bevy::prelude::Color;
use protocol::{DesignationKind, EntityKind, LightKind, Material};

#[derive(Debug, Clone, Copy)]
pub struct LightProperties {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    /// Fraction of the base intensity available to presentation-only flicker.
    pub flicker_amplitude: f32,
    pub flicker_hz: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct EntityAppearance {
    pub color: Color,
    pub scale: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct NightLighting {
    pub sky: Color,
    pub star: Color,
    pub ambient: Color,
    pub ambient_brightness: f32,
    pub aurora: Color,
    /// The directional fill's own tint — desaturated aurora, not the curtain colour raw: a
    /// saturated green-blue light on blue materials is what turned the boot3 field electric.
    pub directional: Color,
    pub directional_illuminance: f32,
}

/// The light budget is set from MEASUREMENT, not estimate — twice now. Round 5 scaled the
/// round-4 capture's black field (ground median 21 sRGB vs the artifact's 123) up by ~18x
/// linear; the boot3 vehicle capture then measured 156 — overshot 26% sRGB — with shadows
/// flooded (p05 87 vs the artifact's 28) and a heavy blue-green cast (the old saturated
/// ambient/directional tints multiplied onto already-blue materials). This table divides the
/// budget the other way: a small desaturated ambient so shadow faces go genuinely dark, and a
/// desaturated cool directional carrying most of the load so lit faces keep their modelling.
pub fn night_lighting() -> NightLighting {
    NightLighting {
        sky: Color::srgb_u8(5, 12, 28),
        star: Color::srgb_u8(173, 196, 220),
        ambient: Color::srgb_u8(120, 140, 165),
        ambient_brightness: 4_500.0,
        aurora: Color::srgb_u8(73, 157, 144),
        directional: Color::srgb_u8(150, 190, 180),
        directional_illuminance: 22_000.0,
    }
}

pub fn light_properties(kind: LightKind) -> LightProperties {
    match kind {
        // Intensities sized against the boot3 measurement: the white-clip radius scales as
        // sqrt(intensity), and 72M lm blew a ~9-tile pool to flat white where the artifact
        // keeps white for the emitter faces alone (AC9).
        LightKind::Torch => LightProperties {
            color: Color::srgb_u8(255, 140, 62),
            intensity: 14_000_000.0,
            range: 20.0,
            flicker_amplitude: 0.30,
            flicker_hz: 1.7,
        },
        LightKind::Campfire => LightProperties {
            color: Color::srgb_u8(255, 173, 92),
            // RULED 2026-08-22 (Wolf), closing 6.2's carried-open "camp is too blown out".
            // The blow-out was never in this number -- it is in the PEAK. Commit 04e6de5 raised
            // the flicker amplitude 0.11 -> 0.40, taking the peak 35.5M -> 44.8M, 26% past the
            // value 5.4 sized against the approved artifact, while this still frame never moved.
            // Option (d) of four put to Wolf: drop the base so the PEAK lands back on 5.4's
            // approved ceiling (25.0M x 1.40 = 35.0M) while 6.1's visible breathing survives
            // intact. The camp rests 22% dimmer; the white-clip radius, which scales as
            // sqrt(intensity), shrinks ~12%. Rejected: amplitude -> 0.25, which leaves the peak
            // 12.6% high and the blown pool only 5.5% smaller -- it treats the still frame, which
            // was never the complaint.
            intensity: 25_000_000.0,
            range: 28.0,
            flicker_amplitude: 0.40,
            flicker_hz: 0.9,
        },
        LightKind::Lantern => LightProperties {
            color: Color::srgb_u8(255, 195, 110),
            intensity: 5_000_000.0,
            range: 14.0,
            flicker_amplitude: 0.05,
            flicker_hz: 1.3,
        },
    }
}

/// Client-side light animation: deterministic from the delivered id and local elapsed time.
pub fn flicker_scale(kind: LightKind, id: u32, seconds: f32) -> f32 {
    let properties = light_properties(kind);
    let phase = id as f32 * 1.618_034 + kind as u8 as f32 * 0.73;
    let primary = (seconds * properties.flicker_hz * std::f32::consts::TAU + phase).sin();
    let secondary =
        (seconds * (properties.flicker_hz * 1.73) * std::f32::consts::TAU + phase * 0.37).sin()
            * 0.3;
    1.0 + properties.flicker_amplitude * (primary + secondary) / 1.3
}

/// Neutral crushed stone is intentionally independent of the removed wire tile material.
pub fn debris_color() -> Color {
    Color::srgb_u8(86, 91, 106)
}

// NOTE: these intentionally do not match the TUI. Dig amber would read as false firelight over
// a large rock face, so gui keeps all work marks cold or neutral to preserve the warm-camp read.
//
// RETUNED at the 2026-08-21 review, for two reasons the first values got wrong.
// (1) COLLISION: dig was (92, 174, 224) — BYTE-IDENTICAL to the TUI's CHANNEL blue
// (`crates/tui/src/palette.rs:110`). Breaking with the TUI is deliberate; landing exactly on a
// DIFFERENT TUI order was not, and on the two windows Wolf runs side by side one RGB meant two
// things. Every mark here is now >= 50 from every TUI mark colour as well as from terrain.
// (2) AXIS: dig and channel were two blues separated almost entirely on GREEN (174 vs 120), and
// the shipped directional is a desaturated cool (150, 190, 180) over cool ambient — it multiplies
// toward teal and compresses exactly that axis, so the 40-unit floor was measured on unlit
// literals that the renderer then pushes together. They now separate on RED (56 vs 150), which
// this light does not compress, and sit 103 apart unlit against the old 51.
pub fn designation_color(kind: DesignationKind) -> Color {
    match kind {
        DesignationKind::Dig => Color::srgb_u8(56, 132, 250),
        DesignationKind::Channel => Color::srgb_u8(150, 96, 230),
    }
}

/// RULED 2026-08-22 (Wolf): darker and colder. The pale teal that shipped was the tightest thing
/// on the board -- 46 units from snow-capped foliage once the cool light is applied, against marks
/// that otherwise had 75+ of room, and a pale slab on pale foliage is precisely the near-neighbour
/// case the separation floor exists to prevent. This slate-teal drops luminance 187 -> 105 and
/// takes the worst lit pair to 56.
pub fn zone_color() -> Color {
    Color::srgb_u8(40, 120, 150)
}

/// The hover is not an order. Keep it cyan so it remains distinct from every designation mark
/// and well away from the near-white stars and emitter faces.
pub fn hover_highlight_color() -> Color {
    Color::srgb_u8(80, 220, 210)
}

/// A stone item is rubble left standing at a dug tile, not a replacement block.
///
/// Until 2026-08-20 the item branch inserted a mesh and material without touching the spawned
/// `Transform`, so it inherited scale 1.0 — a cube the exact size of a terrain cube, in stone
/// material, standing in the tile it was just dug out of. Two consequences, both found by eye on
/// the vehicle and invisible to every instrument: a dug tile visually refilled, so a worked face
/// read as untouched rock; and the debris chips (within +/-0.39 of the tile centre) sat inside
/// the item's own +/-0.5 volume, so AC8's chips could never be seen where an item stood. The
/// capture self-test passed throughout, because the pixels DID change.
pub const STONE_ITEM_SCALE: f32 = 0.4;

/// Rests the shrunken item on the tile floor rather than leaving it floating mid-voxel, which is
/// where a centred sub-unit cube would otherwise sit. The chips are already low for this reason.
pub const STONE_ITEM_DROP: f32 = -(0.5 - STONE_ITEM_SCALE / 2.0);

#[cfg(test)]
mod flicker_tests {
    use super::*;

    #[test]
    fn flicker_is_bounded_distinct_and_deterministic() {
        assert_eq!(light_properties(LightKind::Torch).flicker_amplitude, 0.30);
        assert_eq!(
            light_properties(LightKind::Campfire).flicker_amplitude,
            0.40
        );
        // The band is asserted against HAND-WRITTEN literals, never against the table the
        // function reads. `flicker_scale` is `1.0 + amplitude * (..) / 1.3` with the bracket
        // normalised to +/-1.3, so a table-derived bound holds by construction for ANY
        // amplitude and cannot go red — the self-referential-test shape this project has
        // already been bitten by three times.
        for (kind, low, high) in [
            (LightKind::Torch, 0.70, 1.30),
            (LightKind::Campfire, 0.60, 1.40),
        ] {
            for step in 0..1000 {
                let scale = flicker_scale(kind, 6, step as f32 * 0.01);
                assert!(
                    (low..=high).contains(&scale),
                    "{kind:?} left its named band at {step}: {scale}"
                );
            }
        }
        // The band must also be REACHED, or a flicker of zero amplitude would satisfy it.
        let torch_peak = (0..1000)
            .map(|step| flicker_scale(LightKind::Torch, 6, step as f32 * 0.01))
            .fold(1.0f32, f32::max);
        assert!(
            torch_peak > 1.20,
            "the torch must actually use its band, peaked at {torch_peak}"
        );
        assert_ne!(
            flicker_scale(LightKind::Torch, 6, 1.0),
            flicker_scale(LightKind::Torch, 7, 1.0)
        );
        assert_ne!(
            flicker_scale(LightKind::Torch, 6, 1.0),
            flicker_scale(LightKind::Campfire, 6, 1.0)
        );
        assert_eq!(
            flicker_scale(LightKind::Torch, 6, 1.0),
            flicker_scale(LightKind::Torch, 6, 1.0)
        );
    }
}

/// RULED 2026-08-28 (Wolf, story 9.4): foliage shifts GREEN, not brown. It sat at `(55,73,84)`,
/// only **9.9** from stone `(60,70,92)` on the same Euclidean measure the marks are held to at a
/// 40.0 floor — trees separated from ground by snow cap and taper alone, the base cubes near
/// camouflage. `(44,100,58)` clears stone by **48.1** and soil by **49.6**. The epic said
/// "brown/green"; brown is unreachable because every terrain material must keep blue >= red (the
/// invariant asserted below), and brown is red over blue. Trees therefore separate on GREEN, the
/// axis the cool directional does not compress.
pub fn material_color(material: Material) -> Color {
    match material {
        Material::Stone => Color::srgb_u8(60, 70, 92),
        Material::Soil => Color::srgb_u8(56, 52, 62),
        Material::Ice => Color::srgb_u8(104, 128, 170),
        Material::Snow => Color::srgb_u8(136, 150, 178),
        Material::TreeTrunk => Color::srgb_u8(43, 47, 58),
        Material::TreeFoliage => Color::srgb_u8(44, 100, 58),
    }
}

/// Settled snow is brighter than the underlying snow terrain without becoming emissive white.
/// Trimmed ~8% at round 7: at the boot pitch the caps dominate the visible area, so the
/// field's measured brightness tracks THIS albedo more than the light table — boot4 proved
/// the light lever weak (a 2.6x ambient cut moved the field only 7%).
pub fn snow_cap_color() -> Color {
    Color::srgb_u8(146, 158, 184)
}

/// Snow caught on an exposed spruce crown. Started as the artifact's `SPRUCE_SNOW` (172,186,210)
/// and trimmed at round 7: the artifact shows that colour on thin sprite tops, while our cubes
/// show whole faces of it — every tree glowing at near-cap brightness is what made the boot4
/// foreground read as clutter.
pub fn foliage_snow_color() -> Color {
    Color::srgb_u8(156, 170, 196)
}

/// How many quantised steps the world-edge dissolve uses. Per-tile materials would mean one
/// material per cube; five shared steps read as a gradient and cost five handles per slot.
pub const RIM_LEVELS: usize = 13;

/// Blends a terrain colour toward the night sky so the world's boundary fades out instead of
/// ending on a lit cube face. Level 0 is untouched interior; the last level is pure sky.
///
/// NOTE: this dissolves the edge by COLOUR only. The tiles are still drawn, deliberately — the
/// draw set is pinned by AC18's 53,365-cube oracle and must not change to hide an edge.
pub fn rim_dissolved_color(base: Color, level: usize) -> Color {
    let steps = (RIM_LEVELS - 1) as f32;
    let blend = (level.min(RIM_LEVELS - 1) as f32 / steps).clamp(0.0, 1.0);
    let base = base.to_srgba();
    let sky = night_lighting().sky.to_srgba();
    Color::srgb(
        base.red + (sky.red - base.red) * blend,
        base.green + (sky.green - base.green) * blend,
        base.blue + (sky.blue - base.blue) * blend,
    )
}

pub fn entity_appearance(kind: EntityKind) -> EntityAppearance {
    match kind {
        EntityKind::Dwarf => EntityAppearance {
            color: Color::srgb_u8(151, 116, 96),
            scale: 0.65,
        },
        EntityKind::Torch => EntityAppearance {
            color: Color::srgb_u8(255, 140, 62),
            scale: 0.28,
        },
        EntityKind::Campfire => EntityAppearance {
            color: Color::srgb_u8(255, 173, 92),
            scale: 0.55,
        },
    }
}

#[cfg(test)]
mod tests {
    use bevy::color::ColorToPacked;
    use protocol::{DesignationKind, EntityKind, LightKind, Material};

    use super::{
        RIM_LEVELS, designation_color, entity_appearance, foliage_snow_color,
        hover_highlight_color, light_properties, material_color, night_lighting,
        rim_dissolved_color, snow_cap_color, zone_color,
    };

    #[test]
    fn appearance_tables_pin_the_cold_boot_palette() {
        let foliage = material_color(Material::TreeFoliage)
            .to_srgba()
            .to_u8_array_no_alpha();
        for (name, terrain) in [("stone", [60, 70, 92]), ("soil", [56, 52, 62])] {
            let separation = channel_distance(foliage, terrain);
            assert!(
                separation >= MIN_MARK_SEPARATION,
                "foliage {foliage:?} sits {separation:.1} from {name} {terrain:?}, inside the \
                 {MIN_MARK_SEPARATION} separation floor"
            );
        }

        let lights = [
            (LightKind::Torch, [255, 140, 62], 14_000_000.0, 20.0),
            (LightKind::Campfire, [255, 173, 92], 25_000_000.0, 28.0),
            // Dropped from 11M/16 on 2026-08-20: five moving lanterns over five static
            // emitters read blown out on the vehicle, which no range check can see.
            (LightKind::Lantern, [255, 195, 110], 5_000_000.0, 14.0),
        ];
        for (kind, rgb, intensity, range) in lights {
            let actual = light_properties(kind);
            assert_eq!(actual.color.to_srgba().to_u8_array_no_alpha(), rgb);
            assert_eq!(actual.intensity, intensity);
            assert_eq!(actual.range, range);
            let actual_rgb = actual.color.to_srgba().to_u8_array_no_alpha();
            assert!(
                actual_rgb[0] > actual_rgb[2],
                "every light table entry stays warm"
            );
        }

        let terrain = [
            (Material::Stone, [60, 70, 92]),
            (Material::Soil, [56, 52, 62]),
            (Material::Ice, [104, 128, 170]),
            (Material::Snow, [136, 150, 178]),
            (Material::TreeTrunk, [43, 47, 58]),
            (Material::TreeFoliage, [44, 100, 58]),
        ];
        for (material, rgb) in terrain {
            assert_eq!(
                material_color(material).to_srgba().to_u8_array_no_alpha(),
                rgb
            );
            let actual_rgb = material_color(material).to_srgba().to_u8_array_no_alpha();
            assert!(
                actual_rgb[2] >= actual_rgb[0],
                "night terrain stays blueward of red"
            );
        }

        let crown = foliage_snow_color().to_srgba().to_u8_array_no_alpha();
        assert_eq!(crown, [156, 170, 196]);
        assert!(crown[2] >= crown[0], "lit crowns stay on the cold side");
        assert!(
            crown[0]
                > material_color(Material::TreeFoliage)
                    .to_srgba()
                    .to_u8_array_no_alpha()[0],
            "a snow-laden crown must be visibly brighter than bare foliage"
        );

        let cap = snow_cap_color().to_srgba().to_u8_array_no_alpha();
        assert_eq!(cap, [146, 158, 184]);
        assert!(cap[2] >= cap[0], "settled snow stays on the cold side");
        assert!(
            cap[0]
                > material_color(Material::Snow)
                    .to_srgba()
                    .to_u8_array_no_alpha()[0],
            "the cap must remain visibly brighter than snow terrain"
        );

        let lighting = night_lighting();
        assert_eq!(lighting.sky.to_srgba().to_u8_array_no_alpha(), [5, 12, 28]);
        assert_eq!(
            lighting.star.to_srgba().to_u8_array_no_alpha(),
            [173, 196, 220]
        );
        assert_eq!(
            lighting.ambient.to_srgba().to_u8_array_no_alpha(),
            [120, 140, 165]
        );
        assert_eq!(
            lighting.directional.to_srgba().to_u8_array_no_alpha(),
            [150, 190, 180]
        );
        assert_eq!(
            lighting.aurora.to_srgba().to_u8_array_no_alpha(),
            [73, 157, 144]
        );
        assert_eq!(lighting.ambient_brightness, 4_500.0);
        assert_eq!(lighting.directional_illuminance, 22_000.0);

        let entities = [
            (EntityKind::Dwarf, [151, 116, 96], 0.65),
            (EntityKind::Torch, [255, 140, 62], 0.28),
            (EntityKind::Campfire, [255, 173, 92], 0.55),
        ];
        for (kind, rgb, scale) in entities {
            let actual = entity_appearance(kind);
            assert_eq!(actual.color.to_srgba().to_u8_array_no_alpha(), rgb);
            assert_eq!(actual.scale, scale);
        }
    }

    #[test]
    fn mark_colours_are_distinct_cold_literals() {
        let marks = [
            (
                "dig",
                designation_color(DesignationKind::Dig),
                [56, 132, 250],
            ),
            (
                "channel",
                designation_color(DesignationKind::Channel),
                [150, 96, 230],
            ),
            ("zone", zone_color(), [40, 120, 150]),
        ];
        let terrain = [
            Material::Stone,
            Material::Soil,
            Material::Ice,
            Material::Snow,
            Material::TreeTrunk,
            Material::TreeFoliage,
        ]
        .map(|material| material_color(material).to_srgba().to_u8_array_no_alpha())
        .into_iter()
        .chain([
            snow_cap_color().to_srgba().to_u8_array_no_alpha(),
            foliage_snow_color().to_srgba().to_u8_array_no_alpha(),
            super::debris_color().to_srgba().to_u8_array_no_alpha(),
        ])
        .collect::<Vec<_>>();

        for (name, color, expected) in marks {
            let rgb = color.to_srgba().to_u8_array_no_alpha();
            assert_eq!(rgb, expected, "{name} must retain its named colour");
            assert!(rgb[2] >= rgb[0], "{name} must remain cold or neutral");
            for other in &terrain {
                let separation = channel_distance(rgb, *other);
                assert!(
                    separation >= MIN_MARK_SEPARATION,
                    "{name} {rgb:?} sits {separation:.0} from terrain {other:?}, inside the \
                     {MIN_MARK_SEPARATION} floor — AC4/AC5 ask for VISUALLY distinguishable, and \
                     mere inequality is satisfied by two shades of the same pale blue"
                );
            }
        }
        for (i, (name, _, rgb)) in marks.iter().enumerate() {
            for (other_name, _, other) in marks.iter().skip(i + 1) {
                let separation = channel_distance(*rgb, *other);
                assert!(
                    separation >= MIN_MARK_SEPARATION,
                    "{name} and {other_name} sit {separation:.0} apart, inside the \
                     {MIN_MARK_SEPARATION} floor"
                );
            }
        }
        // Hand-copied from `crates/tui/src/palette.rs` — `gui` must never depend on `tui`, so
        // these are literals carrying a pointer to their source rather than an import. Wolf runs
        // both clients side by side, and dig shipped BYTE-IDENTICAL to the TUI's CHANNEL blue:
        // one RGB meaning two different orders across two windows. Mere non-identity is not the
        // guard, because two near-neighbour blues confuse just as well as one shared value.
        let tui_marks = [
            ("TUI dig", [232, 176, 72]),
            ("TUI channel", [92, 174, 224]),
            ("TUI zone", [88, 190, 118]),
        ];
        for (name, _, rgb) in marks {
            for (tui_name, tui_rgb) in tui_marks {
                let separation = channel_distance(rgb, tui_rgb);
                assert!(
                    separation >= MIN_MARK_SEPARATION,
                    "gui's {name} {rgb:?} sits {separation:.0} from {tui_name} {tui_rgb:?}, \
                     inside the {MIN_MARK_SEPARATION} floor — the two clients are read side by \
                     side, so one colour must not name two different orders"
                );
            }
        }
    }

    #[test]
    fn hover_highlight_colour_is_a_distinct_cold_literal() {
        let hover = hover_highlight_color().to_srgba().to_u8_array_no_alpha();
        assert!(
            hover[2] >= hover[0],
            "the hover must remain cold or neutral"
        );
        // The guard this replaces was `hover.iter().any(|channel| *channel < 240)`, which holds
        // for [255, 255, 239] and for pure red alike — it could not fail for the property it
        // named, the defect class the MIN_MARK_SEPARATION docstring below already records. What
        // "clear of the near-white" actually means is separation from the bright presentations
        // themselves, measured the same way every other separation in this file is.
        for (name, bright) in [
            ("the night sky's stars", [173, 196, 220]),
            ("a lit emitter face", [255, 195, 110]),
            ("white", [255, 255, 255]),
        ] {
            let separation = channel_distance(hover, bright);
            assert!(
                separation >= MIN_MARK_SEPARATION,
                "hover {hover:?} sits {separation:.0} from {name} {bright:?}, inside the \
                 {MIN_MARK_SEPARATION} floor — the hover must stay clear of the near-white"
            );
        }
        for mark in [[56, 132, 250], [150, 96, 230], [40, 120, 150]] {
            assert!(
                channel_distance(hover, mark) >= MIN_MARK_SEPARATION,
                "hover {hover:?} sits too close to mark {mark:?}"
            );
        }
        // The literal pin comes LAST so a colour perturbed toward a mark or toward the
        // near-white trips the property it violates, and names it, before this fires.
        assert_eq!(hover, [80, 220, 210]);
    }

    /// Euclidean RGB separation. Crude next to a perceptual metric, and deliberately so — this
    /// guards against a mark that is a near-neighbour of the terrain it is drawn on, which is a
    /// gross failure a crude measure catches perfectly well.
    fn channel_distance(a: [u8; 3], b: [u8; 3]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (f32::from(*x) - f32::from(*y)).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// The floor exists because the first values chosen here passed a `!=` check while sitting
    /// **16 units** from `Material::Snow` (channel) and **22** from `foliage_snow_color` (zone,
    /// which the terrain list did not even include). Both would have reached Wolf's live viewing
    /// as "distinguishable" on the strength of an assertion that could not fail for the property
    /// it claimed. 40 separates every current mark from every terrain presentation with room to
    /// spare; raise it if a mark is ever tuned toward the palette rather than away from it.
    const MIN_MARK_SEPARATION: f32 = 40.0;

    /// Re-derived at review-patch round 5 against the APPROVED ARTIFACT rather than against
    /// an assumed ratio. Measuring the artifact shows the camp is only ~1.3x the field in
    /// luminance (135.9 vs 104.3) — the warm/cold read is carried mostly by HUE, with the
    /// camp's R/B going 0.72 -> 0.97. The old assertion demanded a 3x luminance floor with no
    /// ceiling, which is satisfiable both by the 1/1000-scale table that shipped dark and by a
    /// blown-out white camp. A band plus a chromatic term pins what the artifact actually does.
    #[test]
    fn campfire_keeps_local_contrast_over_the_midtone_cold_fill() {
        let camp_distance_squared = 36.0;
        let campfire = light_properties(LightKind::Campfire);
        // THE PEAK, not the base. `flicker_lights` multiplies the base by `1 +/- amplitude`
        // every frame, so the brightness a viewer actually sees -- and the brightness that blows
        // a pool to white -- is the peak. Reading `.intensity` alone made this band structurally
        // blind to the whole flicker term: 6.1 raised the amplitude 3.6x and pushed the peak 26%
        // past what Wolf approved, and NOTHING here could go red. The only instrument that caught
        // it was Wolf's eye at a vehicle session, two epics later. Fixed 2026-08-22.
        let peak_intensity = campfire.intensity * (1.0 + campfire.flicker_amplitude);
        let warm_camp_lux = peak_intensity / (4.0 * std::f32::consts::PI * camp_distance_squared);
        let lighting = night_lighting();
        let cold_fill = lighting.ambient_brightness + lighting.directional_illuminance;
        let ratio = warm_camp_lux / cold_fill;

        // The band above is a broad sanity range and, on its own, STILL would not have caught
        // 6.1's raise -- 44.8M sits at ratio 3.74, comfortably inside 6.0. What was missing is a
        // pin to the value story 5.4 actually sized against the artifact Wolf approved: base
        // 32.0M at the then-amplitude of 0.11, i.e. a 35.52M peak. Hand-written here rather than
        // derived from the table, so it cannot drift with the thing it is guarding.
        const APPROVED_PEAK: f32 = 35_520_000.0;
        assert!(
            peak_intensity <= APPROVED_PEAK,
            "the campfire's flicker PEAK is {peak_intensity}, above the {APPROVED_PEAK} Wolf \
             approved at 5.4 -- raising either the base or the amplitude past this is what blew \
             the camp out at 6.2's sign-off, and neither shows up in the still frame"
        );

        assert!(
            ratio >= 1.2,
            "the campfire must lift its six-unit neighbourhood above the cold fill; ratio {ratio}"
        );
        assert!(
            ratio <= 6.0,
            "the campfire must not blow the camp to white — only emissive approaches white (AC9); ratio {ratio}"
        );
    }

    #[test]
    fn the_rim_dissolve_runs_from_the_untouched_material_to_the_bare_sky() {
        let snow = material_color(Material::Snow);
        let sky = night_lighting().sky.to_srgba().to_u8_array_no_alpha();

        assert_eq!(
            rim_dissolved_color(snow, 0)
                .to_srgba()
                .to_u8_array_no_alpha(),
            snow.to_srgba().to_u8_array_no_alpha(),
            "interior terrain must be untouched by the edge treatment"
        );
        assert_eq!(
            rim_dissolved_color(snow, RIM_LEVELS - 1)
                .to_srgba()
                .to_u8_array_no_alpha(),
            sky,
            "the outermost step must be indistinguishable from the sky"
        );

        // Monotonic: every step must move toward the sky, or the edge reads as banding.
        let distances: Vec<f32> = (0..RIM_LEVELS)
            .map(|level| {
                let c = rim_dissolved_color(snow, level).to_srgba();
                let s = night_lighting().sky.to_srgba();
                (c.red - s.red).abs() + (c.green - s.green).abs() + (c.blue - s.blue).abs()
            })
            .collect();
        for pair in distances.windows(2) {
            assert!(
                pair[1] < pair[0],
                "each rim step must close on the sky; got {distances:?}"
            );
        }
    }

    /// The half of the warm/cold invariant the luminance ratio cannot express.
    #[test]
    fn the_cold_fill_is_chromatically_cold_and_the_camp_is_chromatically_warm() {
        let lighting = night_lighting();
        let ambient = lighting.ambient.to_srgba().to_u8_array_no_alpha();
        assert!(
            ambient[2] > ambient[0],
            "the ambient fill must stay blueward of red"
        );

        let ambient_warmth = ambient[0] as f32 / ambient[2] as f32;
        for kind in [LightKind::Torch, LightKind::Campfire, LightKind::Lantern] {
            let light = light_properties(kind)
                .color
                .to_srgba()
                .to_u8_array_no_alpha();
            let light_warmth = light[0] as f32 / light[2] as f32;
            assert!(
                light_warmth >= ambient_warmth * 2.0,
                "{kind:?} must read warm against the cold fill; {light_warmth} vs {ambient_warmth}"
            );
        }
    }
}
