use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy_rapier3d::prelude::*;


pub struct DronePlugin;

impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_viewer)
            .add_systems(Update, (
                on_mouse,
                on_keybord,
            ));
    }
}

#[derive(Component)]
struct Drone;

fn spawn_viewer(mut commands: Commands) {
    commands
        .spawn(Drone)
        .insert(SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 400.0),
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
            parent.spawn(
                Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 70.0_f32.to_radians(),
                        ..default()
                    }.into(),
                    ..default()
                }
            );
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
    if keyboard_input.just_pressed(KeyCode::KeyW) {
        external_force.force = 50.0 * -transform.local_z();
    } else if keyboard_input.just_released(KeyCode::KeyW) {
        external_force.force = Vect::ZERO;
    }
}
