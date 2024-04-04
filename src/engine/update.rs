use std::marker::PhantomData;

use bevy::{app::{Plugin, Update}, ecs::{entity::Entity, schedule::{common_conditions::in_state, IntoSystemConfigs}, system::{Commands, Local, Query, Res}}, input::{keyboard::KeyCode, Input}, sprite::TextureAtlasSprite};

use super::loading::{Fonts, GridEditor, Grids, SetGridValue, SvarogStates, Tilesets};

pub fn change_random_updates(input: Res<Input<KeyCode>>, mut commands: Commands, grids: Res<Grids>, mut counter: Local<i32>) {
    let mut grid = GridEditor::new(&mut commands, &grids);

    if input.just_pressed(KeyCode::Space) {
        *counter += 1;
        grid.rect("uiTL", 0, 0, 15, 10, " ");
        grid.print("uiTL", 0, 0, &format!("COUNT: {}", *counter));
        grid.print("uiTL", 0, 1, if *counter % 2 == 0 { "¶" } else { " " });
    }
}

pub fn grid_update_values(
    mut commands: Commands,
    tilesets: Res<Tilesets>,
    fonts: Res<Fonts>,
    mut changed_sprite_query: Query<(Entity, &mut TextureAtlasSprite, &SetGridValue)>,
) {
    for (entity, mut sprite, SetGridValue { tileset, value }) in &mut changed_sprite_query {
        let Some(tileset) = tilesets.tilesets.get(tileset) else { println!("NO TILESET {}", tileset); continue; };
        let Some(font) = fonts.fonts.get(&tileset.font) else { println!("NO FONT {} {:?}", tileset.font, tilesets.tilesets); continue; };
        let Some(glyph) = font.glyphs.get(value) else { println!("NO GLYPH {} {:?}", value, font.glyphs); continue; };
        let index = (glyph.x - 1) + (glyph.y - 1) * tileset.columns;
        sprite.index = index as usize;
        commands.entity(entity).remove::<SetGridValue>();
    }
}

#[derive(Default)]
pub struct SvarogGridPlugin<S: SvarogStates>(PhantomData<S>);

impl<S: SvarogStates> Plugin for SvarogGridPlugin<S> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, change_random_updates.run_if(in_state(S::done_loading_state())));
        app.add_systems(Update, grid_update_values.after(change_random_updates).run_if(in_state(S::done_loading_state())));
    }
}
