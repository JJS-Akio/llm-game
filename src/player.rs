use bevy::prelude::*;

use crate::food::{Food, FoodTracker};
use crate::world::{HEIGHT, PLAYER_SIZE, WIDTH, WORLD_TILE_SIZE};
const MOVE_SPEED: f32 = 140.0;
const LOW_STAMINA_SPEED_FACTOR: f32 = 1.0 / 3.0;
const ATLAS_COLUMNS: u32 = 8;
const FOOD_COLLISION_RADIUS: f32 = 12.0;
pub const FOOD_BAR_MAX: f32 = 100.0;
const STATS_MAX: f32 = 100.0;
const DEATH_OVERLAY_ALPHA: f32 = 0.8;
const STATUS_PIPS: usize = 4;
const STATUS_CHUNK: f32 = 25.0;
const STATUS_ICON_SIZE: f32 = 24.0;
const STATUS_PANEL_PADDING: f32 = 6.0;
const STATUS_ROW_GAP: f32 = 6.0;
const STATUS_PIP_GAP: f32 = 4.0;
const STATUS_PANEL_ALPHA: f32 = 1.0;
const STATUS_ROW_ALPHA: f32 = 1.0;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct StatusPip {
    kind: StatusKind,
    index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatusKind {
    Food,
    Health,
    Stamina,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PipState {
    Empty,
    Half,
    Full,
}

#[derive(Resource, Clone)]
struct StatusIconHandles {
    food_empty: Handle<Image>,
    food_half: Handle<Image>,
    food_full: Handle<Image>,
    health_empty: Handle<Image>,
    health_half: Handle<Image>,
    health_full: Handle<Image>,
    stamina_empty: Handle<Image>,
    stamina_half: Handle<Image>,
    stamina_full: Handle<Image>,
}

#[derive(Resource)]
struct DeathRespawnState {
    is_dead: bool,
}

impl DeathRespawnState {
    fn new() -> Self {
        Self { is_dead: false }
    }
}

#[derive(Component)]
struct DeathOverlay;

impl StatusIconHandles {
    fn new(asset_server: &AssetServer) -> Self {
        Self {
            food_empty: asset_server.load("food_empty.png"),
            food_half: asset_server.load("food_half.png"),
            food_full: asset_server.load("food_full.png"),
            health_empty: asset_server.load("health_empty.png"),
            health_half: asset_server.load("health_half.png"),
            health_full: asset_server.load("health_full.png"),
            stamina_empty: asset_server.load("stamina_empty.png"),
            stamina_half: asset_server.load("stamina_half.png"),
            stamina_full: asset_server.load("stamina_full.png"),
        }
    }

    fn handle_for(&self, kind: StatusKind, state: PipState) -> Handle<Image> {
        match (kind, state) {
            // Swap empty/full visuals so "full" reads as brighter.
            (StatusKind::Food, PipState::Empty) => self.food_full.clone(),
            (StatusKind::Food, PipState::Half) => self.food_half.clone(),
            (StatusKind::Food, PipState::Full) => self.food_empty.clone(),
            (StatusKind::Health, PipState::Empty) => self.health_full.clone(),
            (StatusKind::Health, PipState::Half) => self.health_half.clone(),
            (StatusKind::Health, PipState::Full) => self.health_empty.clone(),
            (StatusKind::Stamina, PipState::Empty) => self.stamina_full.clone(),
            (StatusKind::Stamina, PipState::Half) => self.stamina_half.clone(),
            (StatusKind::Stamina, PipState::Full) => self.stamina_empty.clone(),
        }
    }
}


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
    let texture: Handle<Image> = asset_server.load("player.png");
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
        Stats {
            health: STATS_MAX,
            stamina: STATS_MAX,
            food_bar: FOOD_BAR_MAX,
        },
        MovementTracker { seconds: 0.0, is_moving: false},
    ));
}

