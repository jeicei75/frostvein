use bevy::prelude::{Component, Transform, Vec3};

use crate::transform::world_to_render;

const MIN_PITCH: f32 = 0.15;
const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.15;
const BOOT_YAW: f32 = 0.7;
const BOOT_PITCH: f32 = 0.8;
const BOOT_DISTANCE: f32 = 90.0;

#[derive(Component, Debug, Clone, Copy)]
pub struct CameraRig {
    pub focus: [i32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl CameraRig {
    pub fn new(focus: [i32; 3]) -> Self {
        Self {
            focus,
            yaw: BOOT_YAW,
            pitch: BOOT_PITCH,
            distance: BOOT_DISTANCE,
        }
    }

    pub fn orbit(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw += yaw_delta;
        self.pitch = (self.pitch + pitch_delta).clamp(MIN_PITCH, MAX_PITCH);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(4.0, 500.0);
    }

    pub fn transform(&self) -> Transform {
        let focus = world_to_render(self.focus);
        let horizontal = self.distance * self.pitch.cos();
        let offset = Vec3::X * (horizontal * self.yaw.cos())
            + Vec3::Y * (self.distance * self.pitch.sin())
            + Vec3::Z * (horizontal * self.yaw.sin());
        Transform::from_translation(focus + offset).looking_at(focus, Vec3::Y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbit_reaches_every_yaw_and_clamps_pitch() {
        let mut rig = CameraRig::new([64, 64, 9]);
        rig.orbit(std::f32::consts::TAU * 3.0, -100.0);
        assert!(rig.yaw > std::f32::consts::TAU);
        assert_eq!(rig.pitch, MIN_PITCH);
        rig.orbit(0.0, 100.0);
        assert_eq!(rig.pitch, MAX_PITCH);
    }

    #[test]
    fn zoom_never_moves_the_focus() {
        let mut rig = CameraRig::new([64, 64, 9]);
        let focus = world_to_render(rig.focus);
        rig.zoom(-10_000.0);
        assert!((rig.transform().translation.distance(focus) - 4.0).abs() < 0.001);
        rig.zoom(10_000.0);
        assert!((rig.transform().translation.distance(focus) - 500.0).abs() < 0.001);
    }
}
