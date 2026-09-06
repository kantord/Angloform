# Linting any markdown file

`just lint-file <file.md>` (`scripts/lint-file.py`) works on any markdown
file, not only this project's own ADR-shaped docs. The structural parsing
that makes that possible lives in `scripts/mdblocks.py`, shared with
`scripts/extract-sentences.py` (feeds `just autofix`/`just
autofix-paragraphs`) — one block classifier instead of three drifting
copies (a third, independent one already existed in `scripts/docjudge.py`
for a genuinely different job — see "What stays untouched" below).

## Block classification

`mdblocks.parse_blocks(text)` turns a document into typed blocks. Each
gets one of three treatments:

**Linted as prose** (full angloform sentence grammar applies): plain
paragraphs, blockquote content (the `> ` marker strips; the quoted text is
linted exactly like an unquoted paragraph), and plain bullet/task-list
items (each item is its own unit — a bullet holding two sentences splits
into two, the same as a paragraph would).

**Linted as a block** (the Enumeration or Step Block grammar applies, ADR
0028/0034): a paragraph ending in `:` immediately followed by a bullet
list becomes one Enumeration unit, intro plus items; consecutive
`Given`/`When`/`Then`/`And`/`Feature:`/`Scenario:` lines become one Step
Block. A bullet item whose own text ends in `:` and has a nested sub-list
folds the same way, recursively — `- the agent has 2 tasks:` with two
indented sub-items becomes one Enumeration, not a broken parent bullet.

**Excluded entirely** (not prose, never sent to the linter): fenced code
blocks, tables, horizontal rules, YAML frontmatter (`---` at the very
start of the file through the closing `---`), a raw HTML *tag line*
(`<details>`, a badge `<img>`, `<div align=…>` — content wrapped between
such tags still lints as prose; only the tag lines themselves are
structural markup, not text), footnote definitions (`[^1]: …`), reference-
link definitions (`[ref]: url`), and headings (see below — excluded from
the *prose* path, not from linting altogether).

**A standalone image is a heading, not a sentence.** A paragraph that is
*only* `![alt](url)` — a banner, a badge, a screenshot — gets the same
vocabulary-only check as a heading (below), not full sentence grammar:
alt text is title-shaped, not sentence-shaped, the same reason a heading
is. An image *inside* a sentence with other text just contributes its alt
text the way a link contributes its visible text (next paragraph).

**Resolved inline, wherever they appear** (link/image/emphasis/code/
footnote-marker syntax, applied to whatever text a block keeps):
`[text](url)` becomes `text` — the URL is data, not language, and a
reader parses what the link says, not where it points; `![alt](url)`
resolves to `alt` the same way, when it's part of a larger sentence (see
above for a standalone image); `` `code` `` becomes an angloform quoted
identifier, matching how this project already treats inline code in its
own docs; `**bold**`/`*italic*`/`~~strikethrough~~` strip to their inner
text; an inline footnote marker (`[^1]`) drops — its content lives in the
definition, excluded at the block level, not woven into the sentence.

Nested lists track indentation depth. A line indented deeper than its
list *without* its own bullet marker is a wrapped continuation of the
current item's text, not a new item — this is what stops a long bullet
that wraps across two physical lines from being read as two items. A
deeper line that *does* carry its own bullet marker is a genuine nested
sub-list: its items stay separate "- " lines under the parent item, never
concatenated into one run-on sentence. Whether that parses depends on the
parent's own text: if it ends in `:`, this is an Enumeration (ADR 0028)
and the grammar accepts it; if it doesn't, it correctly fails linting
with an actionable "not an angloform word" reason at the seam, rather than
silently mashing three separate bullet points into one nonsense sentence.

## Headings get a different rule, not the sentence grammar

A heading is a title, not a sentence — real headings are almost never
grammatical angloform, even in this project's own docs:

    0037 — Full-clause coordination: a comma before "but"/"and"/"or"
    0014 — every / no, and the first-token telegraph principle

