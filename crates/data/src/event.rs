use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CStr;


// ===============================================================================================
//
// Monte Carlo event data.
//
// ===============================================================================================

#[derive(Default, Deserialize, Serialize)]
pub struct Events (pub HashMap<usize, Event>);

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

#[repr(C)]
#[derive(Deserialize, Serialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Deserialize, Serialize)]
pub struct Vertex {
    pub energy: f32,
    pub position: Vec3,
    pub process: String,
    pub volume: String,
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
        let position = Vec3 {
            x: (vertex.position[0] as f32) * CM,
            y: (vertex.position[1] as f32) * CM,
            z: (vertex.position[2] as f32) * CM,
        };
        let process = CStr::from_bytes_until_nul(&vertex.process).unwrap();
        let process = process.to_str().unwrap().to_string();
        let volume = CStr::from_bytes_until_nul(&vertex.volume).unwrap();
        let volume = volume.to_str().unwrap().to_string();
        Self { energy, position, process, volume }
    }
}
