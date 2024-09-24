use bevy::prelude::*;
use bevy_stl::StlPlugin;


pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(StlPlugin)
            .add_systems(Startup, spawn_geometry);
    }
}

fn spawn_geometry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    commands.spawn(PbrBundle {
        mesh: asset_server.load("foreground.stl"),
        material: materials.add(Color::WHITE),
        ..default()
    });
}
