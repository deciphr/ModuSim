// valve.rs
// Copyright (C) 2025 deciphr
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// >>> Constants <<<
const DEFAULT_SPAWN_RATE: f32 = 1.0;

// >>> Components <<<
#[derive(Component)]
pub struct Valve {
    pub coil_address: u16,
    pub holding_address: u16,
}

#[derive(Component)]
pub struct ValvePosition(pub Vec2);

#[derive(Component)]
pub struct Ball;

// >>> Resources <<<
#[derive(Resource)]
pub struct ValveState {
    pub is_open: bool,
    pub spawn_rate: f32,
}

impl Default for ValveState {
    fn default() -> Self {
        Self {
            is_open: false,
            spawn_rate: DEFAULT_SPAWN_RATE,
        }
    }
}

#[derive(Resource)]
pub struct BallSpawner {
    timer: Timer,
}

impl Default for BallSpawner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(DEFAULT_SPAWN_RATE, TimerMode::Repeating),
        }
    }
}

// >>> Bundle <<<
#[derive(Bundle)]
pub struct ValveBundle {
    valve: Valve,
    position: ValvePosition,
    sprite: Sprite,
    transform: Transform,
}

impl Valve {
    pub fn new(coil_address: u16, holding_address: u16, position: Vec2) -> ValveBundle {
        ValveBundle {
            valve: Valve {
                coil_address,
                holding_address,
            },
            position: ValvePosition(position),
            sprite: Sprite::from_color(
                Color::srgb(0.8, 0.2, 0.2), // Red when closed (default)
                Vec2::new(20.0, 5.0),
            ),
            transform: Transform::from_translation(position.extend(0.0)),
        }
    }
}

// >>> Input System <<<
pub fn handle_valve_input(
    mut valve_state: ResMut<ValveState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        valve_state.is_open = !valve_state.is_open;
        println!("Valve manually toggled to: {}", valve_state.is_open);
    }
}

// >>> Modbus Synchronization <<<
pub fn sync_valves_to_modbus(
    valves: Query<&Valve>,
    valve_state: Res<ValveState>,
    modbus_state: Res<super::ModbusState>,
) {
    if valve_state.is_changed() {
        if let Ok(mut coils) = modbus_state.coils.lock() {
            for valve in valves.iter() {
                coils.insert(valve.coil_address, valve_state.is_open);
            }
        }

        if let Ok(mut holdings) = modbus_state.holding_registers.lock() {
            for valve in valves.iter() {
                holdings.insert(valve.holding_address, valve_state.spawn_rate as u16);
            }
        }
    }
}

pub fn sync_modbus_to_valves(
    valves: Query<&Valve>,
    mut valve_state: ResMut<ValveState>,
    modbus_state: Res<super::ModbusState>,
) {
    if let Ok(coils) = modbus_state.coils.lock() {
        for valve in valves.iter() {
            if let Some(&coil_state) = coils.get(&valve.coil_address) {
                if valve_state.is_open != coil_state {
                    valve_state.is_open = coil_state;
                    break;
                }
            }
        }
    }

    if let Ok(holdings) = modbus_state.holding_registers.lock() {
        for valve in valves.iter() {
            if let Some(&holding_state) = holdings.get(&valve.holding_address) {
                if valve_state.spawn_rate != holding_state as f32 {
                    valve_state.spawn_rate = holding_state as f32;
                    break;
                }
            }
        }
    }
}

// >>> Visual System <<<
pub fn update_valve_visuals(
    valve_state: Res<ValveState>,
    mut valves: Query<&mut Sprite, With<Valve>>,
) {
    if valve_state.is_changed() {
        let color = if valve_state.is_open {
            Color::srgb(0.2, 0.8, 0.2) // Green when open
        } else {
            Color::srgb(0.8, 0.2, 0.2) // Red when closed
        };

        for mut sprite in valves.iter_mut() {
            sprite.color = color;
        }
    }
}

// >>> Ball Spawning System <<<
pub fn update_ball_spawner_timer(
    valve_state: Res<ValveState>,
    mut ball_spawner: ResMut<BallSpawner>,
) {
    if valve_state.is_changed() {
        ball_spawner.timer.set_duration(Duration::from_secs_f32(valve_state.spawn_rate));
        ball_spawner.timer.reset();
    }
}

pub fn spawn_balls(
    time: Res<Time>,
    valve_state: Res<ValveState>,
    valves: Query<&ValvePosition, With<Valve>>,
    mut ball_spawner: ResMut<BallSpawner>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !valve_state.is_open {
        return;
    }

    ball_spawner.timer.tick(time.delta());

    if ball_spawner.timer.just_finished() {
        for valve_position in valves.iter() {
            spawn_ball(&mut commands, valve_position.0, &mut meshes, &mut materials);
        }
    }
}
fn spawn_ball(
    commands: &mut Commands,
    position: Vec2,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(10.0),
        Mesh2d(meshes.add(Circle::new(10.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.3, 0.7, 1.0)))),
        Transform::from_translation(position.extend(0.0)),
        Ball,
        BallLifetime::new(30.0),
    ));
}

// >>> Ball Cleanup System <<<
#[derive(Component)]
pub struct BallLifetime {
    timer: Timer,
}

impl BallLifetime {
    pub fn new(lifetime_seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(lifetime_seconds, TimerMode::Once),
        }
    }
}

pub fn cleanup_old_balls(
    time: Res<Time>,
    mut commands: Commands,
    mut balls: Query<(Entity, &mut BallLifetime), With<Ball>>,
) {
    for (entity, mut lifetime) in balls.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn cleanup_fallen_balls(
    mut commands: Commands,
    balls: Query<(Entity, &Transform), With<Ball>>,
) {
    const DESPAWN_Y_THRESHOLD: f32 = -500.0;

    for (entity, transform) in balls.iter() {
        if transform.translation.y < DESPAWN_Y_THRESHOLD {
            commands.entity(entity).despawn();
        }
    }
}

pub fn limit_ball_count(mut commands: Commands, balls: Query<Entity, With<Ball>>) {
    const MAX_BALLS: usize = 100;

    let ball_count = balls.iter().count();
    if ball_count > MAX_BALLS {
        let balls_to_remove = ball_count - MAX_BALLS;
        for entity in balls.iter().take(balls_to_remove) {
            commands.entity(entity).despawn();
        }
    }
}

// >>> Plugin <<<
pub struct ValvePlugin;

impl Plugin for ValvePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ValveState::default())
            .insert_resource(BallSpawner::default())
            .add_systems(
                Update,
                (
                    handle_valve_input,
                    sync_valves_to_modbus,
                    sync_modbus_to_valves,
                    update_valve_visuals,
                    update_ball_spawner_timer,
                    spawn_balls,
                    cleanup_old_balls,
                    cleanup_fallen_balls,
                    limit_ball_count,
                )
                    .chain(),
            );
    }
}
