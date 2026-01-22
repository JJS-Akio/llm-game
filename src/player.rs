use bevy::prelude::*;

use crate::world::{HEIGHT, PLAYER_SIZE, WIDTH, WORLD_TILE_SIZE};
const MOVE_SPEED: f32 = 140.0;
const LOW_STAMINA_SPEED_FACTOR: f32 = 1.0 / 3.0;
const ATLAS_COLUMNS: u32 = 8;

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    Up,
    Left,
    Down,
    Right,
    UpRight,
    DownRight,
    UpLeft,
    DownLeft,
}

#[derive(Component)]
pub struct Stats {
    pub health: f32,
    pub stamina: f32,
    pub food_bar: f32,
}

#[derive(Component)]
pub struct MovementTracker {
    seconds: f32,
    is_moving: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerState {
    pub facing: Facing,
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("player.ppm");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(PLAYER_SIZE as u32, PLAYER_SIZE as u32),
        ATLAS_COLUMNS,
        1,
        None,
        None,
    );
    let layout_handle = atlas_layouts.add(layout);

    let facing = Facing::Down;

    let center_x = (WIDTH as f32 / 2.0).floor() * WORLD_TILE_SIZE;
    let center_y = (HEIGHT as f32 / 2.0).floor() * WORLD_TILE_SIZE;

    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: layout_handle,
                index: facing_index(facing),
            },
        ),
        Transform::from_translation(Vec3::new(center_x, center_y, 0.0)),
        Player,
        PlayerState { facing },
        Stats { health: 100.0, stamina: 100.0, food_bar: 100.0},
        MovementTracker { seconds: 0.0, is_moving: false},
    ));
}

fn energy_system(
    time: Res<Time>,
    mut query: Query<(&MovementTracker, &mut Stats)> 
){
    let Ok((tracker, mut stats)) = query.single_mut() else {
        return;
    };

    let stamina_drain_per_sec = 8.0;
    let stamina_regen_per_sec = 6.0;
    let health_drain_per_sec = 3.0;
    let food_bar_drain_per_sec = 2.0;
    let food_bar_empty_drain_per_sec = 4.0;
    let food_bar_empty_health_drain_per_sec = 10.0;
    let dt = time.delta_secs();

    stats.food_bar = (stats.food_bar - food_bar_drain_per_sec * dt).max(0.0);

    if stats.food_bar <= 0.0{
        stats.health = (stats.health - food_bar_empty_health_drain_per_sec * dt).max(0.0)
    }

    if tracker.is_moving {
        stats.stamina = (stats.stamina - stamina_drain_per_sec * dt).max(0.0);
        if stats.stamina <= 0.0{
            stats.health = (stats.health - health_drain_per_sec * dt).max(0.0);
        }
    }
    let allow_regen = stats.stamina < 100.0 && stats.food_bar > 0.0;
    if !tracker.is_moving{
        if allow_regen {
            stats.stamina = (stats.stamina + stamina_regen_per_sec * dt).min(100.0);
            stats.food_bar = (stats.food_bar - food_bar_empty_drain_per_sec * dt).max(0.0);
        }
    }
}

fn move_player(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &mut PlayerState,
            &mut Sprite,
            &mut MovementTracker,
            &Stats,
        ),
        With<Player>,
    >,
) {
    let Ok((mut transform, mut state, mut sprite, mut tracker, stats)) = query.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    if direction != Vec2::ZERO {
        tracker.is_moving = true;
        tracker.seconds += time.delta_secs();
        let speed = if stats.stamina <= 0.0 {
            MOVE_SPEED * LOW_STAMINA_SPEED_FACTOR
        } else {
            MOVE_SPEED
        };
        let delta = direction.normalize() * speed * time.delta_secs();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;

        if direction.x != 0.0 && direction.y != 0.0 {
            state.facing = if direction.x > 0.0 && direction.y > 0.0 {
                Facing::UpRight
            } else if direction.x > 0.0 && direction.y < 0.0 {
                Facing::DownRight
            } else if direction.x < 0.0 && direction.y > 0.0 {
                Facing::UpLeft
            } else {
                Facing::DownLeft
            };
        } else if direction.x != 0.0 {
            state.facing = if direction.x > 0.0 {
                Facing::Right
            } else {
                Facing::Left
            };
        } else {
            state.facing = if direction.y > 0.0 { Facing::Up } else { Facing::Down };
        }
    }
    let rest_rate: f32 = 1.0;
    if direction == Vec2::ZERO {
        tracker.is_moving = false;
        tracker.seconds = f32::max(0.0, tracker.seconds - rest_rate * time.delta_secs());
    }

    if let Some(atlas) = sprite.texture_atlas.as_mut() {
        atlas.index = facing_index(state.facing);
    }

    let min_x = WORLD_TILE_SIZE;
    let max_x = (WIDTH as f32 - 2.0) * WORLD_TILE_SIZE;
    let min_y = WORLD_TILE_SIZE;
    let max_y = (HEIGHT as f32 - 2.0) * WORLD_TILE_SIZE;

    transform.translation.x = transform.translation.x.clamp(min_x, max_x);
    transform.translation.y = transform.translation.y.clamp(min_y, max_y);
}

fn facing_index(facing: Facing) -> usize {
    match facing {
        Facing::Up => 0,
        Facing::Left => 1,
        Facing::Down => 2,
        Facing::Right => 3,
        Facing::UpRight => 4,
        Facing::DownRight => 5,
        Facing::UpLeft => 6,
        Facing::DownLeft => 7,
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (move_player, (energy_system)).chain());
    }
}
