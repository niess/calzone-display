use bevy::prelude::*;
use bevy::math::Vec3A;
use bevy::math::bounding::{BoundingSphere, RayCast3d};
use bevy::window::PrimaryWindow;
use crate::app::AppState;
use super::{EventCamera, VertexSize};


pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_hits.run_if(in_state(AppState::Display)));
    }
}

fn update_hits(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<EventCamera>>,
    vertices: Query<(Entity, &Transform, &VertexSize)>,
) {
    if window.is_empty() || camera.is_empty() {
        return
    }

    let Some(cursor) = window.single().cursor_position() else { return };
    let (camera, camera_transform) = camera.single();
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor) else { return };

    for (vertex, transform, size) in vertices.iter() {
        let bounding_sphere = BoundingSphere {
            center: Vec3A::from(transform.translation),
            sphere: Sphere { radius: size.0 },
        };
        let raycast = RayCast3d::from_ray(ray, f32::MAX);
        if let Some(distance) = raycast.sphere_intersection_at(&bounding_sphere) {
            info!("XXX vertex = {}", vertex.index());
        }
    }
}
