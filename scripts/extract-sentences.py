#!/usr/bin/env python3
"""Extract prose sentences from a markdown file, one per line, for linting.
Backticked spans become angloform quoted identifiers, links resolve to their
visible text, bold/italic strip; an Enumeration or Step Block prints as one
line with its internal breaks marked by ⏎. Headings, tables, code fences,
and horizontal rules are excluded — see docs/markdown-linting.md."""
import re, sys, os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mdblocks

text = open(sys.argv[1]).read()
for b in mdblocks.parse_blocks(text):
    if b.kind == "prose":
        body = b.text.replace("e.g.", "e~g~").replace("i.e.", "i~e~").replace("cf.", "cf~")
        for s in re.split(r"(?<=[.!?]) ", body):
            s = s.strip().rstrip(".").strip().replace("e~g~", "e.g.").replace("i~e~", "i.e.").replace("cf~", "cf.")
            if not s or len(s.split()) < 3 or re.fullmatch(r"[\d\W]+", s):
                continue
            print(s)
    elif b.kind in ("enumeration", "step_block"):
        print(b.text.replace("\n", "⏎"))
