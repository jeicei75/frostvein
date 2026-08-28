use bevy::{
    ecs::change_detection::DetectChanges,
    input::{ButtonInput, mouse::MouseButton},
    prelude::{
        Color, Commands, GlobalZIndex, KeyCode, Node, PositionType, Res, ResMut, Resource, Text,
        TextColor, TextFont, px,
    },
};
use protocol::{Command, DesignationKind, Rect};

use client_core::Mirror;

use crate::{
    command::PendingCommands,
    ingest::MirrorResource,
    pick::{PickedCell, PickedTile},
    project::ClientLocal,
    slice::SliceLevel,
    transform::render_to_world,
};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesignateMode {
    #[default]
    None,
    Dig,
    Channel,
    Stockpile,
    Clear,
}

/// The cell a drag was anchored at, WITH the face its ray entered.
///
/// The face is load-bearing, not diagnostic: channel and stockpile designate the neighbour across
/// it (see `designation_target`), and clear has to reach both that neighbour and the picked cell
/// to remove what is actually under the cursor.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DragAnchor(pub Option<PickedCell>);

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
#[allow(clippy::too_many_arguments)]
pub fn designation_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    picked: Res<PickedTile>,
    mirror: Res<MirrorResource>,
    slice: Res<SliceLevel>,
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
        anchor.0 = picked.0;
        drag_mode.0 = anchor.0.map(|_| *mode);
    }

    if mouse.just_released(MouseButton::Left) {
        let Some(anchor_cell) = anchor.0 else {
            return;
        };
        if let Some(release_cell) = picked.0 {
            let mode = drag_mode.0.unwrap_or(*mode);
            let mirror = &mirror.0;
            // Dig keeps AC4's single-z rect at the anchor's level: cutting one level into a slope
            // is what dig is for, and the shared helper deliberately discards the release z.
            let picked_rect = client_core::rect_on_level(
                (anchor_cell.tile[0], anchor_cell.tile[1]),
                (release_cell.tile[0], release_cell.tile[1]),
                anchor_cell.tile[2],
            );
            let surface = client_core::rects_for_cells(&client_core::surface_targets(
                mirror,
                slice.level(),
                designation_target(mirror, anchor_cell, mode),
                designation_target(mirror, release_cell, mode),
            ));
            for command in commands_for(mode, picked_rect, &surface) {
                pending.push(command);
            }
        }
        // A missed release must never leave a stale anchor for the next drag.
        anchor.0 = None;
        drag_mode.0 = None;
    }
}

/// Which cell a mode actually designates, given the cell the ray hit and the face it entered.
///
/// **Dig wants the cell the ray hit.** `sim-core` filters dig on `Tile::Solid`, and picking only
/// ever resolves a solid or ramp cell (`is_visible_at_slice`), so the picked cell is already the
/// right one.
///
/// **Channel and stockpile want a STANDABLE cell** — `Tile::Empty` with support beneath — and the
/// picked cell can never be one, because picking cannot return air. Sending the picked cell is
/// what made both modes completely inert: the daemon accepted the command and kept nothing, with
/// no error, no ack and no log, through a whole code review.
///
/// RULED 2026-08-27 (Wolf): the target is the neighbour across the face the ray ENTERED. A top
/// face channels the air directly above, which is the common case and reads as "turn this block
/// into a ramp"; a cliff face targets the cell you are looking into, which is standable exactly
/// when it borders a ledge. The face was already computed for AC13's highlight, so this gives it
/// a second consumer and makes it behavioural rather than decorative.
pub fn designation_target(mirror: &Mirror, cell: PickedCell, mode: DesignateMode) -> [i32; 3] {
    match mode {
        // Clear is here because it must REACH the standable cell to remove a channel or a
        // stockpile. Its other half — the dig at the cell the ray hit — is covered by the second
        // rect `commands_for` receives, so clear is the one mode that needs both.
        DesignateMode::Channel | DesignateMode::Stockpile | DesignateMode::Clear => {
            // `render_to_world` is the single axis conversion, per AC2 — the face normal is a
            // render-space unit vector and must not be re-derived by hand here.
            let [dx, dy, dz] = render_to_world(cell.face.normal());
            let neighbour = [cell.tile[0] + dx, cell.tile[1] + dy, cell.tile[2] + dz];
            if client_core::is_standable(mirror, neighbour) {
                return neighbour;
            }
            // MEASURED 2026-08-27 on the real world: the face neighbour is standable for 100% of
            // TOP-face hits and only 8.5-11.8% of side-face hits, because on flat ground the cell
            // beside a block is another block. Pointing at the front edge of a surface block
            // rather than its top therefore designated nothing — Wolf's "dragging might skip 2
            // first blocks". RULED 2026-08-27 (Wolf): fall back to the cell directly above the
            // block, which is standable for 100% of surface blocks, while keeping the face
            // neighbour where it IS standable so pointing at a wall still targets the ledge it
            // borders.
            let above = [cell.tile[0], cell.tile[1], cell.tile[2] + 1];
            if client_core::is_standable(mirror, above) {
                return above;
            }
            // Neither is standable. Return the face neighbour so the caller sees the mode's own
            // answer; the preview filter and the sim both drop it, visibly and consistently.
            neighbour
        }
        DesignateMode::Dig | DesignateMode::None => cell.tile,
    }
}

