use bevy::prelude::*;

use crate::{screens::Screen, theme::widget::ui_root};

mod sidebar;
mod tray;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), init_level_ui);
    app.add_systems(
        Update,
        (
            sidebar::update_sidebar_text,
            sidebar::update_sidebar_header,
            tray::update_drag_icons,
        )
            .run_if(resource_changed::<super::level::CurrentLevel>.and(in_state(Screen::Gameplay))),
    );
}

fn init_level_ui(mut commands: Commands) {
    commands.spawn((
        ui_root("UI root"),
        StateScoped(Screen::Gameplay),
        children![sidebar::sidebar(), tray::tray()],
    ));
}
