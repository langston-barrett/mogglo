use regex::Regex;
use rlua::{prelude::LuaError, Lua};
use tree_sitter::Node;

use crate::env::{Env, Metavar};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct LuaData<'a> {
    pub(crate) env: &'a Env<'a>,
    pub(crate) text: &'a str,
}

impl<'a> LuaData<'a> {
    fn node_text(&self, node: &Node) -> &str {
        node.utf8_text(self.text.as_bytes()).unwrap()
    }
}

pub(crate) fn eval_lua_scope<
    'lua,
    'scope,
    T: Clone + Default + for<'l> rlua::FromLua<'l> + 'static,
>(
    lua_ctx: rlua::prelude::LuaContext<'lua>,
    scope: &rlua::Scope<'lua, 'scope>,
    chunk: rlua::Chunk,
    data: &'scope LuaData<'scope>,
) -> Result<T, LuaError>
where
    'lua: 'scope,
{
    let globals = lua_ctx.globals();
    for (mvar, val) in &data.env.0 {
        if let Some(v) = val.iter().next() {
            globals.set(mvar.0.clone(), data.node_text(v))?;
        }
    }

    globals.set(
        "meta",
        scope.create_function(|_, k: String| {
            Ok(data.env.0.get(&Metavar(k)).and_then(|s| {
                let v: Vec<_> = s.iter().collect();
                if v.len() == 1 {
                    Some(v[0].utf8_text(data.text.as_bytes()).unwrap())
                } else {
                    None
                }
            }))
        })?,
    )?;

    globals.set(
        "rx",
        scope.create_function(|_, (r, s): (String, String)| match Regex::new(&r) {
            Err(_) => {
                eprintln!("Bad regex in Lua code: {r}");
                Ok(false)
            }
            Ok(x) => Ok(x.is_match(&s)),
        })?,
    )?;

    chunk.eval::<T>()
}

pub(crate) fn eval_lua_ctx<T: Clone + Default + for<'l> rlua::FromLua<'l> + 'static>(
    lua_ctx: rlua::prelude::LuaContext,
    code: &str,
    data: &LuaData,
) -> Result<T, LuaError>
where
{
    let loaded = lua_ctx.load(code.as_bytes());
    lua_ctx.scope(|scope| eval_lua_scope(lua_ctx, scope, loaded, data))
}

pub(crate) fn eval_lua<T: Clone + Default + for<'l> rlua::FromLua<'l> + 'static>(
    lua: &Lua,
    code: &str,
    data: &LuaData,
) -> Result<T, LuaError>
where
{
    lua.context(|lua_ctx| eval_lua_ctx(lua_ctx, code, data))
}
