require "json"
require "tree_sitter_language_pack"

RSpec.describe "TreeSitterLanguagePack smoke tests" do
  fixtures_dir = File.expand_path("../fixtures", __dir__)
  basic_fixtures = JSON.parse(File.read(File.join(fixtures_dir, "basic.json")))

  describe "basic fixtures" do
    basic_fixtures.each do |fixture|
      it fixture["name"] do
        case fixture["test"]
        when "language_count"
          count = TreeSitterLanguagePack.language_count
          expect(count).to be >= fixture["expected_min"]
        when "has_language"
          result = TreeSitterLanguagePack.has_language(fixture["language"])
          expect(result).to eq(fixture["expected"])
        when "available_languages"
          langs = TreeSitterLanguagePack.available_languages
          fixture["expected_contains"].each do |lang|
            expect(langs).to include(lang)
          end
        else
          raise "Unknown test type: #{fixture["test"]}"
        end
      end
    end
  end

  describe "parse validation" do
    it "parses Python code" do
      tree = TreeSitterLanguagePack.parse_string("python", "def hello(): pass\n")
      expect(tree.root_node_type).to eq("module")
      expect(tree.root_child_count).to be >= 1
      expect(tree.has_error_nodes).to be false
    end

    it "raises on invalid language" do
      expect {
        TreeSitterLanguagePack.parse_string("nonexistent_xyz_123", "code")
      }.to raise_error
    end
  end
end
