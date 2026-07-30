//! Structure-intelligence matrix for the languages whose `LanguageIntel` modules
//! are shape-driven rather than a plain node-kind table.
//!
//! Each case pins the FULL flattened structure of a sample file — kind, nesting
//! path and visibility — so a grammar bump that silently changes node kinds fails
//! here instead of degrading `--structure` output in the field.

use tree_sitter_language_pack::{ProcessConfig, StructureItem, StructureKind, process};

/// `Kind parent/name [visibility]`, depth-first, parents before children.
fn flatten(items: &[StructureItem], prefix: &str, out: &mut Vec<String>) {
    for item in items {
        let kind = match &item.kind {
            StructureKind::Other(other) => format!("Other:{other}"),
            other => format!("{other:?}"),
        };
        let name = item.name.clone().unwrap_or_default();
        let visibility = item.visibility.as_ref().map(|v| format!(" [{v}]")).unwrap_or_default();
        out.push(format!("{kind} {prefix}{name}{visibility}"));
        flatten(&item.children, &format!("{prefix}{name}/"), out);
    }
}

fn check(language: &'static str, source: &str, expected: &[&str]) {
    let config = ProcessConfig::new(language);
    let result = process(source, &config).expect("grammar available and parse succeeded");
    assert_eq!(
        result.metrics.error_count, 0,
        "{language} sample must parse without ERROR nodes"
    );
    let mut actual = Vec::new();
    flatten(&result.structure, "", &mut actual);
    let expected: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
    assert_eq!(actual, expected, "{language} structure");
}

#[test]
fn haskell_structure() {
    check(
        "haskell",
        r##"module Data.Widget
  ( Widget(..)
  , mkWidget
  ) where

import qualified Data.Map as M
import Data.List (sort)

-- | A widget identifier.
type WidgetId = Int

data Widget = Widget
  { widgetId   :: WidgetId
  , widgetName :: String
  }
  deriving (Show, Eq)

newtype Wrapper a = Wrapper { unwrap :: a }

class Renderable a where
  render :: a -> String
  render _ = "?"

instance Renderable Widget where
  render w = widgetName w

-- | Build a widget.
mkWidget :: WidgetId -> String -> Widget
mkWidget i n = Widget { widgetId = i, widgetName = n }

defaultName :: String
defaultName = "anon"

main :: IO ()
main = putStrLn (render (mkWidget 1 defaultName))
"##,
        &[
            "Module Data.Widget",
            "Type WidgetId",
            "Type Widget",
            "Type Wrapper",
            "Trait Renderable",
            "Function Renderable/render",
            "Impl Renderable Widget",
            "Function Renderable Widget/render",
            "Function mkWidget",
            "Constant defaultName",
            "Constant main",
        ],
    );
}

#[test]
fn ocaml_structure() {
    check(
        "ocaml",
        r##"open Printf
module M = Map.Make (String)

type color = Red | Green | Blue

type point = { x : int; y : int }

exception Bad_input of string

let origin = { x = 0; y = 0 }

let add a b = a + b

let rec fact n = if n <= 1 then 1 else n * fact (n - 1)

module Shape = struct
  type t = { name : string }

  let area _ = 0.0
end

class counter =
  object
    val mutable n = 0
    method incr = n <- n + 1
  end

let () = printf "%d\n" (fact 5)
"##,
        &[
            "Module M",
            "Type color",
            "Type point",
            "Other:exception Bad_input",
            "Constant origin",
            "Function add",
            "Function fact",
            "Module Shape",
            "Class counter",
            "Constant ()",
        ],
    );
}

#[test]
fn nix_structure() {
    check(
        "nix",
        r##"{ lib, stdenv, fetchurl }:

let
  version = "1.2.3";
  pname = "widget";
  mkHelper = name: "hello-${name}";
in
stdenv.mkDerivation rec {
  inherit pname version;

  src = fetchurl {
    url = "https://example.com/${pname}-${version}.tar.gz";
    sha256 = "0000";
  };

  buildInputs = [ lib ];

  meta = with lib; {
    description = "A widget";
    license = licenses.mit;
  };
}
"##,
        &[
            "Constant version",
            "Constant pname",
            "Function mkHelper",
            "Constant src",
            "Constant buildInputs",
            "Constant meta",
        ],
    );
}

