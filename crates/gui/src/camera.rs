use bevy::prelude::{Component, Transform, Vec2, Vec3};

use crate::transform::world_to_render;

const MIN_PITCH: f32 = 0.15;
const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.15;
const BOOT_YAW: f32 = 0.7;
const BOOT_PITCH: f32 = 0.45;
const BOOT_DISTANCE: f32 = 90.0;
// The camera orbits the camp, but composes the larger valley behind it into the frame.
// The push runs along the view direction and straight up — NEVER along the camera's right
// vector, which slides the camp sideways out of the approved composition (the world -Z
// push this replaced put it at 23% of the frame instead of the artifact's 48%).
// Re-solved 2026-08-16 for Wolf's live ruling that the artifact's top-third sky is too much
// in the real frame: the skyline now sits at 24% from the top instead of 30%.
const BOOT_COMPOSITION_FORWARD: f32 = 33.0;
const BOOT_COMPOSITION_LIFT: f32 = -0.5;

/// The direction the boot camera looks, flattened onto the ground plane. The sky geometry
/// is placed against this so the aurora sits where the opening frame can see it.
pub fn boot_horizontal_forward() -> Vec3 {
    Vec3::new(-BOOT_YAW.cos(), 0.0, -BOOT_YAW.sin())
}

/// The boot composition push, expressed in the boot camera's own view plane.
fn boot_composition_offset() -> Vec3 {
    boot_horizontal_forward() * BOOT_COMPOSITION_FORWARD + Vec3::Y * BOOT_COMPOSITION_LIFT
}
pub const BOOT_VERTICAL_FOV: f32 = std::f32::consts::FRAC_PI_4;
pub const BOOT_ASPECT_RATIO: f32 = 16.0 / 9.0;

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
        let focus = self.composition_target();
        let horizontal = self.distance * self.pitch.cos();
        let offset = Vec3::X * (horizontal * self.yaw.cos())
            + Vec3::Y * (self.distance * self.pitch.sin())
            + Vec3::Z * (horizontal * self.yaw.sin());
        Transform::from_translation(focus + offset).looking_at(focus, Vec3::Y)
    }

    fn composition_target(&self) -> Vec3 {
        // Keep the camp in front of the camera at close zoom while retaining the approved
        // composition at the boot distance and beyond.
        let composition_scale = (self.distance / BOOT_DISTANCE).min(1.0);
        world_to_render(self.focus) + boot_composition_offset() * composition_scale
    }

    /// Projects a render-space point to normalized screen coordinates at this rig's camera.
    pub fn project_render_point(&self, point: Vec3) -> Option<Vec2> {
        self.project_render_point_with_depth(point)
            .map(|(screen, _)| screen)
    }

    /// The projection above, plus the point's distance along the view axis.
    ///
    /// The depth is what lets a screen-space consumer size a tolerance in pixels: a tile's
    /// apparent half-extent shrinks as `1/depth`, so a fixed pixel window is honest at exactly
    /// one distance. The capture instrument's oracle needs it for that reason and no other.
    pub fn project_render_point_with_depth(&self, point: Vec3) -> Option<(Vec2, f32)> {
        let camera = self.transform();
        let offset = point - camera.translation;
        let depth = offset.dot(camera.forward().as_vec3());
        if depth <= 0.0 {
            return None;
        }
        let half_vertical = (BOOT_VERTICAL_FOV * 0.5).tan();
        Some((
            Vec2::new(
                0.5 + offset.dot(camera.right().as_vec3())
                    / (2.0 * depth * half_vertical * BOOT_ASPECT_RATIO),
                0.5 - offset.dot(*camera.up()) / (2.0 * depth * half_vertical),
            ),
            depth,
        ))
    }

    /// Projects a simulation-world point to normalized screen coordinates.
    pub fn project_world_point(&self, point: [i32; 3]) -> Option<Vec2> {
        self.project_render_point(world_to_render(point))
    }

    /// Projects a simulation-world point to normalized screen coordinates and its view depth.
    pub fn project_world_point_with_depth(&self, point: [i32; 3]) -> Option<(Vec2, f32)> {
        self.project_render_point_with_depth(world_to_render(point))
    }
}

