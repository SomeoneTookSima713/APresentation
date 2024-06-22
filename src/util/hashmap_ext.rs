#![allow(unused)]

use std::cmp::Eq;
use std::collections::{ HashMap, hash_map::RandomState };
use std::hash::Hash;

pub trait HashMapExt<K: Hash + Eq, V> {
    /// Checks if there is a value stored at the specified key. If not, the
    /// given value is inserted.
    /// 
    /// Returns [`Err(to_insert)`](Err) if there already is a value at the specified key.
    fn insert_if_none(&mut self, key: K, to_insert: V) -> Result<(), V>;

    /// Checks if there is a value stored at the given key. If not, the closure
    /// is executed to evaluate and insert a value at the key.
    /// 
    /// # Returns
    /// Returns a [`bool`] indicating if the closure was executed or not.
    fn create_if_none(&mut self, key: K, creator: impl FnOnce() -> V) -> bool {
        self.try_create_if_none::<()>(key, || Ok((creator)())).unwrap()
    }

    /// Gets the value at the given key. If there is no value at the key, the
    /// closure gets called to evaluate and store one.
    /// 
    /// # Returns
    /// It always returns a reference to a contained value, as there either
    /// already is a value in the map, or one gets created.
    fn get_or_create(&mut self, key: K, creator: impl FnOnce() -> V) -> &V {
        self.try_get_or_create::<()>(key, || Ok((creator)())).unwrap()
    }

    /// Fallible version of [`create_if_none`](HashMapExt::create_if_none).
    fn try_create_if_none<E>(&mut self, key: K, creator: impl FnOnce() -> Result<V, E>) -> Result<bool, E>;

    /// Fallible version of [`get_or_create`](HashMapExt::get_or_create).
    fn try_get_or_create<E>(&mut self, key: K, creator: impl FnOnce() -> Result<V, E>) -> Result<&V, E>;
}

impl<K: Hash + Eq, V> HashMapExt<K, V> for HashMap<K, V, RandomState> {
    fn insert_if_none(&mut self, key: K, to_insert: V) -> Result<(), V> {
        if self.get(&key).is_none() {
            self.insert(key, to_insert);
            Ok(())
        } else {
            Err(to_insert)
        }
    }

    fn try_create_if_none<E>(&mut self, key: K, creator: impl FnOnce() -> Result<V, E>) -> Result<bool, E> {
        if self.get(&key).is_none() {
            self.insert(key, (creator)()?);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn try_get_or_create<E>(&mut self, key: K, creator: impl FnOnce() -> Result<V, E>) -> Result<&V, E> {
        let exists = self.get(&key).is_some();
        let entry = self.entry(key);
        if !exists {
            let val = (creator)()?;
            Ok(&*entry.or_insert_with(|| val))
        } else {
            Ok(&*entry.or_insert_with(|| panic!("Insert closure called on existing entry in HashMap!")))
        }
    }
}