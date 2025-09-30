use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy::pbr::wireframe::WireframeColor;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::primitives::Aabb;
use crate::app::Removable;
use super::data::{Color, MaterialInfo, ToTransform, VolumeInfo};
use super::meshes::IntoMesh;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};


pub enum VolumeSpawner {
    Standard {
        volume: super::Volume,
        mesh: Mesh3d,
        material: MeshMaterial3d<StandardMaterial>,
        transform: Transform,
        color: bevy::prelude::Color,
    },
    Wireframe {
        volume: super::Volume,
        mesh: Mesh3d,
        transform: Transform,
        color: bevy::prelude::Color,
    },
}

impl VolumeSpawner {
    pub fn new(
        info: VolumeInfo,
        materials: &HashMap<String, MaterialInfo>,
        global_transform: &mut GlobalTransform,
        meshes: &mut Assets<Mesh>,
        standards: &mut Assets<StandardMaterial>,
    ) -> Self {
        let material = materials.get(info.material.as_str()).unwrap();
        let mesh = info.solid.into_mesh();
        let color = material.color().into();
        let transform = info.transform.to_transform();
        *global_transform = global_transform.mul_transform(transform);
        let aabb = compute_aabb(&mesh, global_transform);
        let mesh = Mesh3d(meshes.add(mesh));
        let volume = super::Volume::new(info.name, aabb);
        if (material.state.as_str() == "gas") || (material.density <= 1E-02) {
            Self::Wireframe { volume, mesh, transform, color }
        } else {
            let material = MeshMaterial3d(STANDARD_MATERIALS.lock().unwrap()
                .entry(info.material)
                .or_insert_with(|| {
                    standards.add(StandardMaterial {
                        base_color: color,
                        double_sided: true,
                        cull_mode: None,
                        ..default()
                    })
                }).clone());
            Self::Standard { volume, mesh, material, transform, color }
        }
    }

    pub fn spawn_child<'a>(self, parent: &'a mut ChildSpawnerCommands) -> EntityCommands<'a> {
        match self {
            Self::Standard { volume, mesh, material, transform, color } => {
                parent.spawn((
                    volume, mesh, material, transform, WireframeColor { color }, super::Plain,
                ))
            },
            Self::Wireframe { volume, mesh, transform, color } => {
                parent.spawn((
                    volume, mesh, transform, WireframeColor { color }, super::Transparent,
                ))
            },
        }
    }

    pub fn spawn_root<'a>(self, commands: &'a mut Commands) -> EntityCommands<'a> {
        match self {
            Self::Standard { volume, mesh, material, transform, color } => {
                commands.spawn((
                    volume, mesh, material, transform, WireframeColor { color }, super::RootVolume,
                    super::Plain, Removable
                ))
            },
            Self::Wireframe { volume, mesh, transform, color } => {
                commands.spawn((
                    volume, mesh, transform, WireframeColor { color }, super::RootVolume,
                    super::Transparent, Removable
                ))
            },
        }
    }
}

static STANDARD_MATERIALS: LazyLock<Mutex<HashMap<String, Handle<StandardMaterial>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn compute_aabb(mesh: &Mesh, transform: &GlobalTransform) -> Aabb {
    let transform = transform.affine();
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
