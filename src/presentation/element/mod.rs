//! Elements are the objects that get rendered on screen.

use std::any::TypeId;

use crate::presentation::asset::AssetManager;
use crate::presentation::parser::data::ParsedStructure;

pub mod property;

pub use crate::elements::*;

/// The main trait of this module.
/// 
/// An [`Element`] describes an object that gets rendered onto the screen.
pub trait Element: property::base::BasePropertiesProvider {
    type Renderer: ElementRenderer;

    fn from_structure(structure: ParsedStructure, engine: &rhai::Engine) -> anyhow::Result<Self>
    where Self: Sized;
}

/// *For internal use only!*
/// 
/// An dyn-safe marker for the [`Element`] trait that is automatically
/// implemented by any type implementing `Element`.
pub trait ElemObjS: downcast_rs::Downcast {
    fn underlying_type_id(&self) -> TypeId;
}
downcast_rs::impl_downcast!(ElemObjS);
impl std::fmt::Debug for dyn ElemObjS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dyn ElemObjS({:?})", self.underlying_type_id())
    }
}

impl<T: Element + 'static> ElemObjS for T {
    fn underlying_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

pub trait ElementRenderer {
    type Element: Element<Renderer = Self> + 'static;

    fn init(device: wgpu::Device, queue: wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> Self
    where Self: Sized;

    fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {}

    fn submit_to_render(
        &mut self,
        element: &Self::Element,
        eval_engine: &rhai::Engine,
        eval_scope: rhai::Scope<'static>,
        asset_manager: &AssetManager,
        render_pass: &mut wgpu::RenderPass
    ) -> anyhow::Result<()>;

    fn finish_render(&mut self, render_pass: &mut wgpu::RenderPass);
}

/// *For internal use only!*
/// 
/// An dyn-safe extension of the [`ElementRenderer`] trait that is
/// automatically implemented by any type implementing `ElementRenderer`.
pub trait ElemRendObjS: downcast_rs::Downcast {
    fn submit_to_render_dyn(
        &mut self,
        element: &dyn ElemObjS,
        eval_engine: &rhai::Engine,
        eval_scope: rhai::Scope<'static>,
        asset_manager: &AssetManager,
        render_pass: &mut wgpu::RenderPass
    ) -> anyhow::Result<()>;
}
downcast_rs::impl_downcast!(ElemRendObjS);

impl std::fmt::Debug for dyn ElemRendObjS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ElementRenderer")
    }
}

impl<T: ElementRenderer + 'static> ElemRendObjS for T {
    fn submit_to_render_dyn(
        &mut self,
        element: &dyn ElemObjS,
        eval_engine: &rhai::Engine,
        eval_scope: rhai::Scope<'static>,
        asset_manager: &AssetManager,
        render_pass: &mut wgpu::RenderPass
    ) -> anyhow::Result<()> {
        if let Some(elem) = element.downcast_ref() {
            self.submit_to_render(elem, eval_engine, eval_scope, asset_manager, render_pass)
        } else {
            panic!("Downcast in presentation::element::ElemRendObjS::render_dyn() failed!")
        }
    }
}

/// Basically a custom v-table for the [`Element`] struct because Rust doesn't
/// support dynamic dispatch for methods that don't have a receiver like
/// `&self`.
pub struct RegisteredElement {
    pub renderer_type_id: TypeId,
    pub own_type_id: TypeId,
    constructor_fn: Box<dyn for<'a> Fn(ParsedStructure, &'a rhai::Engine) -> anyhow::Result<Box<dyn ElemObjS>>>
}

/// Basically a custom v-table for the [`ElementRenderer`] struct because Rust doesn't
/// support dynamic dispatch for methods that don't have a receiver like
/// `&self`.
impl RegisteredElement {
    pub fn new<T: Element + 'static>() -> Self {
        Self {
            renderer_type_id: TypeId::of::<T::Renderer>(),
            own_type_id: TypeId::of::<T>(),
            constructor_fn: Box::new(|s, e| T::from_structure(s, e).map(|s| Box::new(s) as Box<dyn ElemObjS>))
        }
    }

    pub fn construct(&self, parsed_structure: ParsedStructure, engine: &rhai::Engine) -> anyhow::Result<Box<dyn ElemObjS>> {
        (self.constructor_fn)(parsed_structure, engine)
    }
}

