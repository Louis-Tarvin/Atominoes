use bevy::prelude::*;

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
    pub entities: (Entity, Entity),
    pub types: (AtomType, AtomType),
    pub positions: (Vec3, Vec3),
    pub movements: (Option<Movement>, Option<Movement>),
}

fn handle_collision(
    trigger: Trigger<CollisionEvent>,
    mut commands: Commands,
    atom_assets: Res<AtomAssets>,
) {
    let event = trigger.event();

    // Despawn the colliding atoms
    commands.entity(event.entities.0).despawn();
    commands.entity(event.entities.1).despawn();

    // Calculate collision position (midpoint)
    let collision_pos = (event.positions.0 + event.positions.1) / 2.0;
    let collision_ivec2 = IVec2::new(
        collision_pos.x.round() as i32,
        collision_pos.y.round() as i32,
    );

    match (event.types.0, event.types.1) {
        // Both atoms are basic - fuse
        (AtomType::Basic, AtomType::Basic) => {
            // Determine movement direction (use the moving atom's direction)
            let movement = event.movements.0.clone().or(event.movements.1.clone());

            let mut entity = commands.spawn((
                atom(AtomType::Splitting, collision_ivec2, &atom_assets),
                LevelEntity,
                CollisionCooldown::default(),
            ));

            if let Some(movement) = movement {
                entity.insert(movement);
            }
        }

        // One atom is splitting - split into two atoms at 45 degree angles
        (AtomType::Splitting, _) | (_, AtomType::Splitting) => {
            // Find the movement direction from either atom
            let base_movement = event.movements.0.clone().or(event.movements.1.clone());

            if let Some(movement) = base_movement {
                let (dir1, dir2) = get_split_directions(movement.direction);

                // Spawn first basic atom
                commands.spawn((
                    atom(AtomType::Basic, collision_ivec2, &atom_assets),
                    Movement::new(dir1),
                    LevelEntity,
                    CollisionCooldown::default(),
                ));

                // Spawn second basic atom
                commands.spawn((
                    atom(AtomType::Basic, collision_ivec2, &atom_assets),
                    Movement::new(dir2),
                    LevelEntity,
                    CollisionCooldown::default(),
                ));
            } else {
                warn!("Collision between two stationary atoms. This shouldn't happen.");
                for _ in 0..2 {
                    commands.spawn((
                        atom(AtomType::Basic, collision_ivec2, &atom_assets),
                        LevelEntity,
                    ));
                }
            }
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
    query: Query<(Entity, &Transform, &AtomType, Option<&Movement>), Without<CollisionCooldown>>,
    mut commands: Commands,
) {
    let atoms: Vec<_> = query.iter().collect();

    for (i, (entity_a, transform_a, atom_type_a, movement_a)) in atoms.iter().enumerate() {
        for (entity_b, transform_b, atom_type_b, movement_b) in atoms.iter().skip(i + 1) {
            let distance = transform_a.translation.distance(transform_b.translation);
            let collision_threshold = 0.5;

            if distance < collision_threshold {
                commands.trigger(CollisionEvent {
                    entities: (*entity_a, *entity_b),
                    types: (**atom_type_a, **atom_type_b),
                    positions: (transform_a.translation, transform_b.translation),
                    movements: (movement_a.cloned(), movement_b.cloned()),
                });
            }
        }
    }
}
