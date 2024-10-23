use pyo3::prelude::*;

mod app;
mod drone;
mod display;
mod geometry;
mod lighting;
mod path;
mod ui;


// XXX Interface to display a calzone.Volume.


/// Display a Calzone geometry.
#[pyfunction]
#[pyo3(name="display", signature=(path,/))]
fn run_display(path: path::PathString) -> PyResult<()> {
    let py = path.0.py();
    let path = path.to_string();
    geometry::GeometryPlugin::load(py, path.as_str())?;
    Ok(())
}


/// CALorimeter ZONE (CalZone) Viewer
#[pymodule]
fn calzone_viewer(module: &Bound<PyModule>) -> PyResult<()> {
    // Spawn the display app in a dedicated thread.
    app::spawn(module)?;

    // Set the module's interface.
    module.add_function(wrap_pyfunction!(run_display, module)?)?;

    Ok(())
}
