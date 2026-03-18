```csharp title="C#"
using System;
using TreeSitterLanguagePack;

class Program
{
    static void Main()
    {
        var result = TsPackClient.Process(
            "using System;\nnamespace App { class Program { static void Main() {} } }",
            "{\"language\": \"csharp\", \"structure\": true, \"imports\": true}"
        );
        Console.WriteLine(result);
    }
}
```
