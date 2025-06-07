//! The title screen that appears after the splash screen.

use bevy::prelude::*;

use crate::{audio::music, menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Title),
        (open_main_menu, start_background_music),
    );
    app.add_systems(OnExit(Screen::Title), close_menu);
}

fn open_main_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

#[derive(Component)]
struct BgmMusic;

fn start_background_music(
    mut commands: Commands,
    assets: Res<crate::audio::AudioAssets>,
    query: Query<&BgmMusic>,
) {
    if query.is_empty() {
        commands.spawn((
            Name::new("Background music"),
            music(assets.bgm.clone()),
            BgmMusic,
        ));
    }
}
