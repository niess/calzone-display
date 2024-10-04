use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use super::nord::NORD;


pub struct Meters {
    x: Meter,
    y: Meter,
    z: Meter,
    speed: Meter,
}

#[derive(Component)]
struct MetersPanel;

impl Meters {
    pub fn new(commands: &mut Commands) -> Self {
        fn spawn_column<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
            commands.spawn(NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                ..default()
            })
        }

        let x = commands.spawn(super::UiText::new_bundle("X")).id();
        let y = commands.spawn(super::UiText::new_bundle("Y")).id();
        let z = commands.spawn(super::UiText::new_bundle("Z")).id();
        let speed = commands.spawn(super::UiText::new_bundle("speed")).id();

        let mut labels = spawn_column(commands);
        labels.push_children(&[x, y, z, speed]);
        let labels = labels.id();

        let x = Meter::new(commands);
        let y = Meter::new(commands);
        let z = Meter::new(commands);
        let speed = Meter::new(commands);

        let mut values = spawn_column(commands);
        values.push_children(&[x.entity, y.entity, z.entity, speed.entity]);
        let values = values.id();

        let mut panel = commands.spawn((
            MetersPanel,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(5.0),
                    right: Val::Px(5.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                background_color: NORD[1].into(),
                border_color: NORD[2].into(),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
        ));
        panel.add_child(labels);
        panel.add_child(values);

        Self { x, y, z, speed }
    }

    pub fn update_speed(&self, value: f32, commands: &mut Commands) {
        self.speed.update(value, commands);
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
