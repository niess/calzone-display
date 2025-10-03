use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use crate::view_to_world;
use crate::ui::UiRoot;


pub struct Meters {
    x: Meter,
    y: Meter,
    z: Meter,
    azimuth: Meter,
    elevation: Meter,
    speed: Meter,
    zoom: Meter,
}

impl Meters {
    pub fn new(commands: &mut Commands) -> Self {
        fn spawn_column<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
            commands.spawn(
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(4.0)),
                    ..default()
                }
            )
        }

        let x = commands.spawn(super::UiText::new_bundle("easting")).id();
        let y = commands.spawn(super::UiText::new_bundle("northing")).id();
        let z = commands.spawn(super::UiText::new_bundle("upward")).id();
        let azimuth = commands.spawn(super::UiText::new_bundle("azimuth")).id();
        let elevation = commands.spawn(super::UiText::new_bundle("elevation")).id();
        let speed = commands.spawn(super::UiText::new_bundle("speed")).id();
        let zoom = commands.spawn(super::UiText::new_bundle("zoom")).id();

        let mut labels = spawn_column(commands);
        labels.add_children(&[x, y, z, azimuth, elevation, speed, zoom]);
        let labels = labels.id();

        let x = Meter::new(commands, "m");
        let y = Meter::new(commands, "m");
        let z = Meter::new(commands, "m");
        let azimuth = Meter::new(commands, "deg");
        let elevation = Meter::new(commands, "deg");
        let speed = Meter::new(commands, "m/s");
        let zoom = Meter::new(commands, "");

        let mut values = spawn_column(commands);
        values.add_children(&[ x.entity, y.entity, z.entity, azimuth.entity, elevation.entity,
                                speed.entity, zoom.entity ]);
        let values = values.id();

        let mut content = commands.spawn(
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
            },
        );
        content.add_children(&[labels, values]);
        let content = content.id();

        let mut window = super::UiWindow::new("Drone", super::WindowLocation::TopRight, commands);
        window.add_child(content);
        window.insert(UiRoot);

        Self { x, y, z, azimuth, elevation, speed, zoom }
    }

    pub fn update_speed(&self, value: f32, commands: &mut Commands) {
        self.speed.update(value, commands);
    }

    pub fn update_transform(&self, transform: &Transform, commands: &mut Commands) {
        let r = view_to_world().transform_point3(transform.translation);
        self.x.update(r.x, commands);
        self.y.update(r.y, commands);
        self.z.update(r.z, commands);

        let u = view_to_world().transform_vector3(*transform.forward());
        let phi = u.y.atan2(u.x).to_degrees();
        let theta = u.z.acos().to_degrees();
        self.azimuth.update(90.0 - phi, commands);
        self.elevation.update(90.0 - theta, commands);
    }

    pub fn update_zoom(&self, value: f32, commands: &mut Commands) {
        self.zoom.update(value, commands);
    }
}

struct Meter {
    entity: Entity,
    unit: &'static str,
}

#[derive(Component)]
struct MeterText;

#[derive(Event)]
struct MeterEvent {
    pub value: f32,
    pub unit: &'static str,
}

impl Meter {
    pub fn new(commands: &mut Commands, unit: &'static str) -> Self {
        let entity = commands.spawn((
            MeterText,
            super::UiText::new_bundle("undefined"),
        ))
        .observe(MeterEvent::update)
        .id();

        Self { entity, unit }
    }

    pub fn update(&self, value: f32, commands: &mut Commands) {
        let unit = self.unit;
        commands.trigger_targets(MeterEvent { value, unit }, self.entity);
    }
}

impl MeterEvent {
    fn update(
        trigger: Trigger<Self>,
        mut texts: Query<&mut Text, With<MeterText>>,
    ) {
        let value = trigger.event().value;
        let unit = trigger.event().unit;
        let mut text = texts.get_mut(trigger.target()).unwrap();
        text.0 = format!("{:8.2}  {:^3}", value, unit);
    }
}
