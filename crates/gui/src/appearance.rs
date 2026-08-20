use bevy::prelude::Color;
use protocol::{EntityKind, LightKind, Material};

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
            intensity: 32_000_000.0,
            range: 28.0,
            flicker_amplitude: 0.40,
            flicker_hz: 0.9,
        },
        LightKind::Lantern => LightProperties {
            color: Color::srgb_u8(255, 195, 110),
            intensity: 11_000_000.0,
            range: 16.0,
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

pub fn material_color(material: Material) -> Color {
    match material {
        Material::Stone => Color::srgb_u8(60, 70, 92),
        Material::Soil => Color::srgb_u8(56, 52, 62),
        Material::Ice => Color::srgb_u8(104, 128, 170),
        Material::Snow => Color::srgb_u8(136, 150, 178),
        Material::TreeTrunk => Color::srgb_u8(43, 47, 58),
        Material::TreeFoliage => Color::srgb_u8(55, 73, 84),
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
    use protocol::{EntityKind, LightKind, Material};

    use super::{
        RIM_LEVELS, entity_appearance, foliage_snow_color, light_properties, material_color,
        night_lighting, rim_dissolved_color, snow_cap_color,
    };

    #[test]
    fn appearance_tables_pin_the_cold_boot_palette() {
        let lights = [
            (LightKind::Torch, [255, 140, 62], 14_000_000.0, 20.0),
            (LightKind::Campfire, [255, 173, 92], 32_000_000.0, 28.0),
            (LightKind::Lantern, [255, 195, 110], 11_000_000.0, 16.0),
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
            (Material::TreeFoliage, [55, 73, 84]),
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

    /// Re-derived at review-patch round 5 against the APPROVED ARTIFACT rather than against
    /// an assumed ratio. Measuring the artifact shows the camp is only ~1.3x the field in
    /// luminance (135.9 vs 104.3) — the warm/cold read is carried mostly by HUE, with the
    /// camp's R/B going 0.72 -> 0.97. The old assertion demanded a 3x luminance floor with no
    /// ceiling, which is satisfiable both by the 1/1000-scale table that shipped dark and by a
    /// blown-out white camp. A band plus a chromatic term pins what the artifact actually does.
    #[test]
    fn campfire_keeps_local_contrast_over_the_midtone_cold_fill() {
        let camp_distance_squared = 36.0;
        let warm_camp_lux = light_properties(LightKind::Campfire).intensity
            / (4.0 * std::f32::consts::PI * camp_distance_squared);
        let lighting = night_lighting();
        let cold_fill = lighting.ambient_brightness + lighting.directional_illuminance;
        let ratio = warm_camp_lux / cold_fill;

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
