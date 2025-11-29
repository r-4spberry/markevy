use bevy::prelude::*;
use rand::prelude::*;
use rand_distr::{Distribution, Normal};

use crate::{
    components::{BuyIntent, SellIntent},
    resources::{DayTimer, Order},
};
const NUM_TRADERS: usize = 50;
const INITIAL_CASH: f64 = 10_000.0;
const INITIAL_SHARES: u32 = 50;
const DAY_ZERO_PRICE: f64 = 100.0;

pub fn setup(mut commands: Commands) {
    commands.insert_resource(crate::resources::Day(0));
    commands.insert_resource(crate::resources::Market {
        last_price: DAY_ZERO_PRICE,
        history: vec![DAY_ZERO_PRICE],
        trades_last_day: 0,
    });
    commands.insert_resource(crate::resources::OrderBook {
        buy_orders: Vec::new(),
        sell_orders: Vec::new(),
    });
    commands.insert_resource(DayTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
    for _ in 0..NUM_TRADERS {
        commands.spawn((crate::components::Trader {
            cash: INITIAL_CASH,
            shares: INITIAL_SHARES,
        },));
    }
}

pub fn tick_day_timer(mut timer: ResMut<DayTimer>, time: Res<Time>) {
    timer.0.tick(time.delta());
}

pub fn tick_day(mut day: ResMut<crate::resources::Day>) {
    day.0 += 1;
}

#[derive(PartialEq)]
enum Intent {
    Hold,
    Sell,
    Buy,
}

const PRICE_DEVIATION: f64 = 5.0;

pub fn generate_intents(
    mut commands: Commands,
    market: Res<crate::resources::Market>,
    query: Query<(Entity, &crate::components::Trader)>,
) {
    let mut rng = rand::rng();
    let price_noise = Normal::new(0., PRICE_DEVIATION).unwrap();
    for (entity, trader) in query {
        let r = rng.random_range(0..3);
        let intent = match r {
            0 => Intent::Hold,
            1 => Intent::Sell,
            2 => Intent::Buy,
            _ => unreachable!(),
        };
        if intent == Intent::Hold {
            continue;
        } else if intent == Intent::Buy {
            let buy_price = market.last_price + price_noise.sample(&mut rng);
            if buy_price < 0.0 {
                continue;
            }
            if buy_price > trader.cash {
                continue;
            }
            let buy_intent = BuyIntent {
                price: buy_price,
                qty: 1,
            };
            commands.entity(entity).insert(buy_intent);
        } else if intent == Intent::Sell {
            let sell_price = market.last_price + price_noise.sample(&mut rng);
            if sell_price < 0.0 {
                continue;
            }
            if trader.shares == 0 {
                continue;
            }
            let sell_intent = SellIntent {
                price: sell_price,
                qty: 1,
            };
            commands.entity(entity).insert(sell_intent);
        }
    }
}

pub fn collect_orders(
    mut order_book: ResMut<crate::resources::OrderBook>,
    query: Query<(
        Entity,
        Option<&crate::components::BuyIntent>,
        Option<&crate::components::SellIntent>,
    )>,
) {
    for (entity, buy, sell) in query {
        if let Some(b) = buy {
            order_book.buy_orders.push(Order {
                trader: entity,
                price: b.price,
                qty: b.qty,
            });
        }
        if let Some(s) = sell {
            order_book.sell_orders.push(Order {
                trader: entity,
                price: s.price,
                qty: s.qty,
            });
        }
    }
}

pub fn match_orders(
    mut order_book: ResMut<crate::resources::OrderBook>,
    mut market: ResMut<crate::resources::Market>,
    mut query: Query<&mut crate::components::Trader>,
) {
    order_book
        .buy_orders
        .sort_by(|a, b| b.price.total_cmp(&a.price));
    order_book
        .sell_orders
        .sort_by(|a, b| a.price.total_cmp(&b.price));

    let buys = &order_book.buy_orders;
    let sells = &order_book.sell_orders;

    let mut i_buys = 0;
    let mut i_sells = 0;

    let mut total_price = 0.0;
    let mut total_number = 0;
    while i_buys < buys.len() && i_sells < sells.len() {
        let buy = &buys[i_buys];
        let sell = &sells[i_sells];

        if buy.qty > 1 || sell.qty > 1 {
            panic!("Not implemented");
        }
        if buy.price < sell.price {
            break;
        }

        let trade_price = (sell.price + buy.price) / 2.0;
        let [mut seller, mut buyer] = query.get_many_mut([sell.trader, buy.trader]).unwrap();

        seller.cash += trade_price;
        buyer.cash -= trade_price;

        seller.shares -= 1;
        buyer.shares += 1;

        total_price += trade_price;
        total_number += 1;

        i_buys += 1;
        i_sells += 1;
    }
    market.trades_last_day = total_number as u32;
    if total_number > 0 {
        market.last_price = total_price / (total_number as f64);
    }
}

pub fn end_day(
    mut commands: Commands,
    mut market: ResMut<crate::resources::Market>,
    mut order_book: ResMut<crate::resources::OrderBook>,
    day: Res<crate::resources::Day>,
    query: Query<Entity, Or<(With<BuyIntent>, With<SellIntent>)>>,
) {
    println!(
        "Day {} | Price: {:.2} | Trades: {} | Buys: {} | Sells: {}",
        day.0,
        market.last_price,
        market.trades_last_day,
        order_book.buy_orders.len(),
        order_book.sell_orders.len()
    );
    
    for entity in query {
        commands.entity(entity).remove::<BuyIntent>();
        commands.entity(entity).remove::<SellIntent>();
    }

    let last_price = market.last_price;
    market.history.push(last_price);

    order_book.buy_orders.clear();
    order_book.sell_orders.clear();
}
