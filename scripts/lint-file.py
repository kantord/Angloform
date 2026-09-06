#!/usr/bin/env python3
"""Document-level lint (measurement only, never gating) for any markdown
file — not only this project's own ADR-shaped docs (docs/markdown-linting.md
has the block-classification and heading rules).

For a markdown file: per-sentence verdicts and parse rate; a heading check
(vocabulary-only — headings are titles, not sentences: see
docs/markdown-linting.md); topic continuity (does the subject noun of each
sentence appear in the sentence before it?); and a relation inventory —
English coherence connectives in the source, grouped by relation type,
marked by whether angloform has a form for them.

Usage: lint-file.py FILE.md [--brief]   (brief: no per-sentence listing)
"""
import re, subprocess, sys, collections, os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mdblocks

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# relation type -> (connectives, angloform form or None)
RELATIONS = {
    "result":         (["so", "therefore", "thus", "hence", "consequently", "as a result"], ", so"),
    "reason":         (["because", "since", "for"], ", because"),
    "contrast":       (["but", "however", "yet", "whereas", "while", "instead"], "but"),
    "concession":     (["although", "though", "even though", "despite", "nevertheless", "still"], None),
    "condition":      (["if", "unless", "when", "whenever", "otherwise"], "if …, then"),
    "sequence":       (["then", "after", "before", "next", "finally", "first", "later", "once"], "after/before/until + noun phrase (ADR 0033)"),
    "exemplification":(["for example", "e.g.", "such as", "for instance"], None),
    "purpose":        (["so that", "in order to"], None),
    "elaboration":    (["in other words", "i.e.", "that is", "also", "moreover", "furthermore", "specifically", "namely"], None),
}
SKIP_FOR = True  # "for" is mostly a preposition; counted only as "for" + clause-ish is unreliable → skip

# a leading ordinal/numbering marker on a heading ("0037 —", "1.", "Step 3:")
# is structure, not vocabulary — never part of the word check
_HEADING_MARKER = re.compile(r"^(?:[A-Za-z]+\s+)?\d+[.\-—:]?\s*")
_ARTICLES = {"a", "an", "the", "of", "is", "are", "and", "or", "not", "to", "in", "on", "for", "with", "as"}


def sentences(text):
    """Prose sentences (with Enumeration/Step Block units kept whole, as
    single multi-line strings) from a block's already-de-markdowned text."""
    if "\n" in text:
        return [text]  # Enumeration / Step Block: one unit
    body = text.replace("e.g.", "e~g~").replace("i.e.", "i~e~").replace("cf.", "cf~")
    out = []
    for s in re.split(r"(?<=[.!?]) ", body):
        s = s.strip().rstrip(".").strip().replace("e~g~", "e.g.").replace("i~e~", "i.e.").replace("cf~", "cf.")
        if not s or len(s.split()) < 3 or re.fullmatch(r"[\d\W]+", s):
            continue
        out.append(s)
    return out


def lint(sents):
    """Batch the sentences through diagnose; return [(ok, first line of detail)].
    A multi-line sentence (an Enumeration/Step Block unit) is echoed back
    over several lines before its verdict detail, so the boundary is found
    by line count, not by "the next non-empty line" (that would grab the
    sentence's own second line as if it were the diagnosis)."""
    if not sents:
        return []
    r = subprocess.run(["cargo", "run", "-q", "-p", "diagnose", "--bin", "diagnose", "--", *sents],
                       cwd=ROOT, capture_output=True, text=True)
    lines = (r.stdout + r.stderr).splitlines()
    verdicts = []
    li = 0
    for s in sents:
        span = s.count("\n") + 1  # physical lines the echoed sentence occupies
        while li < len(lines) and not lines[li].startswith(("✓", "✗")):
            li += 1  # resync if a prior sentence's output ran short
        if li >= len(lines):
            break
        ok = lines[li].startswith("✓")
        li += span  # skip the rest of the echoed (possibly multi-line) sentence
        detail = ""
        while li < len(lines) and not lines[li].startswith(("✓", "✗")):
            if lines[li].strip() and not detail:
                detail = lines[li].strip()
            li += 1
        verdicts.append((ok, detail))
    return verdicts


def load_lexicon():
    lemma, tag, bans, rejects = {}, {}, {}, {}
    for line in open(os.path.join(ROOT, "lexicon.tsv")):
        if line.startswith("#"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) != 4:
            continue
        surface, kind, val3, val4 = f
        if kind == "form":
            tag[surface] = val3
            lemma[surface] = val4
        elif kind == "ban":
            bans[surface] = val4
        elif kind == "reject":
            rejects[surface] = (val3, val4)
    return lemma, tag, bans, rejects


def content_lemmas(sent, lemma, tag):
    """Nouns (by lemma) and names in a sentence; subject = first noun/name."""
    toks = re.findall(r'"[^"]*"|[A-Za-z][\w\-\']*', sent)
    nouns, subj = [], None
    for i, t in enumerate(toks):
        if t.startswith('"') or (t[0].isupper() and i > 0):
            key = t.strip('"').lower()
        else:
            w = t.lower()
            if tag.get(w, "").startswith("NOUN"):
                key = lemma[w]
            else:
                continue
        nouns.append(key)
        if subj is None:
            subj = key
    return subj, set(nouns)


