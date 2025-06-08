//! The main menu (seen on the title screen).

use std::time::Duration;

use bevy::prelude::*;
use bevy_easings::Ease;

use crate::{asset_tracking::LoadResource, menus::Menu, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MainMenuAssets>();
    app.load_resource::<MainMenuAssets>();
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct MainMenuAssets {
    #[dependency]
    pub title: Handle<Image>,
}
impl FromWorld for MainMenuAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            title: assets.load("images/title.png"),
        }
    }
}

fn spawn_main_menu(mut commands: Commands, assets: Res<MainMenuAssets>) {
    let title_bundle = (
        Name::new("Title"),
        Node::default(),
        Node {
            height: Val::Px(0.0),
            ..Default::default()
        }
        .ease_to(
            Node {
                height: Val::Px(134.0),
                ..Default::default()
            },
            bevy_easings::EaseFunction::CubicOut,
            bevy_easings::EasingType::Once {
                duration: Duration::from_millis(1000),
            },
        ),
        ImageNode {
            image: assets.title.clone(),
            ..Default::default()
        },
    );
    commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Main),
        #[cfg(not(target_family = "wasm"))]
        children![
            title_bundle,
            widget::button("Play", open_level_select_menu),
            widget::button("Settings", open_settings_menu),
            widget::button("Credits", open_credits_menu),
            widget::button("Exit", exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            title_bundle,
            widget::button("Play", open_level_select_menu),
            widget::button("Settings", open_settings_menu),
            widget::button("Credits", open_credits_menu),
        ],
    ));
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

fn open_level_select_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::LevelSelect);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
