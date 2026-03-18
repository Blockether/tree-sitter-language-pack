---
description: "Configuring tree-sitter-language-pack — cache directories, pre-downloads, configuration files, and discovery."
---

# Configuration Guide

Tree-sitter-language-pack supports multiple configuration methods: TOML files, programmatic APIs, CLI commands, and environment variables. This guide covers setting up caching, pre-downloading languages, and discovering configuration automatically.

## Overview

Configuration controls:

- **Cache directory**: Where downloaded parser binaries are stored
- **Pre-downloads**: Which languages to download on initialization
- **Discovery**: Automatic config file search paths

The cache is typically:

```text
~/.cache/tree-sitter-language-pack/v1.0.0/libs/
```text

But you can customize it for:

- **CI environments**: Use a job-specific cache directory
- **Offline environments**: Pre-download languages, use a frozen cache
- **Docker**: Mount a persistent volume for the cache
- **Monorepos**: Use a project-local cache directory

## Language Pack Configuration File

The `language-pack.toml` file defines languages to pre-download and cache settings.

### File Format

=== "TOML"

    ```toml
    # language-pack.toml

    # Language names to pre-download
    languages = ["python", "javascript", "typescript", "rust"]

    # Optional: language groups to pre-download
    # groups = ["web", "systems", "data"]

    # Optional: custom cache directory
    # cache_dir = ".cache/parsers"
    ```

### Creating a Configuration File

#### CLI Method

=== "Bash"

    ```bash
    # Create language-pack.toml with interactive prompts
    ts-pack init

    # Create with specific languages
    ts-pack init --languages python,javascript,typescript,rust

    # Specify custom cache directory
    ts-pack init --cache-dir ./local-cache --languages python,rust
    ```

#### Manual Method

Create `language-pack.toml` in your project root:

```toml
[pack]
# Languages to pre-download (run: ts-pack download)
languages = [
    "python",
    "javascript",
    "typescript",
    "rust",
    "go",
]

# Optional: language groups (web, systems, data, jvm, functional, scripting)
# groups = ["web", "systems"]

# Optional: use a project-local cache directory
# cache_dir = ".cache/ts-pack"

# Optional: override default cache directory
# cache_dir = "/var/lib/ts-pack-cache"
```text

### Configuration Discovery

The CLI and binding libraries search for `language-pack.toml` in this order:

1. **Current directory and parent directories (up to 10 levels)**
   - `./language-pack.toml`
   - `../language-pack.toml`
   - `../../language-pack.toml`
   - etc.

2. **XDG config directory**
   - `$XDG_CONFIG_HOME/tree-sitter-language-pack/config.toml`
   - `~/.config/tree-sitter-language-pack/config.toml` (Linux/macOS)
   - `%APPDATA%\tree-sitter-language-pack\config.toml` (Windows)

3. **Environment variable**
   - `TSLP_CONFIG=/path/to/language-pack.toml`

!!! tip "Priority Order"
    CLI flags > Environment variables > Config file > Defaults

    A command-line flag always overrides config file and environment settings.

### Example: Monorepo Setup

For a monorepo with multiple language sub-projects:

```toml
# language-pack.toml (at repo root)

[pack]
languages = [
    # Python backend
    "python",
    "django",  # if available as separate language

    # JavaScript/Node.js frontend
    "javascript",
    "typescript",
    "jsx",
    "tsx",

    # Rust utilities
    "rust",

    # DevOps
    "bash",
    "dockerfile",
    "yaml",
    "json",
]

# Share a single cache across the entire monorepo
cache_dir = ".cache/tree-sitter"
```text

### Example: Docker Setup

In a Dockerfile, pre-download languages to avoid network calls at runtime:

```dockerfile
FROM python:3.11

# Install ts-pack
RUN pip install tree-sitter-language-pack

# Create configuration
RUN mkdir -p /app && cd /app && \
    ts-pack init --languages python,javascript,rust

# Pre-download all languages (bakes them into the image)
RUN ts-pack download --all

# Copy your code
COPY . /app
WORKDIR /app

# Now parsing is fast and offline
RUN python -c "from tree_sitter_language_pack import parse_string; print(parse_string('x = 1', 'python'))"
```text

