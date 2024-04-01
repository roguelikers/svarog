use bevy::{app::App, asset::{Assets, Handle}, ecs::{schedule::{NextState, OnEnter, States}, system::{Commands, Res, ResMut, Resource}}, sprite::TextureAtlas};
use bevy_asset_loader::asset_collection::AssetCollection;
use loading::{Fonts, SvarogLoadingPlugin};
use windows::SvarogWindowPlugin;

pub mod windows;
pub mod loading;
pub mod tables;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    StaticLoading,
    AssetLoading,
    Setup,
    Game,
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(key = "kenney")]
    pub kenney: Handle<TextureAtlas>,
    #[asset(key = "source_code")]
    pub source_code: Handle<TextureAtlas>,
}

pub fn load_fonts(fonts: &mut Fonts) {
    fonts.add("kenney", "kenney/kenney.font.csv");
    fonts.add("sourcecodepro", "sourcecodepro/sourcecodepro.font.csv");
}

pub fn main() {
    let mut app = App::default();
    app.add_plugins(SvarogWindowPlugin);
    app.add_plugins(SvarogLoadingPlugin::default().with_loader(load_fonts)); 
    app.add_systems(OnEnter(GameStates::Setup), 
    |fonts: Res<Fonts>| { 
        println!("{:?}", fonts);
    });
    app.run();
}