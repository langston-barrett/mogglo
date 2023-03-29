use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(
        tree_sitter_typescript::language_typescript(),
        tree_sitter_typescript::TYPESCRIPT_NODE_TYPES,
    )
}
