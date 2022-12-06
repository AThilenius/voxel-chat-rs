use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Default)]
pub struct SerdeTestPlugin;

impl Plugin for SerdeTestPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StoredComponent>()
            .add_asset::<CustomAsset>()
            .init_asset_loader::<CustomAssetLoader>()
            .add_startup_system(setup);
    }
}

#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "91a14c6f-3c94-4abc-ab0e-b47ea6525b04"]
pub struct CustomAsset {
    pub value: i32,
}

#[derive(Default)]
pub struct CustomAssetLoader;

impl AssetLoader for CustomAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let custom_asset = ron::de::from_bytes::<CustomAsset>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct StoredComponent {
    num: i32,
    asset_handle: Handle<CustomAsset>,
}

fn setup(asset_server: Res<AssetServer>) {
    let asset: Handle<CustomAsset> = asset_server.load("asset.custom");
    println!("{:#?}", asset);
}
