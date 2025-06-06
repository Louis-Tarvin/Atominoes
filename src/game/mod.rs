use atom::{AtomAssets, AtomType};
use bevy::prelude::*;
use level::{CurrentLevel, Level, LevelAssets, LevelAtom, LevelGoal};

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
    app.init_resource::<MenuSelection>();
    app.add_systems(OnEnter(Screen::Gameplay), init_level);
}

#[derive(Resource, Default)]
pub enum MenuSelection {
    #[default]
    Editor,
    Level(usize),
}

fn init_level(
    mut current_level: ResMut<CurrentLevel>,
    level_handles: Res<LevelAssets>,
    menu_selection: Res<MenuSelection>,
) {
    if let MenuSelection::Level(index) = *menu_selection {
        current_level.set_level(level_handles.levels[index].clone(), index);
    } else {
        *current_level = CurrentLevel::Editing(Level {
            sidebar_text: "Sandbox".to_string(),
            atoms: vec![LevelAtom::new_with_velocity(
                AtomType::Basic,
                IVec2::new(-3, 0),
                movement::CardinalDirection::E,
            )],
            goal: LevelGoal::None,
            placeable_atoms: vec![
                AtomType::Basic,
                AtomType::Splitting,
                AtomType::Reactive,
                AtomType::Antimatter,
                AtomType::Wall,
            ],
        });
    }
}
