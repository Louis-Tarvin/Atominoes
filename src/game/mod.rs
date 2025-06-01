use atom::{AtomAssets, AtomType};
use bevy::prelude::*;
use level::{CurrentLevel, Level, LevelAtom};
use movement::CardinalDirection;

use crate::{asset_tracking::LoadResource, screens::Screen};

mod animation;
mod atom;
mod collision;
mod level;
mod movement;
pub mod state;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        movement::plugin,
        state::plugin,
        level::plugin,
        collision::plugin,
    ));

    app.register_type::<AtomAssets>();
    app.load_resource::<AtomAssets>();
    app.add_systems(OnEnter(Screen::Gameplay), init_level);
}

fn init_level(mut current_level: ResMut<CurrentLevel>) {
    current_level.0 = Some(Level {
        atoms: vec![
            LevelAtom::new(AtomType::Splitting, (2, 0)),
            LevelAtom::new_with_velocity(AtomType::Basic, (-2, 0), CardinalDirection::E),
            LevelAtom::new(AtomType::Basic, (3, 1)),
        ],
    });
}
