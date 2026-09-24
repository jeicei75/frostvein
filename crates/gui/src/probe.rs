//! THROWAWAY PROBE for story 11.3 Task 4 (Wolf's live day pick). Never merged.
//! `T` cycles the day table A -> B -> C -> D; `+` / `-` step the daemon's speed like the tui.
//! State is printed to stderr only: an on-screen label would add a text node the wiring test pins.

use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::prelude::*;
use protocol::{Command, Speed};

use crate::{
    appearance::LightTable,
    command::{PendingCommands, StaticWorld},
    ingest::MirrorResource,
};

static DAY_CHOICE: AtomicUsize = AtomicUsize::new(0);

const NAMES: [&str; 4] = [
    "A provisional",
    "B soft warm",
    "C crisp blue",
    "D pale overcast",
];

pub fn day_table(night: LightTable) -> LightTable {
    let (sky, ambient, ambient_brightness, directional, directional_illuminance) =
        match DAY_CHOICE.load(Ordering::Relaxed) {
            0 => (
                (110, 155, 205),
                (190, 210, 235),
                4_000.0,
                (255, 244, 228),
                12_000.0,
            ),
            1 => (
                (130, 170, 215),
                (180, 200, 230),
                3_000.0,
                (255, 236, 210),
                11_000.0,
            ),
            2 => (
                (95, 145, 210),
                (170, 195, 235),
                3_500.0,
                (255, 250, 240),
                14_000.0,
            ),
            _ => (
                (175, 190, 210),
                (200, 210, 225),
                5_000.0,
                (235, 238, 245),
                8_000.0,
            ),
        };
    let rgb = |(r, g, b): (u8, u8, u8)| Color::srgb_u8(r, g, b);
    LightTable {
        sky: rgb(sky),
        star: night.star,
        ambient: rgb(ambient),
        ambient_brightness,
        aurora: night.aurora,
        directional: rgb(directional),
        directional_illuminance,
    }
}

pub fn probe_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mirror: Res<MirrorResource>,
    pin: Res<crate::clock::ClockPin>,
    static_world: Res<StaticWorld>,
    pending: Option<ResMut<PendingCommands>>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        let next = (DAY_CHOICE.load(Ordering::Relaxed) + 1) % NAMES.len();
        DAY_CHOICE.store(next, Ordering::Relaxed);
        eprintln!("PROBE day table -> {}", NAMES[next]);
    }
    let speed = mirror.0.speed();
    let plus = keys.any_just_pressed([KeyCode::Equal, KeyCode::NumpadAdd]);
    let minus = keys.any_just_pressed([KeyCode::Minus, KeyCode::NumpadSubtract]);
    let wanted = match (plus, minus, speed) {
        (true, false, Speed::Paused) => Some(Speed::Normal),
        (true, false, Speed::Normal) => Some(Speed::Fast),
        (false, true, Speed::Fast) => Some(Speed::Normal),
        (false, true, Speed::Normal) => Some(Speed::Paused),
        _ => None,
    };
    if let (Some(wanted), Some(mut pending)) = (wanted, pending) {
        if static_world.0 {
            eprintln!("PROBE speed refused: --static-world");
        } else {
            pending.push(Command::SetSpeed {
                speed: wanted,
                at_tick: None,
            });
            eprintln!("PROBE speed -> {wanted:?}");
        }
    }
    if keys.just_pressed(KeyCode::KeyT) || wanted.is_some() {
        let hour = crate::clock::current_hour(&mirror, &pin);
        eprintln!(
            "PROBE day {} | speed {:?} | {:02}:{:02}",
            NAMES[DAY_CHOICE.load(Ordering::Relaxed)],
            wanted.unwrap_or(speed),
            hour as u32,
            (hour.fract() * 60.0) as u32
        );
    }
}
