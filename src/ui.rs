use bevy::prelude::*;
use bevy::color::palettes::css::*;
use super::geometry::{GeometrySet, Volume, RootVolume};


pub struct UiPlugin;

#[derive(Event)]
pub struct TargetEvent(pub Entity);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<TargetEvent>()
            .add_event::<UpdateEvent>()
            .add_systems(Startup, setup_ui.after(GeometrySet))
            .add_systems(Update, (on_button, on_update.after(on_button)));
    }
}

#[derive(Component)]
struct VolumeMenu;

fn setup_ui(
    mut commands: Commands,
    root: Query<Entity, With<RootVolume>>,
    children: Query<&Children, With<Volume>>,
    volumes: Query<&Volume>,
) {
    let menu = commands.spawn((
        VolumeMenu,
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
    )).id();
    update_ui(
        menu,
        &mut commands,
        &root,
        &children,
        &volumes,
    );
}

#[derive(Event)]
struct UpdateEvent(Entity);

fn on_button(
    interactions: Query<(&Interaction, &VolumeButton, &Children), Changed<Interaction>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_query: Query<&mut Text>,
    mut ev_target: EventWriter<TargetEvent>,
    mut ev_update: EventWriter<UpdateEvent>,
) {
    for (interaction, button, children) in interactions.iter() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                if keyboard_input.pressed(KeyCode::ShiftLeft) {
                    ev_target.send(TargetEvent(button.0));
                } else {
                    ev_update.send(UpdateEvent(button.0));
                }
                text.sections[0].style.color = VolumeButton::PRESSED.into();
            }
            Interaction::Hovered => {
                text.sections[0].style.color = VolumeButton::HOVERED.into();
            }
            Interaction::None => {
                text.sections[0].style.color = VolumeButton::NORMAL.into();
            }
        }
    }
}

fn on_update(
    mut commands: Commands,
    mut events: EventReader<UpdateEvent>,
    menu: Query<Entity, With<VolumeMenu>>,
    root: Query<Entity, With<RootVolume>>,
    children: Query<&Children, With<Volume>>,
    mut volumes: Query<&mut Volume>,
) {
    for event in events.read() {
        let mut volume = volumes.get_mut(event.0).unwrap();
        volume.expanded = !volume.expanded;
        update_ui(
            menu.single(),
            &mut commands,
            &root,
            &children,
            &volumes.to_readonly(),
        );
    }
}

fn update_ui(
    menu: Entity,
    commands: &mut Commands,
    root: &Query<Entity, With<RootVolume>>,
    children: &Query<&Children, With<Volume>>,
    volumes: &Query<&Volume>,
) {
    fn add_button(
        depth: usize,
        entity: Entity,
        menu: Entity,
        commands: &mut Commands,
        children: &Query<&Children, With<Volume>>,
        volumes: &Query<&Volume>,
    ) {
        let volume = volumes.get(entity).unwrap();
        let childs = children.get(entity).ok();
        let qualifier = if childs.is_some() && !volume.expanded {
            ".."
        } else {
            ""
        };
        let label = format!("{}{}{}", "  ".repeat(depth), volume.name, qualifier);
        let button = VolumeButton::spawn_button(label.as_str(), entity, commands);
        commands
            .entity(menu)
            .add_child(button);
        if volume.expanded {
            if let Some(childs) = childs {
                for child in childs {
                    add_button(depth + 1, *child, menu, commands, children, volumes);
                }
            }
        }
    }

    clear_ui(menu, commands);
    add_button(0, root.single(), menu, commands, children, volumes);
}

fn clear_ui(menu: Entity, commands: &mut Commands) {
    let mut menu = commands.entity(menu);
    menu.despawn_descendants();
}

#[derive(Component)]
struct VolumeButton(Entity);

impl VolumeButton {

    const NORMAL: Color = Color::srgb(1.0, 1.0, 1.0);
    const HOVERED: Color = Color::srgb(1.0, 1.0, 0.3);
    const PRESSED: Color = Color::srgb(0.3, 0.3, 0.3);

    fn spawn_button(
        name: &str,
        volume: Entity,
        commands: &mut Commands,
    ) -> Entity {
        commands.spawn((
            VolumeButton(volume),
            ButtonBundle {
                style: Style {
                    border: UiRect::all(Val::Px(1.0)),
                    margin: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    name,
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
            );
        })
        .id()
    }
}
