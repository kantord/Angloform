#!/usr/bin/env python3
"""Every high-frequency vocabulary candidate must have a documented fate —
and every rejected one must have a documented EQUIVALENT.

docs/vocabulary-candidates.yaml is a ledger of thesaurus-derived candidate
words (scripts/vocab-candidates.py). Its own header says "no `rejected`
field means accepted by default" — but at the current scale (34k+
candidates) most entries have neither been added to the lexicon NOR given
a `rejected` reason; they're simply unprocessed backlog. That's fine for
low-frequency words, but silently unresolved among the words people
actually use most is a real gap (found 2026-09-11: 62 of the top 100
words by frequency were neither enabled nor rejected).

Stage 1 (--top, default 250): each candidate in that range must be
enabled (seed/definitions, seed.json, or a vocabulary pack) or carry a
`rejected` reason.

Stage 2, added 2026-09-11 per an explicit directive ("we have to have an
equivalent, period"): every REJECTED word in that same range must also
record what a writer should use instead — one of:
  redirect: "<word>"        an already-enabled word/lemma that covers it
  redirect_note: "<text>"   no single word substitutes (e.g. a banned
                             epistemic hedge, or a discourse particle
                             with no lexical content) — the strategy to
                             use instead
  gap: true                 genuinely searched, no existing equivalent —
                             an honest, recorded absence, not a guess
The only entries exempt from stage 2 are real garbage (not a word) or
proper nouns/demonyms/honorifics — tagged `exempt: "garbage"` or
`exempt: "proper-noun"`.

Lower-frequency candidates (beyond --top) stay an honest, unenforced
backlog — same policy as the vocab-ratchet process's own "automated
only, skip manual review at the tail" call.

Usage:
  python3 scripts/candidate-coverage-check.py            # gate, top 250
  python3 scripts/candidate-coverage-check.py --top 500  # gate, top 500
"""
import argparse
import glob
import json
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent


def load_enabled():
    """Every enabled Surface Form, not just lemmas — an irregular inflection
    (e.g. "best" as good's ADJ_SUP, "men" as man's NOUN_PL) is just as
    enabled as its lemma, and candidates are surfaced by surface spelling."""
    enabled = set()
    with open(ROOT / "lexicon.tsv") as f:
        for line in f:
            if line.startswith("#"):
                continue
            parts = line.rstrip("\n").split("\t")
            if len(parts) == 4 and parts[1] == "form":
                enabled.add(parts[0])
    return enabled


def load_freq_rank():
    rank = {}
    with open(ROOT / "data" / "freq" / "en_zipf.tsv") as f:
        for i, line in enumerate(f):
            w = line.split("\t")[0]
            if w not in rank:
                rank[w] = i
    return rank


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--top", type=int, default=250)
    args = ap.parse_args()

    cands = yaml.safe_load((ROOT / "docs" / "vocabulary-candidates.yaml").read_text())
    enabled = load_enabled()
    freq_rank = load_freq_rank()

    ranked = sorted((w for w in cands if w in freq_rank), key=lambda w: freq_rank[w])
    top = ranked[: args.top]

    unresolved = [w for w in top if w not in enabled and "rejected" not in cands[w]]
    rejected = [w for w in top if w not in enabled and "rejected" in cands[w]]

    equiv_fields = ("redirect", "redirect_note", "gap")
    exempt = [w for w in rejected if "exempt" in cands[w]]
    no_equivalent = [w for w in rejected if "exempt" not in cands[w]
                      and not any(f in cands[w] for f in equiv_fields)]

    print(f"top {args.top} candidates by frequency: "
          f"{sum(1 for w in top if w in enabled)} enabled, "
          f"{len(rejected)} rejected-with-reason, "
          f"{len(unresolved)} unresolved; "
          f"of the rejected: {len(exempt)} exempt (garbage/proper-noun), "
          f"{len(rejected) - len(exempt) - len(no_equivalent)} have a recorded equivalent, "
          f"{len(no_equivalent)} missing one")

    fail = False
    if unresolved:
        fail = True
        print(f"\nFAIL: {len(unresolved)} of the top {args.top} candidates have neither been "
              f"enabled nor given a `rejected` reason in docs/vocabulary-candidates.yaml:",
              file=sys.stderr)
        for w in unresolved:
            print(f"  {w} (rank {freq_rank[w] + 1})", file=sys.stderr)

    if no_equivalent:
        fail = True
        print(f"\nFAIL: {len(no_equivalent)} rejected candidates have no recorded equivalent "
              f"(`redirect`, `redirect_note`, or `gap`) and aren't `exempt`:", file=sys.stderr)
        for w in no_equivalent:
            print(f"  {w} (rank {freq_rank[w] + 1})", file=sys.stderr)

    return 1 if fail else 0


if __name__ == "__main__":
    sys.exit(main())
