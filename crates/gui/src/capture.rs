use std::path::PathBuf;

use bevy::{
    app::AppExit,
    ecs::message::MessageWriter,
    prelude::{Commands, On, ResMut, Resource},
    render::render_resource::TextureFormat,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
};

#[derive(Resource)]
pub struct CaptureState {
    path: PathBuf,
    frames: u32,
    elapsed: u32,
    requested: bool,
}

pub const WARM_RED_OVER_BLUE: u8 = 30;
/// `capture-2026-08-15T1717-boot.png` measured 17,648 warm-lit pixels at the boot framing.
/// The old 100 floor (and its ~64 emitter-face estimate) could not distinguish missing point
/// lights from their emissive source faces; 3,000 leaves framing headroom while requiring pools.
pub const WARM_PIXEL_FLOOR: usize = 3_000;

pub fn warm_lit_pixels(rgba: &[[u8; 4]]) -> usize {
    rgba.iter()
        .filter(|pixel| pixel[0].saturating_sub(pixel[2]) > WARM_RED_OVER_BLUE)
        .count()
}

/// The centre of the valley floor, as fractions of the frame. Deliberately inside the world
/// edge dissolve and below the skyline, so the sample is terrain and not sky or rim.
const GROUND_WINDOW_X: (f32, f32) = (0.25, 0.75);
const GROUND_WINDOW_Y: (f32, f32) = (0.50, 0.90);

/// AC9's value discipline made measurable. Sampled in the window above, the APPROVED ARTIFACT
/// reads a median sRGB luminance of 123; the round-4 capture read 21 — a night scene that is
/// simply black. No headless test can see this, so the instrument carries it. The floor sits
/// between the two so the dark-field failure class cannot pass while the light budget is free
/// to land anywhere near the target.
pub const GROUND_LUMINANCE_FLOOR: u8 = 70;

fn luminance(pixel: [u8; 4]) -> f32 {
    0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32
}

/// Median luminance of the valley floor. Median, not mean, so a handful of blown-out emitter
/// faces cannot carry a black field over the floor.
pub fn median_ground_luminance(pixels: &[[u8; 4]], width: u32, height: u32) -> u8 {
    let column_range =
        (width as f32 * GROUND_WINDOW_X.0) as u32..(width as f32 * GROUND_WINDOW_X.1).ceil() as u32;
    let row_range = (height as f32 * GROUND_WINDOW_Y.0) as u32
        ..(height as f32 * GROUND_WINDOW_Y.1).ceil() as u32;
    let mut samples: Vec<u8> = row_range
        .flat_map(|row| column_range.clone().map(move |column| (row, column)))
        .filter_map(|(row, column)| pixels.get((row * width + column) as usize))
        .map(|pixel| luminance(*pixel).round() as u8)
        .collect();
    if samples.is_empty() {
        return 0;
    }
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn decode_rgba8(bytes: &[u8], format: TextureFormat) -> Vec<[u8; 4]> {
    assert!(
        bytes.len().is_multiple_of(4),
        "capture pixel data must contain whole four-channel pixels"
    );
    match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => bytes
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
            .collect(),
        TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => bytes
            .chunks_exact(4)
            .map(|pixel| [pixel[2], pixel[1], pixel[0], pixel[3]])
            .collect(),
        _ => panic!("capture range check cannot decode {format:?} pixels"),
    }
}

impl CaptureState {
    pub fn new(path: PathBuf, frames: u32) -> Self {
        Self {
            path,
            frames,
            elapsed: 0,
            requested: false,
        }
    }
}

/// Captures from the primary window after the real render loop has advanced N frames.
pub fn capture_after_frames(mut commands: Commands, mut capture: ResMut<CaptureState>) {
    if capture.requested {
        return;
    }
    capture.elapsed += 1;
    if capture.elapsed >= capture.frames {
        capture.requested = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(capture.path.clone()))
            .observe(validate_capture_ranges)
            .observe(exit_after_capture);
    }
}

fn exit_after_capture(_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

fn validate_capture_ranges(event: On<ScreenshotCaptured>) {
    let bytes = event
        .image
        .data
        .as_deref()
        .expect("capture screenshot must include pixel data");
    let pixels = decode_rgba8(bytes, event.image.texture_descriptor.format);
    assert!(
        pixels.iter().any(|pixel| pixel[..3] != [0, 0, 0]),
        "capture is black"
    );
    assert!(
        pixels.windows(2).any(|pair| pair[0] != pair[1]),
        "capture is uniform"
    );
    let warm = warm_lit_pixels(&pixels);
    let size = event.image.texture_descriptor.size;
    let ground = median_ground_luminance(&pixels, size.width, size.height);
    println!("capture range check: warm-lit pixels={warm} ground-median-luminance={ground}");
    // NOTE: confirm this source-face-derived floor on the native-Windows vehicle run.
    assert!(
        warm >= WARM_PIXEL_FLOOR,
        "capture contains fewer than {WARM_PIXEL_FLOOR} warm-lit pixels"
    );
    assert!(
        ground >= GROUND_LUMINANCE_FLOOR,
        "the valley floor reads {ground}, below the {GROUND_LUMINANCE_FLOOR} value floor — \
         the frame is a black field, not a lit night"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-written oracle: a 4x4 frame whose centre window is exactly the four pixels at
    /// rows 2..4, columns 1..3. Values chosen so the median is a value that appears ONCE, so a
    /// mean or an off-by-one window cannot produce it by accident.
    #[test]
    fn the_ground_median_reads_the_valley_floor_and_ignores_the_sky() {
        let sky = [0u8, 0, 0, 255];
        let grey = |v: u8| [v, v, v, 255];
        // Rec.709 of a neutral grey is the grey itself.
        let mut frame = vec![sky; 16];
        frame[2 * 4 + 1] = grey(10);
        frame[2 * 4 + 2] = grey(90);
        frame[3 * 4 + 1] = grey(100);
        frame[3 * 4 + 2] = grey(200);

        assert_eq!(median_ground_luminance(&frame, 4, 4), 100);
        // The sky rows are bright here and must NOT be able to lift the reading.
        let mut bright_sky = frame.clone();
        for pixel in bright_sky.iter_mut().take(8) {
            *pixel = grey(255);
        }
        assert_eq!(median_ground_luminance(&bright_sky, 4, 4), 100);
    }

    #[test]
    fn a_black_field_fails_the_value_floor_that_a_lit_one_passes() {
        let black = vec![[12u8, 14, 20, 255]; 64];
        assert!(median_ground_luminance(&black, 8, 8) < GROUND_LUMINANCE_FLOOR);
        let lit = vec![[95u8, 112, 129, 255]; 64];
        assert!(median_ground_luminance(&lit, 8, 8) >= GROUND_LUMINANCE_FLOOR);
    }

    #[test]
    fn bgra_capture_bytes_decode_before_warm_pixel_detection() {
        let pixels = decode_rgba8(&[10, 120, 240, 255], TextureFormat::Bgra8Unorm);

        assert_eq!(pixels, vec![[240, 120, 10, 255]]);
        assert_eq!(warm_lit_pixels(&pixels), 1);
    }
}
