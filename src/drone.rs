use bevy::prelude::*;
use bevy::color::palettes::css::*;
use bevy::input::mouse::MouseMotion;
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
                on_mouse,
                on_keyboard,
                on_target,
            ));
    }
}

#[derive(Component)]
pub struct Drone;

fn spawn_drone(
    mut commands: Commands,
    query: Query<&Volume, With<RootVolume>>,
) {
    let root = query.single();
    commands
        .spawn(Drone)
        .insert(SpatialBundle {
            transform: root.target(),
            ..default()
        })
        .insert(RigidBody::KinematicVelocityBased)
        .insert(AdditionalMassProperties::Mass(1.0))
        .insert(Velocity::default())
        .with_children(|parent| {
            parent.spawn(Camera3dBundle {
                projection: PerspectiveProjection {
                    fov: 70.0_f32.to_radians(),
                    ..default()
                }.into(),
                ..default()
            });
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

fn on_mouse(
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Drone>>,
) {
    let (mut transform, mut velocity) = query.single_mut();
    for motion in mouse_motion.read() {
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

fn on_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Drone>>,
) {
    let (transform, mut velocity) = query.single_mut();

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

    const STRENGTH: f32 = 1.0;
    velocity.linvel = STRENGTH * direction;
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
