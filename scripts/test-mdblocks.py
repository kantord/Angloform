#!/usr/bin/env python3
"""Regression test for scripts/mdblocks.py against
tests/markdown-cases/kitchen-sink.md — one example of every markdown
feature the module handles (docs/markdown-linting.md). Run by
./scripts/check.sh; exits nonzero on any failure.

Asserts three things, not just "it doesn't crash":
1. Every construct in the file lands in the block kind it should
   (structure), and excluded constructs (code/table/hr/frontmatter/HTML/
   footnote-definition) leave no trace in any other block's text (no
   leakage — this is exactly the class of bug the file was built to catch).
2. The real angloform sentences the file deliberately includes still parse
   clean through the actual grammar/linter, not just get classified right —
   a structural fix that broke the underlying sentence would pass part 1
   and fail here.
3. The known-not-angloform content (the guide/diagram/chart words, "Bold"
   opening a sentence, the non-colon nested list) is correctly rejected,
   with a real reason — proves the linter still runs on this content, it
   doesn't just silently pass everything.
"""
import subprocess, sys, os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
import mdblocks

CASE = os.path.join(ROOT, "tests/markdown-cases/kitchen-sink.md")
failures = []


def check(cond, msg):
    if not cond:
        failures.append(msg)


def main():
    text = open(CASE).read()
    blocks = mdblocks.parse_blocks(text)
    all_text = "\n".join(b.text for b in blocks)

    # -- structure: every kind appears, nothing unexpected ------------------
    kinds = [b.kind for b in blocks]
    for kind, min_count in [
        ("heading", 10), ("prose", 8), ("code", 1), ("table", 1),
        ("enumeration", 4), ("step_block", 1), ("hr", 1),
    ]:
        check(kinds.count(kind) >= min_count, f"expected >= {min_count} {kind!r} block(s), got {kinds.count(kind)}")

    # -- frontmatter: excluded, never leaks into any block -------------------
    check("title: Kitchen sink" not in all_text, "YAML frontmatter leaked into a block")
    check("date: 2026" not in all_text, "YAML frontmatter leaked into a block")

    # -- code: excluded entirely, no leakage ---------------------------------
    check("def add" not in all_text and "return a" not in all_text, "fenced code content leaked into a block")

    # -- table: excluded entirely ---------------------------------------------
    table_blocks = [b for b in blocks if b.kind == "table"]
    check(len(table_blocks) == 1 and table_blocks[0].text == "", "table block should carry no text")

    # -- horizontal rule: excluded --------------------------------------------
    check(any(b.kind == "hr" for b in blocks), "horizontal rule not recognized")

    # -- raw HTML: tag lines excluded, wrapped content still lints as prose --
    check("<details>" not in all_text and "<summary>" not in all_text, "raw HTML tag leaked into a block")
    check(any(b.kind == "prose" and "agent stores the report" in b.text for b in blocks),
          "content wrapped in an HTML block should still lint as prose")

    # -- footnotes: marker drops, definition excluded, sentence intact -------
    check("[^1]" not in all_text, "footnote marker did not drop")
    check("A footnote about the agent" not in all_text, "footnote definition leaked into a block")
    check(any(b.kind == "prose" and b.text == "The agent reads the file. The file is old."
              for b in blocks if "reads the file. The file is old" in b.text),
          "sentence around a footnote marker should be unaffected")

    # -- links: resolve to visible text, URL dropped -------------------------
    check(any("the agent reads the guide" in b.text.lower() for b in blocks), "link did not resolve to its text")
    check("example.com" not in all_text, "link URL leaked into a block")

    # -- standalone image: becomes a heading-kind block with the alt text ----
    check(any(b.kind == "heading" and b.text == "A diagram of the pipeline" for b in blocks),
          "a standalone image should become a heading-kind block carrying its alt text")
    check("diagram.png" not in all_text, "image URL leaked into a block")

    # -- inline image: alt text folds into the surrounding sentence ----------
    check(any("a chart" in b.text and b.kind == "prose" for b in blocks),
          "an inline image's alt text should fold into its sentence")

    # -- strikethrough / bold / italic / inline code -------------------------
    check(any("struck text" in b.text for b in blocks), "strikethrough did not strip to its inner text")
    check("**" not in all_text and "~~" not in all_text, "bold/strikethrough markers leaked")
    check(any('"inline code"' in b.text for b in blocks), "inline code did not become a quoted identifier")

    # -- nested lists: colon parent folds to Enumeration, non-colon too, ------
    # -- but neither glues siblings into one run-on sentence ------------------
    nested_ok = [b for b in blocks if b.kind == "enumeration" and "the old report" in b.text and "the new report" in b.text]
    check(len(nested_ok) >= 2, "expected a nested Enumeration under both the colon and non-colon parent items")
    for b in nested_ok:
        check(b.text.count("\n- ") == 2, f"nested items should stay separate '- ' lines, got: {b.text!r}")

    if failures:
        print(f"mdblocks structure: {len(failures)} failure(s)")
        for f in failures:
            print(f"  - {f}")
    else:
        print("mdblocks structure: ok")

    # -- part 2/3: run the real linter over the file -------------------------
    lint_out = subprocess.run(
        ["python3", os.path.join(ROOT, "scripts/lint-file.py"), CASE],
        cwd=ROOT, capture_output=True, text=True,
    ).stdout

    must_pass = [
        "The agent reads the file",
        "The agent stores the file",  # blockquote (1st sentence)
        "The file is old",  # blockquote (2nd sentence)
        "The language has 3 conjunctions:",  # Enumeration
        "Given the agent stores the file",  # Step Block
        "the agent stores the file",
        "the agent deletes the report",
        "The agent has 2 reports:",  # task-list Enumeration
        "the agent has 2 reports:",  # colon nested Enumeration
        "The agent stores the report",  # HTML-wrapped content
        "The agent stores the file after the rule",  # after the hr
    ]
    must_fail = [
        "The agent reads the guide",  # "guide" not angloform
        "The report has a chart",  # "chart" not angloform
        "the agent has 2 reports with no colon",  # missing Enumeration colon
        "Bold text and italic text",  # "Bold" opens a sentence
    ]
    lint_ok = True
    for s in must_pass:
        # a passing sentence is echoed with a ✓ on the same line, or its
        # first line, in the report's "## Sentences" listing
        if not any(s in line and line.strip().startswith("- ✓") for line in lint_out.splitlines()):
            print(f"  - expected a ✓ for: {s!r} (not found as passing)")
            lint_ok = False
    for s in must_fail:
        if any(s in line and line.strip().startswith("- ✓") for line in lint_out.splitlines()):
            print(f"  - expected a ✗ for: {s!r} (found passing instead)")
            lint_ok = False
    print("lint-file.py end to end: ok" if lint_ok else "lint-file.py end to end: FAILED")

    if failures or not lint_ok:
        sys.exit(1)


if __name__ == "__main__":
    main()
