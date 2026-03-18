```rust title="Rust"
use tree_sitter_language_pack::{ProcessConfig, process};

fn main() -> anyhow::Result<()> {
    let config = ProcessConfig::new("python")
        .all()
        .with_chunking(1000);

    let result = process("def hello(): pass\ndef world(): pass", &config)?;

    for item in &result.structure {
        println!("{}: {}", item.kind, item.name);
    }
    for chunk in &result.chunks {
        println!("chunk: lines {}-{}", chunk.start_line, chunk.end_line);
    }
    Ok(())
}
```
