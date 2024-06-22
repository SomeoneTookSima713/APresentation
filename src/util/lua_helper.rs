pub trait IntoLuaObjectSafe<'lua> {
    fn into_lua(&self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::MultiValue<'lua>>;
}

impl<'lua, T: mlua::IntoLuaMulti<'lua> + Clone> IntoLuaObjectSafe<'lua> for T {
    fn into_lua(&self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::MultiValue<'lua>> {
        self.clone().into_lua_multi(lua)
    }
}