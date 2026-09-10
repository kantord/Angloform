#!/usr/bin/env python3
"""Pre-authoring check for a word's cross-POS senses.

Usage: python3 scripts/word-check.py <lemma>

Two modes, picked automatically:

- **File-authoring mode** (seed/definitions/<lemma>.yaml exists): checks
  every attested-but-not-own category (the same union of WordNet index.* +
  moby mobypos.txt that lexgen's own cross-POS check uses, crates/lexgen/
  src/refdata.rs) and requires a "rejected[]" entry using "redirects" (a
  real, already-enabled substitute word) for each one. "advice" doesn't
  satisfy the check — free-text micro-definitions are exactly what this
  project's own authoring-format design doc
  (docs/lexicon-authoring-format-2026-09-07.md) and ADR 0061 both moved
  away from. Exit 0 only if every category has a real redirect.

- **Exploration mode** (no file yet, e.g. before writing one, or after a
  word was dropped and you want to re-check the reasoning without
  recreating the file): tries every attested category as the hypothetical
  "own" one and shows the same diagnostic for each of the others — no
  pass/fail, since nothing is authored yet to judge against. Always
  exits 0.

For each attested-but-uncovered category, prints:
  - the real WordNet gloss(es) for this lemma in that category, when
    WordNet attests it (data.noun/verb/adj/adv) — or a note that only moby
    attests it (moby has POS codes only, no gloss text at all)
  - every already-enabled word in that category, as candidate redirects
  - ADV is flagged as categorically unfixable: Angloform has no open
    adverb category (confirmed this session), so no redirect can ever
    exist for a rejected ADV sense — the word itself needs dropping.

Exit 0 only if every attested-but-not-own category already has a
"redirects" entry. Exit 1 otherwise (a validation failure, per the design
decided in this session — a word with no valid replacement for one of its
senses cannot be admitted as-is).

Side effect on a category that already passes: writes/updates
docs/word-sense-provenance.yaml, recording the WordNet gloss that
justified the chosen redirect. This is NOT part of the checked schema and
is never shown to a word's own definition consumer (e.g. `just define`) —
it exists purely to couple the redirect decision back to the source
thesaurus definition it replaced, for future audit/traceability.
"""
import re
import sys
from collections import defaultdict

ROOT = "/home/kantord/repos/minglish"
POS_LETTER = {"NOUN": "n", "VERB": "v", "ADJ": "a", "ADV": "r"}
POS_FILE = {"n": "noun", "v": "verb", "a": "adj", "r": "adv"}


def load_wordnet_index():
    pos_of = defaultdict(set)
    for letter, name in POS_FILE.items():
        with open(f"{ROOT}/data/wordnet/index.{name}") as f:
            for line in f:
                if line.startswith(" "):
                    continue
                parts = line.split()
                if len(parts) < 2 or "_" in parts[0]:
                    continue
                pos_of[parts[0]].add(letter)
    return pos_of


def load_moby():
    pos_of = defaultdict(set)
    code_map = {
        "N": "n", "p": "n", "h": "n",
        "V": "v", "t": "v", "i": "v",
        "A": "a",
        "v": "r",
    }
    with open(f"{ROOT}/data/moby/mobypos.txt", "rb") as f:
        text = f.read().decode("latin-1")
    for line in text.split("\n"):
        line = line.rstrip("\r")
        if "\\" not in line:
            continue
        word, codes = line.split("\\", 1)
        if not word.isascii() or not word.islower() or not word.isalpha():
            continue
        for c in codes:
            if c in code_map:
                pos_of[word].add(code_map[c])
    return pos_of


def attested_categories(lemma, wn_pos, moby_pos):
    letters = wn_pos.get(lemma, set()) | moby_pos.get(lemma, set())
    inv = {v: k for k, v in POS_LETTER.items()}
    return {inv[letter] for letter in letters}


def parse_data_line(line):
    header, _, gloss = line.partition("|")
    tokens = header.split()
    w_cnt = int(tokens[3], 16)
    words = []
    idx = 4
    for _ in range(w_cnt):
        w = re.sub(r"\(\w+\)$", "", tokens[idx])
        words.append(w.replace("_", " "))
        idx += 2
    return words, gloss.strip()


def glosses_for(lemma, category):
    letter = POS_LETTER[category]
    if letter not in POS_FILE:
        return None
    path = f"{ROOT}/data/wordnet/data.{POS_FILE[letter]}"
    out = []
    with open(path) as f:
        for line in f:
            if line.startswith(" ") or not line.strip():
                continue
            words, gloss = parse_data_line(line)
            if lemma in words:
                out.append(gloss)
    return out


def enabled_words_in(category):
    tags = {
        "NOUN": {"NOUN_SG"},
        "VERB": {"VERB_TRANS_BASE", "VERB_INTRANS_BASE"},
        "ADJ": {"ADJ", "ADJ_LONG"},
        "ADV": set(),  # no open adverb category exists
    }[category]
    words = []
    with open(f"{ROOT}/lexicon.tsv") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 4 or parts[1] != "form":
                continue
            lemma, _, tag, surface = parts
            if not lemma.islower():
                continue  # capitalized domain-model term (ADR 0018), never a valid redirect target
            if tag in tags:
                words.append(lemma)
    return sorted(set(words))


def load_yaml(lemma):
    import yaml
    try:
        with open(f"{ROOT}/seed/definitions/{lemma}.yaml") as f:
            return yaml.safe_load(f)
    except FileNotFoundError:
        return None


