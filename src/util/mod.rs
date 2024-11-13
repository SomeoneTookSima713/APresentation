pub mod consts;
pub mod hashable_value;
pub mod fallible_app_handler;
pub mod hashmap_ext;
pub mod lua_helper;
pub mod debug_state;
pub mod math;
pub mod macros;

pub use hashable_value::*;

pub const fn extended_slice<const A: usize, const B: usize, T: Copy>(base: &'static [T; A], new: [T; B]) -> [T; A + B]
where [T; A + B]: Sized {
    use std::mem::MaybeUninit;

    unsafe {
        let mut arr: [T; A + B] = MaybeUninit::uninit().assume_init();

        std::ptr::copy(base.as_ptr(), arr.as_mut_ptr(), A);
        std::ptr::copy(new.as_ptr(), arr.as_mut_ptr().add(A), B);

        arr
    }
}

pub struct SendSync<T>(pub T);
unsafe impl<T> Send for SendSync<T> {}
unsafe impl<T> Sync for SendSync<T> {}
impl<T> std::ops::Deref for SendSync<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for SendSync<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct DynSlice<'a, T: ?Sized>(&'a dynstack::DynStack<T>);

impl<'a, T: ?Sized> std::ops::Deref for DynSlice<'a, T> {
    type Target = dynstack::DynStack<T>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: ?Sized> DynSlice<'a, T> {
    pub fn new(stack: &'a dynstack::DynStack<T>) -> Self {
        Self(stack)
    }
}

impl<'a, 'lua, T: lua_helper::IntoLuaObjectSafe<'lua> + ?Sized> lua_helper::IntoLuaObjectSafe<'lua> for DynSlice<'a, T> {
    fn into_lua(&self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::MultiValue<'lua>> {
        self.iter().map(|v| v.into_lua(lua)).try_reduce(|acc, e| {
            let mut acc = acc?;
            let mut e = e?;
            while let Some(v) = e.pop_front() {
                acc.push_front(v);
            }
            Ok(Ok(acc))
        }).map(|v|v.unwrap_or(Ok(mlua::MultiValue::new()))).and_then(|v|v)
    }
}

impl<'a, 'lua, T: lua_helper::IntoLuaObjectSafe<'lua> + ?Sized> DynSlice<'a, T> {
    pub fn to_variadic(&'a self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Variadic<mlua::Value<'lua>>> {
        self.iter().map(|v| v.into_lua(lua)).try_reduce(|acc, e| {
            let mut acc = acc?;
            let mut e = e?;
            while let Some(v) = e.pop_front() {
                acc.push_front(v);
            }
            Ok(Ok(acc))
        }).map(|v|v.unwrap_or(Ok(mlua::MultiValue::new()))).and_then(|v|v).map(|m| {
            let mut variadic = mlua::Variadic::new();
            for val in m.iter() {
                variadic.push(val.clone());
            }
            variadic
        })
    }
}

pub struct OwnedParsedFace {
    face: std::mem::ManuallyDrop<ttf_parser::Face<'static>>,
    orig_len: usize,
    orig_cap: usize,
    orig_data_ref: &'static [u8],
    face_index: u32
}

impl OwnedParsedFace {
    pub fn parse(data: Vec<u8>, index: u32) -> Result<Self, ttf_parser::FaceParsingError> {
        let orig_len = data.len();
        let orig_cap = data.capacity();
        let leaked_data = &*data.leak();

        let face = std::mem::ManuallyDrop::new(ttf_parser::Face::parse(leaked_data, index)?);

        Ok(Self { face, orig_len, orig_cap, orig_data_ref: leaked_data, face_index: index })
    }
}

impl std::ops::Deref for OwnedParsedFace {
    type Target = ttf_parser::Face<'static>;

    fn deref(&self) -> &Self::Target {
        &self.face
    }
}

impl std::ops::DerefMut for OwnedParsedFace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.face
    }
}

impl Drop for OwnedParsedFace {
    fn drop(&mut self) {
        unsafe { std::mem::ManuallyDrop::drop(&mut self.face) };
        drop(unsafe { Vec::from_raw_parts(self.orig_data_ref.as_ptr() as *mut u8, self.orig_len, self.orig_cap) });
    }
}

pub type ArcVec<T> = Vec<std::sync::Arc<T>>;
pub type ArcHashmap<K, V> = hashbrown::HashMap<K, std::sync::Arc<V>>;

pub fn get_font_source_from_query(db: &fontdb::Database, query: &fontdb::Query) -> Option<(Vec<u8>, u32)> {
    use fontdb::Source;
    
    Some(match db.face_source(db.query(query)?)? {
        (Source::Binary(d), index) => (d.as_ref().as_ref().to_vec(), index),
        (Source::File(p), index) => (std::fs::read(p).ok()?, index),
        // This uses a memory-mapped file, which is inherently unsafe. Since we
        // don't want to accidentally use this data for long periods of time,
        // it gets copied here.
        (Source::SharedFile(_, d), index) => (d.as_ref().as_ref().iter().copied().collect::<Vec<u8>>(), index)
    })
}