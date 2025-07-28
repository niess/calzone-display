use pyo3::prelude::*;

mod app;
mod drone;
mod display;
mod event;
mod geometry;
mod lighting;
mod path;
mod sky;
mod ui;


///  Close the current display.
#[pyfunction]
#[pyo3(name="close", signature=())]
fn close_display() {
    geometry::GeometryPlugin::unload();
}


/// Display a Calzone geometry.
#[pyfunction]
#[pyo3(name="display", signature=(arg,/, *, data=None))]
fn run_display<'py>(
    py: Python<'py>,
    arg: DisplayArg<'py>,
    data: Option<&Bound<'py, PyAny>>,
) -> PyResult<()> {
    // Load the geometry.
    match arg {
        DisplayArg::Path(path) => {
            let path = path.to_string();
            geometry::GeometryPlugin::load(py, path.as_str())?;
        },
        DisplayArg::Any(any) => geometry::GeometryPlugin::from_volume(&any)?,
    }

    // Parse any tracking data.
    if let Some(data) = data {
        event::EventsData::parse(data)?;
    }

    Ok(())
}

#[derive(FromPyObject)]
enum DisplayArg<'py> {
    Path(path::PathString<'py>),
    Any(Bound<'py, PyAny>),
}


/// A display extension for Calzone (CALorimeter ZONE)
#[pymodule]
fn calzone_display(module: &Bound<PyModule>) -> PyResult<()> {
    // Initialise the events interface.
    let py = module.py();
    event::initialise(py)?;

    // Spawn the display app in a dedicated thread.
    app::spawn(module)?;

    // Set the module's interface.
    module.add_function(wrap_pyfunction!(close_display, module)?)?;
    module.add_function(wrap_pyfunction!(run_display, module)?)?;

    Ok(())
}
