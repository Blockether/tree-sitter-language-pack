```elixir title="Elixir"
{:ok, result} = TreeSitterLanguagePack.process(
  "defmodule MyApp do\n  def hello, do: :world\nend",
  ~s({"language": "elixir", "structure": true})
)
IO.inspect(result)
```
