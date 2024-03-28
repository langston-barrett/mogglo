use std::collections::{HashMap, HashSet};

use rlua::Lua;
use tree_sitter::{Language, Node, Tree};

use crate::{
    env::{Env, Metavar},
    lua::{eval_lua, eval_lua_scope, node::LuaNode, pattern::LuaPattern, LuaData},
    node_types::NodeTypes,
};

pub(crate) fn parse(language: Language, code: &str) -> Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(language)
        .expect("Failed to set tree-sitter parser language");
    parser.parse(code, None).expect("Failed to parse code")
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LuaCode(pub(crate) String);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TmpVar(String);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FindExpr {
    Anonymous,
    Ellipsis,
    Metavar(Metavar),
    Lua(LuaCode),
}

impl FindExpr {
    const ANONYMOUS: &'static str = "_";
    const ELLIPSIS: &'static str = "..";

    pub fn parse(s: String) -> Self {
        if s == Self::ANONYMOUS {
            return Self::Anonymous;
        }
        if s == Self::ELLIPSIS {
            return Self::Ellipsis;
        }
        Self::Metavar(Metavar(s))
    }
}

#[derive(Clone, Debug)]
pub struct Pattern<'nts> {
    exprs: HashMap<TmpVar, FindExpr>,
    lang: Language,
    node_types: &'nts NodeTypes<'nts>,
    root_id: usize,
    text: String,
    tree: Tree,
    r#where: Vec<LuaCode>,
}

#[derive(Copy, Clone)]
struct Goal<'tree> {
    node: Node<'tree>,
    text: &'tree str,
}

impl<'tree> Goal<'tree> {
    fn as_str(&self) -> &'tree str {
        self.node.utf8_text(self.text.as_bytes()).unwrap().trim()
    }

    fn child(&self, i: usize) -> Self {
        Self {
            node: self.node.child(i).unwrap(),
            text: self.text,
        }
    }

    fn _next_named_sibling(&self) -> Option<Self> {
        self.node.next_sibling().map(|node| Self {
            node,
            text: self.text,
        })
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling().map(|node| Self {
            node,
            text: self.text,
        })
    }

    fn _parent(&self) -> Option<Self> {
        self.node.parent().map(|node| Self {
            node,
            text: self.text,
        })
    }
}

#[derive(Copy, Clone)]
pub struct Candidate<'tree> {
    node: Node<'tree>,
    text: &'tree str,
}

