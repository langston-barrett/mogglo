use rlua::UserData;
use tree_sitter::Node;

#[derive(Clone, Copy, Debug)]
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

    fn child(&self, idx: usize) -> Option<Self> {
        self.node.child(idx).map(|n| self.with_node(n))
    }

    fn child_count(&self) -> usize {
        self.node.child_count()
    }

    fn kind(&self) -> &'static str {
        self.node.kind()
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

    fn text(&self) -> &'static str {
        self.node.utf8_text(self.text.as_bytes()).unwrap()
    }
}

impl UserData for LuaNode {
    fn add_methods<'lua, T: rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("child", |_, this, i: usize| Ok(this.child(i)));
        methods.add_method("child_count", |_, this, _: ()| Ok(this.child_count()));
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
        methods.add_method("text", |_, this, _: ()| Ok(this.text()));
    }
}
