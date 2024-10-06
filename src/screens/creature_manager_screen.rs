use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
};
use bevy_kira_audio::{Audio, AudioControl};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    audio::SOUND_EFFECTS_GLOBAL_VOLUME,
    creature::{
        generate_creature, BodyPart, CreatureGeneration, CreatureStats, GenerateCreatureRng,
        PopulationChangedEvent, PopulationSize, CREATURE_SCALE, CREATURE_Z,
    },
    loading::{AudioAssets, TextureAssets},
    rounds::Round,
    ui::{create_basic_button, create_change_state_button, create_mini_button},
    GameState, WINDOW_SIZE,
};

use super::new_creature_screen::{PlayerCreature, MAX_CREATURE_TIER, MIN_CREATURE_TIER};

const CREATURES_Z: f32 = 0.0;
const GRID_SIZE: Vec2 = Vec2::new(8.0, 3.0);
const COUNT_OFFSET: Vec2 = Vec2::new(0.0, 68.0);
const CREATURE_BUTTON_SIZE: Vec2 = Vec2::new(96.0, 96.0);
const BACKGROUND_Z: f32 = -20.0;

pub struct CreatureManagerScreenPlugin;

impl Plugin for CreatureManagerScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedCreaturesCounter>()
            .insert_resource(CombinationRng(StdRng::from_entropy()))
            .add_event::<CreatureCombinedEvent>()
            .add_event::<CombineButtonPressedEvent>()
            .add_systems(
                OnEnter(GameState::CreatureManager),
                (
                    (generate_new_creature, (setup_ui, create_round_counter)).chain(),
                    setup_stats_windows,
                ),
            )
            .add_systems(OnExit(GameState::CreatureManager), cleanup)
            .add_systems(
                Update,
                (
                    handle_inc_dec_buttons,
                    handle_creature_button,
                    handle_combine_button,
                    combine_creatures,
                    show_stats,
                    play_combine_sound,
                    trigger_population_changed,
                )
                    .run_if(in_state(GameState::CreatureManager)),
            )
            .add_systems(
                Update,
                (partial_cleanup, (setup_ui, create_round_counter))
                    .chain()
                    .run_if(on_event::<CreatureCombinedEvent>()),
            );
    }
}

#[derive(Event)]
struct CombineButtonPressedEvent {
    parent1: Entity,
    parent2: Entity,
    population: u32,
}

#[derive(Event)]
pub struct CreatureCombinedEvent;

#[derive(Resource)]
struct CombinationRng(StdRng);

#[derive(Resource, Default)]
struct SelectedCreaturesCounter(u8);

#[derive(Component)]
struct CreatureManagerScreenItem;

#[derive(Component)]
struct CombineButton;

#[derive(Component)]
struct IncButton;

#[derive(Component)]
struct DecButton;

#[derive(Component)]
struct PopulationText;

