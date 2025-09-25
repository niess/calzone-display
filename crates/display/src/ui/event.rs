use bevy::prelude::*;
use crate::app::AppState;
use crate::drone::TargetEvent;
use crate::event::{Event, EventData, Events, Track, TrackData, Vertex};
use std::collections::{HashMap, HashSet};
use super::{PrimaryMenu, Scroll, UiText};


pub fn build(app: &mut App) {
    app
        .init_resource::<TracksExpansion>()
        .add_event::<UpdateEvent>()
        .add_systems(Update, (
            on_button,
            on_update.after(on_button)
        ).run_if(in_state(AppState::Display)));
}

#[derive(Component)]
pub struct UiEvent;

#[derive(Default, Resource)]
struct TracksExpansion(HashMap<i32, bool>);

impl UiEvent {
    pub fn spawn_info(
        commands: &mut Commands,
        cursor: Vec2,
        matches: Vec<(&Track, &Vertex)>
    ) {
        struct TrackData<'a> {
            track: &'a Track,
            vertices: Vec<&'a Vertex>,
        }

        let mut tracks: HashMap<i32, TrackData> = HashMap::new();
        for (track, vertex) in matches.iter() {
            tracks
                .entry(track.tid)
                .and_modify(|data| data.vertices.push(*vertex))
                .or_insert_with(|| {
                    let mut vertices = Vec::new();
                    vertices.push(*vertex);
                    TrackData { track, vertices }
                });
        }
        let mut tracks: Vec<_> = tracks.values().collect();
        tracks.sort_by(|a, b| a.track.tid.cmp(&b.track.tid));

        let mut windows = Vec::new();
        for data in tracks.iter() {
            fn spawn_column<'a, T>(
                commands: &'a mut Commands,
                entries: &[T]
            ) -> Entity
            where
                T: AsRef<str>,
            {
                commands
                    .spawn(
                        NodeBundle {
                            style: Style {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(4.0)),
                                ..default()
                            },
                            ..default()
                        }
                    )
                    .with_children(|parent| {
                        for entry in entries.iter() {
                            let entry: &str = entry.as_ref();
                            parent.spawn(UiText::new_bundle(entry));
                        }
                    }).id()
            }

            let mut labels: Vec<&'static str> = Vec::new();
            let mut values: Vec<String> = Vec::new();

            if data.track.tid > 1 {
                labels.push("creator");
                values.push(
                    format!("{} [{}]", data.track.creator, data.track.parent)
                );
            };

            let n = data.vertices.len();
            let e0 = data.vertices[0].energy;
            let e1 = data.vertices[n - 1].energy;
            if e0 == e1 {
                labels.push("energy");
                values.push(uformat(e0));
            } else {
                labels.push("energies");
                values.push(format!("{} to {}", uformat(e0), uformat(e1)));
            }

            fn dedup(v: &mut Vec<&str>) { // Preserves the initial order.
                let mut set = HashSet::new();
                v.retain(|x| set.insert(*x));
            }

            let mut processes: Vec<&str> = data.vertices
                .iter()
                .map(|vertex| vertex.process.as_str())
                .filter(|process| !process.is_empty())
                .collect();

            dedup(&mut processes);

            if processes.len() == 1 {
                labels.push("process");
                values.push(processes[0].to_string());
            } else if processes.len() > 1 {
                labels.push("processes");
                if processes.len() == 2 {
                    values.push(format!("{} and {}", processes[0], processes[1]))
                } else {
                    values.push(processes.join(", "));
                }
            }

            let mut volumes: Vec<&str> = data.vertices
                .iter()
                .map(|vertex| vertex.volume.as_str())
                .filter(|volume| !volume.is_empty())
                .collect();

            dedup(&mut volumes);

            if volumes.len() == 1 {
                labels.push("volume");
                values.push(volumes[0].to_string());
            } else if volumes.len() > 1 {
                labels.push("volumes");
                if volumes.len() == 2 {
                    values.push(format!("{} and {}", volumes[0], volumes[1]))
                } else {
                    values.push(volumes.join(", "));
                }
            }

            let labels = spawn_column(commands, &labels);
            let values = spawn_column(commands, &values);

            let mut content = commands.spawn(
                NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                },
            );
            content.push_children(&[labels, values]);
            let content = content.id();

            let title = data.track.label();
            let mut window = super::UiWindow::new(
                title.as_str(),
                super::WindowLocation::Relative,
                commands
            );
            window.add_child(content);
            let window = window.id();

            let mut node = commands.spawn(NodeBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            });
            node.add_child(window);
            windows.push(node.id());
        }

        let mut node = commands.spawn((
            UiEvent,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(cursor.y + 12.0),
                    left: Val::Px(cursor.x + 12.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ));
        node.push_children(&windows);
    }

    pub fn spawn_status(
        events: &Events,
        primary_menu: Query<Entity, With<PrimaryMenu>>,
        primary_window: &Window,
        commands: &mut Commands,
    ) {
        if events.data.0.len() == 0 {
            return
        }

        let content = commands.spawn((
            EventContent,
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        )).id();

        commands.insert_resource(TracksExpansion::default());
        update_content(content, events, &TracksExpansion::default(), commands);

        let mut scroll = Scroll::spawn(commands, primary_window);
        scroll.add_child(content);
        let scroll = scroll.id();

        let title = format!("Event [{}]", events.index);
        let mut window = super::UiWindow::new(
            title.as_str(),
            super::WindowLocation::Relative,
            commands
        );
        window.add_child(scroll);
        let window = window.id();

        let mut capsule = commands.spawn(NodeBundle {
            style: Style {
                padding: UiRect::left(Val::Px(4.0)),
                ..default()
            },
            ..default()
        });
        capsule.insert(Event);
        capsule.add_child(window);
        let capsule = capsule.id();

        commands
            .entity(primary_menu.single())
            .add_child(capsule);
    }
}

