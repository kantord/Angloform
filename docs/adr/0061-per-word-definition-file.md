# 0061 — The definition of a word: a new file per Lemma

Date: 2026-09-09
Status: proposed. Extends ADR 0001.

## Context

The Seed does not hold a definition of a word. The maintainers checked 75
words in the file "docs/lexicon-authoring-format-2026-09-07.md". The test
found a pattern for a definition of a noun. The test found a pattern for
a definition of a verb. The test found a pattern for a definition of an
adjective.

The file "seed/seed.json" holds every entry of the Seed. A determiner
does not need a definition. A conjunction does not need a definition. The
Category "BANNED" does not need a definition. A definition does not match
every Category of the Seed.

## Decision

The maintainers split the Seed. The file "seed/seed.json" keeps every
entry of a Category without a definition. Some Categories have a
definition. An entry of the Category gets a new file. The word "file"
gets the file "seed/definitions/file.yaml".

An entry of the new file can hold 5 fields:
- the Category
- the irregular Surface Forms
- the definition
- the antonym
- the Rejected Senses

The definition is a sentence. The Category picks the pattern of the
sentence. A Redirect of a Rejected Sense names one word. The word already
has a definition, so the entry does not repeat the definition of the word.
A Rejected Sense has the advice without a Redirect. The advice is a
sentence.

The file "seed/definitions.schema.json" names every field. The tool
Lexgen checks every entry of "seed/definitions/" against the file. The
tool Lexgen checks the definition against the Grammar. The tool Lexgen
checks the advice against the Grammar.

## Consequences

- A definition of a word becomes a real field of the Seed, and the Linter
  can show the definition to a writer.
- The file "seed/seed.json" exists after the decision "0061". The file
  "seed/definitions/" exists after the decision "0061". The maintainers
  move one entry to the new file.
- The decision "0061" names the shape of the new file. The decision
  "0061" does not name a time for every entry of "seed/seed.json".
