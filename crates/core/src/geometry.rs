use data::geometry::GeometryInfo;
use rmp_serde::Deserializer;
use serde::Deserialize;
use pyo3::prelude::*;
use pyo3::exceptions::{PyNotImplementedError, PyTypeError};
use pyo3::types::PyBytes;
use std::ffi::OsStr;
use std::path::Path;


pub fn load(py: Python, file: &str) -> PyResult<()> {
    let path = Path::new(file);
    match path.extension().and_then(OsStr::to_str) {
        Some("json") | Some("toml") | Some("yml") | Some("yaml") => {
            let data = load_data(py, file)?;

            #[cfg(feature = "ipc")]
            crate::ipc::send_data(py, data)?;

            #[cfg(feature = "thread")]
            display::geometry::set_data(data);
        },
        Some("stl") => {
            let path = path
                .canonicalize()?
                .to_str()
                .unwrap()
                .to_string();

            #[cfg(feature = "ipc")]
            crate::ipc::send_stl(py, path)?;

            #[cfg(feature = "thread")]
            display::geometry::set_stl(path);
        }
        _ => return Err(PyNotImplementedError::new_err("")),
    }
    Ok(())
}

pub fn from_volume(volume: &Bound<PyAny>) -> PyResult<()> {
    let data = extract_data(volume)?;

    #[cfg(feature = "ipc")]
    crate::ipc::send_data(volume.py(), data)?;

    #[cfg(feature = "thread")]
    display::geometry::set_data(data);

    Ok(())
}

fn load_data(py: Python, path: &str) -> PyResult<GeometryInfo> {
    let volume = py.import_bound("calzone")
        .and_then(|x| x.getattr("Geometry"))
        .and_then(|x| x.call1((path,)))
        .and_then(|x| x.getattr("root"))?;
    from_volume_unchecked(&volume)
}

fn extract_data(volume: &Bound<PyAny>) -> PyResult<GeometryInfo> {
    let py = volume.py();
    let ty = py.import_bound("calzone")
        .and_then(|x| x.getattr("Volume"))?;
    if volume.is_instance(&ty)? {
        from_volume_unchecked(volume)
    } else {
        let msg = format!(
            "bad volume (expected a 'calzone.Volume', found '{}')",
            volume.get_type()
        );
        let err = PyTypeError::new_err(msg);
        Err(err)
    }
}

fn from_volume_unchecked(volume: &Bound<PyAny>) -> PyResult<GeometryInfo> {
    let bytes = volume.getattr("to_bytes")
        .and_then(|x| x.call0())?;
    let bytes = bytes.downcast::<PyBytes>()?;

    let mut deserializer = Deserializer::new(bytes.as_bytes());
    Deserialize::deserialize(&mut deserializer)
        .map_err(|err| {
            let msg = format!("{}", err);
            PyTypeError::new_err(msg)
        })
}
