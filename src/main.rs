use bevy::prelude::*;

mod viewer;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(viewer::ViewerPlugin)
        .run();
}
