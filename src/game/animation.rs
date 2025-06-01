use std::time::Duration;

use bevy::prelude::*;

use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.register_type::<Animated>();
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (update_animation_atlas,).chain().in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Animated {
    timer: Timer,
    current_frame: u8,
    total_frames: u8,
}

impl Animated {
    pub fn new(total_frames: u8) -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_frame: 0,
            total_frames,
        }
    }

    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
        self.current_frame = (self.current_frame + 1) % self.total_frames;
    }
}

fn update_animation_timer(time: Res<Time>, mut query: Query<&mut Animated>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

fn update_animation_atlas(mut query: Query<(&Animated, &mut Sprite)>) {
    for (animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.timer.finished() {
            atlas.index = animation.current_frame as usize;
        }
    }
}
