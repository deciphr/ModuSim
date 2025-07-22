// sensor.rs
// Copyright (C) 2025 deciphr
// SPDX-License-Identifier: GPL-3.0-or-later

use std::any::TypeId;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::{
    modbus::ModbusState,
    bottle::Bottle,
    conveyor::ConveyorState,
    valve::{Ball, ValveState},
};

// >>> Components <<<
#[derive(Component)]
pub struct Sensor {
    pub modbus_address: u16,
    pub sensor_tag: String,
    pub sensor_item: TypeId, // Item the sensor should detect
}

#[derive(Debug, Clone)]
pub struct SensorState {
    pub triggered: bool,
    pub last_triggered: bool,
    pub changed: bool,
}

impl Default for SensorState {
    fn default() -> Self {
        Self {
            triggered: false,
            last_triggered: false,
            changed: false,
        }
    }
}

// >>> Resources <<<

#[derive(Resource, Default)]
pub struct GlobalSensorState {
    states: std::collections::HashMap<String, SensorState>,
}

impl GlobalSensorState {
    pub fn add_sensor(&mut self, sensor_tag: String, state: SensorState) {
        self.states.insert(sensor_tag, state);
    }

    pub fn _get_state(&self, sensor_tag: &str) -> Option<&SensorState> {
        self.states.get(sensor_tag)
    }

    pub fn get_state_mut(&mut self, sensor_tag: &str) -> Option<&mut SensorState> {
        self.states.get_mut(sensor_tag)
    }

    // Convenience methods
    pub fn is_triggered(&self, sensor_tag: &str) -> bool {
        self.states
            .get(sensor_tag)
            .map_or(false, |state| state.triggered)
    }

    pub fn set_triggered(&mut self, sensor_tag: &str, triggered: bool) {
        if let Some(state) = self.get_state_mut(sensor_tag) {
            state.last_triggered = state.triggered;
            state.triggered = triggered;
            state.changed = state.triggered != state.last_triggered;
        }
    }

    pub fn _has_changed(&self, sensor_tag: &str) -> bool {
        self.states
            .get(sensor_tag)
            .map_or(false, |state| state.changed)
    }

    // Method to clear the changed flag after processing
    pub fn clear_changed(&mut self, sensor_tag: &str) {
        if let Some(state) = self.get_state_mut(sensor_tag) {
            state.changed = false;
        }
    }

    // Get all sensors that have changed
    pub fn get_changed_sensors(&self) -> Vec<String> {
        self.states
            .iter()
            .filter(|(_, state)| state.changed)
            .map(|(tag, _)| tag.clone())
            .collect()
    }
}

// >>> Bundles <<<
#[derive(Bundle)]
pub struct SensorBundle {
    sensor: Sensor,
    collider: Collider,
    collider_sensor: bevy_rapier2d::geometry::Sensor,
    active_events: ActiveEvents,
    sprite: Sprite,
    transform: Transform,
}

impl Sensor {
    /// Creates a sensor to detect specific objects based on their component type.
    ///
    /// # Parameters
    /// * `sensor_tag` - Unique identifier for the sensor
    /// * `modbus_address` - Modbus address
    /// * `sensor_item` - TypeId of the component the sensor should detect
    /// * `position` - Position of the sensor
    /// * `color` - Color of the sensor
    ///
    /// # Return
    /// The sensor bundle
    pub fn new(
        sensor_tag: String,
        modbus_address: u16,
        sensor_item: TypeId,
        position: Vec2,
        color: Color,
    ) -> SensorBundle {
        SensorBundle {
            sensor: Sensor {
                sensor_tag,
                modbus_address,
                sensor_item,
            },
            collider: Collider::cuboid(10.0, 10.0),
            collider_sensor: bevy_rapier2d::geometry::Sensor,
            sprite: Sprite::from_color(color, Vec2::new(20.0, 20.0)),
            active_events: ActiveEvents::COLLISION_EVENTS,
            transform: Transform::from_translation(position.extend(0.0)),
        }
    }
}

