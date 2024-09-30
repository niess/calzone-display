use bevy::prelude::*;
use bevy::color::palettes::css::*;
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
struct VolumeRef (pub Entity);

fn setup_ui(
    mut commands: Commands,
    query: Query<Entity, With<RootVolume>>,
    children: Query<&Children>,
    volumes: Query<&Volume>,
) {
    let mut node = commands.spawn(NodeBundle {
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
    });

    fn add_volume(
        depth: usize,
        entity: Entity,
        parent: &mut ChildBuilder,
        children: &Query<&Children>,
        volumes: &Query<&Volume>,
    ) {
        let volume = volumes.get(entity).unwrap();
        parent.spawn((
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
            VolumeRef(entity.clone()),
        )).with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    format!("{}{}", " ".repeat(depth), &volume.name),
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
        });

        if let Ok(childs) = children.get(entity) {
            for child in childs.iter() {
                add_volume(depth + 1, *child, parent, children, volumes);
            }
        }
    }

    let root = query.single();
    node.with_children(|parent| {
        add_volume(0, root, parent, &children, &volumes);
    });
}

fn on_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor, &VolumeRef),
        (Changed<Interaction>, With<Button>),
    >,
    volume_query: Query<&Volume>,
    mut transform_query: Query<&mut Transform, With<Drone>>,
) {
    for (interaction, mut background, mut border, volume) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background = background_color(PRESSED).into();
                border.0 = border_color(PRESSED).into();

                let volume = volume_query.get(volume.0).unwrap();
                *transform_query.single_mut() = volume.target(); // XXX Relocate camera?
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
