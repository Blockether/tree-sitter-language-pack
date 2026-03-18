```java title="Java"
import io.github.treesitter.languagepack.TsPackRegistry;

class Main {
    public static void main(String[] args) {
        TsPackRegistry.init("{\"languages\": [\"java\"]}");

        var tree = TsPackRegistry.parseString("java", "class Foo { void bar() {} }");
        System.out.println("Root: " + tree.rootNodeType());
    }
}
```
