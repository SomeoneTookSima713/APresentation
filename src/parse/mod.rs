use std::borrow::Borrow;
use std::cell::{ RefCell, Ref, RefMut };
use std::collections::HashMap;
use std::rc::Rc;

use crate::presentation::property::{ Property, PropertyValue };

#[derive(Clone)]
pub struct ParseableRenderable<'lua> {
    properties: Rc<RefCell<HashMap<String, Property<'lua>>>>
}

impl<'lua> ParseableRenderable<'lua> {
    pub fn new(properties: Rc<RefCell<HashMap<String, Property<'lua>>>>) -> Self {
        Self { properties }
    }

    pub fn get<'a, S: std::hash::Hash + std::cmp::Eq>(&'a self, index: &S) -> Option<Ref<'a, Property<'lua>>>
    where String: Borrow<S>, S: ?Sized {
        (*self.properties).try_borrow().ok().and_then(|borrow| Ref::filter_map(borrow, |b| b.get(index)).ok())
    }

    pub fn get_mut<'a, S: std::hash::Hash + std::cmp::Eq>(&'a mut self, index: &S) -> Option<RefMut<'a, Property<'lua>>>
    where String: Borrow<S>, S: ?Sized {
        (*self.properties).try_borrow_mut().ok().and_then(|borrow| RefMut::filter_map(borrow, |b| b.get_mut(index)).ok())
    }

    pub fn extend<'a>(&'a mut self, by: &'a [(String, Property<'lua>)]) -> Result<(), ()> {
        match (*self.properties).try_borrow_mut().ok() {
            Some(mut borrow) => borrow.extend(by.iter().cloned()),
            None => return Err(())
        }
        Ok(())
    }

    pub fn into_property_value(self) -> PropertyValue<'lua> {
        PropertyValue::Dict(self.properties)
    }
}