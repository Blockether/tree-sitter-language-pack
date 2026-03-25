---
description: "Elixir API reference for tree-sitter-language-pack"
---

# Elixir API Reference

## Installation

Add to `mix.exs`:

```elixir
def deps do
  [
    {:tree_sitter_language_pack, "~> 1.0"}
  ]
end
```text

Then run:

```bash
mix deps.get
```text

## Quick Start

```elixir
# Pre-download languages
TreeSitterLanguagePack.init(languages: ["python", "rust"])

# Get a language
{:ok, language} = TreeSitterLanguagePack.get_language("python")

# Get a pre-configured parser
{:ok, parser} = TreeSitterLanguagePack.get_parser("python")
tree = TreeSitter.Parser.parse(parser, "def hello(): pass")
IO.puts(TreeSitter.Tree.sexp(tree))

# Extract code intelligence
config = TreeSitterLanguagePack.ProcessConfig.new("python")
  |> TreeSitterLanguagePack.ProcessConfig.all()

{:ok, result} = TreeSitterLanguagePack.process("def hello(): pass", config)
IO.inspect(result["structure"])
```text

## Download Management

### `TreeSitterLanguagePack.init(options \\ [])`

Initialize the language pack with optional pre-downloads.

**Parameters:**

- `options` (keyword list):
    - `languages` (list[String]): Languages to download
    - `groups` (list[String]): Language groups to download
    - `cache_dir` (String): Custom cache directory

**Returns:** {:ok, nil} | {:error, reason}

**Example:**

```elixir
# Pre-download specific languages
TreeSitterLanguagePack.init(languages: ["python", "javascript", "rust"])

# Or download language groups
TreeSitterLanguagePack.init(groups: ["web", "data"])

# With custom cache directory
TreeSitterLanguagePack.init(
  languages: ["python"],
  cache_dir: "/opt/ts-pack"
)
```text

### `TreeSitterLanguagePack.configure(options \\ [])`

Apply configuration without downloading.

Use to set custom cache directory before first `get_language` call.

**Parameters:**

- `options` (keyword list):
    - `cache_dir` (String): Custom cache directory

**Returns:** {:ok, nil} | {:error, reason}

**Example:**

```elixir
TreeSitterLanguagePack.configure(cache_dir: "/data/ts-pack")

{:ok, language} = TreeSitterLanguagePack.get_language("python")
```text

### `TreeSitterLanguagePack.download(names)`

Download specific languages to cache.

**Parameters:**

- `names` (list[String]): Language names to download

**Returns:** {:ok, count} | {:error, reason} where count is Integer

**Example:**

```elixir
case TreeSitterLanguagePack.download(["python", "rust", "typescript"]) do
  {:ok, count} -> IO.puts("Downloaded #{count} new languages")
  {:error, reason} -> IO.puts("Error: #{reason}")
end
```text

### `TreeSitterLanguagePack.download_all()`

Download all available languages (248).

**Returns:** {:ok, count} | {:error, reason}

**Example:**

```elixir
{:ok, count} = TreeSitterLanguagePack.download_all()
IO.puts("Downloaded #{count} languages total")
```text

### `TreeSitterLanguagePack.manifest_languages()`

Get all available languages from remote manifest.

**Returns:** {:ok, languages} | {:error, reason}

**Example:**

```elixir
case TreeSitterLanguagePack.manifest_languages() do
  {:ok, languages} ->
    IO.puts("Available: #{length(languages)} languages")
    IO.inspect(Enum.sort(languages))

  {:error, reason} ->
    IO.puts("Error: #{reason}")
end
```text

### `TreeSitterLanguagePack.downloaded_languages()`

Get languages already cached locally.

Does not perform network requests.

**Returns:** list[String]

**Example:**

```elixir
cached = TreeSitterLanguagePack.downloaded_languages()
IO.inspect(cached)
```text

### `TreeSitterLanguagePack.clean_cache()`

Delete all cached parser shared libraries.

**Returns:** :ok | {:error, reason}

**Example:**

```elixir
TreeSitterLanguagePack.clean_cache()
IO.puts("Cache cleaned")
```text

### `TreeSitterLanguagePack.cache_dir()`

Get the current cache directory path.

**Returns:** String

**Example:**

```elixir
dir = TreeSitterLanguagePack.cache_dir()
IO.puts("Cache at: #{dir}")
```text

## Language Discovery

### `TreeSitterLanguagePack.get_language(name)`

Get a tree-sitter Language by name.

Resolves aliases (e.g., `"shell"` → `"bash"`). Auto-downloads if needed.

**Parameters:**

- `name` (String): Language name or alias

**Returns:** {:ok, language} | {:error, reason}

**Example:**

```elixir
case TreeSitterLanguagePack.get_language("python") do
  {:ok, language} ->
    {:ok, parser} = TreeSitter.Parser.new()
    TreeSitter.Parser.set_language(parser, language)
    tree = TreeSitter.Parser.parse(parser, "x = 1")
    IO.puts(tree.root_node.type)

  {:error, reason} ->
    IO.puts("Error: #{reason}")
end
```text

