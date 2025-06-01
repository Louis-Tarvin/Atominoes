use bevy::prelude::*;

use super::{
    atom::{AtomAssets, AtomType, atom},
    movement::{CardinalDirection, Movement},
    state::GameState,
};

#[derive(Resource, Default)]
pub(super) struct CurrentLevel(pub Option<Level>);

pub(super) struct Level {
    pub atoms: Vec<LevelAtom>,
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

/// Marker component for entities that are part of the current level and thus need to be despawned
/// when the level is unloaded.
#[derive(Component)]
pub struct LevelEntity;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CurrentLevel>();
    app.add_systems(OnEnter(GameState::Placement), initialise_level);
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
            atom(
                level_atom.atom_type,
                level_atom.position,
                &atom_assets,
                &mut texture_atlas_layouts,
            ),
            LevelEntity,
        ));
        if let Some(velocity) = &level_atom.velocity {
            entity.insert(velocity.clone());
        }
    }
    Ok(())
}
