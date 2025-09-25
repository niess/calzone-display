use bevy::prelude::*;
use bevy::render::render_resource::encase::matrix::FromMatrixParts;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::jmol::JMOL;
use super::units::Meters;


#[derive(Deserialize, Serialize)]
pub struct GeometryInfo {
    pub(crate) volumes: VolumeInfo,
    pub(crate) materials: HashMap<String, MaterialInfo>,
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

impl TransformInfo {
    pub(crate) fn to_transform(&self) -> Transform {
        let rotation: [[f32; 3]; 3] = std::array::from_fn(|i| 
            std::array::from_fn(|j| self.rotation[i][j] as f32)
        );
        let rotation = Mat3::from_parts(rotation);
        let rotation = Quat::from_mat3(&rotation);
        let translation: [f32; 3] = std::array::from_fn(|i|
            self.translation[i].meters()
        );
        let translation: Vec3 = translation.into();
        Transform::from_rotation(rotation)
            .with_translation(translation)
    }
}

impl MaterialInfo {
    pub(crate) fn color(&self) -> Srgba {
        let mut color = [0.0_f32; 3];
        for (symbol, weight) in self.composition.iter() {
            let rgb = JMOL.get(symbol.as_str())
                .unwrap_or_else(|| &Srgba::WHITE);
            let weight = *weight as f32;
            color[0] += weight * rgb.red;
            color[1] += weight * rgb.green;
            color[2] += weight * rgb.blue;
        }
        Srgba::new(color[0], color[1], color[2], 1.0)
    }
}
