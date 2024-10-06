use core::f32;
use std::f32::consts::PI;

use bevior_tree::{
    node::NodeResult,
    prelude::{delegate_node, ConditionalLoop, Sequence},
    task::{TaskBridge, TaskStatus},
    BehaviorTreeBundle, BehaviorTreePlugin,
};
use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    creature::{
        generate_creature, BodyPart, CreatureStats, GenerateCreatureRng, PhysicalAbility,
        PopulationSize,
    },
    loading::TextureAssets,
    rounds::{Difficulty, Round, RoundOverEvent},
    GameState, WINDOW_SIZE,
};

use super::{game_over_screen::GameResult, new_creature_screen::PlayerCreature};

const CREATURE_Z: f32 = 1.0;
const CREATURE_SCALE: f32 = 1.3;
const MELEE_DISTANCE: f32 = 32.0;
const DAMAGE_EFFECT_DURATION: f32 = 0.1;
const DAMAGE_EFFECT_Z: f32 = 40.0;
const BACKGROUND_Z: f32 = -20.0;
const BLOOD_PUDDLE_Z: f32 = -10.0;

pub struct BattleScreenPlugin;

impl Plugin for BattleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BehaviorTreePlugin::default())
            .insert_resource(CreaturePositionRng(StdRng::from_entropy()))
            .insert_resource(AttackRng(StdRng::from_entropy()))
            .insert_resource(BattleVisualsRng(StdRng::from_entropy()))
            .add_event::<DamageTakenEvent>()
            .add_event::<CreatureDieEvent>()
            .add_systems(
                OnEnter(GameState::Battle),
                (
                    (
                        setup_player_creatures,
                        generate_enemy_creatures,
                        setup_enemy_creatures,
                    )
                        .chain(),
                    setup_environment,
                ),
            )
            .add_systems(OnExit(GameState::Battle), cleanup)
            .add_systems(
                Update,
                (
                    find_nearest_enemy,
                    go_to_nearest_enemy,
                    stats_recovery,
                    attack_enemy,
                    handle_damage_effect,
                    death_system,
                    spawn_blood_puddle,
                    handle_battle_over,
                )
                    .chain()
                    .run_if(in_state(GameState::Battle)),
            );
    }
}

#[derive(Resource)]
struct BattleVisualsRng(StdRng);

#[derive(Event)]
struct DamageTakenEvent(Entity);

#[derive(Event)]
struct CreatureDieEvent {
    pos: Vec2,
}

#[derive(Component, Default)]
struct DamageEffect {
    elapsed: f32,
}

#[derive(Component)]
struct BattleScreenItem;

#[derive(Component)]
struct BattleCreature {
    template: Entity,
    movement_speed: f32,
    stamina_regen: f32,
    max_stamina: f32,
    physical_abilities: Vec<PhysicalAbility>,
}

#[derive(Component, Clone)]
struct BattleCreatureStats {
    hp: f32,
    stamina: f32,
    cooldown: f32,
}

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct BehaviorTreeContext {
    nearest_enemy_pos: Vec2,
    distance_squared_to_nearest_enemy: f32,
    nearest_enemy: Option<Entity>,
}

impl Default for BehaviorTreeContext {
    fn default() -> Self {
        Self {
            nearest_enemy_pos: Default::default(),
            distance_squared_to_nearest_enemy: f32::INFINITY,
            nearest_enemy: Default::default(),
        }
    }
}

#[derive(Resource)]
struct AttackRng(StdRng);

#[derive(Resource)]
struct CreaturePositionRng(StdRng);

fn setup_environment(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        SpriteBundle {
            texture: textures.battle_background.clone(),
            transform: Transform::from_xyz(0.0, 0.0, BACKGROUND_Z),
            ..default()
        },
        BattleScreenItem,
    ));
}

