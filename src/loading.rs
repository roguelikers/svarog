use bevy::{app::{Plugin, Update}, asset::{Asset, AssetEvent, AssetId, AssetServer, Assets, Handle}, 
    ecs::{event::EventReader, schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter}, 
    system::{Commands, Res, ResMut, Resource}}, reflect::TypePath, utils::{hashbrown::{HashMap, HashSet}}};
use bevy_asset_loader::{
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    standard_dynamic_asset::StandardDynamicAssetCollection,
};
use csv::Trim;
use std::{fmt::Debug, sync::Arc};

use super::{GameAssets, GameStates};

#[derive(Default, Debug)]
pub struct Font {
    pub glyphs: HashMap<String, HashSet<Glyph>>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Glyph {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub attributes: String,
}

#[derive(Resource, Default, Debug)]
pub struct Fonts {
    pub fonts: HashMap<String, Font>,
}

impl Fonts {
    pub fn add(&mut self, name: &str, path: &str) {
        let Ok(mut csv) = csv::ReaderBuilder::new()
            .delimiter(b'|')
            .comment(Some(b'#'))
            .trim(Trim::All)
            .from_path(format!("assets/{}", path).as_str()) else { return; };

        let mut font = Font::default();
        for record in csv.deserialize::<Glyph>() {
            if let Ok(record) = record {
                let name = record.name.clone();
                if !font.glyphs.contains_key(&name) {
                    font.glyphs.insert(name.clone(), HashSet::new());
                }
                font.glyphs.get_mut(&name).unwrap().insert(record);
            }
        }

        self.fonts.insert(name.to_string(), font);
    }
}

pub fn start_static_loading(mut next: ResMut<NextState<GameStates>>) {
    next.set(GameStates::AssetLoading);
}

#[derive(Default)]
pub struct SvarogLoadingPlugin {
    loader: Option<Arc<dyn Fn(&mut Fonts) + 'static + Sync + Send>>,
}

impl SvarogLoadingPlugin {
    pub fn with_loader<F: Fn(&mut Fonts) + 'static + Sync + Send>(mut self, f: F) -> Self {
        self.loader = Some(Arc::new(f));
        self
    }
}
impl Plugin for SvarogLoadingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut fonts = Fonts::default();
        (self.loader.as_ref().expect("Expected loader function"))(&mut fonts);
        app.insert_resource(fonts);

        app.add_systems(OnEnter(GameStates::StaticLoading), start_static_loading);
 
        app.add_state::<GameStates>().add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .load_collection::<GameAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("resources.assets.ron")
                .continue_to_state(GameStates::Setup),
        );
    }
}