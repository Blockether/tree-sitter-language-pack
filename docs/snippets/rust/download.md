```rust title="Rust"
use tree_sitter_language_pack::{PackConfig, init, download, downloaded_languages};

fn main() -> anyhow::Result<()> {
    // Pre-download specific languages
    download(&["python", "javascript", "rust"])?;

    // Or initialize with config
    let config = PackConfig {
        languages: Some(vec!["python".into(), "go".into()]),
        cache_dir: Some("/tmp/parsers".into()),
        ..Default::default()
    };
    init(&config)?;

    println!("{:?}", downloaded_languages());
    Ok(())
}
```
