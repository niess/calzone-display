use bevy::prelude::*;
use bevy::color::palettes::css::*;
use bevy::ecs::system::EntityCommands;
use bevy::pbr::wireframe::{WireframeMaterial, WireframePlugin};
use bevy::render::primitives::Aabb;
use crate::app::{AppState, Removable};
use convert_case::{Case, Casing};
use pyo3::prelude::*;
use pyo3::exceptions::PyNotImplementedError;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod bundle;
mod data;
mod jmol;
mod meshes;
mod stl;
mod units;


pub struct GeometryPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeometrySet;

#[derive(Component)]
pub struct RootVolume;

#[derive(Component)]
pub struct Volume {
    pub name: String,
    pub aabb: Aabb,
    pub expanded: bool,
}

#[derive(Component)]
pub struct Plain;

#[derive(Component)]
pub struct Transparent;

#[derive(Default)]
enum Configuration {
    Data(Arc<data::GeometryInfo>),
    Close,
    Stl(String),
    #[default]
    None,
}

static GEOMETRY: Mutex<Configuration> = Mutex::new(Configuration::None);

impl GeometryPlugin{
    pub fn load(py: Python, file: &str) -> PyResult<()> {
        let path = Path::new(file);
        let config = match path.extension().and_then(OsStr::to_str) {
            Some("json") | Some("toml") | Some("yml") | Some("yaml") => {
                let data = data::GeometryInfo::load(py, file)?;
                Configuration::Data(Arc::new(data))
            },
            Some("stl") => {
                let path = path
                    .canonicalize()?
                    .to_str()
                    .unwrap()
                    .to_string();
                Configuration::Stl(path)
            }
            _ => return Err(PyNotImplementedError::new_err("")),
        };
        *GEOMETRY.lock().unwrap() = config;
        Ok(())
    }

    pub fn from_volume(volume: &Bound<PyAny>) -> PyResult<()> {
        let data = data::GeometryInfo::from_volume(volume)?;
        let config = Configuration::Data(Arc::new(data));
        *GEOMETRY.lock().unwrap() = config;
        Ok(())
    }

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

    pub fn unload() {
        *GEOMETRY.lock().unwrap() = Configuration::Close;
    }
}

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(WireframePlugin)
            .add_systems(OnEnter(AppState::Display), setup_geometry.in_set(GeometrySet));
    }
}

fn setup_geometry(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut wireframe_materials: ResMut<Assets<WireframeMaterial>>,
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
                standard_materials: &mut Assets<StandardMaterial>,
                wireframe_materials: &mut Assets<WireframeMaterial>,
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
                            standard_materials,
                            wireframe_materials,
                        )
                        .spawn_child(parent);
                        spawn_them_all(
                            &mut child,
                            volumes,
                            materials_info,
                            transform,
                            meshes,
                            standard_materials,
                            wireframe_materials,
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
                &mut standard_materials,
                &mut wireframe_materials
            )
            .spawn_root(&mut commands);
            spawn_them_all(
                &mut root,
                volumes,
                &geometry.materials,
                transform,
                &mut meshes,
                &mut standard_materials,
                &mut wireframe_materials,
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
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    material: standard_materials.add(StandardMaterial {
                        base_color: SADDLE_BROWN.into(),
                        cull_mode: None,
                        ..default()
                    }),
                    ..default()
                },
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
        let start_position = origin + Vec3::new(-1.5 * dx, -1.5 * dy, 3.0 * dz);
        Transform::from_translation(start_position)
            .looking_at(origin, Vec3::Z)
    }
}
