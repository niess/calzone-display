use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3::sync::GILOnceCell;
use std::process::{Child, Command};
use std::sync::Mutex;

use agent::Data;
use display::event::EventsData;
use display::geometry::GeometryInfo;

struct Pipe {
    process: Child,
    tx: IpcSender<Data>,
}

static PIPE: GILOnceCell<Mutex<Pipe>> = GILOnceCell::new();

pub(crate) fn spawn_agent(py: Python<'_>) -> PyResult<()> {
    let (oss, oss_name) = IpcOneShotServer::new()
        .map_err(|_| PyRuntimeError::new_err("could not create display-oss"))?;
    let path: String = py
        .import_bound("shutil")?
        .call_method1("which", ("calzone-display-agent",))?
        .extract()
        .map_err(|_| PyRuntimeError::new_err("could not locate calzone-display-agent"))?;
    let process = Command::new(path)
        .arg(oss_name)
        .spawn()
        .map_err(|_| PyRuntimeError::new_err("could not spawn calzone-display-agent"))?;
    let (_, tx): (_, IpcSender<Data>) = oss.accept()
        .map_err(|_| PyRuntimeError::new_err("could not connect to display-oss"))?;
    let pipe = Pipe { process, tx };
    PIPE.set(py, Mutex::new(pipe))
        .map_err(|_| PyRuntimeError::new_err("could not set display-pipe"))?;
    Ok(())
}

const GET_FAILED: &str = "could not get display-pipe";
const LOCK_FAILED: &str = "could not lock display-pipe";

macro_rules! get_pipe {
    ($py:ident) => {
        PIPE
            .get($py)
            .ok_or_else(|| PyRuntimeError::new_err(GET_FAILED))?
            .lock()
            .map_err(|_| PyRuntimeError::new_err(LOCK_FAILED))?
    }
}

pub(crate) fn send_close(py: Python<'_>) -> PyResult<()> {
    let pipe = get_pipe!(py);
    pipe.tx.send(Data::Close).unwrap();
    Ok(())
}

pub(crate) fn send_data(py: Python<'_>, data: GeometryInfo) -> PyResult<()> {
    let pipe = get_pipe!(py);
    pipe.tx.send(Data::Geometry(data)).unwrap();
    Ok(())
}

pub(crate) fn send_events(py: Python<'_>, data: EventsData) -> PyResult<()> {
    let pipe = get_pipe!(py);
    pipe.tx.send(Data::Events(data)).unwrap();
    Ok(())
}

pub(crate) fn send_stl(py: Python<'_>, path: String) -> PyResult<()> {
    let pipe = get_pipe!(py);
    pipe.tx.send(Data::Stl(path)).unwrap();
    Ok(())
}

pub(crate) fn send_stop(py: Python<'_>) -> PyResult<()> {
    let mut pipe = get_pipe!(py);
    pipe.tx.send(Data::Stop).unwrap();
    let _ = pipe.process.wait();
    Ok(())
}
