use bevy::prelude::*;
use bevy::color::palettes::css::*;
use super::geometry::GeometrySet;

mod geometry;
mod lighting;

pub use geometry::TargetEvent;


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui.after(GeometrySet));
        geometry::build(app);
        lighting::build(app);
    }
}

#[derive(Component)]
struct UiMenu;

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        UiMenu,
        NodeBundle {
            style: Style {
                width: Val::Auto,
                height: Val::Auto,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            ..default()
        },
    ));
}

impl UiMenu {
    fn add_column<T>(
        component: T,
        commands: &mut Commands,
        ui: Query<Entity, With<Self>>,
    ) -> Entity
    where
        T: Component,
    {
        let column = commands.spawn((
            component,
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
                ..default()
            },
        )).id();
        commands
            .entity(ui.single())
            .add_child(column);
        column
    }
}

struct UiText;

impl UiText {
    const NORMAL: Color = Color::srgb(1.0, 1.0, 1.0);
    const HOVERED: Color = Color::srgb(1.0, 1.0, 0.3);
    const PRESSED: Color = Color::srgb(0.3, 0.3, 0.3);

    fn new_bundle(message: &str) -> TextBundle {
        TextBundle::from_section(
            message,
            TextStyle {
                font_size: 18.0,
                color: Self::NORMAL.into(),
                ..default()
            }
        )
        .with_style(Style {
            margin: UiRect::horizontal(Val::Px(6.0)),
            ..default()
        })
    }

    fn spawn_button<T>(
        component: T,
        message: &str,
        commands: &mut Commands,
    ) -> Entity
    where
        T: Component,
    {
        commands.spawn((
            component,
            ButtonBundle {
                style: Style {
                    margin: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(Self::new_bundle(message));
        })
        .id()
    }
}
