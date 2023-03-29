use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(tree_sitter_rust::language(), tree_sitter_rust::NODE_TYPES)
}
