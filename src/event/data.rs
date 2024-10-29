use bevy::prelude::*;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::{Mutex, MutexGuard};
use super::numpy::PyArray;


// ===============================================================================================
//
// Monte Carlo event data.
//
// ===============================================================================================

pub struct Events (pub HashMap<usize, Event>);

pub struct Event {
    pub index: usize,
    pub tracks: HashMap<i32, Track>
}

pub struct Track {
    pub tid: i32,
    pub parent: i32,
    pub pid: i32,
    pub creator: String,
    pub vertices: Vec<Vertex>,
}

pub struct Vertex {
    pub energy: f32,
    pub position: Vec3,
    pub process: String,
}

static EVENTS: Mutex<Option<Events>> = Mutex::new(None);

impl Events {
    pub fn parse(data: &Bound<PyAny>) -> PyResult<()> {
        let tracks = data.getattr("tracks")?;
        let tracks: &PyArray<CTrack> = tracks.extract()?;
        let vertices = data.getattr("vertices")?;
        let vertices: &PyArray<CVertex> = vertices.extract()?;

        let mut events: HashMap<usize, Event> = HashMap::new();
        for i in 0..tracks.size() {
            let track = tracks.get(i)?;
            events
                .entry(track.event)
                .and_modify(|event| {
                    event.tracks.insert(track.tid, track.into());
                })
                .or_insert_with(|| {
                    let mut event = Event::new(track.event);
                    event.tracks.insert(track.tid, track.into());
                    event
                });
        }

        for i in 0..vertices.size() {
            let vertex = vertices.get(i)?;
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

        *EVENTS.lock().unwrap() = Some(Self(events));

        Ok(())
    }

    pub fn lock() -> MutexGuard<'static, Option<Self>> {
        EVENTS.lock().unwrap()
    }
}

impl Event {
    fn new(index: usize) -> Self {
        let tracks = HashMap::new();
        Self { index, tracks }
    }
}

impl From<CTrack> for Track {
    fn from(track: CTrack) -> Self {
        let creator = CStr::from_bytes_until_nul(&track.creator).unwrap();
        let creator = creator.to_str().unwrap().to_string();
        let vertices = Vec::new();
        Self {
            tid: track.tid,
            parent: track.parent,
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
        Self { energy, position, process }
    }
}


// ===============================================================================================
//
// NumPy / C-structures used by Calzone.
//
// ===============================================================================================

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CTrack {
    event: usize,
    tid: i32,
    parent: i32,
    pid: i32,
    creator: [u8; 16],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CVertex {
    event: usize,
    tid: i32,
    energy: f64,
    position: [f64; 3],
    direction: [f64; 3],
    volume: [u8; 16],
    process: [u8; 16],
}
