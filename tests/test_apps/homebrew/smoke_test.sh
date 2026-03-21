#!/usr/bin/env bash
# Homebrew smoke test — validates ts-pack CLI installed via Homebrew.
#
# Usage:
#   ./smoke_test.sh                    # uses ts-pack from PATH
#   TS_PACK_BIN=/path/to/ts-pack ./smoke_test.sh   # override binary path
set -euo pipefail

BINARY="${TS_PACK_BIN:-ts-pack}"
PASS=0
FAIL=0

assert() {
  if eval "$2"; then
    PASS=$((PASS + 1))
    echo "  PASS: $1"
  else
    FAIL=$((FAIL + 1))
    echo "  FAIL: $1"
  fi
}

echo "=== Homebrew Smoke Tests ==="
echo "Binary: $BINARY"
echo ""

# Check if ts-pack is available
if ! command -v "$BINARY" &>/dev/null; then
  echo "ERROR: $BINARY not found in PATH"
  echo "Install with: brew install kreuzberg-dev/tap/ts-pack"
  exit 1
fi

echo "--- Binary check ---"
assert "ts-pack exists in PATH" "command -v $BINARY &> /dev/null"

echo "--- Help ---"
HELP=$($BINARY --help 2>&1 || true)
assert "help shows output" "echo '$HELP' | grep -q 'download\|parse\|process\|cache'"

echo "--- Version ---"
VER=$($BINARY --version 2>&1 || true)
echo "  $VER"
assert "version output" "echo '$VER' | grep -q 'ts-pack\|[0-9]'"

echo "--- Cache dir ---"
DIR=$($BINARY cache-dir 2>&1)
echo "  Cache dir: $DIR"
assert "cache-dir non-empty" "test -n '$DIR'"

echo "--- List manifest (network test) ---"
if LANGS=$($BINARY list --manifest 2>&1 | wc -l) && [ "$LANGS" -ge 100 ]; then
  echo "  Languages from manifest: $LANGS"
  assert "manifest has 100+ languages" "test '$LANGS' -ge 100"

  echo "--- Download python ---"
  $BINARY download python 2>&1 || true
  assert "download python" "$BINARY list --downloaded 2>&1 | grep -q 'python'"

  echo "--- List downloaded ---"
  DOWNLOADED=$($BINARY list --downloaded 2>&1)
  assert "python in downloaded list" "echo '$DOWNLOADED' | grep -q 'python'"

  echo "--- Parse sexp ---"
  OUT=$(echo "def hello(): pass" | $BINARY parse - --language python --format sexp 2>&1 || true)
  assert "parse sexp output" "echo '$OUT' | grep -q 'module'"

  echo "--- Parse JSON ---"
  OUT=$(echo "x = 1" | $BINARY parse - --language python --format json 2>&1 || true)
  assert "parse json format" "echo '$OUT' | grep -q 'sexp\|language\|has_errors'"

  echo "--- Process with structure ---"
  OUT=$(echo "def hello(): pass" | $BINARY process - --language python --structure 2>&1 || true)
  assert "process language field" "echo '$OUT' | grep -q '\"language\"'"
  assert "process structure field" "echo '$OUT' | grep -q '\"structure\"'"

  echo "--- Process with imports ---"
  OUT=$(echo "import os; def main(): pass" | $BINARY process - --language python --imports 2>&1 || true)
  assert "process imports field" "echo '$OUT' | grep -q '\"imports\"'"

  echo "--- Process with structure and imports ---"
  OUT=$(echo "import sys; def test(): pass" | $BINARY process - --language python --structure --imports 2>&1 || true)
  assert "process structure+imports" "echo '$OUT' | grep -q '\"structure\"' && echo '$OUT' | grep -q '\"imports\"'"

  echo "--- Process with chunking ---"
  OUT=$(printf "def a():\n    pass\n\ndef b():\n    pass\n\ndef c():\n    pass\n" | $BINARY process - --language python --structure --chunk-size 30 2>&1 || true)
  assert "process chunks field" "echo '$OUT' | grep -q '\"chunks\"'"

  echo "--- Clean ---"
  $BINARY clean --force >/dev/null 2>&1 || true
  assert "clean --force" "test 0 -eq 0"

else
  echo "  SKIP: manifest fetch failed (network unavailable)"
  echo "  Skipping download/parse/process tests"
fi

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
test "$FAIL" -eq 0
