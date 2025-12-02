use crate::resources;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui_plot::{Bar, BarChart, Line, Plot};

pub fn ui_root(
    mut contexts: EguiContexts,
    markets: Res<resources::Markets>,
    order_books: Res<resources::OrderBooks>,
    day: Res<resources::Day>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Optional top bar
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Market Dashboard");
            ui.separator();
            ui.label(format!("Day: {}", day.0));
        });
    });

    // Main content: two columns side by side
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.columns(2, |cols| {
            cols[0].push_id("overview_col", |ui| {
                ui_overview(ui, &markets);
            });

            cols[1].push_id("ladders_col", |ui| {
                ui_order_ladders(ui, &markets, &order_books);
            });
        });
    });
}

fn ui_overview(ui: &mut egui::Ui, markets: &resources::Markets) {
    ui.style_mut().spacing.item_spacing.x = 12.0;
    ui.style_mut().spacing.item_spacing.y = 20.0;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let assets_per_row = 3;
            let total = markets.assets.len();

            for row_start in (0..total).step_by(assets_per_row) {
                ui.columns(assets_per_row, |cols| {
                    for col_idx in 0..assets_per_row {
                        let asset_index = row_start + col_idx;
                        if asset_index >= total {
                            break;
                        }

                        let market = &markets.assets[asset_index];
                        let ui = &mut cols[col_idx];

                        ui.heading(format!("Asset {}", asset_index));
                        ui.label(format!("Last Price: {:.2}", market.last_price));
                        ui.label(format!("Trades: {}", market.trades_last_day));

                        if ui.button("Print History").clicked() {
                            println!("History {}: {:?}", asset_index, market.history);
                        }

                        ui.add_space(4.0);

                        let points: egui_plot::PlotPoints = market
                            .history
                            .iter()
                            .enumerate()
                            .map(|(d, &p)| [d as f64, p])
                            .collect();

                        let bevy_color = market.color;
                        let [r, g, b, a] = bevy_color.to_srgba().to_u8_array();
                        let egui_color = egui::Color32::from_rgba_unmultiplied(r, g, b, a);

                        egui_plot::Plot::new(format!("price_plot_{}", asset_index))
                            .height(200.0)
                            .include_y(0.0)
                            .allow_scroll(false)
                            .allow_drag(false)
                            .show(ui, |plot_ui| {
                                plot_ui.line(
                                    egui_plot::Line::new(format!("Asset {}", asset_index), points)
                                        .color(egui_color),
                                );
                            });
                    }
                });

                ui.add_space(10.0);
            }
        });
}

fn to_step_points(prices: &Vec<f64>) -> egui_plot::PlotPoints<'_> {
    let mut pts = Vec::new();
    for (i, price) in prices.iter().enumerate() {
        let x = i as f64;
        let y = *price;
        if i == 0 {
            pts.push([x, y]);
        } else {
            let prev_y = prices[i - 1];
            pts.push([x, prev_y]);
            pts.push([x, y]);
        }
    }
    if let Some(&last) = prices.last() {
        pts.push([prices.len() as f64, last]);
    }
    pts.into()
}

fn ui_order_ladders(
    ui: &mut egui::Ui,
    markets: &resources::Markets,
    _order_books: &resources::OrderBooks,
) {
    ui.style_mut().spacing.item_spacing.x = 12.0;
    ui.style_mut().spacing.item_spacing.y = 20.0;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let ladders_per_row = 3;
            let total = markets.assets.len();

            for row_start in (0..total).step_by(ladders_per_row) {
                ui.columns(ladders_per_row, |cols| {
                    for col_idx in 0..ladders_per_row {
                        let asset = row_start + col_idx;
                        if asset >= total {
                            break;
                        }

                        let market = &markets.assets[asset];
                        //let book = &order_books.books[asset];
                        let ui = &mut cols[col_idx];

                        ui.heading(format!("Asset \"{}\"", market.symbol));

                        let buys = &market.buy_orders_last_day;
                        let sells = &market.sell_orders_last_day;

                        // Debug print
                        println!(
                            "Asset {}: {} buys, {} sells",
                            asset,
                            buys.len(),
                            sells.len()
                        );

                        // trades reconstruction
                        let mut trades = Vec::new();
                        for j in 0..market.trades_last_day as usize {
                            let t = (buys[j].price + sells[j].price) / 2.0;
                            trades.push(t);
                        }

                        let buy_prices: Vec<f64> = buys.iter().map(|o| o.price).collect();
                        let sell_prices: Vec<f64> = sells.iter().map(|o| o.price).collect();

                        let buy_points = to_step_points(&buy_prices);
                        let sell_points = to_step_points(&sell_prices);
                        let trades_points = to_step_points(&trades);

                        let mut overlap = Vec::new();
                        let n = buys.len().max(sells.len());
                        for j in 0..n {
                            if let (Some(b), Some(s)) = (buys.get(j), sells.get(j)) {
                                if b.price >= s.price {
                                    let x = j as f64 + 0.5;
                                    overlap.push(
                                        Bar::new(x, b.price - s.price)
                                            .width(1.0)
                                            .base_offset(s.price)
                                            .fill(egui::Color32::from_rgba_unmultiplied(
                                                255, 0, 255, 100,
                                            )),
                                    );
                                }
                            }
                        }

                        Plot::new(format!("order_ladder_{}", asset))
                            .height(200.0)
                            .include_y(0.0)
                            .allow_scroll(false)
                            .allow_drag(false)
                            .show(ui, |plot_ui| {
                                if !buys.is_empty() {
                                    plot_ui.line(Line::new("Buys", buy_points));
                                }
                                if !sells.is_empty() {
                                    plot_ui.line(Line::new("Sells", sell_points));
                                }
                                if !trades.is_empty() {
                                    plot_ui.line(Line::new("Trades", trades_points));
                                }
                                if !overlap.is_empty() {
                                    plot_ui.bar_chart(BarChart::new("Overlap", overlap));
                                }
                            });
                    }
                });

                ui.add_space(10.0);
            }
        });
}
