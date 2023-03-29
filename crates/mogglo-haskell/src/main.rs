use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(
        tree_sitter_haskell::language(),
        tree_sitter_haskell::NODE_TYPES,
    )
}
