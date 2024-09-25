use bevy::prelude::*;
use bevy::color::palettes::css::*;
use pyo3::prelude::*;
use pyo3::exceptions::PyNotImplementedError;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;

mod data;
mod stl;


pub struct GeometryPlugin (Configuration);

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
        Ok(Self(config))
    }
}

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset_loader::<stl::StlLoader>()
            .add_systems(Startup, (spawn_geometry, setup_light));

        let mut config = app.world_mut()
            .get_resource_or_insert_with::<Configuration>(Default::default);
        *config = self.0.clone();
    }
}

fn spawn_geometry(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    config: Res<Configuration>,
) {
    if let Configuration::Stl(path) = config.as_ref() {
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
