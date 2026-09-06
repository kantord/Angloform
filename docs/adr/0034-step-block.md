# 0034 — Step Block: ordered steps in the shape of Gherkin

Date: 2026-09-03
Status: proposed (tentative). Second Block-level structure after ADR 0028.

## Context

The rewrite of the decisions turned every list of the steps into the prose.
The decision "0028" allows an item, but an item is a Noun Phrase. A step is a
clause, so a step is not an item of an Enumeration. The word "then" opens a
step in English, but the decision "0014" reserves the First Token of a
sentence for the Sentence Shape. The maintainer required an unambiguous
structure with lines. The Enumeration is one structure with lines. Gherkin
has a structure with lines, so a file of Gherkin must be a document of
Angloform.

## Decision

A Step Block is a Block in the shape of Gherkin. A keyword opens every line
of a Step Block. The language has 4 keywords:
- "Given"
- "When"
- "Then"
- "And"

The keyword "Given" marks a precondition, and the keyword "When" marks an
event. The keyword "Then" marks a result, and the keyword "And" marks a
continuation of the prior keyword. Every line holds one clause. The clause
does not contain a Coordination. The clauses of a Conditional follow the
identical rule.

A header opens a Step Block. Gherkin has 2 headers:
- "Feature:"
- "Scenario:"

A header carries a title, and the Linter lints the title. The title is a
sentence of Angloform or is a quoted Name.

The word "Then" opens a line inside a Step Block. The word "then" does not
open a sentence of the prose, so the decision "0014" keeps the rule. The
keyword "But" of Gherkin is a Ban, because the word "but" joins 2 clauses
inside one sentence.

Every tool reads a Step Block. The Block is one unit, so the corpus keeps
the lines of a Block. 3 tools keep the lines of a Block:
- "lint-file"
- the extractor
- the repair of the paragraphs

A file of Gherkin is a document, so the tool "lint-file" lints the file.

## Consequences

- The steps of a procedure have a shape, so the language can say a
  procedure.
- The Linter parses every file of Gherkin in the folder "features". The test
  "feature_files_parse" checks the folder.
- If the word "then" opens a sentence of the prose, then the Linter names
  the home of the word "then".
- A reader sees the shape of a Block, but a spoken rendering needs a
  different shape. The file "docs/ideas.md" records the idea.
