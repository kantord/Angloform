# 0003 — Copula: is/are only, present tense, no passive/progressive

Date: 2026-08-31
Status: proposed (tentative — same review point as ADR 0002; rewritten in angloform 2026-09-02)

## Context

Triage counts about 850 tokens of the word "be" in the corpus, so the word
"be" is the main hole in the Coverage. Texts cannot avoid the Copula, and
the sentence "the file is old" is one example.

The whole Paradigm of "be" adds 4 Surface Forms:
- "was"
- "were"
- "been"
- "being"

The 2 past Surface Forms mark the Tense, and the 2 Participles mark the
aspect. If a Participle follows a Copula, then the sentence is not a plain
statement. The phrase "is stored" is a Passive, and the phrase "is running"
is a Progressive. The maintainers deferred 4 questions of the Grammar:
- the Tense
- the aspect
- the Passive
- the Progressive

The research of the project found the main danger in the Participles.
The Reduced Relative is one example.

## Decision

The language enables 2 Copulas:
- "is"
- "are"

The Copula "is" carries the Form Tag "COPULA_SG", and the Copula "are"
carries the Form Tag "COPULA_PL". The 2 Copulas have a fiat Category. A
Copula takes a Complement but does not take an object. The slot of a Copula
differs from the slot of a verb, so the Category of a Copula is not the
Category of a verb.

A Complement is an adjective or is a Noun Phrase. A Participle cannot be a
Complement. The first version of the language does not allow the Passive
and does not allow the Progressive. The maintainers plan the rule of the
Complement for the Grammar. No tool enforces the rule before the Grammar.

The language does not enable 4 Surface Forms of "be":
- "be"
- "am"
- "been"
- "being"

The decision "0010" added "was" and added "were". The decision "0032"
allowed the phrase "must be".

## Consequences

- The language can describe a property and can name a class. The sentence
  "the queue is empty" is the first example. The sentence "the parser is
  a program" is the second example.
- The first version of the language cannot say a past state and cannot say
  an ongoing process. The maintainers revisit the question in the decision
  of the Tense.
- The rule of the Complement is a rule of the documents before the Grammar.
  Triage checks tokens, so Triage cannot see a construction. Triage accepts
  every token of a Copula.
