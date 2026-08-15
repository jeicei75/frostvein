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
/// Four 0.28-unit torches plus one 0.55-unit campfire project to fewer than 100 pixels at
/// the 90-unit boot distance; this floor therefore requires light pools, not source faces.
pub const WARM_PIXEL_FLOOR: usize = 100;

pub fn warm_lit_pixels(rgba: &[[u8; 4]]) -> usize {
    rgba.iter()
        .filter(|pixel| pixel[0].saturating_sub(pixel[2]) > WARM_RED_OVER_BLUE)
        .count()
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
    println!("capture range check: warm-lit pixels={warm}");
    // NOTE: confirm this source-face-derived floor on the native-Windows vehicle run.
    assert!(
        warm >= WARM_PIXEL_FLOOR,
        "capture contains fewer than {WARM_PIXEL_FLOOR} warm-lit pixels"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bgra_capture_bytes_decode_before_warm_pixel_detection() {
        let pixels = decode_rgba8(&[10, 120, 240, 255], TextureFormat::Bgra8Unorm);

        assert_eq!(pixels, vec![[240, 120, 10, 255]]);
        assert_eq!(warm_lit_pixels(&pixels), 1);
    }

    #[test]
    fn warm_pixel_floor_exceeds_the_emitter_faces_alone() {
        assert!(64 < WARM_PIXEL_FLOOR);
    }
}