### `TreeSitterLanguagePack.get_parser(name)`

Get a pre-configured Parser for a language.

**Parameters:**

- `name` (String): Language name or alias

**Returns:** {:ok, parser} | {:error, reason}

**Example:**

```elixir
case TreeSitterLanguagePack.get_parser("rust") do
  {:ok, parser} ->
    tree = TreeSitter.Parser.parse(parser, "fn main() {}")
    IO.puts(tree.root_node.has_error?)

  {:error, reason} ->
    IO.puts("Error: #{reason}")
end
```text

### `TreeSitterLanguagePack.available_languages()`

List all available language names.

**Returns:** list[String]

**Example:**

```elixir
langs = TreeSitterLanguagePack.available_languages()
Enum.each(langs, &IO.puts/1)
```text

### `TreeSitterLanguagePack.has_language?(name)`

Check if a language is available.

**Parameters:**

- `name` (String): Language name or alias

**Returns:** boolean

**Example:**

```elixir
if TreeSitterLanguagePack.has_language?("python") do
  IO.puts("Python available")
end

unless TreeSitterLanguagePack.has_language?("shell") do
  raise "Shell not available"
end
```text

### `TreeSitterLanguagePack.language_count()`

Get total number of available languages.

**Returns:** integer

**Example:**

```elixir
count = TreeSitterLanguagePack.language_count()
IO.puts("#{count} languages available")
```text

## Parsing

### `TreeSitterLanguagePack.parse_string(source, language)`

Parse source code into a syntax tree.

**Parameters:**

- `source` (String | binary): Source code
- `language` (String): Language name

**Returns:** {:ok, tree} | {:error, reason}

**Example:**

```elixir
case TreeSitterLanguagePack.parse_string("def foo(): pass", "python") do
  {:ok, tree} ->
    IO.puts(TreeSitter.Tree.sexp(tree))

  {:error, reason} ->
    IO.puts("Error: #{reason}")
end
```text

## Code Intelligence

### `TreeSitterLanguagePack.process(source, config)`

Extract code intelligence from source code.

**Parameters:**

- `source` (String): Source code
- `config` (ProcessConfig): Configuration

**Returns:** {:ok, result} | {:error, reason}

**Example:**

```elixir
config = TreeSitterLanguagePack.ProcessConfig.new("python")
  |> TreeSitterLanguagePack.ProcessConfig.all()

case TreeSitterLanguagePack.process("def hello(): pass", config) do
  {:ok, result} ->
    IO.puts("Functions: #{length(result["structure"])}")
    IO.puts("Lines: #{result["metrics"]["total_lines"]}")

  {:error, reason} ->
    IO.puts("Error: #{reason}")
end
```text

## Types

### `TreeSitterLanguagePack.ProcessConfig`

Configuration for code intelligence analysis.

Use with pipe operators for fluent API.

**Constructor:**

```elixir
config = TreeSitterLanguagePack.ProcessConfig.new("python")
```text

**Methods:**

#### `structure() :: ProcessConfig`

Enable structure extraction.

#### `import_exports() :: ProcessConfig`

Enable imports/exports extraction.

#### `comments() :: ProcessConfig`

Enable comment extraction.

#### `docstrings() :: ProcessConfig`

Enable docstring extraction.

#### `symbols() :: ProcessConfig`

Enable symbol extraction.

#### `metrics() :: ProcessConfig`

Enable metric extraction.

#### `diagnostics() :: ProcessConfig`

Enable diagnostic extraction.

#### `with_chunks(max_size :: integer, overlap :: integer) :: ProcessConfig`

Configure code chunking.

#### `all() :: ProcessConfig`

Enable all features.

**Example:**

```elixir
config = TreeSitterLanguagePack.ProcessConfig.new("python")
  |> TreeSitterLanguagePack.ProcessConfig.structure()
  |> TreeSitterLanguagePack.ProcessConfig.import_exports()
  |> TreeSitterLanguagePack.ProcessConfig.comments()
  |> TreeSitterLanguagePack.ProcessConfig.with_chunks(2000, 400)
```text

### Result Map

Result from `process` function.

**Keys:**

- `"language"` (String) - Language name
- `"metrics"` (Map) - File metrics
    - `"total_lines"` (integer)
    - `"code_lines"` (integer)
    - `"comment_lines"` (integer)
    - `"blank_lines"` (integer)
- `"structure"` (list) - Code structure items
    - Each item has `"kind"`, `"name"`, `"line"`, `"column"`, etc.
- `"imports"` (list) - Import statements
- `"exports"` (list) - Export statements
- `"comments"` (list) - Comments
- `"docstrings"` (list) - Docstrings
- `"symbols"` (list) - Symbols
- `"diagnostics"` (list) - Diagnostics
- `"chunks"` (list) - Code chunks
- `"parse_errors"` (integer) - Number of parse errors

**Example:**

```elixir
{:ok, result} = TreeSitterLanguagePack.process(source, config)

language = result["language"]
structure = result["structure"]

Enum.each(structure, fn item ->
  IO.puts("#{item["kind"]}: #{item["name"]}")
end)
```text

