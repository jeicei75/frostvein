use bevy::{
    ecs::change_detection::DetectChanges,
    input::{ButtonInput, mouse::MouseButton},
    prelude::{
        Color, Commands, GlobalZIndex, KeyCode, Node, PositionType, Res, ResMut, Resource, Text,
        TextColor, TextFont, px,
    },
};
use protocol::{Command, DesignationKind, Rect};

use crate::{command::PendingCommands, pick::PickedTile, project::ClientLocal};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesignateMode {
    #[default]
    None,
    Dig,
    Channel,
    Stockpile,
    Clear,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DragAnchor(pub Option<[i32; 3]>);

/// The mode the in-flight drag was started in. A drag commits in the mode it began in, so a mode
/// key pressed mid-drag takes effect on the NEXT drag rather than silently changing what the
/// release will issue.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DragMode(pub Option<DesignateMode>);

#[derive(bevy::prelude::Component)]
pub struct DesignateHint;

pub fn designation_hint(mode: DesignateMode, dragging: bool) -> &'static str {
    match (mode, dragging) {
        (DesignateMode::None, _) => "1 dig  2 channel  3 stockpile  4 clear",
        (DesignateMode::Dig, false) => "dig: drag to designate  Esc leave",
        (DesignateMode::Dig, true) => "dig: release to designate  Esc abort",
        (DesignateMode::Channel, false) => "channel: drag to designate  Esc leave",
        (DesignateMode::Channel, true) => "channel: release to designate  Esc abort",
        (DesignateMode::Stockpile, false) => "stockpile: drag to place  Esc leave",
        (DesignateMode::Stockpile, true) => "stockpile: release to place  Esc abort",
        (DesignateMode::Clear, false) => "clear: drag to remove  Esc leave",
        (DesignateMode::Clear, true) => "clear: release to remove  Esc abort",
    }
}

pub fn setup_designate_hint(mut commands: Commands) {
    commands.spawn((
        Text::new(designation_hint(DesignateMode::None, false)),
        TextFont::from_font_size(22.0),
        TextColor(Color::srgb(0.86, 0.91, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(16),
            left: px(16),
            ..Default::default()
        },
        GlobalZIndex(i32::MAX - 16),
        DesignateHint,
        ClientLocal,
    ));
}

pub fn update_designate_hint(
    mode: Res<DesignateMode>,
    anchor: Res<DragAnchor>,
    drag_mode: Res<DragMode>,
    mut hints: bevy::prelude::Query<&mut Text, bevy::prelude::With<DesignateHint>>,
) {
    if !mode.is_changed() && !anchor.is_changed() && !drag_mode.is_changed() {
        return;
    }
    // While a drag is live the bar names the mode that will actually commit, not the mode key
    // last pressed — those differ the moment a mode key is pressed mid-drag.
    let text = designation_hint(drag_mode.0.unwrap_or(*mode), anchor.0.is_some());
    for mut hint in &mut hints {
        *hint = Text::new(text);
    }
}

/// Handles mode keys and the real press-drag-release interaction after the current frame's pick.
pub fn designation_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    picked: Res<PickedTile>,
    mut mode: ResMut<DesignateMode>,
    mut anchor: ResMut<DragAnchor>,
    mut drag_mode: ResMut<DragMode>,
    mut pending: ResMut<PendingCommands>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        *mode = DesignateMode::Dig;
    } else if keys.just_pressed(KeyCode::Digit2) {
        *mode = DesignateMode::Channel;
    } else if keys.just_pressed(KeyCode::Digit3) {
        *mode = DesignateMode::Stockpile;
    } else if keys.just_pressed(KeyCode::Digit4) {
        *mode = DesignateMode::Clear;
    }

    let abort = keys.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right);
    if abort {
        if anchor.0.is_some() {
            anchor.0 = None;
            drag_mode.0 = None;
        } else if keys.just_pressed(KeyCode::Escape) {
            *mode = DesignateMode::None;
        }
        return;
    }

    if mouse.just_pressed(MouseButton::Left) && *mode != DesignateMode::None {
        anchor.0 = picked.tile();
        drag_mode.0 = anchor.0.map(|_| *mode);
    }

    if mouse.just_released(MouseButton::Left) {
        let Some(anchor_tile) = anchor.0 else {
            return;
        };
        if let Some(release_tile) = picked.tile() {
            // NOTE: drags up a cliff designate on the anchor's level; the shared rect contract
            // is single-z and deliberately discards the release tile's z.
            let rect = client_core::rect_on_level(
                (anchor_tile[0], anchor_tile[1]),
                (release_tile[0], release_tile[1]),
                anchor_tile[2],
            );
            for command in commands_for(drag_mode.0.unwrap_or(*mode), rect) {
                pending.push(command);
            }
        }
        // A missed release must never leave a stale anchor for the next drag.
        anchor.0 = None;
        drag_mode.0 = None;
    }
}

