use bevy::{audio::Volume, prelude::*};

use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Music>();
    app.register_type::<SoundEffect>();
    app.register_type::<AudioAssets>();
    app.load_resource::<AudioAssets>();

    app.add_systems(
        Update,
        apply_global_volume.run_if(resource_changed::<GlobalVolume>),
    );
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "music" category (e.g. global background music, soundtrack).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Music;

/// A music audio instance.
pub fn music(handle: Handle<AudioSource>) -> impl Bundle {
    (
        AudioPlayer(handle),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.5)),
        Music,
    )
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "sound effect" category (e.g. footsteps, the sound of a magic spell, a door opening).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SoundEffect;

/// A sound effect audio instance.
pub fn sound_effect(handle: Handle<AudioSource>) -> impl Bundle {
    (
        AudioPlayer(handle),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(1.5)),
        SoundEffect,
    )
}

/// [`GlobalVolume`] doesn't apply to already-running audio entities, so this system will update them.
fn apply_global_volume(
    global_volume: Res<GlobalVolume>,
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink)>,
) {
    for (playback, mut sink) in &mut audio_query {
        sink.set_volume(global_volume.volume * playback.volume);
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AudioAssets {
    #[dependency]
    pub bgm: Handle<AudioSource>,
    #[dependency]
    pub merge_sfx: Handle<AudioSource>,
    #[dependency]
    pub split_sfx: Handle<AudioSource>,
    #[dependency]
    pub hit_sfx: Handle<AudioSource>,
    #[dependency]
    pub split_big_sfx: Handle<AudioSource>,
    #[dependency]
    pub level_complete_sfx: Handle<AudioSource>,
}
impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            bgm: assets.load("audio/music/bgm.ogg"),
            merge_sfx: assets.load("audio/sound_effects/sfx1.ogg"),
            split_sfx: assets.load("audio/sound_effects/sfx2.ogg"),
            hit_sfx: assets.load("audio/sound_effects/sfx3.ogg"),
            split_big_sfx: assets.load("audio/sound_effects/sfx4.ogg"),
            level_complete_sfx: assets.load("audio/sound_effects/sfx5.ogg"),
        }
    }
}
