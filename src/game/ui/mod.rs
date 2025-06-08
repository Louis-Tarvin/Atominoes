use bevy::prelude::*;

use crate::{screens::Screen, theme::widget::ui_root};

use super::level::{CurrentLevel, Level};

mod sidebar;
mod tray;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (init_level_ui, tray::update_drag_icons)
            .chain()
            .before(bevy_easings::EasingsLabel)
            .run_if(resource_changed::<super::level::CurrentLevel>.and(in_state(Screen::Gameplay))),
    );
}

#[derive(Component)]
struct UiRoot;

fn init_level_ui(
    prev_root: Query<Entity, With<UiRoot>>,
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    level_assets: Res<Assets<Level>>,
) {
    // despawn previous level UI if present
    for entity in prev_root {
        commands.entity(entity).despawn();
    }
    // re-draw UI for current level
    commands.spawn((
        ui_root("UI root"),
        StateScoped(Screen::Gameplay),
        UiRoot,
        children![
            sidebar::sidebar(&current_level, &level_assets),
            tray::tray()
        ],
    ));
}
