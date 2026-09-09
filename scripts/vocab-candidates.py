#!/usr/bin/env python3
"""Vocabulary-rebuild candidate collection (docs/vocabulary-rebuild-2026-09-09.md).

Usage: python3 scripts/vocab-candidates.py [N]   (default N=500)

For each of the first N unique WordNet-resolvable lemmas by frequency, count
senses per POS from data/wordnet/index.*. A lemma's candidate category is
whichever POS has exactly 1 sense; ties at 1, or no category at 1, defer it.
Lemmas already present in seed.json (any category) are excluded up front.

Reproducible: admit-eligible lemmas (strong + weak margin) are merged into
docs/vocabulary-candidates.yaml. The only field a human ever hand-edits there
is `rejected` (a short reason) — no `rejected` field means accepted by
default. Re-running with a larger N adds fresh, unreviewed entries for
genuinely new lemmas and leaves every existing entry (including its
`rejected` reason) untouched; a lemma that falls out of the candidate set
(promoted into the real vocabulary, or outside a smaller N) is dropped from
the ledger — its fate is tracked by the real vocabulary from then on.
"""
import re
import sys
from collections import defaultdict

ROOT = "/home/kantord/repos/minglish"
POS_FILES = {"n": "noun", "v": "verb", "a": "adj", "r": "adv"}
PILOT_SIZE = int(sys.argv[1]) if len(sys.argv) > 1 else 500

def load_index(pos_tag, filename):
    counts = {}
    with open(f"{ROOT}/data/wordnet/index.{filename}") as f:
        for line in f:
            if line.startswith("  "):
                continue
            parts = line.split()
            if len(parts) < 3:
                continue
            lemma = parts[0].replace("_", " ")
            try:
                synset_cnt = int(parts[2])
            except ValueError:
                continue
            counts[lemma] = synset_cnt
    return counts

def load_exc(filename):
    m = defaultdict(list)
    with open(f"{ROOT}/data/wordnet/{filename}.exc") as f:
        for line in f:
            parts = line.split()
            if len(parts) < 2:
                continue
            inflected, bases = parts[0], parts[1:]
            m[inflected].extend(bases)
    return m

print("loading index files...", file=sys.stderr)
index = {name: load_index(tag, name) for tag, name in POS_FILES.items()}
exc = {name: load_exc(name) for name in POS_FILES.values()}
lemma_sets = {name: set(index[name].keys()) for name in POS_FILES.values()}

def regular_deinflect(word):
    """Crude regular-suffix fallback, tried only after .exc lookup fails."""
    candidates = []
    if word.endswith("ies") and len(word) > 3:
        candidates.append(word[:-3] + "y")
    if word.endswith("es"):
        candidates.append(word[:-2])
    if word.endswith("s") and not word.endswith("ss"):
        candidates.append(word[:-1])
    if word.endswith("ing"):
        stem = word[:-3]
        # silent-e reconstruction and doubled-consonant undoing both apply
        # to real, common patterns ("making"->"make", "running"->"run") and
        # should win over the bare stem, which is more likely to spuriously
        # collide with an unrelated WordNet entry ("making"->"mak", a
        # real bug found by this pilot: "mak" independently exists as an
        # obscure lemma, silently winning if checked before "make").
        candidates.append(stem + "e")
        if len(stem) >= 2 and stem[-1] == stem[-2]:
            candidates.append(stem[:-1])
        candidates.append(stem)
    if word.endswith("ed"):
        stem = word[:-2]
        candidates.append(stem + "e")
        if len(stem) >= 2 and stem[-1] == stem[-2]:
            candidates.append(stem[:-1])
        candidates.append(stem)
    return candidates

def lemmatize(word):
    """Return the best-guess lemma for a wordform, checked against every
    POS's lemma set. De-inflection (via .exc, then regular suffix rules) is
    tried BEFORE accepting the word as its own lemma — fixes the round-3
    bug where an inflected form (e.g. "getting") resolved to its own rare
    index entry instead of reducing to its base ("get").

    -ing/-ed candidates are only accepted if the candidate is itself
    attested as a VERB — a second bug found by this pilot's own first run:
    "morning" was wrongly stemmed to "morn" (real, but noun-only — no verb
    "to morn" exists) instead of being recognized as its own common noun.
    -ing/-ed are exclusively verb inflections, so requiring the stem to
    exist as a verb (not just any POS) rules this out structurally, rather
    than by another ad hoc word-specific exception."""
    for name in POS_FILES.values():
        for base in exc[name].get(word, []):
            if base in lemma_sets[name]:
                return base
    is_verbal_suffix = word.endswith("ing") or word.endswith("ed")
    for cand in regular_deinflect(word):
        if is_verbal_suffix:
            if cand in lemma_sets["verb"]:
                return cand
            continue
        for name in POS_FILES.values():
            if cand in lemma_sets[name]:
                return cand
    if any(word in lemma_sets[name] for name in POS_FILES.values()):
        return word
    return None

def load_seed_lemmas():
    import json
    lemmas = set()
    for path in ("seed/seed.json",):
        with open(f"{ROOT}/{path}") as f:
            for e in json.load(f):
                lemmas.add(e["lemma"])
    import os
    for fname in os.listdir(f"{ROOT}/seed/definitions"):
        if fname.endswith(".yaml"):
            lemmas.add(fname[:-5])
    return lemmas

seed_lemmas = load_seed_lemmas()
print(f"{len(seed_lemmas)} lemmas already curated (excluded up front)", file=sys.stderr)

