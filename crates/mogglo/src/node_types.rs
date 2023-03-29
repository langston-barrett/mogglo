// For use of Cow, see https://github.com/serde-rs/serde/issues/1413

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

/// node-types.json
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Node<'a> {
    #[serde(rename(deserialize = "type", serialize = "type"))]
    ty: Cow<'a, str>,
    named: bool,
    #[serde(default, borrow)] // empty
    children: Children<'a>,
    #[serde(default)] // empty
    fields: HashMap<Cow<'a, str>, Field<'a>>,
    #[serde(default)] // empty
    subtypes: Vec<Subtype<'a>>,
}

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
struct Children<'a> {
    multiple: bool,
    required: bool,
    #[serde(borrow)]
    types: Vec<Subtype<'a>>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Field<'a> {
    pub multiple: bool,
    pub required: bool,
    #[serde(borrow)]
    pub types: Vec<Subtype<'a>>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Subtype<'a> {
    #[serde(rename(deserialize = "type", serialize = "type"))]
    pub ty: Cow<'a, str>,
    pub named: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeTypes<'a> {
    parents: HashMap<Cow<'a, str>, HashSet<Cow<'a, str>>>,
    children: HashMap<Cow<'a, str>, HashSet<Cow<'a, str>>>,
}

impl<'a> NodeTypes<'a> {
    pub fn new(node_types_json_str: &'a str) -> Result<Self, serde_json::Error> {
        let nodes: Vec<Node> = serde_json::from_str(node_types_json_str)?;
        let mut parents = HashMap::with_capacity(nodes.len());
        let mut children = HashMap::with_capacity(nodes.len());
        for node in nodes {
            let mut subs = HashSet::with_capacity(node.subtypes.len());
            for sub in node.subtypes.into_iter() {
                parents
                    .entry(sub.ty.clone())
                    .or_insert_with(HashSet::new)
                    .insert(node.ty.clone());
                subs.insert(sub.ty);
            }
            children.insert(node.ty, subs);
        }
        Ok(NodeTypes { parents, children })
    }

    pub(crate) fn is_child_of(&self, child: &str, parent: &str) -> bool {
        self.children
            .get(parent)
            .map_or(false, |cs| cs.contains(child))
    }

    pub(crate) fn is_descendant_of(&self, desc: &str, ansc: &str) -> bool {
        ansc == desc
            || self.children.get(ansc).map_or(false, |cs| {
                cs.contains(desc) || cs.iter().any(|c| self.is_descendant_of(desc, c))
            })
    }

    pub(crate) fn is_parent_of(&self, parent: &str, child: &str) -> bool {
        self.parents
            .get(child)
            .map_or(false, |ps| ps.contains(parent))
    }

    pub(crate) fn is_ancestor_of(&self, ansc: &str, desc: &str) -> bool {
        ansc == desc
            || self.children.get(ansc).map_or(false, |cs| {
                cs.contains(desc) || cs.iter().any(|c| self.is_descendant_of(c, desc))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optional() {
        let nt = NodeTypes::new(tree_sitter_rust::NODE_TYPES).unwrap();
        assert!(nt.is_child_of("array_expression", "_expression"));
        assert!(nt.is_descendant_of("array_expression", "_expression"));
        assert!(nt.is_parent_of("_expression", "array_expression"));
        assert!(!nt.is_child_of("_expression", "empty_statement"));
        assert!(!nt.is_child_of("empty_statement", "_expression"));
    }
}
