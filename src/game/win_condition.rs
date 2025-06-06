use bevy::{prelude::*, render::view::RenderLayers};

use crate::{AppSystems, PausableSystems, screens::Screen};

use super::{
    atom::{AtomAssets, AtomType},
    level::{CurrentLevel, Level},
    placement::GridPos,
    state::GameState,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (check_goal_collisions, check_win_condition)
            .run_if(in_state(GameState::Running))
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component)]
pub struct Goal(pub AtomType);

pub fn goal(atom_type: AtomType, position: IVec2, atom_assets: &AtomAssets) -> impl Bundle {
    (
        Name::new("goal"),
        atom_type.get_ghost_sprite(atom_assets),
        Goal(atom_type),
        GridPos(position),
        Transform::from_xyz(position.x as f32, position.y as f32, 0.0)
            .with_scale(Vec3::splat(0.002)),
        RenderLayers::layer(2),
        StateScoped(Screen::Gameplay),
    )
}

fn check_goal_collisions(
    mut commands: Commands,
    goals: Query<(Entity, &Transform, &Goal)>,
    atoms: Query<(Entity, &Transform, &AtomType), Without<Goal>>,
) {
    for (goal_entity, goal_transform, goal) in &goals {
        for (atom_entity, atom_transform, atom_type) in &atoms {
            // Check if atom type matches goal type
            if *atom_type == goal.0 {
                let distance = goal_transform
                    .translation
                    .distance(atom_transform.translation);
                let collision_threshold = 0.5;

                if distance < collision_threshold {
                    // Destroy both the goal and the atom
                    commands.entity(goal_entity).despawn();
                    commands.entity(atom_entity).despawn();
                }
            }
        }
    }
}

fn check_win_condition(
    goals: Query<&Goal>,
    atoms: Query<&AtomType, Without<Goal>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_level: Res<CurrentLevel>,
    level_assets: Res<Assets<Level>>,
) {
    if let CurrentLevel::Loaded {
        level_handle,
        level_index: _,
    } = &*current_level
    {
        match level_assets
            .get(level_handle)
            .expect("Current level should exist")
            .goal
        {
            super::level::LevelGoal::None => return,
            super::level::LevelGoal::ReachPositions(_) => {
                if goals.is_empty() {
                    next_state.set(GameState::LevelComplete);
                }
            }
            super::level::LevelGoal::CreateAtom(atom_type) => {
                if atoms
                    .iter()
                    .any(|&existing_atom_type| existing_atom_type == atom_type)
                {
                    next_state.set(GameState::LevelComplete);
                }
            }
        }
    }
}