#[derive(Component)]
struct EventContent;

fn clear_content(content: Entity, commands: &mut Commands) {
    let mut content = commands.entity(content);
    content.despawn_descendants();
}

fn update_content(
    content: Entity,
    events: &Events,
    expansions: &TracksExpansion,
    commands: &mut Commands,
) {
    fn add_button(
        depth: usize,
        event: &EventData,
        track: &TrackData,
        content: Entity,
        expansions: &TracksExpansion,
        commands: &mut Commands,
    ) {
        let expanded = *expansions.0.get(&track.tid).unwrap_or(&false);
        let qualifier = if (track.daughters.len() > 0) && !expanded {
            ".."
        } else {
            ""
        };
        let label = Track::label_from_parts(track.tid, track.pid);
        let energy = track.vertices
            .get(0)
            .map(|vertex| uformat(vertex.energy))
            .unwrap_or("?".to_string());
        let message = format!("{}{}, {}, {}{}",
            "  ".repeat(depth),
            label,
            track.creator,
            energy,
            qualifier,
        );
        let button = TrackButton::spawn_button(&message, track.tid, commands);
        commands
            .entity(content)
            .add_child(button);
        if expanded {
            for daughter in track.daughters.iter() {
                let daughter = &event.tracks[daughter];
                add_button(depth + 1, event, daughter, content, expansions, commands);
            }
        }
    }

    let event = &events.data.0[&events.index];
    add_button(0, event, &event.tracks[&1], content, expansions, commands);
}

#[derive(Component)]
struct TrackButton(i32);

impl TrackButton {
    fn spawn_button(
        message: &str,
        tid: i32,
        commands: &mut Commands,
    ) -> Entity {
        let component = TrackButton(tid);
        UiText::spawn_button(component, message, commands)
    }
}

#[derive(Event)]
struct UpdateEvent(i32, bool);

fn on_button(
    interactions: Query<(&Interaction, &TrackButton, &Children), Changed<Interaction>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    events: Res<Events>,
    mut text_query: Query<&mut Text>,
    mut ev_target: EventWriter<TargetEvent>,
    mut ev_update: EventWriter<UpdateEvent>,
) {
    for (interaction, button, children) in interactions.iter() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                if keyboard_input.pressed(KeyCode::ShiftLeft) {
                    let event = &events.data.0[&events.index];
                    let track = &event.tracks[&button.0];
                    ev_target.send(TargetEvent(track.target()));
                } else {
                    let recursive = keyboard_input.pressed(KeyCode::ControlLeft);
                    ev_update.send(UpdateEvent(button.0, recursive));
                }
                text.sections[0].style.color = UiText::PRESSED.into();
            }
            Interaction::Hovered => {
                text.sections[0].style.color = UiText::HOVERED.into();
            }
            Interaction::None => {
                text.sections[0].style.color = UiText::NORMAL.into();
            }
        }
    }
}

fn on_update(
    mut commands: Commands,
    mut reader: EventReader<UpdateEvent>,
    content: Query<Entity, With<EventContent>>,
    events: Res<Events>,
    mut expansions: ResMut<TracksExpansion>,
) {
    for UpdateEvent (tid, recursive) in reader.read() {
        expansions.0
            .entry(*tid)
            .and_modify(|expanded| *expanded = !(*expanded))
            .or_insert(true);
        if *recursive {
            fn recurse(
                expanded: bool,
                tid: i32,
                event: &EventData,
                expansions: &mut TracksExpansion
            ) {
                for daughter in event.tracks[&tid].daughters.iter() {
                    expansions.0
                        .entry(*daughter)
                        .and_modify(|e| *e = expanded)
                        .or_insert(expanded);
                    recurse(expanded, *daughter, event, expansions);
                }
            }
            let event = &events.data.0[&events.index];
            recurse(expansions.0[tid], *tid, event, &mut expansions);
        }

        let content = content.single();
        clear_content(content, &mut commands);
        update_content(content, &events, &expansions, &mut commands);
    }
}

fn uformat(energy: f32) -> String {
    let scale = energy.log10() as i64 + 6;
    if scale <= 2 {
        format!("{:.3} eV", energy * 1E+06)
    } else if scale <= 5 {
        format!("{:.3} keV", energy * 1E+03)
    } else if scale <= 8 {
        format!("{:.3} MeV", energy)
    } else if scale <= 11 {
        format!("{:.3} GeV", energy * 1E-03)
    } else if scale <= 14 {
        format!("{:.3} TeV", energy * 1E-06)
    } else if scale <= 17 {
        format!("{:.3} PeV", energy * 1E-09)
    } else if scale <= 20 {
        format!("{:.3} EeV", energy * 1E-12)
    } else {
        format!("{:.3} ZeV", energy * 1E-15)
    }
}
