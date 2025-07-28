use bevy::prelude::*;
use bevy::log::{Level, LogPlugin};
use bevy::window::{ExitCondition::DontExit, PrimaryWindow};
use bevy::winit::{EventLoopProxy, WakeUp, WinitPlugin};
use bevy_rapier3d::prelude::*;
use pyo3::prelude::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use super::display::DisplayPlugin;
use super::drone::DronePlugin;
use super::event::EventPlugin;
use super::geometry::GeometryPlugin;
use super::lighting::LightingPlugin;
use super::sky::SkyPlugin;
use super::ui::UiPlugin;


#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    Display,
    #[default]
    Iddle,
}

#[derive(Component)]
pub struct Removable;

static HANDLE: Mutex<Option<thread::JoinHandle<AppExit>>> = Mutex::new(None);

static EXIT: AtomicBool = AtomicBool::new(false);

pub fn spawn(module: &Bound<PyModule>) -> PyResult<()> {
    let handle = thread::spawn(start);
    HANDLE
        .lock()
        .unwrap()
        .replace(handle);

    let stopper = wrap_pyfunction!(stop, module)?;
    module.py().import_bound("atexit")?
      .call_method1("register", (stopper,))?;
    Ok(())
}

fn start() -> AppExit {
    let mut winit = WinitPlugin::<WakeUp>::default();
    winit.run_on_any_thread = true;

    let window = WindowPlugin {
        primary_window: None,
        exit_condition: DontExit,
        close_when_requested: true,
    };

    let log = if cfg!(debug_assertions) {
        LogPlugin {
            filter: "wgpu=error".to_string(),
            ..default()
        }
    } else {
        LogPlugin {
            level: Level::ERROR,
            filter: "".to_string(),
            ..default()
        }
    };

    let mut app = App::new();
    app
        .add_plugins((
            DefaultPlugins.build()
                .set(log)
                .set(window)
                .set(winit),
            RapierPhysicsPlugin::<NoUserData>::default(),
            DisplayPlugin,
            DronePlugin,
            EventPlugin,
            GeometryPlugin,
            LightingPlugin,
            SkyPlugin,
            UiPlugin,
        ))
        .init_state::<AppState>()
        .add_systems(Startup, setup_physics)
        .add_systems(OnExit(AppState::Display), clear_all)
        .add_systems(Update, (
            iddle_system.run_if(in_state(AppState::Iddle)),
            display_system.run_if(in_state(AppState::Display)),
        ))
        .run()
}

#[pyfunction]
fn stop() {
    EXIT.store(true, Ordering::Relaxed);
    let handle = HANDLE
        .lock()
        .unwrap()
        .take();
    if let Some(handle) = handle {
        handle.join().unwrap();
    }
}

fn setup_physics(mut config: ResMut<RapierConfiguration>) {
    config.gravity = 9.81 * Vect::NEG_Z;
}

fn iddle_system(
    mut commands: Commands,
    window: Query<&Window>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    event_loop_proxy: NonSend<EventLoopProxy<WakeUp>>,
) {
    if GeometryPlugin::is_data() {
        if window.is_empty() {
            commands.spawn((
                Window {
                    title: "Calzone Display".to_owned(),
                    ..default()
                },
                PrimaryWindow,
            ))
            .observe(on_window_closed);
            let _ = event_loop_proxy.send_event(WakeUp); // To trigger a winit redraw.
        }
        next_state.set(AppState::Display);
    } else {
        if EXIT.load(Ordering::Relaxed) {
            exit.send(AppExit::Success);
        }
    }
}

pub fn clear_all(
    entities: Query<Entity, With<Removable>>,
    mut commands: Commands,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn display_system(mut next_state: ResMut<NextState<AppState>>) {
    if GeometryPlugin::is_some() {
        next_state.set(AppState::Iddle); // Despawn the current display.
    }
}

fn on_window_closed(
    _trigger: Trigger<OnRemove, PrimaryWindow>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    next_state.set(AppState::Iddle); // Despawn the current display.
}
