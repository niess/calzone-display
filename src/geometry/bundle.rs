use bevy::prelude::*;
use bevy::color::palettes::css::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::primitives::Aabb;
use super::data::VolumeInfo;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use super::units::Meters;


type Materials = HashMap<String, Handle<StandardMaterial>>;

static MATERIALS: LazyLock<Mutex<Materials>> = LazyLock::new(|| Mutex::new(Materials::new()));

#[derive(Bundle)]
pub struct VolumeBundle {
    pub volume: super::Volume,
    pub pbr: PbrBundle,
}

impl VolumeBundle {
    pub fn new(
        volume: VolumeInfo,
        global_transform: &mut GlobalTransform,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        let mesh = Mesh::from(volume.solid);
        let material = MATERIALS.lock().unwrap()
            .entry(volume.material)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: WHITE.into(),
                    cull_mode: None,
                    ..default()
                })
            }).clone();
        let transform = Transform::from_xyz(
            volume.transform.translation[0].meters(),
            volume.transform.translation[1].meters(),
            volume.transform.translation[2].meters(),
        );
        // XXX Apply rotation as well.
        *global_transform = global_transform.mul_transform(transform);
        let aabb = compute_aabb(&mesh, global_transform);
        let mesh = meshes.add(mesh);
        let pbr = PbrBundle { mesh, material, transform, ..default() };
        let volume = super::Volume { name: volume.name, aabb };
        Self { volume, pbr }
    }
}

fn compute_aabb(mesh: &Mesh, transform: &GlobalTransform) -> Aabb {
    let transform = transform.affine(); // XXX Check this.
    let mut min = Vec3::INFINITY;
    let mut max = Vec3::NEG_INFINITY;
    let VertexAttributeValues::Float32x3(vertices) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
    else {
        panic!()
    };
    for vertex in vertices {
        let vertex = transform.transform_point3((*vertex).into());
        min = min.min(vertex);
        max = max.max(vertex);
    }
    Aabb::from_min_max(min, max)
}
