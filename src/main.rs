use bevy::prelude::*;

use config::SvarogConfigPlugin;
use grid::SvarogGridPlugin;
use tiles::SvarogTilePlugin;
use windows::SvarogWindowPlugin;

pub mod config;
pub mod grid;
pub mod tiles;
pub mod windows;

fn main() {
    App::new()
        .add_plugins(SvarogWindowPlugin)
        .add_plugins(SvarogConfigPlugin)
        .add_plugins(SvarogTilePlugin::from_config("tiles.csv"))
        .add_plugins(SvarogGridPlugin::from_config("grids.csv"))
        .run();
}