#[derive(Component)]
struct CreatureButton {
    entity: Entity,
    selected: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum StatLabel {
    MovementSpeed,
    HP,
    Stamina,
    StaminaRegen,
    PhysicalAbility,
}

#[derive(Component)]
struct StatWindow;

fn create_round_counter(mut commands: Commands, textures: Res<TextureAssets>, round: Res<Round>) {
    commands
        .spawn((
            SpriteBundle {
                texture: textures.round_holder.clone(),
                transform: Transform::from_translation(
                    (WINDOW_SIZE * Vec2::new(0.0, -0.4)).extend(0.0),
                )
                .with_scale(Vec2::splat(0.4).extend(1.0)),
                ..default()
            },
            CreatureManagerScreenItem,
        ))
        .with_children(|children| {
            children.spawn(Text2dBundle {
                text: Text::from_section(
                    round.0.to_string(),
                    TextStyle {
                        font_size: 96.0,
                        ..default()
                    },
                ),
                ..default()
            });
        });
}

fn generate_new_creature(
    mut commands: Commands,
    mut generate_creature_rng: ResMut<GenerateCreatureRng>,
    mut creature_generation: ResMut<CreatureGeneration>,
    textures: Res<TextureAssets>,
    round: Res<Round>,
) {
    let mut count = 0;
    if round.0 == 1 {
        count += 1;
    }
    if round.0 % 2 == 1 {
        count += 1;
    }

    for _ in 0..count {
        let tier = generate_creature_rng
            .0
            .gen_range(MIN_CREATURE_TIER..=MAX_CREATURE_TIER);
        let entity = generate_creature(
            &mut commands,
            &mut generate_creature_rng.0,
            &textures,
            tier,
            creature_generation.0,
            1.2,
        );

        creature_generation.0 += 1;
        commands.entity(entity).insert(PlayerCreature);
    }
}

const STATS_X: f32 = 0.37;
const STATS_Y1: f32 = 0.31;
const STATS_Y2: f32 = -0.05;
const STATS_SIZE: Vec2 = Vec2::new(240.0, 240.0);
const STAT_FONT_SIZE: f32 = 18.0;
const STAT_LABEL_X: f32 = -STATS_SIZE.x / 2.0 + 10.0;
const STAT_LABEL_Z: f32 = 20.0;
const ABILITIES_FONT_SIZE: f32 = 16.0;

fn setup_stats_windows(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let color_material_handle = materials.add(Color::linear_rgba(0.16, 0.16, 0.16, 0.0));

    let stats_text_style = TextStyle {
        font_size: STAT_FONT_SIZE,
        ..default()
    };
    let abilities_text_style = TextStyle {
        font_size: ABILITIES_FONT_SIZE,
        ..default()
    };

    commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(STATS_SIZE.x, STATS_SIZE.y))),
                material: color_material_handle.clone(),
                transform: Transform::from_translation(
                    (WINDOW_SIZE * Vec2::new(STATS_X, STATS_Y1)).extend(0.0),
                ),
                ..default()
            },
            CreatureManagerScreenItem,
            StatWindow,
        ))
        .with_children(|children| {
            let labels = [
                ("Movement Speed: ", StatLabel::MovementSpeed),
                ("HP: ", StatLabel::HP),
                ("Stamina: ", StatLabel::Stamina),
                ("Stamina Regen: ", StatLabel::StaminaRegen),
            ];
            let value_label = "00.00";

            for (i, (label, label_type)) in labels.iter().enumerate() {
                children
                    .spawn((Text2dBundle {
                        text: Text {
                            sections: vec![
                                TextSection {
                                    value: label.to_string(),
                                    style: stats_text_style.clone(),
                                },
                                TextSection {
                                    value: value_label.to_string(),
                                    style: stats_text_style.clone(),
                                },
                            ],
                            ..default()
                        },
                        text_anchor: bevy::sprite::Anchor::CenterLeft,
                        transform: Transform::from_xyz(
                            STAT_LABEL_X,
                            STATS_SIZE.y / 2.0 - (i * 2 + 1) as f32 * STAT_FONT_SIZE,
                            STAT_LABEL_Z,
                        ),
                        ..default()
                    },))
                    .insert(*label_type);
            }
        });

    commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(STATS_SIZE.x, STATS_SIZE.y))),
                material: color_material_handle.clone(),
                transform: Transform::from_translation(
                    (WINDOW_SIZE * Vec2::new(STATS_X, STATS_Y2)).extend(0.0),
                ),
                ..default()
            },
            CreatureManagerScreenItem,
            StatWindow,
        ))
        .with_children(|children| {
            // There are alway 3 phys abilities.
            for i in 0..3 {
                let text_sections = vec![
                    TextSection {
                        value: "Name:".to_string(),
                        style: abilities_text_style.clone(),
                    },
                    TextSection {
                        value: "\n- Damage: ".to_string(),
                        style: abilities_text_style.clone(),
                    },
                    TextSection {
                        value: "00.00".to_string(),
                        style: abilities_text_style.clone(),
                    },
                    TextSection {
                        value: "\n- Stamina Cost: ".to_string(),
                        style: abilities_text_style.clone(),
                    },
                    TextSection {
                        value: "00.00".to_string(),
                        style: abilities_text_style.clone(),
                    },
                    TextSection {
                        value: "\n- Global Cooldown: ".to_string(),
                        style: abilities_text_style.clone(),
                    },
                    TextSection {
                        value: "00.00".to_string(),
                        style: abilities_text_style.clone(),
                    },
                ];

                children
                    .spawn((Text2dBundle {
                        text: Text {
                            sections: text_sections,
                            ..default()
                        },
                        text_anchor: bevy::sprite::Anchor::CenterLeft,
                        transform: Transform::from_xyz(
                            STAT_LABEL_X,
                            STATS_SIZE.y / 2.0 - (((i * 5) as f32) + 2.5) * ABILITIES_FONT_SIZE,
                            STAT_LABEL_Z,
                        ),
                        ..default()
                    },))
                    .insert(StatLabel::PhysicalAbility);
            }
        });
}

