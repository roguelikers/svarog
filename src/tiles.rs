use std::{collections::HashMap, fs::File};

use bevy::prelude::*;
use csv::Trim;
use itertools::Itertools;

pub struct SvarogTilePlugin {
    pub config: String,    
}

#[derive(Default, Clone, Debug)]
pub struct InvariantTileset {
    pub name: String,
    pub file: String,
    pub weight: i32,
    pub tile_size: Option<IVec2>,
}

#[derive(serde::Deserialize, Default, Asset, TypePath, Clone, Debug)]
pub struct Tileset {
    pub name: String,
    pub tile_size: IVec2,
    pub variants: HashMap<i32, String>,
}

impl InvariantTileset {
    pub fn from(mapping: HashMap<String, String>) -> Option<Self> {
        let Some(name) = mapping.get(&"name".to_string()) else { println!("NAME! {:?}", mapping); return None; };
        let Some(file) = mapping.get(&"file".to_string()) else { println!("FILE!"); return None; };
        let Some(weight) = mapping.get(&"weight".to_string()) else { println!("WEIGHT!"); return None; };
        let Ok(weight) = weight.parse::<i32>() else { return None; };

        let tile_size = {
            let Some(width) = mapping.get(&"width".to_string()) else { println!("WIDTH!"); return None; };
            let Some(height) = mapping.get(&"height".to_string()) else { println!("HEIGHT!"); return None; };
            
            let w = width.parse::<i32>();
            let h = height.parse::<i32>();

            match (w, h) {
                (Ok(w), Ok(h)) => Some(IVec2::new(w, h)),
                _ => None
            }
        };
        
        Some(InvariantTileset {
            name: name.clone(),
            file: file.clone(),
            weight,
            tile_size
        })
    } 
}

#[derive(Resource, Clone, Debug)]
struct TilesConfig(String);

impl SvarogTilePlugin {
    pub fn from_config(name: &str) -> Self {
        Self { config: name.to_string() }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct Tilesets(pub HashMap<String, Tileset>);

#[derive(Resource)]
pub struct DefaultTileset(pub String);

fn initialize_tilesets(mut commands: Commands, mut tilesets: ResMut<Tilesets>, tiles_config: Res<TilesConfig>) {
    let Ok(file) = File::open(format!("assets/{}", &tiles_config.0)) else {
        println!("Cannot find grid config assets/{}", tiles_config.0);
        return;
    };
    
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b'|')
        .comment(Some(b'#'))
        .double_quote(true)
        .flexible(true)
        .trim(Trim::All)
        .quoting(false)
        .from_reader(file);

    let headers = reader.headers().into_iter().next().unwrap().iter().map(|s| s.to_string()).collect::<Vec<_>>();
    
    let loaded_invariant_tilesets = reader.records().filter_map(|record| {
        let mut mapping = HashMap::new();
        if let Ok(record) = record.as_ref() {
            if record.len() > 1 {
                for (index, header) in headers.iter().enumerate() {
                    mapping.insert(header.clone(), record.get(index).unwrap().to_string());
                }
            }
        }

        InvariantTileset::from(mapping)
    }).collect::<Vec<_>>();

    let mut sizes = HashMap::new();
    let loaded_tilesets = loaded_invariant_tilesets.into_iter()
        .group_by(|it| it.name.clone())
        .into_iter().map(|it| { 

            (it.0.clone(), it.1.map(|f| {
                if let Some(tile_size) = f.tile_size {
                    sizes.insert(it.0.clone(), tile_size);
                }
                
                (f.weight, f.file.clone())
            }).collect_vec())
        }).collect_vec();

    loaded_tilesets.into_iter().for_each(|(key, group)| {
        tilesets.0.insert(key.clone(), Tileset { name: key.clone(), variants: HashMap::from_iter(group), tile_size: *sizes.get(&key).unwrap() });
    });
    
    commands.remove_resource::<TilesConfig>();

    println!("{:?}", tilesets.0);
}

fn check_tilesets(default_tileset: Res<DefaultTileset>, tilesets: Res<Tilesets>) {
    if !tilesets.0.contains_key(&default_tileset.0) {
        println!("Cannot find the default tileset, quitting.");
        panic!();
    }
}

impl Plugin for SvarogTilePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(TilesConfig(self.config.clone()));
        app.init_resource::<Tilesets>();
        app.add_systems(Startup, initialize_tilesets);
        app.add_systems(Startup, check_tilesets.after(initialize_tilesets));
    }
}
