// environment.rs
// Copyright (C) 2025 deciphr
// SPDX-License-Identifier: GPL-3.0-or-later

use std::any::TypeId;
use bevy::prelude::*;

use crate::components::{
    conveyor::Conveyor,
    bottle::{BOTTLE_HEIGHT, Bottle},
    sensor::Sensor,
    valve::{Ball, Valve},
};

pub const CONVEYOR_HEIGHT: f32 = 100.0;

pub fn setup_environment(mut commands: Commands, window: Query<&Window>) {
    let window = window.single().unwrap();
    let width = window.resolution.width();

    // Conveyor
    let conveyor_width: f32 = width / 2.0;
    commands.spawn(Conveyor::new(
        0x0000,
        0x0000,
        Vec2::new(-conveyor_width / 2.0, -150.0),
        conveyor_width,
        CONVEYOR_HEIGHT
    ));

    // Water valve
    commands.spawn(Valve::new(0x0001, 0x0001, Vec2::new(-30.0, 70.0)));

    // Bottle sensor
    commands.spawn(Sensor::new(
        "bottle_sensor".to_string(),
        0x0000,
        TypeId::of::<Bottle>(),
        Vec2::new(0.0, -CONVEYOR_HEIGHT),
        Color::srgb(1.0, 0.0, 0.0),
    ));

    // Water sensor
    commands.spawn(Sensor::new(
        "water_sensor".to_string(),
        0x0001,
        TypeId::of::<Ball>(),
        Vec2::new(0.0, -CONVEYOR_HEIGHT + BOTTLE_HEIGHT),
        Color::srgb(0.0, 0.0, 1.0),
    ));

}
