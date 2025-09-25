use process_path::get_dylib_path;
use pyo3::prelude::*;
use pyo3::exceptions::PySystemError;
use pyo3::sync::GILOnceCell;
use std::path::{Path, PathBuf};

mod app;
mod event;
mod geometry;
mod numpy;
mod path;

#[cfg(feature = "ipc")]
pub mod ipc;


static PREFIX: GILOnceCell<PathBuf> = GILOnceCell::new();

fn set_prefix(py: Python) -> PyResult<()> {
    let filename = match get_dylib_path() {
        Some(path) => path
                        .to_string_lossy()
                        .to_string(),
        None => return Err(PySystemError::new_err("could not resolve module path")),
    };
    let prefix = match Path::new(&filename).parent() {
        None => Path::new(".").to_path_buf(),
        Some(path) => path.to_path_buf(),
    };
    PREFIX
        .set(py, prefix).unwrap();
    Ok(())
}

/// Close the current display.
#[pyfunction]
#[pyo3(name="close")]
fn close_display(_py: Python<'_>) -> PyResult<()> {
    #[cfg(feature = "ipc")]
    {
        crate::ipc::send_close(_py)
    }

    #[cfg(feature = "thread")]
    {
        display::geometry::set_close();
        Ok(())
    }
}

/// Display a Calzone geometry.
#[pyfunction]
#[pyo3(name="display", signature=(arg,/, *, data=None))]
fn update_display<'py>(
    py: Python<'py>,
    arg: DisplayArg<'py>,
    data: Option<&Bound<'py, PyAny>>,
) -> PyResult<()> {
    // Load the geometry.
    match arg {
        DisplayArg::Path(path) => {
            let path = path.to_string();
            geometry::load(py, path.as_str())?;
        },
        DisplayArg::Any(any) => geometry::from_volume(&any)?,
    }

    // Parse any tracking data.
    if let Some(data) = data {
        event::parse(data)?;
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
#[pyo3(name = "_core")]
fn init(module: &Bound<PyModule>) -> PyResult<()> {
    // Set the module prefix.
    let py = module.py();
    set_prefix(py)?;

    // Initialise the events interfaces.
    numpy::initialise(py)?;

    // Spawn the display app.
    app::spawn(module)?;

    // Set the module's interface.
    module.add_function(wrap_pyfunction!(close_display, module)?)?;
    module.add_function(wrap_pyfunction!(update_display, module)?)?;

    Ok(())
}
