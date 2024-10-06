use bevy::prelude::*;
use rand::{rngs::StdRng, SeedableRng};
use rand_distr::{Distribution, Normal};

const MIN_ENEMY_TIER: f32 = 1.0;
const MIN_ENEMY_COUNT: f32 = 1.0;
const MIN_ENEMY_POP_MULT: f32 = 0.5;

const ENEMY_TIER_INC: f32 = 0.5;
const ENEMY_COUNT_INC: f32 = 0.5;
const ENEMY_POP_INC: f32 = 0.25;

const ENEMY_TIER_STD_DEV: f32 = 1.0;
const ENEMY_COUNT_STD_DEV: f32 = 0.5;
const ENEMY_POP_MULT_STD_DEV: f32 = 0.3;

pub struct RoundsPlugin;

impl Plugin for RoundsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Round>()
            .init_resource::<Difficulty>()
            .add_event::<RoundOverEvent>();
    }
}

#[derive(Event)]
pub struct RoundOverEvent;

#[derive(Resource)]
pub struct Round(pub u32);

impl Default for Round {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Resource)]
pub struct Difficulty {
    enemy_tier: f32,
    enemy_count: f32,
    enemy_pop_mult: f32,
    rng: StdRng,
}

impl Difficulty {
    pub fn inc_difficulty(&mut self) {
        self.enemy_tier += ENEMY_TIER_INC;
        self.enemy_count += ENEMY_COUNT_INC;
        self.enemy_pop_mult += ENEMY_POP_INC;
    }

    pub fn enemy_tier(&mut self) -> u8 {
        self.gen_value(self.enemy_tier, ENEMY_TIER_STD_DEV, MIN_ENEMY_TIER) as u8
    }

    pub fn enemy_count(&mut self) -> u8 {
        self.gen_value(self.enemy_count, ENEMY_COUNT_STD_DEV, MIN_ENEMY_COUNT) as u8
    }

    pub fn enemy_pop_mult(&mut self) -> f32 {
        self.gen_value(
            self.enemy_pop_mult,
            ENEMY_POP_MULT_STD_DEV,
            MIN_ENEMY_POP_MULT,
        )
    }

    fn gen_value(&mut self, mean: f32, std_dev: f32, min: f32) -> f32 {
        let normal_dist = Normal::new(mean, std_dev).unwrap();
        let result = normal_dist.sample(&mut self.rng);

        result.max(min)
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        Self {
            enemy_tier: 1.0,
            enemy_count: 1.0,
            enemy_pop_mult: 1.0,
            rng: StdRng::from_entropy(),
        }
    }
}
