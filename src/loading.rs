use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
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
    #[asset(path = "audio/attack.ogg")]
    pub attack: Handle<AudioSource>,
    #[asset(path = "audio/click.ogg")]
    pub click: Handle<AudioSource>,
    #[asset(path = "audio/combine.ogg")]
    pub combine: Handle<AudioSource>,
    #[asset(path = "audio/defeat.ogg")]
    pub defeat: Handle<AudioSource>,
    #[asset(path = "audio/die.ogg")]
    pub die: Handle<AudioSource>,
    #[asset(path = "audio/spell.ogg")]
    pub _spell: Handle<AudioSource>,
    #[asset(path = "audio/victory.ogg")]
    pub victory: Handle<AudioSource>,
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
    #[asset(path = "textures/creature_manager_background.png")]
    pub creature_manager_background: Handle<Image>,
    #[asset(path = "textures/menu_background.png")]
    pub menu_background: Handle<Image>,
    #[asset(path = "textures/round_holder.png")]
    pub round_holder: Handle<Image>,
    #[asset(path = "textures/creature_manager_tutorial.png")]
    pub creature_manager_tutorial: Handle<Image>,
}
