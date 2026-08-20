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
            // NOTE: several deltas can drain in one frame, and every one after the first measures
            // ~0 elapsed because its predecessor just zeroed the clock. That is a burst, not a
            // faster cadence: taking it would pin the interval to its floor and saturate the
            // blend for the rest of the frame. Keep the last real measurement instead.
            if self.elapsed >= MIN_TICK_INTERVAL {
                self.interval = self.elapsed.clamp(MIN_TICK_INTERVAL, MAX_TICK_INTERVAL);
            }
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

    /// Read back so a test can assert that production — not the test — drove the clock.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn interval(&self) -> f32 {
        self.interval
    }

    pub fn last_tick(&self) -> u64 {
        self.last_tick
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
    fn a_burst_of_ticks_in_one_frame_keeps_the_measured_cadence() {
        let mut clock = TickClock::default();
        clock.advance(0.1);
        clock.observe_tick(1);
        assert_eq!(clock.interval(), 0.1, "the first tick measures the cadence");

        // Both deltas drain in the same frame, so the second measures no elapsed time at all.
        clock.observe_tick(2);
        assert_eq!(
            clock.interval(),
            0.1,
            "a same-frame burst must not collapse the interval to its floor"
        );

        clock.advance(0.05);
        assert_eq!(clock.factor(), 0.5, "half a measured interval is half way");
    }

    #[test]
    fn the_low_end_of_the_interval_clamp_holds() {
        let mut clock = TickClock::default();
        clock.advance(0.001);
        clock.observe_tick(1);
        assert_eq!(
            clock.interval(),
            MAX_TICK_INTERVAL,
            "a sub-floor measurement is a burst, not a cadence, and is discarded"
        );

        let mut fast = TickClock::default();
        fast.advance(MIN_TICK_INTERVAL);
        fast.observe_tick(1);
        assert_eq!(
            fast.interval(),
            MIN_TICK_INTERVAL,
            "a real fast-forward cadence at the floor is still measured"
        );
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
