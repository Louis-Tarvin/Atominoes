use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    input::common_conditions::input_just_released,
    platform::collections::HashMap,
    prelude::*,
    render::view::RenderLayers,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{LEVELS, asset_tracking::LoadResource, screens::Screen};

use super::{
    atom::{AtomAssets, AtomType, atom},
    movement::{CardinalDirection, Movement},
    state::GameState,
    win_condition::goal,
};

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<Level>()
        .init_asset_loader::<LevelAssetLoader>();
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
    app.init_resource::<CurrentLevel>();
    app.init_resource::<PlacedLevelAtoms>();
    app.add_systems(OnEnter(Screen::Gameplay), draw_2d_grid);
    app.add_systems(OnEnter(GameState::Placement), initialise_level);
    app.add_systems(Update, draw_arrows.run_if(in_state(GameState::Placement)));
    app.add_systems(
        Update,
        print_level_ron.run_if(input_just_released(KeyCode::F2)),
    );
}

#[derive(Resource, Default)]
pub enum CurrentLevel {
    #[default]
    Uninitialised,
    Loaded {
        level_handle: Handle<Level>,
        level_index: usize,
    },
    Editing(Level),
}

impl CurrentLevel {
    pub fn set_level(&mut self, handle: Handle<Level>, index: usize) {
        *self = Self::Loaded {
            level_handle: handle,
            level_index: index,
        };
    }

    pub fn get_level<'a>(
        &'a self,
        level_assets: &'a Assets<Level>,
    ) -> Result<&'a Level, GetLevelError> {
        match self {
            CurrentLevel::Uninitialised => Err(GetLevelError::NoLevel),
            CurrentLevel::Loaded {
                level_handle,
                level_index: _,
            } => level_assets
                .get(level_handle)
                .ok_or(GetLevelError::InvalidHandle),
            CurrentLevel::Editing(level) => Ok(level),
        }
    }

    pub fn get_index(&self) -> Option<usize> {
        match self {
            CurrentLevel::Uninitialised => None,
            CurrentLevel::Loaded { level_index, .. } => Some(*level_index),
            CurrentLevel::Editing(_) => None,
        }
    }

    pub fn is_editing(&self) -> bool {
        matches!(self, CurrentLevel::Editing(_))
    }
}

#[derive(Debug, Error)]
pub enum GetLevelError {
    #[error("Attempted to read level, but no level was loaded!")]
    NoLevel,
    #[error("Current level handle was invalid!")]
    InvalidHandle,
}

#[derive(Asset, TypePath, Debug, Serialize, Deserialize)]
pub struct Level {
    pub sidebar_text: String,
    pub level_complete_text: String,
    pub atoms: Vec<LevelAtom>,
    pub goal: LevelGoal,
    pub placeable_atoms: Vec<AtomType>,
}

impl Level {
    pub fn remove_atom_at_position(&mut self, position: IVec2) -> Option<LevelAtom> {
        if let Some(index) = self.atoms.iter().position(|atom| atom.position == position) {
            Some(self.atoms.remove(index))
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LevelAtom {
    pub atom_type: AtomType,
    pub position: IVec2,
    pub velocity: Option<Movement>,
}

#[allow(dead_code)]
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

#[derive(Serialize, Deserialize, Debug)]
pub enum LevelGoal {
    None,
    ReachPositions(Vec<LevelGoalPosition>),
    CreateAtom(AtomType),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LevelGoalPosition {
    pub atom_type: AtomType,
    pub position: IVec2,
}

#[allow(dead_code)]
impl LevelGoalPosition {
    pub fn new<P: Into<IVec2>>(atom_type: AtomType, position: P) -> Self {
        Self {
            atom_type,
            position: position.into(),
        }
    }
}

#[derive(Default)]
struct LevelAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
enum LevelAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for LevelAssetLoader {
    type Asset = Level;
    type Settings = ();
    type Error = LevelAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = ron::de::from_bytes::<Level>(&bytes)?;
        Ok(custom_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    pub levels: Vec<Handle<Level>>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let mut levels = Vec::new();
        for level in LEVELS {
            levels.push(assets.load(format!("levels/{}", level)));
        }
        Self { levels }
    }
}

/// Contains atoms that have been placed by the player and are not part of the level
#[derive(Resource, Default)]
pub struct PlacedLevelAtoms(HashMap<IVec2, AtomType>);
impl PlacedLevelAtoms {
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn add(&mut self, atom_type: AtomType, position: IVec2) {
        if self.0.insert(position, atom_type).is_some() {
            warn!("Tried to place an atom in an occupied position");
        }
    }
    pub fn remove(&mut self, position: &IVec2) {
        if self.0.remove(position).is_none() {
            warn!("Tried to remove a placed atom, but none existed at that location");
        }
    }
}

/// Marker component for entities that are part of the current level and thus need to be despawned
/// when the level is unloaded.
#[derive(Component)]
pub struct LevelEntity;

fn initialise_level(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    level_assets: Res<Assets<Level>>,
    placed_atoms: Res<PlacedLevelAtoms>,
    atom_assets: Res<AtomAssets>,
    atom_entities: Query<Entity, With<AtomType>>,
    level_entities: Query<Entity, (With<LevelEntity>, Without<AtomType>)>,
) -> Result {
    // First clean up existing level entities
    for entity in atom_entities.iter().chain(level_entities) {
        commands.entity(entity).despawn();
    }

    let level = current_level.get_level(&level_assets)?;

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
    // Spawn placed atoms
    for (position, atom_type) in &placed_atoms.0 {
        commands.spawn(atom(*atom_type, *position, &atom_assets));
    }

    // Spawn goal zones
    if let LevelGoal::ReachPositions(zones) = &level.goal {
        for goal_zone in zones {
            commands.spawn((
                goal(goal_zone.atom_type, goal_zone.position, &atom_assets),
                LevelEntity,
            ));
        }
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
        let direction = movement.direction.as_velocity();
        let position = transform.translation.xy();
        gizmos.arrow_2d(
            position + (direction * 0.3),
            position + (direction * 1.2),
            LinearRgba::rgb(0.4, 0.4, 0.8),
        );
    }
}

fn print_level_ron(current_level: Res<CurrentLevel>, level_assets: Res<Assets<Level>>) -> Result {
    let level = current_level.get_level(&level_assets)?;
    println!(
        "{}",
        ron::ser::to_string_pretty(&level, ron::ser::PrettyConfig::default()).unwrap()
    );

    Ok(())
}
