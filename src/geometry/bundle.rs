use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy::pbr::wireframe::WireframeMaterial;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::primitives::Aabb;
use crate::app::Removable;
use super::data::{MaterialInfo, VolumeInfo};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};


pub enum VolumeSpawner {
    Standard(VolumeBundle<StandardMaterial>),
    Wireframe(VolumeBundle<WireframeMaterial>),
}

impl VolumeSpawner {
    pub fn new(
        volume: VolumeInfo,
        materials: &HashMap<String, MaterialInfo>,
        global_transform: &mut GlobalTransform,
        meshes: &mut Assets<Mesh>,
        standards: &mut Assets<StandardMaterial>,
        wireframes: &mut Assets<WireframeMaterial>,
    ) -> Self {
        let material = materials.get(volume.material.as_str()).unwrap();
        if (material.state.as_str() == "gas") || (material.density <= 1E-02) {
            let bundle = VolumeBundle::<WireframeMaterial>::new(
                volume,
                material,
                global_transform,
                meshes,
                wireframes,
            );
            Self::Wireframe(bundle)
        } else {
            let bundle = VolumeBundle::<StandardMaterial>::new(
                volume,
                material,
                global_transform,
                meshes,
                standards,
            );
            Self::Standard(bundle)
        }
    }

    pub fn spawn_child<'a>(self, parent: &'a mut ChildBuilder) -> EntityCommands<'a> {
        match self {
            Self::Standard(bundle) => parent.spawn((bundle, super::Plain)),
            Self::Wireframe(bundle) => parent.spawn((bundle, super::Transparent)),
        }
    }

    pub fn spawn_root<'a>(self, commands: &'a mut Commands) -> EntityCommands<'a> {
        match self {
            Self::Standard(bundle) => commands.spawn(
                (bundle, super::RootVolume, super::Plain, Removable)
            ),
            Self::Wireframe(bundle) => commands.spawn(
                (bundle, super::RootVolume, super::Transparent, Removable)
            ),
        }
    }
}

#[derive(Bundle)]
pub struct VolumeBundle<T>
where
    T: Material,
{
    volume: super::Volume,
    bundle: MaterialMeshBundle<T>,
}

impl<T> VolumeBundle<T>
where
    T: Material,
    Named<T>: GetMaterial::<Material = T>,
{
    pub fn new(
        volume: VolumeInfo,
        material: &MaterialInfo,
        global_transform: &mut GlobalTransform,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<T>,
    ) -> Self {
        let mesh = Mesh::from(volume.solid);
        let material = Named::<T>::get_material(volume.material, material, materials);
        let transform = volume.transform.to_transform();
        *global_transform = global_transform.mul_transform(transform);
        let aabb = compute_aabb(&mesh, global_transform);
        let mesh = meshes.add(mesh);
        let volume = super::Volume::new(volume.name, aabb);
        // let pbr = T::make_bundle(mesh, material, transform);
        let bundle = MaterialMeshBundle { mesh, material, transform, ..default() };
        Self { volume, bundle }
    }
}

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

pub struct Named<T: Material> (HashMap<String, Handle<T>>);

impl<T:Material> Named<T> {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

pub trait GetMaterial
where
    <Self as GetMaterial>::Material: Asset,
{
    type Material;

    fn get_material(
        name: String,
        info: &MaterialInfo,
        materials: &mut Assets<Self::Material>
    ) -> Handle<Self::Material>;
}

static STANDARD_MATERIALS: LazyLock<Mutex<Named<StandardMaterial>>> =
    LazyLock::new(|| Mutex::new(Named::<StandardMaterial>::new()));

impl GetMaterial for Named<StandardMaterial> {
    type Material = StandardMaterial;

    fn get_material(
        name: String,
        info: &MaterialInfo,
        materials: &mut Assets<Self::Material>
    ) -> Handle<Self::Material> {
        STANDARD_MATERIALS.lock().unwrap().0
            .entry(name)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: info.color().into(),
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })
            }).clone()
    }
}

static WIREFRAME_MATERIALS: LazyLock<Mutex<Named<WireframeMaterial>>> =
    LazyLock::new(|| Mutex::new(Named::<WireframeMaterial>::new()));

impl GetMaterial for Named<WireframeMaterial> {
    type Material = WireframeMaterial;

    fn get_material(
        name: String,
        info: &MaterialInfo,
        materials: &mut Assets<Self::Material>
    ) -> Handle<Self::Material> {
        WIREFRAME_MATERIALS.lock().unwrap().0
            .entry(name)
            .or_insert_with(|| {
                materials.add(WireframeMaterial {
                    color: info.color().into()
                })
            }).clone()
    }
}
