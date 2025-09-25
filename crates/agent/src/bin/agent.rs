use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use std::env;

use agent::Data;


fn main() -> Result<(), u8> {
    let oss = env::args().nth(1)
        .expect("missing OSS name");

    let (tx, rx): (IpcSender<Data>, IpcReceiver<Data>) = ipc::channel().unwrap();
    let oss = IpcSender::connect(oss).unwrap();
    oss.send(tx).unwrap();
    let receiver = std::thread::spawn(move || loop {
        match rx.try_recv() {
            Ok(data) => match data {
                Data::Close => display::geometry::set_close(),
                Data::Events(events) => display::event::set(events),
                Data::Geometry(data) => display::geometry::set_data(data),
                Data::Stop => {
                    display::app::set_exit();
                    break
                },
                Data::Stl(path) => display::geometry::set_stl(path),
            },
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    });
    let rc = display::app::start();
    if let Err(err) = receiver.join() {
        std::panic::resume_unwind(err);
    }
    match rc {
        0 => Ok(()),
        rc => Err(rc),
    }
}
