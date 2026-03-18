```go title="Go"
package main

import (
    "fmt"
    tslp "github.com/kreuzberg-dev/tree-sitter-language-pack/packages/go/v1"
)

func main() {
    registry, _ := tslp.NewRegistry()
    defer registry.Free()

    result, _ := registry.Process(
        "package main\nimport \"fmt\"\nfunc hello() { fmt.Println(\"hi\") }",
        `{"language": "go", "structure": true, "imports": true}`,
    )
    fmt.Println(result)
}
```
