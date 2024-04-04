use std::marker::PhantomData;

use bevy::{app::{Plugin, Update}, ecs::{entity::Entity, schedule::{common_conditions::in_state, IntoSystemConfigs}, system::{Commands, Local, Query, Res, ResMut}}, input::{keyboard::KeyCode, Input}, sprite::TextureAtlasSprite};
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand, resource::GlobalEntropy};
use rand_core::RngCore;

use super::loading::{Fonts, GridEditor, Grids, SetGridValue, Strings, SvarogStates, Tilesets};

pub fn randomize_background(mut commands: Commands, mut grids: ResMut<Grids>, mut strings: ResMut<Strings>, mut rng: ResMut<GlobalEntropy<WyRand>>) {
    let mut grid = GridEditor::new(&mut commands, &mut grids, &mut strings);
    let tiles = [ "dirt1", "dirt2", "dirt3", "grass1", "grass2", "grass3", "grass4" ];
    for i in 0..200 {
        for j in 0..200 {
            grid.set("tiles", i, j, tiles[rng.next_u32() as usize % tiles.len()]);
        }
    }
}

pub fn change_random_updates(input: Res<Input<KeyCode>>, mut commands: Commands, mut grids: ResMut<Grids>, mut strings: ResMut<Strings>, mut counter: Local<i32>) {
    let mut grid = GridEditor::new(&mut commands, &mut grids, &mut strings);

    if input.just_pressed(KeyCode::Space) {
        *counter += 1;
        grid.boxed("uiTL", 0, 0, 15, 10, &[ "topleft", "topright", "bottomleft", "bottomright", "top", "bottom", "left", "right", " " ]);
        grid.print("uiTL", 3, 0, &format!(" COUNT: {} ", *counter));
        grid.print("uiTL", 2, 2, "C:>/block/ed");
        grid.set("uiTL", 6, 2, if *counter % 2 == 0 { "block" } else { "" });
    }
}

pub fn grid_update_values(
    mut commands: Commands,
    tilesets: Res<Tilesets>,
    fonts: Res<Fonts>,
    mut grids: ResMut<Grids>,
    mut strings: ResMut<Strings>,
    mut changed_sprite_query: Query<(Entity, &mut TextureAtlasSprite, &SetGridValue)>,
) {
    for (entity, mut sprite, SetGridValue { tileset, value }) in &mut changed_sprite_query {
        let Some(tileset) = tilesets.tilesets.get(strings.out(*tileset).unwrap()) else { println!("NO TILESET {}", tileset); continue; };
        let Some(font) = fonts.fonts.get(&tileset.font) else { println!("NO FONT {}", tileset.font); continue; };
        let Some(glyph) = font.glyphs.get(strings.out(*value).unwrap()) else { println!("NO GLYPH {}", value); continue; };
        let index = (glyph.x - 1) + (glyph.y - 1) * tileset.columns;
        sprite.index = index as usize;
        commands.entity(entity).remove::<SetGridValue>();
    }
}

#[derive(Default)]
pub struct SvarogGridPlugin<S: SvarogStates>(PhantomData<S>);

impl<S: SvarogStates> Plugin for SvarogGridPlugin<S> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EntropyPlugin::<WyRand>::default());
        app.add_systems(Update, 
        (
            randomize_background, 
            change_random_updates
        ).chain().run_if(in_state(S::done_loading_state())));
        app.add_systems(Update, grid_update_values.after(change_random_updates).run_if(in_state(S::done_loading_state())));
    }
}
