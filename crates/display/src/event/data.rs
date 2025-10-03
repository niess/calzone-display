use bevy::prelude::*;
use crate::world_to_view;
use crate::drone::Drone;
use std::sync::Mutex;

pub(crate) use data::event::{Events, Event, Track, Vertex};

// ===============================================================================================
//
// Monte Carlo event data.
//
// ===============================================================================================

static EVENTS: Mutex<Option<Events>> = Mutex::new(None);

pub(crate) fn take() -> Option<Events> {
    EVENTS.lock().unwrap().take()
}

pub fn set(events: Events) {
    *EVENTS.lock().unwrap() = Some(events);
}

pub(crate) trait Target {
    fn target(&self) -> Transform;
}

impl Target for Track {
    fn target(&self) -> Transform {
        let mut min = Vec3::MAX;
        let mut max = Vec3::MIN;
        for vertex in self.vertices.iter() {
            min = min.min(vertex.position.to_view());
            max = max.max(vertex.position.to_view());
        }
        let half_width = 0.5 * (max - min);
        let &[mut dx, _, mut dz] = half_width.as_ref();
        if dx.abs() < Drone::NEAR {
            dx = Drone::NEAR.copysign(dx);
        }
        if dz.abs() < Drone::NEAR {
            dz = Drone::NEAR.copysign(dz);
        }
        let origin = 0.5 * (min + max);
        let start_position = origin + Vec3::new(-1.5 * dx, 0.0, -1.5 * dz);
        Transform::from_translation(start_position)
            .looking_at(origin, Vec3::Y)
    }
}

pub(crate) trait ToView {
    fn to_view(&self) -> Vec3;
}

impl ToView for data::event::Vec3 {
    #[inline]
    fn to_view(&self) -> Vec3 {
        let p = Vec3::new(self.x, self.y, self.z);
        world_to_view().transform_point3(p)
    }
}
