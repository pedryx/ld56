use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    creature::{create_creature, CreatureStats, GenerateCreatureRng, PopulationSize},
    loading::TextureAssets,
    GameState, WINDOW_SIZE,
};

use super::new_creature_screen::PlayerCreature;

const CREATURE_Z: f32 = 1.0;
const CREATURE_SCALE: f32 = 0.5;

const ENEMY_CREATURE_TIER: u8 = 1;
const ENEMY_CREATURE_COUNT: u8 = 1;

pub struct BattleScreenPlugin;

impl Plugin for BattleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CreaturePositionRng(StdRng::from_entropy()))
            .add_systems(
                OnEnter(GameState::Battle),
                (
                    setup_player_creatures,
                    generate_enemy_creatures,
                    setup_enemy_creatures,
                )
                    .chain(),
            )
            .add_systems(OnExit(GameState::NewCreature), cleanup);
    }
}

#[derive(Component)]
struct BattleScreenItem;

#[derive(Component)]
struct BattleCreature {
    template: Entity,
    is_enemy: bool,
}

#[derive(Resource)]
struct CreaturePositionRng(StdRng);

fn create_population(
    commands: &mut Commands,
    rng: &mut StdRng,
    entity: Entity,
    texture: &Handle<Image>,
    stats: &CreatureStats,
    count: u32,
    is_enemy: bool,
) {
    for _ in 0..count {
        let mut position = Vec3::new(
            rng.gen_range(-WINDOW_SIZE.x / 2.0..-WINDOW_SIZE.x / 6.0),
            rng.gen_range(-WINDOW_SIZE.y / 2.0..WINDOW_SIZE.y / 2.0),
            CREATURE_Z,
        );

        if is_enemy {
            position.x *= -1.0;
        }

        commands
            .spawn((
                SpriteBundle {
                    transform: Transform::from_translation(position)
                        .with_scale(Vec2::splat(CREATURE_SCALE).extend(1.0)),
                    texture: texture.clone(),
                    sprite: Sprite {
                        flip_x: is_enemy,
                        ..default()
                    },
                    ..default()
                },
                BattleCreature {
                    template: entity,
                    is_enemy,
                },
                BattleScreenItem,
            ))
            .insert(stats.clone());
    }
}

fn setup_player_creatures(
    mut commands: Commands,
    mut creature_position_rng: ResMut<CreaturePositionRng>,
    mut query: Query<
        (
            Entity,
            &mut Visibility,
            &Handle<Image>,
            &PopulationSize,
            &CreatureStats,
        ),
        With<PlayerCreature>,
    >,
) {
    for (entity, mut visibility, texture, &PopulationSize(population_size), stats) in
        query.iter_mut()
    {
        *visibility = Visibility::Hidden;
        create_population(
            &mut commands,
            &mut creature_position_rng.0,
            entity,
            texture,
            stats,
            population_size,
            false,
        );
    }
}

fn generate_enemy_creatures(
    mut commands: Commands,
    mut generate_creature_rng: ResMut<GenerateCreatureRng>,
    textures: Res<TextureAssets>,
) {
    for _ in 0..ENEMY_CREATURE_COUNT {
        create_creature(
            &mut commands,
            &mut generate_creature_rng.0,
            &textures,
            ENEMY_CREATURE_TIER,
        );
    }
}

fn setup_enemy_creatures(
    mut commands: Commands,
    mut creature_position_rng: ResMut<CreaturePositionRng>,
    mut query: Query<
        (Entity, &Handle<Image>, &PopulationSize, &CreatureStats),
        Without<PlayerCreature>,
    >,
) {
    for (entity, texture, &PopulationSize(population_size), stats) in query.iter_mut() {
        create_population(
            &mut commands,
            &mut creature_position_rng.0,
            entity,
            texture,
            stats,
            population_size,
            true,
        );
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<BattleScreenItem>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
