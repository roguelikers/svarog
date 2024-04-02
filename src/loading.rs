use bevy::{app::Plugin, core_pipeline::core_2d::Camera2dBundle, ecs::{component::Component, entity::Entity, query::With, schedule::{IntoSystemConfigs, NextState, OnEnter}, system::{Commands, Query, Res, ResMut, Resource}}, hierarchy::BuildChildren, math::{IVec2, Vec3}, render::view::{InheritedVisibility, Visibility}, sprite::{SpriteSheetBundle, TextureAtlasSprite}, transform::components::{GlobalTransform, Transform}, utils::hashbrown::HashMap, window::{PrimaryWindow, Window}};
use bevy_asset_loader::{
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    standard_dynamic_asset::StandardDynamicAssetCollection,
};
use csv::Trim;
use itertools::Itertools;
use std::{collections::HashSet, fmt::Debug};

use super::{GameAssets, GameStates};

#[derive(Default, Debug)]
pub struct Font {
    pub glyphs: HashMap<String, HashSet<Glyph>>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct PreGlyph {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub attributes: String,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Glyph {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub attributes: Vec<String>,
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
        for record in csv.deserialize::<PreGlyph>() {
            if let Ok(record) = record {
                let is_quote = record.attributes.starts_with("\"");
                if is_quote {
                    let attributes = record.attributes.clone();
                    let mut attribs = attributes.chars().into_iter();
                    attribs.next_back();
                    attribs.next();
                    for (dx, letter) in attribs.enumerate() {
                        let mut name = letter.to_string();
                        if name.as_str() == "€" {
                            name = "|".to_string();
                        }
                        let glyph = Glyph {
                            name: name.clone(),
                            x: record.x + dx as i32,
                            y: record.y, 
                            attributes: vec![ name.clone() ],
                        };

                        if !font.glyphs.contains_key(&name) {
                            font.glyphs.insert(name.clone(), HashSet::new());
                        }
                        font.glyphs.get_mut(&name).unwrap().insert(glyph);
                    }
                } else {
                    let name = record.name.clone();
                    if !font.glyphs.contains_key(&name) {
                        font.glyphs.insert(name.clone(), HashSet::new());
                    }
                    let glyph = Glyph {
                        name: name.clone(),
                        x: record.x,
                        y: record.y,
                        attributes: record.attributes
                            .split(';')
                            .map(&str::trim)
                            .map(&str::to_owned)
                            .collect_vec(),
                    };
                    font.glyphs.get_mut(&name).unwrap().insert(glyph);
                }
            }
        }

        self.fonts.insert(name.to_string(), font);
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum GridKind {
    Glyph,
    Entity,
    Boolean,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum GridAlign {
    None,
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

pub type AlignFn = Box<dyn Fn(f32, f32, f32, f32) -> f32>;

impl Grid {
    pub fn align(&self, tileset: &Tileset, window: &Window) -> Option<Vec3> {
        let (window_width_in_px, window_height_in_px) = (window.width(), window.height());
        let grid_width_in_chars = self.width as f32 * tileset.width as f32;
        let grid_height_in_chars = self.height as f32 * tileset.height as f32;
        let grid_offset_x_in_chars = (self.x * tileset.width) as f32;
        let grid_offset_y_in_chars = (self.y * tileset.height) as f32;
        let (one_char, one_row) = (tileset.width as f32, tileset.height as f32);

        fn left(window_width_in_px: f32, one_char: f32, _grid_width_in_chars: f32, grid_offset_x_in_chars: f32) -> f32 {
            (-window_width_in_px + one_char) * 0.5 + grid_offset_x_in_chars
        }

        fn right(window_width_in_px: f32, _one_char: f32, grid_width_in_chars: f32, grid_offset_x_in_chars: f32) -> f32 {
            window_width_in_px * 0.5 - grid_width_in_chars - grid_offset_x_in_chars
        }
        
        fn top(window_height_in_px: f32, one_row: f32, grid_height_in_chars: f32, grid_offset_y_in_chars: f32) -> f32 {
            (window_height_in_px + one_row) * 0.5 - grid_height_in_chars - grid_offset_y_in_chars
        }

        fn bottom(window_height_in_px: f32, one_row: f32, _grid_height_in_chars: f32, grid_offset_y_in_chars: f32) -> f32 {
            -window_height_in_px * 0.5 + grid_offset_y_in_chars + 0.5 * one_row
        }

        fn hor_center(_window_width_in_px: f32, one_char: f32, grid_width_in_chars: f32, grid_offset_x_in_chars: f32) -> f32 {
            (-grid_width_in_chars + one_char) * 0.5 + grid_offset_x_in_chars
        }

        fn ver_center(_window_height_in_px: f32, one_row: f32, grid_height_in_chars: f32, grid_offset_y_in_chars: f32) -> f32 {
            (-grid_height_in_chars + one_row) * 0.5 + grid_offset_y_in_chars
        }

        let hor_ver: Option<(AlignFn, AlignFn)> = match self.align {
            GridAlign::None => None,
            GridAlign::TopLeft => Some((Box::new(left), Box::new(top))), 
            GridAlign::BottomLeft => Some((Box::new(left), Box::new(bottom))),
            GridAlign::TopRight => Some((Box::new(right), Box::new(top))),
            GridAlign::BottomRight => Some((Box::new(right), Box::new(bottom))),
            GridAlign::Top => Some((Box::new(hor_center), Box::new(top))),
            GridAlign::Bottom => Some((Box::new(hor_center), Box::new(bottom))),
            GridAlign::Left => Some((Box::new(left), Box::new(ver_center))),
            GridAlign::Right => Some((Box::new(right), Box::new(ver_center))),
            GridAlign::Center => Some((Box::new(hor_center), Box::new(ver_center))),
        };

        hor_ver.map(|(h, v)| {
            Vec3::new(
                h(window_width_in_px, one_char, grid_width_in_chars, grid_offset_x_in_chars), 
                v(window_height_in_px, one_row, grid_height_in_chars, grid_offset_y_in_chars),
                0.0
            )
        })
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Grid {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub x: i32,
    pub y: i32,
    pub kind: GridKind,
    pub tileset: String,
    pub align: GridAlign,
    #[serde(skip_deserializing)]
    pub entities: Vec<Entity>,
    #[serde(skip_deserializing)]
    pub entity: Option<Entity>,
}

impl Grid {
    pub fn get(&self, x: i32, y: i32) -> Option<&Entity> {
        self.entities.get(((self.x + y) * self.width + (self.y + x)) as usize)
    }
}

#[derive(Resource, Default, Debug)]
pub struct Grids {
    pub grids: HashMap<String, Grid>,
}

impl Grids {
    pub fn add(&mut self, path: &str) {
        let Ok(mut csv) = csv::ReaderBuilder::new()
            .delimiter(b'|')
            .comment(Some(b'#'))
            .trim(Trim::All)
            .from_path(format!("assets/{}", path).as_str()) else { return; };

        for record in csv.deserialize::<Grid>() {
            if let Ok(record) = record {
                println!("{:?}", record);
                self.grids.insert(record.name.clone(), record);
            }
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Tileset {
    pub name: String,
    pub font: String,
    pub weight: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Resource, Default, Debug)]
pub struct Tilesets {
    pub tilesets: HashMap<String, Tileset>,
}

impl Tilesets {
    pub fn add(&mut self, path: &str, fonts: &mut Fonts) {
        let Ok(mut csv) = csv::ReaderBuilder::new()
            .delimiter(b'|')
            .comment(Some(b'#'))
            .trim(Trim::All)
            .from_path(format!("assets/{}", path).as_str()) else { return; };

        for record in csv.deserialize::<Tileset>() {
            if let Ok(record) = record {
                fonts.add(&record.name, &record.font);
                self.tilesets.insert(record.name.clone(), record);
            }
        }
    }
}

pub fn start_static_loading(mut next: ResMut<NextState<GameStates>>) {
    next.set(GameStates::AssetLoading);
}

#[derive(Default)]
pub struct SvarogLoadingPlugin {
    loader: Option<Box<dyn Fn(&mut Tilesets, &mut Fonts, &mut Grids) + 'static + Sync + Send>>,
}

impl SvarogLoadingPlugin {
    pub fn with_loader<F: Fn(&mut Tilesets, &mut Fonts, &mut Grids) + 'static + Sync + Send>(mut self, f: F) -> Self {
        self.loader = Some(Box::new(f));
        self
    }
}

#[derive(Component)]
pub struct GridTag(pub String);

#[derive(Component)]
pub struct CameraTag;

pub fn create_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
            ..Default::default()
        }, 
        Visibility::Visible,
        InheritedVisibility::default(),
        CameraTag));
}

pub fn create_grid_entities(
    mut commands: Commands, 
    mut grids: ResMut<Grids>,
    assets: Res<GameAssets>, 
    tilesets: Res<Tilesets>, 
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<Entity, With<CameraTag>>,
    mut next: ResMut<NextState<GameStates>>) {

    let Ok(window) = window.get_single() else { println!("NO WINDOW!"); return; };
    let Ok(camera) = camera.get_single() else { println!("NO CAMERA!"); return; };

    for (_, grid) in &mut grids.grids {
        if grid.kind == GridKind::Glyph {
            let Some(tileset) = tilesets.tilesets.get(&grid.tileset) else { 
                println!("NO TILESET: {}", grid.tileset);
                return; 
            };

            let (pos, camera_aligned) = {
                if let Some(pos) = grid.align(tileset, window) {
                    (pos, true)
                } else {
                    (Vec3::ZERO, false)
                }
            };

            let id = commands
                .spawn((
                    GridTag(grid.name.to_string()),
                    Transform::from_translation(pos),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                ))
                .with_children(|f| {
                    for j in 0..grid.height {
                        for i in 0..grid.width {
                            let handle = f.spawn((SpriteSheetBundle {
                                sprite: TextureAtlasSprite { index: (i as usize + j as usize) % 90, ..Default::default() },
                                texture_atlas: assets.get(&tileset.name).expect(&format!("NO FONT: {}", tileset.name)),
                                transform: Transform::from_translation(Vec3::new(
                                    (i * tileset.width) as f32, 
                                    (j * tileset.height) as f32, 
                                    grid.depth as f32)),
                                visibility: Visibility::Visible,
                                ..Default::default()
                            },)).id();
    
                            grid.entities.push(handle);
                        }
                    }
                }).id();

            grid.entity = Some(id);

            if camera_aligned {
                commands.entity(camera).push_children(&[id]);
            }
        }
    }

    next.set(GameStates::Game);
}

impl Plugin for SvarogLoadingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut tilesets = Tilesets::default();
        let mut fonts = Fonts::default();
        let mut grids = Grids::default();
        (self.loader.as_ref().expect("Expected loader function"))(&mut tilesets, &mut fonts, &mut grids);
        app.insert_resource(tilesets);
        app.insert_resource(fonts);
        app.insert_resource(grids);

        app.add_systems(OnEnter(GameStates::StaticLoading), start_static_loading);
        app.add_systems(OnEnter(GameStates::StaticLoading), create_camera);
 
        app.add_state::<GameStates>().add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .load_collection::<GameAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("resources.assets.ron")
                .continue_to_state(GameStates::Setup),
        );

        app.add_systems(OnEnter(GameStates::Setup), create_grid_entities);
    }
}