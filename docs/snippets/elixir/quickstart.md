```elixir title="Elixir"
TreeSitterLanguagePack.init(~s({"languages": ["elixir"]}))

{:ok, tree} = TreeSitterLanguagePack.parse_string("elixir", "defmodule M do end")
{:ok, node_type} = TreeSitterLanguagePack.tree_root_node_type(tree)
IO.puts("Root: #{node_type}")
```
