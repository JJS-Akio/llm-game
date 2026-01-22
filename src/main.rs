// Replace your main.rs with the following code.
mod player;
mod light;
mod world;

use bevy::prelude::*;
use crate::player::{Player, PlayerPlugin};
use crate::light::LightPlugin;
use crate::world::{WorldPlugin, HEIGHT, WORLD_TILE_SIZE, WIDTH};

fn main() {
	App::new()
	.add_plugins(DefaultPlugins)
	.add_systems(Startup, setup)
	.add_systems(Update, follow_player_camera)
    .add_plugins(PlayerPlugin)
    .add_plugins(WorldPlugin)
    .add_plugins(LightPlugin)
	.run();
}

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
	let center_x = (WIDTH as f32 / 2.0).floor() * WORLD_TILE_SIZE;
	let center_y = (HEIGHT as f32 / 2.0).floor() * WORLD_TILE_SIZE;
	commands.spawn((
		Camera2d,
		MainCamera,
		Transform::from_translation(Vec3::new(center_x, center_y, 10.0)),
	));
}

fn follow_player_camera(
	player_query: Query<&Transform, With<Player>>,
	mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
	let Ok(player_transform) = player_query.single() else {
		return;
	};
	let Ok(mut camera_transform) = camera_query.single_mut() else {
		return;
	};
	camera_transform.translation.x = player_transform.translation.x;
	camera_transform.translation.y = player_transform.translation.y;
}
