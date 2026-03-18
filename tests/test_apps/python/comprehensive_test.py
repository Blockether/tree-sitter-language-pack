"""Comprehensive fixture-driven tests for tree-sitter-language-pack."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
import tree_sitter_language_pack as tslp
from tree_sitter_language_pack import ProcessConfig

FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"


@pytest.fixture(scope="session", autouse=True)
def _setup_languages() -> None:
    """Ensure required languages are downloaded before running tests."""
    try:
        # Initialize the download system
        tslp.init()

        # Check which languages are already downloaded
        downloaded = set(tslp.downloaded_languages())
        manifest = set(tslp.manifest_languages())

        # Determine which languages are needed for tests
        # (all fixtures use python, javascript, go, rust, c at minimum)
        required = {"python", "javascript", "go", "rust", "c"}
        to_download = required - downloaded

        if to_download:
            # Download missing languages (mark as network test)
            missing = to_download & manifest
            if missing:
                tslp.download(list(missing))
    except Exception:
        # If download fails, tests will still run against available languages
        pass


def load_fixtures(name: str) -> list[dict]:
    return json.loads((FIXTURES_DIR / name).read_text())


def make_config(fixture_config: dict) -> ProcessConfig:
    return ProcessConfig(
        language=fixture_config["language"],
        structure=fixture_config.get("structure", True),
        imports=fixture_config.get("imports", True),
        exports=fixture_config.get("exports", True),
        comments=fixture_config.get("comments", False),
        docstrings=fixture_config.get("docstrings", False),
        symbols=fixture_config.get("symbols", False),
        diagnostics=fixture_config.get("diagnostics", False),
        chunk_max_size=fixture_config.get("chunk_max_size"),
    )


class TestProcess:
    """Validate process() API with various configs."""

    @pytest.mark.parametrize(
        "fixture",
        load_fixtures("process.json"),
        ids=lambda f: f["name"],
    )
    def test_process_fixture(self, fixture: dict) -> None:
        config = make_config(fixture["config"])
        result = tslp.process(fixture["source"], config)
        expected = fixture["expected"]

        if "language" in expected:
            assert result["language"] == expected["language"]
        if "structure_min" in expected:
            assert len(result["structure"]) >= expected["structure_min"], (
                f"structure count {len(result['structure'])} < min {expected['structure_min']}"
            )
        if "imports_min" in expected:
            assert len(result["imports"]) >= expected["imports_min"], (
                f"imports count {len(result['imports'])} < min {expected['imports_min']}"
            )
        if "metrics_total_lines_min" in expected:
            assert (
                result["metrics"]["total_lines"] >= expected["metrics_total_lines_min"]
            )
        if "error_count" in expected:
            assert result["metrics"]["error_count"] == expected["error_count"]


class TestChunking:
    """Validate process() with chunking config."""

    @pytest.mark.parametrize(
        "fixture",
        load_fixtures("chunking.json"),
        ids=lambda f: f["name"],
    )
    def test_chunking_fixture(self, fixture: dict) -> None:
        config = make_config(fixture["config"])
        result = tslp.process(fixture["source"], config)
        expected = fixture["expected"]

        if "chunks_min" in expected:
            assert len(result["chunks"]) >= expected["chunks_min"], (
                f"chunks count {len(result['chunks'])} < min {expected['chunks_min']}"
            )
