//! Elements are the objects that get rendered on screen.

use std::any::TypeId;

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
pub trait ElemObjS: downcast_rs::Downcast {}
downcast_rs::impl_downcast!(ElemObjS);
impl std::fmt::Debug for dyn ElemObjS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dyn ElemObjS")
    }
}

impl<T: Element + 'static> ElemObjS for T {}

pub trait ElementRenderer {
    type Element: Element<Renderer = Self> + 'static;

    fn init(device: wgpu::Device, queue: wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> Self
    where Self: Sized;

    fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {}

    fn render(
        &mut self,
        element: &Self::Element,
        eval_engine: &rhai::Engine,
        eval_scope: rhai::Scope<'static>,
        render_pass: &mut wgpu::RenderPass
    );
}

/// *For internal use only!*
/// 
/// An dyn-safe variant of the [`ElementRenderer`] trait that is automatically
/// implemented by any type implementing `ElementRenderer`.
pub trait ElemRendObjS: downcast_rs::Downcast {
    fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {}

    fn render_dyn(
        &mut self,
        element: &dyn ElemObjS,
        eval_engine: &rhai::Engine,
        eval_scope: rhai::Scope<'static>,
        render_pass: &mut wgpu::RenderPass
    );
}
downcast_rs::impl_downcast!(ElemRendObjS);

impl<T: ElementRenderer + 'static> ElemRendObjS for T {
    fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {
        <T as ElementRenderer>::reconfigure(self, surface_config);
    }

    fn render_dyn(
        &mut self,
        element: &dyn ElemObjS,
        eval_engine: &rhai::Engine,
        eval_scope: rhai::Scope<'static>,
        render_pass: &mut wgpu::RenderPass
    ) {
        if let Some(elem) = element.downcast_ref() {
            self.render(elem, eval_engine, eval_scope, render_pass);
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
    constructor_fn: Box<dyn for<'a> Fn(ParsedStructure, &'a rhai::Engine) -> anyhow::Result<Box<dyn ElemObjS>>>
}

/// Basically a custom v-table for the [`ElementRenderer`] struct because Rust doesn't
/// support dynamic dispatch for methods that don't have a receiver like
/// `&self`.
impl RegisteredElement {
    pub fn new<T: Element + 'static>() -> Self {
        Self { renderer_type_id: TypeId::of::<T::Renderer>(), constructor_fn: Box::new(|s, e| T::from_structure(s, e).map(|s| Box::new(s) as Box<dyn ElemObjS>)) }
    }

    pub fn construct(&self, parsed_structure: ParsedStructure, engine: &rhai::Engine) -> anyhow::Result<Box<dyn ElemObjS>> {
        (self.constructor_fn)(parsed_structure, engine)
    }
}

pub struct RegisteredElementRenderer {
    element_type_id: TypeId,
    init_fn: Box<dyn for<'a> Fn(wgpu::Device, wgpu::Queue, &'a wgpu::SurfaceConfiguration) -> Box<dyn ElemRendObjS>>,
    reconfigure_fn: Box<dyn for<'a, 'b> Fn(&'a mut dyn ElemRendObjS, &'b wgpu::SurfaceConfiguration)>,
    render_fn: Box<dyn for<'a, 'b> Fn(
        &'a mut dyn ElemRendObjS,
        &'b dyn ElemObjS,
        &rhai::Engine,
        rhai::Scope<'static>,
        &'b mut wgpu::RenderPass
    )>
}

impl RegisteredElementRenderer {
    pub fn new<T: ElementRenderer + 'static>() -> Self {
        Self {
            element_type_id: TypeId::of::<T::Element>(),
            init_fn: Box::new(|d, q, c| Box::new(T::init(d, q, c))),
            reconfigure_fn: Box::new(|s, c| match s.downcast_mut() {
                Some(s) => <T as ElemRendObjS>::reconfigure(s, c),
                None => panic!("Downcast in <RegisteredEleentRenderer>.reconfigure_fn failed!")
            }),
            render_fn: Box::new(|s, el, e, sc, r| match s.downcast_mut() {
                Some(s) => <T as ElemRendObjS>::render_dyn(s, el, e, sc, r),
                None => panic!("Downcast in <RegisteredEleentRenderer>.render_fn failed!")
            })
        }
    }
}

pub struct RegisteredElements {
    pub element_types: hashbrown::HashMap<String, RegisteredElement>,
    pub element_renderer: hashbrown::HashMap<TypeId, RegisteredElementRenderer>
}

impl RegisteredElements {
    pub fn new() -> Self {
        Self { element_types: hashbrown::HashMap::new(), element_renderer: hashbrown::HashMap::new() }
    }

    pub fn register_element<T: Element + 'static>(&mut self, ident: String) {
        self.element_types.insert(ident, RegisteredElement::new::<T>());
    }

    pub fn register_element_renderer<T: ElementRenderer + 'static>(&mut self) {
        self.element_renderer.insert(TypeId::of::<T>(), RegisteredElementRenderer::new::<T>());
    }
}