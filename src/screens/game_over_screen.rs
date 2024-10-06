use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};

use crate::rounds::GameEndedEvent;
use crate::statistics::GameStatistics;
use crate::GameResult;
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
        app.add_systems(
            OnEnter(GameState::GameOver(GameResult::Victory)),
            (setup, trigger_game_ended_event),
        )
        .add_systems(
            OnEnter(GameState::GameOver(GameResult::Defeat)),
            (setup, trigger_game_ended_event),
        )
        .add_systems(OnExit(GameState::GameOver(GameResult::Victory)), cleanup)
        .add_systems(OnExit(GameState::GameOver(GameResult::Defeat)), cleanup);
    }
}

#[derive(Component)]
pub struct GameOverScreenItem;

fn trigger_game_ended_event(mut ew_game_ended: EventWriter<GameEndedEvent>) {
    ew_game_ended.send(GameEndedEvent);
}

fn setup(
    mut commands: Commands,
    game_state: Res<State<GameState>>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    game_statistics: Res<GameStatistics>,
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

    if *game_state.get() == GameState::GameOver(GameResult::Defeat) {
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
    } else if *game_state.get() == GameState::GameOver(GameResult::Victory) {
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

    let text_style = TextStyle {
        font_size: 64.0,
        ..default()
    };

    let seconds = game_statistics.elapsed_seconds as usize % 60;
    let minutes = game_statistics.elapsed_seconds as usize / 60;
    let play_time = format!("{}:{:02}", minutes, seconds);    

    commands.spawn((
        Text2dBundle {
            text: Text::from_sections([
                TextSection::new("play time: ", text_style.clone()),
                TextSection::new(play_time, text_style.clone()),
                TextSection::new("\nsurvived rounds: ", text_style.clone()),
                TextSection::new(game_statistics.survived_rounds.to_string(), text_style.clone()),
                TextSection::new("\nenemies killed: ", text_style.clone()),
                TextSection::new(game_statistics.ally_kills.to_string(), text_style.clone()),
                TextSection::new("\nally died: ", text_style.clone()),
                TextSection::new(game_statistics.ally_deaths.to_string(), text_style.clone()),
                TextSection::new("\ncombination count: ", text_style.clone()),
                TextSection::new(game_statistics.combination_count.to_string(), text_style.clone()),
            ]),
            text_anchor: bevy::sprite::Anchor::Center,
            transform: Transform::from_translation(
                (WINDOW_SIZE * Vec2::new(0.0, -0.1)).extend(0.0),
            ),
            ..default()
        },
        GameOverScreenItem,
    ));

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
