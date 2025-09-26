use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use std::env;

use data::ipc::Token;


fn main() -> Result<(), u8> {
    let oss = env::args().nth(1)
        .expect("missing OSS name");

    let (tx, rx): (IpcSender<Token>, IpcReceiver<Token>) = ipc::channel().unwrap();
    let oss = IpcSender::connect(oss).unwrap();
    oss.send(tx).unwrap();
    let receiver = std::thread::spawn(move || loop {
        match rx.try_recv() {
            Ok(data) => match data {
                Token::Close => display::geometry::set_close(),
                Token::Events(events) => display::event::set(events),
                Token::Geometry(data) => display::geometry::set_data(data),
                Token::Stop => {
                    display::app::set_exit();
                    break
                },
                Token::Stl(path) => display::geometry::set_stl(path),
            },
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    });
    let rc = display::app::run();
    if let Err(err) = receiver.join() {
        std::panic::resume_unwind(err);
    }
    match rc {
        0 => Ok(()),
        rc => Err(rc),
    }
}
