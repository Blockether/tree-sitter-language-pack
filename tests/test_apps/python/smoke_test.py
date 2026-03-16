"""Smoke tests for tree-sitter-language-pack published package."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
import tree_sitter_language_pack as tslp

FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"


def load_fixtures(name: str) -> list[dict]:
    return json.loads((FIXTURES_DIR / name).read_text())


class TestBasic:
    """Validate basic language discovery API."""

    @pytest.fixture(autouse=True)
    def _load_fixtures(self) -> None:
        self.fixtures = load_fixtures("basic.json")

    def test_package_imports(self) -> None:
        assert hasattr(tslp, "available_languages")
        assert hasattr(tslp, "has_language")
        assert hasattr(tslp, "language_count")

    @pytest.mark.parametrize(
        "fixture",
        load_fixtures("basic.json"),
        ids=lambda f: f["name"],
    )
    def test_basic_fixture(self, fixture: dict) -> None:
        match fixture["test"]:
            case "language_count":
                count = tslp.language_count()
                assert count >= fixture["expected_min"], (
                    f"language_count {count} < expected min {fixture['expected_min']}"
                )
            case "has_language":
                result = tslp.has_language(fixture["language"])
                assert result == fixture["expected"], (
                    f"has_language({fixture['language']!r}) = {result}, expected {fixture['expected']}"
                )
            case "available_languages":
                langs = tslp.available_languages()
                for lang in fixture["expected_contains"]:
                    assert lang in langs, f"available_languages missing {lang!r}"
            case other:
                pytest.fail(f"Unknown test type: {other}")


class TestErrorHandling:
    """Validate error handling for invalid inputs."""

    def test_invalid_language_process(self) -> None:
        with pytest.raises(Exception):
            tslp.process("some code", {"language": "nonexistent_xyz_123"})

    def test_has_language_returns_false_for_invalid(self) -> None:
        assert tslp.has_language("nonexistent_xyz_123") is False
