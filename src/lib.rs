use pyo3::prelude::*;

mod app;
mod drone;
mod display;
mod geometry;
mod lighting;
mod ui;


/// Run the viewer.
#[pyfunction]
fn run(py: Python, path: &str) -> PyResult<()> {
    geometry::GeometryPlugin::load(py, path)?;
    Ok(())
}


/// CALorimeter ZONE (CalZone) Viewer
#[pymodule]
fn calzone_viewer(module: &Bound<PyModule>) -> PyResult<()> {
    // Spawn the display app in a dedicated thread.
    app::spawn(module)?;

    // Set the module's interface.
    module.add_function(wrap_pyfunction!(run, module)?)?;

    Ok(())
}
