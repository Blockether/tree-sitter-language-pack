```python title="Python"
from tree_sitter_language_pack import get_parser, process, ProcessConfig

# Parsers download automatically on first use
parser = get_parser("python")
tree = parser.parse(b"def hello():\n    print('world')\n")
print(tree.root_node.sexp())

# Extract code intelligence
config = ProcessConfig(language="python", structure=True, imports=True)
result = process("def hello():\n    print('world')\n", config)
print(f"Functions: {len(result['structure'])}")
```
