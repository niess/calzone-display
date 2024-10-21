use bevy::prelude::*;
use bevy::color::palettes::css::*;
use chrono::{TimeZone, Utc};
use crate::app::AppState;
use super::geometry::GeometrySet;


pub struct LightingPlugin;

#[derive(Event)]
pub struct Shadows(bool);

#[derive(Resource)]
pub struct Sun {
    illuminance: f32,
    latitude: f32,
    longitude: f32,
    time: f32,
    entity: Entity,
}

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<Shadows>()
            .add_systems(OnEnter(AppState::Display), setup_light.after(GeometrySet));
    }
}

fn setup_light(mut commands: Commands) {
    let mut sun = Sun::default();
    sun.entity = commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: LIGHT_YELLOW.into(),
            illuminance: sun.illuminance,
            shadows_enabled: true,
            ..default()
        },
        transform: sun.compute_transform(),
        ..default()
    })
    .observe(Shadows::modify_sun)
    .id();
    commands.insert_resource(sun);
}

impl Shadows {
    pub fn enable(commands: &mut Commands, sun: &Res<Sun>) {
        commands.trigger_targets(Self(true), sun.entity);
    }

    pub fn disable(commands: &mut Commands, sun: &Res<Sun>) {
        commands.trigger_targets(Self(false), sun.entity);
    }

    fn modify_sun(
        trigger: Trigger<Self>,
        mut lights: Query<&mut DirectionalLight>,
    ) {
        let mut light = lights
            .get_mut(trigger.entity())
            .unwrap();
        light.shadows_enabled = trigger.event().0;
    }
}

impl Sun {
    fn compute_transform(&self) -> Transform {
        // Compute sun azimuth & elevation angles.
        let h = self.time.floor();
        let m = ((self.time - h) * 60.0).floor();
        let s = ((self.time - h) * 3600.0 - m * 60.0).floor();
        let utc = Utc.with_ymd_and_hms(
            2024,
            6,
            21,
            (h as u32).clamp(0, 24),
            (m as u32).clamp(0, 60),
            (s as u32).clamp(0, 60)
        )
            .single()
            .unwrap();
        let sun_position = spa::solar_position::<spa::StdFloatOps>(
            utc, self.latitude as f64, self.longitude as f64,
        ).unwrap();

        // Convert to spherical coordinates.
        let theta = sun_position.zenith_angle.to_radians() as f32;
        let phi = (90.0 - sun_position.azimuth).to_radians() as f32;

        // Apply the transform.
        Transform::from_xyz(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        ).looking_at(Vec3::ZERO, Vec3::Z)
    }
}

impl Default for Sun {
    fn default() -> Self {
        let illuminance = light_consts::lux::OVERCAST_DAY;
        let latitude = 45.0;
        let longitude = 3.0;
        let time = 12.0;
        let entity = Entity::PLACEHOLDER;
        Self { illuminance, latitude, longitude, time, entity }
    }
}
