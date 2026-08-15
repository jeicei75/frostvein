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
        ambient_brightness: 8.0,
        aurora: Color::srgb_u8(73, 157, 144),
        directional_illuminance: 8.0,
    }
}

pub fn light_properties(kind: LightKind) -> LightProperties {
    // NOTE: lights are static until story 6.1 adds the flicker column.
    match kind {
        LightKind::Torch => LightProperties {
            color: Color::srgb_u8(255, 140, 62),
            intensity: 250_000.0,
            range: 18.0,
        },
        LightKind::Campfire => LightProperties {
            color: Color::srgb_u8(255, 173, 92),
            intensity: 500_000.0,
            range: 26.0,
        },
        LightKind::Lantern => LightProperties {
            color: Color::srgb_u8(255, 195, 110),
            intensity: 180_000.0,
            range: 14.0,
        },
    }
}

pub fn material_color(material: Material) -> Color {
    match material {
        Material::Stone => Color::srgb_u8(40, 57, 82),
        Material::Soil => Color::srgb_u8(56, 69, 80),
        Material::Ice => Color::srgb_u8(84, 133, 160),
        Material::Snow => Color::srgb_u8(118, 139, 157),
        Material::TreeTrunk => Color::srgb_u8(43, 47, 58),
        Material::TreeFoliage => Color::srgb_u8(55, 73, 84),
    }
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

    use super::{entity_appearance, light_properties, material_color, night_lighting};

    #[test]
    fn appearance_tables_pin_the_cold_boot_palette() {
        let lights = [
            (LightKind::Torch, [255, 140, 62], 250_000.0, 18.0),
            (LightKind::Campfire, [255, 173, 92], 500_000.0, 26.0),
            (LightKind::Lantern, [255, 195, 110], 180_000.0, 14.0),
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
            (Material::Stone, [40, 57, 82]),
            (Material::Soil, [56, 69, 80]),
            (Material::Ice, [84, 133, 160]),
            (Material::Snow, [118, 139, 157]),
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

        let lighting = night_lighting();
        assert_eq!(lighting.ambient_brightness, 8.0);
        assert_eq!(lighting.directional_illuminance, 8.0);

        let camp_distance_squared = 36.0;
        let warm_camp_lux = light_properties(LightKind::Campfire).intensity
            / (4.0 * std::f32::consts::PI * camp_distance_squared);
        let cold_fill = lighting.ambient_brightness + lighting.directional_illuminance;
        assert!(
            warm_camp_lux >= cold_fill * 60.0,
            "the campfire must dominate cold fill six world units from camp"
        );

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
}
