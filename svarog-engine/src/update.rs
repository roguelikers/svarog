use std::marker::PhantomData;

use bevy::{app::{Plugin, PostUpdate}, ecs::{entity::Entity, schedule::{common_conditions::in_state, IntoSystemConfigs}, 
    system::{Commands, Query, Res, ResMut}}, render::view::Visibility, sprite::TextureAtlasSprite};
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand};

use crate::loading::strings;

use super::loading::{Fonts, SetGridValue, Strings, SvarogStates, Tilesets};

pub fn grid_update_values(
    mut commands: Commands,
    tilesets: Res<Tilesets>,
    fonts: Res<Fonts>,
    mut changed_sprite_query: Query<(Entity, &mut TextureAtlasSprite, &mut Visibility, &SetGridValue)>,
) {
    for (entity, mut sprite, mut visibility, SetGridValue { tileset, value }) in &mut changed_sprite_query {
        *visibility = if *value != 0 { Visibility::Visible } else { Visibility::Hidden };
        
        if *value != 0 {
            let mut strings = strings().lock().unwrap();
            let Some(tileset) = tilesets.tilesets.get(strings.out(*tileset).unwrap()) else { println!("NO TILESET {}", tileset); continue; };
            let Some(font) = fonts.fonts.get(&tileset.font) else { println!("NO FONT {}", tileset.font); continue; };
            let Some(glyph) = font.glyphs.get(strings.out(*value).unwrap()) else { println!("NO GLYPH {}", value); continue; };
            let index = (glyph.x - 1) + (glyph.y - 1) * tileset.columns;
            sprite.index = index as usize;
        }
        commands.entity(entity).remove::<SetGridValue>();
    }
}

#[derive(Default)]
pub struct SvarogGridPlugin<S: SvarogStates>(PhantomData<S>);

impl<S: SvarogStates> Plugin for SvarogGridPlugin<S> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostUpdate, grid_update_values.run_if(in_state(S::done_loading_state())));
    }
}
