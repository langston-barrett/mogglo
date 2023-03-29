use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(tree_sitter_html::language(), tree_sitter_html::NODE_TYPES)
}
