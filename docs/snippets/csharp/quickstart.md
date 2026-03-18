```csharp title="C#"
using System;
using TreeSitterLanguagePack;

class Program
{
    static void Main()
    {
        TsPackClient.Init("{\"languages\": [\"csharp\"]}");

        var tree = TsPackClient.ParseString("csharp", "class Foo { void Bar() {} }");
        Console.WriteLine($"Root: {tree.RootNodeType()}");
    }
}
```
