use bevy::mesh::Mesh;
use bevy::prelude::*;

use crate::player::{Facing, Player, PlayerState};
use crate::world::{set_chunk_tile_color, WorldChunks, WorldGrid, HEIGHT, WIDTH, WORLD_TILE_SIZE};

const MAX_DISTANCE: usize = 124;
const VIEW_ANGLE_DEGREES: f32 = 120.0;
const RENDER_PADDING_TILES: i32 = 8;
const PIXEL_LEVELS: f32 = 6.0;
const DITHER_STRENGTH: f32 = 0.8;
const LIGHT_SNAP: f32 = 1.0;

fn in_bounds(x: i32, y: i32) -> bool {
    let lower_bound = x >= 0 && y >= 0;
    let upper_bound = x < WIDTH as i32 && y < HEIGHT as i32;

    lower_bound && upper_bound
}

fn set_visible(field: &mut Vec<Vec<bool>>, x: i32, y: i32, visible: bool) {
    if in_bounds(x, y) {
        let ux = x as usize;
        let uy = y as usize;
        field[uy][ux] = visible;
    }
}

fn facing_dir(facing: Facing) -> IVec2 {
    match facing {
        Facing::Up => IVec2::new(0, 1),
        Facing::UpRight => IVec2::new(1, 1),
        Facing::Right => IVec2::new(1, 0),
        Facing::DownRight => IVec2::new(1, -1),
        Facing::Down => IVec2::new(0, -1),
        Facing::DownLeft => IVec2::new(-1, -1),
        Facing::Left => IVec2::new(-1, 0),
        Facing::UpLeft => IVec2::new(-1, 1),
    }
}

fn is_visible_in_cone(
    tile_center: Vec2,
    player_pos: Vec2,
    facing: Facing,
    range: f32,
    spread: f32,
) -> bool {
    let delta = (tile_center - player_pos) / WORLD_TILE_SIZE;
    let dir = facing_dir(facing).as_vec2();

    let forward = delta.dot(dir);
    if forward <= 0.0 {
        return false;
    }

    let forward_scale = (dir.x.abs() + dir.y.abs()).max(1.0);
    let forward_steps = forward / forward_scale;
    if forward_steps > range {
        return false;
    }

    let side = delta.x * -dir.y + delta.y * dir.x;
    side.abs() <= forward_steps * spread
}

fn bayer_4x4(x: usize, y: usize) -> f32 {
    const BAYER: [f32; 16] = [
        0.0 / 16.0,
        8.0 / 16.0,
        2.0 / 16.0,
        10.0 / 16.0,
        12.0 / 16.0,
        4.0 / 16.0,
        14.0 / 16.0,
        6.0 / 16.0,
        3.0 / 16.0,
        11.0 / 16.0,
        1.0 / 16.0,
        9.0 / 16.0,
        15.0 / 16.0,
        7.0 / 16.0,
        13.0 / 16.0,
        5.0 / 16.0,
    ];
    let idx = (x & 3) + ((y & 3) << 2);
    BAYER[idx]
}

fn update_visibility(
    mut grid: ResMut<WorldGrid>,
    time: Res<Time>,
    player_query: Query<(&Transform, &PlayerState), With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunks: Res<WorldChunks>,
) {
    let Ok((player_transform, player_state)) = player_query.single() else {
        return;
    };

    let raw_pos = player_transform.translation.truncate();
    let light_pos = if LIGHT_SNAP > 0.0 {
        (raw_pos / LIGHT_SNAP).round() * LIGHT_SNAP
    } else {
        raw_pos
    };
    let player_tile_x = (light_pos.x / WORLD_TILE_SIZE).floor() as i32;
    let player_tile_y = (light_pos.y / WORLD_TILE_SIZE).floor() as i32;
    let range = MAX_DISTANCE as f32;
    let spread = (VIEW_ANGLE_DEGREES.to_radians() * 0.5).tan();

    let max_brightness = 0.93;
    let hidden_brightness = 0.0;
    let brightness_curve = 0.70;
    let distance_bias = 1.05;
    let side_bias = 1.15;
    let smooth_speed = 60.0;
    let lerp_alpha = (smooth_speed * time.delta_secs()).clamp(0.0, 1.0);

    let inner_bound = range.ceil() as i32 + 2;
    let outer_bound = inner_bound + RENDER_PADDING_TILES;
    let min_x = (player_tile_x - outer_bound).max(0);
    let max_x = (player_tile_x + outer_bound).min(WIDTH as i32 - 1);
    let min_y = (player_tile_y - outer_bound).max(0);
    let max_y = (player_tile_y + outer_bound).min(HEIGHT as i32 - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let ux = x as usize;
            let uy = y as usize;
            if grid.walls[uy][ux] {
                continue;
            }
            let in_inner = x >= player_tile_x - inner_bound
                && x <= player_tile_x + inner_bound
                && y >= player_tile_y - inner_bound
                && y <= player_tile_y + inner_bound;
            let tile_center = Vec2::new(
                x as f32 * WORLD_TILE_SIZE + WORLD_TILE_SIZE * 0.5,
                y as f32 * WORLD_TILE_SIZE + WORLD_TILE_SIZE * 0.5,
            );
            let visible = if in_inner {
                is_visible_in_cone(
                    tile_center,
                    light_pos,
                    player_state.facing,
                    range,
                    spread,
                )
            } else {
                false
            };
            set_visible(&mut grid.field, x, y, visible);
            let target_brightness = if visible {
                let delta = (tile_center - light_pos) / WORLD_TILE_SIZE;
                let distance = delta.length();
                let t_distance = (distance / range).clamp(0.0, 1.0).powf(distance_bias);

                let dir = facing_dir(player_state.facing).as_vec2();
                let forward = delta.dot(dir);
                let forward_scale = (dir.x.abs() + dir.y.abs()).max(1.0);
                let forward_steps = forward / forward_scale;
                let side = delta.x * -dir.y + delta.y * dir.x;
                let side_denom = (forward_steps * spread).abs().max(0.0001);
                let side_ratio = (side.abs() / side_denom)
                    .clamp(0.0, 1.0)
                    .powf(side_bias);

                let t = t_distance.max(side_ratio).clamp(0.0, 1.0);
                let falloff = (1.0 - t).clamp(0.0, 1.0).powf(brightness_curve);
                max_brightness * falloff
            } else {
                hidden_brightness
            };
            let current = grid.brightness[uy][ux];
            let next = current + (target_brightness - current) * lerp_alpha;
            if (next - current).abs() > 0.001 {
                grid.brightness[uy][ux] = next;
                let normalized = if max_brightness > 0.0 {
                    (next / max_brightness).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let dx = (x - player_tile_x).rem_euclid(4) as usize;
                let dy = (y - player_tile_y).rem_euclid(4) as usize;
                let dither = bayer_4x4(dx, dy) * DITHER_STRENGTH;
                let stepped = ((normalized * PIXEL_LEVELS) + dither).floor() / PIXEL_LEVELS;
                let display = max_brightness * stepped.clamp(0.0, 1.0);
                let color = Color::srgb(display, display, display).to_linear();
                let color = [color.red, color.green, color.blue, color.alpha];
                set_chunk_tile_color(&mut meshes, &chunks, ux, uy, color);
            }
        }
    }
}

pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_visibility);
    }
}
