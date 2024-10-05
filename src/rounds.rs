use bevy::prelude::*;

pub struct RoundsPlugin;

impl Plugin for RoundsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Round>();
    }
}

#[derive(Resource)]
pub struct Round(pub u32);

impl Default for Round {
    fn default() -> Self {
        Self(1)
    }
}
