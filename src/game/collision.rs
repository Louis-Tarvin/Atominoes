use bevy::{platform::collections::HashMap, prelude::*};

use crate::{AppSystems, PausableSystems};

use super::{
    atom::{AtomAssets, AtomType, atom},
    level::LevelEntity,
    movement::{CardinalDirection, Movement},
    state::GameState,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(handle_collision);
    app.add_systems(
        Update,
        detect_atom_collisions
            .run_if(in_state(GameState::Running))
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        tick_collision_cooldown
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems),
    );
}

#[derive(Event)]
pub struct CollisionEvent {
    pub entities: Vec<Entity>,
    pub position: IVec2,
}

fn handle_collision(
    trigger: Trigger<CollisionEvent>,
    mut commands: Commands,
    atom_assets: Res<AtomAssets>,
    atom_query: Query<(&AtomType, Option<&Movement>)>,
) {
    let event = trigger.event();

    if event.entities.len() < 2 {
        return;
    }

    let mut atom_data = Vec::new();

    for &entity in &event.entities {
        if let Ok((atom_type, movement)) = atom_query.get(entity) {
            atom_data.push((entity, *atom_type, movement.cloned()));
        }
    }

    // Check for wall collisions first
    let wall_count = atom_data
        .iter()
        .filter(|(_, t, _)| *t == AtomType::Wall)
        .count();

    if wall_count > 0 {
        // Handle wall collision - bounce non-wall atoms
        for (entity, atom_type, movement) in atom_data {
            if atom_type == AtomType::Wall {
                // Keep walls intact - don't despawn them
                continue;
            } else if let Some(mut movement) = movement {
                // Bounce the atom by reversing its direction
                movement.direction = movement.direction.opposite();

                // Despawn the original atom
                commands.entity(entity).despawn();

                // Spawn bounced atom
                commands.spawn((
                    atom(atom_type, event.position, &atom_assets),
                    movement,
                    LevelEntity,
                    CollisionCooldown::default(),
                ));
            } else {
                // Stationary atom hitting wall - just despawn it
                commands.entity(entity).despawn();
            }
        }
        return;
    }

    // Handle normal collisions (no walls involved)
    // Despawn all colliding atoms
    for &entity in &event.entities {
        commands.entity(entity).despawn();
    }

    // Handle collision based on atom types
    let basic_count = atom_data
        .iter()
        .filter(|(_, t, _)| *t == AtomType::Basic)
        .count();
    let splitting_count = atom_data
        .iter()
        .filter(|(_, t, _)| *t == AtomType::Splitting)
        .count();

    if splitting_count > 0 {
        // Any splitting atom causes a split reaction
        let movement = atom_data.iter().find_map(|(_, _, m)| m.as_ref()).cloned();

        if let Some(movement) = movement {
            let (dir1, dir2) = get_split_directions(movement.direction);

            // Spawn basic atoms at split angles
            commands.spawn((
                atom(AtomType::Basic, event.position, &atom_assets),
                Movement::new(dir1),
                LevelEntity,
                CollisionCooldown::default(),
            ));

            commands.spawn((
                atom(AtomType::Basic, event.position, &atom_assets),
                Movement::new(dir2),
                LevelEntity,
                CollisionCooldown::default(),
            ));
        } else {
            // Fallback for stationary splitting atoms
            for _ in 0..2 {
                commands.spawn((
                    atom(AtomType::Basic, event.position, &atom_assets),
                    LevelEntity,
                ));
            }
        }
    } else if basic_count >= 2 {
        // Multiple basic atoms fuse into a splitting atom
        let movement = atom_data.iter().find_map(|(_, _, m)| m.as_ref()).cloned();

        let mut entity = commands.spawn((
            atom(AtomType::Splitting, event.position, &atom_assets),
            LevelEntity,
            CollisionCooldown::default(),
        ));

        if let Some(movement) = movement {
            entity.insert(movement);
        }
    }
}

/// Get the two directions for splitting at 45 degree angles
fn get_split_directions(direction: CardinalDirection) -> (CardinalDirection, CardinalDirection) {
    match direction {
        CardinalDirection::N => (CardinalDirection::NE, CardinalDirection::NW),
        CardinalDirection::E => (CardinalDirection::NE, CardinalDirection::SE),
        CardinalDirection::S => (CardinalDirection::SE, CardinalDirection::SW),
        CardinalDirection::W => (CardinalDirection::SW, CardinalDirection::NW),
        CardinalDirection::NE => (CardinalDirection::N, CardinalDirection::E),
        CardinalDirection::SE => (CardinalDirection::E, CardinalDirection::S),
        CardinalDirection::SW => (CardinalDirection::S, CardinalDirection::W),
        CardinalDirection::NW => (CardinalDirection::W, CardinalDirection::N),
    }
}

/// Used to prevent newly spawned atoms from immediately colliding again
#[derive(Component)]
pub struct CollisionCooldown(Timer);

impl Default for CollisionCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Once))
    }
}

fn tick_collision_cooldown(
    time: Res<Time>,
    mut query: Query<(Entity, &mut CollisionCooldown)>,
    mut commands: Commands,
) {
    for (entity, mut cc) in &mut query {
        cc.0.tick(time.delta());
        if cc.0.finished() {
            commands.entity(entity).remove::<CollisionCooldown>();
        }
    }
}

fn detect_atom_collisions(
    moving_atoms_query: Query<
        (Entity, &Transform, &Movement),
        (With<AtomType>, Without<CollisionCooldown>),
    >,
    stationary_atoms_query: Query<(Entity, &Transform), (With<AtomType>, Without<Movement>)>,
    mut commands: Commands,
) {
    let epsilon = 0.05;
    let mut potentially_colliding_positions: HashMap<IVec2, Vec<Entity>> = HashMap::new();

    for (moving_entity, moving_transform, _) in moving_atoms_query.iter() {
        // check if we're at a grid intersection (only time that collisions should occur)
        let pos = moving_transform.translation;
        let nearest_x = pos.x.round();
        let nearest_y = pos.y.round();
        let dx = (pos.x - nearest_x).abs();
        let dy = (pos.y - nearest_y).abs();

        if dx <= epsilon && dy <= epsilon {
            potentially_colliding_positions
                .entry(IVec2::new(nearest_x as i32, nearest_y as i32))
                .or_default()
                .push(moving_entity);
        }
    }

    if !potentially_colliding_positions.is_empty() {
        for (entity, transform) in stationary_atoms_query.iter() {
            let pos = transform.translation;
            // It's assumed that stationary atoms are already at grid intersections
            potentially_colliding_positions
                .entry(IVec2::new(pos.x.round() as i32, pos.y.round() as i32))
                .or_default()
                .push(entity);
        }
    }

    for (pos, entities) in potentially_colliding_positions.into_iter() {
        if entities.len() > 1 {
            debug!("Collision at {pos}");
            commands.trigger(CollisionEvent {
                entities,
                position: pos,
            });
        }
    }
}
