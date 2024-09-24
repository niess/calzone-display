use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod drone;
mod geometry;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(drone::DronePlugin)
        .add_plugins(geometry::GeometryPlugin)
        .add_systems(Startup, setup_physics)
        .run();
}

fn setup_physics(mut config: ResMut<RapierConfiguration>) {
    config.gravity = Vect::ZERO;
}
