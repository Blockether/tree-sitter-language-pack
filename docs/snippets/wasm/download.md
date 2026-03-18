```javascript title="WebAssembly"
// Note: WASM bindings have parsers compiled in — no download needed.
import { availableLanguages, hasLanguage } from "@kreuzberg/tree-sitter-language-pack-wasm";

console.log(`Has Python: ${hasLanguage("python")}`);
console.log(`Languages: ${availableLanguages().length}`);
```
