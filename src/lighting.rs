use bevy::prelude::*;
use bevy::color::palettes::css::*;
use chrono::{NaiveDate, TimeZone, Utc};
use crate::app::{AppState, Removable};
use super::geometry::GeometrySet;


pub struct LightingPlugin;

#[derive(Event)]
pub struct Shadows(bool);

#[derive(Resource)]
pub struct Sun {
    pub illuminance: f32,
    pub latitude: f32,
    pub time: f32,
    pub day: u32,
    pub month: u32,
    pub year: i32,
    entity: Entity,
}

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Sun::default())
            .add_event::<Shadows>()
            .add_systems(OnEnter(AppState::Display), setup_light.after(GeometrySet))
            .add_systems(OnExit(AppState::Display), remove_light)
            .add_systems(Update, update_light.run_if(in_state(AppState::Display)));
    }
}

#[derive(Component)]
struct SunLight;

fn setup_light(
    mut commands: Commands,
    mut sun: ResMut<Sun>,
) {
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
    .insert(SunLight)
    .insert(Removable)
    .observe(Shadows::modify_sun)
    .id();
}

fn remove_light(mut sun: ResMut<Sun>) {
    sun.entity = Entity::PLACEHOLDER;
}

fn update_light(
    sun: Res<Sun>,
    mut transform: Query<&mut Transform, With<SunLight>>,
) {
    if transform.is_empty() || !sun.is_changed() {
        return
    }
    *transform.single_mut() = sun.compute_transform();
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
    pub fn compute_position(&self) -> spa::SolarPos {
        const DAYS: [ u32; 12 ] = [ 31, 0, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 ];
        let max_day = if self.month == 2 {
            if NaiveDate::from_ymd_opt(self.year, 1, 1).unwrap().leap_year() {
                29
            } else {
                28
            }
        } else {
            DAYS[(self.month - 1) as usize]
        };

        let h = self.time.floor();
        let m = ((self.time - h) * 60.0).floor();
        let s = ((self.time - h) * 3600.0 - m * 60.0).floor();
        let datetime = Utc.with_ymd_and_hms(
            self.year,
            self.month,
            self.day.min(max_day),
            (h as u32) % 24,
            (m as u32) % 60,
            (s as u32) % 60,
        )
            .single()
            .unwrap();
        spa::solar_position::<spa::StdFloatOps>(
            datetime, self.latitude as f64, 0.0,
        ).unwrap()
    }

    fn compute_transform(&self) -> Transform {
        // Compute sun azimuth & elevation angles.
        let sun_position = self.compute_position();

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
        let time = 12.0;
        let day = 21;
        let month = 6;
        let year = 2024;
        let entity = Entity::PLACEHOLDER;
        Self { illuminance, latitude, time, day, month, year, entity }
    }
}
