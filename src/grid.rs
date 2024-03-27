use std::{collections::HashMap, fs::File};

use bevy::prelude::*;
use csv::Trim;

#[derive(serde::Deserialize, Default, Clone, Copy, Debug)]
pub enum GridKind {
    #[default]
    Boolean,
    Entity,
    Glyph,
    Number,
}

impl GridKind {
    pub fn from(s: &str) -> Option<GridKind> {
        if s == "boolean" {
            Some(GridKind::Boolean)
        } else if s == "entity" {
            Some(GridKind::Entity)
        } else if s == "number" {
            Some(GridKind::Number)
        } else if s == "glyph" {
            Some(GridKind::Glyph)
        } else {
            None
        }
    }
}

#[derive(serde::Deserialize, Default, Asset, TypePath, Clone, Debug)]
pub struct Grid {
    pub name: String,
    pub dimensions: IVec2,
    pub grid_kind: GridKind,
    pub tileset: Option<String>,
}

impl Grid {
    pub fn from(mapping: HashMap<String, String>) -> Option<Self> {
        let Some(name) = mapping.get(&"name".to_string()) else {
            println!("NAME! {:?}", mapping);
            return None;
        };
        let Some(kind) = mapping.get(&"kind".to_string()) else {
            println!("KIND!");
            return None;
        };
        let Some(kind) = GridKind::from(kind.as_str()) else {
            println!("KIND?");
            return None;
        };
        let tileset = mapping.get(&"tileset".to_string());

        let Some(grid_w) = mapping.get(&"grid size x".to_string()) else {
            println!("GW");
            return None;
        };
        let Ok(grid_w) = grid_w.parse::<i32>() else {
            println!("GW?");
            return None;
        };
        let Some(grid_h) = mapping.get(&"grid size y".to_string()) else {
            println!("GH");
            return None;
        };
        let Ok(grid_h) = grid_h.parse::<i32>() else {
            println!("GH?");
            return None;
        };

        Some(Grid {
            name: name.clone(),
            grid_kind: kind,
            dimensions: IVec2::new(grid_w, grid_h),
            tileset: tileset.cloned(),
        })
    }
}

#[derive(Default)]
pub struct SvarogGridPlugin {
    pub config: String,
}

impl SvarogGridPlugin {
    pub fn from_config(name: &str) -> Self {
        Self {
            config: name.to_string(),
        }
    }
}

#[derive(Resource, Clone, Debug)]
struct GridsConfig(String);

#[derive(Resource, Default, Clone, Debug)]
pub struct Grids(Vec<Grid>);

fn initialize_grids(
    mut commands: Commands,
    mut grids: ResMut<Grids>,
    grid_config: Res<GridsConfig>,
) {
    let Ok(file) = File::open(format!("assets/{}", &grid_config.0)) else {
        println!("Cannot find grid config assets/{}", grid_config.0);
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

    let headers = reader
        .headers()
        .into_iter()
        .next()
        .unwrap()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let loaded_grids = reader
        .records()
        .filter_map(|record| {
            let mut mapping = HashMap::new();
            if let Ok(record) = record.as_ref() {
                if record.len() > 1 {
                    for (index, header) in headers.iter().enumerate() {
                        mapping.insert(header.clone(), record.get(index).unwrap().to_string());
                    }
                }
            }

            Grid::from(mapping)
        })
        .collect::<Vec<_>>();

    grids.0.extend_from_slice(loaded_grids.as_slice());
    commands.remove_resource::<GridsConfig>();
}

impl Plugin for SvarogGridPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(GridsConfig(self.config.clone()));
        app.init_resource::<Grids>();
        app.add_systems(Startup, initialize_grids);
    }
}
