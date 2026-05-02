```elixir title="Elixir"
TreeSitterLanguagePack.init(~s({"languages": ["elixir"]}))

{:ok, language_count} = TreeSitterLanguagePack.language_count()
IO.puts("Languages: #{language_count}")
```
