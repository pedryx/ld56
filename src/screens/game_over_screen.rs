use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};

use crate::{
    audio::SOUND_EFFECTS_GLOBAL_VOLUME,
    creature::CreatureStats,
    loading::AudioAssets,
    rounds::{Difficulty, Round},
    ui::create_change_state_button,
    GameState, WINDOW_SIZE,
};

pub struct GameOverScreenPlugin;

impl Plugin for GameOverScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameResult>()
            .add_systems(OnEnter(GameState::GameOver), setup)
            .add_systems(OnExit(GameState::GameOver), cleanup);
    }
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameResult {
    Victory,
    #[default]
    Lose,
}

#[derive(Component)]
pub struct GameOverScreenItem;

fn setup(
    mut commands: Commands,
    game_result: Res<State<GameResult>>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Game Over",
                TextStyle {
                    font_size: 128.0,
                    color: Color::linear_rgb(1.0, 1.0, 1.0),
                    ..default()
                },
            ),
            text_anchor: bevy::sprite::Anchor::Center,
            transform: Transform::from_translation((WINDOW_SIZE * Vec2::new(0.0, 0.4)).extend(0.0)),
            ..default()
        },
        GameOverScreenItem,
    ));

    if *game_result.get() == GameResult::Lose {
        audio
            .play(audio_assets.defeat.clone())
            .with_volume(SOUND_EFFECTS_GLOBAL_VOLUME);

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    "Defeat",
                    TextStyle {
                        font_size: 64.0,
                        color: Color::linear_rgb(1.0, 0.0, 0.0),
                        ..default()
                    },
                ),
                text_anchor: bevy::sprite::Anchor::Center,
                transform: Transform::from_translation(
                    (WINDOW_SIZE * Vec2::new(0.0, 0.25)).extend(0.0),
                ),
                ..default()
            },
            GameOverScreenItem,
        ));
    } else if *game_result.get() == GameResult::Victory {
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    "Victory",
                    TextStyle {
                        font_size: 64.0,
                        color: Color::linear_rgb(1.0, 0.8, 0.0),
                        ..default()
                    },
                ),
                text_anchor: bevy::sprite::Anchor::Center,
                transform: Transform::from_translation(
                    (WINDOW_SIZE * Vec2::new(0.0, 0.25)).extend(0.0),
                ),
                ..default()
            },
            GameOverScreenItem,
        ));
    }

    let entity = create_change_state_button(
        &mut commands,
        "Main Menu",
        WINDOW_SIZE * Vec2::new(0.5, 0.9),
        GameState::Menu,
    );
    commands.entity(entity).insert(GameOverScreenItem);
}

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, With<GameOverScreenItem>>,
    creature_query: Query<Entity, With<CreatureStats>>,
    mut difficulty: ResMut<Difficulty>,
    mut round: ResMut<Round>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in creature_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    *difficulty = Difficulty::default();
    *round = Round::default();
}
