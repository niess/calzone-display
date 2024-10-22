use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::window::{ExitCondition::DontExit, PrimaryWindow};
use bevy::winit::{WakeUp, WinitPlugin};
use pyo3::prelude::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use super::display::DisplayPlugin;
use super::drone::DronePlugin;
use super::geometry::GeometryPlugin;
use super::lighting::LightingPlugin;
use super::ui::UiPlugin;


#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    Display,
    #[default]
    Iddle,
}

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

    let mut app = App::new();
    app
        .add_plugins((
            DefaultPlugins.build()
                .set(window)
                .set(winit),
            RapierPhysicsPlugin::<NoUserData>::default(),
            DisplayPlugin,
            DronePlugin,
            GeometryPlugin,
            LightingPlugin,
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
) {
    if GeometryPlugin::is_some() {
        if window.is_empty() {
            commands.spawn((
                Window {
                    title: "Calzone Display".to_owned(),
                    ..default()
                },
                PrimaryWindow,
            ))
            .observe(on_window_closed);
        }
        next_state.set(AppState::Display);
    } else {
        if EXIT.load(Ordering::Relaxed) {
            exit.send(AppExit::Success);
        }
    }
}

fn clear_all(world: &mut World) {
    world.clear_entities();
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