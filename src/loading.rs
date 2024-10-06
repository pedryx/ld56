use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::CreatureManager)
                .load_collection::<AudioAssets>()
                .load_collection::<TextureAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/base_soundtrack.ogg")]
    pub base_soundtrack: Handle<AudioSource>,
    #[asset(path = "audio/battle_soundtrack.ogg")]
    pub battle_soundtrack: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
    #[asset(path = "textures/body_parts.png")]
    pub body_parts: Handle<Image>,
    #[asset(path = "textures/damaged.png")]
    pub damaged: Handle<Image>,
    #[asset(path = "textures/battle_background.png")]
    pub battle_background: Handle<Image>,
    #[asset(path = "textures/blood.png")]
    pub blood: Handle<Image>,
}
