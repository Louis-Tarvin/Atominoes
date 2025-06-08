use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{AppSystems, PausableSystems};

use super::state::GameState;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        move_atoms_system
            .run_if(in_state(GameState::Running))
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Movement {
    pub direction: CardinalDirection,
    pub speed: f32,
}

impl Movement {
    pub fn new(direction: CardinalDirection) -> Self {
        Self {
            direction,
            speed: 2.0,
        }
    }

    pub fn velocity(&self) -> Vec2 {
        self.direction.as_velocity() * self.speed
    }
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            direction: CardinalDirection::N,
            speed: 2.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CardinalDirection {
    N,
    E,
    S,
    W,
    NE,
    SE,
    SW,
    NW,
}

impl CardinalDirection {
    pub fn as_velocity(&self) -> Vec2 {
        match self {
            CardinalDirection::N => Vec2::new(0.0, 1.0),
            CardinalDirection::E => Vec2::new(1.0, 0.0),
            CardinalDirection::S => Vec2::new(0.0, -1.0),
            CardinalDirection::W => Vec2::new(-1.0, 0.0),
            CardinalDirection::NE => Vec2::new(1.0, 1.0).normalize(),
            CardinalDirection::SE => Vec2::new(1.0, -1.0).normalize(),
            CardinalDirection::SW => Vec2::new(-1.0, -1.0).normalize(),
            CardinalDirection::NW => Vec2::new(-1.0, 1.0).normalize(),
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            CardinalDirection::N => CardinalDirection::S,
            CardinalDirection::E => CardinalDirection::W,
            CardinalDirection::S => CardinalDirection::N,
            CardinalDirection::W => CardinalDirection::E,
            CardinalDirection::NE => CardinalDirection::SW,
            CardinalDirection::SE => CardinalDirection::NW,
            CardinalDirection::SW => CardinalDirection::NE,
            CardinalDirection::NW => CardinalDirection::SE,
        }
    }

    pub fn clockwise(&self) -> Self {
        match self {
            CardinalDirection::N => CardinalDirection::NE,
            CardinalDirection::NE => CardinalDirection::E,
            CardinalDirection::E => CardinalDirection::SE,
            CardinalDirection::SE => CardinalDirection::S,
            CardinalDirection::S => CardinalDirection::SW,
            CardinalDirection::SW => CardinalDirection::W,
            CardinalDirection::W => CardinalDirection::NW,
            CardinalDirection::NW => CardinalDirection::N,
        }
    }

    pub fn anticlockwise(&self) -> Self {
        match self {
            CardinalDirection::N => CardinalDirection::NW,
            CardinalDirection::NW => CardinalDirection::W,
            CardinalDirection::W => CardinalDirection::SW,
            CardinalDirection::SW => CardinalDirection::S,
            CardinalDirection::S => CardinalDirection::SE,
            CardinalDirection::SE => CardinalDirection::E,
            CardinalDirection::E => CardinalDirection::NE,
            CardinalDirection::NE => CardinalDirection::N,
        }
    }
}

fn move_atoms_system(mut query: Query<(&mut Transform, &Movement)>, time: Res<Time>) {
    for (mut transform, movement) in &mut query {
        let velocity = movement.velocity();

        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}