## Error Handling

Use pattern matching with case/with for error handling:

```elixir
case TreeSitterLanguagePack.get_language("python") do
  {:ok, language} ->
    IO.puts("Got Python")

  {:error, :language_not_found} ->
    IO.puts("Language not available")

  {:error, :download_failed} ->
    IO.puts("Download failed")

  {:error, reason} ->
    IO.puts("Error: #{inspect(reason)}")
end
```text

Or with `with` for multi-step operations:

```elixir
with {:ok, parser} <- TreeSitterLanguagePack.get_parser("python"),
     {:ok, tree} <- TreeSitter.Parser.parse(parser, source),
     {:ok, result} <- TreeSitterLanguagePack.process(source, config) do
  IO.inspect(result)
else
  {:error, reason} -> IO.puts("Error: #{reason}")
end
```text

## Usage Patterns

### Pre-download in Application Start

```elixir
# lib/my_app/application.ex
defmodule MyApp.Application do
  use Application

  @impl true
  def start(_type, _args) do
    TreeSitterLanguagePack.init(
      languages: ["python", "rust", "typescript", "javascript"]
    )

    children = [
      # ... other children
    ]

    opts = [strategy: :one_for_one, name: MyApp.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
```text

### Custom Cache Directory

```elixir
# lib/my_app/config.ex
TreeSitterLanguagePack.configure(cache_dir: "/data/ts-pack-cache")
```text

### Process Batch Files

```elixir
defmodule MyApp.Analyzer do
  @spec analyze_files(String.t(), String.t()) :: :ok
  def analyze_files(dir, language) do
    config = TreeSitterLanguagePack.ProcessConfig.new(language)
      |> TreeSitterLanguagePack.ProcessConfig.all()

    dir
    |> File.ls!()
    |> Enum.filter(&String.ends_with?(&1, ".py"))
    |> Enum.each(fn file ->
      path = Path.join(dir, file)
      source = File.read!(path)

      case TreeSitterLanguagePack.process(source, config) do
        {:ok, result} ->
          IO.puts("#{file}: #{length(result["structure"])} items")

        {:error, reason} ->
          IO.puts("Error: #{reason}")
      end
    end)

    :ok
  end
end
```text

### Concurrent Processing with Tasks

```elixir
defmodule MyApp.ConcurrentAnalyzer do
  @spec analyze_files_async(list(String.t()), String.t()) :: list(map)
  def analyze_files_async(files, language) do
    config = TreeSitterLanguagePack.ProcessConfig.new(language)
      |> TreeSitterLanguagePack.ProcessConfig.all()

    files
    |> Task.async_stream(fn file ->
      case File.read(file) do
        {:ok, source} ->
          case TreeSitterLanguagePack.process(source, config) do
            {:ok, result} -> {:ok, result}
            {:error, reason} -> {:error, {file, reason}}
          end

        {:error, reason} ->
          {:error, {file, reason}}
      end
    end)
    |> Enum.map(&elem(&1, 1))
  end
end
```text

### Parse and Walk Tree

```elixir
defmodule MyApp.TreeWalker do
  @spec walk_tree(any(), non_neg_integer()) :: :ok
  def walk_tree(node, depth \\ 0) do
    indent = String.duplicate("  ", depth)
    type = TreeSitter.Node.type(node)
    IO.puts("#{indent}#{type}")

    node
    |> TreeSitter.Node.children()
    |> Enum.each(&walk_tree(&1, depth + 1))
  end
end
```text

### Extract Specific Patterns

```elixir
defmodule MyApp.FunctionFinder do
  @spec find_functions(String.t(), String.t()) :: list(map)
  def find_functions(source, language) do
    config = TreeSitterLanguagePack.ProcessConfig.new(language)
      |> TreeSitterLanguagePack.ProcessConfig.structure()

    case TreeSitterLanguagePack.process(source, config) do
      {:ok, result} ->
        result["structure"]
        |> Enum.filter(&(&1["kind"] == "function"))

      {:error, _reason} ->
        []
    end
  end
end
```text

### ExUnit Tests

```elixir
defmodule MyApp.AnalyzerTest do
  use ExUnit.Case

  setup do
    TreeSitterLanguagePack.init(languages: ["python"])
    :ok
  end

  test "analyzes python code" do
    source = """
    def hello():
        pass
    """

    config = TreeSitterLanguagePack.ProcessConfig.new("python")
      |> TreeSitterLanguagePack.ProcessConfig.all()

    {:ok, result} = TreeSitterLanguagePack.process(source, config)

    assert result["language"] == "python"
    assert length(result["structure"]) > 0
  end
end
```text

## Type Specifications

Define specs for type safety with Dialyzer:

```elixir
@spec analyze(String.t(), String.t()) :: {:ok, map()} | {:error, String.t()}
def analyze(source, language) do
  config = TreeSitterLanguagePack.ProcessConfig.new(language)
    |> TreeSitterLanguagePack.ProcessConfig.all()

  TreeSitterLanguagePack.process(source, config)
end
```
