use bevy::prelude::*;
use bevy::core_pipeline::{bloom::Bloom, tonemapping::Tonemapping};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::render::camera::Exposure;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use crate::app::{AppState, Removable};
use crate::event::{EventBundle, EventCamera};
use crate::geometry::{GeometrySet, RootVolume, Volume};
use crate::ui::{Meters, TextInputSet, TextInputState, UiRoot};


pub struct DronePlugin;

impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<TargetEvent>()
            .add_systems(OnEnter(AppState::Display), Drone::spawn.after(GeometrySet))
            .add_systems(Update, (
                on_mouse_button,
                on_mouse_motion,
                on_mouse_wheel,
                on_keyboard
                    .after(TextInputSet)
                    .run_if(in_state(TextInputState::Inactive)),
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

#[derive(Event)]
pub struct TargetEvent(pub Transform);

impl Drone {
    pub fn spawn(
        mut commands: Commands,
        query: Query<&Volume, With<RootVolume>>,
    ) -> Result<()> {
        let drone = Drone::new(&mut commands);
        let root = query.single()?;
        commands
            .spawn(drone)
            .insert(root.target())
            .insert(Visibility::default())
            .insert(RigidBody::KinematicVelocityBased)
            .insert(Velocity::default())
            .insert(Removable)
            .with_children(|parent| {
                parent.spawn((
                    DroneCamera,
                    Camera3d::default(),
                    Camera {
                        hdr: true,
                        ..default()
                    },
                    Projection::Perspective(PerspectiveProjection {
                        fov: Drone::FOV_MAX,
                        near: Drone::NEAR,
                        ..default()
                    }),
                    Bloom::NATURAL,
                    Exposure::SUNLIGHT,
                    Tonemapping::AgX,
                    AmbientLight {
                        brightness: 0.0, // disable default.
                        ..default()
                    },
                ));
                parent.spawn(EventBundle::new(Drone::FOV_MAX));
            });
        Ok(())
    }
}

fn on_mouse_button(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drone: Query<&mut Drone>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) -> Result<()> {
    if window.is_empty() {
        return Ok(()); // The window might have been closed.
    }
    let mut drone = drone.single_mut()?;
    let mut window = window.single_mut()?;

    if buttons.just_pressed(MouseButton::Right) {
        if let Some(position) = window.cursor_position() {
            drone.cursor = Some(position);
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        }
    }

    if buttons.just_released(MouseButton::Right) {
        if let Some(position) = drone.cursor {
            window.set_cursor_position(Some(position));
        }
        drone.cursor = None;
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
    Ok(())
}

fn on_mouse_motion(
    mut motions: EventReader<MouseMotion>,
    mut query: Query<(&Drone, &mut Transform, &mut Velocity)>,
    mut camera: Query<&mut Projection, With<DroneCamera>>,
) -> Result<()> {
    let (drone, mut transform, mut velocity) = query.single_mut()?;
    if drone.cursor.is_none() {
        return Ok(())
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
        return Ok(())
    }

    let mut zoom = 1.0;
    if let Projection::Perspective(perspective) = camera.single_mut()?.into_inner() {
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
    transform.rotate_y(yaw);
    transform.rotate_local_x(pitch);
    if let Some(r0i) = r0i {
        velocity.linvel = (transform.rotation * r0i) * velocity.linvel;
    }
    Ok(())
}

fn on_mouse_wheel(
    mut wheels: EventReader<MouseWheel>,
    mut camera: Query<&mut Projection, (
        With<DroneCamera>, Without<EventCamera>,
    )>,
    event_camera: Query<&mut Projection, (
        With<EventCamera>, Without<DroneCamera>,
    )>,
    drone: Query<&Drone>,
    uis: Query<(&ComputedNode, &GlobalTransform), With<UiRoot>>,
    window: Query<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
) -> Result<()> {
    let Ok(window) = window.single() else { return Ok(()) };
    let Some(cursor) = window.cursor_position() else { return Ok(()) };
    let mut scroll = 0.0;
    for wheel in wheels.read() {
        scroll += wheel.y;
    }
    if scroll == 0.0 {
        return Ok(())
    }
    for (node, transform) in uis.iter() {
        let rect = Rect::from_center_size(
            transform.translation().xy(),
            node.size,
        );
        if rect.contains(cursor) {
            return Ok(())
        }
    }

    if let Projection::Perspective(perspective) = camera.single_mut().unwrap().into_inner() {
        perspective.fov = (perspective.fov * (-0.05 * scroll).exp())
            .clamp(Drone::FOV_MIN, Drone::FOV_MAX);
        update_zoom(
            drone.single().unwrap(), perspective, event_camera, &mut commands
        )?;
    }
    Ok(())
}

fn update_zoom(
    drone: &Drone,
    perspective: &PerspectiveProjection,
    mut event_camera: Query<&mut Projection, (
        With<EventCamera>, Without<DroneCamera>,
    )>,
    commands: &mut Commands,
) -> Result<()> {
    drone.meters.update_zoom(Drone::FOV_MAX / perspective.fov, commands);

    if let Projection::Perspective(event_camera) = event_camera.single_mut()?.into_inner() {
        event_camera.fov = perspective.fov;
    }
    Ok(())
}

fn on_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut Drone)>,
    mut commands: Commands,
) -> Result<()> {
    let (transform, mut velocity, mut drone) = query.single_mut()?;

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
    Ok(())
}

fn on_target(
    mut events: EventReader<TargetEvent>,
    mut drone: Query<(Entity, &Drone, &mut Transform, &mut Velocity)>,
    mut commands: Commands,
    mut drone_camera: Query<&mut Projection, (
        With<DroneCamera>, Without<EventCamera>,
    )>,
    event_camera: Query<&mut Projection, (
        With<EventCamera>, Without<DroneCamera>,
    )>,
) -> Result<()> {
    let mut reset_zoom = false;
    for event in events.read() {
        let (entity, _, mut transform, mut velocity) = drone.single_mut()?;
        commands.entity(entity).insert(RigidBodyDisabled); // Disable rapier phys. before warping.
        *transform = event.0;
        let magnitude = velocity.linvel.length();
        if magnitude != 0.0 {
            velocity.linvel = magnitude * transform.forward();
        }
        reset_zoom = true;
    }

    if reset_zoom {
        if let Projection::Perspective(perspective) = drone_camera.single_mut()?.into_inner() {
            let (_, drone, ..) = drone.single_mut()?;
            perspective.fov = Drone::FOV_MAX;
            update_zoom(drone, perspective, event_camera, &mut commands)?;
        }
    }
    Ok(())
}

fn on_transform(
    mut commands: Commands,
    query: Query<(&Drone, &Transform), Changed<Transform>>,
    disabled: Query<Entity, With<RigidBodyDisabled>>,
) {
    if query.is_empty() {
        return
    }
    if let Ok(entity) = disabled.single() { // Restore Rapier physics.
        commands.entity(entity).remove::<RigidBodyDisabled>();
    }
    let (drone, transform) = query.single().unwrap();
    drone.meters.update_transform(transform, &mut commands);
}

impl Drone {
    const FOV_MIN: f32 = 0.012217;
    pub const FOV_MAX: f32 = 1.2217;

    const VELOCITY_MIN: f32 = 0.01;
    const VELOCITY_MAX: f32 = 1000.0;

    pub const NEAR: f32 = 1E-02;

    fn new(commands: &mut Commands) -> Self {
        let velocity = 1.0;
        let meters = Meters::new(commands);
        meters.update_speed(velocity, commands);
        meters.update_zoom(1.0, commands);
        let cursor = None;
        Self { velocity, meters, cursor }
    }
}
