#!/usr/bin/env bash
# Test all grammars — parses a representative snippet for each language
# and verifies the tree-sitter output is valid (non-empty, no crashes).
#
# Usage:
#   ./test_all_grammars.sh                    # uses ts-pack from PATH
#   TS_PACK_BIN=./target/release/ts-pack ./test_all_grammars.sh
set -euo pipefail

BINARY="${TS_PACK_BIN:-ts-pack}"
PASS=0
FAIL=0
SKIP=0

test_parse() {
  local lang="$1" source="$2"
  local out
  if out=$(echo "$source" | "$BINARY" parse - --language "$lang" 2>&1); then
    if [ -n "$out" ]; then
      PASS=$((PASS + 1))
    else
      echo "  FAIL: $lang (empty output)"
      FAIL=$((FAIL + 1))
    fi
  else
    echo "  FAIL: $lang (exit code $?)"
    FAIL=$((FAIL + 1))
  fi
}

echo "=== Grammar Test Suite ==="
echo "Binary: $BINARY"
echo ""

# Core languages
test_parse python "def hello(): pass"
test_parse javascript "function test() { return 1; }"
test_parse typescript "const x: number = 1;"
test_parse tsx "const App = () => <div>hello</div>;"
test_parse rust "fn main() { println!(\"hello\"); }"
test_parse go "package main; func main() {}"
test_parse java "class Foo { void bar() {} }"
test_parse c "int main() { return 0; }"
test_parse cpp "int main() { return 0; }"
#test_parse csharp — no tree-sitter grammar
test_parse ruby "def hello; end"
test_parse php "<?php function hello() {} ?>"
test_parse swift "func hello() {}"
test_parse kotlin "fun main() {}"
test_parse scala "object Main { def main(args: Array[String]) = {} }"
test_parse dart "void main() {}"
test_parse elixir "defmodule M do def hello, do: :ok end"
test_parse erlang "-module(test). -export([hello/0]). hello() -> ok."
test_parse haskell "main = putStrLn \"hello\""
test_parse lua "function hello() end"
test_parse r "hello <- function() { 1 }"
test_parse julia "function hello() end"
test_parse perl "sub hello { 1 }"
test_parse bash "echo hello"
test_parse fish "function hello; echo hi; end"
test_parse powershell "function Hello { Write-Host hi }"
test_parse zig "pub fn main() !void {}"
test_parse nim "proc hello() = discard"
test_parse odin "main :: proc() {}"
test_parse d "void main() {}"
test_parse fortran "program hello; end program"
test_parse ocaml "let hello = 1"
test_parse elm "hello = 1"
test_parse clojure "(defn hello [] 1)"
test_parse scheme "(define (hello) 1)"
test_parse racket "(define (hello) 1)"
test_parse commonlisp "(defun hello () 1)"
test_parse fennel "(fn hello [] 1)"
test_parse gleam "pub fn hello() { Nil }"
test_parse purescript "hello = 1"
test_parse fsharp "let hello = 1"

# Web
test_parse html "<html><body>hello</body></html>"
test_parse css "body { color: red; }"
test_parse scss "body { .inner { color: red; } }"
test_parse json '{"key": "value"}'
test_parse xml "<root><child/></root>"
test_parse graphql "query { hello }"
test_parse svelte "<script>let x = 1;</script><p>hello</p>"
test_parse vue "<template><div>hello</div></template>"
test_parse astro "---\nconst x = 1;\n---\n<p>hello</p>"

# Data / Config
test_parse toml '[section]\nkey = "value"'
#test_parse yaml — no tree-sitter grammar
test_parse ini "[section]\nkey=value"
test_parse csv "a,b,c"
test_parse sql "SELECT * FROM users;"
test_parse proto 'syntax = "proto3"; message Foo {}'
test_parse kdl "node 1 2 3"
test_parse ron "(hello: 1)"
test_parse properties "key=value"

# DevOps / Config
test_parse dockerfile "FROM alpine:latest"
test_parse hcl 'resource "null" "x" {}'
test_parse terraform 'resource "null" "x" {}'
test_parse nix "{ pkgs }: pkgs.hello"
test_parse cmake "cmake_minimum_required(VERSION 3.0)"
test_parse make "all:\n\techo hello"
test_parse ninja "rule cc\n  command = gcc"

# Markup / Docs
test_parse markdown "# Hello\n\nWorld"
test_parse latex "\\documentclass{article}"
test_parse rst "Hello\n=====\n\nWorld"
test_parse org "* Hello\n** World"
test_parse bibtex "@article{key, author={me}}"

# Systems
test_parse verilog "module test; endmodule"
test_parse vhdl "entity test is end;"
test_parse asm "mov eax, 1"
test_parse cuda "__global__ void kernel() {}"
test_parse glsl "void main() { gl_FragColor = vec4(1); }"
test_parse hlsl "float4 main() : SV_Target { return 0; }"
test_parse wgsl "fn main() -> vec4f { return vec4f(1); }"

# JVM
test_parse groovy "def hello() { 1 }"

# Misc
test_parse vim "function! Hello()\nendfunction"
#test_parse regex — no tree-sitter grammar
test_parse gitignore "*.log\nnode_modules/"
test_parse gitcommit "fix: hello world"
test_parse comment "// hello world"
test_parse jsdoc "/** @param {string} x */"

echo ""
echo "=== Results: $PASS passed, $FAIL failed, $SKIP skipped ==="
if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
