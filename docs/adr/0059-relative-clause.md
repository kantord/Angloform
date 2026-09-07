# 0059 — A relative clause after a clause

Date: 2026-09-07
Status: proposed (tentative). Revises the decision "0010". A real test
did not confirm the naturalness claim of the decision — see the
Consequences.

## Context

The report "docs/judge-report.md" names a shape. The shape repeats a
Noun Phrase in the subject position of every sentence in a row. The
shape fails the most paragraphs of the report. The decision "0010"
banned every Reduced Relative. The decision "0010" named the risk (the
sentence "the file stored in the cache"). The old ban covered a
relative clause with a real relative word too, not only the Reduced
Relative.

## Decision

The Grammar allows a new sentence: a clause, then the word "which" or
the word "who", then a predicate. A comma opens the new sentence. The
word "that" stays banned. The word "that" already has 3 senses (the
decisions "0002" and "0010"). The word "who" names a human Noun Phrase.
The word "which" names every other Noun Phrase.

The predicate of the new sentence takes the subject role of the word
"which" or the word "who". No gap exists in the role.

The maintainers built a first version. The first version put the new
predicate inside the object of the verb. The first version caused a
real conflict of the tool "LALRPOP". The Grammar did not decide the
role of a comma with one token of the lookahead. The maintainers moved
the new sentence beside the decision "0037" and the decision "0026".
The move follows the process of the decision "0004": the Grammar
reduces the clause first, then a single token after the comma decides
the sentence.

The Grammar cannot verify 2 facts after the move. The Grammar cannot
verify the Noun Phrase before the comma. The Grammar cannot verify the
agreement of the predicate. The Linter must verify the 2 facts in a
future decision.

## Consequences

- A writer can attach a fact to a clause without a repeated subject.
- The Grammar cannot force the Noun Phrase before the comma into the
  object of the clause. A clause without an object keeps only the
  subject as the Noun Phrase before the comma. The gap needs a future
  Linter check.
- The Grammar cannot force the agreement of the predicate to the real
  number of the Noun Phrase before the comma. The gap needs a second
  future Linter check.
- A test compared 2 real paragraphs of the decision "0001" and the
  decision "0022". The test did not find 3 wins of 3 documents. The
  test found one tie and one loss. A judge named the reason for the
  loss: the new sentence made a claim about the wrong Noun Phrase.
- The maintainers keep the Grammar. The decision "0054" set a
  precedent: a real ability outranks a naturalness claim without a
  confirmation.
- A future decision can revisit the subject Gap and the agreement Gap,
  once the Linter checks exist.
