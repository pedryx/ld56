#![allow(clippy::type_complexity)]

mod creature;
mod loading;
mod menu;
mod player;
mod screens;
mod ui;

use crate::creature::CreaturePlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;
use crate::screens::new_creature::NewCreatureScreenPlugin;
use crate::ui::UIPlugin;

use bevy::app::App;
//#[cfg(debug_assertions)]
//use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    Menu,
    NewCreature,
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
            PlayerPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            //app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }

        app.add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
