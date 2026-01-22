// grids and tiles live here
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::prelude::MeshMaterial2d;

use crate::player::{Facing, Player, PlayerState};

pub const HEIGHT: usize = 600;
pub const WIDTH: usize = 600;

const MAX_DISTANCE: usize = 72;
pub const WORLD_TILE_SIZE: f32 = 4.0;
pub const PLAYER_SIZE: f32 = 24.0;
const VIEW_ANGLE_DEGREES: f32 = 90.0;
const RENDER_PADDING_TILES: i32 = 8;
const CHUNK_SIZE: usize = 25;
const PIXEL_LEVELS: f32 = 6.0;
const DITHER_STRENGTH: f32 = 0.6;
const LIGHT_SNAP: f32 = WORLD_TILE_SIZE * 0.25;

pub type Field = Vec<Vec<bool>>;

#[derive(Resource, Debug, Clone)]
pub struct WorldGrid {
    pub field: Field,
    pub brightness: Vec<Vec<f32>>,
    pub walls: Vec<Vec<bool>>,
}

#[derive(Resource, Debug, Clone)]
pub struct WorldChunks {
    pub cols: usize,
    pub rows: usize,
    pub meshes: Vec<Handle<Mesh>>,
}

fn vector_field() -> Field {
    let field = vec![vec![false; WIDTH]; HEIGHT];
    return field;
}

fn brightness_field() -> Vec<Vec<f32>> {
    vec![vec![0.0; WIDTH]; HEIGHT]
}

fn walls_field() -> Vec<Vec<bool>> {
    let mut walls = vec![vec![false; WIDTH]; HEIGHT];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let is_wall = x == 0 || y == 0 || x == WIDTH - 1 || y == HEIGHT - 1;
            walls[y][x] = is_wall;
        }
    }
    walls
}

fn is_wall_tile(grid: &WorldGrid, x: usize, y: usize) -> bool {
    grid.walls[y][x]
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

fn set_chunk_tile_color(
    meshes: &mut Assets<Mesh>,
    chunks: &WorldChunks,
    x: usize,
    y: usize,
    color: [f32; 4],
) {
    let chunk_x = x / CHUNK_SIZE;
    let chunk_y = y / CHUNK_SIZE;
    let local_x = x % CHUNK_SIZE;
    let local_y = y % CHUNK_SIZE;
    let index = chunk_y * chunks.cols + chunk_x;
    let Some(handle) = chunks.meshes.get(index) else {
        return;
    };
    let Some(mesh) = meshes.get_mut(handle) else {
        return;
    };
    let Some(VertexAttributeValues::Float32x4(colors)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
    else {
        return;
    };
    let base = (local_y * CHUNK_SIZE + local_x) * 4;
    if base + 3 >= colors.len() {
        return;
    }
    colors[base] = color;
    colors[base + 1] = color;
    colors[base + 2] = color;
    colors[base + 3] = color;
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

fn spawn_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    grid: Res<WorldGrid>,
    mut chunks: ResMut<WorldChunks>,
) {
    let cols = (WIDTH + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let rows = (HEIGHT + CHUNK_SIZE - 1) / CHUNK_SIZE;
    chunks.cols = cols;
    chunks.rows = rows;
    chunks.meshes.clear();
    chunks.meshes.reserve(cols * rows);

    let material = materials.add(ColorMaterial::from(Color::WHITE));

    for chunk_y in 0..rows {
        for chunk_x in 0..cols {
            let start_x = chunk_x * CHUNK_SIZE;
            let start_y = chunk_y * CHUNK_SIZE;
            let end_x = (start_x + CHUNK_SIZE).min(WIDTH);
            let end_y = (start_y + CHUNK_SIZE).min(HEIGHT);
            let chunk_w = end_x - start_x;
            let chunk_h = end_y - start_y;

            let mut positions = Vec::with_capacity(chunk_w * chunk_h * 4);
            let mut uvs = Vec::with_capacity(chunk_w * chunk_h * 4);
            let mut colors = Vec::with_capacity(chunk_w * chunk_h * 4);
            let mut indices = Vec::with_capacity(chunk_w * chunk_h * 6);

            for local_y in 0..chunk_h {
                for local_x in 0..chunk_w {
                    let world_x = start_x + local_x;
                    let world_y = start_y + local_y;
                    let x0 = local_x as f32 * WORLD_TILE_SIZE;
                    let y0 = local_y as f32 * WORLD_TILE_SIZE;
                    let x1 = x0 + WORLD_TILE_SIZE;
                    let y1 = y0 + WORLD_TILE_SIZE;

                    let base = positions.len() as u32;
                    positions.extend_from_slice(&[
                        [x0, y0, 0.0],
                        [x1, y0, 0.0],
                        [x1, y1, 0.0],
                        [x0, y1, 0.0],
                    ]);
                    uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

                    let color = if is_wall_tile(&grid, world_x, world_y) {
                        Color::srgb(0.6, 0.6, 0.6).to_linear()
                    } else {
                        Color::BLACK.to_linear()
                    };
                    let color = [color.red, color.green, color.blue, color.alpha];
                    colors.extend_from_slice(&[color; 4]);

                    indices.extend_from_slice(&[
                        base,
                        base + 2,
                        base + 1,
                        base,
                        base + 3,
                        base + 2,
                    ]);
                }
            }

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            mesh.insert_indices(Indices::U32(indices));

            let handle = meshes.add(mesh);
            chunks.meshes.push(handle.clone());

            let chunk_origin = Vec3::new(
                start_x as f32 * WORLD_TILE_SIZE,
                start_y as f32 * WORLD_TILE_SIZE,
                -1.0,
            );
            commands.spawn((
                Mesh2d(handle),
                MeshMaterial2d(material.clone()),
                Transform::from_translation(chunk_origin),
            ));
        }
    }
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

    let max_brightness = 0.85;
    let hidden_brightness = 0.0;
    let brightness_curve = 1.35;
    let distance_bias = 1.05;
    let side_bias = 1.15;
    let smooth_speed = 48.0;
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
            if is_wall_tile(&grid, ux, uy) {
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

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(WorldGrid {
                field: vector_field(),
                brightness: brightness_field(),
                walls: walls_field(),
            })
            .insert_resource(WorldChunks {
                cols: 0,
                rows: 0,
                meshes: Vec::new(),
            })
            .add_systems(Startup, spawn_chunks)
            .add_systems(PostUpdate, update_visibility);
    }
}
