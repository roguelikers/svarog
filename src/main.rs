use bevy::{app::{App, Update}, asset::Handle, core_pipeline::core_2d::{Camera2d, Camera2dBundle}, ecs::{query::With, schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter, States}, system::{Commands, Query, Res, ResMut, Resource}}, hierarchy::BuildChildren, input::{keyboard::KeyCode, Input}, math::{Quat, Vec3}, render::view::{InheritedVisibility, Visibility}, sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite}, transform::components::{GlobalTransform, Transform}};
use bevy_asset_loader::asset_collection::AssetCollection;
use loading::{CameraTag, Fonts, Grids, SvarogLoadingPlugin, Tilesets};
use windows::SvarogWindowPlugin;

use crate::loading::GridKind;

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
    #[asset(key = "kenney_colour")]
    pub kenney_colour: Handle<TextureAtlas>,
    #[asset(key = "kenney_mono")]
    pub kenney_mono: Handle<TextureAtlas>,
    #[asset(key = "source_code")]
    pub source_code: Handle<TextureAtlas>,
}

impl GameAssets {
    pub fn get(&self, name: &str) -> Option<Handle<TextureAtlas>> {
        match name {
            "kenney-colour" => Some(self.kenney_colour.clone_weak()),
            "kenney-mono" => Some(self.kenney_mono.clone_weak()),
            "sourcecodepro" => Some(self.source_code.clone_weak()),
            _ => None
        }
    }
}

pub fn load_fonts(tilesets: &mut Tilesets, fonts: &mut Fonts, grids: &mut Grids) {
    tilesets.add("tilesets.csv", fonts);
    grids.add("grids.csv");
}

pub fn move_camera(mut camera: Query<&mut Transform, With<CameraTag>>, input: Res<Input<KeyCode>>) {
    let mut dir = Vec3::ZERO;
    if input.pressed(KeyCode::Left) {
        dir.x = -5.0;
    } else if input.pressed(KeyCode::Right) {
        dir.x = 5.0;
    }

    if input.pressed(KeyCode::Up) {
        dir.y = -5.0;
    } else if input.pressed(KeyCode::Down) {
        dir.y = 5.0;
    }

    if let Ok(mut camera) = camera.get_single_mut() {
        camera.translation += dir;
    }
}

pub fn main() {
    let mut app = App::default();
    app.add_plugins(SvarogWindowPlugin);
    app.add_plugins(SvarogLoadingPlugin::default()
        .with_loader(load_fonts));
    app.add_systems(Update, move_camera.run_if(in_state(GameStates::Game)));
    app.run();
}