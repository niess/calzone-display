use bevy::prelude::*;
use _core::ipc::run_agent;
use std::env;


fn main() -> AppExit {
    let oss_name = env::args().nth(1)
        .expect("missing OSS name");
    run_agent(oss_name)
}
