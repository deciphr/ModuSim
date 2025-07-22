// bottle.rs
// Copyright (C) 2025 deciphr
// SPDX-License-Identifier: GPL-3.0-or-later

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::conveyor::ConveyorState;

// >>> Constants <<<
pub const BOTTLE_HEIGHT: f32 = 100.0;
pub const BOTTLE_WIDTH: f32 = 50.0;
const BOTTLE_THICKNESS: f32 = 5.0;
const BOTTLE_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);

// >>> Components <<<
#[derive(Component)]
pub struct Bottle;

#[derive(Component)]
pub struct BottlePosition(pub Vec2);

// >>> Resources <<<
#[derive(Resource)]
pub struct BottleSpawner {
    timer: Timer,
}

impl Default for BottleSpawner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

// >>> Bundles <<<
#[derive(Bundle)]
pub struct BottleBundle {
    bottle: Bottle,
    position: BottlePosition,
    transform: Transform,
    global_transform: GlobalTransform,
    rigid_body: RigidBody,
    collider: Collider,
    ccd: Ccd,
    sleeping: Sleeping,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
}

impl Bottle {
    pub fn new(position: Vec2) -> impl Bundle {
        BottleBundle {
            bottle: Bottle,
            position: BottlePosition(position),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::compound(vec![
                (
                    Vec2::new((-BOTTLE_WIDTH / 2.0) + position.x, position.y),
                    0.0,
                    Collider::cuboid(BOTTLE_THICKNESS / 2.0, BOTTLE_HEIGHT / 2.0),
                ),
                (
                    Vec2::new(position.x, (-BOTTLE_HEIGHT / 2.0) + position.y),
                    0.0,
                    Collider::cuboid(BOTTLE_WIDTH / 2.0, BOTTLE_THICKNESS / 2.0),
                ),
                (
                    Vec2::new((BOTTLE_WIDTH / 2.0) + position.x, position.y),
                    0.0,
                    Collider::cuboid(BOTTLE_THICKNESS / 2.0, BOTTLE_HEIGHT / 2.0),
                ),
            ]),
            ccd: Ccd::enabled(),
            sleeping: Sleeping::disabled(),
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::default(),
        }
    }
}

// >>> Systems <<<
pub fn add_bottle_sprite(
    mut commands: Commands,
    query: Query<(Entity, &BottlePosition), Added<Bottle>>,
) {
    for (entity, BottlePosition(position)) in query.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Transform::from_translation(
                    Vec2::new(position.x - BOTTLE_WIDTH / 2.0, position.y).extend(0.0),
                ),
                GlobalTransform::default(),
                Sprite {
                    color: BOTTLE_COLOR,
                    custom_size: Some(Vec2::new(BOTTLE_THICKNESS, BOTTLE_HEIGHT)),
                    ..Default::default()
                },
            ));
            parent.spawn((
                Transform::from_translation(
                    Vec2::new(position.x, position.y - BOTTLE_HEIGHT / 2.0).extend(0.0),
                ),
                GlobalTransform::default(),
                Sprite {
                    color: BOTTLE_COLOR,
                    custom_size: Some(Vec2::new(BOTTLE_WIDTH, BOTTLE_THICKNESS)),
                    ..Default::default()
                },
            ));
            parent.spawn((
                Transform::from_translation(
                    Vec2::new(position.x + BOTTLE_WIDTH / 2.0, position.y).extend(0.0),
                ),
                GlobalTransform::default(),
                Sprite {
                    color: BOTTLE_COLOR,
                    custom_size: Some(Vec2::new(BOTTLE_THICKNESS, BOTTLE_HEIGHT)),
                    ..Default::default()
                },
            ));
        });
    }
}

pub fn spawn_bottle_on_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    window: Query<&Window>,
) {
    let window = window.single();
    let width = window.unwrap().resolution.width();

    if keyboard.just_pressed(KeyCode::Enter) {
        commands
            .spawn(Bottle::new(Vec2::new(
                -width / 2.0 + BOTTLE_WIDTH / 2.0,
                BOTTLE_HEIGHT,
            )))
            .insert(SolverGroups::new(Group::GROUP_1, Group::GROUP_2));
        info!("Spawned a new bottle!");
    }
}

pub fn spawn_bottles(
    time: Res<Time>,
    mut commands: Commands,
    window: Query<&Window>,
    mut bottle_spawner: ResMut<BottleSpawner>,
    conveyor_state: Res<ConveyorState>,
) {
    if !conveyor_state.is_running {
        return;
    }

    bottle_spawner.timer.tick(time.delta());
    if bottle_spawner.timer.just_finished() {
        let window = window.single();
        let width = window.unwrap().resolution.width();

        commands
            .spawn(Bottle::new(Vec2::new(
                -width / 2.0 + BOTTLE_WIDTH / 2.0,
                BOTTLE_HEIGHT,
            )))
            .insert(SolverGroups::new(Group::GROUP_1, Group::GROUP_2));
    }
}

// >>> Plugin <<<
pub struct BottlePlugin;

impl Plugin for BottlePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BottleSpawner::default())
        .add_systems(
            Update,
            (spawn_bottles, spawn_bottle_on_input, add_bottle_sprite),
        );
    }
}
