use bevy::prelude::*;
use bevy::color::palettes::css::*;
use super::geometry::{GeometrySet, Volume, RootVolume};


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_ui.after(GeometrySet));
    }
}

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
            padding: UiRect::all(Val::Px(6.0)),
            margin: UiRect::all(Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
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
        let name = volumes.get(entity).unwrap().name.clone();
        parent.spawn(TextBundle::from_section(
            format!("{}{}", " ".repeat(depth), name),
            TextStyle {
                font_size: 18.0,
                ..default()
            },
        ));

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