def lint_heading(text, tag, bans, rejects):
    """A heading is a title, not a sentence (docs/markdown-linting.md): no
    angloform sentence grammar applies. The check is vocabulary-only, and
    only over lowercase words — a Capitalized word is a name/proper noun by
    the same convention prose uses (ADR 0018) and is never checked, so this
    stays usable on markdown that names real-world things angloform has no
    entry for (a product, a brand, a person). Returns (banned, wrong_sense,
    unknown) word lists; an empty heading check is a pass."""
    body = _HEADING_MARKER.sub("", text, count=1)
    banned, wrong, unknown = [], [], []
    for tok in re.findall(r'"[^"]*"|[A-Za-z][A-Za-z\'-]*|\d+', body):
        if tok.startswith('"') or tok[0].isdigit() or tok[0].isupper():
            continue
        w = tok.lower()
        if w in _ARTICLES or w in tag:
            continue
        if w in bans:
            banned.append((w, bans[w]))
        elif w in rejects:
            wrong.append((w, *rejects[w]))
        else:
            unknown.append(w)
    return banned, wrong, unknown


def relation_inventory(text):
    low = " " + re.sub(r"\s+", " ", text.lower()) + " "
    counts = collections.OrderedDict()
    for rel, (words, form) in RELATIONS.items():
        n = 0
        for w in words:
            if w == "for" and SKIP_FOR:
                continue
            pat = r"(?<![\w-])" + re.escape(w) + r"(?![\w-])"
            if w == "then":
                pat = r"(?<!, )" + pat  # ", then" is the conditional's marker, not sequence
            n += len(re.findall(pat, low))
        counts[rel] = (n, form)
    return counts


def main():
    path = sys.argv[1]
    brief = "--brief" in sys.argv
    text = open(path).read()
    lemma, tag, bans, rejects = load_lexicon()
    blocks = mdblocks.parse_blocks(text)

    headings = [b for b in blocks if b.kind == "heading"]
    prose_blocks = [b for b in blocks if b.kind in ("prose", "enumeration", "step_block", "mapping")]
    all_sents = [(bi, s) for bi, b in enumerate(prose_blocks) for s in sentences(b.text)]
    verdicts = lint([s for _, s in all_sents])
    ok = sum(1 for v, _ in verdicts if v)

    print(f"# Document lint: {os.path.relpath(path, ROOT)}\n")
    print(f"- Sentences: {len(all_sents)}; parse: {ok}/{len(all_sents)}"
          f" ({100*ok/max(1,len(all_sents)):.0f}%)")

    heading_checks = [(h, lint_heading(h.text, tag, bans, rejects)) for h in headings]
    h_bad = sum(1 for _, (b, w, u) in heading_checks if b or w)
    print(f"- Headings: {len(headings)}; with a banned or wrong-sense word: {h_bad}")

    # topic continuity within paragraphs
    pairs, breaks = 0, []
    prev = None
    for (bi, s), (v, _) in zip(all_sents, verdicts):
        subj, nouns = content_lemmas(s, lemma, tag)
        if prev is not None and prev[0] == bi and subj is not None:
            pairs += 1
            if subj not in prev[2]:
                breaks.append((subj, s))
        prev = (bi, subj, nouns)
    cont = pairs - len(breaks)
    print(f"- Topic continuity: {cont}/{pairs} consecutive pairs share the subject with the sentence before"
          f" ({100*cont/max(1,pairs):.0f}%)")
    inv = relation_inventory(text)
    print("- Relation inventory (source connectives, indicative counts — hand-check before deciding; ✓ = angloform has a form):")
    for rel, (n, form) in inv.items():
        if n:
            print(f"  - {rel}: {n}" + (f" ✓ {form}" if form else " ✗ no form"))

    if not brief:
        if heading_checks:
            print("\n## Headings\n")
            for h, (b, w, u) in heading_checks:
                mark = "✗" if (b or w) else "✓"
                print(f"- {mark} {'#' * h.level} {h.text}")
                for word, advice in b:
                    print(f"  - banned: \"{word}\" — {advice}")
                for word, cat, sug in w:
                    print(f"  - wrong sense: \"{word}\" is attested as {cat} — {sug}")
                if u:
                    print(f"  - not in the angloform vocabulary: {', '.join(sorted(set(u)))}")
        print("\n## Sentences\n")
        for (bi, s), (v, d) in zip(all_sents, verdicts):
            mark = "✓" if v else "✗"
            print(f"- {mark} {s}" + ("" if v else f"\n  - {d}"))
        if breaks:
            print("\n## Topic breaks (subject not in the previous sentence)\n")
            for subj, s in breaks:
                print(f"- *{subj}*: {s}")

if __name__ == "__main__":
    main()