pub struct RegisteredElementRenderer {
    element_type_id: TypeId,
    init_fn: Box<dyn for<'a> Fn(wgpu::Device, wgpu::Queue, &'a wgpu::SurfaceConfiguration) -> Box<dyn ElemRendObjS>>,
    reconfigure_fn: Box<dyn for<'a> Fn(&'a mut dyn ElemRendObjS, &'a wgpu::SurfaceConfiguration)>,
    submit_to_render_fn: Box<dyn for<'a> Fn(
        &'a mut dyn ElemRendObjS,
        &'a dyn ElemObjS,
        &'a rhai::Engine,
        rhai::Scope<'static>,
        &'a AssetManager,
        &'a mut wgpu::RenderPass
    ) -> anyhow::Result<()>>,
    finish_render_fn: Box<dyn for<'a> Fn(&'a mut dyn ElemRendObjS, &'a mut wgpu::RenderPass)>
}

impl RegisteredElementRenderer {
    pub fn new<T: ElementRenderer + 'static>() -> Self {
        Self {
            element_type_id: TypeId::of::<T::Element>(),
            init_fn: Box::new(|d, q, c| Box::new(T::init(d, q, c))),
            reconfigure_fn: Box::new(|s, c| match s.downcast_mut() {
                Some(s) => T::reconfigure(s, c),
                None => panic!("Downcast in <RegisteredEleentRenderer>.reconfigure_fn failed!")
            }),
            submit_to_render_fn: Box::new(|s, el, e, sc, a, r| match s.downcast_mut() {
                Some(s) => <T as ElemRendObjS>::submit_to_render_dyn(s, el, e, sc, a, r),
                None => panic!("Downcast in <RegisteredEleentRenderer>.submit_to_render_fn failed!")
            }),
            finish_render_fn: Box::new(|s, r| match s.downcast_mut() {
                Some(s) => T::finish_render(s, r),
                None => panic!("Downcast in <RegisteredEleentRenderer>.finish_render_fn failed!")
            }),
        }
    }

    pub fn init(&self, device: wgpu::Device, queue: wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> Box<dyn ElemRendObjS> {
        (self.init_fn)(device, queue, surface_config)
    }

    pub fn reconfigure(&self, elem_renderer: &mut dyn ElemRendObjS, surface_config: &wgpu::SurfaceConfiguration) {
        (self.reconfigure_fn)(elem_renderer, surface_config)
    }

    pub fn submit_to_render(
        &self,
        elem_renderer: &mut dyn ElemRendObjS,
        elem: &dyn ElemObjS,
        engine: &rhai::Engine,
        scope: rhai::Scope<'static>,
        asset_manager: &AssetManager,
        render_pass: &mut wgpu::RenderPass
    ) -> anyhow::Result<()> {
        (self.submit_to_render_fn)(elem_renderer, elem, engine, scope, asset_manager, render_pass)
    }

    pub fn finish_render(&self, elem_renderer: &mut dyn ElemRendObjS, render_pass: &mut wgpu::RenderPass) {
        (self.finish_render_fn)(elem_renderer, render_pass)
    }
}

pub struct RegisteredElements {
    pub element_types: hashbrown::HashMap<String, RegisteredElement>,
    pub element_renderer: hashbrown::HashMap<TypeId, RegisteredElementRenderer>
}

impl std::fmt::Debug for RegisteredElements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RegisteredElements[{}]", self.element_types.iter().map(|(k, v)| format!("{}: {:?}", k, v.own_type_id)).reduce(|mut a, e| {a.push_str(&e); a}).unwrap_or(String::new()))
    }
}

impl RegisteredElements {
    pub fn new() -> Self {
        Self { element_types: hashbrown::HashMap::new(), element_renderer: hashbrown::HashMap::new() }
    }

    pub fn register_element<T: Element + 'static>(&mut self, ident: String) {
        self.element_types.insert(ident, RegisteredElement::new::<T>());
    }

    pub fn register_element_renderer<T: ElementRenderer + 'static>(&mut self) {
        self.element_renderer.insert(TypeId::of::<T::Element>(), RegisteredElementRenderer::new::<T>());
    }
}