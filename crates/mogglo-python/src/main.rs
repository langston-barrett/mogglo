use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(
        tree_sitter_python::language(),
        tree_sitter_python::NODE_TYPES,
    )
}