## Programmatic Configuration

### Python

=== "Basic Usage"

    ```python
    from tree_sitter_language_pack import init, configure, get_parser

    # Pre-download specific languages
    init(languages=["python", "javascript", "rust"])

    # Now use the library
    parser = get_parser("python")
    tree = parser.parse(b"x = 1")
    ```

=== "Custom Cache Directory"

    ```python
    from tree_sitter_language_pack import configure, get_parser

    # Set custom cache before the first parse
    configure(cache_dir="/opt/ts-pack-cache")

    # Now all future downloads use this cache
    parser = get_parser("python")
    ```

=== "Language Groups"

    ```python
    from tree_sitter_language_pack import init

    # Download by language group
    init(groups=["web"])       # JavaScript, TypeScript, HTML, CSS
    init(groups=["systems"])   # C, C++, Rust, Go
    init(groups=["data"])      # Python, R, SQL, JSON

    # Combine languages and groups
    init(
        languages=["python"],
        groups=["web", "systems"]
    )
    ```

=== "Loading from File"

    ```python
    from tree_sitter_language_pack import PackConfig, init
    from pathlib import Path

    # Load from language-pack.toml
    config = PackConfig.from_toml_file(Path("language-pack.toml"))

    if config.languages:
        init(languages=config.languages)
    if config.groups:
        init(groups=config.groups)
    ```

=== "Discovering Configuration"

    ```python
    from tree_sitter_language_pack import PackConfig, init

    # Search for language-pack.toml in current dir and parents
    config = PackConfig.discover()

    if config:
        print(f"Found config with languages: {config.languages}")
        if config.languages:
            init(languages=config.languages)
    else:
        print("No configuration found, using defaults")
    ```

### Node.js/TypeScript

=== "Basic Usage"

    ```typescript
    import { init, getParser } from "@kreuzberg/tree-sitter-language-pack";

    // Pre-download specific languages
    await init({ languages: ["python", "javascript", "rust"] });

    // Now use the library
    const parser = await getParser("python");
    const tree = parser.parse("x = 1");
    ```

=== "Custom Cache Directory"

    ```typescript
    import { configure, getParser } from "@kreuzberg/tree-sitter-language-pack";

    // Set custom cache before the first parse
    await configure({ cacheDir: "/opt/ts-pack-cache" });

    // Now all future downloads use this cache
    const parser = await getParser("python");
    ```

=== "Language Groups"

    ```typescript
    import { init } from "@kreuzberg/tree-sitter-language-pack";

    // Download by language group
    await init({ groups: ["web"] });         // JS, TS, HTML, CSS
    await init({ groups: ["systems"] });     // C, C++, Rust, Go
    await init({ groups: ["data"] });        // Python, R, SQL

    // Combine languages and groups
    await init({
      languages: ["python"],
      groups: ["web", "systems"]
    });
    ```

### Rust

=== "Using PackConfig"

    ```rust
    use ts_pack_core::PackConfig;
    use std::path::Path;

    // Create configuration programmatically
    let config = PackConfig {
        cache_dir: Some(Path::new("/opt/cache").to_path_buf()),
        languages: Some(vec![
            "python".to_string(),
            "rust".to_string(),
            "typescript".to_string(),
        ]),
        groups: None,
    };

    // Load from file
    let config = PackConfig::from_toml_file(Path::new("language-pack.toml"))?;

    // Discover configuration
    if let Some(config) = PackConfig::discover() {
        println!("Found languages: {:?}", config.languages);
    }
    ```

## Environment Variables

Configure behavior via environment variables (useful for CI/CD):

