use bevy::prelude::*;
use bevy::math::Vec3A;
use bevy::math::bounding::{BoundingSphere, RayCast3d};
use bevy::window::PrimaryWindow;
use crate::app::AppState;
use super::{EventCamera, Track, Vertex, VertexSize};


// XXX Check for any masking by an UI element.
// XXX Sort tracks, vertices etc. and pretty display using an UI window (with a trigger/event).
// XXX Indicate vertex medium.

pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cursor_selection.run_if(in_state(AppState::Display)));
    }
}

#[derive(Event)]
pub struct PickingEvent (Vec<(Track, Vec<Vertex>)>);

#[derive(Component)]
struct PickingText;

fn cursor_selection(
    window: Query<&mut Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<EventCamera>>,
    tracks: Query<&Track>,
    vertices: Query<(&Vertex, &VertexSize, &Transform, &Parent)>,
    text: Query<Entity, With<PickingText>>,
    mut commands: Commands,
) {
    if !text.is_empty() {
        commands.entity(text.single()).despawn_recursive();
    }
    if window.is_empty() || camera.is_empty() || tracks.is_empty() || vertices.is_empty() {
        return
    }

    let Some(cursor) = window.single().cursor_position() else { return };
    let (camera, camera_transform) = camera.single();
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor) else { return };

    let mut matches = Vec::new();
    for (vertex, size, transform, parent) in vertices.iter() {
        let bounding_sphere = BoundingSphere {
            center: Vec3A::from(transform.translation),
            sphere: Sphere { radius: size.0 },
        };
        let raycast = RayCast3d::from_ray(ray, f32::MAX);
        if let Some(_) = raycast.sphere_intersection_at(&bounding_sphere) {
            let track = tracks.get(parent.get()).unwrap();
            matches.push((track, vertex))
        }
    }
    if matches.is_empty() {
        return
    }

    let text: Vec<_> = matches
        .iter()
        .map(|vertex| format!("{:?}", vertex))
        .collect();
    let text = text.join("\n");

    commands.spawn((
        PickingText,
        TextBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(cursor.x + 12.0),
                top: Val::Px(cursor.y + 12.0),
                ..default()
            },
            ..default()
        },
    ));
}
