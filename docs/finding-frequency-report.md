# Finding frequency — real-usage signal for which antiparsers to build next

Not a coverage target (see docs/STATUS.md, "EWT triage numbers are telemetry, not targets"); this measures which *rejection explanations* fire, not how much of English parses.

## Near-miss angloform (tests/paragraph-cases + tests/agent-cases)

Real repair-attempt proposals/outputs — the higher-relevance source — 6643 sentences.

### Outcomes

- Ambiguous: 91 (1.4%)
- Clean: 2528 (38.1%)
- Style: 1284 (19.3%)
- Unknown: 70 (1.1%)
- Word: 2670 (40.2%)

### The generic fallback ("restructure into one of the angloform templates")

Fired 28 times.

- i refers to the speaker
- you refers to the hearer
- "triage" checked the corpora "ud-ewt"
- A categorical Ban overshoots the target
- The bounds of the load are the constraint, because a categorical rule against a Sentence Shape overshoots
- The primary tool is the set of valid Sentence Shapes
- A sentence must need a small prior context
- The first tool is the set of valid Sentence Shapes
- A sentence must need a small prior context
- The primary tool is the set of valid Sentence Shapes
- The repetition of nouns reduces the Context Need
- A sentence needs a small prior text
- A sentence must need a small prior context
- the rule fails the criterion
- the rule fails every criterion
- the rule fails the criterion
- Every rule repeats the nouns or adds "then"
- The rule fails the criterion
- The rule fails the criterion
- the rule fails the criterion
- the rule fails the criterion
- the rule fails the criterion
- the rule fails the criterion
- a rule does not remove an ambiguity, so the rule fails the criterion
- the rule fails the criterion
- Every future Grammar ADR must justify the choice
- Every future Grammar ADR justifies the choice
- A Grammar ADR is a document

### STYLE finding kinds, ranked (quoted words normalized to `X`)

- 480 — X — a singular noun needs a determiner: X (mass nouns take X)
  example: the declarations list contains the words i and you and my and your
- 273 — X — noun-noun compounds are not angloform; write X, or one transparent word (ADR 0015)
  example: the system does not have an anaphora mechanism
- 121 — X is a defined term — write X (see CONTEXT.md)
  example: the system does not have plans for an anaphora or a discourse layer
- 114 — this is an inline list — write an Enumeration block: a statement ending in X whose last noun phrase is plural or counted, then one X per line (ADR 0028)
  example: the allowed words include i, you, my, and your
- 108 — X is transitive in angloform and needs an object
  example: the system does not have plans for an anaphora or a discourse layer
- 97 — a comma before X is mandatory when a new clause follows — X; no comma when X only joins a predicate under the same subject (ADR 0037)
  example: the declarations list contains the words i and you and my and your
- 76 — X is a verb in angloform and cannot follow a determiner
  example: Angloform is the name of the language
- 76 — noun phrases cannot be coordinated — write the colon-list: X (ADR 0041), or split the sentence (ADR 0004). Repeating the verb does not help here: the same verb twice is itself banned (ADR 0048)
  example: A Copula uses "is not" or "are not"
- 69 — X is a verb in angloform — as a noun use X
  example: the cost is consistent with the clarity of the project
- 52 — X — a Name takes no determiner: write X, or introduce it with a noun: X (ADR 0018)
  example: The Linter can lint a Negated statement
- 52 — X — a verb form cannot modify a noun; say who does it: X, or split the sentence
  example: a note describes the allowed words
- 51 — X is subject-only — write X instead (ADR 0014)
  example: the repository has no anaphora
- 45 — X is an adjective — add the noun it describes: X
  example: my is a possessive
- 41 — X — a verb cannot take an object and an adjective; use a verb that carries the result (X), or 2 sentences
  example: The second criterion keeps the loss small
- 32 — X needs the standard: X (ADR 0030)
  example: A rule forces longer text
- 32 — X — an adjective cannot take a prepositional phrase yet; restructure with a verb, or split the sentence (deferred, ADR 0023)
  example: the cost is consistent with the clarity of the project
- 28 — this structure is outside the sanctioned sentence shapes — restructure into one of the angloform templates
  example: i refers to the speaker
