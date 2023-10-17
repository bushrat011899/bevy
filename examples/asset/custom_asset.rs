//! Implements loader for a custom asset type.

use bevy::utils::thiserror;
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use futures_lite::AsyncReadExt;
use serde::Deserialize;
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct CustomAsset {
    pub value: i32,
}

#[derive(Default)]
pub struct CustomAssetLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CustomAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for CustomAssetLoader {
    type Asset = CustomAsset;
    type Settings = ();
    type Error = CustomAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let custom_asset = ron::de::from_bytes::<CustomAsset>(&bytes)?;
            Ok(custom_asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct Blob {
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct BlobAssetLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BlobAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for BlobAssetLoader {
    type Asset = Blob;
    type Settings = ();
    type Error = BlobAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            info!("Loading Blob...");
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            Ok(Blob { bytes })
        })
    }

    fn extensions(&self) -> &[&str] {
        // This loader won't be used for any files by default
        &[]
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<State>()
        .init_asset::<CustomAsset>()
        .init_asset::<Blob>()
        .init_asset_loader::<CustomAssetLoader>()
        .init_asset_loader::<BlobAssetLoader>()
        .add_systems(Startup, setup)
        .add_systems(Update, print_on_load)
        .run();
}

#[derive(Resource, Default)]
struct State {
    handle: Handle<CustomAsset>,
    other_handle: Handle<CustomAsset>,
    blob: Handle<Blob>,
    printed: bool,
}

fn setup(mut state: ResMut<State>, asset_server: Res<AssetServer>) {
    // Recommended way to load an asset
    state.handle = asset_server.load("data/asset.custom");

    // File extensions are optional, but are recommended
    state.other_handle = asset_server.load("data/asset_no_extension");

    // Will use BlobAssetLoader instead of CustomAssetLoader thanks to type inference
    state.blob = asset_server.load("data/asset.custom");
}

fn print_on_load(
    mut state: ResMut<State>,
    custom_assets: Res<Assets<CustomAsset>>,
    blob_assets: Res<Assets<Blob>>,
) {
    let custom_asset = custom_assets.get(&state.handle);
    let other_custom_asset = custom_assets.get(&state.other_handle);
    let blob = blob_assets.get(&state.blob);

    // Can't print results if the assets aren't ready
    if state.printed {
        return;
    }

    if custom_asset.is_none() {
        info!("Custom Asset Not Ready");
        return;
    }

    if other_custom_asset.is_none() {
        info!("Other Custom Asset Not Ready");
        return;
    }

    if blob.is_none() {
        info!("Blob Not Ready");
        return;
    }

    info!("Custom asset loaded: {:?}", custom_asset.unwrap());
    info!("Custom asset loaded: {:?}", other_custom_asset.unwrap());
    info!("Blob Size: {:?} Bytes", blob.unwrap().bytes.len());

    // Once printed, we won't print again
    state.printed = true;
}
