use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(tree_sitter_html::language())
}
