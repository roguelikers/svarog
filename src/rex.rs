use bevy::asset::{AssetLoader, AsyncReadExt};
use rexpaint::XpFile;


#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct RexpaintDocument(pub XpFile);

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum RexpaintLoaderError {

}

#[derive(Default)]
pub struct RexpaintDocumentLoader;

impl AssetLoader for RexpaintDocumentLoader {
    type Asset = RexpaintDocument;
    type Settings = ();
    type Error = RexpaintLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let rexpaint_doc = XpFile::read(&mut bytes)?;
        Ok(RexpaintDocument(rexpaint_doc))
    }

    fn extensions(&self) -> &[&str] {
        &["xp"]
    }
}