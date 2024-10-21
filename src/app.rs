use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::window::{ExitCondition::DontExit, PrimaryWindow};
use bevy::winit::{WakeUp, WinitPlugin};
use pyo3::prelude::*;
use std::sync::Mutex;
use std::thread;
use super::display::DisplayPlugin;
use super::drone::DronePlugin;
use super::geometry::GeometryPlugin;
use super::lighting::LightingPlugin;
use super::ui::UiPlugin;


#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Iddle,
    Display,
}

#[derive(Debug)]
struct AppManager {
    handler: thread::JoinHandle<AppExit>,
}

static APP_MANAGER: Mutex<Option<AppManager>> = Mutex::new(None);

pub fn spawn(module: &Bound<PyModule>) -> PyResult<()> {
    let handler = thread::spawn(start);
    let manager = AppManager { handler };
    APP_MANAGER
        .lock()
        .unwrap()
        .replace(manager);

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
        .add_systems(Update, (
            iddle_system.run_if(in_state(AppState::Iddle)),
            display_system.run_if(in_state(AppState::Display)),
        ))
        .run()
}

#[pyfunction]
fn stop() {
    let manager = APP_MANAGER
        .lock()
        .unwrap()
        .take();
    if let Some(manager) = manager {
        manager.handler.join().unwrap();
    }
}

fn setup_physics(mut config: ResMut<RapierConfiguration>) {
    config.gravity = 9.81 * Vect::NEG_Z;
}

fn iddle_system(
    mut commands: Commands,
    window: Query<&Window>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if GeometryPlugin::is_some() {
        if window.is_empty() {
            commands.spawn((
                Window {
                    title: "Calzone Display".to_owned(),
                    ..default()
                },
                PrimaryWindow,
            ));
            // XXX Got to the Iddle state if the window is closed.
        }
        next_state.set(AppState::Display);
    }
}

fn display_system(mut next_state: ResMut<NextState<AppState>>) {
    if GeometryPlugin::is_some() {
        next_state.set(AppState::Iddle); // Despawn the current display.
    }
}
