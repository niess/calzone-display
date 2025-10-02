use bevy::prelude::*;
use bevy::diagnostic::{
    DiagnosticsStore, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
};
use core::time::Duration;
use crate::app::AppState;
use super::{TextInputSet, TextInputState, UiRoot, UiText, UiWindow};


pub fn build(app: &mut App) {
    app
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            SystemInformationDiagnosticsPlugin
        ))
        .init_state::<StatsState>()
        .add_systems(Startup, initialise)
        .add_systems(OnEnter(AppState::Display),
            setup_panel
                .run_if(in_state(StatsState::Enabled))
        )
        .add_systems(Update, (
            update_text
                .run_if(in_state(StatsState::Enabled)),
            toggle_stats
                .after(TextInputSet)
                .run_if(in_state(TextInputState::Inactive)),
        ).run_if(in_state(AppState::Display)));
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum StatsState {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Component)]
enum Property {
    Cpu,
    Fps,
    Memory,
}

#[derive(Component)]
struct StatsPanel;

fn initialise (
    mut store: ResMut<DiagnosticsStore>,
) {
    for diagnostic in store.iter_mut() {
        diagnostic.is_enabled = false;
    }
}

fn setup_panel(
    store: Res<DiagnosticsStore>,
    mut commands: Commands,
) {
    Property::spawn_panel(&store, &mut commands);
}

fn update_text(
    store: Res<DiagnosticsStore>,
    query: Query<(Entity, &Property)>,
    mut writer: TextUiWriter,
    time: Res<Time>,
    mut time_since_rerender: Local<Duration>,
) {
    *time_since_rerender += time.delta();
    if *time_since_rerender >= Duration::from_millis(100) {
        *time_since_rerender = Duration::ZERO;
        for (entity, property) in query {
            let value = property.get(&store);
            *writer.text(entity, 0) = property.format(value);
        }
    }
}

fn toggle_stats (
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut store: ResMut<DiagnosticsStore>,
    current_state: Res<State<StatsState>>,
    mut next_state: ResMut<NextState<StatsState>>,
    panel: Query<Entity, With<StatsPanel>>,
    mut commands: Commands,
) {
    if !keyboard_input.just_pressed(KeyCode::F4) { return }
    let paths = [
        &SystemInformationDiagnosticsPlugin::PROCESS_CPU_USAGE,
        &SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE,
        &FrameTimeDiagnosticsPlugin::FPS,
    ];
    match **current_state {
        StatsState::Disabled => {
            for path in paths {
                if let Some(diagnostic) = store.get_mut(path) {
                    diagnostic.is_enabled = true;
                }
            }
            next_state.set(StatsState::Enabled);
            Property::spawn_panel(&store, &mut commands);
        },
        StatsState::Enabled => {
            for path in paths {
                if let Some(diagnostic) = store.get_mut(path) {
                    diagnostic.is_enabled = false;
                }
            }
            next_state.set(StatsState::Disabled);
            if let Ok(panel) = panel.single() {
                commands.entity(panel).despawn();
            }
        },
    }
}

impl Property {
    fn format<T: std::fmt::Display>(&self, value: Option<T>) -> String {
        match value {
            Some(value) => match self {
                Self::Cpu => format!("{:7.0}", value),
                Self::Fps => format!("{:7.0}", value),
                Self::Memory => format!("{:7.0}", value),
            },
            None => format!("{:7.7}", ""),
        }
    }

    fn get(&self, store: &DiagnosticsStore) -> Option<f64> {
        match self {
            Property::Cpu => store.get(&SystemInformationDiagnosticsPlugin::PROCESS_CPU_USAGE)
                .and_then(|diagnostic| diagnostic.value()),
            Property::Fps => store.get(&FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|diagnostic| diagnostic.smoothed()),
            Property::Memory => store.get(&SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE)
                .and_then(|diagnostic| diagnostic.value()),
        }
    }

    fn spawn_panel(
        store: &DiagnosticsStore,
        commands: &mut Commands,
    ) {
        const LABELS: [&'static str; 3] = [ "cpu", "memory", "fps", ];
        const UNITS: [&'static str; 3] = [ " %", " %", "Hz", ];

        let labels = LABELS.map(|label| commands.spawn(UiText::new_bundle(label)).id());
        let units = UNITS.map(|label| commands.spawn(UiText::new_bundle(label)).id());

        let values = [
            Property::Cpu,
            Property::Memory,
            Property::Fps,
        ];
        let values = values.map(|property| {
            let value = property.get(&store);
            let value = property.format(value);
            commands.spawn((
                UiText::new_bundle(&value),
                property,
            )).id()
        });

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

        let mut panel = UiWindow::new("Statistics", super::WindowLocation::BottomLeft, commands);
        panel.add_child(content);
        panel.insert(StatsPanel);
        panel.insert(UiRoot);
    }
}
