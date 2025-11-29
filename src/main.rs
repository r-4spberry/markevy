mod components;
mod resources;
mod systems;
mod trading_plugin;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins,))
        .add_plugins(trading_plugin::TradingPlugin)
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, ui_test)
        .run()
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn ui_test(mut contexts: EguiContexts, market: Res<resources::Market>, day: Res<resources::Day>) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Market Info").show(ctx, |ui| {
            ui.label(format!("Day: {}", day.0));
            ui.label(format!("Last Price: {:.2}", market.last_price));
            ui.label(format!("Trades Last Day: {}", market.trades_last_day));
            if ui.button("Print Price History").clicked() {
                println!("Price History: {:?}", market.history);
            }
        });

        //draw price history chart
        egui::Window::new("Price").show(ctx, |ui| {
        use egui_plot::{Line, Plot, PlotPoints};
        let points: PlotPoints = market
            .history
            .iter()
            .enumerate()
            .map(|(i, &p)| [i as f64, p as f64])
            .collect();

        Plot::new("price_plot")
            .height(220.0)
            .width(420.0)
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new("Price", points));
            });
    });
    }
}
