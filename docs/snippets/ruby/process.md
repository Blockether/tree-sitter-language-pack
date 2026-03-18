```ruby title="Ruby"
require "tree_sitter_language_pack"

result = TreeSitterLanguagePack.process(
  "require 'json'\ndef parse(data)\n  JSON.parse(data)\nend",
  '{"language": "ruby", "structure": true, "imports": true}'
)
puts result
```
