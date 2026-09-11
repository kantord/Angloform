#!/usr/bin/env python3
"""Find genuine WordNet-synonym redundancy in the lexicon.

"Genuine" is defined narrowly: two enabled words are redundant only if
WordNet places them in the *same synset* for the matching part of speech —
i.e. WordNet itself considers them interchangeable senses, not merely
similar in meaning. This is the same test that found the real `household`/
`home`/`house` duplicate on 2026-09-11.

Scope (mirrors the homophone-check retroactive/legacy policy):
  - curated words (seed/definitions/*.yaml) vs each other: checked, and any
    hit is a real problem to fix now (gated in check.sh via --curated-only).
  - curated words vs legacy seed.json words: checked, since a legacy word
    still "occupies" the sense even though it hasn't been migrated yet.
    Also gated.
  - legacy seed.json words vs each other: checked in the default report
    only, never gated. seed.json is unreviewed scaffolding on its way out;
    most hits here are expected noise (deliberately-kept near-synonyms
    like create/build/make), but this is where the real `household` find
    came from, so it stays a standing, honest backlog — surfaced, not
    forced.

Only NOUN / ADJ / VERB_TRANS / VERB_INTRANS categories have WordNet data;
closed-class categories (DET, PRON, CONJ, ...) are skipped.

Usage:
  python3 scripts/redundancy-check.py                  # full report, incl. legacy-vs-legacy backlog
  python3 scripts/redundancy-check.py --curated-only    # gate: exit 1 on curated-vs-curated hit only
"""
import json
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
WORDNET = ROOT / "data" / "wordnet"

POS_FILES = {
    "NOUN": "data.noun",
    "ADJ": "data.adj",
    "VERB_TRANS": "data.verb",
    "VERB_INTRANS": "data.verb",
}
# WordNet has one verb pos and one adjective pos; our two verb categories
# and adjective-ish categories collapse onto WordNet's namespace.
POS_KEY = {
    "NOUN": "noun",
    "ADJ": "adj",
    "VERB_TRANS": "verb",
    "VERB_INTRANS": "verb",
}


def load_wordnet_index():
    """pos_key -> lemma -> {(synset_offset, gloss)}"""
    index = {}
    for pos_key, filename in {"noun": "data.noun", "adj": "data.adj", "verb": "data.verb"}.items():
        path = WORDNET / filename
        lemma_map = {}
        with open(path, encoding="latin-1") as f:
            for line in f:
                if line.startswith("  "):
                    continue  # license header
                data, _, gloss = line.partition(" | ")
                fields = data.split()
                if not fields:
                    continue
                offset = fields[0]
                w_cnt = int(fields[3], 16)
                words = []
                idx = 4
                for _ in range(w_cnt):
                    word = fields[idx]
                    idx += 2
                    words.append(word)
                if len(words) < 2:
                    continue  # a synset of one word can't cause redundancy
                for w in words:
                    if "_" in w:
                        continue  # multi-word lemma, not a Surface Form clash
                    lemma = w.lower()
                    lemma_map.setdefault(lemma, set()).add((offset, gloss.strip()))
        index[pos_key] = lemma_map
    return index


def load_curated():
    """category -> {lemma: 'seed/definitions/<lemma>.yaml'}"""
    out = {}
    for path in sorted((ROOT / "seed" / "definitions").glob("*.yaml")):
        y = yaml.safe_load(path.read_text())
        cat = y["category"]
        if cat not in POS_FILES:
            continue
        out.setdefault(cat, {})[path.stem] = str(path.relative_to(ROOT))
    return out


def load_legacy():
    """category -> {lemma: 'seed/seed.json'}"""
    out = {}
    entries = json.loads((ROOT / "seed" / "seed.json").read_text())
    for e in entries:
        cat = e["category"]
        if cat not in POS_FILES:
            continue
        out.setdefault(cat, {})[e["lemma"]] = "seed/seed.json (legacy)"
    return out


