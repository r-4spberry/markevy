use bevy::prelude::*;

#[derive(Component)]
pub struct Trader{
    pub cash: f64,
    pub portfolio: Vec<u32>,
}


