use bevy::{app::Plugin, asset::Handle, core_pipeline::core_2d::Camera2dBundle, ecs::{component::Component, entity::Entity, query::With, schedule::{NextState, OnEnter, States}, 
    system::{Commands, Query, Res, ResMut, Resource}}, hierarchy::BuildChildren, math::Vec3, render::view::{InheritedVisibility, Visibility}, 
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite}, transform::components::{GlobalTransform, Transform}, 
    utils::hashbrown::HashMap, window::{PrimaryWindow, Window}};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt}, standard_dynamic_asset::StandardDynamicAssetCollection};
use csv::Trim;
use std::{collections::HashSet, fmt::Debug, hash::{DefaultHasher, Hash, Hasher}, marker::PhantomData, sync::{Mutex, OnceLock}};

//use super::{GameAssets, GameStates};

pub trait SvarogStates : States {
    fn static_loading_state() -> Self;
    fn asset_loading_state() -> Self;
    fn setup_state() -> Self;
    fn done_loading_state() -> Self;
}

pub trait SvarogTextureAtlases : AssetCollection + Default {
    fn get(&self, name: &str) -> Option<Handle<TextureAtlas>>;
}

#[derive(Default, Debug)]
pub struct Font {
    pub glyphs: HashMap<String, Glyph>,
    pub attributes: HashMap<String, HashSet<Glyph>>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct PreGlyph {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub attributes: String,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
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
    pub fn add(&mut self, path: &str) {
        let Ok(mut csv) = csv::ReaderBuilder::new()
            .delimiter(b'|')
            .comment(Some(b'#'))
            .trim(Trim::All)
            .from_path(format!("assets/{}", path).as_str()) else { return; };

        let mut font = Font::default();
        for record in csv.deserialize::<PreGlyph>().flatten() {
            let is_quote = record.attributes.starts_with('\"');
            if is_quote {
                let attributes = record.attributes.clone();
                let mut attribs = attributes.chars();
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

                    if font.glyphs.contains_key(&name) {
                        println!("Warning: font overrides previous glyph: {}", name);
                    }

                    font.glyphs.insert(name.clone(), glyph.clone());

                    if !font.attributes.contains_key(&name) {
                        font.attributes.insert(name.clone(), HashSet::new());    
                    }
                    
                    font.attributes.get_mut(&name).unwrap().insert(glyph);
                }
            } else {
                let name = record.name.clone();
                if font.glyphs.contains_key(&name) {
                    println!("Warning: font overrides previous glyph: {}", name);
                }

                if !font.attributes.contains_key(&name) {
                    font.attributes.insert(name.clone(), HashSet::new());
                }
                
                let glyph = Glyph {
                    name: name.clone(),
                    x: record.x,
                    y: record.y,
                    attributes: record.attributes
                        .split(';')
                        .map(&str::trim)
                        .map(&str::to_owned)
                        .collect::<Vec<_>>(),
                };
                
                font.attributes.get_mut(&name).unwrap().insert(glyph.clone());
                font.glyphs.insert(name, glyph);
            }
        }

        self.fonts.insert(path.to_string(), font);
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
        self.entities.get((y * self.width + (x + 1)) as usize)
    }
}

#[derive(Resource, Default, Debug)]
pub struct Grids {
    pub grids: HashMap<String, Grid>,
    pub inputs: HashMap<u64, Vec<Word>>,
}

pub fn strings() -> &'static Mutex<Strings> {
    static STRINGS: OnceLock<Mutex<Strings>> = OnceLock::new();
    STRINGS.get_or_init(|| Mutex::new(Strings::default()))
}

#[derive(Default, Debug)]
pub struct Strings(pub HashMap<u64, String>);

impl Strings {
    fn get(&self, s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    pub fn pass(&mut self, s: &str) -> u64 {
        let id = self.get(s);

        if !self.0.contains_key(&id) {
            self.0.insert(id, s.to_owned());
        }

        return id;
    }

    pub fn out(&mut self, id: u64) -> Option<&String> {
        self.0.get(&id)
    }
}

#[derive(Component)]
pub struct SetGridValue {
    pub tileset: u64,
    pub value: u64,
}

#[derive(Debug)]
enum Token {
    Token(Vec<char>),
    Var(Vec<char>),
}

#[derive(Debug, Clone, Copy)]
pub enum Word {
    Text(u64),
    Var(u64),
}

impl Grids {
    pub fn add(&mut self, path: &str) {
        let Ok(mut csv) = csv::ReaderBuilder::new()
            .delimiter(b'|')
            .comment(Some(b'#'))
            .trim(Trim::All)
            .from_path(format!("assets/{}", path).as_str()) else { return; };

        for record in csv.deserialize::<Grid>().flatten() {
            self.grids.insert(record.name.clone(), record);
        }
    }

