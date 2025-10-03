use bevy::prelude::*;
use bevy::color::palettes::css::*;
use bevy::ecs::system::EntityCommands;
use bevy::pbr::wireframe::{WireframeColor, WireframePlugin};
use bevy::render::{mesh::MeshAabb, primitives::Aabb};
use crate::app::{AppState, Removable};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod bundle;
mod data;
mod jmol;
mod meshes;
mod stl;
mod units;

pub use data::GeometryInfo;


pub(crate) struct GeometryPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GeometrySet;

#[derive(Component)]
pub(crate) struct RootVolume;

#[derive(Component)]
pub(crate) struct Volume {
    pub name: String,
    pub aabb: Aabb,
    pub expanded: bool,
}

#[derive(Component)]
pub(crate) struct Plain;

#[derive(Component)]
pub(crate) struct Transparent;

#[derive(Default)]
pub(crate) enum Configuration {
    Data(Arc<data::GeometryInfo>),
    Close,
    Stl(String),
    #[default]
    None,
}

static GEOMETRY: Mutex<Configuration> = Mutex::new(Configuration::None);

pub fn set_close() {
    *GEOMETRY.lock().unwrap() = Configuration::Close;
}

pub fn set_data(data: data::GeometryInfo) {
    let config = Configuration::Data(Arc::new(data));
    *GEOMETRY.lock().unwrap() = config;
}

pub fn set_stl(path: String) {
    let config = Configuration::Stl(path);
    *GEOMETRY.lock().unwrap() = config;
}

impl GeometryPlugin{
    pub fn is_data() -> bool {
        match *GEOMETRY.lock().unwrap() {
            Configuration::Data(_) => true,
            Configuration::Stl(_) => true,
            _ => false,
        }
    }

    pub fn is_some() -> bool {
        match *GEOMETRY.lock().unwrap() {
            Configuration::None => false,
            _ => true,
        }
    }
}

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(WireframePlugin::default())
            .add_systems(OnEnter(AppState::Display), setup_geometry.in_set(GeometrySet));
    }
}

fn setup_geometry(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let config = std::mem::take(GEOMETRY.lock().unwrap().deref_mut());
    match config {
        Configuration::Data(geometry) => {
            fn spawn_them_all( // recursively.
                parent: &mut EntityCommands,
                volumes: Vec<data::VolumeInfo>,
                materials_info: &HashMap<String, data::MaterialInfo>,
                transform: GlobalTransform,
                meshes: &mut Assets<Mesh>,
                materials: &mut Assets<StandardMaterial>,
            ) {
                parent.with_children(|parent| {
                    for mut volume in volumes {
                        let volumes = std::mem::take(&mut volume.daughters);
                        let mut transform = transform.clone();
                        let mut child = bundle::VolumeSpawner::new(
                            volume,
                            materials_info,
                            &mut transform,
                            meshes,
                            materials,
                        )
                        .spawn_child(parent);
                        spawn_them_all(
                            &mut child,
                            volumes,
                            materials_info,
                            transform,
                            meshes,
                            materials,
                        );
                    }
                });
            }

            let mut geometry = Arc::into_inner(geometry).unwrap();
            let volumes = std::mem::take(&mut geometry.volumes.daughters);
            let mut transform = GlobalTransform::IDENTITY;
            let mut root = bundle::VolumeSpawner::new(
                geometry.volumes,
                &geometry.materials,
                &mut transform,
                &mut meshes,
                &mut materials,
            )
            .spawn_root(&mut commands);
            spawn_them_all(
                &mut root,
                volumes,
                &geometry.materials,
                transform,
                &mut meshes,
                &mut materials,
            );
        },
        Configuration::Stl(path) => {
            let mesh = stl::load(path.as_str(), None)
                .unwrap_or_else(|err| panic!("{}", err));
            let aabb = mesh.compute_aabb().unwrap();
            let name = Path::new(path.as_str())
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .to_case(Case::Pascal);
            let color = SADDLE_BROWN.into();
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    cull_mode: None,
                    ..default()
                })),
                WireframeColor { color },
                RootVolume,
                Removable,
                Plain,
                Volume::new(name, aabb),
            ));
        },
        Configuration::Close => (),
        Configuration::None => (),
    }
}

impl Volume {
    fn new(name: String, aabb: Aabb) -> Self {
        let expanded = false;
        Self { name, aabb, expanded }
    }

    pub fn target(&self) -> Transform {
        let [dx, dy, dz] = self.aabb.half_extents.into();
        let origin = Vec3::from(self.aabb.center);
        let start_position = origin + Vec3::new(-1.5 * dx, 3.0 * dy, -1.5 * dz);
        Transform::from_translation(start_position)
            .looking_at(origin, Vec3::Y)
    }
}