#[test]
fn terraform_structure() {
    check(
        "terraform",
        r##"terraform {
  required_version = ">= 1.5"
}

variable "region" {
  type    = string
  default = "eu-central-1"
}

locals {
  name = "widget-${var.region}"
}

provider "aws" {
  region = var.region
}

resource "aws_s3_bucket" "assets" {
  bucket = local.name
  tags = {
    Env = "prod"
  }
}

module "network" {
  source = "./modules/network"
}

data "aws_ami" "ubuntu" {
  most_recent = true
}

output "bucket_arn" {
  value = aws_s3_bucket.assets.arn
}
"##,
        &[
            "Other:terraform terraform",
            "Variable region",
            "Other:locals locals",
            "Other:provider aws",
            "Other:resource aws_s3_bucket.assets",
            "Module network",
            "Other:data aws_ami.ubuntu",
            "Other:output bucket_arn",
        ],
    );
}

#[test]
fn graphql_structure() {
    check(
        "graphql",
        r##"schema {
  query: Query
}

"A widget."
type Widget implements Node {
  id: ID!
  name: String!
}

interface Node {
  id: ID!
}

union Thing = Widget | Gadget

enum Color {
  RED
  GREEN
}

input WidgetInput {
  name: String!
}

scalar DateTime

type Query {
  widget(id: ID!): Widget
}

type Mutation {
  createWidget(input: WidgetInput!): Widget
}

fragment WidgetParts on Widget {
  id
  name
}

query GetWidget($id: ID!) {
  widget(id: $id) { ...WidgetParts }
}
"##,
        &[
            "Other:schema ",
            "Other:type Widget",
            "Field Widget/id",
            "Field Widget/name",
            "Other:interface Node",
            "Field Node/id",
            "Other:union Thing",
            "Other:enum Color",
            "Constant Color/RED",
            "Constant Color/GREEN",
            "Other:input WidgetInput",
            "Field WidgetInput/name",
            "Other:scalar DateTime",
            "Other:type Query",
            "Field Query/widget",
            "Other:type Mutation",
            "Field Mutation/createWidget",
            "Other:fragment WidgetParts",
            "Other:query GetWidget",
        ],
    );
}

#[test]
fn groovy_structure() {
    check(
        "groovy",
        r##"package com.example

import groovy.transform.CompileStatic

@CompileStatic
class Widget implements Serializable {
    String name
    int size = 1

    Widget(String name) {
        this.name = name
    }

    String render() {
        return "$name:$size"
    }

    static Widget of(String n) { new Widget(n) }
}

interface Renderable {
    String render()
}

trait Loud {
    String shout() { render().toUpperCase() }
}

enum Color { RED, GREEN }

def helper(String s) {
    s.trim()
}

println new Widget("a").render()
"##,
        &[
            "Class Widget",
            "Field Widget/name",
            "Field Widget/size",
            "Constructor Widget/Widget",
            "Method Widget/render",
            "Method Widget/of",
            "Interface Renderable",
            "Method Renderable/render",
            "Trait Loud",
            "Method Loud/shout",
            "Enum Color",
            "Function helper",
        ],
    );
}

#[test]
fn gradle_structure() {
    check(
        "groovy",
        r##"plugins {
    id 'java'
    id 'application'
}

group = 'com.example'
version = '1.0.0'

repositories {
    mavenCentral()
}

dependencies {
    implementation 'com.google.guava:guava:33.0.0-jre'
    testImplementation 'junit:junit:4.13.2'
}

task hello {
    doLast {
        println 'hello'
    }
}

def computeVersion(String base) {
    return base + '-SNAPSHOT'
}
"##,
        &[
            "Other:block plugins",
            "Constant group",
            "Constant version",
            "Other:block repositories",
            "Other:block dependencies",
            "Other:block task hello",
            "Function computeVersion",
        ],
    );
}