def counts_for(lemma):
    return {name: index[name].get(lemma, 0) for name in POS_FILES.values()}

seen = set()
results = []
walked = 0
with open(f"{ROOT}/data/freq/en_zipf.tsv") as f:
    for line in f:
        if line.startswith("#"):
            continue
        word = line.split("\t")[0].strip()
        if not word or not re.fullmatch(r"[a-z][a-z.'-]*", word):
            continue
        walked += 1
        lemma = lemmatize(word)
        if lemma is None or lemma in seen:
            continue
        seen.add(lemma)
        if lemma in seed_lemmas:
            continue
        c = counts_for(lemma)
        results.append((lemma, c))
        if len(results) >= PILOT_SIZE:
            break

print(f"walked {walked} frequency rows to collect {len(results)} new candidate lemmas", file=sys.stderr)

import yaml as pyyaml

LEDGER_PATH = f"{ROOT}/docs/vocabulary-candidates.yaml"

strong, weak, tied, none_ = [], [], [], []
for lemma, c in results:
    ones = [pos for pos, n in c.items() if n == 1]
    if len(ones) == 0:
        none_.append((lemma, c))
    elif len(ones) > 1:
        tied.append((lemma, c, ones))
    else:
        winner = ones[0]
        others = {pos: n for pos, n in c.items() if pos != winner and n > 0}
        if others:
            weak.append((lemma, c, winner, others))
        else:
            strong.append((lemma, c, winner))

def fmt_counts(c):
    return ", ".join(f"{pos}={n}" for pos, n in c.items() if n > 0) or "none"

out = []
out.append(f"# Vocabulary candidate pilot ({len(results)} new lemmas, top {walked} frequency rows walked)\n")
out.append(f"- strong clean admits: {len(strong)}")
out.append(f"- weak-margin clean admits: {len(weak)}")
out.append(f"- tied at 1: {len(tied)}")
out.append(f"- no clean winner: {len(none_)}\n")

out.append("## Clean admits, strong\n")
out.append("| lemma | category | counts |\n|---|---|---|")
for lemma, c, winner in strong:
    out.append(f"| {lemma} | {winner} | {fmt_counts(c)} |")

out.append("\n## Clean admits, weak margin\n")
out.append("| lemma | category | counts |\n|---|---|---|")
for lemma, c, winner, others in weak:
    out.append(f"| {lemma} | {winner} | {fmt_counts(c)} |")

out.append("\n## Deferred: tied at 1\n")
out.append("| lemma | tied categories | counts |\n|---|---|---|")
for lemma, c, ones in tied:
    out.append(f"| {lemma} | {', '.join(ones)} | {fmt_counts(c)} |")

with open(f"{ROOT}/docs/vocabulary-candidates-pilot-2026-09-09.md", "w") as f:
    f.write("\n".join(out) + "\n")

print(f"strong={len(strong)} weak={len(weak)} tied={len(tied)} none={len(none_)}", file=sys.stderr)
print("wrote docs/vocabulary-candidates-pilot-2026-09-09.md", file=sys.stderr)

# -- reproducible ledger --------------------------------------------------
# Only "admit-eligible" words (strong + weak) go in the ledger — tied/none
# are deferred by the mechanical rule itself, no human accept/reject
# decision needed yet. Absence of `rejected` = accepted by default; the
# ONLY field a human ever hand-edits is `rejected` (a short reason). A
# re-run with a larger word count keeps every existing entry (including
# its `rejected` reason, if any) untouched; only genuinely new lemmas get
# a fresh, unreviewed entry. A lemma that drops out of the candidate set
# entirely (promoted into the real vocabulary, or no longer resolvable)
# is removed from the ledger — its fate is tracked by the real vocabulary
# from that point on, not here.
try:
    with open(LEDGER_PATH) as f:
        existing_ledger = pyyaml.safe_load(f) or {}
except FileNotFoundError:
    existing_ledger = {}

admit_eligible = {}
for lemma, c, winner in strong:
    admit_eligible[lemma] = {"category": winner, "bucket": "strong", "counts": {k: v for k, v in c.items() if v > 0}}
for lemma, c, winner, others in weak:
    admit_eligible[lemma] = {"category": winner, "bucket": "weak", "counts": {k: v for k, v in c.items() if v > 0}}

merged = {}
kept, new = 0, 0
for lemma, entry in admit_eligible.items():
    if lemma in existing_ledger:
        merged[lemma] = existing_ledger[lemma]
        kept += 1
    else:
        merged[lemma] = entry
        new += 1
dropped = [lemma for lemma in existing_ledger if lemma not in admit_eligible]

with open(LEDGER_PATH, "w") as f:
    f.write(
        "# Auto-regenerated by scripts/vocab-candidates.py — the ONLY field a\n"
        "# human ever hand-edits is `rejected` (a short reason). No `rejected`\n"
        "# field means accepted by default. Re-running with a larger word count\n"
        "# preserves every existing entry (including its `rejected` reason)\n"
        "# untouched; only genuinely new lemmas get a fresh, unreviewed entry.\n"
    )
    pyyaml.safe_dump(dict(sorted(merged.items())), f, sort_keys=False, default_flow_style=False, allow_unicode=True, width=100)

print(f"ledger: {kept} kept as-is, {new} new (unreviewed), {len(dropped)} dropped (no longer a candidate)", file=sys.stderr)
if dropped:
    print(f"  dropped: {dropped}", file=sys.stderr)
print(f"wrote {LEDGER_PATH}", file=sys.stderr)
