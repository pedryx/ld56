use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCreatures>();
    }
}

#[derive(Resource, Default)]
pub struct PlayerCreatures(pub Vec<Entity>);
