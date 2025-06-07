use bevy::{ecs::spawn::SpawnWith, prelude::*};

use crate::{
    game::{
        CurrentLevel,
        atom::{AtomAssets, AtomType},
        level::Level,
        placement::{DraggingState, atom_placement_ghost},
    },
    theme::{palette::*, prelude::InteractionPalette},
};

#[derive(Component)]
pub(super) struct UiTray;

pub(super) fn tray() -> impl Bundle {
    (
        Name::new("Tray"),
        Node {
            bottom: Val::Px(0.0),
            height: Val::Px(64.0),
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect::all(Val::Px(5.0)),
            display: Display::Flex,
            position_type: PositionType::Absolute,
            column_gap: Val::Px(5.0),
            ..Default::default()
        },
        UiTray,
        BackgroundColor(BACKGROUND),
        BorderColor(ACCENT),
    )
}

#[derive(Component)]
struct AtomDragIcon;

fn drag_icon(atom_type: AtomType, atom_assets: &AtomAssets) -> impl Bundle {
    let atom_assets = atom_assets.clone();
    (
        Name::new(format!("{:?} drag point", atom_type)),
        Node::default(),
        AtomDragIcon,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Node {
                        height: Val::Px(50.0),
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::horizontal(Val::Px(30.0)),
                        aspect_ratio: Some(1.0),
                        ..Default::default()
                    },
                    ImageNode {
                        image: atom_type.get_image_handle(&atom_assets),
                        texture_atlas: Some(TextureAtlas {
                            layout: atom_assets.atlas_layout.clone(),
                            index: 8,
                        }),
                        ..Default::default()
                    },
                    BorderColor(OFF_WHITE),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    InteractionPalette {
                        none: BUTTON_BACKGROUND,
                        hovered: BUTTON_HOVERED_BACKGROUND,
                        pressed: BUTTON_PRESSED_BACKGROUND,
                    },
                ))
                .observe(
                    move |_: Trigger<Pointer<Pressed>>,
                          mut commands: Commands,
                          mut dragging_state: ResMut<DraggingState>| {
                        if let DraggingState::NotDragging = *dragging_state {
                            commands.spawn(atom_placement_ghost(
                                atom_type,
                                &atom_assets,
                                &mut dragging_state,
                            ));
                        }
                    },
                );
        })),
    )
}

pub(super) fn update_drag_icons(
    mut commands: Commands,
    tray_query: Option<Single<(Entity, Option<&Children>), With<UiTray>>>,
    atom_assets: Res<AtomAssets>,
    current_level: Res<CurrentLevel>,
    level_assets: Res<Assets<Level>>,
) {
    if let Some((tray_entity, tray_children)) = tray_query.map(|q| q.into_inner()) {
        if let Some(children) = tray_children {
            for child in children {
                commands.entity(*child).despawn();
            }
        }
        if let Ok(level) = current_level.get_level(&level_assets) {
            commands.entity(tray_entity).with_children(|tray| {
                for atom in &level.placeable_atoms {
                    tray.spawn(drag_icon(*atom, &atom_assets));
                }
            });
        }
    }
}
