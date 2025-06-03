use bevy::{
    ecs::component::{Immutable, StorageType},
    platform::collections::HashSet,
    prelude::*,
    render::view::RenderLayers,
    window::PrimaryWindow,
};

use crate::{AppSystems, PausableSystems, screens::Screen};

use super::{
    animation::Animated,
    atom::{AtomAssets, AtomType, atom},
    state::GameState,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OccupiedGridPositions>()
        .register_type::<OccupiedGridPositions>();
    app.init_resource::<DraggingState>()
        .register_type::<DraggingState>();
    app.add_event::<PlaceGhostAtom>();
    app.add_systems(Update, update_held_timer.in_set(AppSystems::TickTimers));
    app.add_systems(
        Update,
        update_drag_ghost_position_on_mouse_move
            .run_if(in_state(GameState::Placement))
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        update_dragging_state.run_if(in_state(GameState::Placement)),
    );
    app.add_observer(handle_place_atom);
}

#[derive(Resource, Default, Debug, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct OccupiedGridPositions(HashSet<IVec2>);

pub struct GridPos(pub IVec2);
impl Component for GridPos {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Immutable;

    fn on_insert() -> Option<bevy::ecs::component::ComponentHook> {
        Some(|mut world, context| {
            let value = world.get::<Self>(context.entity).unwrap().0;
            world.resource_mut::<OccupiedGridPositions>().insert(value);
        })
    }

    fn on_replace() -> Option<bevy::ecs::component::ComponentHook> {
        Some(|mut world, context| {
            let value = world.get::<Self>(context.entity).unwrap().0;
            world.resource_mut::<OccupiedGridPositions>().remove(&value);
        })
    }
}

#[derive(Component)]
pub struct DraggingGhost;

fn update_drag_ghost_position_on_mouse_move(
    mut query: Query<&mut Transform, With<DraggingGhost>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    occupied_positions: Res<OccupiedGridPositions>,
) {
    if let Some(mouse_pos) = window.cursor_position() {
        if let Ok(mut transform) = query.single_mut() {
            if let Some((camera, camera_transform)) =
                camera_query.iter().find(|(cam, _)| cam.order == 2)
            {
                // Convert screen coordinates to world coordinates
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, mouse_pos) {
                    let grid_pos =
                        IVec2::new(world_pos.x.round() as i32, world_pos.y.round() as i32);

                    if occupied_positions.contains(&grid_pos) {
                        return;
                    }

                    transform.translation.x = grid_pos.x as f32;
                    transform.translation.y = grid_pos.y as f32;
                }
            }
        }
    }
}

pub fn atom_placement_ghost(
    atom_type: AtomType,
    atom_assets: &AtomAssets,
    dragging_state: &mut DraggingState,
) -> impl Bundle {
    *dragging_state = DraggingState::Held(0.0);
    (
        Name::new("Placement Ghost"),
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
        DraggingGhost,
        Animated::new(8),
        Transform::from_scale(Vec3::splat(0.002)),
        RenderLayers::layer(2),
        StateScoped(Screen::Gameplay),
    )
}

#[derive(Event)]
pub struct PlaceGhostAtom;

fn handle_place_atom(
    _trigger: Trigger<PlaceGhostAtom>,
    mut commands: Commands,
    ghost_query: Query<(Entity, &AtomType, &Transform), With<DraggingGhost>>,
    atom_assets: Res<AtomAssets>,
) {
    if let Ok((entity, atom_type, transform)) = ghost_query.single() {
        // Despawn the ghost
        commands.entity(entity).despawn();
        // Spawn the actual atom
        let grid_pos = IVec2::new(
            transform.translation.x.round() as i32,
            transform.translation.y.round() as i32,
        );
        commands.spawn(atom(*atom_type, grid_pos, &atom_assets));
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub enum DraggingState {
    #[default]
    NotDragging,
    /// Dragging with mouse1 held down. Contains how long its been held for
    Held(f32),
    /// Dragging after having released mouse1
    Clicked,
}

fn update_held_timer(time: Res<Time>, mut dragging_state: ResMut<DraggingState>) {
    if let DraggingState::Held(timer) = *dragging_state {
        *dragging_state = DraggingState::Held(timer + time.delta_secs());
    }
}

fn update_dragging_state(
    buttons: Res<ButtonInput<MouseButton>>,
    mut dragging_state: ResMut<DraggingState>,
    mut commands: Commands,
) {
    if buttons.just_released(MouseButton::Left) {
        if let DraggingState::Held(timer) = *dragging_state {
            if timer < 0.3 {
                *dragging_state = DraggingState::Clicked;
            } else {
                *dragging_state = DraggingState::NotDragging;
                commands.trigger(PlaceGhostAtom);
            }
        }
    } else if buttons.just_pressed(MouseButton::Left)
        && matches!(*dragging_state, DraggingState::Clicked)
    {
        *dragging_state = DraggingState::NotDragging;
        commands.trigger(PlaceGhostAtom);
    }
}
