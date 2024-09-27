use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use pyo3::types::PyBytes;
use rmp_serde::Deserializer;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct VolumeInfo {
    pub solid: SolidInfo,
    pub material: String,
    pub transform: TransformInfo,
    pub daughters: Vec<VolumeInfo>,
}

#[derive(Deserialize)]
pub enum SolidInfo {
    Box(BoxInfo),
    Orb(OrbInfo),
    Sphere(SphereInfo),
    Tessellation(TessellationInfo),
    Tubs(TubsInfo),
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct BoxInfo {
    pub size: [f64; 3]
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct OrbInfo {
    pub radius: f64,
}

#[derive(Deserialize)]
pub struct SphereInfo {
    pub inner_radius: f64,
    pub outer_radius: f64,
    pub start_phi: f64,
    pub delta_phi: f64,
    pub start_theta: f64,
    pub delta_theta: f64,
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct TessellationInfo (pub Vec<f32>);

#[derive(Deserialize)]
pub struct TransformInfo {
    pub translation: [f64; 3],
    pub rotation: [[f64; 3]; 3],
}

#[derive(Deserialize)]
pub struct TubsInfo {
    pub inner_radius: f64,
    pub outer_radius: f64,
    pub length: f64,
    pub start_phi: f64,
    pub delta_phi: f64,
}

impl VolumeInfo {
    pub fn new(py: Python, path: &str) -> PyResult<Self> {
        let bytes = py.import_bound("calzone")
            .and_then(|x| x.getattr("Geometry"))
            .and_then(|x| x.call1((path,)))
            .and_then(|x| x.getattr("root"))
            .and_then(|x| x.getattr("bytes"))
            .and_then(|x| x.call0())?;
        let bytes = bytes.downcast::<PyBytes>()?;

        let mut deserializer = Deserializer::new(bytes.as_bytes());
        Deserialize::deserialize(&mut deserializer)
            .map_err(|err| {
                let msg = format!("{}", err);
                PyTypeError::new_err(msg)
            })
    }
}
