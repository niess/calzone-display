use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_polyline::prelude::*;
use crate::app::{AppState, Removable};

mod colours;
mod data;
mod numpy;
mod picking;

pub use data::Events as EventsData;
pub use numpy::initialise;


pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(PolylinePlugin)
            .add_plugins(picking::PickingPlugin)
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

#[derive(Component)]
struct VertexSize (f32);

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

const EVENT_LAYER: usize = 2;

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
            // Remove any existing event.
            for entity in current_event.iter() {
                commands
                    .entity(entity)
                    .despawn_recursive();
            }

            // Spawan the current event.
            commands
                .spawn((
                    Event (event.index),
                    SpatialBundle::default(),
                    Removable,
                ))
                .with_children(|parent| {
                    for track in event.tracks.values() {
                        let vertex_size = match track.pid {
                            22 => 5E-04,
                            _ => 3E-04,
                        };
                        let vertex_mesh = Sphere::new(vertex_size).mesh().build();
                        let vertex_mesh = meshes.add(vertex_mesh);
                        let color = match colours::COLOURS.get(&track.pid) {
                            Some(color) => *color,
                            None => LinearRgba::WHITE,
                        };
                        let vertex_material = StandardMaterial {
                            base_color: color.into(),
                            unlit: true,
                            ..default()
                        };
                        let vertex_material = materials.add(vertex_material);
                        let vertices: Vec<Vec3> = track.vertices
                            .iter()
                            .map(|v| v.position)
                            .collect();
                        let polyline = Polyline { vertices };
                        let material = PolylineMaterial {
                            width: 1.0,
                            color,
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
                                let n = track.vertices.len();
                                for vertex in track.vertices[1..n].iter() {
                                    parent.spawn((
                                        Vertex::from(vertex),
                                        VertexSize(vertex_size),
                                        PbrBundle {
                                            material: vertex_material.clone(),
                                            mesh: vertex_mesh.clone(),
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