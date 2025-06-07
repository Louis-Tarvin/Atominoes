use bevy::{ecs::spawn::SpawnIter, prelude::*};

use crate::{
    LEVELS, asset_tracking::ResourceHandles, game::MenuSelection, menus::Menu, screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::LevelSelect), spawn_level_select_menu);
}

fn spawn_level_select_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Level Select Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::LevelSelect),
        Children::spawn((
            Spawn(widget::header("Choose Level:")),
            Spawn((
                Name::new("Levels wrapper"),
                Node {
                    display: Display::Flex,
                    flex_wrap: FlexWrap::Wrap,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(10.0),
                    column_gap: Val::Px(5.0),
                    ..Default::default()
                },
                Children::spawn((SpawnIter(LEVELS.iter().enumerate().map(|(i, _)| {
                    (widget::button(
                        (i + 1).to_string(),
                        move |_: Trigger<Pointer<Click>>,
                              resource_handles: Res<ResourceHandles>,
                              mut next_screen: ResMut<NextState<Screen>>,
                              mut menu_selection: ResMut<MenuSelection>| {
                            *menu_selection = MenuSelection::Level(i);
                            if resource_handles.is_all_done() {
                                next_screen.set(Screen::Gameplay);
                            } else {
                                next_screen.set(Screen::Loading);
                            }
                        },
                    ),)
                })),)),
            )),
            Spawn(widget::button("Sandbox", start_with_level_editor)),
            Spawn(widget::button("Back", go_back)),
        )),
    ));
}

fn start_with_level_editor(
    _: Trigger<Pointer<Click>>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut menu_selection: ResMut<MenuSelection>,
) {
    *menu_selection = MenuSelection::Editor;
    if resource_handles.is_all_done() {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn go_back(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}
