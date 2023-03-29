use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(tree_sitter_c::language(), tree_sitter_c::NODE_TYPES)
}
