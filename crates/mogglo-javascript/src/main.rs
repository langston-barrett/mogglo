use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(
        tree_sitter_javascript::language(),
        tree_sitter_javascript::NODE_TYPES,
    )
}