def find_collisions(wn_index, curated, legacy):
    """Yields (lemma_a, source_a, lemma_b, source_b, offset, gloss)."""
    for cat, pos_file in POS_FILES.items():
        pos_key = POS_KEY[cat]
        lemma_map = wn_index[pos_key]
        curated_words = curated.get(cat, {})
        legacy_words = legacy.get(cat, {})
        # comparison pool: curated words are checked against (curated ∪ legacy);
        # legacy-vs-legacy pairs are never compared (out of scope, see docstring).
        pool = {**legacy_words, **curated_words}  # curated wins on lemma clash (can't happen: disjoint sets)
        for lemma_a, path_a in curated_words.items():
            synsets_a = lemma_map.get(lemma_a, set())
            if not synsets_a:
                continue
            for lemma_b, path_b in pool.items():
                if lemma_a == lemma_b:
                    continue
                synsets_b = lemma_map.get(lemma_b, set())
                shared = synsets_a & synsets_b
                for offset, gloss in shared:
                    yield (lemma_a, path_a, lemma_b, path_b, cat, offset, gloss)


def find_legacy_collisions(wn_index, legacy):
    """Yields (lemma_a, source_a, lemma_b, source_b, cat, offset, gloss) for
    legacy-vs-legacy pairs only. Report-only backlog, never gated."""
    for cat in POS_FILES:
        pos_key = POS_KEY[cat]
        lemma_map = wn_index[pos_key]
        legacy_words = sorted(legacy.get(cat, {}).items())
        for i, (lemma_a, path_a) in enumerate(legacy_words):
            synsets_a = lemma_map.get(lemma_a, set())
            if not synsets_a:
                continue
            for lemma_b, path_b in legacy_words[i + 1:]:
                synsets_b = lemma_map.get(lemma_b, set())
                shared = synsets_a & synsets_b
                for offset, gloss in shared:
                    yield (lemma_a, path_a, lemma_b, path_b, cat, offset, gloss)


def dedupe(pairs):
    seen = set()
    out = []
    for lemma_a, path_a, lemma_b, path_b, cat, offset, gloss in pairs:
        key = tuple(sorted((lemma_a, lemma_b))) + (cat, offset)
        if key in seen:
            continue
        seen.add(key)
        out.append((lemma_a, path_a, lemma_b, path_b, cat, offset, gloss))
    out.sort(key=lambda h: (h[4], h[0], h[2]))
    return out


def print_hits(hits):
    for lemma_a, path_a, lemma_b, path_b, cat, offset, gloss in hits:
        print(f"  [{cat}] {lemma_a!r} ({path_a})  ==  {lemma_b!r} ({path_b})")
        print(f"      WordNet {offset}: {gloss}")


def main():
    curated_only_gate = "--curated-only" in sys.argv

    wn_index = load_wordnet_index()
    curated = load_curated()
    legacy = load_legacy()

    gated_hits = dedupe(find_collisions(wn_index, curated, legacy))

    print(f"{sum(len(v) for v in curated.values())} curated words checked against WordNet "
          f"({sum(len(v) for v in legacy.values())} legacy seed.json words also in scope as targets)\n")

    if not gated_hits:
        print("0 same-synset redundancy pairs found (curated vs. curated+legacy).")
    else:
        print(f"{len(gated_hits)} same-synset redundancy pair(s) (curated vs. curated+legacy):\n")
        print_hits(gated_hits)

    if curated_only_gate:
        curated_vs_curated = [h for h in gated_hits if "seed/definitions" in h[1] and "seed/definitions" in h[3]]
        if curated_vs_curated:
            print(f"\nFAIL: {len(curated_vs_curated)} pair(s) are between two curated words — fix before committing.",
                  file=sys.stderr)
            return 1
        return 0

    legacy_hits = dedupe(find_legacy_collisions(wn_index, legacy))
    print(f"\n--- legacy seed.json backlog (informational, never gated, {len(legacy)} categories scanned) ---\n")
    if not legacy_hits:
        print("0 same-synset pairs found within legacy seed.json.")
    else:
        print(f"{len(legacy_hits)} same-synset pair(s) within legacy seed.json "
              f"(expected noise from deliberate near-synonyms; review individually before acting):\n")
        print_hits(legacy_hits)

    return 0


if __name__ == "__main__":
    sys.exit(main())
