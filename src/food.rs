use bevy::prelude::*;
use std::time::Duration;
use std::collections::HashSet;
use rand::Rng;
use crate::{
    player::{Player, Stats},
    world::{HEIGHT, WIDTH, WORLD_TILE_SIZE},
};

const X_SPAWN_GENERATION: i32 = HEIGHT as i32 - 32;
const Y_SPAWN_GENERATION: i32 = WIDTH as i32 - 32;

const MAX_SPAWN_ATTEMPTS: i32 = 10;
const FOOD_BAR_MAX: f32 = 100.0;
const BAR_WIDTH: f32 = 200.0;
const BAR_HEIGHT: f32 = 14.0;
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
                FoodStats { food_bar_regen: 10.0 },
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

fn setup_food_ui(
    mut commands: Commands,
) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(16.0),
            top: px(16.0),
            width: px(BAR_WIDTH),
            height: px(BAR_HEIGHT),
            border: UiRect::all(px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
        BorderColor::all(Color::srgb(0.45, 0.45, 0.45)),
        children![(
            Node {
                width: percent(100.0),
                height: percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.9, 0.25)),
            FoodBarFill,
        )],
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(16.0),
            top: px(36.0),
            width: px(BAR_WIDTH),
            height: px(BAR_HEIGHT),
            border: UiRect::all(px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
        BorderColor::all(Color::srgb(0.45, 0.45, 0.45)),
        children![(
            Node {
                width: percent(100.0),
                height: percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.9, 0.2, 0.2)),
            HealthBarFill,
        )],
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(16.0),
            top: px(56.0),
            width: px(BAR_WIDTH),
            height: px(BAR_HEIGHT),
            border: UiRect::all(px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
        BorderColor::all(Color::srgb(0.45, 0.45, 0.45)),
        children![(
            Node {
                width: percent(100.0),
                height: percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.6, 0.95)),
            StaminaBarFill,
        )],
    ));
}

fn update_food_ui(
    player_query: Query<&Stats, With<Player>>,
    mut bar_queries: ParamSet<(
        Query<&mut Node, With<FoodBarFill>>,
        Query<&mut Node, With<HealthBarFill>>,
        Query<&mut Node, With<StaminaBarFill>>,
    )>,
) {
    let Ok(stats) = player_query.single() else {
        return;
    };
    if let Ok(mut node) = bar_queries.p0().single_mut() {
        let pct = (stats.food_bar / FOOD_BAR_MAX).clamp(0.0, 1.0) * 100.0;
        node.width = percent(pct);
    }
    if let Ok(mut node) = bar_queries.p1().single_mut() {
        let pct = (stats.health / 100.0).clamp(0.0, 1.0) * 100.0;
        node.width = percent(pct);
    }
    if let Ok(mut node) = bar_queries.p2().single_mut() {
        let pct = (stats.stamina / 100.0).clamp(0.0, 1.0) * 100.0;
        node.width = percent(pct);
    }
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
        app.add_systems(Startup, (setup_food_spawning, setup_food_ui))
            .add_systems(Update, (spawn_food, food_pickup, update_food_ui));
    }
}

#[derive(Component)]
struct FoodBarFill;

#[derive(Component)]
struct HealthBarFill;

#[derive(Component)]
struct StaminaBarFill;
