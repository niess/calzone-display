use bevy::prelude::*;
use bevy::color::palettes::tailwind;
use bevy::input::mouse::MouseMotion;
use bevy_stl::StlPlugin;


pub struct ViewerPlugin;

impl Plugin for ViewerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(StlPlugin)
            .add_systems(Startup, (
                spawn_geometry,
                spawn_lights,
                spawn_viewer,
            ))
            .add_systems(Update, (
                translate_viewer,
                rotate_viewer,
            ));
    }
}

#[derive(Component)]
struct Viewer {
    speed: f32,
}

fn spawn_geometry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    commands.spawn(PbrBundle {
        mesh: asset_server.load("foreground.stl"),
        material: materials.add(Color::WHITE),
        ..default()
    });
}

fn spawn_lights(mut commands: Commands) {
    commands.spawn(
        PointLightBundle {
            point_light: PointLight {
                color: Color::from(tailwind::ROSE_300),
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(400.0, 400.0, 800.0),
            ..default()
        }
    );
}

fn spawn_viewer(mut commands: Commands) {
    commands
        .spawn((
            Viewer { speed: 100.0 },
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 400.0),
                ..default()
            },
        ))
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

fn rotate_viewer(
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Viewer>>,
) {
    let mut transform = query.single_mut();
    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;
        // The rrder of rotations is important.
        // See: https://gamedev.stackexchange.com/a/136175/103059.
        transform.rotate_z(yaw);
        transform.rotate_local_x(pitch);
    }
}

fn translate_viewer(
    mut query: Query<(&mut Transform, &mut Viewer)>,
    time: Res<Time>,
) {
    let (mut transform, viewer) = query.single_mut();
    let direction = transform.local_z();
    transform.translation -= direction * viewer.speed * time.delta_seconds();
}
