defmodule SmokeTest do
  use ExUnit.Case

  @fixtures_dir Path.join([__DIR__, "..", "..", "fixtures"])

  defp load_fixtures(name) do
    @fixtures_dir
    |> Path.join(name)
    |> File.read!()
    |> Jason.decode!()
  end

  describe "basic fixtures" do
    test "language_count is positive" do
      count = TreeSitterLanguagePack.language_count()
      assert count >= 100, "language_count #{count} < expected min 100"
    end

    test "has_language for known languages" do
      assert TreeSitterLanguagePack.has_language("python") == true
      assert TreeSitterLanguagePack.has_language("javascript") == true
      assert TreeSitterLanguagePack.has_language("rust") == true
      assert TreeSitterLanguagePack.has_language("go") == true
    end

    test "has_language returns false for nonexistent" do
      assert TreeSitterLanguagePack.has_language("nonexistent_xyz") == false
    end

    test "available_languages contains expected languages" do
      langs = TreeSitterLanguagePack.available_languages()
      assert is_list(langs)
      assert "python" in langs
      assert "javascript" in langs
      assert "rust" in langs
      assert "go" in langs
    end
  end

  describe "parse validation" do
    test "parses Python code" do
      {:ok, tree} = TreeSitterLanguagePack.parse_string("python", "def hello(): pass\n")
      {:ok, node_type} = TreeSitterLanguagePack.tree_root_node_type(tree)
      assert node_type == "module"

      {:ok, child_count} = TreeSitterLanguagePack.tree_root_child_count(tree)
      assert child_count >= 1

      {:ok, has_errors} = TreeSitterLanguagePack.tree_has_error_nodes(tree)
      assert has_errors == false
    end

    test "errors on invalid language" do
      assert {:error, _reason} = TreeSitterLanguagePack.parse_string("nonexistent_xyz_123", "code")
    end
  end
end
