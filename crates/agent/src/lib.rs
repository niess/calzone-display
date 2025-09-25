use serde::{Deserialize, Serialize};

use display::event::EventsData;
use display::geometry::GeometryInfo;


#[derive(Serialize, Deserialize)]
pub enum Data {
    Close,
    Events(EventsData),
    Geometry(GeometryInfo),
    Stop,
    Stl(String),
}
