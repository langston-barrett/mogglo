use std::{
    fs,
    io::{self, Read},
    ops::Range,
    process,
};

use anyhow::{Context, Result};
use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use clap::Parser;
use tree_sitter::{Language, Tree};

use crate::{
    env::Env,
    pattern::{LuaCode, Pattern},
};

/// A multi-language AST-based code search and rewriting (codemod) tool
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Show details
    #[arg(long)]
    pub detail: bool,

    /// Print replacements, don't perform them
    #[arg(short, long)]
    pub dry_run: bool,

    /// Limit to this number of matches per file
    #[arg(long)]
    limit: Option<usize>,

    // Number of threads (TODO)
    // #[arg(short, long, default_value_t = num_cpus::get())]
    // pub jobs: usize,
    /// Behavior on parse errors
    #[arg(long, default_value_t = OnParseError::Ignore, value_name = "CHOICE")]
    on_parse_error: OnParseError,

    /// Print only the matched code
    #[arg(long)]
    pub only_matching: bool,

    /// Recursively match patterns
    #[arg(long)]
    pub recursive: bool,

    /// Pattern to replace with
    #[arg(short, long)]
    pub replace: Option<String>,

    /// Additional conditions on the match
    #[arg(short, long, value_name = "LUA")]
    pub r#where: Vec<String>,

    /// Pattern to search for, see the guide for details on pattern syntax
    #[arg()]
    pub pattern: String,

    /// Input files, use `-` to pass a single file on stdin
    #[arg(value_name = "FILE", required = true, num_args = 1..)]
    pub files: Vec<String>,
}

fn read_file(file: &str) -> Result<String> {
    fs::read_to_string(file).with_context(|| format!("Failed to read file {}", file))
}

#[inline]
fn stdin_string() -> Result<String> {
    let mut stdin_str: String = String::new();
    io::stdin().read_to_string(&mut stdin_str)?;
    Ok(stdin_str)
}

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum OnParseError {
    Ignore,
    Warn,
    Error,
}

impl std::fmt::Display for OnParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OnParseError::Ignore => write!(f, "ignore"),
            OnParseError::Warn => write!(f, "warn"),
            OnParseError::Error => write!(f, "error"),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for OnParseError {
    fn default() -> Self {
        OnParseError::Ignore
    }
}

fn handle_parse_errors(path: &str, tree: &Tree, on_parse_error: &OnParseError) {
    let node = tree.root_node();
    match on_parse_error {
        OnParseError::Ignore => (),
        OnParseError::Warn if !node.has_error() => (),
        OnParseError::Error if !node.has_error() => (),
        OnParseError::Warn => {
            eprintln!("[WARN] Parse error in {}", path);
        }
        OnParseError::Error => {
            eprintln!("[ERROR] Parse error in {}", path);
            process::exit(1);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn match_report(
    title: &str,
    path: &str,
    text: &str,
    range: Range<usize>,
    pattern: &str,
    env: &Env,
    detail: bool,
    match_title: &str,
) -> Result<()> {
    let mut colors = ColorGenerator::new();
    let mut builder = Report::build(ReportKind::Custom(title, Color::Green), path, range.start)
        .with_label(
            Label::new((path, range))
                .with_message(
                    if pattern.len() < 24 && detail {
                        pattern
                    } else {
                        match_title
                    }
                    .fg(Color::Cyan),
                )
                .with_order(i32::MAX)
                .with_color(Color::Cyan),
        );
    if detail {
        let mut multiple = Vec::new();
        for (mvar, nodes) in &env.0 {
            let color = colors.next();
            for (i, node) in nodes.iter().enumerate() {
                builder = builder.with_label(
                    Label::new((path, node.byte_range()))
                        .with_message(format!("${}", mvar.0).fg(color))
                        .with_color(color),
                );
                if i > 0 {
                    multiple.push((mvar, color));
                }
            }
        }
        for (mvar, color) in multiple {
            builder = builder.with_note(format!(
                "Multiple occurrences of {} were structurally equal",
                format!("${}", mvar.0).fg(color)
            ))
        }
    }
    builder.finish().print((path, Source::from(&text)))?;
    Ok(())
}

pub fn main(language: Language) -> Result<()> {
    let args = Args::parse();

    let mut pat = Pattern::parse(language, args.pattern.clone());
    pat.r#where(&mut args.r#where.into_iter().map(LuaCode));

    // TODO: Parallelize
    for f in &args.files {
        let (tree, text) = if f == "-" {
            let text = stdin_string()?;
            let tree = crate::pattern::parse(language, &text);
            (tree, text)
        } else {
            let text = read_file(f)?;
            let tree = crate::pattern::parse(language, &text);
            (tree, text)
        };
        handle_parse_errors(f, &tree, &args.on_parse_error);
        for m in pat.matches(&tree, &text, &Env::default(), args.recursive, args.limit) {
            if let Some(replace) = &args.replace {
                if args.only_matching {
                    let p = Pattern::parse(language, replace.to_string());
                    println!("{}", p.replace(vec![m], text.to_string()));
                    continue;
                }
                match_report(
                    if args.dry_run {
                        "Would replace"
                    } else {
                        "Replacing"
                    },
                    f,
                    &text,
                    m.root.byte_range(),
                    &args.pattern,
                    &m.env,
                    args.detail,
                    "Match",
                )?;
                let p = Pattern::parse(language, replace.to_string());
                // TODO: Computes replacement twice...
                let replacement = p.replacement(&m, &text);
                let replaced = p.replace(vec![m.clone()], text.clone());
                let start = replaced.find(&replacement).unwrap();
                match_report(
                    "With",
                    f,
                    &replaced,
                    start..start + replacement.len(),
                    &args.pattern,
                    &Env::default(),
                    args.detail,
                    "Replacement",
                )?;
                if !args.dry_run && f != "-" {
                    std::fs::write(f, replaced)?;
                }
            } else if args.only_matching {
                println!("{}", m.root.utf8_text(text.as_bytes()).unwrap());
            } else {
                match_report(
                    "Match",
                    f,
                    &text,
                    m.root.byte_range(),
                    &args.pattern,
                    &m.env,
                    args.detail,
                    "Match",
                )?;
            }
        }
    }
    Ok(())
}
