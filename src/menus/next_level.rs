use bevy::{prelude::*, render::render_resource::encase::private::Length};

use crate::{
    audio::{AudioAssets, sound_effect},
    game::{
        level::{CurrentLevel, LevelAssets, PlacedLevelAtoms},
        state::GameState,
    },
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::LevelComplete), spawn_next_level_menu);
}

fn spawn_next_level_menu(mut commands: Commands, audio_assets: Res<AudioAssets>) {
    commands.spawn((
        widget::bouncy_ui_root("Next Level Menu"),
        GlobalZIndex(2),
        StateScoped(GameState::LevelComplete),
        children![
            widget::header("Level complete!"),
            widget::button("Continue", goto_next_level),
        ],
    ));
    commands.spawn((
        Name::new("Menu Overlay"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        GlobalZIndex(1),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        StateScoped(GameState::LevelComplete),
    ));
    commands.spawn(sound_effect(audio_assets.level_complete_sfx.clone()));
}

fn goto_next_level(
    _: Trigger<Pointer<Click>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
    mut placed_atoms: ResMut<PlacedLevelAtoms>,
    level_handles: Res<LevelAssets>,
) {
    placed_atoms.clear();
    let new_index = current_level.get_index().unwrap() + 1;
    if new_index >= level_handles.levels.length() {
        error!("No more levels");
    } else {
        current_level.set_level(level_handles.levels[new_index].clone(), new_index);
        next_state.set(GameState::Placement);
    }
}
