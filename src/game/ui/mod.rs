use bevy::prelude::*;

use crate::screens::Screen;

mod sidebar;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), init_level_ui);
    app.add_systems(
        Update,
        sidebar::update_sidebar_text.run_if(resource_changed::<super::level::CurrentLevel>),
    );
}

fn init_level_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("UI Root"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        StateScoped(Screen::Gameplay),
        children![sidebar::sidebar()],
    ));
}
