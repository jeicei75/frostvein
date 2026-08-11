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
            .observe(exit_after_capture);
    }
}

fn exit_after_capture(_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
