use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use pyo3::prelude::*;

mod drone;
mod display;
mod geometry;
mod lighting;
mod ui;


/// Run the viewer.
#[pyfunction]
fn run(py: Python, path: &str) -> PyResult<()> {
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(geometry::GeometryPlugin::new(py, path)?)
        .add_plugins(display::DisplayPlugin)
        .add_plugins(drone::DronePlugin)
        .add_plugins(lighting::LightingPlugin)
        .add_plugins(ui::UiPlugin)
        .add_systems(Startup, setup_physics)
        .run();
    Ok(())
}

fn setup_physics(mut config: ResMut<RapierConfiguration>) {
    config.gravity = Vect::ZERO;
}


/// CALorimeter ZONE (CalZone) Viewer
#[pymodule]
fn calzone_viewer(module: &Bound<PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(run, module)?)?;
    Ok(())
}
