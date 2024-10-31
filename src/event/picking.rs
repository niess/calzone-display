use bevy::prelude::*;
use bevy::math::Vec3A;
use bevy::math::bounding::{BoundingSphere, RayCast3d};
use bevy_mod_picking::prelude::*;
use bevy_mod_picking::backend::prelude::*;
use crate::app::AppState;
use super::{EventCamera, VertexSize};


pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(DefaultPickingPlugins)
            .add_systems(PreUpdate,
                update_hits
                    .in_set(PickSet::Backend)
                    .run_if(in_state(AppState::Display))
            );
    }
}

fn update_hits(
    rays: Res<RayMap>,
    cameras: Query<(), With<EventCamera>>,
    vertices: Query<(Entity, &Transform, &VertexSize)>,
    mut output_events: EventWriter<PointerHits>,
) {
    for (&id, &ray) in rays.map().iter() {
        let Ok(_) = cameras.get(id.camera) else { continue };
        let mut picks = Vec::new();
        for (vertex, transform, size) in vertices.iter() {
            let bounding_sphere = BoundingSphere {
                center: Vec3A::from(transform.translation),
                sphere: Sphere { radius: size.0 },
            };
            let raycast = RayCast3d::from_ray(ray, f32::MAX);
            if let Some(distance) = raycast.sphere_intersection_at(&bounding_sphere) {
                let data = HitData {
                    camera: id.camera,
                    depth: distance,
                    position: Some(ray.get_point(distance)),
                    normal: None,
                };
                info!("XXX vertex = {}", vertex.index());
                picks.push((vertex, data));
            }
        }
        if !picks.is_empty() {
            output_events.send(PointerHits::new(id.pointer, picks, 0.0));
        }
    }
}
