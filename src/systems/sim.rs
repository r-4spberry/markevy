use bevy::prelude::*;
use rand::prelude::*;
use rand_distr::{Distribution, Normal};

use crate::{components::*, resources::*};
const NUM_TRADERS: usize = 50;
const INITIAL_CASH: f64 = 10_000.0;
const INITIAL_SHARES: u32 = 50;
const DAY_ZERO_PRICE: f64 = 0.0;
const COMPANIES_PER_TYPE: usize = 3;
const NUMBER_OF_TYPES: usize = 5;
const TOTAL_ASSETS: usize = COMPANIES_PER_TYPE * NUMBER_OF_TYPES;
pub fn setup(mut commands: Commands) {
    commands.insert_resource(Day(0));
    let mut assets = Vec::new();
    for i in 0..(TOTAL_ASSETS) {
        let company_type = match i / COMPANIES_PER_TYPE {
            0 => CompanyType::Tech,
            1 => CompanyType::Retail,
            2 => CompanyType::Finance,
            3 => CompanyType::Mining,
            4 => CompanyType::Industrial,
            _ => unreachable!(),
        };
        let mut rng: ThreadRng = rand::rng();
        let symbol = format!("COMP{}", i);
        assets.push(AssetMarket {
            symbol,
            last_price: DAY_ZERO_PRICE,
            history: vec![DAY_ZERO_PRICE],
            buy_orders_last_day: Vec::new(),
            sell_orders_last_day: Vec::new(),
            trades_last_day: 0,
            company_type,
            color: Color::srgb(
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
            ),
        });
    }
    commands.insert_resource(Markets { assets });
    commands.insert_resource(OrderBooks {
        books: vec![
            OrderBook {
                buy_orders: Vec::new(),
                sell_orders: Vec::new(),
            };
            TOTAL_ASSETS
        ],
    });

    commands.insert_resource(DayTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
    for _ in 0..NUM_TRADERS {
        commands.spawn((Trader {
            cash: INITIAL_CASH,
            portfolio: vec![INITIAL_SHARES; TOTAL_ASSETS],
        },));
    }
}

pub fn tick_day_timer(mut timer: ResMut<DayTimer>, time: Res<Time>) {
    timer.0.tick(time.delta());
}

pub fn tick_day(mut day: ResMut<Day>) {
    day.0 += 1;
}

#[derive(PartialEq)]
enum IntentType {
    Hold,
    Sell,
    Buy,
}

const PRICE_DEVIATION: f64 = 5.0;

pub fn create_orders(
    markets: Res<Markets>,
    query: Query<(Entity, &Trader)>,
    mut order_books: ResMut<OrderBooks>,
) {
    let mut rng = rand::rng();
    let price_noise = Normal::new(0., PRICE_DEVIATION).unwrap();
    for asset_id in 0..order_books.books.len() {
        order_books.books[asset_id].buy_orders.clear();
        order_books.books[asset_id].sell_orders.clear();

        for (entity, trader) in query {
            let r = rng.random_range(0..3);
            let intent = match r {
                0 => IntentType::Hold,
                1 => IntentType::Sell,
                2 => IntentType::Buy,
                _ => unreachable!(),
            };
            let asset = &markets.assets[asset_id];
            if intent == IntentType::Hold {
                continue;
            } else if intent == IntentType::Buy {
                let buy_price = asset.last_price + price_noise.sample(&mut rng);
                if buy_price < 0.0 {
                    continue;
                }
                if buy_price > trader.cash {
                    continue;
                }
                order_books.books[asset_id].buy_orders.push(Order {
                    trader: entity,
                    price: buy_price,
                    qty: 1,
                });
            } else if intent == IntentType::Sell {
                if trader.portfolio[asset_id] == 0 {
                    continue;
                }
                let sell_price = asset.last_price + price_noise.sample(&mut rng);
                if sell_price < 0.0 {
                    continue;
                }
                order_books.books[asset_id].sell_orders.push(Order {
                    trader: entity,
                    price: sell_price,
                    qty: 1,
                });
            }
        }
    }
}


pub fn match_orders(
    mut order_books: ResMut<OrderBooks>,
    mut markets: ResMut<Markets>,
    mut query: Query<&mut Trader>,
) {
    //sort
    order_books.books.iter_mut().for_each(|order_book| {
        order_book
            .buy_orders
            .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
        order_book
            .sell_orders
            .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
    });

    for asset_id in 0..order_books.books.len() {
        let buys = &order_books.books[asset_id].buy_orders;
        let sells = &order_books.books[asset_id].sell_orders;

        markets.assets[asset_id].buy_orders_last_day = buys.clone();
        markets.assets[asset_id].sell_orders_last_day = sells.clone();

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

            seller.portfolio[asset_id] -= 1;
            buyer.portfolio[asset_id] += 1;

            total_price += trade_price;
            total_number += 1;

            i_buys += 1;
            i_sells += 1;
        }

        markets.assets[asset_id].trades_last_day = total_number as u32;
        if total_number > 0 {
            markets.assets[asset_id].last_price = total_price / (total_number as f64);
        };
    }
}

pub fn end_day(
    mut markets: ResMut<Markets>,
    mut order_books: ResMut<OrderBooks>,
    day: Res<Day>,
) {
    println!("=== END OF DAY {} ===", day.0);


    for (i, (market, book)) in markets
        .assets
        .iter_mut()
        .zip(order_books.books.iter_mut())
        .enumerate()
    {
        println!(
            "Asset {} | Price: {:.2} | Trades: {} | Buys: {} | Sells: {}",
            i,
            market.last_price,
            market.trades_last_day,
            book.buy_orders.len(),
            book.sell_orders.len()
        );

        // Save price history
        market.history.push(market.last_price);

        // Clear order books for next day
        book.buy_orders.clear();
        book.sell_orders.clear();
    }
}