pub fn register_sensors(
    query: Query<&Sensor, Added<Sensor>>,
    mut global_state: ResMut<GlobalSensorState>,
    modbus_state: Res<ModbusState>,
) {
    for sensor in query.iter() {
        let initial_state = SensorState::default();
        global_state.add_sensor(sensor.sensor_tag.clone(), initial_state.clone());

        if let Ok(mut discretes) = modbus_state.discrete_inputs.lock() {
            discretes.insert(sensor.modbus_address, initial_state.triggered);

            #[cfg(debug_assertions)]
            info!(
                "Initialized {}'s Modbus\n\tAddress: {:x?}\n\tInitial Value: {}",
                sensor.sensor_tag, sensor.modbus_address, initial_state.triggered
            );
        } else {
            warn!(
                "Failed to lock Modbus discrete inputs for sensor {}",
                sensor.sensor_tag
            );
        }

        info!("Registered sensor: {}", sensor.sensor_tag);
    }
}

pub fn handle_sensor_feedback_prefiltered(
    mut collision_events: EventReader<CollisionEvent>,

    sensor_query: Query<&Sensor>,
    bottle_query: Query<(), With<Bottle>>,
    ball_query: Query<Entity, With<Ball>>,

    mut global_state: ResMut<GlobalSensorState>,
    mut conveyor_state: ResMut<ConveyorState>,
    mut valve_state: ResMut<ValveState>,
) {
    let bottle_type_id = TypeId::of::<Bottle>();
    let ball_type_id = TypeId::of::<Ball>();

    for collision_event in collision_events.read() {
        let (sensor_entity, other_entity, is_started) = match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if sensor_query.contains(*e1) {
                    (*e1, *e2, true)
                } else if sensor_query.contains(*e2) {
                    (*e2, *e1, true)
                } else {
                    continue;
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                if sensor_query.contains(*e1) {
                    (*e1, *e2, false)
                } else if sensor_query.contains(*e2) {
                    (*e2, *e1, false)
                } else {
                    continue;
                }
            }
        };

        // Handle sensor logic
        let sensor = sensor_query.get(sensor_entity).unwrap();

        if sensor.sensor_item == bottle_type_id && bottle_query.contains(other_entity) {
            global_state.set_triggered(&sensor.sensor_tag, is_started);
            if is_started {
                info!("Sensor {} triggered by bottle!", sensor.sensor_tag);
                conveyor_state.is_running = false;
                valve_state.is_open = true;
            } else {
                info!(
                    "Sensor {} no longer triggered by bottle!",
                    sensor.sensor_tag
                );
            }
        } else if sensor.sensor_item == ball_type_id && ball_query.contains(other_entity) {
            global_state.set_triggered(&sensor.sensor_tag, is_started);
            if is_started {
                info!("Sensor {} triggered by ball!", sensor.sensor_tag);
                valve_state.is_open = false;
                conveyor_state.is_running = true;
            } else {
                info!("Sensor {} no longer triggered by ball!", sensor.sensor_tag);
            }
        }
    }
}

// >>> Modbus Synchronization <<<
pub fn sync_sensors_to_modbus(
    sensors: Query<&Sensor>,
    mut global_state: ResMut<GlobalSensorState>,
    modbus_state: Res<ModbusState>,
) {
    let changed_sensors = global_state.get_changed_sensors();

    if !changed_sensors.is_empty() {
        if let Ok(mut discretes) = modbus_state.discrete_inputs.lock() {
            for sensor_tag in &changed_sensors {
                if let Some(sensor) = sensors.iter().find(|s| s.sensor_tag == *sensor_tag) {
                    let is_triggered = global_state.is_triggered(sensor_tag);
                    discretes.insert(sensor.modbus_address, is_triggered);

                    // Clear the changed flag after processing
                    global_state.clear_changed(sensor_tag);

                    info!("Updated {}'s Modbus state to: {}", sensor_tag, is_triggered)
                }
            }
        }
    }
}

// >>> Plugin <<<
pub struct SensorPlugin;

impl Plugin for SensorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalSensorState::default())
            .add_systems(
                Update,
                (
                    register_sensors,
                    handle_sensor_feedback_prefiltered,
                    sync_sensors_to_modbus,
                ),
            );
    }
}
