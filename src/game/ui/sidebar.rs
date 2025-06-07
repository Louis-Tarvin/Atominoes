use bevy::prelude::*;

use crate::{
    game::{
        level::{CurrentLevel, Level, PlacedLevelAtoms},
        state::GameState,
    },
    screens::Screen,
    theme::{palette::*, widget},
};

#[derive(Component)]
struct UiSidebar;

#[derive(Component)]
pub(super) struct UiSidebarText;

#[derive(Component)]
pub(super) struct UiSidebarHeader;

pub(super) fn sidebar() -> impl Bundle {
    (
        Name::new("Sidebar"),
        UiSidebar,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
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
        BorderColor(ACCENT.into()),
        BackgroundColor(BACKGROUND.into()),
        children![
            (
                Name::new("Sidebar Header"),
                Text::new("Level"),
                UiSidebarHeader,
                TextFont {
                    font_size: 28.0,
                    ..Default::default()
                },
                TextColor(OFF_WHITE.into()),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..Default::default()
                },
            ),
            (
                Name::new("Sidebar Text"),
                UiSidebarText,
                Text::new("Loading..."),
                TextFont {
                    font_size: 14.0,
                    ..Default::default()
                },
                TextColor(LABEL_TEXT.into()),
                Node {
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
            ),
            widget::sidebar_button("Start / Stop simulation", start_stop),
            widget::sidebar_button("Reset level", reset),
            widget::sidebar_button("Quit to title", quit_to_title),
        ],
    )
}

pub(super) fn update_sidebar_text(
    current_level: Res<CurrentLevel>,
    level_assets: Res<Assets<Level>>,
    mut text_query: Query<&mut Text, With<UiSidebarText>>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        if let Ok(level) = current_level.get_level(&level_assets) {
            text.0 = format!("{}", level.sidebar_text);
        } else {
            text.0 = "Sandbox".to_string();
        }
    }
}

pub(super) fn update_sidebar_header(
    current_level: Res<CurrentLevel>,
    mut text_query: Query<&mut Text, With<UiSidebarHeader>>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        if let CurrentLevel::Loaded {
            level_handle: _,
            level_index,
        } = *current_level
        {
            text.0 = format!("Level {}", level_index + 1);
        } else {
            text.0 = "Sandbox".to_string();
        }
    }
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