fn energy_system(
    time: Res<Time>,
    death_state: Res<DeathRespawnState>,
    mut query: Query<(&MovementTracker, &mut Stats)> 
){
    if death_state.is_dead {
        return;
    }

    let Ok((tracker, mut stats)) = query.single_mut() else {
        return;
    };

    let stamina_drain_per_sec = 8.0;
    let stamina_regen_per_sec = 12.0;
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
    food_tracker: Res<FoodTracker>,
    death_state: Res<DeathRespawnState>,
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
    if death_state.is_dead {
        return;
    }

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

    let dt = time.delta_secs();
    let mut did_move = false;
    if direction != Vec2::ZERO {
        let speed = if stats.stamina <= 0.0 {
            MOVE_SPEED * LOW_STAMINA_SPEED_FACTOR
        } else {
            MOVE_SPEED
        };
        let delta = direction.normalize() * speed * dt;
        let proposed_x = transform.translation.x + delta.x;
        let proposed_y = transform.translation.y + delta.y;
        let collision_radius_sq = FOOD_COLLISION_RADIUS * FOOD_COLLISION_RADIUS;
        let blocked = food_tracker.iter_locations().any(|location| {
            let food_x = location.x as f32 * WORLD_TILE_SIZE;
            let food_y = location.y as f32 * WORLD_TILE_SIZE;
            let dx = proposed_x - food_x;
            let dy = proposed_y - food_y;
            (dx * dx + dy * dy) <= collision_radius_sq
        });
        if !blocked {
            transform.translation.x = proposed_x;
            transform.translation.y = proposed_y;
            did_move = true;
        } else {
            tracker.is_moving = false;
        }

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
    if did_move {
        tracker.is_moving = true;
        tracker.seconds += dt;
    } else {
        tracker.is_moving = false;
        tracker.seconds = f32::max(0.0, tracker.seconds - rest_rate * dt);
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

fn setup_death_respawn(mut commands: Commands) {
    commands.insert_resource(DeathRespawnState::new());
}

fn setup_death_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                width: percent(100.0),
                height: percent(100.0),
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, DEATH_OVERLAY_ALPHA)),
            GlobalZIndex(100),
            Visibility::Hidden,
            DeathOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("You Died\nPress Enter (or R) for New Game"),
                TextFont::from_font_size(48.0),
                TextColor(Color::srgb(0.95, 0.1, 0.1)),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

fn handle_death_and_respawn(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut death_state: ResMut<DeathRespawnState>,
    mut food_tracker: ResMut<FoodTracker>,
    food_entities: Query<Entity, With<Food>>,
    mut overlay_query: Query<&mut Visibility, With<DeathOverlay>>,
    mut query: Query<
        (&mut Transform, &mut Stats, &mut MovementTracker, &mut PlayerState),
        With<Player>,
    >,
) {
    let Ok((mut transform, mut stats, mut tracker, mut player_state)) = query.single_mut() else {
        return;
    };
    let Ok(mut overlay_visibility) = overlay_query.single_mut() else {
        return;
    };

    if !death_state.is_dead && stats.health <= 0.0 {
        death_state.is_dead = true;
        tracker.is_moving = false;
        tracker.seconds = 0.0;
        *overlay_visibility = Visibility::Visible;

        for entity in &food_entities {
            commands.entity(entity).despawn();
        }
        food_tracker.clear();
        return;
    }

    if !death_state.is_dead {
        *overlay_visibility = Visibility::Hidden;
        return;
    }

    tracker.is_moving = false;
    let new_game_pressed = input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::KeyR);
    if !new_game_pressed {
        return;
    }

    let center_x = (WIDTH as f32 / 2.0).floor() * WORLD_TILE_SIZE;
    let center_y = (HEIGHT as f32 / 2.0).floor() * WORLD_TILE_SIZE;

    transform.translation.x = center_x;
    transform.translation.y = center_y;
    stats.health = STATS_MAX;
    stats.stamina = STATS_MAX;
    stats.food_bar = FOOD_BAR_MAX;
    player_state.facing = Facing::Down;
    death_state.is_dead = false;
    *overlay_visibility = Visibility::Hidden;

    for entity in &food_entities {
        commands.entity(entity).despawn();
    }
    food_tracker.clear();
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


fn setup_status_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let icon_handles = StatusIconHandles::new(&asset_server);
    commands.insert_resource(icon_handles.clone());

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(16.0),
                top: px(16.0),
                padding: UiRect::all(px(STATUS_PANEL_PADDING)),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: px(STATUS_ROW_GAP),
                ..default()
            },
            BackgroundColor(Color::srgba(0.86, 0.86, 0.86, STATUS_PANEL_ALPHA)),
            BorderColor::all(Color::srgb(0.25, 0.25, 0.25)),
        ))
        .with_children(|panel| {
            spawn_status_row(panel, &icon_handles, StatusKind::Food);
            spawn_status_row(panel, &icon_handles, StatusKind::Health);
            spawn_status_row(panel, &icon_handles, StatusKind::Stamina);
        });
}

fn update_status_ui(
    player_query: Query<&Stats, With<Player>>,
    icon_handles: Res<StatusIconHandles>,
    mut pip_query: Query<(&StatusPip, &mut ImageNode)>,
) {
    let Ok(stats) = player_query.single() else {
        return;
    };

    for (pip, mut image) in &mut pip_query {
        let value = status_value(stats, pip.kind);
        let state = pip_state(value, pip.index);
        image.image = icon_handles.handle_for(pip.kind, state);
    }
}

fn spawn_status_row(
    parent: &mut ChildSpawnerCommands,
    icon_handles: &StatusIconHandles,
    kind: StatusKind,
) {
    parent
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                column_gap: px(STATUS_PIP_GAP),
                padding: UiRect::all(px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.93, 0.93, 0.93, STATUS_ROW_ALPHA)),
        ))
        .with_children(|row| {
            for index in 0..STATUS_PIPS {
                row.spawn((
                    Node {
                        width: px(STATUS_ICON_SIZE),
                        height: px(STATUS_ICON_SIZE),
                        ..default()
                    },
                    ImageNode::new(icon_handles.handle_for(kind, PipState::Full)),
                    StatusPip { kind, index },
                ));
            }
        });
}

fn status_value(stats: &Stats, kind: StatusKind) -> f32 {
    match kind {
        StatusKind::Food => stats.food_bar,
        StatusKind::Health => stats.health,
        StatusKind::Stamina => stats.stamina,
    }
}

fn pip_state(value: f32, index: usize) -> PipState {
    let clamped_value = value.clamp(0.0, 100.0);
    let start = index as f32 * STATUS_CHUNK;
    let fill = ((clamped_value - start) / STATUS_CHUNK).clamp(0.0, 1.0);

    if fill > 0.5 {
        PipState::Full
    } else if fill > 0.0 {
        PipState::Half
    } else {
        PipState::Empty
    }
}


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup_death_respawn,
                spawn_player,
                setup_status_ui,
                setup_death_overlay,
            ),
        )
            .add_systems(
                Update,
                (
                    handle_death_and_respawn,
                    move_player,
                    update_status_ui,
                    (energy_system),
                )
                    .chain(),
            );
    }
}
