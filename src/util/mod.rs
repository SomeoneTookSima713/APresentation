pub mod consts;
pub mod hashable_value;
pub mod atomic_vec;
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

pub trait FontDBSourceExt {
    fn with_data<P, T>(&self, p: P) -> Option<T>
    where
        P: FnOnce(&[u8]) -> T;
    
    fn get_data(&self) -> Option<Vec<u8>> {
        self.with_data(|d| Vec::from(d))
    }
}

impl FontDBSourceExt for fontdb::Source {
    /// Copied from the internally used function which for some reason isn't public.
    /// 
    /// (Like seriously, the [`Database::with_face_data()`](fontdb::Database::with_face_data()) function, which uses this exact function, actually clones the data beforehand, making this whole closure thing completely useless as you could also just return the cloned data directly)
    fn with_data<P, T>(&self, p: P) -> Option<T>
    where
        P: FnOnce(&[u8]) -> T,
        {
        use fontdb::Source;
        match &self {
            Source::File(ref path) => {
                let file = std::fs::File::open(path).ok()?;
                let data = unsafe { &memmap2::MmapOptions::new().map(&file).ok()? };

                Some(p(data))
            }
            Source::Binary(ref data) => Some(p(data.as_ref().as_ref())),
            Source::SharedFile(_, ref data) => Some(p(data.as_ref().as_ref())),
        }
    }
}

pub struct OwnedParsedFace {
    face: ttf_parser::Face<'static>,
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

        let face = ttf_parser::Face::parse(leaked_data, index)?;

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
        drop(unsafe { Vec::from_raw_parts(self.orig_data_ref.as_ptr() as *mut u8, self.orig_len, self.orig_cap) });
    }
}