fn commands_for(mode: DesignateMode, rect: Rect) -> Vec<Command> {
    match mode {
        DesignateMode::None => Vec::new(),
        DesignateMode::Dig => vec![Command::Designate {
            kind: DesignationKind::Dig,
            rect,
        }],
        DesignateMode::Channel => vec![Command::Designate {
            kind: DesignationKind::Channel,
            rect,
        }],
        DesignateMode::Stockpile => vec![Command::PlaceStockpile { rect }],
        DesignateMode::Clear => vec![
            Command::CancelDesignation { rect },
            Command::RemoveStockpile { rect },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::{DesignateMode, commands_for, designation_hint};
    use protocol::{Command, DesignationKind, Rect};

    #[test]
    fn clear_issues_both_existing_commands_in_tui_order() {
        let rect = Rect {
            min: [1, 2, 3],
            max: [4, 5, 3],
        };
        assert_eq!(
            commands_for(DesignateMode::Clear, rect),
            vec![
                Command::CancelDesignation { rect },
                Command::RemoveStockpile { rect }
            ]
        );
    }

    #[test]
    fn channel_and_stockpile_map_to_their_own_distinct_commands() {
        let rect = Rect {
            min: [1, 2, 3],
            max: [4, 5, 3],
        };
        // Channel is NOT dig, and a stockpile is not a designation at all. Both arms could be
        // rewritten to emit dig, or nothing, with the whole suite green before this.
        assert_eq!(
            commands_for(DesignateMode::Channel, rect),
            vec![Command::Designate {
                kind: DesignationKind::Channel,
                rect
            }]
        );
        assert_eq!(
            commands_for(DesignateMode::Stockpile, rect),
            vec![Command::PlaceStockpile { rect }]
        );
        assert_ne!(
            commands_for(DesignateMode::Channel, rect),
            commands_for(DesignateMode::Dig, rect),
            "channel and dig must not collapse to the same wire command"
        );
    }

    #[test]
    fn every_hint_is_ascii() {
        for mode in [
            DesignateMode::None,
            DesignateMode::Dig,
            DesignateMode::Channel,
            DesignateMode::Stockpile,
            DesignateMode::Clear,
        ] {
            for dragging in [false, true] {
                assert!(designation_hint(mode, dragging).is_ascii());
            }
        }
    }

    #[test]
    fn dig_mapping_uses_the_existing_designate_shape() {
        let rect = Rect {
            min: [7, 8, 9],
            max: [10, 11, 9],
        };
        assert_eq!(
            commands_for(DesignateMode::Dig, rect),
            vec![Command::Designate {
                kind: DesignationKind::Dig,
                rect
            }]
        );
    }

    // NOTE: the abort paths used to be pinned HERE, by `run_system_once(designation_input)`
    // with `DragAnchor` and `PickedTile` inserted by hand. That is the shape D6 forbids and the
    // shape that hid 8.1's `--cursor` defect through a whole mutation round: it starts downstream
    // of the production drive line, so it cannot see a drag that never anchors. AC14 asks for the
    // abort paths driven through the shared registration point, and they now are — see
    // `tests/headless.rs`, `right_button_*`, `escape_during_*` and `escape_with_no_drag_*`.

    #[test]
    fn designation_input_uses_the_shared_rect_helper_not_local_normalization() {
        let production = include_str!("designate.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the production module precedes its tests");
        assert!(
            production.contains(&["client_core::rect", "_on_level("].concat()),
            "AC3 requires the shared rect helper at the wire boundary"
        );
        assert!(
            !production.contains("anchor_tile[0].min("),
            "AC3 forbids a second local corner normalization in gui"
        );
    }
}
