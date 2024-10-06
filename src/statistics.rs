use bevy::prelude::*;

use crate::{
    rounds::{GameEndedEvent, GameStartedEvent, RoundOverEvent},
    screens::{battle_screen::CreatureDieEvent, creature_manager_screen::CreatureCombinedEvent},
};

pub struct StatisticsPlugin;

impl Plugin for StatisticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameStatistics>()
            .add_systems(Update, update_statistics);
    }
}

#[derive(Resource, Default)]
pub struct GameStatistics {
    is_game_on: bool,
    pub elapsed_seconds: f32,
    pub survived_rounds: usize,
    pub ally_kills: usize,
    pub ally_deaths: usize,
    pub combination_count: usize,
}

fn update_statistics(
    mut game_statistics: ResMut<GameStatistics>,
    mut er_game_started: EventReader<GameStartedEvent>,
    mut er_game_ended: EventReader<GameEndedEvent>,
    mut er_round_over: EventReader<RoundOverEvent>,
    mut er_creature_die: EventReader<CreatureDieEvent>,
    mut er_creature_combined: EventReader<CreatureCombinedEvent>,
    time: Res<Time>,
) {
    for _ in er_game_started.read() {
        *game_statistics = GameStatistics::default();
        game_statistics.is_game_on = true;
    }

    for _ in er_game_ended.read() {
        game_statistics.is_game_on = false;
    }

    if !game_statistics.is_game_on {
        return;
    }

    game_statistics.elapsed_seconds += time.delta_seconds();

    for _ in er_round_over.read() {
        game_statistics.survived_rounds += 1;
    }

    for event in er_creature_die.read() {
        if event.is_enemy {
            game_statistics.ally_kills += 1;
        } else {
            game_statistics.ally_deaths += 1;
        }
    }

    for _ in er_creature_combined.read() {
        game_statistics.combination_count += 1;
    }
}