=== "Python"

    ```bash
    # Override cache directory
    export TSLP_CACHE_DIR="/tmp/ts-pack-cache"

    # Specify config file location
    export TSLP_CONFIG="/path/to/language-pack.toml"

    # Enable verbose output
    export TSLP_VERBOSE=1

    # Disable color output
    export TSLP_NO_COLOR=1

    python -c "from tree_sitter_language_pack import get_parser; parser = get_parser('python')"
    ```

=== "CLI"

    ```bash
    # Override cache directory
    ts-pack parse main.py --cache-dir /tmp/ts-pack-cache

    # Specify config file
    ts-pack parse main.py --config /path/to/language-pack.toml

    # Verbose output
    ts-pack parse main.py --verbose

    # Disable colors
    ts-pack parse main.py --no-color
    ```

**Environment variables:**

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `TSLP_CACHE_DIR` | string | `~/.cache/tree-sitter-language-pack/v{version}/libs/` | Cache directory |
| `TSLP_CONFIG` | string | Discovered | Path to config file |
| `TSLP_VERBOSE` | flag | off | Enable verbose output |
| `TSLP_NO_COLOR` | flag | off | Disable ANSI colors |

## CLI Configuration Commands

### `ts-pack cache-dir`

Print the effective cache directory:

```bash
# Show current cache location
ts-pack cache-dir
# /home/user/.cache/tree-sitter-language-pack/v1.0.0/libs/

# Use in scripts
CACHE=$(ts-pack cache-dir)
du -sh "$CACHE"  # Show cache size
```text

### `ts-pack init`

Initialize a configuration file:

```bash
# Interactive setup
ts-pack init
# Creates language-pack.toml with defaults

# Pre-configured languages
ts-pack init --languages python,javascript,typescript,rust

# With cache directory
ts-pack init --cache-dir ./local-cache --languages python
```text

### `ts-pack download`

Download languages specified in `language-pack.toml`:

```bash
# Download all languages from config
ts-pack download

# Download specific languages
ts-pack download python rust javascript

# Download all available languages
ts-pack download --all

# Force re-download (useful after upgrading)
ts-pack download --force

# Download by group
ts-pack download --groups web,systems
```text

### `ts-pack status`

Show download status of configured languages:

```bash
ts-pack status
# Language         Status
# ─────────────────────────────────
# python           ✓ cached
# javascript       ✓ cached
# typescript       ✓ cached
# rust             ✗ not downloaded
```text

### `ts-pack list`

List available languages:

```bash
# List all available languages
ts-pack list

# List only cached languages
ts-pack list --downloaded

# Output as JSON
ts-pack list --format json | jq '.[].name'

# Filter by name
ts-pack list --filter python
```text

## CI/CD Integration

### GitHub Actions

=== "Basic CI Setup"

    ```yaml
    name: CI

    on: [push, pull_request]

    jobs:
      analyze:
        runs-on: ubuntu-latest
        steps:
          - uses: actions/checkout@v4

          - name: Install ts-pack
            run: |
              brew install kreuzberg-dev/tap/ts-pack

          - name: Download languages
            run: ts-pack download

          - name: Analyze code
            run: |
              ts-pack process src/ --structure --format json > analysis.json
              jq '.structure | length' analysis.json
    ```

=== "With Cache"

    ```yaml
    name: CI

    on: [push, pull_request]

    jobs:
      analyze:
        runs-on: ubuntu-latest
        steps:
          - uses: actions/checkout@v4

          - name: Install ts-pack
            run: brew install kreuzberg-dev/tap/ts-pack

          - name: Cache parsers
            uses: actions/cache@v4
            with:
              path: ~/.cache/tree-sitter-language-pack
              key: ${{ runner.os }}-ts-pack-${{ hashFiles('language-pack.toml') }}
              restore-keys: |
                ${{ runner.os }}-ts-pack-

          - name: Download languages
            run: ts-pack download

          - name: Analyze code
            run: ts-pack process src/ --all --format json > analysis.json
    ```

