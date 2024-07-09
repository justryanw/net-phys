#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use crate::client::ExampleClientPlugin;
use crate::server::ExampleServerPlugin;
use crate::shared::SharedPlugin;
use bevy::prelude::*;
use lightyear::prelude::client::PredictionConfig;
use serde::{Deserialize, Serialize};

mod client;
mod protocol;
mod server;
mod shared;
mod app;
mod settings;

fn main() {
    let cli = Cli::default();
    let settings_str = include_str!("../assets/settings.ron");
    let settings = read_settings::<MySettings>(settings_str);
    let mut apps = Apps::new(settings.common, cli);
    apps.update_lightyear_client_config(|config| {
        config.prediction.minimum_input_delay_ticks = settings.input_delay_ticks;
        config.prediction.correction_ticks_factor = settings.correction_ticks_factor;
    })
    .add_lightyear_plugins()
    .add_user_plugins(
        ExampleClientPlugin,
        ExampleServerPlugin {
            predict_all: settings.predict_all,
        },
        SharedPlugin {
            show_confirmed: settings.show_confirmed,
        },
    );
    // run the app
    apps.run();
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MySettings {
    pub common: Settings,
    pub(crate) predict_all: bool,
    pub(crate) input_delay_ticks: u16,
    pub(crate) correction_ticks_factor: f32,
    pub(crate) show_confirmed: bool,
}