#!/usr/bin/env python3
"""Re-source drifted grammars from their repos at the definitions-pinned revs.

The deploy workflow's parser-sources bundle is frozen at upstream v1.10.3
(upstream stopped attaching bundles to releases). Any curated grammar whose
rev moved past that bundle — plus the Blockether clojure fork and newly
curated languages — is re-sourced here so shipped sources always match
sources/language_definitions.json.

Env: OVERRIDE_LANGS — space-separated language names (required).
Writes parsers/<lang>/{src,queries} in the CWD, replacing bundle contents.
"""
import json
import os
import pathlib
import re
import shutil
import subprocess

defs = json.load(open("sources/language_definitions.json"))
for lang in os.environ["OVERRIDE_LANGS"].split():
    e = defs[lang]
    repo, rev = e["repo"], (e.get("rev") or e.get("revision"))
    sub = e.get("directory") or ""
    work = pathlib.Path("/tmp/grammars") / lang
    shutil.rmtree(work, ignore_errors=True)
    subprocess.run(["git", "clone", repo, str(work)], check=True, capture_output=True)
    subprocess.run(["git", "-C", str(work), "checkout", rev], check=True, capture_output=True)
    gdir = work / sub if sub else work
    if not (gdir / "src" / "parser.c").exists():
        # generate:true grammars (e.g. swift) do not commit parser.c —
        # generate with the repo-pinned tree-sitter-cli.
        subprocess.run(["npm", "install", "--no-audit", "--no-fund"], cwd=gdir, check=True)
        subprocess.run(["npx", "--no-install", "tree-sitter", "generate"], cwd=gdir, check=True)
    dest = pathlib.Path("parsers") / lang
    shutil.rmtree(dest, ignore_errors=True)
    dest.mkdir(parents=True)
    shutil.copytree(gdir / "src", dest / "src")
    # Multi-grammar repos (tree-sitter-php) keep a shared common/ ABOVE the
    # grammar dir, included as "../../common/…" from src/scanner.c — which a
    # src/-only copy breaks. Mirror upstream clone_vendors.py: relocate
    # common/ into src/ and rewrite the relative includes in every .c file.
    common = work / "common"
    if common.exists():
        shutil.copytree(common, dest / "src" / "common")
        for c in (dest / "src").glob("**/*.c"):
            txt = c.read_text()
            rel = os.path.relpath(dest / "src" / "common", c.parent).replace("\\", "/") + "/"
            c.write_text(re.sub(r"\.\.[/\\](?:\.\.[/\\])*common[/\\]", rel, txt))
    q = gdir / "queries" if (gdir / "queries").exists() else work / "queries"
    if q.exists():
        shutil.copytree(q, dest / "queries")
    print(f"{lang}: sourced from {repo}@{rev[:10]}" + (" (generated)" if e.get("generate") else ""))
