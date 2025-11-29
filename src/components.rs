use bevy::prelude::*;

#[derive(Component)]
pub struct Trader{
    pub cash: f64,
    pub shares: u32,
}

#[derive(Component)]
pub struct BuyIntent{
    pub price: f64,
    pub qty: u32,
}

#[derive(Component)]
pub struct SellIntent{
    pub price: f64,
    pub qty: u32,
}