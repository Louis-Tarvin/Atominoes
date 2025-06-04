use atom::AtomAssets;
use bevy::prelude::*;
use level::{CurrentLevel, LevelAssets};

use crate::{asset_tracking::LoadResource, screens::Screen};

mod animation;
mod atom;
mod collision;
pub mod level;
mod movement;
mod placement;
pub mod state;
mod ui;
mod win_condition;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        movement::plugin,
        state::plugin,
        level::plugin,
        collision::plugin,
        win_condition::plugin,
        placement::plugin,
        ui::plugin,
    ));

    app.register_type::<AtomAssets>();
    app.load_resource::<AtomAssets>();
    app.add_systems(OnEnter(Screen::Gameplay), init_level);
}

fn init_level(mut current_level: ResMut<CurrentLevel>, level_handles: Res<LevelAssets>) {
    current_level.set_level(level_handles.levels[0].clone(), 0);
}
