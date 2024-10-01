use bevy::prelude::*;
use bevy::color::palettes::css::*;
use super::geometry::GeometrySet;


pub struct LightingPlugin;

#[derive(Resource)]
pub struct SunLight {
    intensity: f32,
    latitude: f32,
    time: f32,
    shadows: bool,
}

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SunLight>()
            .add_systems(Startup, setup_light.after(GeometrySet));
    }
}

fn setup_light(
    mut commands: Commands,
    sun: Res<SunLight>,
) {
    commands.spawn(
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: LIGHT_YELLOW.into(),
                illuminance: sun.intensity,
                shadows_enabled: sun.shadows,
                ..default()
            },
            transform: sun.compute_transform(),
            ..default()
        },
    );
}

impl SunLight {
    fn compute_transform(&self) -> Transform {
        let mut transform = Transform::IDENTITY;
        transform.rotate_x(-self.latitude.to_radians()); // XXX Check this.
        transform.rotate_local_y((15.0 * (self.time - 12.0)).to_radians());
        transform
    }
}

impl Default for SunLight {
    fn default() -> Self {
        let intensity = light_consts::lux::OVERCAST_DAY;
        let latitude = 45.0;
        let time = 12.0;
        let shadows = true;
        Self { intensity, latitude, time, shadows }
    }
}
