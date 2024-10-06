use bevy::prelude::*;

use crate::{loading::TextureAssets, GameState, WINDOW_SIZE};

pub struct TutorialScreenPlugin;

impl Plugin for TutorialScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tutorial>()
            .add_systems(OnEnter(GameState::Tutorial), setup)
            .add_systems(OnExit(GameState::Tutorial), cleanup)
            .add_systems(
                Update,
                handle_tutorial_end_button.run_if(in_state(GameState::Tutorial)),
            );
    }
}

#[derive(Resource, Default)]
struct Tutorial {
    shown: bool,
}

#[derive(Component)]
struct TutorialScreenItem;

#[derive(Component)]
struct EndTutorialButton;

fn setup(
    mut commands: Commands,
    mut tutorial: ResMut<Tutorial>,
    mut next_state: ResMut<NextState<GameState>>,
    textures: Res<TextureAssets>,
) {
    if tutorial.shown {
        next_state.set(GameState::CreatureManager);
        return;
    }
    tutorial.shown = true;

    commands.spawn((
        ButtonBundle {
            image: UiImage::new(textures.creature_manager_tutorial.clone()),
            style: Style {
                width: Val::Px(WINDOW_SIZE.x),
                height: Val::Px(WINDOW_SIZE.y),
                ..default()
            },
            ..default()
        },
        TutorialScreenItem,
        EndTutorialButton,
    ));
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<TutorialScreenItem>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_tutorial_end_button(
    query: Query<&Interaction, (With<EndTutorialButton>, Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for &interaction in query.iter() {
        if interaction == Interaction::Pressed {
            next_state.set(GameState::CreatureManager);
        }
    }
}
