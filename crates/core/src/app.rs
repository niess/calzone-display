use pyo3::prelude::*;

#[cfg(not(feature = "ipc"))]
use std::{sync::Mutex, thread};

#[cfg(not(feature = "ipc"))]
static HANDLE: Mutex<Option<thread::JoinHandle<u8>>> = Mutex::new(None);

pub fn spawn(module: &Bound<PyModule>) -> PyResult<()> {
    #[cfg(feature = "ipc")]
    crate::ipc::spawn_agent(module.py())?;

    #[cfg(not(feature = "ipc"))]
    {
        let handle = thread::spawn(display::app::start);
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

#[cfg(not(feature = "ipc"))]
#[pyfunction]
fn stop(_py: Python<'_>) -> PyResult<()> {
    display::app::set_exit();
    let handle = HANDLE
        .lock()
        .unwrap()
        .take();
    if let Some(handle) = handle {
        handle.join().unwrap();
    }
    Ok(())
}
