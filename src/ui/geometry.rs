use bevy::prelude::*;
use crate::geometry::{Volume, RootVolume};
use super::{setup_ui, UiMenu, UiText};


#[derive(Event)]
pub struct TargetEvent(pub Entity);

pub fn build(app: &mut App) {
    app
        .add_event::<TargetEvent>()
        .add_event::<UpdateEvent>()
        .add_systems(Startup, setup_menu.after(setup_ui))
        .add_systems(Update, (
            on_button,
            on_update.after(on_button)
        ));
}

#[derive(Component)]
struct VolumeMenu;

pub fn setup_menu(
    mut commands: Commands,
    ui: Query<Entity, With<UiMenu>>,
    root: Query<Entity, With<RootVolume>>,
    children: Query<&Children, With<Volume>>,
    volumes: Query<&Volume>,
) {
    let menu = UiMenu::add_column(
        VolumeMenu,
        &mut commands,
        ui
    );
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
                text.sections[0].style.color = UiText::PRESSED.into();
            }
            Interaction::Hovered => {
                text.sections[0].style.color = UiText::HOVERED.into();
            }
            Interaction::None => {
                text.sections[0].style.color = UiText::NORMAL.into();
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
pub struct VolumeButton(Entity);

impl VolumeButton {
    fn spawn_button(
        message: &str,
        volume: Entity,
        commands: &mut Commands,
    ) -> Entity {
        let component = VolumeButton(volume);
        UiText::spawn_button(component, message, commands)
    }
}
