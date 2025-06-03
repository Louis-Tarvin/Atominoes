use bevy::{prelude::*, render::view::RenderLayers};

use crate::screens::Screen;

use super::{
    atom::{AtomAssets, AtomType, atom},
    movement::{CardinalDirection, Movement},
    state::GameState,
    win_condition::goal,
};

#[derive(Resource, Default)]
pub(super) struct CurrentLevel(pub Option<Level>);

pub(super) struct Level {
    pub sidebar_text: String,
    pub atoms: Vec<LevelAtom>,
    pub goals: Vec<LevelGoal>,
    pub placeable_atoms: Vec<AtomType>,
}

pub(super) struct LevelAtom {
    pub atom_type: AtomType,
    pub position: IVec2,
    pub velocity: Option<Movement>,
}

impl LevelAtom {
    pub fn new<P: Into<IVec2>>(atom_type: AtomType, position: P) -> Self {
        Self {
            atom_type,
            position: position.into(),
            velocity: None,
        }
    }

    pub fn new_with_velocity<P: Into<IVec2>>(
        atom_type: AtomType,
        position: P,
        direction: CardinalDirection,
    ) -> Self {
        Self {
            atom_type,
            position: position.into(),
            velocity: Some(Movement::new(direction)),
        }
    }
}

pub(super) struct LevelGoal {
    pub atom_type: AtomType,
    pub position: IVec2,
}

impl LevelGoal {
    pub fn new<P: Into<IVec2>>(atom_type: AtomType, position: P) -> Self {
        Self {
            atom_type,
            position: position.into(),
        }
    }
}

/// Marker component for entities that are part of the current level and thus need to be despawned
/// when the level is unloaded.
#[derive(Component)]
pub struct LevelEntity;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CurrentLevel>();
    app.add_systems(OnEnter(Screen::Gameplay), draw_2d_grid);
    app.add_systems(OnEnter(GameState::Placement), initialise_level);
    app.add_systems(Update, draw_arrows.run_if(in_state(GameState::Placement)));
}

fn initialise_level(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    atom_assets: Res<AtomAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    level_entities: Query<Entity, With<LevelEntity>>,
) -> Result {
    // First clean up existing level entities
    for entity in level_entities.iter() {
        commands.entity(entity).despawn();
    }

    let level = current_level
        .0
        .as_ref()
        .ok_or("Attempted to load level, but no level was loaded!")?;

    // Spawn level atoms
    for level_atom in &level.atoms {
        let mut entity = commands.spawn((
            atom(level_atom.atom_type, level_atom.position, &atom_assets),
            LevelEntity,
        ));
        if let Some(velocity) = &level_atom.velocity {
            entity.insert(velocity.clone());
        }
    }

    // Spawn goal zones
    for goal_zone in &level.goals {
        commands.spawn((
            goal(
                goal_zone.atom_type,
                goal_zone.position,
                &atom_assets,
                &mut texture_atlas_layouts,
            ),
            LevelEntity,
        ));
    }
    Ok(())
}

fn draw_2d_grid(mut commands: Commands, mut gizmo_assets: ResMut<Assets<GizmoAsset>>) {
    let mut gizmo = GizmoAsset::new();
    gizmo.grid_2d(
        Isometry2d::IDENTITY,
        UVec2::new(30, 20),
        Vec2::splat(1.0),
        LinearRgba::gray(0.05),
    );
    commands.spawn((
        Gizmo {
            handle: gizmo_assets.add(gizmo),
            ..Default::default()
        },
        RenderLayers::layer(1),
        StateScoped(Screen::Gameplay),
    ));
}

fn draw_arrows(mut gizmos: Gizmos, moving_atoms: Query<(&Movement, &Transform), With<AtomType>>) {
    for (movement, transform) in moving_atoms.iter() {
        let direction = movement.direction.to_velocity();
        let position = transform.translation.xy();
        gizmos.arrow_2d(
            position + (direction * 0.3),
            position + (direction * 1.2),
            LinearRgba::rgb(0.4, 0.4, 0.8),
        );
    }
}
