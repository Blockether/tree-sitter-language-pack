```typescript title="Node.js"
const { process } = require("@kreuzberg/tree-sitter-language-pack");

const result = process(
  "function hello() { console.log('world'); }",
  { language: "javascript", structure: true, imports: true },
);

console.log(`Language: ${result.language}`);
console.log(`Functions: ${result.structure.length}`);
```