    pub fn set(&mut self, commands: &mut Commands, grid: &str, x: i32, y: i32, value: &str) {
        if let Some(grid) = self.grids.get(grid) {
            if let Some(tile_entity) = grid.get(x - 1, grid.height - 1 - y) {
                let mut strings = strings().lock().unwrap();
                if value.len() > 0 {
                    commands.entity(*tile_entity).insert(SetGridValue { tileset: strings.pass(&grid.tileset), value: strings.pass(value) });
                } else {
                    commands.entity(*tile_entity).insert(SetGridValue { tileset: strings.pass(&grid.tileset), value: 0 });
                }
            } else {
                println!("No grid at x, y: {} {}", x, grid.height - 1 - y);
            }
        } else {
            println!("No grid {}", grid);
        }
    }

    pub fn print(&mut self, commands: &mut Commands, grid: &str, x: i32, y: i32, value: &str) {
        let input = { let mut strings = strings().lock().unwrap(); strings.pass(value) };
        let results = self.inputs.get(&input).cloned().unwrap_or(
        {
            let (token, mut results) = value.chars().fold(
                (Token::Token(vec![]), vec![]), 
            |(token, mut parts), next_char| {
                match (token, next_char) {
                    (Token::Token(token), '/') if token.len() == 0 => {
                        (Token::Var(vec![]), parts)
                    },
                    (Token::Token(token), '/') => {
                        let mut strings = strings().lock().unwrap();
                        parts.extend(vec![ Word::Text(strings.pass(&token.iter().collect::<String>())) ]);
                        (Token::Var(vec![]), parts)
                    },
                    (Token::Token(mut token), next_char) => {
                        token.extend(vec![ next_char ]);
                        (Token::Token(token), parts)
                    },
                    (Token::Var(token), '/') if token.len() == 0 => {
                        let mut strings = strings().lock().unwrap();
                        parts.extend(vec![ Word::Text(strings.pass("/")) ]);
                        (Token::Token(vec![]), parts)
                    },
                    (Token::Var(token), '/') => {
                        let mut strings = strings().lock().unwrap();
                        parts.extend(vec![ Word::Var(strings.pass(&token.iter().collect::<String>())) ]);
                        (Token::Token(vec![]), parts)
                    },
                    (Token::Var(mut token), next_char) => {
                        token.extend(vec![ next_char ]);
                        (Token::Var(token), parts)
                    },
                }
            });

            match token {
                Token::Token(token) | Token::Var(token) if token.is_empty() => {},
                Token::Token(token) => { 
                    let mut strings = strings().lock().unwrap();
                    results.extend(vec![ Word::Text(strings.pass(&token.iter().collect::<String>())) ]); 
                },
                Token::Var(token) => { 
                    let mut strings = strings().lock().unwrap();
                    results.extend(vec![ Word::Var(strings.pass(&token.iter().collect::<String>())) ]); 
                },
            }

            results
        });

        let mut index = 0;
        for ch in results.iter() {
            match ch {
                Word::Text(text) => {
                    let str = { 
                        let mut strings = strings().lock().unwrap();
                        strings.out(*text).unwrap().clone()
                    };

                    for c in 0..str.len() {
                        self.set(commands, grid, x + index as i32, y, &str[c..c+1]);
                        index += 1;
                    }
                },

                Word::Var(text) => {
                    let text = { 
                        let mut strings = strings().lock().unwrap();
                        strings.out(*text).unwrap().clone()
                    };
                    self.set(commands, grid, x + index as i32, y, &text);
                    index += 1;
                }
            }
        }
    }

    pub fn rect(&mut self, commands: &mut Commands, grid: &str, x: i32, y: i32, w: i32, h: i32, value: &str) {
        for dx in x..=(x + w) {
            for dy in y..=(y + h) {
                self.set(commands, grid, dx, dy, value);
            }
        }
    }

