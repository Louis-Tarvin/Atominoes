use bevy::{prelude::*, render::view::RenderLayers};

use crate::screens::Screen;

use super::animation::Animated;

#[derive(Clone, Copy, Component, PartialEq, Eq, PartialOrd)]
pub enum AtomType {
    Basic,
    Splitting,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AtomAssets {
    #[dependency]
    pub basic: Handle<Image>,
    #[dependency]
    pub splitting: Handle<Image>,
}

impl FromWorld for AtomAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            basic: assets.load("images/atom1.png"),
            splitting: assets.load("images/atom2.png"),
        }
    }
}

pub fn atom(
    atom_type: AtomType,
    position: IVec2,
    atom_assets: &AtomAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::new(364, 304), 9, 1, None, None);
    (
        Name::new("Atom"),
        Sprite::from_atlas_image(
            match atom_type {
                AtomType::Basic => atom_assets.basic.clone(),
                AtomType::Splitting => atom_assets.splitting.clone(),
            },
            TextureAtlas {
                layout: texture_atlas_layouts.add(layout),
                index: 0,
            },
        ),
        atom_type,
        Animated::new(8),
        Transform::from_xyz(position.x as f32, position.y as f32, 0.0)
            .with_scale(Vec3::splat(0.002)),
        RenderLayers::layer(2),
        StateScoped(Screen::Gameplay),
    )
}
