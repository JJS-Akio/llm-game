// grids and tiles live here
use bevy::prelude::*;

use crate::player::{Facing, Player, PlayerState};

pub const HEIGHT: usize = 100;
pub const WIDTH: usize = 100;

const MAX_DISTANCE: usize = 8;
pub const TILE_SIZE: f32 = 24.0;
const VIEW_ANGLE_DEGREES: f32 = 90.0;

pub type Field = Vec<Vec<bool>>;

#[derive(Resource, Debug, Clone)]
pub struct WorldGrid {
    pub field: Field,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Wall;

fn vector_field() -> Field {
    let field = vec![vec![false; WIDTH]; HEIGHT];
    return field;
}

fn in_bounds(x: i32, y: i32) -> bool {
    let lower_bound = x >= 0 && y >= 0;
    let upper_bound = x < WIDTH as i32 && y < HEIGHT as i32;

    return lower_bound && upper_bound;
}

fn set_visible(field: &mut Field, x: i32, y:i32, visible: bool){
    if in_bounds(x, y) {
        let ux = x as usize;
        let uy = y as usize;
        field[uy][ux] = visible;
    }
}

fn facing_dir(facing: Facing) -> Vec2 {
    match facing {
        Facing::Up => Vec2::new(0.0, 1.0),
        Facing::UpRight => Vec2::new(1.0, 1.0).normalize_or_zero(),
        Facing::Right => Vec2::new(1.0, 0.0),
        Facing::DownRight => Vec2::new(1.0, -1.0).normalize_or_zero(),
        Facing::Down => Vec2::new(0.0, -1.0),
        Facing::DownLeft => Vec2::new(-1.0, -1.0).normalize_or_zero(),
        Facing::Left => Vec2::new(-1.0, 0.0),
        Facing::UpLeft => Vec2::new(-1.0, 1.0).normalize_or_zero(),
    }
}

fn is_visible_in_cone(
    tile_center: Vec2,
    player_pos: Vec2,
    facing: Facing,
    range: f32,
    cos_half_angle: f32,
) -> bool {
    let delta = tile_center - player_pos;
    let dist2 = delta.length_squared();

    if dist2 > range * range {
        return false;
    }

    let dir = facing_dir(facing);
    let dot = delta.dot(dir);

    if dot <= 0.0 {
        return false; // behind the player
    }

    dot * dot >= dist2 * (cos_half_angle * cos_half_angle)
}

fn spawn_tiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture: Handle<Image> = asset_server.load("ground.ppm");
    let wall_texture: Handle<Image> = asset_server.load("wall.ppm");

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let is_wall = x == 0 || y == 0 || x == WIDTH - 1 || y == HEIGHT - 1;
            let position = Vec3::new(
                x as f32 * TILE_SIZE,
                y as f32 * TILE_SIZE,
                -1.0,
            );
            let sprite = Sprite::from_image(if is_wall {
                wall_texture.clone()
            } else {
                texture.clone()
            });
            let mut entity = commands.spawn((
                sprite,
                Transform::from_translation(position),
                TileCoord {
                    x: x as i32,
                    y: y as i32,
                },
            ));
            if is_wall {
                entity.insert(Wall);
            }
        }
    }
}

fn update_visibility(
    mut grid: ResMut<WorldGrid>,
    player_query: Query<(&Transform, &PlayerState), With<Player>>,
    mut tiles: Query<(&TileCoord, &mut Sprite), Without<Wall>>,
) {
    let Ok((player_transform, player_state)) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let range = MAX_DISTANCE as f32 * TILE_SIZE;
    let cos_half_angle = (VIEW_ANGLE_DEGREES.to_radians() * 0.5).cos();

    let near_color = Color::srgb(0.85, 0.85, 0.85);
    let mid_color = Color::srgb(0.6, 0.6, 0.6);
    let far_color = Color::srgb(0.35, 0.35, 0.35);
    let hidden_color = Color::BLACK;

    for (coord, mut sprite) in tiles.iter_mut() {
        let tile_center = Vec2::new(
            coord.x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
            coord.y as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        );
        let visible = is_visible_in_cone(
            tile_center,
            player_pos,
            player_state.facing,
            range,
            cos_half_angle,
        );
        set_visible(&mut grid.field, coord.x, coord.y, visible);
        if visible {
            let distance = tile_center.distance(player_pos);
            let t = (distance / range).clamp(0.0, 1.0);
            sprite.color = if t < 0.58 {
                near_color
            } else if t < 0.85 {
                mid_color
            } else {
                far_color
            };
        } else {
            sprite.color = hidden_color;
        }
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(WorldGrid {
                field: vector_field(),
            })
            .add_systems(Startup, spawn_tiles)
            .add_systems(PostUpdate, update_visibility);
    }
}
