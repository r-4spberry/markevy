
use bevy::prelude::*;
pub struct TradingPlugin;
use crate::{resources::one_second_passed, systems::*};



impl Plugin for TradingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, crate::systems::setup);
        app.add_systems(Update, (
            tick_day,
            generate_intents,
            collect_orders,
            match_orders,
            end_day
        ).chain().run_if(one_second_passed))
        .add_systems(Update, tick_day_timer);
    }
}