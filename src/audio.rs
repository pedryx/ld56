use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{loading::AudioAssets, GameState};

const SOUNDTRACK_GLOBAL_VOLUME: f64 = 1.4;
pub const SOUND_EFFECTS_GLOBAL_VOLUME: f64 = 0.1;

pub struct InternalAudioPlugin;

impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .init_resource::<Soundtracks>()
            .add_systems(OnExit(GameState::Loading), start_soundtrack);
    }
}

#[derive(Resource, Default)]
pub struct Soundtracks {
    pub base: Handle<AudioInstance>,
    pub battle: Handle<AudioInstance>,
}

fn start_soundtrack(
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
    mut soundtrack: ResMut<Soundtracks>,
) {
    soundtrack.base = audio
        .play(audio_assets.base_soundtrack.clone())
        .looped()
        .with_volume(SOUNDTRACK_GLOBAL_VOLUME * 0.2)
        .handle();

    soundtrack.battle = audio
        .play(audio_assets.battle_soundtrack.clone())
        .looped()
        .with_volume(SOUNDTRACK_GLOBAL_VOLUME * 0.6)
        .paused()
        .handle();
}
