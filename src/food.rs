use bevy::prelude::*;
use std::time::Duration;
use std::collections::HashSet;
use rand::Rng;
use crate::{
    player::{FOOD_BAR_MAX, Player, Stats},
    world::{HEIGHT, WIDTH, WORLD_TILE_SIZE},
};

const X_SPAWN_GENERATION: i32 = HEIGHT as i32 - 32;
const Y_SPAWN_GENERATION: i32 = WIDTH as i32 - 32;

const MAX_SPAWN_ATTEMPTS: i32 = 10;
const FOOD_PICKUP_RADIUS_TILES: i32 = 32;


#[derive(Component)]
pub struct Food;

#[derive(Component)]
pub struct FoodStats {
    pub food_bar_regen: f32,
}

#[derive(Component, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Location2D {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource)]
pub struct FoodTracker {
    food_spawn_location: HashSet<Location2D>,
    pub food_amount: i32,
}

impl FoodTracker {
    pub fn iter_locations(&self) -> impl Iterator<Item = &Location2D> {
        self.food_spawn_location.iter()
    }

    pub fn clear(&mut self) {
        self.food_spawn_location.clear();
        self.food_amount = 0;
    }
}

#[derive(Resource)]
pub struct FoodSpawnConfig {
    pub timer: Timer,
}


fn spawn_food(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut config: ResMut<FoodSpawnConfig>,
    mut food_stats: ResMut<FoodTracker>,
    player_query: Query<&Transform, With<Player>>,
) {

    let texture: Handle<Image> = asset_server.load("apple.png");

    config.timer.tick(time.delta());

    let food_spawn_flag = food_stats.food_amount < 5;

    if config.timer.is_finished() && food_spawn_flag {
        let Ok(player_transform) = player_query.single() else {
            return;
        };
        let player_tile_x =
            (player_transform.translation.x / WORLD_TILE_SIZE).floor() as i32;
        let player_tile_y =
            (player_transform.translation.y / WORLD_TILE_SIZE).floor() as i32;
        if let Some(location) =
            food_generate_location(food_stats.as_mut(), player_tile_x, player_tile_y)
        {
            let Location2D { x, y } = location;
            let world_x = x as f32 * WORLD_TILE_SIZE;
            let world_y = y as f32 * WORLD_TILE_SIZE;
            commands.spawn((
                Food,
                location,
                Sprite {
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..Sprite::from_image(texture)
                },
                Transform::from_translation(Vec3::new(world_x, world_y, 1.0)),
                FoodStats { food_bar_regen: 20.0 },
            ));
            food_stats.food_amount += 1;
        }
    }
}

fn setup_food_spawning(
    mut commands: Commands,
) {
    commands.insert_resource(FoodSpawnConfig {
        timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
    });
    commands.insert_resource(FoodTracker {
        food_spawn_location: HashSet::new(),
        food_amount: 0,
    });
}

fn food_generate_location(
    food_stats: &mut FoodTracker,
    player_x: i32,
    player_y: i32,
) -> Option<Location2D> {
    let mut rng = rand::rng();

    for _ in 0..MAX_SPAWN_ATTEMPTS {
        let x: i32 = rng.random_range(1..X_SPAWN_GENERATION);
        let y: i32 = rng.random_range(1..Y_SPAWN_GENERATION);
        if check_allowed_generation(&food_stats.food_spawn_location, player_x, player_y, x, y) {
            let location = Location2D { x, y };
            food_stats
                .food_spawn_location
                .insert(location);
            return Some(location);
        }
    }
    None
}

fn food_pickup(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut food_stats: ResMut<FoodTracker>,
    mut player_query: Query<(&Transform, &mut Stats), With<Player>>,
    food_query: Query<(Entity, &FoodStats, &Location2D), With<Food>>,
) {
    if !input.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok((player_transform, mut stats)) = player_query.single_mut() else {
        return;
    };
    let player_tile_x =
        (player_transform.translation.x / WORLD_TILE_SIZE).floor() as i32;
    let player_tile_y =
        (player_transform.translation.y / WORLD_TILE_SIZE).floor() as i32;

    let max_dist_sq = FOOD_PICKUP_RADIUS_TILES * FOOD_PICKUP_RADIUS_TILES;
    for (entity, food, location) in &food_query {
        let dx = location.x - player_tile_x;
        let dy = location.y - player_tile_y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq > 0 && dist_sq <= max_dist_sq {
            stats.food_bar =
                (stats.food_bar + food.food_bar_regen).min(FOOD_BAR_MAX);
            food_stats.food_amount = food_stats.food_amount.saturating_sub(1);
            food_stats.food_spawn_location.remove(location);
            commands.entity(entity).despawn();
        }
    }
}

fn check_allowed_generation(
    occupied: &HashSet<Location2D>,
    player_x: i32,
    player_y: i32,
    x: i32,
    y: i32,
) -> bool {
    let is_player_tile = player_x == x && player_y == y;
    let is_free = !occupied.contains(&Location2D{ x, y });
    is_free && !is_player_tile
}

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App){
        app.add_systems(Startup, setup_food_spawning)
            .add_systems(Update, (spawn_food, food_pickup));
    }
}
