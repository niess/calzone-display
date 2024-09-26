use bevy::prelude::*;
use bevy::color::palettes::css::*;
use pyo3::prelude::*;
use pyo3::exceptions::PyNotImplementedError;
use std::ffi::OsStr;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod data;
mod stl;


pub struct GeometryPlugin (Mutex<Configuration>);

#[derive(Clone, Default, Resource)]
enum Configuration {
    Data(Arc<data::GeometryData>),
    Stl(String),
    #[default]
    None,
}

impl GeometryPlugin{
    pub fn new(py: Python, file: &str) -> PyResult<Self> {
        let path = Path::new(file);
        let config = match path.extension().and_then(OsStr::to_str) {
            Some("json") | Some("toml") | Some("yml") | Some("yaml") => {
                let data = data::GeometryData::new(py, file)?;
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
            .add_systems(Startup, (setup_geometry, setup_light));

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
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut config: ResMut<Configuration>,
) {
    let config = std::mem::take(config.as_mut());
    if let Configuration::Stl(path) = config {
        commands.spawn(PbrBundle {
            mesh: asset_server.load(path),
            material: materials.add(
                StandardMaterial {
                base_color: BROWN.into(),
                ..default()
            }),
            ..default()
        });

    }
}

fn setup_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}
