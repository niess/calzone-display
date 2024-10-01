use bevy::prelude::*;
use super::{geometry, UiMenu, UiText};


pub fn build(app: &mut App) {
    app
        .add_systems(Startup, setup_menu.after(geometry::setup_menu));
}

#[derive(Clone, Copy, Component, Default)]
struct LightingMenu {
    expanded: bool,
}

fn setup_menu(
    mut commands: Commands,
    ui: Query<Entity, With<UiMenu>>,
) {
    let component = LightingMenu::default();
    let menu = UiMenu::add_column(
        component,
        &mut commands,
        ui
    );
    component.update_ui(menu, &mut commands);
}

#[derive(Component)]
struct HeaderButton;

impl LightingMenu {
    fn update_ui(&self, menu: Entity, commands: &mut Commands) {
        Self::clear_ui(menu, commands);
        let qualifier = if self.expanded {
            ""
        } else {
            ".."
        };
        let message = format!("Sunlight{}", qualifier);
        let header = UiText::spawn_button(HeaderButton, message.as_str(), commands);
        commands
            .entity(menu)
            .add_child(header);
    }

    fn clear_ui(menu: Entity, commands: &mut Commands) {
        let mut menu = commands.entity(menu);
        menu.despawn_descendants();
    }
}
