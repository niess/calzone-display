use bevy::prelude::*;
use bevy::color::palettes::css::*;

mod stl;


pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset_loader::<stl::StlLoader>()
            .add_systems(Startup, (spawn_geometry, setup_light));
    }
}

fn spawn_geometry(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(PbrBundle {
        mesh: asset_server.load("foreground.stl"),
        material: materials.add(
            StandardMaterial {
            base_color: BROWN.into(),
            ..default()
        }),
        ..default()
    });
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
