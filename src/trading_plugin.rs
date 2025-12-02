
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
pub struct TradingPlugin;
use crate::systems::sim;
use crate::systems::ui;
use crate::resources;

impl Plugin for TradingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sim::setup);
        app.add_systems(Update, (
            sim::tick_day,
            sim::create_orders,
            sim::match_orders,
            sim::end_day
        ).chain().run_if(resources::one_second_passed));
        app.add_systems(Update, sim::tick_day_timer);
        app.add_systems(EguiPrimaryContextPass, ui::ui_root);
    }
}