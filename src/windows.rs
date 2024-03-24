use std::fs::File;

use bevy::prelude::*;

use bevy::window::PrimaryWindow;
use bevy::window::WindowMode;
use bevy::window::WindowResolution;

use crate::tiles::DefaultTileset;
use crate::tiles::Tilesets;

#[derive(serde::Deserialize)]
struct Config {
    pub title: String,
    pub mode: SvarogWindowMode,
    pub tileset: String,
}

#[derive(PartialEq, Eq, serde::Deserialize)]
pub enum SvarogWindowMode {
    Fullscreen,
    Windowed(u32, u32),
}

#[derive(Resource)]
pub struct SvarogWindowSize(u32, u32);

pub struct SvarogWindowPlugin;

impl Plugin for SvarogWindowPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        let config_file = File::open("config.ron").expect("Failed to open config file");
        let Ok(config): Result<Config, _> = ron::de::from_reader(config_file) else { 
            println!("No config found. Quitting.");
            return;
        };
        
        let mut defaults = DefaultPlugins.build();
        defaults = defaults.set(ImagePlugin::default_nearest());

        if let SvarogWindowMode::Windowed(w, h) = config.mode {
            defaults = defaults.set(WindowPlugin {
                primary_window: Some(Window {
                    title: config.title.clone(),
                    mode: WindowMode::Windowed,
                    resolution: WindowResolution::new(w as f32, h as f32),
                    ..Default::default()
                }),
                ..Default::default()
            });
        } else {
            defaults = defaults.set(WindowPlugin {
                primary_window: Some(Window {
                    title: config.title.clone(),
                    mode: WindowMode::BorderlessFullscreen,
                    ..Default::default()
                }),
                ..Default::default()
            });
        }

        bevy.add_plugins(defaults);
        println!("Default tileset: {:?}", config.tileset.clone());
        bevy.insert_resource(DefaultTileset(config.tileset.clone()));
        
        bevy.add_systems(
            Startup,
            |mut commands: Commands, window: Query<&Window, With<PrimaryWindow>>| {
                let win = window.single();
                commands.insert_resource(SvarogWindowSize(win.width() as u32, win.height() as u32));
            },
        );
    }
}
