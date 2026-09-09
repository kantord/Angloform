# 0060 — Every Rejected Sense gives a reason: no silent Waiver

Date: 2026-09-09
Status: accepted (retroactive — documents a shipped change).

## Context

The entry of a word held the field "waive". The field "waive" rejected a
sense and did not name a Redirect. A writer used the sense. The writer did
not get a reason.

## Decision

The maintainers removed the field "waive" from the Seed. The entry of a
word can name the Redirect of a Rejected Sense. The entry of a word can
write the advice of a Rejected Sense. The advice is a sentence. The Linter
shows the sentence to a writer.

## Consequences

- Every Rejected Sense gives a reason to a writer.
- The file "CONTEXT.md" showed the old term "Waiver". The decision "0060"
  removes the term from the file "domain/model.json".
- A future entry of the Seed cannot record a silent Rejection. The tool
  Lexgen rejects the unknown field "waive".
