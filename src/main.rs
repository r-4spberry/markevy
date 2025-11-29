mod components;
mod resources;
mod trading_plugin;
mod systems;

use bevy::prelude::*;
// use bevy_egui::EguiPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins,))
        .add_plugins(trading_plugin::TradingPlugin)
        .run()
}