=== "Matrix Testing"

    ```yaml
    name: Test

    on: [push]

    jobs:
      test:
        runs-on: ${{ matrix.os }}
        strategy:
          matrix:
            os: [ubuntu-latest, macos-latest, windows-latest]
            language: [python, javascript, rust, go]
        steps:
          - uses: actions/checkout@v4

          - name: Install ts-pack
            run: cargo install ts-pack

          - name: Download language
            run: ts-pack download ${{ matrix.language }}

          - name: Parse samples
            run: ts-pack parse examples/${{ matrix.language }}/sample.*
    ```

### Docker

=== "Minimal Image"

    ```dockerfile
    FROM python:3.11-slim

    # Install ts-pack from source
    RUN pip install tree-sitter-language-pack

    # Your code
    COPY . /app
    WORKDIR /app

    # Run your analysis
    RUN python analyze.py
    ```

=== "Pre-Downloaded Parsers"

    ```dockerfile
    FROM python:3.11-slim

    # Install ts-pack
    RUN pip install tree-sitter-language-pack

    # Pre-download languages (bakes into image)
    RUN ts-pack download --all

    # Your code
    COPY . /app
    WORKDIR /app

    # Parsing is now offline
    RUN python analyze.py
    ```

=== "With Persistent Cache"

    ```dockerfile
    FROM ubuntu:24.04

    RUN apt-get update && apt-get install -y \
        python3 python3-pip \
        && rm -rf /var/lib/apt/lists/*

    RUN pip install tree-sitter-language-pack

    # Cache will be mounted as a volume
    WORKDIR /app

    ENTRYPOINT ["ts-pack", "process"]
    ```

    Run with:
    ```bash
    docker run --rm \
      -v "$PWD:/app" \
      -v "$HOME/.cache/ts-pack:/root/.cache/tree-sitter-language-pack" \
      ts-pack-image src/main.py --structure
    ```

## Troubleshooting

### Issue: Parser downloads failing

=== "Diagnosis"

    ```bash
    # Check cache location
    ts-pack cache-dir

    # Try downloading verbosely
    ts-pack download python --verbose

    # Check network access
    curl -I https://releases.kreuzberg.dev/tree-sitter-language-pack/manifest.json
    ```

=== "Solutions"

    - **Offline mode**: Pre-download languages on a machine with network access, copy cache to offline machine
    - **Custom mirror**: Set `TSLP_CACHE_DIR` to a pre-populated cache
    - **Docker**: Use a Docker image with languages pre-baked

### Issue: Stale cache

```bash
# Clear all cached parsers
ts-pack clean --all

# Or remove specific language
ts-pack clean python

# Or manually
rm -rf ~/.cache/tree-sitter-language-pack
```text

### Issue: Running out of disk space

```bash
# Check cache size
du -sh ~/.cache/tree-sitter-language-pack

# Move cache to larger drive
mkdir -p /mnt/large-drive/ts-pack-cache
ts-pack download --cache-dir /mnt/large-drive/ts-pack-cache python

# Symlink for convenience
ln -s /mnt/large-drive/ts-pack-cache ~/.cache/tree-sitter-language-pack
```text

## Summary: Configuration Methods

| Method | Pros | Cons | Best For |
|--------|------|------|----------|
| **language-pack.toml** | Version controlled, team-wide | Requires file creation | Projects, teams, monorepos |
| **Environment variables** | No file needed, simple | Not version controlled | CI/CD, containers, one-off runs |
| **CLI flags** | Immediate, no setup | Verbose, hard to repeat | Quick tests, scripts |
| **Programmatic API** | Full control, flexible | Code required | Dynamic scenarios, libraries |

## Next Steps

- **Pre-download for offline use**: Run `ts-pack download --all` and commit the `language-pack.toml`
- **Set up CI caching**: Use GitHub Actions cache to speed up CI/CD builds
- **Parse code**: Use [Parsing](parsing.md) to get syntax trees
- **Extract intelligence**: Use [Code Intelligence](intelligence.md) to analyze code