    //                        0   1   2   3  4  5  6  7  8
    /// Slices go like this: TL, TR, BL, BR, T, B, L, R, M
    pub fn boxed(&mut self, commands: &mut Commands, grid: &str, x: i32, y: i32, w: i32, h: i32, slices: &[&str; 9]) {
        self.rect(commands, grid, x, y, w, h, slices[8]);

        for i in x..=x+w {
            self.set(commands, grid, i, y, slices[4]);
            self.set(commands, grid, i, y+h, slices[5]);
        }

        for j in y..=y+h {
            self.set(commands, grid, x, j, slices[6]);
            self.set(commands, grid, x+w, j, slices[7]);
        }

        self.set(commands, grid, x, y, slices[0]);
        self.set(commands, grid, x+w, y, slices[1]);
        self.set(commands, grid, x, y+h, slices[2]);
        self.set(commands, grid, x+w, y+h, slices[3]);
    }
}

pub struct GridEditor<'a, 'w, 's> {
    pub grids: &'a mut Grids,
    pub commands: &'a mut Commands<'w, 's>,
}

impl<'a, 'w, 's> GridEditor<'a, 'w, 's> {
    pub fn new(commands: &'a mut Commands<'w, 's>, grids: &'a mut Grids) -> Self {
        Self {
            commands, 
            grids,
        }
    }

    pub fn set(&mut self, grid: &str, x: i32, y: i32, value: &str) {
        self.grids.set(self.commands, grid, x, y, value);
    }

    pub fn print(&mut self, grid: &str, x: i32, y: i32, value: &str) {
        self.grids.print(self.commands, grid, x, y, value);
    }

    pub fn rect(&mut self, grid: &str, x: i32, y: i32, w: i32, h: i32, value: &str) {
        self.grids.rect(self.commands, grid, x, y, w - 1, h - 1, value);
    }

    /// Slices go like this: TL, TR, BL, BR, T, B, L, R, M
    pub fn custom_frame(&mut self, grid: &str, x: i32, y: i32, w: i32, h: i32, slices: &[&str; 9]) {
        self.grids.boxed(self.commands, grid, x, y, w - 1, h - 1, slices);
    }

    pub fn frame(&mut self, grid: &str, x: i32, y: i32, w: i32, h: i32) {
        self.custom_frame(grid, x, y, w, h, &[ "topleft", "topright", "bottomleft", "bottomright", "top", "bottom", "left", "right", " " ]);
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Tileset {
    pub name: String,
    pub font: String,
    pub weight: i32,
    pub width: i32,
    pub height: i32,
    pub columns: i32,
    pub rows: i32,
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

        for record in csv.deserialize::<Tileset>().flatten() {
            fonts.add(&record.font);
            self.tilesets.insert(record.name.clone(), record);
        }
    }
}

pub fn start_static_loading<GameStates: SvarogStates>(mut next: ResMut<NextState<GameStates>>) {
    next.set(GameStates::asset_loading_state());
}

#[allow(clippy::type_complexity)]
#[derive(Default)]
pub struct SvarogLoadingPlugin<A: AssetCollection, S: SvarogStates> {
    loader: Option<Box<dyn Fn(&mut Tilesets, &mut Fonts, &mut Grids) + 'static + Sync + Send>>,
    phantom: PhantomData<(A, S)>
}

#[allow(clippy::type_complexity)]
impl<A: AssetCollection, S: SvarogStates> SvarogLoadingPlugin<A, S> {
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

pub fn create_grid_entities<GameAssets: SvarogTextureAtlases, GameStates: SvarogStates>(
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
                                sprite: TextureAtlasSprite { index: 0, ..Default::default() },
                                texture_atlas: assets.get(&tileset.name).unwrap_or_else(|| panic!("NO FONT: {}", tileset.name)),
                                transform: Transform::from_translation(Vec3::new(
                                    ((grid.x + i) * tileset.width) as f32, 
                                    ((if grid.align == GridAlign::None { grid.y } else { 0 } + j) * tileset.height) as f32, 
                                    grid.depth as f32)),
                                visibility: Visibility::Hidden,
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

    next.set(GameStates::done_loading_state());
}

impl<A: SvarogTextureAtlases, S: SvarogStates> Plugin for SvarogLoadingPlugin<A, S> {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut tilesets = Tilesets::default();
        let mut fonts = Fonts::default();
        let mut grids = Grids::default();

        (self.loader.as_ref().expect("Expected loader function"))(&mut tilesets, &mut fonts, &mut grids);
        app.insert_resource(tilesets);
        app.insert_resource(fonts);
        app.insert_resource(grids);
        app.add_systems(OnEnter(S::static_loading_state()), start_static_loading::<S>);
        app.add_systems(OnEnter(S::static_loading_state()), create_camera);
 
        app.add_state::<S>().add_loading_state(
            LoadingState::new(S::asset_loading_state())
                .load_collection::<A>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("resources.assets.ron")
                .continue_to_state(S::setup_state()),
        );

        app.add_systems(OnEnter(S::setup_state()), create_grid_entities::<A, S>);
    }
}