- 26 — X — only X attaches to a noun; X attaches to the verb. Write X, or move the phrase after the verb (ADR 0011)
  example: a mechanism for the anaphora does not appear
- 25 — X — comparatives of a quantity are deferred (ADR 0030); write X, or restructure
  example: Every rule removes more ambiguity than the added length costs
- 25 — passive (X) is not in angloform — name the doer and use active voice (ADR 0003)
  example: the prose is accepted
- 23 — X is a verb form in angloform and cannot follow a determiner — name the thing with a noun
  example: the agent needs a discourse layer for the finding of the referent of the anaphora
- 23 — a comma cannot join 2 clauses — write 2 sentences, or X / X (ADR 0026)
  example: the prose is repetitive, the agent mentions the agent
- 22 — X negates the verb only — write X or X; a noun phrase cannot carry X (ADR 0005)
  example: the design of the project chooses clarity and not naturalness
- 21 — X used as a word must be quoted: Xyour\X (ADR 0018)
  example: the allowed words include i, you, my, and your
- 20 — an adjective cannot take a prepositional phrase yet (X); restructure with a verb, or split the sentence (deferred, ADR 0023)
  example: The text must be cheap to process
- 16 — X exists only before digits (X); for a topic write X: X, or restructure (ADR 0025)
  example: The Linter must tell the writer about the ban
- 15 — X — an adjective cannot modify a Name; write X alone, or X (ADR 0018)
  example: The future "validator" must check the message
- 15 — X — the name follows its noun: X (ADR 0018)
  example: i did not "design a" mechanism for the discourse
- 14 — X used as a word must be quoted: Xmy\X (ADR 0018)
  example: my is a possessive
- 12 — X — a word mentioned as a word must be quoted: Xthe\X (ADR 0018)
  example: The project does not value the naturalness
- 11 — X is not a Pronoun — repeat the noun: X (ADR 0016)
  example: If 2 valid formulations tie on the load, the density, and the Context Need, then the Grammar prefers the more expressive one
- 10 — X used as a word must be quoted: Xi\X (ADR 0018)
  example: the sentence allows the word i
- 10 — X — a word mentioned as a word must be quoted: Xmy\X (ADR 0018)
  example: the tool Lexgen allows the adjective my
- 10 — X — the copula takes an adjective or a noun phrase, not a prepositional phrase; use a verb: X (ADR 0003)
  example: the mechanism is for anaphora
- 10 — phrases cannot be coordinated — split the sentence, one phrase each (ADR 0004)
  example: the Copula uses "not" after "is" or after "were"
- 9 — X needs its noun: X (ADR 0029)
  example: The second measure is the Dependency Length
- 9 — X — X cannot follow X; write X, or make the every-phrase the subject (ADR 0014)
  example: The language bans the possessive of every Anaphoric Pronoun
- 9 — a number needs its noun: X; a bare value is deferred (ADR 0022)
  example: the count is about 2200
- 8 — X is scope-ambiguous — for none write X; for not-all write X (ADR 0014)
  example: every agent must not check the input
- 8 — X used as a word must be quoted: Xyou\X (ADR 0018)
  example: the sentence allows the word you
- 8 — X — a word mentioned as a word must be quoted: Xyour\X (ADR 0018)
  example: the tool Lexgen allows the adjective your
- 7 — X — a word mentioned as a word must be quoted: Xi\X (ADR 0018)
  example: the sentence allows the word i
- 7 — X — a word mentioned as a word must be quoted: Xyou\X (ADR 0018)
  example: the sentence allows the word you
- 7 — [AntiClauseObject] X — X starts a second clause after X; a clause cannot be the object of a verb. State the fact in its own sentence, or give it a Name and use the Name here
  example: the result says the ambiguity reappears
- 7 — condition must come first — write: if <clause>, then <clause> (ADR 0007)
  example: the structure is: if a sentence splits, then the reference ambiguity reappears
- 6 — X repeats across the coordination — write the colon-list instead: X (ADR 0041, ADR 0048)
  example: the Linter bans Anaphoric Pronouns and bans demonstratives