fn setup_ui(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Visibility,
            &mut Transform,
            &PopulationSize,
            &CreatureStats,
        ),
        With<PlayerCreature>,
    >,
    textures: Res<TextureAssets>,
) {
    // background
    commands.spawn((
        SpriteBundle {
            texture: textures.creature_manager_background.clone(),
            transform: Transform::from_xyz(0.0, 0.0, BACKGROUND_Z),
            ..default()
        },
        CreatureManagerScreenItem,
    ));

    let mut generations = query
        .iter()
        .filter(|(_, _, _, &PopulationSize(size), _)| size > 0)
        .map(|(_, _, _, _, stats)| stats.generation)
        .collect::<Vec<_>>();
    generations.sort();
    let order = generations
        .iter()
        .enumerate()
        .map(|(i, g)| (g, i))
        .collect::<HashMap<_, _>>();

    for (entity, mut visibility, mut transform, &PopulationSize(count), stats) in query.iter_mut() {
        if count == 0 {
            continue;
        }

        *visibility = Visibility::Visible;

        let i = order[&stats.generation];
        let x = i % (GRID_SIZE.x as usize - 2);
        let y = i / (GRID_SIZE.x as usize - 2);

        let grid_pos = Vec2::new(x as f32 + 0.5, y as f32 + 0.4);
        let cell_size = Vec2::new(WINDOW_SIZE.x, WINDOW_SIZE.y * 3.0 / 4.0) / GRID_SIZE;
        let pos_no_transform = cell_size * grid_pos;
        let mut pos = pos_no_transform;
        pos.y = WINDOW_SIZE.y - pos.y;
        pos -= WINDOW_SIZE / 2.0;
        transform.translation = pos.extend(CREATURES_Z);

        commands.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(CREATURE_BUTTON_SIZE.x),
                    height: Val::Px(CREATURE_BUTTON_SIZE.y),
                    position_type: PositionType::Absolute,
                    left: Val::Px(pos_no_transform.x - CREATURE_BUTTON_SIZE.x / 2.0),
                    top: Val::Px(pos_no_transform.y - CREATURE_BUTTON_SIZE.y / 2.0),
                    border: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                background_color: Color::NONE.into(),
                border_radius: BorderRadius::all(Val::Px(9999.0)),
                border_color: Color::NONE.into(),
                ..default()
            },
            CreatureManagerScreenItem,
            CreatureButton {
                entity,
                selected: false,
            },
        ));

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    count.to_string() + "x",
                    TextStyle {
                        font_size: 32.0,
                        color: Color::BLACK,
                        ..default()
                    },
                ),
                text_anchor: bevy::sprite::Anchor::Center,
                transform: Transform::from_translation((pos - COUNT_OFFSET).extend(0.0)),
                ..default()
            },
            CreatureManagerScreenItem,
        ));
    }

    let button = create_change_state_button(
        &mut commands,
        "Continue",
        WINDOW_SIZE * Vec2::new(0.9, 0.86),
        GameState::Battle,
    );
    commands.entity(button).insert(CreatureManagerScreenItem);

    let button = create_basic_button(
        &mut commands,
        "Combine",
        WINDOW_SIZE * Vec2::new(0.11, 0.85),
    );
    commands
        .entity(button)
        .insert((CreatureManagerScreenItem, CombineButton));

    let button = create_mini_button(&mut commands, "+", WINDOW_SIZE * Vec2::new(0.26, 0.85));
    commands
        .entity(button)
        .insert((CreatureManagerScreenItem, IncButton));
    let button = create_mini_button(&mut commands, "-", WINDOW_SIZE * Vec2::new(0.315, 0.85));
    commands
        .entity(button)
        .insert((CreatureManagerScreenItem, DecButton));

    let mut pos = WINDOW_SIZE * Vec2::new(0.16, 0.95);
    pos.y = WINDOW_SIZE.y - pos.y;
    pos -= WINDOW_SIZE / 2.0;
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "selected population:",
                TextStyle {
                    font_size: 32.0,
                    color: Color::BLACK,
                    ..default()
                },
            ),
            text_anchor: bevy::sprite::Anchor::Center,

            transform: Transform::from_translation(pos.extend(0.0)),
            ..default()
        },
        CreatureManagerScreenItem,
    ));

    pos.x += WINDOW_SIZE.x * 0.135;
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "1",
                TextStyle {
                    font_size: 32.0,
                    color: Color::BLACK,
                    ..default()
                },
            ),
            text_anchor: bevy::sprite::Anchor::CenterLeft,

            transform: Transform::from_translation(pos.extend(0.0)),
            ..default()
        },
        CreatureManagerScreenItem,
        PopulationText,
    ));
}

