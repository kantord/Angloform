#!/usr/bin/env python3
"""Static scan for one garden-path shape in the Tier-1 grammar (2026-09-06):
"X COMMA X" — two constituents of the same syntactic category joined only
by a comma, with no conjunction between them. This is the exact structural
signature behind the Appositive's confirmed garden path (docs/garden-paths-2026-09-06.md):
a reader cannot tell "an aside" from "an asyndetic list" until much later
material (often the predicate's agreement) resolves it, if anything does.

Not a general ambiguity checker — LALRPOP's own build already proves the
grammar has zero *formal* ambiguity. This looks for a specific *human*
garden-path shape a formally-unambiguous grammar can still contain, as a
candidate list for a human/agent reading test before shipping, not a final
verdict — severity still needs the same empirical check this file's own
history used (see docs/garden-paths-2026-09-06.md).

Usage: python3 scripts/garden-path-scan.py [crates/grammar/src/angloform.lalrpop]
"""
import re
import sys

# Terminal-wrapper nonterminals (leaves) all follow this project's own
# naming convention: "L" + the token name (LDet, LComma, LConj, ...). They
# never carry independent structure, so they're never part of a same-
# category-repeat risk on their own.
LEAF_PREFIX = "L"

# Manually curated equivalence classes: nonterminals that a reader would
# treat as "the same kind of thing" even when their grammar names differ.
# This is the one piece of real linguistic judgment a pure name-match can't
# supply — e.g. SubjSG and ApposContent are different productions but both
# resolve to a bare noun phrase, which is exactly why "SubjSG COMMA
# ApposContent" was the Appositive's actual risk shape, not "X COMMA X"
# with matching names.
EQUIV_CLASSES = [
    {"SubjSG", "SubjPL", "Subj1", "NPAny", "NPSG", "NPPL", "NPEvery", "NPNo",
     "NPSome", "BarePl", "ApposContent", "Item", "NPAnyOrOther"},
    {"Clause", "CoordClause"},
    {"Statement", "Sentence"},
]


def same_class(a, b):
    if a == b:
        return True
    return any(a in cls and b in cls for cls in EQUIV_CLASSES)


def strip_comments(text):
    return re.sub(r"//[^\n]*", "", text)


def alt_patterns(body):
    """One alternative's pattern per source line — this grammar's actual,
    consistent formatting puts exactly one `<pat> => <action>,` per line,
    so splitting on `=>` per line sidesteps needing a real bracket-depth
    parser for the Rust action code that follows it (which is riddled
    with `>` characters — `vec![...]`, `Some(...)` — that a naive angle-
    bracket depth counter mistakes for closing generics)."""
    for line in body.splitlines():
        if "=>" in line:
            yield line.split("=>", 1)[0]


def find_productions(text):
    """Yield (name, body) for each `Name<...>: Tree = BODY;` production,
    tracking brace depth so a production's body (which may itself contain
    `;`-free nested `{}`) is captured whole."""
    i = 0
    n = len(text)
    pat = re.compile(r"(?:pub\s+)?(\w+)(<[^>]*>)?\s*:\s*Tree\s*=\s*")
    while True:
        m = pat.search(text, i)
        if not m:
            return
        name = m.group(1)
        j = m.end()
        depth = 0
        start = j
        while j < n:
            if text[j] in "{(":
                depth += 1
            elif text[j] in "})":
                depth -= 1
            elif text[j] == ";" and depth == 0:
                break
            j += 1
        yield name, text[start:j]
        i = j + 1


def alt_symbols(alt):
    """Extract the ordered grammar-symbol names from one alternative's
    pattern (the part before `=>`, or the whole thing if there's no
    action). Handles `<binding:Name>` and bare `Name` references."""
    pattern = alt.split("=>")[0]
    return re.findall(r"<\w+\s*:\s*(\w+)>|(?<![\w:])([A-Z]\w*)(?!\s*:)", pattern)


def flatten(matches):
    return [a or b for a, b in matches]


def scan(path):
    text = strip_comments(open(path).read())
    findings = []
    for name, body in find_productions(text):
        for alt in alt_patterns(body):
            syms = flatten(alt_symbols(alt))
            for i in range(len(syms) - 2):
                if syms[i + 1] == "LComma" and same_class(syms[i], syms[i + 2]):
                    # Same-category repeat right across a bare comma. Flag
                    # only if nothing between position i+2 and the next
                    # comma/end is a conjunction wrapper (LConj) — if a
                    # conjunction already sits immediately after the
                    # SECOND item too, this is closer to a normal N-ary
                    # list already carrying its own marker nearby, still
                    # worth a look but lower priority.
                    tail = syms[i + 3 : i + 5]
                    marked = "LConj" in tail or "LNamely" in tail
                    findings.append((name, alt.strip()[:80], syms[i], syms[i + 2], marked))
    return findings


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "crates/grammar/src/angloform.lalrpop"
    findings = scan(path)
    if not findings:
        print("garden-path-scan: no same-category comma-only junctions found")
        return
    print(f"garden-path-scan: {len(findings)} candidate(s) — same-category constituents joined only by a comma\n")
    for prod, alt, a, b, marked in findings:
        flag = "marked nearby (lower priority)" if marked else "UNMARKED — read this one first"
        print(f"  [{prod}] {a} COMMA {b}  ({flag})")
        print(f"    alt: {alt}")
    print("\nThis is a candidate list, not a verdict — confirm real severity the same way")
    print("docs/garden-paths-2026-09-06.md did: a fresh agent reading the construction")
    print("incrementally, word by word, before trusting or dismissing any finding here.")


if __name__ == "__main__":
    main()
