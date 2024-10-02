use bevy::prelude::*;
use bevy::color::palettes::css::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;
use super::geometry::{GeometrySet, RootVolume, Volume};
use super::ui::TargetEvent;


pub struct DronePlugin;

impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_drone.after(GeometrySet))
            .add_systems(Update, (
                on_mouse_motion,
                on_mouse_wheel,
                on_keyboard,
                on_target,
            ));
    }
}

#[derive(Component)]
pub struct Drone {
    velocity: f32,
}

#[derive(Component)]
struct DroneCamera;

fn spawn_drone(
    mut commands: Commands,
    query: Query<&Volume, With<RootVolume>>,
) {
    let root = query.single();
    commands
        .spawn(Drone::default())
        .insert(SpatialBundle {
            transform: root.target(),
            ..default()
        })
        .insert(RigidBody::KinematicVelocityBased)
        .insert(AdditionalMassProperties::Mass(1.0))
        .insert(Velocity::default())
        .with_children(|parent| {
            parent.spawn((
                DroneCamera,
                Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 70.0_f32.to_radians(),
                        ..default()
                    }.into(),
                    ..default()
                },
            ));
            for x in [-1.0, 1.0] {
                parent.spawn(SpotLightBundle {
                    transform: Transform::from_xyz(x, 0.0, 0.0)
                        .looking_at(Vec3::new(x, 0.0, -1.0), Vec3::Y),
                    spot_light: SpotLight {
                        intensity: light_consts::lux::DIRECT_SUNLIGHT,
                        range: 300.0,
                        color: ALICE_BLUE.into(),
                        shadows_enabled: true,
                        inner_angle: PI / 6.0,
                        outer_angle: PI / 3.0,
                        ..default()
                    },
                    ..default()
                });
            }
        });
}

fn on_mouse_motion(
    mut motions: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Drone>>,
) {
    let (mut transform, mut velocity) = query.single_mut();
    for motion in motions.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;
        transform.rotate_z(yaw);
        transform.rotate_local_x(pitch);
    }
    let magnitude = velocity.linvel.length();
    if magnitude != 0.0 {
        velocity.linvel = magnitude * transform.forward();
    }
}

fn on_mouse_wheel(
    mut wheels: EventReader<MouseWheel>,
    mut camera: Query<&mut Projection, With<DroneCamera>>,
) {
    if let Projection::Perspective(perspective) = camera.single_mut().into_inner() {
        let mut scroll = 0.0;
        for wheel in wheels.read() {
            scroll += wheel.y;
        }
        perspective.fov = (perspective.fov - 0.05 * scroll)
            .clamp(Drone::FOV_MIN, Drone::FOV_MAX);
    }
}

fn on_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut Drone)>,
) {
    let (transform, mut velocity, mut drone) = query.single_mut();

    let mut direction = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += *transform.forward();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction += *transform.back();
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction += *transform.left();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += *transform.right();
    }
    if keyboard_input.pressed(KeyCode::KeyE) {
        direction += *transform.up();
    }
    if keyboard_input.pressed(KeyCode::KeyQ) {
        direction += *transform.down();
    }
    if keyboard_input.pressed(KeyCode::NumpadAdd) {
        drone.velocity = (drone.velocity * 1.05).min(Drone::VELOCITY_MAX);
    }
    if keyboard_input.pressed(KeyCode::NumpadSubtract) {
        drone.velocity = (drone.velocity * 0.95).max(Drone::VELOCITY_MIN);
    }

    velocity.linvel = drone.velocity * direction;
}

fn on_target(
    mut events: EventReader<TargetEvent>,
    mut volumes: Query<&mut Volume>,
    mut transform: Query<(&mut Transform, &mut Velocity), With<Drone>>,
) {
    for event in events.read() {
        let volume = volumes.get_mut(event.0).unwrap();
        let (mut transform, mut velocity) = transform.single_mut();
        *transform = volume.target();
        let magnitude = velocity.linvel.length();
        if magnitude != 0.0 {
            velocity.linvel = magnitude * transform.forward();
        }
    }
}

impl Drone {
    const FOV_MIN: f32 = PI / 10.0;
    const FOV_MAX: f32 = 2.0 * PI / 3.0;

    const VELOCITY_MIN: f32 = 0.1;
    const VELOCITY_MAX: f32 = 100.0;
}

impl Default for Drone {
    fn default() -> Self {
        Self { velocity: 1.0 }
    }
}
