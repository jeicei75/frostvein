use bevy::prelude::Resource;

use crate::ingest::MirrorResource;

pub const TICKS_PER_DAY: u64 = 24_000;
pub const TICKS_PER_HOUR: f32 = 1_000.0;
pub const BOOT_HOUR: f32 = 22.0;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq)]
pub struct ClockPin(pub Option<f32>);

#[derive(Resource)]
pub struct ExplicitClock;

pub fn hour_at(tick: u64) -> f32 {
    (BOOT_HOUR + (tick % TICKS_PER_DAY) as f32 / TICKS_PER_HOUR) % 24.0
}

pub fn current_hour(mirror: &MirrorResource, pin: &ClockPin) -> f32 {
    pin.0.unwrap_or_else(|| hour_at(mirror.0.tick()))
}

#[cfg(test)]
mod tests {
    use super::{BOOT_HOUR, TICKS_PER_DAY, hour_at};

    #[test]
    fn tick_clock_moves_fractionally_and_wraps() {
        assert_eq!(hour_at(0), BOOT_HOUR);
        assert_eq!(hour_at(500), BOOT_HOUR + 0.5);
        assert_eq!(hour_at(1_000), BOOT_HOUR + 1.0);
        assert_eq!(hour_at(2_000), 0.0);
        assert_eq!(hour_at(TICKS_PER_DAY), BOOT_HOUR);
    }
}
