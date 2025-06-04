use bevy::prelude::*;

use crate::{
    game::level::{CurrentLevel, Level},
    menus::Menu,
    screens::Screen,
    theme::{palette::*, widget},
};

#[derive(Component)]
struct UiSidebar;

#[derive(Component)]
pub(super) struct UiSidebarText;

pub(super) fn sidebar() -> impl Bundle {
    (
        Name::new("Sidebar"),
        UiSidebar,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(320.0),
            height: Val::Percent(100.0),
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
                Text::new("Level Info"),
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
            widget::button("Settings", open_settings_menu),
            widget::button("Quit to title", quit_to_title),
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
            text.0 = "No level loaded".to_string();
        }
    }
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
