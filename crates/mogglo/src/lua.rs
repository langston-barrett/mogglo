use regex::Regex;
use rlua::{prelude::LuaError, Lua, UserData};
use tree_sitter::Node;

use crate::env::{Env, Metavar};

pub(crate) struct LuaNode {
    pub(crate) node: Node<'static>,
    pub(crate) text: &'static str,
}

// This will probably turn out fine...
unsafe impl Send for LuaNode {}

impl LuaNode {
    pub(crate) fn new(n: Node, text: &str) -> Self {
        // Yeah, no worries...
        Self {
            node: unsafe { std::mem::transmute(n) },
            text: unsafe { std::mem::transmute(text) },
        }
    }

    fn with_node(&self, n: Node) -> Self {
        Self {
            node: unsafe { std::mem::transmute(n) },
            text: self.text,
        }
    }

    fn kind(&self) -> String {
        String::from(self.node.kind())
    }

    fn next_named_sibling(&self) -> Option<Self> {
        self.node.next_named_sibling().map(|n| self.with_node(n))
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling().map(|n| self.with_node(n))
    }

    fn parent(&self) -> Option<Self> {
        self.node.parent().map(|n| self.with_node(n))
    }

    fn prev_named_sibling(&self) -> Option<Self> {
        self.node.prev_named_sibling().map(|n| self.with_node(n))
    }

    fn prev_sibling(&self) -> Option<Self> {
        self.node.prev_sibling().map(|n| self.with_node(n))
    }
}

impl UserData for LuaNode {
    fn add_methods<'lua, T: rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("kind", |_, this, _: ()| Ok(this.kind()));
        methods.add_method("next_named_sibling", |_, this, _: ()| {
            Ok(this.next_named_sibling())
        });
        methods.add_method("next_sibling", |_, this, _: ()| Ok(this.next_sibling()));
        methods.add_method("parent", |_, this, _: ()| Ok(this.parent()));
        methods.add_method("prev_named_sibling", |_, this, _: ()| {
            Ok(this.prev_named_sibling())
        });
        methods.add_method("prev_sibling", |_, this, _: ()| Ok(this.prev_sibling()));
    }
}

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
