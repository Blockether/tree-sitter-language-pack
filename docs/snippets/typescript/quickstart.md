```typescript title="Node.js"
import { getParser, process } from "@kreuzberg/tree-sitter-language-pack";

// Parsers download automatically on first use
const parser = await getParser("javascript");
const tree = parser.parse("function hello() { console.log('world'); }");
console.log(tree.rootNode.toString());

// Extract code intelligence
const result = await process(
  "function hello() { console.log('world'); }",
  { language: "javascript", structure: true, imports: true }
);
console.log(`Functions: ${result.structure.length}`);
```
