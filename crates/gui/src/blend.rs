use bevy::prelude::{Resource, Vec3};

use crate::transform::world_to_render;

/// Fast-forward can deliver at 50 ticks per second. A longer gap means a stalled server rather
/// than a slower simulation, so it must not stretch presentation across the whole stall.
pub const MIN_TICK_INTERVAL: f32 = 0.02;
pub const MAX_TICK_INTERVAL: f32 = 0.50;

/// Client-local playback clock for interpolation between delivered mirror ticks.
#[derive(Resource, Debug)]
pub struct TickClock {
    elapsed: f32,
    interval: f32,
    last_tick: u64,
}

impl Default for TickClock {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            interval: MAX_TICK_INTERVAL,
            last_tick: 0,
        }
    }
}

impl TickClock {
    pub fn advance(&mut self, seconds: f32) {
        self.elapsed += seconds;
    }

    pub fn observe_tick(&mut self, tick: u64) {
        if tick > self.last_tick {
            self.interval = self.elapsed.clamp(MIN_TICK_INTERVAL, MAX_TICK_INTERVAL);
            self.elapsed = 0.0;
            self.last_tick = tick;
        }
    }

    pub fn reset(&mut self, tick: u64) {
        self.elapsed = 0.0;
        self.last_tick = tick;
    }

    pub fn factor(&self) -> f32 {
        (self.elapsed / self.interval).clamp(0.0, 1.0)
    }
}

/// Returns only a delivered position: previous-to-current while in range, otherwise current.
pub fn blended_translation(previous: Option<[i32; 3]>, current: [i32; 3], factor: f32) -> Vec3 {
    match previous {
        Some(previous) => {
            world_to_render(previous).lerp(world_to_render(current), factor.clamp(0.0, 1.0))
        }
        None => world_to_render(current),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measured_tick_interval_is_bounded_and_never_extrapolates() {
        let mut clock = TickClock::default();
        clock.advance(10.0);
        clock.observe_tick(1);
        clock.advance(1.0);
        assert_eq!(clock.factor(), 1.0);
    }

    #[test]
    fn midpoint_and_snap_are_literal_wire_positions() {
        assert_eq!(
            blended_translation(Some([2, 3, 4]), [4, 3, 4], 0.5),
            Vec3::new(3.0, 4.0, -3.0)
        );
        assert_eq!(
            blended_translation(Some([2, 3, 4]), [4, 3, 4], 1.0),
            Vec3::new(4.0, 4.0, -3.0)
        );
        assert_eq!(
            blended_translation(None, [4, 3, 4], 0.5),
            Vec3::new(4.0, 4.0, -3.0)
        );
        assert_eq!(
            blended_translation(Some([2, 3, 4]), [4, 3, 4], 9.0),
            Vec3::new(4.0, 4.0, -3.0)
        );
    }
}
