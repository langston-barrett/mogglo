use std::collections::{HashMap, HashSet};

use tree_sitter::Node;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Metavar(pub(crate) String);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Env<'tree>(pub(crate) HashMap<Metavar, HashSet<Node<'tree>>>);

impl<'tree> Env<'tree> {
    pub fn extend(&mut self, env: Env<'tree>) {
        for (mvar, bindings) in env.0 {
            self.0
                .entry(mvar)
                .or_insert_with(HashSet::new)
                .extend(bindings);
        }
    }

    pub fn insert(&mut self, mvar: Metavar, node: Node<'tree>) {
        self.0.entry(mvar).or_insert_with(HashSet::new).insert(node);
    }
}
