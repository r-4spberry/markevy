use bevy::prelude::*;

#[derive(Resource)]
pub struct Day(pub u32);

// #[derive(Resource)]
// pub struct Market {
//     pub last_price: f64,
//     pub history: Vec<f64>,
//     pub trades_last_day: u32,
//     pub buy_orders_last_day: Vec<Order>,
//     pub sell_orders_last_day: Vec<Order>,
// }

#[derive(Clone)]
pub struct Order {
    pub trader: Entity,
    pub price: f64,
    pub qty: u32,
}


#[derive(Resource)]
pub struct OrderBooks {
    pub books: Vec<OrderBook>, // one per asset
}

#[derive(Clone)]
pub struct OrderBook {
    pub buy_orders: Vec<Order>,
    pub sell_orders: Vec<Order>,
}

#[derive(Resource)]
pub struct DayTimer(pub Timer);

pub fn one_second_passed(timer: Res<DayTimer>) -> bool {
    timer.0.just_finished()
}   

#[derive(Resource)]
pub struct Markets {
    pub assets: Vec<AssetMarket>,
}

pub enum CompanyType {
    Tech,
    Retail,
    Finance,
    Mining,
    Industrial,
}

pub struct AssetMarket {
    pub symbol: String,
    pub last_price: f64,
    pub history: Vec<f64>,
    pub buy_orders_last_day: Vec<Order>,
    pub sell_orders_last_day: Vec<Order>,
    pub trades_last_day: u32,
    pub company_type: CompanyType,
    pub color: Color,
}