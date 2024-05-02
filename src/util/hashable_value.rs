use std::hash::{ Hash, Hasher };

pub struct HashableLuaValue<'lua>(mlua::Value<'lua>);

impl<'lua> std::fmt::Display for HashableLuaValue<'lua> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use mlua::Value;
        match &self.0 {
            Value::Boolean(b) => write!(f, "bool:{b}"),
            Value::Error(e) => write!(f, "err:{e}"),
            Value::Function(func) => write!(f, "function:{}",func.to_pointer() as usize),
            Value::Integer(i) => write!(f, "int:{i}"),
            Value::LightUserData(ptr) => write!(f, "userdata_ref:{}",ptr.0 as usize),
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "float:{n}"),
            Value::String(s) => write!(f, "{}",s.to_string_lossy()),
            Value::Table(t) => write!(f, "table:{}",t.to_pointer() as usize),
            Value::Thread(t) => write!(f, "thread:{}",t.to_pointer() as usize),
            Value::UserData(u) => write!(f, "userdata:{}",u.to_pointer() as usize),
        }
    }
}

impl<'lua> From<mlua::Value<'lua>> for HashableLuaValue<'lua> {
    fn from(value: mlua::Value<'lua>) -> Self {
        Self(value)
    }
}

impl<'lua> std::cmp::PartialEq for HashableLuaValue<'lua> {
    fn eq(&self, other: &Self) -> bool {
        use mlua::Value;
        
        match (&self.0, &other.0) {
            (Value::Boolean(bs), Value::Boolean(bo)) => bs == bo,
            (Value::Function(fs), Value::Function(fo)) => (fs.to_pointer() as usize) == (fo.to_pointer() as usize),
            (Value::Integer(is), Value::Integer(io)) => *is == *io,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(fs), Value::Number(fo)) => *fs == *fo,
            (Value::String(ss), Value::String(so)) => ss.to_string_lossy().eq(so.to_string_lossy().as_ref()),
            (Value::Table(ts), Value::Table(to)) => (ts.to_pointer() as usize) == (to.to_pointer() as usize),
            _ => false
        }
    }
}
impl<'lua> std::cmp::Eq for HashableLuaValue<'lua> {}

impl<'lua> Hash for HashableLuaValue<'lua> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use mlua::Value;
        
        match &self.0 {
            Value::Boolean(b) => {
                state.write_u8(0);
                (if *b { "true" } else { "false" }).hash(state);
            },
            Value::Error(e) => {
                state.write_u8(1);
                format!("{e}").hash(state);
            },
            Value::Function(f) => {
                state.write_u8(2);
                state.write_usize(f.to_pointer() as usize);
                // Maybe also write the function as binary chunk?
            },
            Value::Integer(i) => {
                state.write_u8(3);
                state.write_i64(*i);
            },
            Value::LightUserData(ptr) => {
                state.write_u8(4);
                state.write_usize(ptr.0 as usize);
            },
            Value::Nil => {
                state.write_u8(5);
            },
            Value::Number(f) => {
                state.write_u8(6);
                format!("{f}").hash(state);
            },
            Value::String(s) => {
                state.write_u8(7);
                s.hash(state);
            },
            Value::Table(t) => {
                state.write_u8(8);
                state.write_usize(t.to_pointer() as usize);
            },
            Value::Thread(t) => {
                state.write_u8(9);
                state.write_usize(t.to_pointer() as usize);
            },
            Value::UserData(u) => {
                state.write_u8(10);
                state.write_usize(u.to_pointer() as usize);
            }
        }
    }
}