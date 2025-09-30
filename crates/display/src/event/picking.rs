use bevy::prelude::*;
use bevy::math::Vec3A;
use bevy::math::bounding::{BoundingSphere, RayCast3d};
use bevy::window::PrimaryWindow;
use crate::app::AppState;
use crate::ui::{UiEvent, UiRoot};
use super::{EventCamera, Track, Vertex, VertexSize};


pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cursor_selection.run_if(in_state(AppState::Display)));
    }
}

fn cursor_selection(
    window: Query<&mut Window, With<PrimaryWindow>>,
    uis: Query<(&ComputedNode, &GlobalTransform), With<UiRoot>>,
    camera: Query<(&Camera, &GlobalTransform), With<EventCamera>>,
    tracks: Query<&Track>,
    vertices: Query<(&Vertex, &VertexSize, &Transform, &ChildOf)>,
    ui_event: Query<Entity, With<UiEvent>>,
    mut commands: Commands,
) {
    if !ui_event.is_empty() {
        commands.entity(ui_event.single().unwrap()).despawn();
    }
    if window.is_empty() || camera.is_empty() || tracks.is_empty() || vertices.is_empty() {
        return
    }

    let Some(cursor) = window.single().unwrap().cursor_position() else { return };
    for (node, transform) in uis.iter() {
        let rect = Rect::from_center_size(
            transform.translation().xy(),
            node.size,
        );
        if rect.contains(cursor) {
            return
        }
    }

    let (camera, camera_transform) = camera.single().unwrap();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else { return };

    let mut matches = Vec::new();
    for (vertex, size, transform, childof) in vertices.iter() {
        let bounding_sphere = BoundingSphere {
            center: Vec3A::from(transform.translation),
            sphere: Sphere { radius: size.0 },
        };
        let raycast = RayCast3d::from_ray(ray, f32::MAX);
        if let Some(_) = raycast.sphere_intersection_at(&bounding_sphere) {
            let track = tracks.get(childof.parent()).unwrap();
            matches.push((track, vertex))
        }
    }
    if matches.is_empty() {
        return
    }

    UiEvent::spawn_info(&mut commands, cursor, matches);
}
