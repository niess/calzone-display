use bevy::prelude::*;
use super::geometry::GeometrySet;


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_ui.after(GeometrySet));
    }
}

fn setup_ui(mut commands: Commands) {
    const FONT_SIZE: f32 = 20.0;

    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(200.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::srgb(0.25, 0.25, 0.25).into(),
        ..default()
    })
    .with_children(|parent| {
        // Title
        parent.spawn((
            TextBundle::from_section(
                "Volumes",
                TextStyle {
                    font_size: FONT_SIZE,
                    ..default()
                },
            ),
            Label,
        ));
    });
}
