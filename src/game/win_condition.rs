use bevy::{prelude::*, render::view::RenderLayers};

use crate::{AppSystems, PausableSystems, screens::Screen};

use super::{
    atom::{AtomAssets, AtomType},
    state::GameState,
};

#[derive(Component)]
pub struct Goal(pub AtomType);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (check_goal_collisions, check_win_condition)
            .run_if(in_state(GameState::Running))
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

pub fn goal(
    atom_type: AtomType,
    position: IVec2,
    atom_assets: &AtomAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::new(364, 304), 9, 1, None, None);
    (
        Name::new("goal"),
        Sprite {
            image: match atom_type {
                AtomType::Basic => atom_assets.basic.clone(),
                AtomType::Splitting => atom_assets.splitting.clone(),
            },
            color: Color::srgba(1.0, 1.0, 1.0, 0.5), // 50% transparent
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layouts.add(layout),
                index: 8,
            }),
            ..default()
        },
        Goal(atom_type),
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

fn check_win_condition(goals: Query<&Goal>) {
    if goals.is_empty() {
        // TODO: Handle win condition
        info!("Win condition met!");
    }
}
