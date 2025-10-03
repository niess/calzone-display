use pyo3::prelude::*;

#[cfg(feature = "thread")]
use std::{sync::Mutex, thread};

#[cfg(feature = "thread")]
static HANDLE: Mutex<Option<thread::JoinHandle<u8>>> = Mutex::new(None);

pub fn spawn(module: &Bound<PyModule>) -> PyResult<()> {
    #[cfg(feature = "ipc")]
    crate::ipc::spawn_agent(module.py())?;

    #[cfg(feature = "thread")]
    {
        let handle = thread::spawn(display::app::run);
        HANDLE
            .lock()
            .unwrap()
            .replace(handle);
    }

    let stopper = wrap_pyfunction!(stop, module)?;
    module.py().import_bound("atexit")?
      .call_method1("register", (stopper,))?;

    Ok(())
}

#[cfg(feature = "ipc")]
#[pyfunction]
fn stop(py: Python<'_>) -> PyResult<()> {
    crate::ipc::send_stop(py)
}

#[cfg(feature = "thread")]
#[pyfunction]
fn stop(_py: Python<'_>) -> PyResult<()> {
    display::app::set_exit();
    let handle = HANDLE
        .lock()
        .unwrap()
        .take();
    if let Some(handle) = handle {
        match handle.join() {
            Ok(_) => Ok(()),
            Err(err) => std::panic::resume_unwind(err),
        }
    } else {
        Ok(())
    }
}