Em dashes, colons, slashes, and sentence fragments joined by `and`/`or`
are the norm, not well-formed subject-verb-object angloform. Requiring a
heading to parse as a sentence — or even as a single Noun Phrase — would
fail on nearly every real heading, making the checker useless on real
files. Confirmed empirically before deciding this: sampling every `#`/`##`
heading across this project's own ADRs, essentially none would pass a
strict grammatical check.

So the heading rule is **vocabulary-only**: every lowercase word gets
checked against the lexicon (any category — no sentence role or agreement
check, since a heading has no sentence structure to hold one), split into
three outcomes:

- **banned** — a real, actionable find. `# 0028 — Enumeration: a Block
  that lists things` flags `"that"` (ADR 0002's banned relative pronoun) —
  a genuine problem in this project's own title, caught by this checker
  while writing this doc.
- **wrong sense** — the word is attested, but only in a different category
  (the noun/verb-redirect situation ADR 0008 already handles for prose).
- **not in the vocabulary** — informational only, never a failure. Any
  markdown file talking about real things (a product, a person, a
  concept) will have plenty of these; that's expected, not a defect.

A leading number-and-separator (`"0037 —"`, `"1."`, `"Step 3:"`) is
structure, stripped before the word check, never treated as vocabulary. A
Capitalized word is never checked at all — the same convention prose
already uses (a capitalized word mid-sentence is a name, ADR 0018) — so
the checker stays usable on markdown that names real-world things
angloform has no entry for.

This is intentionally a much lighter bar than prose linting, and it's the
right one for a title: it catches real vocabulary problems without
demanding a structure headings don't have.

## What stays untouched

`scripts/docjudge.py`'s own `paragraphs()` splitter does a different job —
it hands whole markdown paragraphs to LLM judges (`docs/prejudge.md`), who
read arbitrary markdown fine and don't need it de-markdowned first. Only
the strict-grammar path (lint-file.py, extract-sentences.py) needed the
shared classifier.

## The permanent example and its test

`tests/markdown-cases/kitchen-sink.md` is one file exercising every
construct above — frontmatter, a blockquote, code, a table, an
Enumeration, a Step Block, plain/task/nested lists (both the colon and
non-colon nested case), links, a standalone and an inline image, raw
HTML, footnotes, bold/italic/strikethrough/inline code, and a horizontal
rule. `scripts/test-mdblocks.py` (`./scripts/check.sh` runs it, so it's a
real CI gate, not a script someone has to remember to run by hand) checks
three things against it: every construct lands in the block kind it
should and none leaks into another block's text; the real angloform
sentences the file deliberately includes still parse clean through the
actual grammar; the deliberately-not-angloform content (an unknown word, a
missing Enumeration colon, a bare capitalized word opening a sentence) is
still correctly rejected with a real reason, proving the linter runs on
this content rather than everything just being silently excluded.

## Regression baseline

Every existing ADR (0001–0037) still lints at 100% sentence-parse — after
both passes below — confirmed by rerunning `just lint-file` across all of
them and `just replay`/the corpus sweep/`just coherence`, not just
spot-checked.

Built in two passes, both driven by synthetic torture-test files rather
than guessing which constructs mattered:

**Pass one** (tables, a blockquote, a fenced code block, links, plain and
task lists, bold/italic/inline code) found and fixed three real bugs:
fenced code content leaking into a garbage pseudo-sentence, a
`Status:`/`Date:` line that wraps across two physical lines losing its
continuation to the next paragraph, and a multi-line Enumeration/Step
Block unit that fails linting had its diagnosis detail misattributed to
the unit's own second line — a latent bug that predates this file and
never surfaced because no committed ADR had a *failing* multi-line unit.

**Pass two**, prompted by "did every markdown feature actually get a
rule, or did some just get skipped" — a fair challenge, since the answer
at the end of pass one was no. It added the constructs pass one didn't
touch at all (YAML frontmatter, raw HTML blocks, footnote markers and
definitions, strikethrough) and fixed two designed-but-incomplete pieces:
a generic nested list under a parent that doesn't end in `:` was gluing
sibling sub-items into one run-on sentence instead of keeping them
separate (see "Block classification" above); a standalone image was
silently dropped instead of getting the same heading-shaped vocabulary
check its alt text deserves.
