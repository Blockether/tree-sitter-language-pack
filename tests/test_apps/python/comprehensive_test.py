"""Comprehensive fixture-driven tests for tree-sitter-language-pack."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
import tree_sitter_language_pack as tslp

FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"


def load_fixtures(name: str) -> list[dict]:
    return json.loads((FIXTURES_DIR / name).read_text())


class TestProcess:
    """Validate process() API with various configs."""

    @pytest.mark.parametrize(
        "fixture",
        load_fixtures("process.json"),
        ids=lambda f: f["name"],
    )
    def test_process_fixture(self, fixture: dict) -> None:
        result = tslp.process(fixture["source"], fixture["config"])
        expected = fixture["expected"]

        if "language" in expected:
            assert result.language == expected["language"]
        if "structure_min" in expected:
            assert len(result.structure) >= expected["structure_min"], (
                f"structure count {len(result.structure)} < min {expected['structure_min']}"
            )
        if "imports_min" in expected:
            assert len(result.imports) >= expected["imports_min"], (
                f"imports count {len(result.imports)} < min {expected['imports_min']}"
            )
        if "metrics_total_lines_min" in expected:
            assert result.metrics.total_lines >= expected["metrics_total_lines_min"]
        if "error_count" in expected:
            assert result.metrics.error_count == expected["error_count"]


class TestChunking:
    """Validate process() with chunking config."""

    @pytest.mark.parametrize(
        "fixture",
        load_fixtures("chunking.json"),
        ids=lambda f: f["name"],
    )
    def test_chunking_fixture(self, fixture: dict) -> None:
        result = tslp.process(fixture["source"], fixture["config"])
        expected = fixture["expected"]

        if "chunks_min" in expected:
            assert len(result.chunks) >= expected["chunks_min"], (
                f"chunks count {len(result.chunks)} < min {expected['chunks_min']}"
            )
