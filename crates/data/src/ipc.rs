use serde::{Deserialize, Serialize};

use super::event::Events;
use super::geometry::GeometryInfo;


#[derive(Serialize, Deserialize)]
pub enum Token {
    Close,
    Events(Events),
    Geometry(GeometryInfo),
    Stop,
    Stl(String),
}
