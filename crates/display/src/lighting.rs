use bevy::prelude::*;
use bevy::color::palettes::css::*;
use bevy::pbr::Atmosphere;
use chrono::{NaiveDate, TimeZone, Utc};
use crate::world_to_view;
use crate::app::{AppState, Removable};
use crate::drone::{Drone, DroneCamera};
use crate::ui::{LocationState, TextInputSet, TextInputState};
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
#[repr(i32)]
enum LightingState {
    #[default]
    Overhead,
    Sun,
    Atmosphere,
    Guard,
}

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Sun::default())
            .init_state::<LightingState>()
            .add_event::<Shadows>()
            .add_systems(OnEnter(AppState::Display),
                setup_light
                    .after(GeometrySet)
                    .after(Drone::spawn)
            )
            .add_systems(OnExit(AppState::Display), remove_light)
            .add_systems(OnEnter(LightingState::Overhead),
                setup_overhead
                    .after(setup_light)
                    .run_if(in_state(AppState::Display))
            )
            .add_systems(OnEnter(LightingState::Sun),
                setup_sun
                    .after(setup_light)
                    .run_if(in_state(AppState::Display))
            )
            .add_systems(OnEnter(LightingState::Atmosphere),
                setup_atmosphere
                    .after(setup_light)
                    .run_if(in_state(AppState::Display))
            )
            .add_systems(OnExit(LightingState::Atmosphere),
                remove_atmosphere
                    .after(setup_light)
                    .run_if(in_state(AppState::Display))
            )
            .add_systems(Update, (
                on_keyboard
                    .after(TextInputSet)
                    .run_if(in_state(TextInputState::Inactive)),
                follow_sun
                    .run_if(in_state(LightingState::Sun).or(in_state(LightingState::Atmosphere))),
                follow_drone
                    .run_if(in_state(LightingState::Overhead)),
            ).run_if(in_state(AppState::Display)));
    }
}

#[derive(Component)]
struct SceneLight;

fn setup_light(
    mut sun: ResMut<Sun>,
    mut commands: Commands,
) {
    sun.entity = commands.spawn((
        DirectionalLight {
            color: WHITE.into(),
            illuminance: sun.illuminance,
            shadows_enabled: true,
            ..default()
        },
        Transform::IDENTITY,
        SceneLight,
        Removable,
    ))
        .observe(Shadows::modify)
        .id();
}

fn remove_light(
    mut sun: ResMut<Sun>,
    mut next_state: ResMut<NextState<LightingState>>,
) {
    sun.entity = Entity::PLACEHOLDER;
    next_state.set(LightingState::default());
}

fn follow_drone(
    drone: Query<&Transform, (With<Drone>, Without<SceneLight>, Changed<Transform>)>,
    mut light: Query<&mut Transform, (With<SceneLight>, Without<Drone>)>,
) -> Result<()> {
    if let Ok(drone) = drone.single() {
        *light.single_mut()? = *drone;
    }
    Ok(())
}

fn follow_sun(
    sun: Res<Sun>,
    mut transform: Query<&mut Transform, With<SceneLight>>,
) {
    if transform.is_empty() || !sun.is_changed() {
        return
    }
    *transform.single_mut().unwrap() = sun.compute_transform();
}

fn on_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<LightingState>>,
    mut next_state: ResMut<NextState<LightingState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        next_state.set(current_state.next());
    }
}

fn setup_overhead(
    drone: Query<&Transform, (With<Drone>, Without<SceneLight>)>,
    mut light: Query<&mut Transform, (With<SceneLight>, Without<Drone>)>,
    current_location: Res<State<LocationState>>,
    mut next_location: ResMut<NextState<LocationState>>,
) -> Result<()> {
    if let Ok(drone) = drone.single() {
        *light.single_mut()? = *drone;
    }
    if let LocationState::Enabled = **current_location {
        next_location.set(LocationState::Disabled);
    }
    Ok(())
}

fn setup_sun(
    sun: Res<Sun>,
    mut transform: Query<&mut Transform, With<SceneLight>>,
    current_location: Res<State<LocationState>>,
    mut next_location: ResMut<NextState<LocationState>>,
) {
    if let Ok(mut transform) = transform.single_mut() {
        *transform = sun.compute_transform();
    }
    if let LocationState::Disabled = **current_location {
        next_location.set(LocationState::Enabled);
    }
}

fn setup_atmosphere(
    sun: Res<Sun>,
    mut transform: Query<&mut Transform, With<SceneLight>>,
    current_location: Res<State<LocationState>>,
    mut next_location: ResMut<NextState<LocationState>>,
    camera: Query<Entity, With<DroneCamera>>,
    mut commands: Commands,
) -> Result<()> {
    *transform.single_mut()? = sun.compute_transform();
    commands.entity(camera.single()?).insert(Atmosphere::EARTH);
    if let LocationState::Disabled = **current_location {
        next_location.set(LocationState::Enabled);
    }
    Ok(())
}

fn remove_atmosphere(
    camera: Query<Entity, With<DroneCamera>>,
    mut commands: Commands,
) -> Result<()> {
    commands.entity(camera.single()?).remove::<Atmosphere>();
    Ok(())
}

impl Shadows {
    pub fn enable(commands: &mut Commands, sun: &Res<Sun>) {
        commands.trigger_targets(Self(true), sun.entity);
    }

    pub fn disable(commands: &mut Commands, sun: &Res<Sun>) {
        commands.trigger_targets(Self(false), sun.entity);
    }

    fn modify(
        trigger: Trigger<Self>,
        mut lights: Query<&mut DirectionalLight, With<SceneLight>>,
    ) {
        let mut light = lights
            .get_mut(trigger.target())
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
        let direction = Vec3::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        );
        let direction = world_to_view().transform_vector3(direction);

        Transform::from_translation(direction)
            .looking_at(Vec3::ZERO, Vec3::Y)
    }
}

impl Default for Sun {
    fn default() -> Self {
        let illuminance = light_consts::lux::RAW_SUNLIGHT;
        let latitude = 45.0;
        let time = 12.0;
        let day = 21;
        let month = 6;
        let year = 2024;
        let entity = Entity::PLACEHOLDER;
        Self { illuminance, latitude, time, day, month, year, entity }
    }
}

impl LightingState {
    fn next(self) -> Self {
        ((self as i32) + 1)
            .rem_euclid(Self::Guard as i32)
            .into()
    }
}

impl From<i32> for LightingState {
    fn from(value: i32) -> Self {
        if value == (Self::Overhead as i32) {
            Self::Overhead
        } else if value == (Self::Sun as i32) {
            Self::Sun
        } else if value == (Self::Atmosphere as i32) {
            Self::Atmosphere
        } else {
            unreachable!()
        }
    }
}
