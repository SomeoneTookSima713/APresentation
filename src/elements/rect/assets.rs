use crate::presentation::asset;

use asset::{ AssetType, AssetLoadingParams, AssetLoadError };

pub struct Image {

}

impl AssetType for Image {
    fn load_asset(data: toml::Table, loading_params: AssetLoadingParams) -> Result<(String, Self), AssetLoadError>
    where Self: Sized {
        // TODO
        Ok(("test".to_string(), Image {  }))
    }
}