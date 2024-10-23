use pyo3::prelude::*;
use pyo3::sync::GILOnceCell;
use pyo3::types::PyString;


// ===============================================================================================
//
// Pathlib.Path wrapper.
//
// ===============================================================================================

pub struct PathString<'py> (pub Bound<'py, PyString>);

impl<'py> FromPyObject<'py> for PathString<'py> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        static TYPE: GILOnceCell<PyObject> = GILOnceCell::new();
        let py = ob.py();
        let tp = TYPE.get_or_try_init(py, || py.import_bound("pathlib")
            .and_then(|m| m.getattr("Path"))
            .map(|m| m.unbind())
        )?.bind(py);
        if ob.is_instance(tp)? {
            let path = ob.str()?;
            Ok(Self(path))
        } else {
            let path: Bound<PyString> = ob.extract()?;
            Ok(Self(path))
        }
    }
}

impl<'py> ToString for PathString<'py> {
    fn to_string(&self) -> String {
        self.0.to_string_lossy().to_string()
    }
}
