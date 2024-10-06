use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal};

use crate::loading::TextureAssets;

const NUM_TIERS: u8 = 10;

const MIN_MOVEMENT_SPEED: f32 = 100.0;
const MAX_MOVEMENT_SPEED: f32 = 500.0;
const MIN_HP: f32 = 50.0;
const MAX_HP: f32 = 250.0;
const MIN_STAMINA: f32 = 50.0;
const MAX_STAMINA: f32 = 250.0;
const MIN_STAMINA_REGEN: f32 = 1.0;
const MAX_STAMINA_REGEN: f32 = 25.0;

const MIN_PHYS_DMG: f32 = 5.0;
const MAX_PHYS_DMG: f32 = 20.0;
const MIN_PHYS_STAMINA_COST: f32 = 10.0;
const MAX_PHYS_STAMINA_COST: f32 = 50.0;
const MIN_PHYS_COOLDOWN: f32 = 0.5;
const MAX_PHYS_COOLDOWN: f32 = 2.0;

const MIN_POPULATION: u32 = 5;
const MAX_POPULATION: u32 = 15;

const MUTATION_CHANCE: f64 = 0.25;

pub const CREATURE_Z: f32 = 10.0;

pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GenerateCreatureRng(StdRng::from_entropy()))
            .init_resource::<CreatureGeneration>()
            .add_systems(Update, delete_empty_creatures);
    }
}

#[derive(Resource, Default)]
pub struct CreatureGeneration(pub u64);

#[derive(Component)]
pub struct PopulationSize(pub u32);

#[derive(Component, Debug, Clone)]
pub struct CreatureStats {
    pub movement_speed: f32,
    pub hp: f32,
    pub stamina: f32,
    pub stamina_regen: f32,
    pub _generation: u64,
    pub physical_abilities: Vec<PhysicalAbility>,
}

impl CreatureStats {
    pub fn mutate(&mut self, rng: &mut StdRng) {
        self.movement_speed +=
            Self::get_mutation_change(rng, MIN_MOVEMENT_SPEED, MAX_MOVEMENT_SPEED);
        self.hp += Self::get_mutation_change(rng, MIN_HP, MAX_HP);
        self.stamina += Self::get_mutation_change(rng, MIN_STAMINA, MAX_STAMINA);
        self.stamina_regen += Self::get_mutation_change(rng, MIN_STAMINA_REGEN, MAX_STAMINA_REGEN);

        for ability in self.physical_abilities.iter_mut() {
            ability.damage += Self::get_mutation_change(rng, MIN_PHYS_DMG, MAX_PHYS_DMG);
            ability.stamina_cost +=
                Self::get_mutation_change(rng, MIN_PHYS_STAMINA_COST, MAX_PHYS_STAMINA_COST);
            ability.global_cooldown +=
                Self::get_mutation_change(rng, MIN_PHYS_COOLDOWN, MAX_PHYS_COOLDOWN);
        }
    }

    fn get_mutation_change(rng: &mut StdRng, min: f32, max: f32) -> f32 {
        if !rng.gen_bool(MUTATION_CHANCE) {
            return 0.0;
        }

        let range_width = max - min;
        let subrange_width = range_width / NUM_TIERS as f32;

        (rng.gen_range(-1.5..=1.5) * subrange_width).max(min)
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalAbility {
    pub _name: &'static str,
    pub stamina_cost: f32,
    pub damage: f32,
    pub global_cooldown: f32,
}

#[derive(Resource)]
pub struct GenerateCreatureRng(pub StdRng);

pub fn generate_creature(
    commands: &mut Commands,
    rng: &mut StdRng,
    textures: &Res<TextureAssets>,
    tier: u8,
    generation: u64,
    pop_multiplier: f32,
) -> Entity {
    let creature = CreatureStats {
        movement_speed: generate_stat_value(
            MIN_MOVEMENT_SPEED,
            MAX_MOVEMENT_SPEED,
            tier,
            rng,
            false,
        ),
        hp: generate_stat_value(MIN_HP, MAX_HP, tier, rng, false),
        stamina: generate_stat_value(MIN_STAMINA, MAX_STAMINA, tier, rng, false),
        stamina_regen: generate_stat_value(MIN_STAMINA_REGEN, MAX_STAMINA_REGEN, tier, rng, false),
        physical_abilities: vec![
            generate_physical_ability("Bite", tier, rng),
            generate_physical_ability("Punch", tier, rng),
            generate_physical_ability("Kick", tier, rng),
        ],
        _generation: generation,
    };

    let population = rng.gen_range(MIN_POPULATION..=MAX_POPULATION) as f32;
    let population = population * pop_multiplier;
    let population = PopulationSize(population as u32);

    commands
        .spawn(SpriteBundle {
            texture: textures.creature.clone(),
            visibility: Visibility::Hidden,
            transform: Transform::from_scale(Vec2::splat(1.4).extend(CREATURE_Z)),
            ..default()
        })
        .insert(creature)
        .insert(population)
        .id()
}

fn generate_stat_value(min: f32, max: f32, tier: u8, rng: &mut StdRng, inverse: bool) -> f32 {
    let range_width = max - min;
    let subrange_width = range_width / NUM_TIERS as f32;
    let subrange_center = min + (tier as f32 - 0.5) * subrange_width;
    let std_dev = subrange_width / 4.0;
    let normal_dist = Normal::new(subrange_center, std_dev).unwrap();

    let mut stat_value = normal_dist.sample(rng);
    stat_value = stat_value.clamp(min, max);

    if inverse {
        stat_value = max - (stat_value - min);
    }

    stat_value
}

fn generate_physical_ability(name: &'static str, tier: u8, rng: &mut StdRng) -> PhysicalAbility {
    PhysicalAbility {
        _name: name,
        stamina_cost: generate_stat_value(
            MIN_PHYS_STAMINA_COST,
            MAX_PHYS_STAMINA_COST,
            tier,
            rng,
            false,
        ),
        damage: generate_stat_value(MIN_PHYS_DMG, MAX_PHYS_DMG, tier, rng, false),
        global_cooldown: generate_stat_value(MIN_PHYS_COOLDOWN, MAX_PHYS_COOLDOWN, tier, rng, true),
    }
}

fn delete_empty_creatures(
    mut commands: Commands,
    query: Query<(Entity, &PopulationSize), Changed<PopulationSize>>,
) {
    for (entity, &PopulationSize(size)) in query.iter() {
        if size == 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
