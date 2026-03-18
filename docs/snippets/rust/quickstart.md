```rust title="Rust"
use ts_pack_core::{get_parser, process, ProcessConfig};

fn main() -> anyhow::Result<()> {
    // Parsers download automatically on first use
    let mut parser = get_parser("rust")?;
    let tree = parser.parse("fn main() { println!(\"hello\"); }", None).unwrap();
    println!("{}", tree.root_node().to_sexp());

    // Extract code intelligence
    let config = ProcessConfig::new("rust").structure(true).imports(true);
    let result = process("fn main() { println!(\"hello\"); }", &config)?;
    println!("Functions: {}", result.structure.len());
    Ok(())
}
```
