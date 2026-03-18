```ruby title="Ruby"
require "tree_sitter_language_pack"

TreeSitterLanguagePack.init('{"languages": ["ruby"]}')

tree = TreeSitterLanguagePack.parse_string("ruby", "def hello; puts 'world'; end")
puts "Root: #{tree.root_node_type}"
```
