use std::marker::PhantomData;

use bevy::app::App;
use self::{loading::{Fonts, Grids, SvarogLoadingPlugin, SvarogStates, SvarogTextureAtlases, Tilesets}, 
    update::SvarogGridPlugin, windows::SvarogWindowPlugin};

pub mod windows;
pub mod loading;
pub mod tables;
pub mod rex;
pub mod update;

pub struct Svarog<A: SvarogTextureAtlases, S: SvarogStates>(pub(crate) App, PhantomData<(A, S)>);

impl<A: SvarogTextureAtlases, S: SvarogStates> Default for Svarog<A, S> {
    fn default() -> Self {
        Self({
            let mut app = App::default();
            app.add_plugins(SvarogWindowPlugin);
            app.add_plugins(SvarogGridPlugin::<S>::default());
            app
        }, PhantomData)
    }
}

impl<A: SvarogTextureAtlases, S: SvarogStates> Svarog<A, S> {
    pub fn with_loader<F: Fn(&mut Tilesets, &mut Fonts, &mut Grids) + 'static + Sync + Send>(mut self, f: F) -> Self {
        self.0.add_plugins(SvarogLoadingPlugin::<A, S>::default().with_loader(f));
        self
    }

    pub fn as_bevy(self) -> App {
        self.0
    }

    pub fn run(&mut self) {
        self.0.run()
    }
}