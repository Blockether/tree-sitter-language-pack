using TreeSitterLanguagePack;

var languages = TsPackClient.AvailableLanguages();
Console.WriteLine($"Available languages: {languages.Length}");

if (languages.Length == 0)
{
    throw new Exception("no languages available");
}

if (!TsPackClient.HasLanguage("python"))
{
    throw new Exception("python not found");
}

using var tree = TsPackClient.Parse("python", "def hello(): pass");
Console.WriteLine($"Root node type: {tree.RootNodeType()}");
Console.WriteLine($"Root child count: {tree.RootChildCount()}");

var result = TsPackClient.Process("def hello(): pass", new ProcessConfig
{
    Language = "python"
});
Console.WriteLine($"Structure items: {result.Structure.Count}");
Console.WriteLine($"Metrics - lines: {result.Metrics.TotalLines}, bytes: {result.Metrics.TotalBytes}");

Console.WriteLine("C# smoke test passed");
