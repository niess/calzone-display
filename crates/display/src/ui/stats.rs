use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use core::time::Duration;
use crate::app::AppState;
use super::{UiRoot, UiText, UiWindow};


pub fn build(app: &mut App) {
    app
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_systems(OnEnter(AppState::Display), setup_panel)
        .add_systems(Update,
            update_text
                .run_if(in_state(AppState::Display))
        );
}

#[derive(Component)]
enum Property {
    Fps,
}

fn setup_panel(
    mut commands: Commands,
) {
    const LABELS: [&'static str; 1] = [ "fps", ];
    const UNITS: [&'static str; 1] = [ "Hz", ];

    let labels = LABELS.map(|label| commands.spawn(UiText::new_bundle(label)).id());
    let units = UNITS.map(|label| commands.spawn(UiText::new_bundle(label)).id());

    fn format<T>(property: Property, value: T) -> (Property, String)
    where
        T: std::fmt::Display
    {
        let value = property.format(value);
        (property, value)
    }

    let values = [
        format(Property::Fps, 0.0),
    ];
    let values = values.map(
        |(property, value)| commands.spawn((
            UiText::new_input(&value, (7.5 * UiText::font_width()).round()),
            property,
        )).id()
    );

    let columns = [labels, values, units];
    let columns = columns.map(|column| {
        let mut entity = commands.spawn(
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            }
        );
        entity.add_children(&column);
        entity.id()
    });

    let mut content = commands.spawn(
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    );
    content.add_children(&columns);
    let content = content.id();

    let mut panel = UiWindow::new("Statistics", super::WindowLocation::BottomLeft, &mut commands);
    panel.add_child(content);
    panel.insert(UiRoot);
}

fn update_text(
    diagnostic: Res<DiagnosticsStore>,
    query: Query<(Entity, &Property)>,
    mut writer: TextUiWriter,
    time: Res<Time>,
    mut time_since_rerender: Local<Duration>,
) {
    *time_since_rerender += time.delta();
    if *time_since_rerender >= Duration::from_millis(100) {
        *time_since_rerender = Duration::ZERO;
        for (entity, property) in query {
            match property {
                Property::Fps => {
                    if let Some(fps) = diagnostic.get(&FrameTimeDiagnosticsPlugin::FPS)
                        && let Some(value) = fps.smoothed()
                    {
                        *writer.text(entity, 1) = property.format(value);
                    }
                },
            }
        }
    }
}

impl Property {
    fn format<T: std::fmt::Display>(&self, value: T) -> String {
        match self {
            Self::Fps => format!("{:7.0}", value),
        }
    }
}
