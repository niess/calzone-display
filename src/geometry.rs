use bevy::prelude::*;
use bevy::color::palettes::css::*;
use bevy::ecs::system::EntityCommands;
use bevy::render::primitives::Aabb;
use pyo3::prelude::*;
use pyo3::exceptions::PyNotImplementedError;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod bundle;
mod data;
mod meshes;
mod stl;
mod units;


pub struct GeometryPlugin (Mutex<Configuration>);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeometrySet;

#[derive(Component)]
pub struct GeometryExtent (pub Aabb);

#[derive(Clone, Default, Resource)]
enum Configuration {
    Data(Arc<data::VolumeInfo>),
    Stl(String),
    #[default]
    None,
}

impl GeometryPlugin{
    pub fn new(py: Python, file: &str) -> PyResult<Self> {
        let path = Path::new(file);
        let config = match path.extension().and_then(OsStr::to_str) {
            Some("json") | Some("toml") | Some("yml") | Some("yaml") => {
                let data = data::VolumeInfo::new(py, file)?;
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
        Ok(Self(Mutex::new(config)))
    }
}

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset_loader::<stl::StlLoader>()
            .add_systems(Startup, (setup_geometry, setup_light).in_set(GeometrySet));

        // Promote the geometry data to a Resource.
        match &mut self.0.lock() {
            Err(_) => unimplemented!(),
            Ok(config) => {
                app
                    .world_mut()
                    .insert_resource::<Configuration>(std::mem::take(config));
            },
        }
    }
}

fn setup_geometry(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut config: ResMut<Configuration>,
) {
    let config = std::mem::take(config.as_mut());
    match config {
        Configuration::Data(root) => {
            fn spawn_them_all( // recursively.
                parent: &mut EntityCommands,
                volumes: Vec<data::VolumeInfo>,
                meshes: &mut Assets<Mesh>,
                materials: &mut Assets<StandardMaterial>,
            ) {
                parent.with_children(|parent| {
                    for mut volume in volumes {
                        let volumes = std::mem::take(&mut volume.daughters);
                        let mut child = parent.spawn(
                            bundle::VolumeBundle::new(volume, meshes, materials)
                        );
                        spawn_them_all(&mut child, volumes, meshes, materials);
                    }
                });
            }

            let mut root = Arc::into_inner(root).unwrap();
            let volumes = std::mem::take(&mut root.daughters);
            let root = bundle::VolumeBundle::new(root, &mut meshes, &mut materials);
            let extent = GeometryExtent(
                meshes
                    .get(&root.0.mesh)
                    .unwrap()
                    .compute_aabb()
                    .unwrap()
            );
            let mut root = commands.spawn((root, extent));
            spawn_them_all(&mut root, volumes, &mut meshes, &mut materials);
        },
        Configuration::Stl(path) => {
            commands.spawn(PbrBundle {
                mesh: asset_server.load(path),
                material: materials.add(StandardMaterial {
                    base_color: SADDLE_BROWN.into(),
                    ..default()
                }),
                ..default()
            });
        },
        Configuration::None => (),
    }
}

fn setup_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: LIGHT_YELLOW.into(),
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}
