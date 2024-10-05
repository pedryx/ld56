#![allow(clippy::type_complexity)]

mod creature;
mod loading;
mod menu;
mod screens;
mod ui;

use crate::creature::CreaturePlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::screens::new_creature_screen::NewCreatureScreenPlugin;
use crate::ui::UIPlugin;

use bevy::app::App;
use bevy::prelude::*;
use screens::battle_screen::BattleScreenPlugin;
use screens::creature_manager_screen::CreatureManagerScreenPlugin;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    Menu,
    NewCreature,
    Battle,
    CreatureManager,
}

pub const WINDOW_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_plugins((
            LoadingPlugin,
            MenuPlugin,
            UIPlugin,
            NewCreatureScreenPlugin,
            CreaturePlugin,
            BattleScreenPlugin,
            CreatureManagerScreenPlugin,
        ));

        app.add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Srgba::hex("#136d15").unwrap().into()),
            ..default()
        },
        ..default()
    });
}
