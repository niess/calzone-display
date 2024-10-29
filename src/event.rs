use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::app::{AppState, Removable};
use crate::geometry::GeometrySet;
use std::ops::Deref;
use std::sync::Mutex;

mod data;
mod numpy;

pub use data::Events;
pub use numpy::initialise;


pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Display), setup_event.after(GeometrySet));
    }
}


#[derive(Component)]
struct Event (usize);

#[derive(Component)]
struct Track {
    tid: i32,
    parent: i32,
    pid: i32,
    creator: String,
}

#[derive(Component)]
struct Vertex {
    energy: f32,
    process: String,
}

struct VertexAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

const EVENT_LAYER: usize = 2;

static VERTEX_ASSETS: Mutex<Option<VertexAssets>> = Mutex::new(None);

fn setup_event(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some(events) = Events::lock().deref() {
        if let Some(event) = events.0.get(&0) {
            // Get or create vertices mesh.
            let mut vertex_assets = VERTEX_ASSETS.lock().unwrap();
            if vertex_assets.is_none() {
                let mesh = Sphere::new(0.001).mesh().build();
                let mesh = meshes.add(mesh);
                let material = StandardMaterial {
                    base_color: Srgba::rgb(1.0, 1.0, 0.0).into(),
                    unlit: true,
                    ..default()
                };
                let material = materials.add(material);
                *vertex_assets = Some(VertexAssets { mesh, material });
            }

            commands
                .spawn((
                    Event (event.index),
                    SpatialBundle::default(),
                    Removable,
                ))
                .with_children(|parent| {
                    for track in event.tracks.values() {
                        parent
                            .spawn((
                                Track::from(track),
                                SpatialBundle::default(),
                            ))
                            .with_children(|parent| {
                                for vertex in track.vertices.iter() {
                                    parent.spawn((
                                        Vertex::from(vertex),
                                        PbrBundle {
                                            material: vertex_assets
                                                .as_ref()
                                                .unwrap()
                                                .material
                                                .clone(),
                                            mesh: vertex_assets
                                                .as_ref()
                                                .unwrap()
                                                .mesh
                                                .clone(),
                                            transform: Transform::from_translation(
                                                vertex.position
                                            ),
                                            ..default()
                                        },
                                        RenderLayers::layer(EVENT_LAYER),
                                    ));
                                }
                          });
                    }
                });
        }
    }
}

impl<'a> From<&'a data::Track> for Track {
    fn from(track: &'a data::Track) -> Self {
        Self {
            tid: track.tid,
            parent: track.parent,
            pid: track.pid,
            creator: track.creator.clone(),
        }
    }
}

impl<'a> From<&'a data::Vertex> for Vertex {
    fn from(vertex: &'a data::Vertex) -> Self {
        Self {
            energy: vertex.energy,
            process: vertex.process.clone(),
        }
    }
}

#[derive(Component)]
pub struct EventCamera;

#[derive(Bundle)]
pub struct EventBundle (EventCamera, Camera3dBundle, RenderLayers);

impl EventBundle {
    pub fn new(fov: f32) -> Self {
        Self (
            EventCamera,
            Camera3dBundle {
                camera: Camera {
                    order: 1,
                    ..default()
                },
                projection: PerspectiveProjection {
                    fov,
                    ..default()
                }.into(),
                ..default()
            },
            RenderLayers::layer(EVENT_LAYER),
        )
    }
}
