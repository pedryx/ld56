use bevy::prelude::*;
use rand::Rng;

use crate::{
    creature::{create_creature, GenerateCreatureRng},
    loading::TextureAssets,
    ui::create_change_state_button,
    GameState, WINDOW_SIZE,
};

const MIN_CREATURE_TIER: u8 = 1;
const MAX_CREATURE_TIER: u8 = 3;

pub struct NewCreatureScreenPlugin;

#[derive(Component)]
pub struct PlayerCreature;

#[derive(Component)]
struct NewCreatureScreenItem;

impl Plugin for NewCreatureScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::NewCreature), (setup_ui, spawn_creature))
            .add_systems(OnExit(GameState::NewCreature), cleanup_ui);
    }
}

fn setup_ui(mut commands: Commands) {
    let button = create_change_state_button(
        &mut commands,
        "Continue",
        WINDOW_SIZE * Vec2::new(0.5, 0.9),
        GameState::Battle,
    );
    commands.entity(button).insert(NewCreatureScreenItem);
}

fn cleanup_ui(mut commands: Commands, query: Query<Entity, With<NewCreatureScreenItem>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_creature(
    mut commands: Commands,
    mut generate_creature_eng: ResMut<GenerateCreatureRng>,
    textures: Res<TextureAssets>,
) {
    let tier = generate_creature_eng
        .0
        .gen_range(MIN_CREATURE_TIER..=MAX_CREATURE_TIER);
    let entity = create_creature(&mut commands, &mut generate_creature_eng.0, textures, tier);

    commands.entity(entity).insert(PlayerCreature);
}
