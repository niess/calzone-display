use bevy::prelude::*;
use crate::drone::Drone;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::Mutex;


// ===============================================================================================
//
// Monte Carlo event data.
//
// ===============================================================================================

#[derive(Default, Deserialize, Serialize)]
pub struct Events (pub(crate) HashMap<usize, Event>);

#[derive(Default, Deserialize, Serialize)]
pub struct Event {
    pub tracks: HashMap<i32, Track>
}

#[derive(Deserialize, Serialize)]
pub struct Track {
    pub tid: i32,
    pub parent: i32,
    pub daughters: Vec<i32>,
    pub pid: i32,
    pub creator: String,
    pub vertices: Vec<Vertex>,
}

#[derive(Deserialize, Serialize)]
pub struct Vertex {
    pub energy: f32,
    pub position: Vec3,
    pub process: String,
    pub volume: String,
}

static EVENTS: Mutex<Option<Events>> = Mutex::new(None);

pub(crate) fn take() -> Option<Events> {
    EVENTS.lock().unwrap().take()
}

pub fn set(events: Events) {
    *EVENTS.lock().unwrap() = Some(events);
}

impl Events {
    pub fn new<E, T, V>(
        tracks: T,
        vertices: V,
    ) -> Result<Self, E>
    where
        E: std::error::Error,
        T: IntoIterator<Item=Result<CTrack, E>>,
        V: IntoIterator<Item=Result<CVertex, E>>,
    {
        let mut events: HashMap<usize, Event> = HashMap::new();
        for track in tracks {
            let track = track?;
            events
                .entry(track.event)
                .and_modify(|event| {
                    event.tracks.insert(track.tid, track.into());
                })
                .or_insert_with(|| {
                    let mut event = Event::default();
                    event.tracks.insert(track.tid, track.into());
                    event
                });
        }

        for vertex in vertices {
            let vertex = vertex?;
            events
                .entry(vertex.event)
                .and_modify(|event| {
                    event.tracks
                        .entry(vertex.tid)
                        .and_modify(|track| {
                            track.vertices.push(vertex.into());
                        });
                });
        }

        for event in events.values_mut() {
            let mut daughters = HashMap::<i32, Vec<i32>>::new();
            for track in event.tracks.values() {
                if track.parent <= 0 {
                    continue
                }
                daughters
                    .entry(track.parent)
                    .and_modify(|daughters| {
                        daughters.push(track.tid);
                    })
                    .or_insert_with(|| vec![track.tid]);
            }
            for (tid, mut daughters) in daughters.drain() {
                event.tracks
                    .entry(tid)
                    .and_modify(|track| {
                        daughters.sort();
                        track.daughters = daughters
                    });
            }
        }

        let events = Self(events);
        Ok(events)
    }
}

impl Track {
    pub fn target(&self) -> Transform {
        let mut min = Vec3::MAX;
        let mut max = Vec3::MIN;
        for vertex in self.vertices.iter() {
            min = min.min(vertex.position);
            max = max.max(vertex.position);
        }
        let half_width = 0.5 * (max - min);
        let [mut dx, mut dy, _] = half_width.as_ref();
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


// ===============================================================================================
//
// Input format (From NumPy arrays).
//
// ===============================================================================================

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CTrack {
    pub event: usize,
    pub tid: i32,
    pub parent: i32,
    pub pid: i32,
    pub creator: [u8; 16],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CVertex {
    pub event: usize,
    pub tid: i32,
    pub energy: f64,
    pub position: [f64; 3],
    pub direction: [f64; 3],
    pub volume: [u8; 16],
    pub process: [u8; 16],
}

impl From<CTrack> for Track {
    fn from(track: CTrack) -> Self {
        let daughters = Vec::new();
        let creator = CStr::from_bytes_until_nul(&track.creator).unwrap();
        let creator = creator.to_str().unwrap().to_string();
        let vertices = Vec::new();
        Self {
            tid: track.tid,
            parent: track.parent,
            daughters,
            pid: track.pid,
            creator,
            vertices,
        }
    }
}

impl From<CVertex> for Vertex {
    fn from(vertex: CVertex) -> Self {
        const CM: f32 = 1E-02;
        let energy = vertex.energy as f32;
        let position = Vec3::new(
            (vertex.position[0] as f32) * CM,
            (vertex.position[1] as f32) * CM,
            (vertex.position[2] as f32) * CM,
        );
        let process = CStr::from_bytes_until_nul(&vertex.process).unwrap();
        let process = process.to_str().unwrap().to_string();
        let volume = CStr::from_bytes_until_nul(&vertex.volume).unwrap();
        let volume = volume.to_str().unwrap().to_string();
        Self { energy, position, process, volume }
    }
}
