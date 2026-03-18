```elixir title="Elixir"
TreeSitterLanguagePack.init(~s({"languages": ["elixir", "erlang"]}))
TreeSitterLanguagePack.download(["python", "rust"])

{:ok, langs} = TreeSitterLanguagePack.downloaded_languages()
IO.inspect(langs)
```
