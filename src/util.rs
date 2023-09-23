#![allow(dead_code)]

use std::ops::{ Deref, DerefMut };

/// Like the [`Option`]-Enum, but it automatically dereferences and panics if it is [`None`]
pub enum PanickingOption<T> {
    Some(T),
    None
}
impl<T> Deref for PanickingOption<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Some(ref v) => v,
            Self::None => panic!("Tried dereferencing a None-Value!")
        }
    }
}
impl<T> DerefMut for PanickingOption<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Some(ref mut v) => v,
            Self::None  => panic!("Tried dereferencing a None-Value!")
        }
    }
}
impl<T> From<T> for PanickingOption<T> {
    fn from(value: T) -> Self {
        PanickingOption::Some(value)
    }
}

/// Like the [`Option`]-Enum, but it implements [`From`] for the wrapped type and a helper function for getting the contained value or a default one, depending if it's `Some` or `None`
pub enum DefaultingOption<T> {
    Some(T),
    None
}
impl<T> From<T> for DefaultingOption<T> {
    fn from(value: T) -> Self {
        DefaultingOption::Some(value)
    }
}
impl<T> From<Option<T>> for DefaultingOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => DefaultingOption::Some(v),
            None => DefaultingOption::None
        }
    }
}
impl<T> Into<Option<T>> for DefaultingOption<T> {
    fn into(self) -> Option<T> {
        match self {
            Self::Some(v) => Some(v),
            Self::None => None
        }
    }
}
impl<T> DefaultingOption<T> {
    pub fn get<'a>(&'a self, default: &'a T) -> &'a T {
        match self {
            Self::Some(ref v) => v,
            Self::None => default
        }
    }

    pub fn get_mut<'a>(&'a mut self, default: &'a mut T) -> &'a mut T {
        match self {
            Self::Some(ref mut v) => v,
            Self::None => default
        }
    }

    pub fn consume(self, default: T) -> T {
        match self {
            Self::Some(v) => v,
            Self::None => default
        }
    }
}

/// Wrapper that is always Send or Sync, independant of the contents.
pub struct AssumeThreadSafe<T>(pub T);

unsafe impl<T> Send for AssumeThreadSafe<T> {}
unsafe impl<T> Sync for AssumeThreadSafe<T> {}

impl<T> Deref for AssumeThreadSafe<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for AssumeThreadSafe<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}