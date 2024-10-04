use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;


pub struct Meters {
    x: Meter,
    y: Meter,
    z: Meter,
    azimuth: Meter,
    elevation: Meter,
    speed: Meter,
}

impl Meters {
    pub fn new(commands: &mut Commands) -> Self {
        fn spawn_column<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
            commands.spawn(
                NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
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

        let mut labels = spawn_column(commands);
        labels.push_children(&[x, y, z, azimuth, elevation, speed]);
        let labels = labels.id();

        let x = Meter::new(commands);
        let y = Meter::new(commands);
        let z = Meter::new(commands);
        let azimuth = Meter::new(commands);
        let elevation = Meter::new(commands);
        let speed = Meter::new(commands);

        let mut values = spawn_column(commands);
        values.push_children(&[ x.entity, y.entity, z.entity, azimuth.entity, elevation.entity,
                                speed.entity ]);
        let values = values.id();

        let mut content = commands.spawn(
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                ..default()
            },
        );
        content.push_children(&[labels, values]);
        let content = content.id();

        let mut window = super::UiWindow::new("Drone", super::WindowLocation::TopRight, commands);
        window.add_child(content);

        Self { x, y, z, azimuth, elevation, speed }
    }

    pub fn update_speed(&self, value: f32, commands: &mut Commands) {
        self.speed.update(value, commands);
    }

    pub fn update_transform(&self, transform: &Transform, commands: &mut Commands) {
        self.x.update(transform.translation.x, commands);
        self.y.update(transform.translation.y, commands);
        self.z.update(transform.translation.z, commands);

        let r = transform.forward();
        let phi = r.y.atan2(r.x).to_degrees();
        let theta = r.z.acos().to_degrees();
        self.azimuth.update(90.0 - phi, commands);
        self.elevation.update(90.0 - theta, commands);
    }
}

struct Meter {
    entity: Entity,
}

#[derive(Component)]
struct MeterText;

#[derive(Event)]
struct MeterEvent(pub f32);

impl Meter {
    pub fn new(commands: &mut Commands) -> Self {
        let entity = commands.spawn((
            MeterText,
            super::UiText::new_bundle("undefined"),
        ))
        .observe(MeterEvent::update)
        .id();

        Self { entity }
    }

    pub fn update(&self, value: f32, commands: &mut Commands) {
        commands.trigger_targets(MeterEvent(value), self.entity);
    }
}

impl MeterEvent {
    fn update(
        trigger: Trigger<Self>,
        mut texts: Query<&mut Text, With<MeterText>>,
    ) {
        let value = trigger.event().0;
        let mut text = texts.get_mut(trigger.entity()).unwrap();
        text.sections[0].value = format!("{}", value);
    }
}
