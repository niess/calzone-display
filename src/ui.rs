use bevy::prelude::*;
use bevy::color::palettes::css::*;
use bevy::ecs::system::EntityCommands;
use super::drone::Drone;
use super::geometry::{GeometrySet, Volume, RootVolume};


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_ui.after(GeometrySet))
            .add_systems(Update, on_button);
    }
}

fn setup_ui(
    mut commands: Commands,
    root: Query<Entity, With<RootVolume>>,
    children: Query<&Children, With<Volume>>,
    volumes: Query<&Volume>,
) {
    let root = VolumeEntry::new(root.single());
    let root = root.spawn_tree(&mut commands, &children, &volumes);

    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Auto,
                height: Val::Auto,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            background_color: Color::srgb(0.25, 0.25, 0.25).into(),
            border_color: RED.into(),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
    ))
    .add_child(root);
}

fn on_button(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Parent, &Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<VolumeButton>),
    >,
    mut entry_query: Query<&mut VolumeEntry>,
    children_query: Query<&Children, With<Volume>>,
    volume_query: Query<&Volume>,
    mut transform_query: Query<&mut Transform, With<Drone>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (parent, interaction, mut background, mut border) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background = background_color(PRESSED).into();
                border.0 = border_color(PRESSED).into();

                if keyboard_input.pressed(KeyCode::ShiftLeft) {
                    let entry = entry_query.get(**parent).unwrap();
                    let volume = volume_query.get(entry.volume).unwrap();
                    *transform_query.single_mut() = volume.target(); // XXX Relocate camera?
                } else {
                    let mut entry = entry_query.get_mut(**parent).unwrap();
                    entry.expand = !entry.expand;
                    if entry.expand {
                        let button = entry.spawn_button(
                            &mut commands,
                            &children_query,
                            &volume_query
                        );
                        commands
                            .entity(entry.node)
                            .add_child(button);
                    } else {
                    }
                }
            }
            Interaction::Hovered => {
                *background = background_color(HOVERED).into();
                border.0 = border_color(HOVERED).into();
            }
            Interaction::None => {
                *background = background_color(NORMAL).into();
                border.0 = border_color(NORMAL).into();
            }
        }
    }
}

const NORMAL: [f32; 3] = [0.2, 0.2, 0.2];
const HOVERED: [f32; 3] = [0.3, 0.3, 0.3];
const PRESSED: [f32; 3] = [0.7, 0.7, 0.3];

fn background_color(array: [f32; 3]) -> Color {
    Color::srgb_from_array(array)
}

fn border_color(array: [f32; 3]) -> Color {
    let array: [f32; 3] = std::array::from_fn(|i| { array[i] + 0.1 });
    Color::srgb_from_array(array)
}

#[derive(Component)]
struct VolumeEntry {
    node: Entity,
    volume: Entity,
    expand: bool,
}

#[derive(Component)]
struct VolumeButton;

impl VolumeEntry {
    fn new(volume: Entity) -> Self {
        let node = Entity::PLACEHOLDER;
        let expand = false;
        Self { node, volume, expand }
    }

    fn spawn_tree(
        self,
        commands: &mut Commands,
        children: &Query<&Children, With<Volume>>,
        volumes: &Query<&Volume>,
    ) -> Entity {
        let button = self.spawn_button(commands, children, volumes);
        let root = self.spawn_node(commands, children, volumes);
        commands
            .entity(root)
            .add_child(button)
            .id()
    }

    fn spawn_button(
        &self,
        commands: &mut Commands,
        children: &Query<&Children, With<Volume>>,
        volumes: &Query<&Volume>,
    ) -> Entity {
        let volume = volumes.get(self.volume).unwrap();
        commands.spawn((
            VolumeButton,
            ButtonBundle {
                style: Style {
                    width: Val::Auto,
                    height: Val::Auto,
                    border: UiRect::all(Val::Px(1.0)),
                    margin: UiRect::vertical(Val::Px(2.0)),
                    align_items: AlignItems::Default,
                    ..default()
                },
                background_color: background_color(NORMAL).into(),
                border_color: border_color(NORMAL).into(),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    &volume.name,
                    TextStyle {
                        font_size: 18.0,
                        ..default()
                    }
                )
                .with_style(Style {
                    margin: UiRect::horizontal(Val::Px(6.0)),
                    ..default()
                })
            );
        })
        .id()
    }

    fn spawn_node(
        mut self,
        commands: &mut Commands,
        children: &Query<&Children, With<Volume>>,
        volumes: &Query<&Volume>,
    ) -> Entity {
        let mut daughters = Vec::<Entity>::new();
        if let Ok(childs) = children.get(self.volume) {
            for child in childs.iter() {
                let entry = Self::new(*child);
                daughters.push(entry.spawn_node(commands, children, volumes));
            }
        }

        let mut node = commands.spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        });
        self.node = node.id();
        node.insert(self);

        for daughter in daughters {
            node.add_child(daughter);
        }
        node.id()
    }
}
