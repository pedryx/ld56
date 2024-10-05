use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    creature::{Creature, PopulationSize},
    GameState, WINDOW_SIZE,
};

use super::new_creature_screen::PlayerCreature;

const CREATURE_Z: f32 = 1.0;
const CREATURE_SCALE: f32 = 0.5;

pub struct BattleScreenPlugin;

impl Plugin for BattleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CreaturePositionRng(StdRng::from_entropy()))
            .add_systems(OnEnter(GameState::Battle), setup_player_creatures)
            .add_systems(OnExit(GameState::NewCreature), cleanup);
    }
}

#[derive(Component)]
struct BattleScreenItem;

#[derive(Component)]
struct BattleCreature(Entity);

#[derive(Resource)]
struct CreaturePositionRng(StdRng);

fn setup_player_creatures(
    mut commands: Commands,
    mut creature_position_rng: ResMut<CreaturePositionRng>,
    mut query: Query<
        (
            Entity,
            &mut Visibility,
            &Handle<Image>,
            &PopulationSize,
            &Creature,
        ),
        With<PlayerCreature>,
    >,
) {
    for (entity, mut visibility, texture, &PopulationSize(population_size), creature) in
        query.iter_mut()
    {
        *visibility = Visibility::Hidden;

        for _ in 0..population_size {
            let position = Vec3::new(
                creature_position_rng
                    .0
                    .gen_range(-WINDOW_SIZE.x / 2.0..-WINDOW_SIZE.x / 6.0),
                creature_position_rng
                    .0
                    .gen_range(-WINDOW_SIZE.y / 2.0..WINDOW_SIZE.y / 2.0),
                CREATURE_Z,
            );

            commands
                .spawn((
                    SpriteBundle {
                        transform: Transform::from_translation(position)
                            .with_scale(Vec2::splat(CREATURE_SCALE).extend(1.0)),
                        texture: texture.clone(),
                        ..default()
                    },
                    BattleCreature(entity),
                    BattleScreenItem,
                ))
                .insert(creature.clone());
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<BattleScreenItem>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/*
spawn player creatures
generate enemy creatures
spawn enemy creatures
*/
