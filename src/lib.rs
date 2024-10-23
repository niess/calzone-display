use pyo3::prelude::*;

mod app;
mod drone;
mod display;
mod geometry;
mod lighting;
mod path;
mod ui;


/// Display a Calzone geometry.
#[pyfunction]
#[pyo3(name="display", signature=(arg,/))]
fn run_display(arg: DisplayArg) -> PyResult<()> {
    match arg {
        DisplayArg::Path(path) => {
            let py = path.0.py();
            let path = path.to_string();
            geometry::GeometryPlugin::load(py, path.as_str())?;
        },
        DisplayArg::Any(any) => geometry::GeometryPlugin::from_volume(&any)?,
    }
    Ok(())
}

#[derive(FromPyObject)]
enum DisplayArg<'py> {
    Path(path::PathString<'py>),
    Any(Bound<'py, PyAny>),
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
