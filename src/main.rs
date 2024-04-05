pub mod engine;
pub mod gameplay;

use bevy::ecs::{schedule::States, system::Resource};
use bevy::{asset::Handle, sprite::TextureAtlas};
use bevy_asset_loader::asset_collection::AssetCollection;

use engine::loading::{Fonts, Grids, SvarogStates, SvarogTextureAtlases, Tilesets};
use engine::Svarog;
use svarog_macros::*;

#[derive(Default)]
#[svarog_states]
pub enum GameStates {
    #[default]
    #[static_loading]
    StaticLoading,

    #[asset_loading]
    AssetLoading,

    #[setup]
    Setup,

    #[done_loading]
    Game,
}

#[svarog_texture_atlases]
pub struct TextureAtlases {
    #[asset(key = "kenney-colour")]
    pub kenney_colour: Handle<TextureAtlas>,
    #[asset(key = "kenney-mono")]
    pub kenney_mono: Handle<TextureAtlas>,
    #[asset(key = "sourcecodepro")]
    pub sourcecodepro: Handle<TextureAtlas>,
}

pub fn load_static_data(tilesets: &mut Tilesets, fonts: &mut Fonts, grids: &mut Grids) {
    tilesets.add("tilesets.csv", fonts);
    grids.add("grids.csv");
}

pub fn main() {
    Svarog::<TextureAtlases, GameStates>::default()
        .with_loader(load_static_data)
        .run();
}