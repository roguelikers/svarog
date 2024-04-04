use std::marker::PhantomData;

use bevy::app::App;

use self::{loading::{Fonts, Grids, SvarogLoadingPlugin, SvarogStates, SvarogTextureAssets, Tilesets}, windows::SvarogWindowPlugin};

pub mod windows;
pub mod loading;
pub mod tables;
pub mod rex;

pub struct Svarog<A: SvarogTextureAssets, S: SvarogStates>(pub(crate) App, PhantomData<(A, S)>);

impl<A: SvarogTextureAssets, S: SvarogStates> Default for Svarog<A, S> {
    fn default() -> Self {
        Self({
            let mut app = App::default();
            app.add_plugins(SvarogWindowPlugin);
            app
        }, PhantomData)
    }
}

impl<A: SvarogTextureAssets, S: SvarogStates> Svarog<A, S> {
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