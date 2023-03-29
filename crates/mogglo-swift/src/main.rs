use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(tree_sitter_swift::language(), tree_sitter_swift::NODE_TYPES)
}
