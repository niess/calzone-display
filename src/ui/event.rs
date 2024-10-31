use bevy::prelude::*;
use crate::event::{Track, Vertex};
use std::borrow::Cow;
use std::collections::HashMap;


#[derive(Component)]
pub struct UiEvent;

impl UiEvent {
    pub fn spawn(
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
            let particle = match data.track.pid {
                11 => Cow::Borrowed("e-"),
                -11 => Cow::Borrowed("e+"),
                13 => Cow::Borrowed("mu-"),
                -13 => Cow::Borrowed("mu+"),
                22 => Cow::Borrowed("gamma"),
                _ => Cow::Owned(format!("[{}]", data.track.pid)),
            };
            let parent = if data.track.tid == 1 {
                "".to_owned()
            } else {
                format!(" (created by [{}])", data.track.parent)
            };
            let title = format!(
                "{}[{}]{}",
                particle,
                data.track.tid,
                parent,
            );
            let window = super::UiWindow::new(
                title.as_str(),
                super::WindowLocation::Relative,
                commands
            ).id();
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
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                ..default()
            },
        ));
        node.push_children(&windows);
    }
}
