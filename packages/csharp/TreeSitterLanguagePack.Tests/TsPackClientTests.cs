using TreeSitterLanguagePack;
using Xunit;

namespace TreeSitterLanguagePack.Tests;

public class TsPackClientTests
{
    [Fact]
    public void AvailableLanguages_ReturnsNonEmpty()
    {
        var languages = TsPackClient.AvailableLanguages();
        Assert.NotEmpty(languages);
    }

    [Fact]
    public void LanguageCount_ReturnsPositive()
    {
        var count = TsPackClient.LanguageCount();
        Assert.True(count > 0, "should have at least one language");
    }

    [Fact]
    public void HasLanguage_ReturnsTrueForPython()
    {
        Assert.True(TsPackClient.HasLanguage("python"));
    }

    [Fact]
    public void HasLanguage_ReturnsFalseForUnknown()
    {
        Assert.False(TsPackClient.HasLanguage("nonexistent_language_xyz_42"));
    }

    [Fact]
    public void Parse_Python_ReturnsTree()
    {
        using var tree = TsPackClient.Parse("python", "def hello(): pass");
        Assert.Equal("module", tree.RootNodeType());
        Assert.True(tree.RootChildCount() >= 1);
        Assert.True(tree.ContainsNodeType("function_definition"));
        Assert.False(tree.ContainsNodeType("nonexistent_node_xyz"));
        Assert.False(tree.HasErrorNodes());
    }

    [Fact]
    public void Parse_BrokenCode_HasErrors()
    {
        using var tree = TsPackClient.Parse("python", "def (broken syntax @@@ !!!");
        Assert.True(tree.HasErrorNodes());
        Assert.True(tree.ErrorCount() > 0);
    }

    [Fact]
    public void Parse_InvalidLanguage_Throws()
    {
        var ex = Assert.Throws<TsPackException>(() =>
            TsPackClient.Parse("nonexistent_language_xyz_42", "hello"));
        Assert.NotEmpty(ex.Message);
    }

    [Fact]
    public void Parse_ToSexp_ReturnsNonNull()
    {
        using var tree = TsPackClient.Parse("python", "x = 1");
        var sexp = tree.ToSexp();
        Assert.NotNull(sexp);
        Assert.Contains("module", sexp);
    }

    [Fact]
    public void Process_Python_ReturnsResult()
    {
        var result = TsPackClient.Process("def hello(): pass", new ProcessConfig
        {
            Language = "python"
        });

        Assert.Equal("python", result.Language);
        Assert.True(result.Metrics.TotalLines >= 1);
        Assert.True(result.Metrics.TotalBytes > 0);
        Assert.NotEmpty(result.Structure);
    }

    [Fact]
    public void Process_WithChunking_ReturnsChunks()
    {
        var source = string.Join("\n", Enumerable.Range(0, 50).Select(i => $"x_{i} = {i}"));
        var result = TsPackClient.Process(source, new ProcessConfig
        {
            Language = "python",
            ChunkMaxSize = 100
        });

        Assert.NotEmpty(result.Chunks);
    }

    [Fact]
    public void Process_InvalidLanguage_Throws()
    {
        var ex = Assert.Throws<TsPackException>(() =>
            TsPackClient.Process("hello", new ProcessConfig
            {
                Language = "nonexistent_xyz_42"
            }));
        Assert.NotEmpty(ex.Message);
    }
}
