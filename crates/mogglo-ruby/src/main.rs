use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(tree_sitter_ruby::language(), tree_sitter_ruby::NODE_TYPES)
}
