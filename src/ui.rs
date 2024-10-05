use bevy::prelude::*;

const BUTTON_SIZE: Vec2 = Vec2::new(160.0, 64.0);

#[derive(Component)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::linear_rgb(0.15, 0.15, 0.15),
            hovered: Color::linear_rgb(0.25, 0.25, 0.25),
        }
    }
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_button_hover);
    }
}

pub fn create_button(commands: &mut Commands, title: &'static str, pos: Vec2) -> Entity {
    let button_colors = ButtonColors::default();
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(BUTTON_SIZE.x),
                    height: Val::Px(BUTTON_SIZE.y),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::new(
                        Val::Px(pos.x - BUTTON_SIZE.x / 2.0),
                        Val::ZERO,
                        Val::Px(pos.y - BUTTON_SIZE.y / 2.0),
                        Val::ZERO,
                    ),
                    ..default()
                },
                background_color: button_colors.normal.into(),
                ..default()
            },
            button_colors,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                title,
                TextStyle {
                    font_size: 32.0,
                    color: Color::linear_rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        })
        .id()
}

fn handle_button_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonColors),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, button_colors) in &mut query {
        match *interaction {
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
            _ => {}
        }
    }
}