/// `picked_rect` is AC4's single-z rect at the cells the ray hit — dig's whole answer, and the
/// half of clear that removes digs. `surface` is the followed ground, one rect per merged run —
/// what channel and stockpile designate, and the half of clear that removes them.
fn commands_for(mode: DesignateMode, picked_rect: Rect, surface: &[Rect]) -> Vec<Command> {
    match mode {
        DesignateMode::None => Vec::new(),
        DesignateMode::Dig => vec![Command::Designate {
            kind: DesignationKind::Dig,
            rect: picked_rect,
        }],
        DesignateMode::Channel => surface
            .iter()
            .map(|rect| Command::Designate {
                kind: DesignationKind::Channel,
                rect: *rect,
            })
            .collect(),
        DesignateMode::Stockpile => surface
            .iter()
            .map(|rect| Command::PlaceStockpile { rect: *rect })
            .collect(),
        // Clear means "remove what is under the cursor", and after the targeting fix that is two
        // different cells: a dig sits at the cell the ray hit, while a channel or a stockpile
        // sits one cell across the entered face. Clearing only one of them leaves the other
        // standing with no way for the boss to remove it at all.
        //
        // NOTE: three commands per clear rather than two, which brings the 256-command bound
        // fractionally closer. That bound's split-pair hazard is already an open deferred item
        // and is not made materially worse by one more command.
        DesignateMode::Clear => std::iter::once(Command::CancelDesignation { rect: picked_rect })
            .chain(surface.iter().flat_map(|rect| {
                [
                    Command::CancelDesignation { rect: *rect },
                    Command::RemoveStockpile { rect: *rect },
                ]
            }))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use protocol::{Command, DesignationKind, Rect};

    use super::{DesignateMode, commands_for, designation_hint};

    /// Dig and clear read the picked rect; channel and stockpile read the followed surface. The
    /// helper passes the same rect as both so a single-rect expectation stays readable, and the
    /// call sites that care about the distinction spell it out.
    fn commands_at(mode: DesignateMode, rect: Rect) -> Vec<Command> {
        commands_for(mode, rect, &[rect])
    }

    #[test]
    fn clear_reaches_both_the_picked_cell_and_the_standable_one() {
        // The dig lives at the cell the ray hit; a channel or a stockpile lives one cell across
        // the entered face. Clear has to reach BOTH, or the boss can designate something he can
        // never remove. Collapsing these to one rect is the defect this test exists to catch.
        let picked_rect = Rect {
            min: [1, 2, 3],
            max: [4, 5, 3],
        };
        let standable_rect = Rect {
            min: [1, 2, 4],
            max: [4, 5, 4],
        };
        assert_eq!(
            commands_for(DesignateMode::Clear, picked_rect, &[standable_rect]),
            vec![
                Command::CancelDesignation { rect: picked_rect },
                Command::CancelDesignation {
                    rect: standable_rect
                },
                Command::RemoveStockpile {
                    rect: standable_rect
                }
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
            commands_at(DesignateMode::Channel, rect),
            vec![Command::Designate {
                kind: DesignationKind::Channel,
                rect
            }]
        );
        assert_eq!(
            commands_at(DesignateMode::Stockpile, rect),
            vec![Command::PlaceStockpile { rect }]
        );
        assert_ne!(
            commands_at(DesignateMode::Channel, rect),
            commands_at(DesignateMode::Dig, rect),
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
            commands_at(DesignateMode::Dig, rect),
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
