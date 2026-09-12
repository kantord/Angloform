#!/usr/bin/env python3
"""Find enabled words with zero usage anywhere in load-bearing content.

Distinct from redundancy-check.py (which asks "is this word a duplicate
of another enabled word?") — this asks a simpler, purely mechanical
question with no semantic judgment involved: does ANYTHING actually use
this word's surface form? A word can be non-redundant and non-colliding
and still be dead weight if nothing in the corpus, domain model, curated
definitions, or test suite ever needed it.

"Used" means: at least one of the word's enabled Surface Forms appears
as a whole word in one of:
  - corpus/*.tsv (the Angloform column specifically, not the English source)
  - domain/model.json (definition text, examples)
  - seed/definitions/*.yaml (definition, antonym, redirects text)
  - seed/seed.json (another word's reject/redirect target)
  - crates/**/tests/*.rs and crates/**/src/*.rs (string literals — the
    Rust test suite is real, load-bearing coverage of the grammar)

A word here is NOT necessarily wrong to remove — many exist precisely to
be available for future prose, not because something already needs them
today. This is a report, not a gate: it surfaces candidates for a human
judgment call (matching household/readable/numeral precedent), the same
way the legacy redundancy backlog does.

Usage:
  python3 scripts/dead-weight-check.py
"""
import glob
import json
import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent


def load_lexicon_forms():
    """surface -> lemma, for every enabled 'form' row."""
    forms = {}
    lemma_forms = {}
    with open(ROOT / "lexicon.tsv") as f:
        for line in f:
            if line.startswith("#"):
                continue
            parts = line.rstrip("\n").split("\t")
            if len(parts) == 4 and parts[1] == "form":
                surface, _, _, lemma = parts
                forms[surface] = lemma
                lemma_forms.setdefault(lemma, set()).add(surface)
    return forms, lemma_forms


def load_usage_text():
    """Return one big list of (source_label, text) chunks to search."""
    chunks = []

    for p in glob.glob(str(ROOT / "corpus" / "*.tsv")):
        with open(p, encoding="utf-8", errors="ignore") as f:
            for line in f:
                parts = line.rstrip("\n").split("\t")
                if len(parts) >= 2:
                    chunks.append((f"corpus:{Path(p).name}", parts[1]))

    model = json.loads((ROOT / "domain" / "model.json").read_text())
    for e in model:
        chunks.append(("domain/model.json", e.get("definition", "")))
        for ex in e.get("examples", []) or []:
            if isinstance(ex, str):
                chunks.append(("domain/model.json", ex))

    for p in glob.glob(str(ROOT / "seed" / "definitions" / "*.yaml")):
        y = yaml.safe_load(Path(p).read_text())
        chunks.append((f"seed/definitions/{Path(p).name}", y.get("definition", "")))
        if y.get("antonym"):
            chunks.append((f"seed/definitions/{Path(p).name}", y["antonym"]))
        for r in y.get("rejected", []) or []:
            for w in r.get("redirects", []) or []:
                chunks.append((f"seed/definitions/{Path(p).name}", w))
            if r.get("advice"):
                chunks.append((f"seed/definitions/{Path(p).name}", r["advice"]))

    seed = json.loads((ROOT / "seed" / "seed.json").read_text())
    for e in seed:
        reject = e.get("reject", {})
        if isinstance(reject, dict):
            for v in reject.values():
                if isinstance(v, str):
                    chunks.append(("seed/seed.json (redirect)", v))

    for pattern in ("crates/**/tests/*.rs", "crates/**/src/*.rs"):
        for p in glob.glob(str(ROOT / pattern), recursive=True):
            chunks.append((p.replace(str(ROOT) + "/", ""), Path(p).read_text(encoding="utf-8", errors="ignore")))

    return chunks


def main():
    forms, lemma_forms = load_lexicon_forms()
    chunks = load_usage_text()

    # build one lowercase haystack per source is too slow to re-scan per word;
    # instead tokenize every chunk once into a set of words seen, unioned.
    seen_words = set()
    for _, text in chunks:
        for w in re.findall(r"[a-z]+(?:'[a-z]+)?", text.lower()):
            seen_words.add(w)

    curated_lemmas = {Path(p).stem for p in glob.glob(str(ROOT / "seed" / "definitions" / "*.yaml"))}
    legacy_lemmas = {e["lemma"] for e in json.loads((ROOT / "seed" / "seed.json").read_text())}

    unused_curated = []
    unused_legacy = []
    for lemma in sorted(curated_lemmas | legacy_lemmas):
        surfaces = lemma_forms.get(lemma, {lemma})
        if any(s.lower() in seen_words for s in surfaces):
            continue
        (unused_curated if lemma in curated_lemmas else unused_legacy).append(lemma)

    print(f"{len(curated_lemmas)} curated + {len(legacy_lemmas)} legacy lemmas checked "
          f"against corpus, domain model, definitions, redirects, and the Rust test suite\n")
    print(f"{len(unused_curated)} curated word(s) with zero usage anywhere:")
    for w in unused_curated:
        print(f"  {w}")
    print(f"\n{len(unused_legacy)} legacy word(s) with zero usage anywhere:")
    for w in unused_legacy:
        print(f"  {w}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
