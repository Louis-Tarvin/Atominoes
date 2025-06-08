use bevy::{prelude::*, render::view::RenderLayers};
use serde::{Deserialize, Serialize};

use crate::screens::Screen;

use super::{animation::Animated, placement::GridPos};

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum AtomType {
    Basic,
    Splitting,
    Wall,
    Reactive,
    Antimatter,
}

impl AtomType {
    pub fn get_image_handle(&self, atom_assets: &AtomAssets) -> Handle<Image> {
        match self {
            AtomType::Basic => atom_assets.basic.clone(),
            AtomType::Splitting => atom_assets.splitting.clone(),
            AtomType::Wall => atom_assets.wall.clone(),
            AtomType::Reactive => atom_assets.reactive.clone(),
            AtomType::Antimatter => atom_assets.antimatter.clone(),
        }
    }

    pub fn get_sprite(&self, atom_assets: &AtomAssets) -> Sprite {
        match self {
            AtomType::Wall => Sprite::from_image(self.get_image_handle(atom_assets)),
            _ => Sprite::from_atlas_image(
                self.get_image_handle(atom_assets),
                TextureAtlas {
                    layout: atom_assets.atlas_layout.clone(),
                    index: 0,
                },
            ),
        }
    }
    pub fn get_ghost_sprite(&self, atom_assets: &AtomAssets) -> Sprite {
        match self {
            AtomType::Wall => {
                Sprite {
                    image: self.get_image_handle(atom_assets),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.5), // 50% transparent
                    ..Default::default()
                }
            }
            _ => {
                Sprite {
                    image: self.get_image_handle(atom_assets),
                    texture_atlas: Some(TextureAtlas {
                        layout: atom_assets.atlas_layout.clone(),
                        index: 8,
                    }),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.5), // 50% transparent
                    ..Default::default()
                }
            }
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AtomAssets {
    pub atlas_layout: Handle<TextureAtlasLayout>,
    #[dependency]
    pub basic: Handle<Image>,
    #[dependency]
    pub splitting: Handle<Image>,
    #[dependency]
    pub wall: Handle<Image>,
    #[dependency]
    pub reactive: Handle<Image>,
    #[dependency]
    pub antimatter: Handle<Image>,
    #[dependency]
    pub circle: Handle<Image>,
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
            wall: assets.load("images/atom3.png"),
            reactive: assets.load("images/atom4.png"),
            antimatter: assets.load("images/atom5.png"),
            circle: assets.load("images/circle.png"),
        }
    }
}

pub fn atom(atom_type: AtomType, position: IVec2, atom_assets: &AtomAssets) -> impl Bundle {
    (
        Name::new("Atom"),
        atom_type.get_sprite(atom_assets),
        atom_type,
        GridPos(position),
        Animated::new(8),
        Transform::from_xyz(position.x as f32, position.y as f32, 0.0)
            .with_scale(Vec3::splat(0.002)),
        RenderLayers::layer(2),
        StateScoped(Screen::Gameplay),
    )
}