fn partial_cleanup(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            With<CreatureManagerScreenItem>,
            Without<StatWindow>,
            Without<StatLabel>,
        ),
    >,
    mut selected_creatures_counter: ResMut<SelectedCreaturesCounter>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    selected_creatures_counter.0 = 0;
}

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, With<CreatureManagerScreenItem>>,
    mut selected_creatures_counter: ResMut<SelectedCreaturesCounter>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    selected_creatures_counter.0 = 0;
}

fn handle_inc_dec_buttons(
    inc_button_query: Query<&Interaction, (With<IncButton>, Changed<Interaction>)>,
    dec_button_query: Query<&Interaction, (With<DecButton>, Changed<Interaction>)>,
    mut population_text_query: Query<&mut Text, With<PopulationText>>,
) {
    let mut change: i32 = 0;

    if !inc_button_query.is_empty() && *inc_button_query.single() == Interaction::Pressed {
        change += 1;
    }
    if !dec_button_query.is_empty() && *dec_button_query.single() == Interaction::Pressed {
        change -= 1;
    }

    let text = &mut population_text_query.single_mut().sections[0].value;

    let mut value = text.parse::<i32>().unwrap();
    value += change;
    if value < 1 {
        value = 1;
    }

    *text = value.to_string();
}

fn handle_creature_button(
    mut creature_button_query: Query<
        (&Interaction, &mut BorderColor, &mut CreatureButton),
        (With<CreatureButton>, Changed<Interaction>),
    >,
    mut selected_creatures_counter: ResMut<SelectedCreaturesCounter>,
) {
    for (interaction, mut border_color, mut creature_button) in creature_button_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if !creature_button.selected && selected_creatures_counter.0 >= 2 {
            continue;
        }

        creature_button.selected = !creature_button.selected;
        *border_color = if creature_button.selected {
            selected_creatures_counter.0 += 1;
            Color::WHITE.with_alpha(0.2).into()
        } else {
            selected_creatures_counter.0 -= 1;
            Color::NONE.into()
        };
    }
}

fn handle_combine_button(
    combine_button_query: Query<&Interaction, (With<CombineButton>, Changed<Interaction>)>,
    creature_buttons_query: Query<&CreatureButton>,
    population_text_query: Query<&Text, With<PopulationText>>,
    mut creature_query: Query<&mut PopulationSize>,
    mut ew_combine_button_pressed: EventWriter<CombineButtonPressedEvent>,
) {
    if combine_button_query.is_empty() || *combine_button_query.single() != Interaction::Pressed {
        return;
    }

    let population = population_text_query.single().sections[0]
        .value
        .parse::<u32>()
        .unwrap();
    let entities = creature_buttons_query
        .iter()
        .filter(|button| button.selected)
        .map(|button| button.entity)
        .collect::<Vec<_>>();

    if entities.len() != 2 {
        return;
    }

    let mut population1 = creature_query.get_mut(entities[0]).unwrap();
    if population1.0 < population {
        return;
    } else {
        population1.0 -= population;
    }
    let mut population2 = creature_query.get_mut(entities[1]).unwrap();
    if population2.0 < population {
        return;
    } else {
        population2.0 -= population;
    }

    ew_combine_button_pressed.send(CombineButtonPressedEvent {
        parent1: entities[0],
        parent2: entities[1],
        population,
    });
}

