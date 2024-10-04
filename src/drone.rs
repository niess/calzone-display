use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy_rapier3d::prelude::*;
use crate::geometry::{GeometrySet, RootVolume, Volume};
use crate::ui::{Meters, TargetEvent};
use std::f32::consts::PI;


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
                on_transform,
            ));
    }
}

#[derive(Component)]
pub struct Drone {
    velocity: f32,
    meters: Meters,
}

#[derive(Component)]
struct DroneCamera;

fn spawn_drone(
    mut commands: Commands,
    query: Query<&Volume, With<RootVolume>>,
) {
    let drone = Drone::new(&mut commands);
    let root = query.single();
    commands
        .spawn(drone)
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
        });
}

fn on_mouse_motion(
    mut motions: EventReader<MouseMotion>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Drone>>,
) {
    if !buttons.pressed(MouseButton::Right) {
        return
    }

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
    mut commands: Commands,
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
    if keyboard_input.pressed(KeyCode::Space) {
        direction += *transform.up();
    }
    if keyboard_input.pressed(KeyCode::KeyC) {
        direction += *transform.down();
    }
    if keyboard_input.pressed(KeyCode::KeyE) {
        drone.velocity = (drone.velocity * 1.05).min(Drone::VELOCITY_MAX);
        drone.meters.update_speed(drone.velocity, &mut commands);
    }
    if keyboard_input.pressed(KeyCode::KeyQ) {
        drone.velocity = (drone.velocity * 0.95).max(Drone::VELOCITY_MIN);
        drone.meters.update_speed(drone.velocity, &mut commands);
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

fn on_transform(
    mut commands: Commands,
    query: Query<(&Drone, &Transform), Changed<Transform>>,
) {
    if query.is_empty() {
        return
    }
    let (drone, transform) = query.single();
    drone.meters.update_transform(transform, &mut commands);
}

impl Drone {
    const FOV_MIN: f32 = PI / 40.0;
    const FOV_MAX: f32 = PI / 2.0;

    const VELOCITY_MIN: f32 = 0.01;
    const VELOCITY_MAX: f32 = 1000.0;

    fn new(commands: &mut Commands) -> Self {
        let velocity = 1.0;
        let meters = Meters::new(commands);
        Self { velocity, meters }
    }
}
