use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(tree_sitter_css::language(), tree_sitter_css::NODE_TYPES)
}
