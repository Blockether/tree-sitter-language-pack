```java title="Java"
import io.github.treesitter.languagepack.TsPackRegistry;
import java.util.List;

class Main {
    public static void main(String[] args) {
        TsPackRegistry.init("{\"languages\": [\"java\", \"kotlin\"]}");
        TsPackRegistry.download(List.of("python", "rust"));

        var langs = TsPackRegistry.downloadedLanguages();
        System.out.println("Downloaded: " + langs);
    }
}
```
