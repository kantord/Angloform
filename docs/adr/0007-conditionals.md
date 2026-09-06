# 0007 — Conditionals: if … , then … (one template, condition first)

Date: 2026-08-31
Status: proposed (tentative — same review point as ADR 0002–0005; rewritten in angloform 2026-09-02)

## Context

Triage counts about 87 tokens of the word "if" in the corpus. Instructions
need Conditionals. The instruction "if the test fails, then the agent
retries the request" is a Conditional. The initial design of the language
included the Conditional.

A bare Conditional has an ambiguous boundary and has an ambiguous
attachment. English allows the consequent at the front of the sentence. The
sentence "the agent retries the request if the test fails" puts the
condition after the consequent, so the reader holds the consequent until
the condition.

## Decision

The language adds 2 Function Words:
- "if"
- "then"

The word "if" has the Form Tag "SCONJ_COND", and the word "then" has the
Form Tag "THEN".

The Conditional has one Sentence Shape. The Sentence Shape "if <clause>,
then <clause>" puts the condition at the front of the sentence, so the
reader has the context before the consequent. The condition is one clause,
so the reader holds one clause before the consequent.

The Conditional needs the comma and needs the word "then". The 2 markers
show the boundary of the consequent, so the reader does not follow a wrong
Parse at the seam of the 2 clauses.

The consequent cannot start the sentence, and the condition cannot contain a
nested Conditional.

The language does not allow 4 words:
- "unless"
- "when"
- "whenever"
- "else"

## Consequences

- The language can say a rule and can say a reactive instruction in one
  unambiguous Sentence Shape.
- A writer must restructure the sentence "X if Y" into the sentence "if Y,
  then X". The rewrite is mechanical, so a future validator can suggest the
  rewrite.
- Triage accepts every token of a Conditional, because Triage does not see
  the Sentence Shape. The Grammar enforces the Sentence Shape.
