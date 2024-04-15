pub mod health;
pub mod gameplay;

use bevy::ecs::component::Component;
use bevy::ecs::schedule::OnEnter;
use bevy::ecs::{schedule::States, system::Resource};
use bevy::math::{vec2, Vec3};
use bevy::render::texture::Image;
use bevy::render::view::Visibility;
use bevy::sprite::{Sprite, SpriteBundle};
use bevy::transform::components::Transform;
use bevy::{asset::Handle, sprite::TextureAtlas};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy::{app::Update, ecs::{schedule::{common_conditions::in_state, IntoSystemConfigs}, 
    system::{Commands, Local, Res, ResMut}}, input::{keyboard::KeyCode, Input}};

use gameplay::random::{Random, Coin, SvarogRandomPlugin};
use noisy_bevy::simplex_noise_2d_seeded;

use svarog_engine::loading::{GridEditor, Fonts, Grids, SvarogStates, SvarogTextureAtlases, Tilesets};
use svarog_engine::Svarog;

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
    #[asset(key = "sourcecodepro")]
    pub sourcecodepro: Handle<TextureAtlas>,
    #[asset(key = "oryx")]
    pub oryx: Handle<TextureAtlas>,
    #[asset(key = "oryx-trans")]
    pub oryx_trans: Handle<TextureAtlas>,
    #[asset(key = "dragon")]
    pub dragon: Handle<Image>,
}

#[derive(Resource)]
pub struct Seed(pub i32);

pub fn load_static_data(tilesets: &mut Tilesets, fonts: &mut Fonts, grids: &mut Grids) {
    tilesets.add("tilesets.csv", fonts);
    grids.add("grids.csv");
}

pub fn draw_ground(mut commands: Commands, mut grids: ResMut<Grids>) {
    let mut grid = GridEditor::new(&mut commands, &mut grids);

    for i in 0..200 {
        for j in 0..200 {
            grid.set("ground", i, j, "empty");
        }
    }

    grid.custom_frame("ground", 100, 100, 10, 5, &[ 
        "full_wall", "full_wall", "wall_brick", "wall_brick",
        "wall_brick", "wall_brick", "full_wall", "full_wall",
        "empty"
    ]);

    grid.set("ground", 102, 104, "door");
    grid.set("ground", 100, 102, "door");

    grid.set("tiles", 101, 101, "hero1");
    grid.set("tiles", 102, 101, "hero2");
    grid.set("tiles", 103, 101, "hero3");
}

pub fn change_random_updates(input: Res<Input<KeyCode>>, mut commands: Commands, mut grids: ResMut<Grids>, mut seed: ResMut<Seed>, mut first: Local<bool>) {
    let mut grid = GridEditor::new(&mut commands, &mut grids);

    if input.just_pressed(KeyCode::Space) || !*first {
        *first = true;
        seed.0 += 1;
        grid.frame("ui_topleft", 0, 0, 50, 5);
        grid.print("ui_topleft", 3, 0, &format!(" COUNT: {} ", seed.0));
        grid.print("ui_topleft", 2, 2, "Press space to regenerate!");
    }
}

#[derive(Component)]
pub struct PictureOverlay;

pub fn main() {
    Svarog::<TextureAtlases, GameStates>::default()
        .with_loader(load_static_data)
        .as_bevy()
        .insert_resource(Seed(1))
        .add_plugins(SvarogRandomPlugin)
        .add_systems(OnEnter(GameStates::Game), |mut commands: Commands, textures: Res<TextureAtlases>, mut grids: ResMut<Grids>| {
            let mut grid = GridEditor::new(&mut commands, &mut grids);

            for i in 0..200 {
                for j in 0..200 {
                    grid.set("tiles", i, j, "");
                }
            }

            // 19x26
            commands.spawn((SpriteBundle {
                    texture: textures.dragon.clone_weak(),
                    sprite: Sprite {
                        color: bevy::render::color::Color::Rgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 0.2 },
                        ..Default::default()
                    },
                    visibility: Visibility::Hidden,
                    transform: Transform::from_xyz(0., 0., -0.5).with_scale(Vec3::new(1.0, 1.0, 1.0)),
                    ..Default::default()
                }, PictureOverlay));
        
        })
        .add_systems(Update, 
            ( 
                draw_ground, change_random_updates
            ).chain().run_if(in_state(GameStates::done_loading_state())))
        .run();
}