fn combine_creatures(
    mut commands: Commands,
    creature_query: Query<(&CreatureStats, &Children)>,
    sprite_query: Query<(&Sprite, &Handle<Image>, &Transform), With<BodyPart>>,
    mut combination_rng: ResMut<CombinationRng>,
    mut creature_generation: ResMut<CreatureGeneration>,
    mut ew_creature_created: EventWriter<CreatureCombinedEvent>,
    mut er_combine_button_pressed: EventReader<CombineButtonPressedEvent>,
) {
    for event in er_combine_button_pressed.read() {
        let (parent1, children1) = creature_query.get(event.parent1).unwrap();
        let (parent2, children2) = creature_query.get(event.parent2).unwrap();

        let mut children_stats = CreatureStats {
            hp: if combination_rng.0.gen_bool(0.5) {
                parent1.hp
            } else {
                parent2.hp
            },
            movement_speed: if combination_rng.0.gen_bool(0.5) {
                parent1.movement_speed
            } else {
                parent2.movement_speed
            },
            stamina: if combination_rng.0.gen_bool(0.5) {
                parent1.stamina
            } else {
                parent2.stamina
            },
            stamina_regen: if combination_rng.0.gen_bool(0.5) {
                parent1.stamina_regen
            } else {
                parent2.stamina_regen
            },
            physical_abilities: Vec::new(),
            generation: creature_generation.0,
        };
        creature_generation.0 += 1;
        // There are always 3 physical abilities.
        for i in 0..3 {
            if combination_rng.0.gen_bool(0.5) {
                children_stats
                    .physical_abilities
                    .push(parent1.physical_abilities[i].clone());
            } else {
                children_stats
                    .physical_abilities
                    .push(parent2.physical_abilities[i].clone());
            }
        }
        children_stats.mutate(&mut combination_rng.0);

        let mut entity = commands.spawn((
            SpriteBundle {
                visibility: Visibility::Hidden,
                transform: Transform::from_scale(Vec2::splat(CREATURE_SCALE).extend(CREATURE_Z)),
                ..default()
            },
            // Child are born in pairs.
            PopulationSize(event.population * 2),
            PlayerCreature,
        ));
        entity.insert(children_stats);

        let children = children1.iter().zip(children2).map(|(child1, child2)| {
            if combination_rng.0.gen_bool(0.5) {
                child1
            } else {
                child2
            }
        });
        for &child in children {
            let (sprite, texture, transform) = sprite_query.get(child).unwrap();

            entity.with_children(|children| {
                children.spawn((
                    SpriteBundle {
                        sprite: sprite.clone(),
                        texture: texture.clone(),
                        transform: *transform,
                        ..default()
                    },
                    BodyPart,
                ));
            });
        }

        ew_creature_created.send(CreatureCombinedEvent);
    }
}

fn show_stats(
    creature_button_query: Query<(&Interaction, &CreatureButton)>,
    mut stat_window_query: Query<&mut Visibility, With<StatWindow>>,
    mut stat_label_query: Query<(&mut Text, &StatLabel)>,
    creature_query: Query<&CreatureStats, With<PlayerCreature>>,
) {
    let mut hovered_creature = None;
    for (interaction, creature_button) in creature_button_query.iter() {
        if *interaction == Interaction::Hovered {
            hovered_creature = Some(creature_button.entity);
        }
    }

    for mut visibility in stat_window_query.iter_mut() {
        *visibility = if hovered_creature.is_none() {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

    if hovered_creature.is_none() {
        return;
    }
    let stats = creature_query.get(hovered_creature.unwrap()).unwrap();

    let (mut text, _) = stat_label_query
        .iter_mut()
        .find(|&(_, &label)| label == StatLabel::HP)
        .unwrap();
    text.sections[1].value = format!("{:.2}", stats.hp);
    let (mut text, _) = stat_label_query
        .iter_mut()
        .find(|&(_, &label)| label == StatLabel::MovementSpeed)
        .unwrap();
    text.sections[1].value = format!("{:.2}", stats.movement_speed);
    let (mut text, _) = stat_label_query
        .iter_mut()
        .find(|&(_, &label)| label == StatLabel::Stamina)
        .unwrap();
    text.sections[1].value = format!("{:.2}", stats.stamina);
    let (mut text, _) = stat_label_query
        .iter_mut()
        .find(|&(_, &label)| label == StatLabel::StaminaRegen)
        .unwrap();
    text.sections[1].value = format!("{:.2}", stats.stamina_regen);

    let phys_ability_texts = stat_label_query
        .iter_mut()
        .filter(|&(_, &label)| label == StatLabel::PhysicalAbility)
        .map(|(text, _)| text);
    for (ability_index, mut text) in phys_ability_texts.enumerate() {
        assert!(ability_index < 3);

        text.sections[0].value = stats.physical_abilities[ability_index].name.to_string();
        text.sections[2].value = format!("{:.2}", stats.physical_abilities[ability_index].damage);
        text.sections[4].value = format!(
            "{:.2}",
            stats.physical_abilities[ability_index].stamina_cost
        );
        text.sections[6].value = format!(
            "{:.2}",
            stats.physical_abilities[ability_index].global_cooldown
        );
    }
}

fn play_combine_sound(
    mut er_creatures_combined: EventReader<CreatureCombinedEvent>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    for _ in er_creatures_combined.read() {
        audio
            .play(audio_assets.combine.clone())
            .with_volume(SOUND_EFFECTS_GLOBAL_VOLUME);
    }
}

fn trigger_population_changed(
    mut er_creature_combined: EventReader<CreatureCombinedEvent>,
    mut ew_population_changed: EventWriter<PopulationChangedEvent>,
) {
    for _ in er_creature_combined.read() {
        ew_population_changed.send(PopulationChangedEvent);
    }
}
