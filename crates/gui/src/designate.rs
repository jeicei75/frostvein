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
    mut hints: bevy::prelude::Query<&mut Text, bevy::prelude::With<DesignateHint>>,
) {
    if !mode.is_changed() && !anchor.is_changed() {
        return;
    }
    let text = designation_hint(*mode, anchor.0.is_some());
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
        } else if keys.just_pressed(KeyCode::Escape) {
            *mode = DesignateMode::None;
        }
        return;
    }

    if mouse.just_pressed(MouseButton::Left) && *mode != DesignateMode::None {
        anchor.0 = picked.tile();
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
            for command in commands_for(*mode, rect) {
                pending.push(command);
            }
        }
        // A missed release must never leave a stale anchor for the next drag.
        anchor.0 = None;
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
}
