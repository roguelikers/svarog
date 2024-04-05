pub mod engine;
pub mod gameplay;

use bevy::ecs::{schedule::States, system::Resource};
use bevy::{asset::Handle, sprite::TextureAtlas};
use bevy_asset_loader::asset_collection::AssetCollection;

use engine::loading::{Fonts, Grids, SvarogStates, SvarogTextureAssets, Tilesets};
use engine::Svarog;
use svarog_macros::{SvarogStates, svarog_states};

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

// impl SvarogStates for GameStates {
//     fn static_loading_state() -> Self { Self::StaticLoading }
//     fn asset_loading_state() -> Self { Self::AssetLoading }
//     fn setup_state() -> Self { Self::Setup }
//     fn done_loading_state() -> Self { Self::Game }
// }

#[derive(AssetCollection, Resource, Default)]
pub struct GameAssets {
    #[asset(key = "kenney-colour")]
    pub kenney_colour: Handle<TextureAtlas>,
    #[asset(key = "kenney-mono")]
    pub kenney_mono: Handle<TextureAtlas>,
    #[asset(key = "sourcecodepro")]
    pub sourcecodepro: Handle<TextureAtlas>,
}

impl SvarogTextureAssets for GameAssets {
    fn get(&self, name: &str) -> Option<Handle<TextureAtlas>> {
        match name {
            "kenney-colour" => Some(self.kenney_colour.clone_weak()),
            "kenney-mono" => Some(self.kenney_mono.clone_weak()),
            "sourcecodepro" => Some(self.sourcecodepro.clone_weak()),
            _ => None
        }
    }
}

pub fn load_fonts(tilesets: &mut Tilesets, fonts: &mut Fonts, grids: &mut Grids) {
    tilesets.add("tilesets.csv", fonts);
    grids.add("grids.csv");
}

pub fn main() {
    Svarog::<GameAssets, GameStates>::default()
        .with_loader(load_fonts)
        .run();
}