def update_provenance(lemma, category, gloss, redirect=None, gap=False):
    import yaml
    path = f"{ROOT}/docs/word-sense-provenance.yaml"
    try:
        with open(path) as f:
            data = yaml.safe_load(f) or {}
    except FileNotFoundError:
        data = {}
    data.setdefault(lemma, {})[category] = {
        "gloss": gloss if gloss else None,
        "redirect": redirect,
        "gap": gap,
    }
    with open(path, "w") as f:
        f.write(
            "# Provenance for rejected-sense redirects (scripts/word-check.py).\n"
            "# Not part of the checked schema, never shown to a word's own\n"
            "# definition consumer (e.g. `just define`) — couples each redirect\n"
            "# decision back to the WordNet gloss that justified it, for audit.\n"
        )
        yaml.safe_dump(dict(sorted(data.items())), f, sort_keys=True, default_flow_style=False, allow_unicode=True, width=100)


def check_one_category(lemma, cat, entry):
    """Print the diagnostic for one attested-but-not-own category. Returns
    True if it's satisfied (a real "redirects" entry already exists,
    file-authoring mode) or, in exploration mode (entry is None), just
    reports what exists — no pass/fail, nothing authored yet to judge."""
    print(f"\n=== {lemma} / {cat} (attested, not the enabled sense) ===")

    if cat == "ADV":
        print("  Angloform has no open adverb category — no redirect can ever exist for this.")
        if entry is not None:
            rejected = entry.get("rejected") or []
            if any(r["category"] == "ADV" and r.get("gap") for r in rejected):
                print("  GAP: acknowledged — this is the only valid way to reject ADV.")
                update_provenance(lemma, "ADV", None, gap=True)
                return True
            print("  FAIL: needs \"gap: true\" (never a redirect, never advice)")
            return False
        print("  Only \"gap: true\" can ever satisfy this category.")
        return None

    wn_glosses = glosses_for(lemma, cat)
    if wn_glosses:
        print("  WordNet gloss(es):")
        for g in wn_glosses:
            print(f"    - {g}")
        gloss_for_log = wn_glosses[0]
    else:
        print("  no WordNet gloss — attested only via moby (POS codes, no gloss text)")
        gloss_for_log = None

    candidates = enabled_words_in(cat)
    print(f"  {len(candidates)} already-enabled candidate word(s) in {cat}:")
    print("    " + ", ".join(candidates) if candidates else "    (none enabled in this category at all)")

    if entry is None:
        return None  # exploration mode: nothing authored yet, nothing to judge

    rejected = entry.get("rejected") or []
    existing_redirects = {r["category"]: r["redirects"][0] for r in rejected if r.get("redirects")}
    existing_gaps = {r["category"] for r in rejected if r.get("gap")}

    chosen = existing_redirects.get(cat)
    if chosen:
        if chosen not in candidates:
            print(f"  FAIL: rejected[].redirects points to \"{chosen}\", which is not an enabled {cat} word")
            return False
        print(f"  OK: redirects to \"{chosen}\" (already enabled)")
        update_provenance(lemma, cat, gloss_for_log, redirect=chosen)
        return True

    if cat in existing_gaps:
        print("  GAP: acknowledged as a real, structural absence — no redirect exists yet.")
        print("  Not a failure, but re-check with word-check.py as the vocabulary grows.")
        update_provenance(lemma, cat, gloss_for_log, gap=True)
        return True

    uses_advice = any(r["category"] == cat and r.get("advice") for r in rejected)
    if uses_advice:
        print("  FAIL: uses \"advice\" (a hand-written micro-definition) instead of \"redirects\"/\"gap\"")
    else:
        print("  FAIL: no rejected[] entry for this category at all")
    return False


def main():
    if len(sys.argv) != 2:
        print("usage: python3 scripts/word-check.py <lemma>", file=sys.stderr)
        sys.exit(2)
    lemma = sys.argv[1]

    wn_pos = load_wordnet_index()
    moby_pos = load_moby()
    attested = attested_categories(lemma, wn_pos, moby_pos)
    if not attested:
        print(f"{lemma}: not attested in any category (WordNet or moby) at all", file=sys.stderr)
        sys.exit(2)

    entry = load_yaml(lemma)

    if entry is not None:
        # file-authoring mode: one fixed own_category, from the file
        own_category = entry["category"]
        own_category = "VERB" if own_category.startswith("VERB") else own_category
        other = sorted(attested - {own_category})
        if not other:
            print(f"{lemma}: no other attested category — nothing to reject, clean.")
            sys.exit(0)
        ok = all(check_one_category(lemma, cat, entry) for cat in other)
        print()
        if ok:
            print(f"{lemma}: PASS — every attested-but-not-own category has a real redirect or an acknowledged gap")
            sys.exit(0)
        print(f"{lemma}: FAIL — not ready to admit as-is (see above)")
        sys.exit(1)

    # exploration mode: no seed/definitions/<lemma>.yaml yet — try every
    # attested category as the hypothetical "own" one, so a word can be
    # scouted before ever writing a file for it (or re-checked after
    # dropping one, without recreating it)
    print(f"seed/definitions/{lemma}.yaml doesn't exist yet — exploring every attested category\n")
    for own_category in sorted(attested):
        other = sorted(attested - {own_category})
        print(f"### if \"{lemma}\" were {own_category} ###")
        if not other:
            print("  no other attested category — would be clean, nothing to reject.\n")
            continue
        for cat in other:
            check_one_category(lemma, cat, None)
        print()
    sys.exit(0)


if __name__ == "__main__":
    main()
