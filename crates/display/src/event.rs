use bevy::prelude::*;
use bevy::render::camera::CameraOutputMode;
use bevy::render::view::RenderLayers;
use bevy::render::render_resource::BlendState;
use bevy::window::PrimaryWindow;
use bevy_polyline::prelude::*;
use crate::app::{AppState, Removable};
use crate::drone::Drone;
use crate::ui::{PrimaryMenu, TextInputSet, TextInputState, UiEvent};
use std::borrow::Cow;

mod colours;
mod data;
mod picking;

pub use data::set;

pub(crate) use data::Events as EventsData;
pub(crate) use data::Event as EventData;
pub(crate) use data::Target;
pub(crate) use data::Track as TrackData;
use data::ToView;


pub(crate) struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(PolylinePlugin)
            .add_plugins(picking::PickingPlugin)
            .init_resource::<Events>()
            .add_systems(Update, (
                    update_events,
                    draw_event,
                    on_keyboard
                        .after(TextInputSet)
                        .run_if(in_state(TextInputState::Inactive)),
                ).run_if(in_state(AppState::Display))
            );
    }
}

#[derive(Default, Resource)]
pub(crate) struct Events {
    pub data: EventsData,
    pub index: usize,
}

#[derive(Component)]
pub(crate) struct Event;

#[derive(Component)]
pub(crate) struct Track {
    pub tid: i32,
    pub parent: i32,
    pub pid: i32,
    pub creator: String,
}

#[derive(Component)]
pub(crate) struct Vertex {
    pub energy: f32,
    pub process: String,
    pub volume: String,
}

#[derive(Component)]
struct VertexSize (f32);

fn update_events(mut events: ResMut<Events>) {
    if let Some(data) = data::take() {
        *events = Events {
            data,
            index: 0,
        }
    }
}

const EVENT_LAYER: usize = 1;

fn draw_event(
    events: Res<Events>,
    current_event: Query<Entity, With<Event>>,
    primary_menu: Query<Entity, With<PrimaryMenu>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polymats: ResMut<Assets<PolylineMaterial>>,
) {
    let Ok(primary_window) = primary_window.single() else { return };

    if events.is_changed() && (events.index < events.data.0.len()) {
        if let Some(event) = events.data.0.get(&events.index) {
            // Remove any existing event.
            for entity in current_event.iter() {
                commands
                    .entity(entity)
                    .despawn();
            }

            // Spawn the current event.
            commands
                .spawn((
                    Event,
                    Transform::default(),
                    Visibility::default(),
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
                            .map(|v| v.position.to_view())
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
                                    polyline: PolylineHandle(polylines.add(polyline)),
                                    material: PolylineMaterialHandle(polymats.add(material)),
                                    ..default()
                                },
                                RenderLayers::layer(EVENT_LAYER),
                            ))
                            .with_children(|parent| {
                                let n = track.vertices.len();
                                for vertex in track.vertices[0..n].iter() {
                                    parent.spawn((
                                        Vertex::from(vertex),
                                        VertexSize(vertex_size),
                                        MeshMaterial3d(vertex_material.clone()),
                                        Mesh3d(vertex_mesh.clone()),
                                        Transform::from_translation(vertex.position.to_view()),
                                        RenderLayers::layer(EVENT_LAYER),
                                    ));
                                }
                          });
                    }
                });
            UiEvent::spawn_status(&events, primary_menu, &primary_window, &mut commands);
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
            volume: vertex.volume.clone(),
        }
    }
}

#[derive(Component)]
pub(crate) struct EventCamera;

#[derive(Bundle)]
pub(crate) struct EventBundle (EventCamera, Camera3d, Camera, Projection, RenderLayers);

impl EventBundle {
    pub fn new(fov: f32) -> Self {
        Self (
            EventCamera,
            Camera3d::default(),
            Camera {
                order: 1,
                // HDR compat with multiple cameras (see bevy issue #18901).
                clear_color: ClearColorConfig::Custom(Color::NONE),
                output_mode: CameraOutputMode::Write {
                    blend_state: Some(BlendState::ALPHA_BLENDING),
                    clear_color: ClearColorConfig::None,
                },
                ..default()
            },
            Projection::Perspective(PerspectiveProjection {
                fov,
                near: Drone::NEAR,
                ..default()
            }),
            RenderLayers::layer(EVENT_LAYER),
        )
    }
}

impl Track {
    pub fn label(&self) -> String {
        Self::label_from_parts(self.tid, self.pid)
    }

    pub fn label_from_parts(tid: i32, pid: i32) -> String {
        let particle = match pid {
            11 => Cow::Borrowed("e-"),
            -11 => Cow::Borrowed("e+"),
            13 => Cow::Borrowed("mu-"),
            -13 => Cow::Borrowed("mu+"),
            22 => Cow::Borrowed("gamma"),
            _ => Cow::Owned(format!("[{}]", pid)),
        };
        format!(
            "{} [{}]",
            particle,
            tid,
        )
    }
}
