use crate::loading::TextureAssets;
use crate::rounds::GameSettings;
use crate::{GameState, WINDOW_SIZE};
use bevy::prelude::*;

const BACKGROUND_Z: f32 = -20.0;
const TITLE_Z: f32 = 0.0;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(Update, click_play_button.run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

#[derive(Component)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::BLACK,
            hovered: Color::linear_rgb(0.25, 0.25, 0.25),
        }
    }
}

#[derive(Component)]
struct Menu;

#[derive(Component)]
struct InfinityModeButton;

fn setup_menu(mut commands: Commands, textures: Res<TextureAssets>) {
    // background
    commands.spawn((
        SpriteBundle {
            texture: textures.menu_background.clone(),
            transform: Transform::from_xyz(0.0, 0.0, BACKGROUND_Z),
            ..default()
        },
        Menu,
    ));

    // title
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Tiny Legion",
                TextStyle {
                    font_size: 128.0,
                    ..default()
                },
            ),
            transform: Transform::from_translation(
                (WINDOW_SIZE * Vec2::new(0.0, 0.3)).extend(TITLE_Z),
            ),
            ..default()
        },
        Menu,
    ));

    info!("menu");
    let mut entity = commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
        Menu,
    ));
    entity.with_children(|children| {
        let button_colors = ButtonColors::default();
        children
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(240.0),
                        height: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: button_colors.normal.into(),
                    ..Default::default()
                },
                button_colors,
                ChangeState(GameState::Tutorial),
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Normal Mode",
                    TextStyle {
                        font_size: 30.0,
                        color: Color::linear_rgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ));
            });
    });
    entity.with_children(|children| {
        let button_colors = ButtonColors::default();
        children
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(240.0),
                        height: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(10.0)),
                        ..Default::default()
                    },
                    background_color: button_colors.normal.into(),
                    transform: Transform::from_xyz(0.0, -80.0, 0.0),
                    ..Default::default()
                },
                button_colors,
                ChangeState(GameState::Tutorial),
                InfinityModeButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Infinity Mode",
                    TextStyle {
                        font_size: 30.0,
                        color: Color::linear_rgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ));
            });
    });
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    bottom: Val::Px(5.),
                    width: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            Menu,
        ))
        .with_children(|children| {
            children
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(170.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::SpaceAround,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(5.)),
                            ..Default::default()
                        },
                        background_color: Color::NONE.into(),
                        ..Default::default()
                    },
                    ButtonColors {
                        normal: Color::NONE,
                        ..default()
                    },
                    OpenLink("https://bevyengine.org"),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Made with Bevy",
                        TextStyle {
                            font_size: 15.0,
                            color: Color::linear_rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                    parent.spawn(ImageBundle {
                        image: textures.bevy.clone().into(),
                        style: Style {
                            width: Val::Px(32.),
                            ..default()
                        },
                        ..default()
                    });
                });
            children
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(170.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::SpaceAround,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(5.)),
                            ..default()
                        },
                        background_color: Color::NONE.into(),
                        ..Default::default()
                    },
                    ButtonColors {
                        normal: Color::NONE,
                        hovered: Color::linear_rgb(0.25, 0.25, 0.25),
                    },
                    OpenLink("https://github.com/pedryx/tiny-legion"),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Open source",
                        TextStyle {
                            font_size: 15.0,
                            color: Color::linear_rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                    parent.spawn(ImageBundle {
                        image: textures.github.clone().into(),
                        style: Style {
                            width: Val::Px(32.),
                            ..default()
                        },
                        ..default()
                    });
                });
        });
}

#[derive(Component)]
struct ChangeState(GameState);

#[derive(Component)]
struct OpenLink(&'static str);

fn click_play_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ButtonColors,
            Option<&ChangeState>,
            Option<&OpenLink>,
            Option<&InfinityModeButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_settings: ResMut<GameSettings>,
) {
    for (interaction, mut color, button_colors, change_state, open_link, infinity_mode_button) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                if let Some(state) = change_state {
                    game_settings.infinity_mode_on = infinity_mode_button.is_some();
                    next_state.set(state.0.clone());
                } else if let Some(link) = open_link {
                    if let Err(error) = webbrowser::open(link.0) {
                        warn!("Failed to open link {error:?}");
                    }
                }
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
