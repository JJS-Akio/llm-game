// grids and tiles live here
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::prelude::MeshMaterial2d;

pub const HEIGHT: usize = 1000;
pub const WIDTH: usize = 1000;

pub const WORLD_TILE_SIZE: f32 = 1.0;
pub const PLAYER_SIZE: f32 = 24.0;
const CHUNK_SIZE: usize = 25;

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

pub fn set_chunk_tile_color(
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
            .add_systems(Startup, spawn_chunks);
    }
}
