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
    } else if event.entities.len() >= 3 {
        // Despawn all colliding atoms
        for &entity in &event.entities {
            commands.entity(entity).despawn();
        }
        // 3 way collision always spawns a (motionless) reactive atom
        commands.spawn((
            atom(AtomType::Reactive, event.position, &atom_assets),
            LevelEntity,
            CollisionCooldown::default(),
        ));
        return;
    }

    // We know there are exactly 2 entities at this point
    let [entity1, entity2] = [event.entities[0], event.entities[1]];

    let Ok((atom_type1, movement1)) = atom_query.get(entity1) else {
        return;
    };
    let Ok((atom_type2, movement2)) = atom_query.get(entity2) else {
        return;
    };

    // Handle collision based on atom types
    match (atom_type1, atom_type2) {
        // Wall and antimatter - destroy both
        (AtomType::Wall, AtomType::Antimatter) | (AtomType::Antimatter, AtomType::Wall) => {
            commands.entity(entity1).despawn();
            commands.entity(entity2).despawn();
        }

        // Wall collisions - bounce the non-wall atom
        (AtomType::Wall, other_type) => {
            handle_wall_collision(
                entity2,
                *other_type,
                movement2,
                event.position,
                &mut commands,
                &atom_assets,
            );
        }
        (other_type, AtomType::Wall) => {
            handle_wall_collision(
                entity1,
                *other_type,
                movement1,
                event.position,
                &mut commands,
                &atom_assets,
            );
        }

        // Two basic atoms fuse into a splitting atom
        (AtomType::Basic, AtomType::Basic) => {
            commands.entity(entity1).despawn();
            commands.entity(entity2).despawn();

            let movement = movement1.cloned().or_else(|| movement2.cloned());

            let mut entity = commands.spawn((
                atom(AtomType::Splitting, event.position, &atom_assets),
                LevelEntity,
                CollisionCooldown::default(),
            ));

            if let Some(movement) = movement {
                entity.insert(movement);
            }
        }

        // Reactive and splitting - spawn antimatter
        (AtomType::Reactive, AtomType::Splitting) | (AtomType::Splitting, AtomType::Reactive) => {
            commands.entity(entity1).despawn();
            commands.entity(entity2).despawn();

            commands.spawn((
                atom(AtomType::Antimatter, event.position, &atom_assets),
                LevelEntity,
                CollisionCooldown::default(),
            ));
        }

        // Basic and antimatter - similar to splitting but in opposite direction
        (AtomType::Basic, AtomType::Antimatter) | (AtomType::Antimatter, AtomType::Basic) => {
            commands.entity(entity1).despawn();
            commands.entity(entity2).despawn();

            let movement = movement1.cloned().or_else(|| movement2.cloned());

            match movement {
                Some(movement) => {
                    let (dir1, dir2) = get_split_directions(movement.direction.opposite());

                    // Spawn basic atoms at 45 degree angles
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
                }
                None => {
                    warn!("Collision between two stationary atoms. This shouldn't happen.");
                    commands.spawn((
                        atom(AtomType::Basic, event.position, &atom_assets),
                        LevelEntity,
                    ));
                }
            }
        }

        // Splitting atom collisions - always split
        (AtomType::Splitting, _) | (_, AtomType::Splitting) => {
            commands.entity(entity1).despawn();
            commands.entity(entity2).despawn();

            let movement = movement1.cloned().or_else(|| movement2.cloned());

            match movement {
                Some(movement) => {
                    let (dir1, dir2) = get_split_directions(movement.direction);

                    // Spawn basic atoms at 45 degree angles
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
                }
                None => {
                    warn!("Collision between two stationary atoms. This shouldn't happen.");
                    commands.spawn((
                        atom(AtomType::Basic, event.position, &atom_assets),
                        LevelEntity,
                    ));
                }
            }
        }

        // Antimatter phases through other atoms
        (AtomType::Antimatter, _) | (_, AtomType::Antimatter) => {
            return;
        }

        _ => {
            warn!(
                "Unhandled collision between {:?} and {:?}",
                atom_type1, atom_type2
            );
            commands.entity(entity1).despawn();
            commands.entity(entity2).despawn();
        }
    }
}

fn handle_wall_collision(
    other_entity: Entity,
    other_type: AtomType,
    other_movement: Option<&Movement>,
    position: IVec2,
    commands: &mut Commands,
    atom_assets: &AtomAssets,
) {
    match other_movement {
        Some(movement) => {
            // Bounce the atom by reversing its direction
            let mut bounced_movement = movement.clone();
            bounced_movement.direction = bounced_movement.direction.opposite();

            // Despawn the original atom
            commands.entity(other_entity).despawn();

            // Spawn bounced atom
            commands.spawn((
                atom(other_type, position, atom_assets),
                bounced_movement,
                LevelEntity,
                CollisionCooldown::default(),
            ));
        }
        None => {
            // Stationary atom hitting wall - just despawn it
            commands.entity(other_entity).despawn();
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
            debug!("{} entities collided at {pos}", entities.len());
            commands.trigger(CollisionEvent {
                entities,
                position: pos,
            });
        }
    }
}
