use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(tree_sitter_cpp::language(), tree_sitter_cpp::NODE_TYPES)
}