impl<'tree> Candidate<'tree> {
    fn as_str(&self) -> &'tree str {
        self.node.utf8_text(self.text.as_bytes()).unwrap().trim()
    }

    fn child(&self, i: usize) -> Self {
        Self {
            node: self.node.child(i).unwrap(),
            text: self.text,
        }
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling().map(|node| Self {
            node,
            text: self.text,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Match<'tree> {
    pub(crate) env: Env<'tree>,
    pub root: Node<'tree>,
}

impl<'nts> Pattern<'nts> {
    fn meta(i: usize) -> TmpVar {
        TmpVar(format!("mogglo_tmp_var_{i}"))
    }

    // TODO: Disallow anything after ellipses
    fn parse_from(
        lang: Language,
        node_types: &'nts NodeTypes<'nts>,
        pat: String,
        mut vars: usize,
        unwrap_until: Option<&str>,
    ) -> Self {
        let mut peek = pat.chars().peekable();
        let mut nest = 0;
        let mut code = String::new();
        let mut text = String::new();
        let mut exprs = HashMap::new();
        while let Some(current) = peek.next() {
            if current == '$' {
                // ${{code}}
                if peek.next_if_eq(&'{').is_some() && peek.next_if_eq(&'{').is_some() {
                    if nest > 0 {
                        code += "${{"
                    }
                    nest += 1;
                    continue;
                } else if nest > 0 {
                    code += &String::from(current);
                    continue;
                }

                // $_
                if peek.next_if_eq(&'_').is_some() {
                    let tvar = Self::meta(vars);
                    vars += 1;
                    text += &tvar.0;
                    exprs.insert(tvar, FindExpr::Anonymous);
                }

                // $..
                if peek.next_if_eq(&'.').is_some() && peek.next_if_eq(&'.').is_some() {
                    let tvar = Self::meta(vars);
                    vars += 1;
                    text += &tvar.0;
                    exprs.insert(tvar, FindExpr::Ellipsis);
                }

                // $x
                let mvar_name: String =
                    peek.clone().take_while(char::is_ascii_alphabetic).collect();
                if !mvar_name.is_empty() {
                    peek.nth(mvar_name.len() - 1);
                }
                if !mvar_name.is_empty() {
                    let tvar = Self::meta(vars);
                    vars += 1;
                    text += &tvar.0;
                    exprs.insert(tvar, FindExpr::Metavar(Metavar(mvar_name)));
                    continue;
                }
            } else if current == '}' && peek.next_if_eq(&'}').is_some() {
                nest -= 1;
                if nest == 0 {
                    let tvar = Self::meta(vars);
                    vars += 1;
                    text += &tvar.0;
                    exprs.insert(tvar, FindExpr::Lua(LuaCode(code)));
                    code = String::new();
                } else {
                    code += "}}"
                }
            } else if nest > 0 {
                code += &String::from(current);
                continue;
            } else {
                text += &String::from(current);
                continue;
            }
        }

        // NOTE[expression-hack]: tree-sitter will try to parse the pattern
        // as a whole program, which can fail if e.g., the language is Rust
        // and the pattern is `$x + $y` (which is not valid at the top level
        // of a program). When parsing the pattern fails, we try wrapping the
        // pattern in braces or ending it with a semicolon, and then unwrapping
        // it into an expression when transforming it into a goal.
        //
        // Weggli appears to work similarly by default.
        let mut tree = parse(lang, &text);
        if tree.root_node().has_error() {
            text = format!("{{ {text} }}");
            tree = parse(lang, &text);
            if tree.root_node().has_error() {
                text = format!("{text};");
                tree = parse(lang, &text);
                if tree.root_node().has_error() {
                    eprintln!("[WARN] Parse error in pattern!");
                }
            }
        }
        let mut root = tree.root_node();
        // Get rid of top-level "program" node
        if root.child_count() == 1 {
            root = root.child(0).unwrap();
        }
        // See NOTE[expression-hack]
        while root.named_child_count() == 1 {
            if Some(root.kind()) == unwrap_until {
                break;
            }
            root = root.named_child(0).unwrap();
        }

        Self {
            exprs,
            lang,
            node_types,
            root_id: root.id(),
            text,
            tree,
            r#where: Vec::new(),
        }
    }

    pub fn parse_kind(
        lang: Language,
        node_types: &'nts NodeTypes<'nts>,
        pat: String,
        kind: &str,
    ) -> Self {
        Self::parse_from(lang, node_types, pat, 0, Some(kind))
    }

    pub fn parse(lang: Language, node_types: &'nts NodeTypes<'nts>, pat: String) -> Self {
        Self::parse_from(lang, node_types, pat, 0, None)
    }

    fn match_leaf_node(goal: Goal, candidate: Candidate) -> bool {
        debug_assert!(goal.node.child_count() == 0);
        goal.as_str() == candidate.as_str()
    }

    fn match_plain_node<'tree>(
        &self,
        lua: &Lua,
        mut env: Env<'tree>,
        goal: Goal,
        candidate: Candidate<'tree>,
    ) -> Option<Match<'tree>> {
        let goal_count = goal.node.child_count();
        let candidate_count = candidate.node.child_count();

        if goal_count == 0 {
            // ex:
            // candidate: { x; }
            // goal: { }
            if candidate_count != 0 {
                return None;
            }
            // ex:
            // candidate: x
            // goal: x
            if Self::match_leaf_node(goal, candidate) {
                return Some(Match {
                    env,
                    root: candidate.node,
                });
            }
            // ex:
            // candidate: x
            // goal: y
            return None;
        }

        if goal.node.kind_id() == candidate.node.kind_id() {
            let mut goal_child = goal.child(0);
            let mut candidate_child = candidate.child(0);
            loop {
                if let Some(FindExpr::Ellipsis) =
                    self.exprs.get(&TmpVar(goal_child.as_str().to_string()))
                {
                    return Some(Match {
                        env,
                        root: candidate.node,
                    });
                }
                if let Some(m) =
                    self.match_node_internal(lua, env.clone(), goal_child, candidate_child)
                {
                    env.extend(m.env);
                    match (goal_child.next_sibling(), candidate_child.next_sibling()) {
                        (Some(gnext), Some(cnext)) => {
                            goal_child = gnext;
                            candidate_child = cnext;
                        }
                        (None, Some(_)) => {
                            return Some(Match {
                                env,
                                root: candidate.node,
                            })
                        }
                        (Some(gnext), None) => {
                            // Might be an ellipsis
                            goal_child = gnext;
                        }
                        (None, None) => {
                            return Some(Match {
                                env,
                                root: candidate.node,
                            })
                        }
                    }
                } else {
                    match candidate_child.next_sibling() {
                        None => return None,
                        Some(cnext) => candidate_child = cnext,
                    }
                }
            }
        } else {
            // Match goal with any child
            for i in 0..candidate.node.child_count() {
                // TODO: rm clone
                if let Some(m) =
                    self.match_node_internal(lua, env.clone(), goal, candidate.child(i))
                {
                    return Some(m);
                }
            }
            None
        }
    }

    fn match_expr<'tree>(
        &self,
        lua: &Lua,
        mut env: Env<'tree>,
        expr: &FindExpr,
        candidate: Candidate<'tree>,
    ) -> Option<Match<'tree>> {
        match expr {
            FindExpr::Anonymous => Some(Match {
                env,
                root: candidate.node,
            }),
            FindExpr::Ellipsis => panic!("Unhandled ellipsis"),
            FindExpr::Metavar(m) => match env.0.get(m) {
                None => {
                    env.insert(m.clone(), candidate.node);
                    Some(Match {
                        env,
                        root: candidate.node,
                    })
                }
                Some(goals) => {
                    let mut extended = env.clone();
                    for goal in goals {
                        // TODO: debug assert all goals are matched
                        let goal = Goal {
                            node: *goal,
                            text: candidate.text,
                        };
                        let mch = self.match_plain_node(lua, extended.clone(), goal, candidate)?;
                        extended.insert(m.clone(), mch.root);
                    }
                    Some(Match {
                        env: extended,
                        root: candidate.node,
                    })
                }
            },
            FindExpr::Lua(LuaCode(code)) => {
                let data = LuaData {
                    env: &env,
                    node_types: self.node_types,
                    text: candidate.text,
                };
                let mut binds = Env::default();
                // TODO: Handle errors
                let matched = lua.context(|lua_ctx| {
                    let loaded = match lua_ctx.load(code).set_name("lua code") {
                        Err(e) => {
                            eprintln!("Bad Lua code: {code}");
                            return Err(e);
                        }
                        Ok(l) => l,
                    };
                    lua_ctx.scope(|scope| {
                        let globals = lua_ctx.globals();
                        globals.set("focus", LuaNode::new(candidate.node, candidate.text))?;
                        globals.set("t", candidate.as_str())?;
                        globals.set(
                            "bind",
                            scope.create_function_mut(|_, m: String| {
                                binds.insert(Metavar(m), candidate.node);
                                Ok(())
                            })?,
                        )?;
                        // TODO: Option to export metavariables
                        globals.set(
                            "match",
                            scope.create_function(|_, p: String| {
                                let pat = Pattern::parse_from(
                                    self.lang,
                                    self.node_types,
                                    p,
                                    self.exprs.len(),
                                    None,
                                );
                                Ok(pat
                                    .match_node_internal(lua, env.clone(), pat.to_goal(), candidate)
                                    .is_some())
                            })?,
                        )?;

                        globals.set(
                            "pat",
                            scope.create_function(|_, p: String| {
                                let pat = Pattern::parse_from(
                                    self.lang,
                                    self.node_types,
                                    p,
                                    self.exprs.len(),
                                    None,
                                );
                                Ok(LuaPattern::new(pat))
                            })?,
                        )?;

                        globals.set(
                            "pmatch",
                            scope.create_function(|_, (p, n): (LuaPattern, LuaNode)| {
                                Ok(p.0
                                    .match_node_internal(
                                        lua,
                                        env.clone(),
                                        p.0.to_goal(),
                                        Candidate {
                                            node: n.node,
                                            text: n.text,
                                        },
                                    )
                                    .is_some())
                            })?,
                        )?;

                        // TODO: Option to export metavariables
                        globals.set(
                            "rec",
                            scope.create_function(|_, p: String| {
                                let pat = Pattern::parse_from(
                                    self.lang,
                                    self.node_types,
                                    p,
                                    self.exprs.len(),
                                    None,
                                );
                                Ok(!pat
                                    .matches_internal(
                                        candidate.text,
                                        candidate.node,
                                        &env,
                                        true,
                                        Some(1),
                                    )
                                    .is_empty())
                            })?,
                        )?;
                        eval_lua_scope::<bool>(lua_ctx, scope, loaded, &data)
                    })
                });
                // TODO: Maybe check for collisions
                env.extend(binds);
                match matched {
                    Ok(true) => Some(Match {
                        env,
                        root: candidate.node,
                    }),
                    Ok(false) => None,
                    Err(e) => {
                        eprintln!("{e}");
                        None
                    }
                }
            }
        }
    }

    fn match_node_internal<'tree>(
        &self,
        lua: &Lua,
        env: Env<'tree>,
        goal: Goal,
        candidate: Candidate<'tree>,
    ) -> Option<Match<'tree>> {
        // TODO: Avoid allocation
        match self.exprs.get(&TmpVar(goal.as_str().to_string())) {
            None => self.match_plain_node(lua, env, goal, candidate),
            Some(expr) => self.match_expr(lua, env, expr, candidate),
        }
    }

    pub fn match_node<'s, 'tree>(
        &'s self,
        env: Env<'tree>,
        candidate: Candidate<'tree>,
    ) -> Option<Match<'tree>>
    where
        'tree: 's,
    {
        let lua = Lua::new();
        if let Some(m) = self.match_node_internal(&lua, env, self.to_goal(), candidate) {
            for LuaCode(c) in &self.r#where {
                let data = LuaData {
                    env: &m.env,
                    node_types: self.node_types,
                    text: candidate.text,
                };
                match eval_lua::<bool>(&lua, c, &data) {
                    Ok(b) if b => (),
                    Ok(_) => return None,
                    Err(e) => {
                        eprintln!("Error in Lua: {c}");
                        eprintln!("{e}");
                        return None;
                    }
                }
            }
            Some(m)
        } else {
            None
        }
    }

    // TODO: Only named children
    // TODO: Minimum match size
    fn matches_internal<'tree>(
        &self,
        text: &'tree str,
        node: Node<'tree>,
        env: &Env<'tree>,
        recursive: bool,
        limit: Option<usize>,
    ) -> Vec<Match<'tree>> {
        let mut cursor = node.walk();
        let mut nodes: Vec<_> = node.children(&mut cursor).collect();
        let mut ms = Vec::new();
        let mut ranges = HashSet::new();
        while !nodes.is_empty() {
            let mut next = Vec::with_capacity(nodes.len()); // guess
            for node in nodes {
                let candidate = Candidate { node, text };
                if let Some(m) = self.match_node(env.clone(), candidate) {
                    if ranges.contains(&m.root.byte_range()) {
                        continue;
                    }
                    ranges.insert(m.root.byte_range());
                    ms.push(m);
                    if limit.map(|l| ms.len() >= l).unwrap_or(false) {
                        return ms;
                    }
                    if !recursive {
                        continue;
                    }
                }
                let mut child_cursor = node.walk();
                for child in node.children(&mut child_cursor) {
                    next.push(child);
                }
            }
            nodes = next;
        }
        ms
    }

    pub fn matches<'tree>(
        &self,
        tree: &'tree Tree,
        text: &'tree str,
        env: &Env<'tree>,
        recursive: bool,
        limit: Option<usize>,
    ) -> Vec<Match<'tree>> {
        self.matches_internal(text, tree.root_node(), env, recursive, limit)
    }

    fn to_goal(&self) -> Goal {
        let mut goal = self.tree.root_node();
        // See NOTE[expression-hack]
        while goal.id() != self.root_id {
            if goal.child_count() == 1 {
                goal = goal.child(0).unwrap();
            } else {
                debug_assert_eq!(goal.named_child_count(), 1);
                goal = goal.named_child(0).unwrap();
            }
        }
        Goal {
            node: goal,
            text: &self.text,
        }
    }

    pub fn replacement(&self, m: &Match, text: &str) -> String {
        // See NOTE[expression-hack] for why this isn't just self.text
        let mut replacement = self
            .to_goal()
            .node
            .utf8_text(self.text.as_bytes())
            .unwrap()
            .to_string();

        for (tvar, expr) in &self.exprs {
            match expr {
                FindExpr::Anonymous => {
                    eprintln!("`$_` is not valid in replacements");
                    return String::new();
                }
                FindExpr::Ellipsis => {
                    eprintln!("`$..` is not valid in replacements");
                    return String::new();
                }
                FindExpr::Metavar(mvar @ Metavar(mtxt)) => match m.env.0.get(mvar) {
                    Some(matching_nodes) => {
                        if let Some(node) = matching_nodes.iter().next() {
                            replacement = replacement
                                .replace(&tvar.0, node.utf8_text(text.as_bytes()).unwrap());
                        }
                    }
                    None => {
                        eprintln!("Bad metavariable in replacement: {mtxt}");
                        return String::new();
                    }
                },
                FindExpr::Lua(LuaCode(code)) => {
                    let lua = Lua::new();
                    let data = LuaData {
                        env: &m.env,
                        node_types: self.node_types,
                        text,
                    };
                    match eval_lua::<String>(&lua, code, &data) {
                        Ok(evaled) => replacement = replacement.replace(&tvar.0, &evaled),
                        Err(e) => {
                            eprintln!("{e}")
                        }
                    };
                }
            }
        }
        replacement
    }

    pub fn replace(&self, m: Match, text: &mut String, offset: isize) -> (usize, usize) {
        let mut start = isize::try_from(m.root.start_byte()).unwrap();
        start += offset;
        let start = usize::try_from(start).unwrap();
        let mut end = isize::try_from(m.root.end_byte()).unwrap();
        end += offset;
        let end = usize::try_from(end).unwrap();
        text.replace_range(start..end, &self.replacement(&m, &text));
        (start, end)
    }

    pub fn r#where(&mut self, iter: &mut impl Iterator<Item = LuaCode>) {
        self.r#where.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use tree_sitter::Tree;
    use tree_sitter_rust::language;

    use crate::node_types::NodeTypes;

    use super::{Candidate, Env, FindExpr, LuaCode, Match, Metavar, Pattern};

    lazy_static::lazy_static! {
        /// This is an example for using doc comment attributes
        static ref NODE_TYPES: NodeTypes<'static> = NodeTypes::new(tree_sitter_rust::NODE_TYPES).unwrap();
    }

    fn pat(s: &str) -> Pattern {
        Pattern::parse(language(), &NODE_TYPES, s.to_string())
    }

    fn match_one<'tree>(s: &str, tree: &'tree Tree, text: &'tree str) -> Option<Env<'tree>> {
        let candidate = Candidate {
            node: tree.root_node(),
            text,
        };
        Pattern::parse(language(), &NODE_TYPES, s.to_string())
            .match_node(Env::default(), candidate)
            .map(|m| m.env)
    }

    fn matches<'tree>(
        s: &str,
        tree: &'tree Tree,
        text: &'tree str,
    ) -> Option<HashMap<Metavar, HashSet<&'tree str>>> {
        match_one(s, tree, text).map(|m| {
            m.0.into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        v.into_iter()
                            .map(|n| n.utf8_text(text.as_bytes()).unwrap())
                            .collect(),
                    )
                })
                .collect()
        })
    }

    fn match_all<'tree>(s: &str, tree: &'tree Tree, text: &'tree str) -> Vec<Match<'tree>> {
        Pattern::parse(language(), &NODE_TYPES, s.to_string()).matches(
            tree,
            text,
            &Env::default(),
            false,
            None,
        )
    }

    fn all_matches<'tree>(
        s: &str,
        tree: &'tree Tree,
        text: &'tree str,
    ) -> Vec<HashMap<Metavar, HashSet<&'tree str>>> {
        match_all(s, tree, text)
            .into_iter()
            .map(|m| {
                m.env
                    .0
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k,
                            v.iter()
                                .map(|n| n.utf8_text(text.as_bytes()).unwrap())
                                .collect(),
                        )
                    })
                    .collect()
            })
            .collect()
    }

    fn replace(text: &str, find: &str, replace: &str) -> String {
        let tree = super::parse(language(), text);
        let candidate = Candidate {
            node: tree.root_node(),
            text,
        };
        let m = Pattern::parse(language(), &NODE_TYPES, find.to_string())
            .match_node(Env::default(), candidate)
            .unwrap();
        let p = Pattern::parse(language(), &NODE_TYPES, replace.to_string());
        let mut text = text.to_string();
        p.replace(m, &mut text, 0);
        text
    }

    #[test]
    fn test_pattern_parse() {
        assert_eq!(HashMap::new(), pat("").exprs);
        assert_eq!(
            HashMap::from([(
                Pattern::meta(0),
                FindExpr::Metavar(Metavar("x".to_string()))
            )]),
            pat("$x").exprs
        );
        assert_eq!(
            HashMap::from([(Pattern::meta(0), FindExpr::Anonymous)]),
            pat("$_").exprs
        );
        assert_eq!(
            HashMap::from([(Pattern::meta(0), FindExpr::Lua(LuaCode("true".to_string())))]),
            pat("${{true}}").exprs
        );
        assert_eq!(
            HashMap::from([
                (Pattern::meta(0), FindExpr::Lua(LuaCode("true".to_string()))),
                (
                    Pattern::meta(1),
                    FindExpr::Lua(LuaCode("false".to_string()))
                )
            ]),
            pat("${{true}} == ${{false}}").exprs
        );
        assert_eq!(
            HashMap::from([(
                Pattern::meta(0),
                FindExpr::Lua(LuaCode(r#"match("$x")"#.to_string()))
            )]),
            pat(r#"${{match("$x")}}"#).exprs
        );
        assert_eq!(
            HashMap::from([
                (
                    Pattern::meta(0),
                    FindExpr::Metavar(Metavar("x".to_string()))
                ),
                (
                    Pattern::meta(1),
                    FindExpr::Metavar(Metavar("y".to_string()))
                )
            ]),
            pat("let $x = $y;").exprs
        );
        assert_eq!(
            HashMap::from([(
                Pattern::meta(0),
                FindExpr::Lua(LuaCode(r#"not match("${{false}}")"#.to_string()))
            ),]),
            pat(r#"${{not match("${{false}}")}}"#).exprs
        );
    }

    #[test]
    fn test_matches() {
        let tree = super::parse(language(), "");
        assert_eq!(Some(Env::default()), match_one("$_", &tree, ""));

        let text = "a";
        let tree = super::parse(language(), text);
        assert_eq!(Some(HashMap::new()), matches("$_", &tree, text));

        let text = "let a = b;";
        let tree = super::parse(language(), text);
        assert_eq!(
            Some(HashMap::from([
                (Metavar("x".to_string()), HashSet::from(["a"])),
                (Metavar("y".to_string()), HashSet::from(["b"]))
            ])),
            matches("let $x = $y;", &tree, text)
        );

        let text = "let a = a;";
        let tree = super::parse(language(), text);
        assert_eq!(
            Some(HashMap::from([(
                Metavar("x".to_string()),
                HashSet::from(["a"])
            )])),
            matches("let $x = $x;", &tree, text)
        );

        let text = "let a = b;";
        let tree = super::parse(language(), text);
        assert_eq!(None, matches("let $x = $x;", &tree, text));

        let text = "0 + 1";
        let tree = super::parse(language(), text);
        assert_eq!(
            Some(HashMap::from([
                (Metavar("x".to_string()), HashSet::from(["0"])),
                (Metavar("y".to_string()), HashSet::from(["1"]))
            ])),
            matches("$x + $y", &tree, text)
        );

        // TODO:
        // let text = "let a = a;";
        // let tree = super::parse(language(), text);
        // assert_eq!(Some(HashMap::new()), matches("$/a/", &tree, text));
        // assert_eq!(Some(HashMap::new()), matches("$/./", &tree, text));

        // TODO:
        // let text = "let foo = 0 == 1;";
        // let text = "0 == 1;";
        // let tree = super::parse(language(), text);
        // assert_eq!(Some(HashMap::new()), matches("$_ == $_", &tree, text));

        let text = "if a ==  () { }";
        let tree = super::parse(language(), text);
        assert_eq!(
            Some(HashMap::from([
                (Metavar("x".to_string()), HashSet::from(["a"])),
                (Metavar("y".to_string()), HashSet::from(["()"]))
            ])),
            matches("if $x == $y {}", &tree, text)
        );

        // let text = "{ a; b; c; }";
        // let tree = super::parse(language(), text);
        // assert_eq!(
        //     Some(HashMap::from([(
        //         Metavar("x".to_string()),
        //         HashSet::from(["a"])
        //     )])),
        //     matches("{ $x; }", &tree, text)
        // );

        let text = "{ a; b; c + d; }";
        let tree = super::parse(language(), text);
        assert_eq!(
            Some(HashMap::from([
                (Metavar("x".to_string()), HashSet::from(["a"])),
                (Metavar("y".to_string()), HashSet::from(["c"])),
                (Metavar("z".to_string()), HashSet::from(["d"]))
            ])),
            matches("{ $x; $y + $z; }", &tree, text)
        );

        let text = "if a == () { let b = c; }";
        let tree = super::parse(language(), text);
        assert_eq!(
            Some(HashMap::from([
                (Metavar("x".to_string()), HashSet::from(["a"])),
                (Metavar("y".to_string()), HashSet::from(["()"]))
            ])),
            matches("if $x == $y { $.. }", &tree, text)
        );
    }

    #[test]
    fn test_ellipses() {
        // let text = "{ a; b; c; }";
        // let tree = super::parse(language(), text);
        // assert_eq!(Some(HashMap::new()), matches("{ $.. }", &tree, text));

        let text = "{ a; b; c; }";
        let tree = super::parse(language(), text);
        assert_eq!(
            Some(HashMap::from([(
                Metavar("x".to_string()),
                HashSet::from(["a"])
            ),])),
            matches("{ $x; $.. }", &tree, text)
        );

        let text = "{ a; b; c; }";
        let tree = super::parse(language(), text);
        assert_eq!(
            Some(HashMap::from([(
                Metavar("x".to_string()),
                HashSet::from(["b"])
            ),])),
            matches("{ $..; $x; $.. }", &tree, text)
        );
    }

    #[test]
    fn test_all_matches() {
        let text = "if a == () { let b = c; }";
        let tree = super::parse(language(), text);
        assert_eq!(
            Vec::from([HashMap::from([
                (Metavar("x".to_string()), HashSet::from(["b"])),
                (Metavar("y".to_string()), HashSet::from(["c"]))
            ])]),
            all_matches("let $x = $y;", &tree, text)
        );
    }

    #[test]
    fn test_replace() {
        assert_eq!("a", replace("let a = b;", "let $x = $y;", "$x"));
        assert_eq!(
            "let b = a;",
            replace("let a = b;", "let $x = $y;", "let $y = $x;")
        );
        assert_eq!("", replace("let a = b;", "let $x = $y;", r#"${{""}}"#));
    }
}
