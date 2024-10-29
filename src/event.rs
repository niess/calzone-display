use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_polyline::prelude::*;
use crate::app::{AppState, Removable};
use std::sync::Mutex;

mod data;
mod numpy;

pub use data::Events as EventsData;
pub use numpy::initialise;


pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(PolylinePlugin)
            .init_resource::<Events>()
            .add_systems(Update, (
                    update_events,
                    draw_event,
                    on_keyboard,
                ).run_if(in_state(AppState::Display))
            );
    }
}

#[derive(Default, Resource)]
pub struct Events {
    data: data::Events,
    index: usize,
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

fn update_events(
    mut events: ResMut<Events>,
) {
    if let Some(data) = data::Events::take() {
        *events = Events {
            data,
            index: 0,
        }
    }
}

struct VertexAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

const EVENT_LAYER: usize = 2;

static VERTEX_ASSETS: Mutex<Option<VertexAssets>> = Mutex::new(None);

fn draw_event(
    events: Res<Events>,
    current_event: Query<Entity, With<Event>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polymats: ResMut<Assets<PolylineMaterial>>,
) {
    if events.is_changed() && (events.index < events.data.0.len()) {
        if let Some(event) = events.data.0.get(&events.index) {
            for entity in current_event.iter() {
                commands
                    .entity(entity)
                    .despawn_recursive();
            }

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
                        let vertices: Vec<Vec3> = track.vertices
                            .iter()
                            .map(|v| v.position)
                            .collect();
                        let polyline = Polyline { vertices };
                        let material = PolylineMaterial {
                            width: 1.0,
                            color: LinearRgba::rgb(1.0, 1.0, 0.0),
                            ..default()
                        };
                        parent
                            .spawn((
                                Track::from(track),
                                PolylineBundle {
                                    polyline: polylines.add(polyline),
                                    material: polymats.add(material),
                                    ..default()
                                },
                                RenderLayers::layer(EVENT_LAYER),
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

fn on_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut events: ResMut<Events>,
) {
    let n = events.data.0.len();
    if n == 0 {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        events.index += 1;
        if events.index >= n {
            events.index = 0;
        }
    }
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        if events.index > 0 {
            events.index -= 1;
        } else {
            events.index = n - 1;
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

/* XXX
#[derive(Default, Resource)]
struct EventsCursor {
    index: usize,
    len: usize,
}

fn setup_cursor(mut cursor: ResMut<EventsCursor>) {
    if let Some(events) = Events::lock().deref() {
        *cursor = EventsCursor {
            index: 0,
            len: events.0.len(),
        }
    }
}
*/
