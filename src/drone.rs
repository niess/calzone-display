use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use crate::app::{AppState, Removable};
use crate::geometry::{GeometrySet, RootVolume, Volume};
use crate::sky::{SkyBundle, SkyCamera};
use crate::ui::{Meters, TargetEvent};


pub struct DronePlugin;

impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Display), Drone::spawn.after(GeometrySet))
            .add_systems(Update, (
                on_mouse_button,
                on_mouse_motion,
                on_mouse_wheel,
                on_keyboard,
                on_target,
                on_transform,
            ).run_if(in_state(AppState::Display)));
    }
}

#[derive(Component)]
pub struct Drone {
    velocity: f32,
    meters: Meters,
    cursor: Option<Vec2>,
}

#[derive(Component)]
pub struct DroneCamera;

impl Drone {
    pub fn spawn(
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
            .insert(Removable)
            .with_children(|parent| {
                parent.spawn((
                    DroneCamera,
                    Camera3dBundle {
                        projection: PerspectiveProjection {
                            fov: Drone::FOV_MAX,
                            ..default()
                        }.into(),
                        ..default()
                    },
                ));
                parent.spawn(SkyBundle::new(Drone::FOV_MAX));
            });
    }
}

fn on_mouse_button(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drone: Query<&mut Drone>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if window.is_empty() {
        return; // The window might have been closed.
    }
    let mut drone = drone.single_mut();
    let mut window = window.single_mut();

    if buttons.just_pressed(MouseButton::Right) {
        if let Some(position) = window.cursor_position() {
            drone.cursor = Some(position);
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
        }
    }

    if buttons.just_released(MouseButton::Right) {
        if let Some(position) = drone.cursor {
            window.set_cursor_position(Some(position));
        }
        drone.cursor = None;
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

fn on_mouse_motion(
    mut motions: EventReader<MouseMotion>,
    mut query: Query<(&Drone, &mut Transform, &mut Velocity)>,
    mut camera: Query<&mut Projection, With<DroneCamera>>,
) {
    let (drone, mut transform, mut velocity) = query.single_mut();
    if drone.cursor.is_none() {
        return
    }

    // Compute the total motion.
    let (x, y) = {
        let mut x = 0.0;
        let mut y = 0.0;
        for motion in motions.read() {
            x += motion.delta.x;
            y += motion.delta.y;
        }
        (x, y)
    };
    if (x == 0.0) && (y == 0.0) {
        return
    }

    let mut zoom = 1.0;
    if let Projection::Perspective(perspective) = camera.single_mut().into_inner() {
        zoom = Drone::FOV_MAX / perspective.fov;
    }
    let yaw = -0.003 * x / zoom;
    let pitch = -0.002 * y / zoom;

    // Rotate the camera and the velocity vector.
    let r0i = if velocity.linvel == Vec3::ZERO {
        Some(transform.rotation.inverse())
    } else {
        None
    };
    transform.rotate_z(yaw);
    transform.rotate_local_x(pitch);
    if let Some(r0i) = r0i {
        velocity.linvel = (transform.rotation * r0i) * velocity.linvel;
    }
}

fn on_mouse_wheel(
    mut wheels: EventReader<MouseWheel>,
    mut camera: Query<&mut Projection, (With<DroneCamera>, Without<SkyCamera>)>,
    mut sky: Query<&mut Projection, (With<SkyCamera>, Without<DroneCamera>)>,
    drone: Query<&Drone>,
    mut commands: Commands,
) {
    if let Projection::Perspective(perspective) = camera.single_mut().into_inner() {
        let mut scroll = 0.0;
        for wheel in wheels.read() {
            scroll += wheel.y;
        }
        perspective.fov = (perspective.fov * (-0.05 * scroll).exp())
            .clamp(Drone::FOV_MIN, Drone::FOV_MAX);
        drone.single().meters.update_zoom(Drone::FOV_MAX / perspective.fov, &mut commands);

        if let Projection::Perspective(sky) = sky.single_mut().into_inner() {
            sky.fov = perspective.fov;
        }
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
    const FOV_MIN: f32 = 0.012217;
    pub const FOV_MAX: f32 = 1.2217;

    const VELOCITY_MIN: f32 = 0.01;
    const VELOCITY_MAX: f32 = 1000.0;

    fn new(commands: &mut Commands) -> Self {
        let velocity = 1.0;
        let meters = Meters::new(commands);
        meters.update_speed(velocity, commands);
        meters.update_zoom(1.0, commands);
        let cursor = None;
        Self { velocity, meters, cursor }
    }
}
