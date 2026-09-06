# 0004 — Coordination: and/or enabled, but deferred

Date: 2026-08-31
Status: proposed (tentative — same review point as ADR 0002/0003; rewritten in angloform 2026-09-02)

## Context

Triage counts about 550 tokens of the word "and" and counts about 77
tokens of the word "or" in the corpus. The Lexicon does not contain the 2
Function Words, so the 2 words hurt the Coverage.

The scope of a Coordination causes a bad Scope Ambiguity in English. The
ambiguity has 2 examples:
- "old men and women"
- "the sensor and the valve in the cabinet"

The Grammar decides the scope of a Coordination, so the 2 words are not
ambiguous. The language needs the 2 words.

Triage counts about 82 tokens of the word "but", but the word "but" marks a
contrast. A contrast is a meaning of the discourse, so the meaning of "but"
is bigger than a claim. The word "but" is the first word with a meaning of
the discourse.

## Decision

The language enables 2 conjunctions:
- "and"
- "or"

The Category of the 2 conjunctions is "CONJ". The maintainers deferred the
word "but" and deferred every different conjunction. The later decision
"0021" enabled the word "but".

The decision records 3 rules for a future Grammar, so the Linter cannot
enforce the rules before the Grammar. The first rule limits a Coordination
to phrases of one Category. The second rule keeps a Modifier inside one
Conjunct, so the writer repeats the Modifiers for every Conjunct. The
language allows the phrase "the old files and the old reports" and bans the
phrase "the old files and reports". The third rule keeps a Prepositional
Phrase inside one Conjunct, so the writer repeats the Prepositional Phrase.

## Consequences

- The language can say a compound statement and can say an alternative.
- The second rule adds words to a text, because the writer repeats the
  Modifiers. The writer pays the words for a clear scope. The second rule
  matches the Ban of the Anaphoric Pronouns, because the writer repeats the
  noun for the Ban.
- Triage accepts every token of the 2 words, because Triage does not see the
  scope. No tool can measure the 3 rules before the Grammar.