fn create_population(
    commands: &mut Commands,
    body_part_query: &Query<(&Sprite, &Handle<Image>, &Transform), With<BodyPart>>,
    rng: &mut StdRng,
    entity: Entity,
    components: (&CreatureStats, &Children),
    count: u32,
    is_enemy: bool,
) {
    let (stats, entity_children) = components;

    for _ in 0..count {
        let mut position = Vec3::new(
            rng.gen_range(-WINDOW_SIZE.x / 2.0..-WINDOW_SIZE.x / 6.0),
            rng.gen_range(-WINDOW_SIZE.y / 2.0..WINDOW_SIZE.y / 2.0),
            CREATURE_Z,
        );

        if is_enemy {
            position.x *= -1.0;
        }

        let mut entity = commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(position)
                    .with_scale(Vec2::splat(CREATURE_SCALE).extend(1.0)),
                sprite: Sprite {
                    flip_x: is_enemy,
                    ..default()
                },
                ..default()
            },
            BattleCreature {
                template: entity,
                movement_speed: stats.movement_speed,
                physical_abilities: stats.physical_abilities.clone(),
                max_stamina: stats.stamina,
                stamina_regen: stats.stamina_regen,
            },
            BattleCreatureStats {
                hp: stats.hp,
                stamina: stats.stamina,
                cooldown: 0.0,
            },
            BattleScreenItem,
            BehaviorTreeContext::default(),
            create_melee_behavior_tree(),
        ));
        entity.with_children(|children| {
            for &child in entity_children.iter() {
                let (sprite, texture, transform) = body_part_query.get(child).unwrap();

                let mut sprite = sprite.clone();
                sprite.flip_x = is_enemy;

                children.spawn((
                    SpriteBundle {
                        sprite,
                        texture: texture.clone(),
                        transform: *transform,
                        ..default()
                    },
                    BodyPart,
                ));
            }
        });

        if is_enemy {
            entity.insert(Enemy);
        }
    }
}

fn setup_player_creatures(
    mut commands: Commands,
    mut creature_position_rng: ResMut<CreaturePositionRng>,
    mut query: Query<
        (
            Entity,
            &mut Visibility,
            &PopulationSize,
            &CreatureStats,
            &Children,
        ),
        With<PlayerCreature>,
    >,
    body_part_query: Query<(&Sprite, &Handle<Image>, &Transform), With<BodyPart>>,
) {
    for (entity, mut visibility, &PopulationSize(population_size), stats, children) in
        query.iter_mut()
    {
        *visibility = Visibility::Hidden;
        create_population(
            &mut commands,
            &body_part_query,
            &mut creature_position_rng.0,
            entity,
            (stats, children),
            population_size,
            false,
        );
    }
}

fn generate_enemy_creatures(
    mut commands: Commands,
    mut generate_creature_rng: ResMut<GenerateCreatureRng>,
    textures: Res<TextureAssets>,
    mut difficulty: ResMut<Difficulty>,
) {
    for _ in 0..difficulty.enemy_count() {
        generate_creature(
            &mut commands,
            &mut generate_creature_rng.0,
            &textures,
            difficulty.enemy_tier(),
            0,
            difficulty.enemy_pop_mult(),
        );
    }
}

