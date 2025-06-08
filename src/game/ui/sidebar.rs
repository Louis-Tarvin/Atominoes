use std::time::Duration;

use bevy::prelude::*;
use bevy_easings::Ease;

use crate::{
    game::{
        level::{CurrentLevel, Level, PlacedLevelAtoms},
        state::GameState,
    },
    screens::Screen,
    theme::{palette::*, widget},
};

pub(super) fn sidebar(current_level: &CurrentLevel, level_assets: &Assets<Level>) -> impl Bundle {
    let text = if let Ok(level) = current_level.get_level(level_assets) {
        format!(
            "{}\n\nControls:\n<esc>: pause\n<spacebar>: start/stop\n Left click and drag an atom from the tray to place it.\nRight click to remove a placed atom.",
            level.sidebar_text
        )
    } else {
        "Sandbox".to_string()
    };
    let heading = if let CurrentLevel::Loaded {
        level_handle: _,
        level_index,
    } = *current_level
    {
        format!("Level {}", level_index + 1)
    } else {
        "Sandbox".to_string()
    };
    (
        Name::new("Sidebar"),
        Node::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(-320.0),
            width: Val::Percent(30.0),
            max_width: Val::Px(320.0),
            height: Val::Percent(100.0),
            overflow: Overflow {
                x: OverflowAxis::Hidden,
                ..Default::default()
            },
            padding: UiRect::all(Val::Px(24.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            border: UiRect::right(Val::Px(2.0)),
            ..Default::default()
        }
        .ease_to(
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(30.0),
                max_width: Val::Px(320.0),
                height: Val::Percent(100.0),
                overflow: Overflow {
                    x: OverflowAxis::Hidden,
                    ..Default::default()
                },
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                border: UiRect::right(Val::Px(2.0)),
                ..Default::default()
            },
            bevy_easings::EaseFunction::BounceOut,
            bevy_easings::EasingType::Once {
                duration: Duration::from_millis(1500),
            },
        ),
        BorderColor(ACCENT),
        BackgroundColor(BACKGROUND),
        children![
            (
                Name::new("Sidebar Header"),
                Text::new(heading),
                TextFont {
                    font_size: 28.0,
                    ..Default::default()
                },
                TextColor(OFF_WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..Default::default()
                },
            ),
            (
                Name::new("Sidebar Text"),
                Text::new(text),
                TextFont {
                    font_size: 14.0,
                    ..Default::default()
                },
                TextColor(LABEL_TEXT),
                Node {
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
            ),
            widget::sidebar_button("Start / Stop experiment", start_stop),
            widget::sidebar_button("Reset level", reset),
            widget::sidebar_button("Quit to title", quit_to_title),
        ],
    )
}

fn start_stop(
    _: Trigger<Pointer<Click>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if matches!(**current_state, GameState::Placement) {
        next_state.set(GameState::Running);
    } else if matches!(**current_state, GameState::Running) {
        next_state.set(GameState::Placement);
    }
}

fn reset(
    _: Trigger<Pointer<Click>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut placed_atoms: ResMut<PlacedLevelAtoms>,
) {
    placed_atoms.clear();
    next_state.set(GameState::RestartLevel);
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
