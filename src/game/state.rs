use bevy::prelude::*;

use crate::{AppSystems, screens::Screen};

#[derive(SubStates, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(Screen = Screen::Gameplay)]
#[states(scoped_entities)]
pub enum GameState {
    #[default]
    Placement,
    Running,
    LevelComplete,
    RestartLevel,
}

pub(super) fn plugin(app: &mut App) {
    app.add_sub_state::<GameState>();
    app.add_systems(
        Update,
        toggle_game_state_system
            .in_set(AppSystems::RecordInput)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(OnEnter(GameState::RestartLevel), handle_restart);
}

fn handle_restart(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Placement);
}

fn toggle_game_state_system(
    input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Space) {
        let new_state = match current_state.get() {
            GameState::Placement => GameState::Running,
            GameState::Running => GameState::Placement,
            GameState::LevelComplete => GameState::LevelComplete,
            GameState::RestartLevel => GameState::RestartLevel,
        };
        next_state.set(new_state);
    }
}
