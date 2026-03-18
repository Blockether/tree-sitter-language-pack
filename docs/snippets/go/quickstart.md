```go title="Go"
package main

import (
    "fmt"
    tslp "github.com/kreuzberg-dev/tree-sitter-language-pack/packages/go/v1"
)

func main() {
    registry, _ := tslp.NewRegistry()
    defer registry.Free()

    tree, _ := registry.ParseString("go", "package main\nfunc hello() {}")
    defer tree.Free()

    fmt.Println("Root:", tree.RootNodeType())
}
```
