```java title="Java"
import io.github.treesitter.languagepack.TsPackRegistry;

class Main {
    public static void main(String[] args) {
        var result = TsPackRegistry.process(
            "import java.util.List;\npublic class App { public void run() {} }",
            "{\"language\": \"java\", \"structure\": true, \"imports\": true}"
        );
        System.out.println(result);
    }
}
```
