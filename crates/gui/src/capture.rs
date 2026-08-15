use std::path::PathBuf;

use bevy::{
    app::AppExit,
    ecs::message::MessageWriter,
    prelude::{Commands, On, ResMut, Resource},
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

pub fn warm_lit_pixels(rgba: &[[u8; 4]]) -> usize {
    rgba.iter()
        .filter(|pixel| pixel[0].saturating_sub(pixel[2]) > WARM_RED_OVER_BLUE)
        .count()
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
    let pixels = bytes
        .chunks_exact(4)
        .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
        .collect::<Vec<_>>();
    assert!(
        pixels.iter().any(|pixel| pixel[..3] != [0, 0, 0]),
        "capture is black"
    );
    assert!(
        pixels.windows(2).any(|pair| pair[0] != pair[1]),
        "capture is uniform"
    );
    let warm = warm_lit_pixels(&pixels);
    assert!(warm > 0, "capture contains no warm-lit pixels");
    println!("capture range check: warm-lit pixels={warm}");
}
