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
                on_keybord,
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
        .insert(RigidBody::Dynamic)
        .insert(AdditionalMassProperties::Mass(1.0))
        .insert(ExternalForce::default())
        .insert(Damping {
            linear_damping: 1.0,
            angular_damping: 0.0,
        })
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
    mut query: Query<(&mut Transform, &mut ExternalForce), With<Drone>>,
) {
    let (mut transform, mut external_force) = query.single_mut();
    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;
        transform.rotate_z(yaw);
        transform.rotate_local_x(pitch);
    }
    let force = external_force.force.length();
    if force != 0.0 {
        external_force.force = force * -transform.local_z();
    }
}

fn on_keybord(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut ExternalForce), With<Drone>>,
) {
    let (transform, mut external_force) = query.single_mut();

    let mut force = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        force -= *transform.local_z();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        force += *transform.local_z();
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        force -= *transform.local_x();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        force += *transform.local_x();
    }
    if keyboard_input.pressed(KeyCode::KeyE) {
        force += *transform.local_y();
    }
    if keyboard_input.pressed(KeyCode::KeyQ) {
        force -= *transform.local_y();
    }

    const STRENGTH: f32 = 50.0;
    external_force.force = STRENGTH * force;
}

fn on_target(
    mut events: EventReader<TargetEvent>,
    mut volumes: Query<&mut Volume>,
    mut transform: Query<&mut Transform, With<Drone>>,
) {
    for event in events.read() {
        let volume = volumes.get_mut(event.0).unwrap();
        *transform.single_mut() = volume.target(); // XXX Relocate camera?
    }
}
