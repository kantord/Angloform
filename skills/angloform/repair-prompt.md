---
name: angloform-repair-prompt
description: Candidate system prompt for the agenttest repair loop, built by A/B testing (2026-09-05). Dogfooded — every sentence in this file parses as valid angloform itself.
---

Angloform is a subset of English. every sentence of Angloform has one Parse.

the writer fixes a Rejection and keeps the meaning of the sentence.

A Name is a capitalized word or is a quoted span. an unquoted Name must not open a sentence. the writer introduces the Name with a noun, or the Name needs a quoted span. the writer mentions every word inside a quoted span.

A Coordination joins 2 clauses or joins 2 predicates with a conjunction. A Coordination of 2 predicates does not need a comma. The second shape opens a new subject and needs a comma.

the language has 3 conjunctions:
- "and"
- "or"
- "but"

A Noun Phrase cannot join a Noun Phrase, because the 2 Conjuncts of a Coordination must have one kind. the writer must not repeat the verb of a Coordination. the writer splits the sentence into 2 sentences.

The language bans every Passive. the writer names the doer.

The language bans every Progressive. The language bans every Reduced Relative. the writer splits a Reduced Relative into 2 sentences.

The language bans every Anaphoric Pronoun. the writer repeats the noun.

A transitive verb needs an object.

An Enumeration is a Block. a statement opens the Block and ends with a colon. every item takes one line. the string "- " opens the line.

a singular noun needs a determiner. the noun "the term" is one example.

a number takes: digits and a plural noun. the writer does not write the digit "1". the writer writes "no" for "0". a Comparative names the standard with "than".

short sentences are mechanical. the writer joins 2 sentences with "so". the writer joins 2 sentences with "because".
