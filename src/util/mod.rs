pub mod consts;
pub mod hashable_value;
pub mod atomic_vec;
pub mod fallible_app_handler;
pub mod hashmap_ext;
pub mod lua_helper;
pub mod debug_state;
pub mod math;

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