- 6 — X — X does not chain; name the inner thing in its own sentence, or drop one level (ADR 0011)
  example: the agent needs a discourse layer for the finding of the referent of the anaphora
- 6 — X — a word mentioned as a word must be quoted: Xa\X (ADR 0018)
  example: A rule can force a text longer than the standard
- 6 — line 1 (X): a step is one clause with no coordination — unexpected NounPl(X) at word 2 — no sanctioned sentence structure continues this way
  example: When a rule forces a longer text, the text must earn the length
- 4 — X is a noun in angloform — as a verb use X
  example: i did not design a discourse mechanism
- 4 — X — a verb takes one object; write X (no ditransitives)
  example: The Sentence Shapes give every sentence a Parse
- 2 — X is ambiguous — for prohibition write, for example, X (bare plural + must not, ADR 0014)
  example: no agent must check the input
- 2 — X opens a line only inside a Step Block (Given / When / Then / And lines, ADR 0034); in prose write the step as a plain sentence
  example: then ambiguity reappears
- 2 — X — a Name follows a noun only after X or X; write X or restructure (ADR 0018)
  example: The Sentence Shapes give every sentence "a" Parse
- 2 — X — a verb form cannot be the subject; name the doer: X
  example: Resolving the Anaphoric Pronoun requires a Discourse Layer
- 2 — a comma before X is mandatory — X (ADR 0026)
  example: The Linter cannot parse the sentence because the word is not in the Lexicon
- 1 — X exists only after a modal: X, X; elsewhere write X or X (ADR 0032)
  example: the team does not find the cost to be a problem
- 1 — X — a count needs its noun: X; for a share write X (ADR 0022, 0024)
  example: Indexical Pronouns are about 2200 of the unknown tokens
- 1 — X — a word mentioned as a word must be quoted: Xif\X (ADR 0018)
  example: The Grammar prefers the expressive formulation if 2 valid formulations tie on the load, the density, and the Context Need
- 1 — X — a word mentioned as a word must be quoted: Xthen\X (ADR 0018)
  example: the Linter then uses the Grammar
- 1 — X — a/an take a singular noun
  example: the system does not produce an ambiguities
- 1 — X — no adverbs; name the time or the way with a phrase: X, X
  example: the Grammar arrives later
- 1 — X — write X (short adjectives inflect, ADR 0030)
  example: If a choice between a more natural version and a more easy version conflicts, then the more easy version wins, but criterion 2 keeps the loss small

## Real English, for comparison (data/ud/en_ewt-ud-test.conllu)

Not curated angloform at all — most of it fails at the WORD level first — 2077 sentences.

### Outcomes

- Clean: 5 (0.2%)
- Style: 27 (1.3%)
- Unknown: 18 (0.9%)
- Word: 2027 (97.6%)

### The generic fallback ("restructure into one of the angloform templates")

Fired 1 times.

- STAY AWAY

### STYLE finding kinds, ranked (quoted words normalized to `X`)

- 8 — X is transitive in angloform and needs an object
  example: Compare the flags to the Fallujah one.
- 7 — X — a Name takes no determiner: write X, or introduce it with a noun: X (ADR 0018)
  example: Compare the flags to the Fallujah one.
- 3 — X — an adjective cannot modify a Name; write X alone, or X (ADR 0018)
  example: Original Margin Call Margin Due Today
- 2 — X is a verb in angloform and cannot follow a determiner
  example: Compare the flags to the Fallujah one.
- 2 — X needs its noun: X (ADR 0029)
  example: First Union Securities, Inc.
- 2 — X — a singular noun needs a determiner: X (mass nouns take X)
  example: frame
- 2 — a comma before X is mandatory — X (ADR 0026)
  example: Not so good
- 1 — X exists only before digits (X); for a topic write X: X, or restructure (ADR 0025)
  example: ~CGoehring
- 1 — X is an adjective — add the noun it describes: X
  example: my bad.
- 1 — X negates the verb only — write X or X; a noun phrase cannot carry X (ADR 0005)
  example: But not so.
- 1 — X — a verb form cannot be the subject; name the doer: X
  example: Winning Attorney!
- 1 — this structure is outside the sanctioned sentence shapes — restructure into one of the angloform templates
  example: STAY AWAY

