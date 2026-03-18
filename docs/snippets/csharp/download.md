```csharp title="C#"
using System;
using System.Collections.Generic;
using TreeSitterLanguagePack;

class Program
{
    static void Main()
    {
        TsPackClient.Init("{\"languages\": [\"csharp\", \"fsharp\"]}");
        TsPackClient.Download(new[] { "python", "rust" });

        var langs = TsPackClient.DownloadedLanguages();
        Console.WriteLine($"Downloaded: {string.Join(", ", langs)}");
    }
}
```