fn setup_enemy_creatures(
    mut commands: Commands,
    mut creature_position_rng: ResMut<CreaturePositionRng>,
    mut query: Query<(Entity, &PopulationSize, &CreatureStats, &Children), Without<PlayerCreature>>,
    body_part_query: Query<(&Sprite, &Handle<Image>, &Transform), With<BodyPart>>,
) {
    for (entity, &PopulationSize(population_size), stats, children) in query.iter_mut() {
        create_population(
            &mut commands,
            &body_part_query,
            &mut creature_position_rng.0,
            entity,
            (stats, children),
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

fn find_nearest_enemy(
    mut ally_query: Query<
        (Entity, &mut BehaviorTreeContext, &Transform),
        (With<BattleCreature>, Without<Enemy>),
    >,
    mut enemy_query: Query<
        (Entity, &mut BehaviorTreeContext, &Transform),
        (With<BattleCreature>, With<Enemy>),
    >,
) {
    for (_, mut context, ally_transform) in ally_query.iter_mut() {
        let position = ally_transform.translation.xy();
        context.distance_squared_to_nearest_enemy = f32::INFINITY;

        for (entity, _, enemy_transform) in enemy_query.iter() {
            let distance_squared = position.distance_squared(enemy_transform.translation.xy());

            if distance_squared < context.distance_squared_to_nearest_enemy {
                context.distance_squared_to_nearest_enemy = distance_squared;
                context.nearest_enemy_pos = enemy_transform.translation.xy();
                context.nearest_enemy = Some(entity);
            }
        }
    }

    for (_, mut context, enemy_transform) in enemy_query.iter_mut() {
        let position = enemy_transform.translation.xy();
        context.distance_squared_to_nearest_enemy = f32::INFINITY;

        for (entity, _, ally_transform) in ally_query.iter() {
            let distance_squared = position.distance_squared(ally_transform.translation.xy());

            if distance_squared < context.distance_squared_to_nearest_enemy {
                context.distance_squared_to_nearest_enemy = distance_squared;
                context.nearest_enemy_pos = ally_transform.translation.xy();
                context.nearest_enemy = Some(entity);
            }
        }
    }
}

fn create_melee_behavior_tree() -> BehaviorTreeBundle {
    BehaviorTreeBundle::from_root(ConditionalLoop::new(
        Sequence::new(vec![
            Box::new(GoToNearestEnemyTask::new()),
            Box::new(AttackEnemyTask::new()),
        ]),
        |In(_)| true,
    ))
}

#[delegate_node(delegate)]
struct GoToNearestEnemyTask {
    delegate: TaskBridge,
}

impl GoToNearestEnemyTask {
    pub fn new() -> Self {
        let checker = move |In(entity): In<Entity>, param: Query<&BehaviorTreeContext>| {
            let context = param.get(entity).unwrap();
            let distance_squared = context.distance_squared_to_nearest_enemy;

            match distance_squared <= MELEE_DISTANCE * MELEE_DISTANCE
                && param.get(context.nearest_enemy.unwrap()).is_ok()
            {
                true => TaskStatus::Complete(NodeResult::Success),
                false => TaskStatus::Running,
            }
        };
        let task = TaskBridge::new(checker).insert_while_running(GoToNearestEnemy);

        Self { delegate: task }
    }
}

#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct GoToNearestEnemy;

fn go_to_nearest_enemy(
    mut query: Query<
        (&mut Transform, &BehaviorTreeContext, &BattleCreature),
        With<GoToNearestEnemy>,
    >,
    entity_query: Query<Entity>,
    time: Res<Time>,
) {
    for (mut transform, context, creature) in query.iter_mut() {
        if entity_query.get(context.nearest_enemy.unwrap()).is_err() {
            continue;
        }

        let pos = transform.translation.xy();

        transform.translation += ((context.nearest_enemy_pos - pos).normalize_or_zero()
            * creature.movement_speed
            * time.delta_seconds())
        .extend(0.0);
    }
}

#[delegate_node(delegate)]
struct AttackEnemyTask {
    delegate: TaskBridge,
}

impl AttackEnemyTask {
    pub fn new() -> Self {
        let checker = move |In(entity): In<Entity>, param: Query<&BehaviorTreeContext>| {
            let context = param.get(entity);

            if context.is_err() {
                return TaskStatus::Complete(NodeResult::Success);
            }
            let context = context.unwrap();

            match context.distance_squared_to_nearest_enemy <= MELEE_DISTANCE * MELEE_DISTANCE
                && param.get(context.nearest_enemy.unwrap()).is_ok()
            {
                true => TaskStatus::Running,
                false => TaskStatus::Complete(NodeResult::Success),
            }
        };
        let task = TaskBridge::new(checker).insert_while_running(AttackEnemy);

        Self { delegate: task }
    }
}

#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct AttackEnemy;

fn attack_enemy(
    attacker_query: Query<(Entity, &BattleCreature, &BehaviorTreeContext), With<AttackEnemy>>,
    mut stats_query: Query<(Entity, &mut BattleCreatureStats)>,
    mut attack_rng: ResMut<AttackRng>,
    mut ew_damage_taken: EventWriter<DamageTakenEvent>,
) {
    for (entity, creature, context) in attacker_query.iter() {
        let (_, stats) = stats_query.get_mut(entity).unwrap();
        let mut stats = stats.clone();
        if stats.cooldown > 0.0 {
            continue;
        }

        let abilities = creature
            .physical_abilities
            .iter()
            .filter(|ability| ability.stamina_cost <= stats.stamina)
            .collect::<Vec<_>>();

        if abilities.is_empty() {
            continue;
        }

        let ability = abilities[attack_rng.0.gen_range(0..abilities.len())];
        stats.stamina -= ability.stamina_cost;
        stats.cooldown = ability.global_cooldown;

        if let Ok((entity, mut target_stats)) = stats_query.get_mut(context.nearest_enemy.unwrap())
        {
            target_stats.hp -= ability.damage;
            ew_damage_taken.send(DamageTakenEvent(entity));
        } else {
            continue;
        }

        *stats_query.get_mut(entity).unwrap().1 = stats;
    }
}

fn death_system(
    mut commands: Commands,
    hp_query: Query<(Entity, &BattleCreature, &BattleCreatureStats, &Transform)>,
    mut population_query: Query<&mut PopulationSize>,
    mut ew_creature_die: EventWriter<CreatureDieEvent>,
) {
    let mut entities_to_die = Vec::new();

    for (entity, creature, stats, transform) in hp_query.iter() {
        if stats.hp <= 0.0 {
            let mut population = population_query.get_mut(creature.template).unwrap();
            population.0 -= 1;

            entities_to_die.push(entity);
            ew_creature_die.send(CreatureDieEvent {
                pos: transform.translation.xy(),
            });
        }
    }

    for entity in entities_to_die {
        commands.entity(entity).despawn_recursive();
    }
}

fn stats_recovery(mut query: Query<(&mut BattleCreatureStats, &BattleCreature)>, time: Res<Time>) {
    for (mut stats, creature) in query.iter_mut() {
        stats.cooldown -= time.delta_seconds();
        if stats.cooldown < 0.0 {
            stats.cooldown = 0.0;
        }

        stats.stamina += creature.stamina_regen * time.delta_seconds();
        if stats.stamina > creature.max_stamina {
            stats.stamina = creature.max_stamina;
        }
    }
}

fn handle_battle_over(
    ally_query: Query<Entity, (With<BattleCreature>, Without<Enemy>)>,
    enemy_query: Query<Entity, (With<BattleCreature>, With<Enemy>)>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_game_result: ResMut<NextState<GameResult>>,
    mut difficulty: ResMut<Difficulty>,
    mut round: ResMut<Round>,
    mut ew_round_over: EventWriter<RoundOverEvent>,
) {
    if ally_query.is_empty() {
        next_game_result.set(GameResult::Lose);
        next_game_state.set(GameState::GameOver);
    } else if enemy_query.is_empty() {
        round.0 += 1;
        difficulty.inc_difficulty();
        next_game_state.set(GameState::CreatureManager);
        ew_round_over.send(RoundOverEvent);
    }

    // TODO: victory on last stage if no infinity mode
}

fn handle_damage_effect(
    mut commands: Commands,
    entity_query: Query<&Children>,
    enemy_query: Query<&Enemy>,
    mut effect_query: Query<(Entity, &mut DamageEffect)>,
    mut er_damage_taken: EventReader<DamageTakenEvent>,
    textures: Res<TextureAssets>,
    time: Res<Time>,
) {
    for &DamageTakenEvent(creature_entity) in er_damage_taken.read() {
        let children = entity_query
            .get(creature_entity)
            .unwrap()
            .iter()
            .find(|&&c| effect_query.get(c).is_ok());

        if let Some(effect_entity) = children {
            let (_, mut effect) = effect_query.get_mut(*effect_entity).unwrap();
            effect.elapsed = DAMAGE_EFFECT_DURATION;
        } else {
            commands.entity(creature_entity).with_children(|children| {
                children.spawn((
                    SpriteBundle {
                        texture: textures.damaged.clone(),
                        transform: Transform::from_xyz(0.0, 0.0, DAMAGE_EFFECT_Z),
                        sprite: Sprite {
                            flip_x: enemy_query.get(creature_entity).is_ok(),
                            ..default()
                        },
                        ..default()
                    },
                    DamageEffect {
                        elapsed: DAMAGE_EFFECT_DURATION,
                    },
                ));
            });
        }
    }

    for (creature_entity, mut effect) in effect_query.iter_mut() {
        effect.elapsed -= time.delta_seconds();
        if effect.elapsed <= 0.0 {
            commands.entity(creature_entity).despawn_recursive();
        }
    }
}

fn spawn_blood_puddle(
    mut commands: Commands,
    mut er_creature_die: EventReader<CreatureDieEvent>,
    textures: Res<TextureAssets>,
    mut battle_visuals_rng: ResMut<BattleVisualsRng>,
) {
    for event in er_creature_die.read() {
        let pos = battle_visuals_rng.0.gen_range(0..5) as f32 * Vec2::X * Vec2::splat(64.0);

        commands.spawn((
            SpriteBundle {
                texture: textures.blood.clone(),
                sprite: Sprite {
                    rect: Some(Rect::from_corners(pos, pos + Vec2::splat(64.0))),
                    ..default()
                },
                transform: Transform::from_translation(event.pos.extend(BLOOD_PUDDLE_Z))
                    .with_rotation(Quat::from_rotation_z(
                        battle_visuals_rng.0.gen_range(0.0..2.0 * PI),
                    )),
                ..default()
            },
            BattleScreenItem,
        ));
    }
}
