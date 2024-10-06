use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    creature::{
        generate_creature, CreatureGeneration, CreatureStats, GenerateCreatureRng, PopulationSize,
        CREATURE_Z,
    },
    loading::TextureAssets,
    rounds::Round,
    ui::{create_basic_button, create_change_state_button, create_mini_button},
    GameState, WINDOW_SIZE,
};

use super::new_creature_screen::{PlayerCreature, MAX_CREATURE_TIER, MIN_CREATURE_TIER};

const CREATURES_Z: f32 = 0.0;
const GRID_SIZE: Vec2 = Vec2::new(8.0, 3.0);
const COUNT_OFFSET: Vec2 = Vec2::new(0.0, 68.0);
const CREATURE_BUTTON_SIZE: Vec2 = Vec2::new(96.0, 96.0);

pub struct CreatureManagerScreenPlugin;

impl Plugin for CreatureManagerScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedCreaturesCounter>()
            .insert_resource(CombinationRng(StdRng::from_entropy()))
            .add_event::<CreatureCombinedEvent>()
            .add_event::<CombineButtonPressedEvent>()
            .add_systems(
                OnEnter(GameState::CreatureManager),
                (generate_new_creature, setup_ui).chain(),
            )
            .add_systems(OnExit(GameState::CreatureManager), cleanup)
            .add_systems(
                Update,
                (
                    handle_inc_dec_buttons,
                    handle_creature_button,
                    handle_combine_button,
                    combine_creatures,
                )
                    .run_if(in_state(GameState::CreatureManager)),
            )
            .add_systems(
                Update,
                (cleanup, setup_ui)
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
struct CreatureCombinedEvent;

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

fn generate_new_creature(
    mut commands: Commands,
    mut generate_creature_rng: ResMut<GenerateCreatureRng>,
    mut creature_generation: ResMut<CreatureGeneration>,
    textures: Res<TextureAssets>,
    round: Res<Round>,
) {
    let mut count = 1;

    if round.0 == 1 {
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
            1.0,
        );

        creature_generation.0 += 1;
        commands.entity(entity).insert(PlayerCreature);
    }
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
) {
    let mut x = 0;
    let mut y = 0;

    for (entity, mut visibility, mut transform, &PopulationSize(count), _) in query.iter_mut() {
        *visibility = Visibility::Visible;

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

        x += 1;
        if x >= GRID_SIZE.x as u32 {
            x = 0;
            y += 1;
            if y >= GRID_SIZE.y as u32 {
                break;
            }
        }
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
        WINDOW_SIZE * Vec2::new(0.11, 0.82),
    );
    commands
        .entity(button)
        .insert((CreatureManagerScreenItem, CombineButton));

    let button = create_mini_button(&mut commands, "+", WINDOW_SIZE * Vec2::new(0.26, 0.82));
    commands
        .entity(button)
        .insert((CreatureManagerScreenItem, IncButton));
    let button = create_mini_button(&mut commands, "-", WINDOW_SIZE * Vec2::new(0.315, 0.82));
    commands
        .entity(button)
        .insert((CreatureManagerScreenItem, DecButton));

    let mut pos = WINDOW_SIZE * Vec2::new(0.16, 0.92);
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
    creature_query: Query<&CreatureStats>,
    textures: Res<TextureAssets>,
    mut combination_rng: ResMut<CombinationRng>,
    mut creature_generation: ResMut<CreatureGeneration>,
    mut ew_creature_created: EventWriter<CreatureCombinedEvent>,
    mut er_combine_button_pressed: EventReader<CombineButtonPressedEvent>,
) {
    for event in er_combine_button_pressed.read() {
        let parent1 = creature_query.get(event.parent1).unwrap();
        let parent2 = creature_query.get(event.parent2).unwrap();

        let mut children = CreatureStats {
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
            _generation: creature_generation.0,
        };

        creature_generation.0 += 1;

        // There are always 3 physical abilities.
        for i in 0..3 {
            if combination_rng.0.gen_bool(0.5) {
                children
                    .physical_abilities
                    .push(parent1.physical_abilities[i].clone());
            } else {
                children
                    .physical_abilities
                    .push(parent2.physical_abilities[i].clone());
            }
        }

        commands
            .spawn((
                SpriteBundle {
                    texture: textures.creature.clone(),
                    visibility: Visibility::Hidden,
                    transform: Transform::from_scale(Vec2::splat(1.4).extend(CREATURE_Z)),
                    ..default()
                },
                // Child are born in pairs.
                PopulationSize(event.population * 2),
                PlayerCreature,
            ))
            .insert(children);

        ew_creature_created.send(CreatureCombinedEvent);
    }
}
