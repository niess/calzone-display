use bevy::prelude::*;
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
            min = min.min(*vertex.position.as_vec3());
            max = max.max(*vertex.position.as_vec3());
        }
        let half_width = 0.5 * (max - min);
        let &[mut dx, mut dy, _] = half_width.as_ref();
        if dx.abs() < Drone::NEAR {
            dx = Drone::NEAR.copysign(dx);
        }
        if dy.abs() < Drone::NEAR {
            dy = Drone::NEAR.copysign(dy);
        }
        let origin = 0.5 * (min + max);
        let start_position = origin + Vec3::new(-1.5 * dx, -1.5 * dy, 0.0);
        Transform::from_translation(start_position)
            .looking_at(origin, Vec3::Z)
    }
}

pub(crate) trait AsVec3 {
    fn as_vec3(&self) -> &Vec3;
}

impl AsVec3 for data::event::Vec3 {
    #[inline]
    fn as_vec3(&self) -> &Vec3 {
        unsafe { std::mem::transmute(self) }
    }
}
