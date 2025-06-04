use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    input::common_conditions::input_just_released,
    prelude::*,
    render::view::RenderLayers,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{asset_tracking::LoadResource, screens::Screen};

use super::{
    atom::{AtomAssets, AtomType, atom},
    movement::{CardinalDirection, Movement},
    state::GameState,
    win_condition::goal,
};

#[derive(Resource, Default)]
pub struct CurrentLevel {
    level_handle: Option<Handle<Level>>,
    level_index: Option<usize>,
}

impl CurrentLevel {
    pub fn set_level(&mut self, handle: Handle<Level>, index: usize) {
        self.level_handle = Some(handle);
        self.level_index = Some(index);
    }

    pub fn get_level<'a>(
        &self,
        level_assets: &'a Assets<Level>,
    ) -> Result<&'a Level, GetLevelError> {
        let level_handle = self.level_handle.as_ref().ok_or(GetLevelError::NoLevel)?;
        level_assets
            .get(level_handle)
            .ok_or(GetLevelError::InvalidHandle)
    }

    pub fn get_index(&self) -> Option<usize> {
        self.level_index
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
    pub atoms: Vec<LevelAtom>,
    pub goals: Vec<LevelGoal>,
    pub placeable_atoms: Vec<AtomType>,
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
pub struct LevelGoal {
    pub atom_type: AtomType,
    pub position: IVec2,
}

#[allow(dead_code)]
impl LevelGoal {
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
        Self {
            levels: vec![assets.load("levels/0.ron"), assets.load("levels/1.ron")],
        }
    }
}

/// Marker component for entities that are part of the current level and thus need to be despawned
/// when the level is unloaded.
#[derive(Component)]
pub struct LevelEntity;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<Level>()
        .init_asset_loader::<LevelAssetLoader>();
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
    app.init_resource::<CurrentLevel>();
    app.add_systems(OnEnter(Screen::Gameplay), draw_2d_grid);
    app.add_systems(OnEnter(GameState::Placement), initialise_level);
    app.add_systems(Update, draw_arrows.run_if(in_state(GameState::Placement)));
    app.add_systems(
        Update,
        print_level_ron.run_if(input_just_released(KeyCode::F2)),
    );
}

fn initialise_level(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    level_assets: Res<Assets<Level>>,
    atom_assets: Res<AtomAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    level_entities: Query<Entity, With<LevelEntity>>,
) -> Result {
    // First clean up existing level entities
    for entity in level_entities.iter() {
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

fn print_level_ron(current_level: Res<CurrentLevel>, level_assets: Res<Assets<Level>>) -> Result {
    let level = current_level.get_level(&level_assets)?;
    println!(
        "{}",
        ron::ser::to_string_pretty(&level, ron::ser::PrettyConfig::default()).unwrap()
    );

    Ok(())
}
