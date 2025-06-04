use bevy::{prelude::*, render::view::RenderLayers};
use serde::{Deserialize, Serialize};

use crate::screens::Screen;

use super::{animation::Animated, placement::GridPos};

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum AtomType {
    Basic,
    Splitting,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AtomAssets {
    pub atlas_layout: Handle<TextureAtlasLayout>,
    #[dependency]
    pub basic: Handle<Image>,
    #[dependency]
    pub splitting: Handle<Image>,
}

impl FromWorld for AtomAssets {
    fn from_world(world: &mut World) -> Self {
        let atlas_layout =
            world
                .resource_mut::<Assets<TextureAtlasLayout>>()
                .add(TextureAtlasLayout::from_grid(
                    UVec2::new(364, 304),
                    9,
                    1,
                    None,
                    None,
                ));
        let assets = world.resource::<AssetServer>();
        Self {
            atlas_layout,
            basic: assets.load("images/atom1.png"),
            splitting: assets.load("images/atom2.png"),
        }
    }
}

pub fn atom(atom_type: AtomType, position: IVec2, atom_assets: &AtomAssets) -> impl Bundle {
    (
        Name::new("Atom"),
        Sprite::from_atlas_image(
            match atom_type {
                AtomType::Basic => atom_assets.basic.clone(),
                AtomType::Splitting => atom_assets.splitting.clone(),
            },
            TextureAtlas {
                layout: atom_assets.atlas_layout.clone(),
                index: 0,
            },
        ),
        atom_type,
        GridPos(position),
        Animated::new(8),
        Transform::from_xyz(position.x as f32, position.y as f32, 0.0)
            .with_scale(Vec3::splat(0.002)),
        RenderLayers::layer(2),
        StateScoped(Screen::Gameplay),
    )
}
