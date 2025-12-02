mod components;
mod resources;
mod systems;
mod trading_plugin;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin};


fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins,))
        .add_plugins(trading_plugin::TradingPlugin)
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_camera_system)
        .run()
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

