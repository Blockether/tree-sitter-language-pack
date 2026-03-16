using System.Text.Json;
using TreeSitterLanguagePack;
using Xunit;

namespace TestApp;

public class SmokeTest
{
    private static readonly string FixturesDir = Path.Combine(
        Directory.GetCurrentDirectory(), "..", "..", "..", "..", "fixtures");

    private static List<JsonElement> LoadFixtures(string name)
    {
        var json = File.ReadAllText(Path.Combine(FixturesDir, name));
        return JsonSerializer.Deserialize<List<JsonElement>>(json)!;
    }

    [Fact]
    public void LanguageCountIsPositive()
    {
        var count = LanguagePack.LanguageCount();
        Assert.True(count >= 100, $"language_count {count} < expected min 100");
    }

    [Theory]
    [InlineData("python", true)]
    [InlineData("javascript", true)]
    [InlineData("rust", true)]
    [InlineData("go", true)]
    [InlineData("nonexistent_xyz", false)]
    public void HasLanguage(string language, bool expected)
    {
        var result = LanguagePack.HasLanguage(language);
        Assert.Equal(expected, result);
    }

    [Fact]
    public void AvailableLanguagesContainsExpected()
    {
        var langs = LanguagePack.AvailableLanguages();
        Assert.Contains("python", langs);
        Assert.Contains("javascript", langs);
        Assert.Contains("rust", langs);
        Assert.Contains("go", langs);
    }

    [Fact]
    public void ParsesPythonCode()
    {
        var tree = LanguagePack.ParseString("python", "def hello(): pass\n");
        Assert.NotNull(tree);
        Assert.Equal("module", tree.RootNodeType());
        Assert.True(tree.RootChildCount() >= 1);
        Assert.False(tree.HasErrorNodes());
    }

    [Fact]
    public void ErrorsOnInvalidLanguage()
    {
        Assert.Throws<Exception>(() =>
            LanguagePack.ParseString("nonexistent_xyz_123", "code"));
    }
}
