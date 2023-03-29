use rlua::UserData;

use crate::pattern::Pattern;

#[derive(Clone, Debug)]
pub(crate) struct LuaPattern(pub(crate) Pattern<'static>);

// This will probably turn out fine...
unsafe impl Send for LuaPattern {}

impl LuaPattern {
    pub(crate) fn new(p: Pattern<'_>) -> Self {
        Self(unsafe { std::mem::transmute(p) })
    }
}

impl UserData for LuaPattern {
    fn add_methods<'lua, T: rlua::UserDataMethods<'lua, Self>>(_methods: &mut T) {}
}
