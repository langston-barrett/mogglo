use anyhow::Result;

use mogglo::cli;

fn main() -> Result<()> {
    cli::main(tree_sitter_java::language(), tree_sitter_java::NODE_TYPES)
}
