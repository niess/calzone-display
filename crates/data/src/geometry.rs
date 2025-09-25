use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Deserialize, Serialize)]
pub struct GeometryInfo {
    pub volumes: VolumeInfo,
    pub materials: HashMap<String, MaterialInfo>,
}

#[derive(Deserialize, Serialize)]
pub struct VolumeInfo {
    pub name: String,
    pub solid: SolidInfo,
    pub material: String,
    pub transform: TransformInfo,
    pub daughters: Vec<VolumeInfo>,
}

#[derive(Deserialize, Serialize)]
pub enum SolidInfo {
    Box(BoxInfo),
    Mesh(MeshInfo),
    Orb(OrbInfo),
    Sphere(SphereInfo),
    Tubs(TubsInfo),
}

#[derive(Deserialize, Serialize)]
pub struct BoxInfo {
    pub size: [f64; 3],
    pub displacement: [f64; 3],
}

#[derive(Deserialize, Serialize)]
pub struct OrbInfo {
    pub radius: f64,
    pub displacement: [f64; 3],
}

#[derive(Deserialize, Serialize)]
pub struct SphereInfo {
    pub inner_radius: f64,
    pub outer_radius: f64,
    pub start_phi: f64,
    pub delta_phi: f64,
    pub start_theta: f64,
    pub delta_theta: f64,
}

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct MeshInfo (pub Vec<f32>);

#[derive(Deserialize, Serialize)]
pub struct TransformInfo {
    pub translation: [f64; 3],
    pub rotation: [[f64; 3]; 3],
}

#[derive(Deserialize, Serialize)]
pub struct TubsInfo {
    pub inner_radius: f64,
    pub outer_radius: f64,
    pub length: f64,
    pub start_phi: f64,
    pub delta_phi: f64,
    pub displacement: [f64; 3],
}

#[derive(Deserialize, Serialize)]
pub struct MaterialInfo {
    pub density: f64,
    pub state: String,
    pub composition: Vec<(String, f64)>,
}
