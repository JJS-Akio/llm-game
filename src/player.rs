use bevy::prelude::*;

const TILE_SIZE: u32 = 24;
const MOVE_SPEED: f32 = 140.0;
const ATLAS_COLUMNS: u32 = 4;

#[derive(Component)]
struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Facing {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Component, Debug, Clone, Copy)]
struct PlayerState {
    facing: Facing,
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("player.ppm");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_SIZE, TILE_SIZE),
        ATLAS_COLUMNS,
        1,
        None,
        None,
    );
    let layout_handle = atlas_layouts.add(layout);

    let facing = Facing::Down;

    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: layout_handle,
                index: facing_index(facing),
            },
        ),
        Transform::from_translation(Vec3::ZERO),
        Player,
        PlayerState { facing },
    ));
}

fn move_player(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PlayerState, &mut Sprite), With<Player>>,
) {
    let Ok((mut transform, mut state, mut sprite)) = query.single_mut() else {
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
        let delta = direction.normalize() * MOVE_SPEED * time.delta_secs();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;

        if direction.x.abs() > direction.y.abs() {
            state.facing = if direction.x > 0.0 {
                Facing::Right
            } else {
                Facing::Left
            };
        } else {
            state.facing = if direction.y > 0.0 { Facing::Up } else { Facing::Down };
        }
    }

    if let Some(atlas) = sprite.texture_atlas.as_mut() {
        atlas.index = facing_index(state.facing);
    }
}

fn facing_index(facing: Facing) -> usize {
    match facing {
        Facing::Up => 0,
        Facing::Left => 1,
        Facing::Down => 2,
        Facing::Right => 3,
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, move_player);
    }
}
