use bevy::prelude::*;

use tiles::SvarogTilePlugin;
use windows::SvarogWindowPlugin;
use grid::SvarogGridPlugin;

pub mod windows;
pub mod grid;
pub mod tiles;
pub mod config;

fn main() {
    App::new()
        .add_plugins(SvarogWindowPlugin)
        .add_plugins(SvarogTilePlugin::from_config("tiles.csv"))
        .add_plugins(SvarogGridPlugin::from_config("grids.csv"))

        .run();
}