#[test]
fn svelte_structure() {
    check(
        "svelte",
        r##"<script context="module" lang="ts">
  export function preload(page) {
    return { id: page.params.id };
  }
</script>

<script lang="ts">
  export let name: string = "world";
  let count = 0;

  function increment() {
    count += 1;
  }

  $: doubled = count * 2;
</script>

<h1>Hello {name}</h1>
{#if count > 0}
  <p>{doubled}</p>
{/if}
<button on:click={increment}>+</button>

<style>
  h1 { color: red; }
</style>
"##,
        &[
            "Other:script script context=\"module\" lang=\"ts\"",
            "Function script context=\"module\" lang=\"ts\"/preload",
            "Other:script script lang=\"ts\"",
            "Variable script lang=\"ts\"/name",
            "Variable script lang=\"ts\"/count",
            "Function script lang=\"ts\"/increment",
            "Other:style style",
        ],
    );
}

#[test]
fn vue_structure() {
    check(
        "vue",
        r##"<template>
  <div class="widget">
    <h1>{{ title }}</h1>
    <button @click="increment">{{ count }}</button>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";

export default defineComponent({
  name: "Widget",
  props: { title: { type: String, required: true } },
  setup() {
    const count = ref(0);
    function increment() {
      count.value += 1;
    }
    return { count, increment };
  },
});
</script>

<style scoped>
.widget { color: red; }
</style>
"##,
        &[
            "Other:template template",
            "Other:script script lang=\"ts\"",
            "Method script lang=\"ts\"/setup",
            "Function script lang=\"ts\"/setup/increment",
            "Other:style style scoped",
        ],
    );
}

#[test]
fn rust_structure() {
    check(
        "rust",
        r##"use std::collections::HashMap;
pub use crate::inner::Thing;

pub const MAX: usize = 10;
static NAME: &str = "widget";

pub type Registry = HashMap<String, u32>;

#[derive(Debug)]
pub struct Widget {
    pub name: String,
}

pub enum Color { Red, Green }

pub trait Render {
    fn render(&self) -> String;
    fn tag(&self) -> &str { "w" }
}

impl Render for Widget {
    fn render(&self) -> String { self.name.clone() }
}

impl Widget {
    pub fn new(name: &str) -> Self { Widget { name: name.into() } }
    fn hidden(&self) {}
}

pub mod inner {
    pub struct Thing;
}

macro_rules! shout {
    ($e:expr) => { $e };
}

pub fn main_helper() -> u32 { 1 }

pub union U { a: u32, b: f32 }
"##,
        &[
            "Constant MAX [pub]",
            "Variable NAME",
            "Type Registry [pub]",
            "Struct Widget [pub]",
            "Enum Color [pub]",
            "Trait Render [pub]",
            "Function Render/render",
            "Function Render/tag",
            "Impl Render for Widget",
            "Function Render for Widget/render",
            "Impl Widget",
            "Function Widget/new [pub]",
            "Function Widget/hidden",
            "Module inner [pub]",
            "Struct inner/Thing [pub]",
            "Macro shout",
            "Function main_helper [pub]",
            "Struct U [pub]",
        ],
    );
}

/// A single-file component's `<script>` imports belong to the component: the
/// outer grammar sees opaque text, so they are recovered by re-parsing the
/// script body — and their spans must land back on the HOST file's lines.
#[test]
fn component_script_imports() {
    let svelte = r##"<script context="module" lang="ts">
  import { load } from "$lib/api";
</script>

<script lang="ts">
  import Button from "./Button.svelte";
  let count = 0;
</script>

<h1>{count}</h1>
"##;
    let vue = r##"<template><div /></template>

<script lang="ts">
import { defineComponent, ref } from "vue";
export default defineComponent({ setup() { return { n: ref(0) }; } });
</script>
"##;
    for (language, source, expected) in [
        (
            "svelte",
            svelte,
            vec![
                ("import { load } from \"$lib/api\";", 1_usize),
                ("import Button from \"./Button.svelte\";", 5),
            ],
        ),
        ("vue", vue, vec![("import { defineComponent, ref } from \"vue\";", 3)]),
    ] {
        let result = process(source, &ProcessConfig::new(language)).expect("grammar available");
        assert_eq!(result.metrics.error_count, 0, "{language} sample must parse");
        let actual: Vec<(&str, usize)> = result
            .imports
            .iter()
            .map(|i| (i.source.as_str(), i.span.start_line))
            .collect();
        assert_eq!(actual, expected, "{language} imports");
        for import in &result.imports {
            assert_eq!(
                &source[import.span.start_byte..import.span.end_byte],
                import.source,
                "{language} import span must address the host file"
            );
        }
    }
}
