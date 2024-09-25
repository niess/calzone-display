use bevy::prelude::*;
use bevy::color::palettes::css::*;
use pyo3::prelude::*;
use std::path::Path;

mod stl;


pub struct GeometryPlugin (Configuration);

#[derive(Clone, Default, Resource)]
struct Configuration (Option<String>);

impl GeometryPlugin{
    pub fn new(path: &str) -> PyResult<Self> {
        let path = Path::new(path)
            .canonicalize()?
            .to_str()
            .unwrap()
            .to_string();
        let config = Configuration (Some(path));
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
    if let Some(path) = &config.0 {
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
