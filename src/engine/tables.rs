use bevy::app::{App, Plugin};
use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetApp, AssetLoader, AsyncReadExt, BoxedFuture, Handle, LoadContext};
use bevy::prelude::TypePath;
use bevy_common_assets::csv::CsvLoaderError;
use std::marker::PhantomData;

/// Plugin to load your asset type `A` from csv files.
pub struct TableAssetPlugin<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

impl<A> Plugin for TableAssetPlugin<A>
where
    for<'de> A: serde::Deserialize<'de> + Asset,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<A>()
            .init_asset::<LoadedCsv<A>>()
            .register_asset_loader(TableAssetLoader::<A> {
                extensions: self.extensions.clone(),
                _marker: PhantomData,
            });
    }
}

impl<A> TableAssetPlugin<A>
where
    for<'de> A: serde::Deserialize<'de> + Asset,
{
    /// Create a new plugin that will load assets from files with the given extensions.
    pub fn new(extensions: &[&'static str]) -> Self {
        Self {
            extensions: extensions.to_owned(),
            _marker: PhantomData,
        }
    }
}

struct TableAssetLoader<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

/// Asset representing a loaded CSV file with rows deserialized to Assets of type `A`
#[derive(TypePath, Asset)]
pub struct LoadedCsv<A>
where
    for<'de> A: serde::Deserialize<'de> + Asset,
{
    /// Handles to the Assets the were loaded from the rows of this CSV file
    pub rows: Vec<Handle<A>>,
}

impl<A> AssetLoader for TableAssetLoader<A>
where
    for<'de> A: serde::Deserialize<'de> + Asset,
{
    type Asset = LoadedCsv<A>;
    type Settings = ();
    type Error = CsvLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(b'|')
                .comment(Some(b'#'))
                .from_reader(bytes.as_slice());
            let mut handles = vec![];
            for (index, result) in reader.deserialize().enumerate() {
                let asset: A = result?;
                handles
                    .push(load_context.add_loaded_labeled_asset(index.to_string(), asset.into()));
            }
            Ok(LoadedCsv { rows: handles })
        })
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}
