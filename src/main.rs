// main.rs
// Copyright (C) 2025 deciphr
// SPDX-License-Identifier: GPL-3.0-or-later


use bevy::prelude::*;

mod bottle;
mod conveyor;
mod environment;
mod modbus;
mod sensor;
mod valve;

use bottle::BottlePlugin;
use conveyor::ConveyorPlugin;
use environment::setup_environment;
use modbus::{ModbusPlugin, ModbusState};
use sensor::SensorPlugin;
use valve::ValvePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(ModbusPlugin)
        .add_plugins(ConveyorPlugin)
        .add_plugins(BottlePlugin)
        .add_plugins(ValvePlugin)
        .add_plugins(SensorPlugin)
        .init_resource::<ModbusState>()
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_environment)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}
