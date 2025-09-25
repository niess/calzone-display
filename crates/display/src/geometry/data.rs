use bevy::prelude::*;
use bevy::render::render_resource::encase::matrix::FromMatrixParts;
use super::jmol::JMOL;
use super::units::Meters;

pub use data::geometry::{
    GeometryInfo, VolumeInfo, SolidInfo, BoxInfo, OrbInfo, SphereInfo, MeshInfo, TransformInfo,
    TubsInfo, MaterialInfo,
};

pub(crate) trait ToTransform {
    fn to_transform(&self) -> Transform;
}

impl ToTransform for TransformInfo {
    fn to_transform(&self) -> Transform {
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

pub(crate) trait Color {
    fn color(&self) -> Srgba;
}

impl Color for MaterialInfo {
    fn color(&self) -> Srgba {
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
