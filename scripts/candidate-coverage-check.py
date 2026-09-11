#!/usr/bin/env python3
"""Every high-frequency vocabulary candidate must have a documented fate.

docs/vocabulary-candidates.yaml is a ledger of thesaurus-derived candidate
words (scripts/vocab-candidates.py). Its own header says "no `rejected`
field means accepted by default" — but at the current scale (34k+
candidates) most entries have neither been added to the lexicon NOR given
a `rejected` reason; they're simply unprocessed backlog. That's fine for
low-frequency words, but silently unresolved among the words people
actually use most is a real gap (found 2026-09-11: 62 of the top 100
words by frequency were neither enabled nor rejected).

This gates only the highest-frequency slice (--top, default 100): each of
those candidates must be enabled (seed/definitions, seed.json, or a
vocabulary pack) or carry a `rejected` reason. Lower-frequency candidates
stay an honest, unenforced backlog — same policy as the vocab-ratchet
process's own "automated only, skip manual review at the tail" call.

Usage:
  python3 scripts/candidate-coverage-check.py            # gate, top 100
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
    ap.add_argument("--top", type=int, default=100)
    args = ap.parse_args()

    cands = yaml.safe_load((ROOT / "docs" / "vocabulary-candidates.yaml").read_text())
    enabled = load_enabled()
    freq_rank = load_freq_rank()

    ranked = sorted((w for w in cands if w in freq_rank), key=lambda w: freq_rank[w])
    top = ranked[: args.top]

    unresolved = [w for w in top if w not in enabled and "rejected" not in cands[w]]

    print(f"top {args.top} candidates by frequency: "
          f"{sum(1 for w in top if w in enabled)} enabled, "
          f"{sum(1 for w in top if 'rejected' in cands[w])} rejected-with-reason, "
          f"{len(unresolved)} unresolved")

    if unresolved:
        print(f"\nFAIL: {len(unresolved)} of the top {args.top} candidates have neither been "
              f"enabled nor given a `rejected` reason in docs/vocabulary-candidates.yaml:",
              file=sys.stderr)
        for w in unresolved:
            print(f"  {w} (rank {freq_rank[w] + 1})", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
