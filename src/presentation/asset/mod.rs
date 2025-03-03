use std::any::{ Any, TypeId };

use hashbrown::HashMap;

use toml::Table;

#[derive(Debug, thiserror::Error)]
pub enum AssetLoadError {
    #[error("input/output error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("couldn't parse TOML data: {0}")]
    ParsingError(anyhow::Error),
    #[error("couldn't create asset: {0}")]
    CreationError(anyhow::Error),
}

#[derive(Clone)]
pub struct AssetLoadingParams {
    pub gpu_device: wgpu::Device,
    pub gpu_queue: wgpu::Queue
}

pub trait AssetType {
    /// This method serves as an initialization call for any resources
    /// necessary to load assets, e.g. bind group layouts for images.
    fn global_init(loading_params: AssetLoadingParams);

    fn load_asset(data: Table, loading_params: AssetLoadingParams) -> Result<(String, Self), AssetLoadError>
    where Self: Sized;
}

pub struct RegisteredAssetType {
    pub asset_type_id: TypeId,
    load_asset_fn: Box<dyn Fn(Table, AssetLoadingParams) -> Result<(String, Box<dyn Any>), AssetLoadError>>
}

impl RegisteredAssetType {
    pub(self) fn new<T: AssetType + 'static>() -> Self {
        Self {
            asset_type_id: TypeId::of::<T>(),
            load_asset_fn: Box::new(|t, p| T::load_asset(t, p).map(|(k, v)| (k, Box::new(v) as Box<dyn Any>)))
        }
    }

    pub(self) fn load_asset(&self, data: Table, loading_params: AssetLoadingParams) -> Result<(String, Box<dyn Any>), AssetLoadError> {
        (self.load_asset_fn)(data, loading_params)
    }
}

pub struct AssetManager {
    registered_asset_types: HashMap<TypeId, RegisteredAssetType>,
    asset_type_names: HashMap<String, TypeId>,
    assets: HashMap<TypeId, HashMap<String, Box<dyn Any>>>
}

impl AssetManager {
    pub fn new() -> Self {
        Self { registered_asset_types: HashMap::new(), asset_type_names: HashMap::new(), assets: HashMap::new() }
    }

    pub fn register_asset_type<A: AssetType + 'static>(&mut self, name: String) {
        self.registered_asset_types.insert(TypeId::of::<A>(), RegisteredAssetType::new::<A>());
        self.asset_type_names.insert(name, TypeId::of::<A>());
    }

    pub fn get_asset_type_from_name<Q>(&self, name: &Q) -> Option<TypeId>
    where Q: std::hash::Hash + hashbrown::Equivalent<String> + ?Sized {
        self.asset_type_names.get(name).copied()
    }

    pub fn load_asset<A: AssetType + 'static>(&mut self, data: Table, loading_params: AssetLoadingParams) -> Result<String, AssetLoadError> {
        if let Some(manager) = self.registered_asset_types.get(&TypeId::of::<A>()) {
            let (k, v) = manager.load_asset(data, loading_params)?;
            self.assets.entry(TypeId::of::<A>()).or_insert_with(||HashMap::new()).insert(k.clone(), v);
            Ok(k)
        } else {
            Err(AssetLoadError::CreationError(anyhow::anyhow!("No asset type '{}' found!", std::any::type_name::<A>())))
        }
    }

    pub fn load_asset_dyn(&mut self, type_tid: TypeId, data: Table, loading_params: AssetLoadingParams) -> Result<String, AssetLoadError> {
        if let Some(manager) = self.registered_asset_types.get(&type_tid) {
            let (k, v) = manager.load_asset(data, loading_params)?;
            self.assets.entry(type_tid).or_insert_with(||HashMap::new()).insert(k.clone(), v);
            Ok(k)
        } else {
            Err(AssetLoadError::CreationError(anyhow::anyhow!("No asset type with id {type_tid:?} found!")))
        }
    }

    pub fn get_asset<A: AssetType + 'static, Q>(&self, index: &Q) -> Option<&A>
    where Q: std::hash::Hash + hashbrown::Equivalent<String> + ?Sized
    {
        self.assets.get(&TypeId::of::<A>())
            .and_then(|m| m.get(index))
            .and_then(|b| b.downcast_ref())
    }
}

pub struct AssetCollection<'a, A: AssetType> {
    manager: &'a AssetManager,
    _phantom: std::marker::PhantomData<A>
}

impl<'a, A: AssetType + 'static> AssetCollection<'a, A> {
    pub fn new(manager: &'a AssetManager) -> Self {
        Self { manager, _phantom: Default::default() }
    }

    pub fn get_asset<Q>(&self, index: &Q) -> Option<&A>
    where Q: std::hash::Hash + hashbrown::Equivalent<String> + ?Sized
    {
        self.manager.get_asset::<A, Q>(index)
    }
}