/// Where world NORTH points on this camera's screen, as one of eight ASCII labels.
///
/// This client had no orientation cue of any kind, and its screen axes are not the world's: the
/// boot camera is yawed 0.7 rad (~40 degrees) and orbits freely with `A`/`D`, so "up" here is a
/// world diagonal that MOVES. The TUI's screen axes, by contrast, ARE the world axes. Wolf hit the
/// consequence on 2026-08-27 — dug at what read as north here, found the stone to the west in the
/// TUI — and the honest answer was that neither client was wrong and neither said which way it
/// faced. At boot, world north points DOWN-LEFT on this screen, which nobody would guess.
///
/// North is world -y, matching the TUI, where -y is up the screen.
pub fn north_on_screen(rig: &CameraRig) -> &'static str {
    let focus = rig.focus;
    // Two projected points rather than an analytic derivation: this reuses the SAME projection
    // the picking ray and every capture assertion go through, so a compass that disagrees with
    // what is drawn is not possible.
    let here = rig.project_world_point(focus);
    let north = rig.project_world_point([focus[0], focus[1] - NORTH_PROBE_TILES, focus[2]]);
    let (Some(here), Some(north)) = (here, north) else {
        // Never guess a bearing. An unprojectable probe means the compass does not know, and a
        // compass that invents a direction is worse than one that admits it cannot say.
        return "?";
    };
    let delta = north - here;
    if delta.length_squared() < f32::EPSILON {
        return "?";
    }
    // Screen y runs DOWN, so a negative dy is up the screen.
    let sector = (delta.y.atan2(delta.x) / std::f32::consts::FRAC_PI_4).round() as i32;
    match sector.rem_euclid(8) {
        0 => "right",
        1 => "down-right",
        2 => "down",
        3 => "down-left",
        4 => "left",
        5 => "up-left",
        6 => "up",
        _ => "up-right",
    }
}

/// Far enough that the projected delta is not swallowed by rounding, close enough that a modest
/// world still contains it.
const NORTH_PROBE_TILES: i32 = 8;

#[cfg(test)]
mod tests {
    use super::*;

    /// Pins the measured boot bearing. At `BOOT_YAW` world north points DOWN-LEFT on screen —
    /// measured 2026-08-27, and the whole reason a compass exists. A camera change that quietly
    /// moves this must show up here rather than in a confused reading months later.
    #[test]
    fn the_compass_reports_where_north_actually_is_and_turns_with_the_camera() {
        let rig = CameraRig::new([64, 64, 9]);
        assert_eq!(
            north_on_screen(&rig),
            "down-left",
            "at boot yaw, world north is down-left on this screen"
        );

        // A quarter turn must move the needle, and by roughly a quarter of the compass.
        let mut turned = CameraRig::new([64, 64, 9]);
        turned.orbit(std::f32::consts::FRAC_PI_2, 0.0);
        let after = north_on_screen(&turned);
        assert_ne!(
            after, "down-left",
            "a compass that does not turn with the camera is a decoration"
        );
        // And a full turn must bring it back, so the needle tracks yaw rather than drifting.
        let mut round = CameraRig::new([64, 64, 9]);
        round.orbit(std::f32::consts::TAU, 0.0);
        assert_eq!(north_on_screen(&round), "down-left");
    }

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
        rig.zoom(-10_000.0);
        assert!(
            (rig.transform()
                .translation
                .distance(rig.composition_target())
                - 4.0)
                .abs()
                < 0.001
        );
        rig.zoom(10_000.0);
        assert!(
            (rig.transform()
                .translation
                .distance(rig.composition_target())
                - 500.0)
                .abs()
                < 0.001
        );
    }

    #[test]
    fn boot_composition_places_the_camp_low_and_the_skyline_at_the_top_third() {
        const TOLERANCE: f32 = 0.03;

        let rig = CameraRig::new([64, 64, 9]);
        let camp = rig
            .project_world_point([64, 64, 9])
            .expect("the camp must be in front of the boot camera");
        let far_terrain = rig
            .project_render_point(Vec3::new(64.0, 26.0, -128.0))
            .expect("the skyline must be in front of the boot camera");

        assert!(
            (camp.x - 0.48).abs() <= TOLERANCE,
            "camp must sit near the approved horizontal anchor; measured {}",
            camp.x
        );
        assert!(
            (camp.y - 0.78).abs() <= TOLERANCE,
            "camp must sit at 78% of the frame from the top; measured {}",
            camp.y
        );
        assert!(
            (far_terrain.y - 0.24).abs() <= TOLERANCE,
            "skyline must leave the top quarter to sky; measured {}",
            far_terrain.y
        );
    }

    #[test]
    fn boot_composition_never_pushes_along_the_camera_right_vector() {
        // The defect this pins: an offset with a lateral component slides the camp sideways
        // as the boot yaw changes, so the vertical framing assertions above stay green while
        // the composition drifts off-frame. Assert the mechanism, not just the symptom.
        let rig = CameraRig::new([64, 64, 9]);
        let camera = rig.transform();
        let lateral = boot_composition_offset().dot(camera.right().as_vec3());
        assert!(
            lateral.abs() <= 0.01,
            "the boot composition push must stay in the camera's view plane; measured {lateral} units along right"
        );
    }

    #[test]
    fn zoom_limits_keep_the_camp_in_front_of_the_camera() {
        let mut rig = CameraRig::new([64, 64, 9]);
        rig.zoom(-10_000.0);
        assert!(
            rig.project_world_point([64, 64, 9]).is_some(),
            "the close zoom limit must keep the camp in front of the camera"
        );
        rig.zoom(10_000.0);
        assert!(
            rig.project_world_point([64, 64, 9]).is_some(),
            "the vista zoom limit must keep the camp in front of the camera"
        );
    }
}
