package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	tslp "github.com/kreuzberg-dev/tree-sitter-language-pack/packages/go/v1"
)

type BasicFixture struct {
	Name             string   `json:"name"`
	Test             string   `json:"test"`
	Language         string   `json:"language,omitempty"`
	Expected         *bool    `json:"expected,omitempty"`
	ExpectedMin      *int     `json:"expected_min,omitempty"`
	ExpectedContains []string `json:"expected_contains,omitempty"`
}

func loadBasicFixtures(t *testing.T) []BasicFixture {
	t.Helper()
	data, err := os.ReadFile(filepath.Join("..", "fixtures", "basic.json"))
	if err != nil {
		t.Fatalf("Failed to read basic.json: %v", err)
	}
	var fixtures []BasicFixture
	if err := json.Unmarshal(data, &fixtures); err != nil {
		t.Fatalf("Failed to parse basic.json: %v", err)
	}
	return fixtures
}

func TestBasicFixtures(t *testing.T) {
	registry, err := tslp.NewRegistry()
	if err != nil {
		t.Fatalf("Failed to create registry: %v", err)
	}
	defer registry.Free()

	fixtures := loadBasicFixtures(t)
	for _, fixture := range fixtures {
		t.Run(fixture.Name, func(t *testing.T) {
			switch fixture.Test {
			case "language_count":
				count := registry.LanguageCount()
				if count < *fixture.ExpectedMin {
					t.Errorf("language_count %d < expected min %d", count, *fixture.ExpectedMin)
				}
			case "has_language":
				result := registry.HasLanguage(fixture.Language)
				if result != *fixture.Expected {
					t.Errorf("has_language(%q) = %v, expected %v", fixture.Language, result, *fixture.Expected)
				}
			case "available_languages":
				langs := registry.AvailableLanguages()
				langSet := make(map[string]bool)
				for _, l := range langs {
					langSet[l] = true
				}
				for _, expected := range fixture.ExpectedContains {
					if !langSet[expected] {
						t.Errorf("available_languages missing %q", expected)
					}
				}
			default:
				t.Fatalf("Unknown test type: %s", fixture.Test)
			}
		})
	}
}

func TestParseValidation(t *testing.T) {
	registry, err := tslp.NewRegistry()
	if err != nil {
		t.Fatalf("Failed to create registry: %v", err)
	}
	defer registry.Free()

	t.Run("parses_python_code", func(t *testing.T) {
		tree, err := registry.ParseString("python", "def hello(): pass\n")
		if err != nil {
			t.Fatalf("ParseString failed: %v", err)
		}
		defer tree.Free()

		nodeType := tree.RootNodeType()
		if nodeType != "module" {
			t.Errorf("root node type = %q, expected %q", nodeType, "module")
		}
		if tree.RootChildCount() < 1 {
			t.Error("root child count < 1")
		}
		if tree.HasErrorNodes() {
			t.Error("tree has error nodes")
		}
	})

	t.Run("errors_on_invalid_language", func(t *testing.T) {
		_, err := registry.ParseString("nonexistent_xyz_123", "code")
		if err == nil {
			t.Error("expected error for invalid language, got nil")
		}
	})
}
