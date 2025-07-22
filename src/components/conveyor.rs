// conveyor.rs
// Copyright (C) 2025 deciphr
// SPDX-License-Identifier: GPL-3.0-or-later

use super::modbus::ModbusState;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// >>> Constants <<<
const CONVEYOR_SPEED: f32 = 100.0;

// >>> Components <<<
#[derive(Component)]
pub struct Conveyor {
    pub coil_address: u16,
    pub holding_address: u16,
}

// >>> Resources <<<
#[derive(Resource)]
pub struct ConveyorState {
    pub is_running: bool,
    pub speed: f32,
}

impl Default for ConveyorState {
    fn default() -> Self {
        ConveyorState {
            is_running: true,
            speed: CONVEYOR_SPEED,
        }
    }
}

// >>> Bundles <<<
#[derive(Bundle)]
pub struct ConveyorBundle {
    conveyor: Conveyor,
    collider: Collider,
    active_hooks: ActiveHooks,
    sprite: Sprite,
    transform: Transform,
}

impl Conveyor {
    pub fn new(
        coil_address: u16,
        holding_address: u16,
        position: Vec2,
        width: f32,
        height: f32,
    ) -> ConveyorBundle {
        ConveyorBundle {
            conveyor: Conveyor {
                coil_address,
                holding_address,
            },
            collider: Collider::cuboid(width / 2.0, height / 2.0),
            active_hooks: ActiveHooks::MODIFY_SOLVER_CONTACTS,
            sprite: Sprite::from_color(Color::BLACK, Vec2::new(width, height)),
            transform: Transform::from_translation(position.extend(0.0)),
        }
    }
}

// >>> Systems <<<
#[derive(SystemParam)]
pub struct ConveyorPhysicsHook<'w> {
    conveyor_state: Res<'w, ConveyorState>,
}

impl BevyPhysicsHooks for ConveyorPhysicsHook<'_> {
    fn modify_solver_contacts(&self, context: ContactModificationContextView) {
        if self.conveyor_state.is_running {
            for solver_contact in &mut *context.raw.solver_contacts {
                solver_contact.tangent_velocity.x = self.conveyor_state.speed;
            }
        } else {
            for solver_contact in &mut *context.raw.solver_contacts {
                solver_contact.tangent_velocity.x = 0.0;
            }
        }
    }
}

pub fn handle_conveyor_input(
    mut conveyor_state: ResMut<ConveyorState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        conveyor_state.is_running = !conveyor_state.is_running;
        info!(
            "Conveyor: {}",
            if conveyor_state.is_running {
                "Running"
            } else {
                "Stopped"
            }
        );
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        conveyor_state.speed += 10.0;
        info!("Conveyor speed: {}", conveyor_state.speed);
    }

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        conveyor_state.speed = conveyor_state.speed - 10.0;
        info!("Conveyor speed: {}", conveyor_state.speed);
    }
}

// >>> Modbus Synchronization <<<
pub fn sync_conveyor_to_modbus(
    conveyors: Query<&Conveyor>,
    conveyor_state: Res<ConveyorState>,
    modbus_state: Res<ModbusState>,
) {
    if conveyor_state.is_changed() {
        if let Ok(mut coils) = modbus_state.coils.lock() {
            for conveyor in conveyors.iter() {
                coils.insert(conveyor.coil_address, conveyor_state.is_running);
            }
        }
        if let Ok(mut holdings) = modbus_state.holding_registers.lock() {
            for conveyor in conveyors.iter() {
                holdings.insert(conveyor.holding_address, conveyor_state.speed as u16);
                info!("Conveyor {}'s speed set to: {}", conveyor.holding_address, conveyor_state.speed);
            }
        }
    }
}

pub fn sync_modbus_to_conveyor(
    conveyors: Query<&Conveyor>,
    mut conveyor_state: ResMut<ConveyorState>,
    modbus_state: Res<ModbusState>,
) {
    if let Ok(coils) = modbus_state.coils.lock() {
        for conveyor in conveyors.iter() {
            if let Some(&coil_state) = coils.get(&conveyor.coil_address) {
                if conveyor_state.is_running != coil_state {
                    conveyor_state.is_running = coil_state;
                    println!("Conveyor {} set to: {}", conveyor.coil_address, coil_state);
                    break;
                }
            }
        }
    }

     if let Ok(holdings) = modbus_state.holding_registers.lock() {
        for conveyor in conveyors.iter() {
            if let Some(&holding_state) = holdings.get(&conveyor.holding_address) {
                if conveyor_state.speed != holding_state as f32 {
                    conveyor_state.speed = holding_state as f32;
                    println!("Conveyor speed {} set to: {}", conveyor.holding_address, holding_state);
                    break;
                }
            }
        }
    }
}

// >>> Plugin <<<
pub struct ConveyorPlugin;

impl Plugin for ConveyorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConveyorState::default())
            .add_plugins(RapierPhysicsPlugin::<ConveyorPhysicsHook>::pixels_per_meter(100.0))
            .add_systems(
                Update,
                (
                    sync_conveyor_to_modbus,
                    sync_modbus_to_conveyor,
                    handle_conveyor_input,
                )
                    .chain(),
            );
    }
}
