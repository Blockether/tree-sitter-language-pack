```ruby title="Ruby"
require "tree_sitter_language_pack"

TreeSitterLanguagePack.init('{"languages": ["ruby"]}')

puts "Ruby available: #{TreeSitterLanguagePack.has_language("ruby")}"
```
