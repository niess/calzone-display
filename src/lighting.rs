use bevy::prelude::*;
use bevy::color::palettes::css::*;
use super::geometry::GeometrySet;


pub struct LightingPlugin;

#[derive(Event)]
pub struct Shadows(bool);

#[derive(Resource)]
pub struct Sun {
    illuminance: f32,
    latitude: f32,
    time: f32,
    entity: Entity,
}

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<Shadows>()
            .add_systems(Startup, setup_light.after(GeometrySet));
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
        let mut transform = Transform::IDENTITY;
        transform.rotate_x(-self.latitude.to_radians()); // XXX Check this.
        transform.rotate_local_y((15.0 * (self.time - 12.0)).to_radians());
        transform
    }
}

impl Default for Sun {
    fn default() -> Self {
        let illuminance = light_consts::lux::OVERCAST_DAY;
        let latitude = 45.0;
        let time = 12.0;
        let entity = Entity::PLACEHOLDER;
        Self { illuminance, latitude, time, entity }
    }
}
