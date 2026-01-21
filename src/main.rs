// Replace your main.rs with the following code.
mod player;

use bevy::prelude::*;
use crate::player::PlayerPlugin;

fn main() {
	App::new()
    .insert_resource(ClearColor(Color::WHITE))
	.add_plugins(DefaultPlugins)
	.add_systems(Startup, setup)
    .add_plugins(PlayerPlugin)
	.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d);
}
