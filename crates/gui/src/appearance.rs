use bevy::prelude::Color;
use protocol::{EntityKind, LightKind, Material};

#[derive(Debug, Clone, Copy)]
pub struct LightProperties {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
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
    pub directional_illuminance: f32,
}

pub fn night_lighting() -> NightLighting {
    NightLighting {
        sky: Color::srgb_u8(5, 12, 28),
        star: Color::srgb_u8(173, 196, 220),
        ambient: Color::srgb_u8(47, 76, 104),
        ambient_brightness: 2_000.0,
        aurora: Color::srgb_u8(73, 157, 144),
        directional_illuminance: 1_500.0,
    }
}

pub fn light_properties(kind: LightKind) -> LightProperties {
    // NOTE: lights are static until story 6.1 adds the flicker column.
    match kind {
        LightKind::Torch => LightProperties {
            color: Color::srgb_u8(255, 140, 62),
            intensity: 2_500_000.0,
            range: 18.0,
        },
        LightKind::Campfire => LightProperties {
            color: Color::srgb_u8(255, 173, 92),
            intensity: 6_000_000.0,
            range: 26.0,
        },
        LightKind::Lantern => LightProperties {
            color: Color::srgb_u8(255, 195, 110),
            intensity: 2_000_000.0,
            range: 14.0,
        },
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
pub fn snow_cap_color() -> Color {
    Color::srgb_u8(158, 170, 196)
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
        entity_appearance, light_properties, material_color, night_lighting, snow_cap_color,
    };

    #[test]
    fn appearance_tables_pin_the_cold_boot_palette() {
        let lights = [
            (LightKind::Torch, [255, 140, 62], 2_500_000.0, 18.0),
            (LightKind::Campfire, [255, 173, 92], 6_000_000.0, 26.0),
            (LightKind::Lantern, [255, 195, 110], 2_000_000.0, 14.0),
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

        let cap = snow_cap_color().to_srgba().to_u8_array_no_alpha();
        assert_eq!(cap, [158, 170, 196]);
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
            [47, 76, 104]
        );
        assert_eq!(
            lighting.aurora.to_srgba().to_u8_array_no_alpha(),
            [73, 157, 144]
        );
        assert_eq!(lighting.ambient_brightness, 2_000.0);
        assert_eq!(lighting.directional_illuminance, 1_500.0);

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
    fn campfire_keeps_local_contrast_over_the_midtone_cold_fill() {
        let camp_distance_squared = 36.0;
        let warm_camp_lux = light_properties(LightKind::Campfire).intensity
            / (4.0 * std::f32::consts::PI * camp_distance_squared);
        let lighting = night_lighting();
        let cold_fill = lighting.ambient_brightness + lighting.directional_illuminance;

        assert!(
            warm_camp_lux >= cold_fill * 3.0,
            "the campfire must dominate its six-unit neighbourhood without blackening the field"
        );
    }
}
