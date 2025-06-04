use bevy::{prelude::*, render::render_resource::encase::private::Length};

use crate::{
    game::{
        level::{CurrentLevel, LevelAssets},
        state::GameState,
    },
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::LevelComplete), spawn_next_level_menu);
}

fn spawn_next_level_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Next Level Menu"),
        GlobalZIndex(2),
        StateScoped(GameState::LevelComplete),
        children![
            widget::header("Level complete!"),
            widget::button("Continue", goto_next_level),
        ],
    ));
}

fn goto_next_level(
    _: Trigger<Pointer<Click>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
    level_handles: Res<LevelAssets>,
) {
    let new_index = current_level.get_index().unwrap() + 1;
    if new_index >= level_handles.levels.length() {
        error!("No more levels");
    } else {
        current_level.set_level(level_handles.levels[new_index].clone(), new_index);
        next_state.set(GameState::Placement);
    }
}
