# 0026 — so / because: two causal connectives, two information structures

Date: 2026-09-01
Status: proposed (tentative)

## Context

The rewrite of the decision "0001" dropped the phrase "hence this ADR",
because the language did not have a causal connective. The decision "0012"
does not allow a Propositional Loss. A causal relation is a claim, so the
rewrite had a Propositional Loss. The decisions have about 11 tokens of the
word "so" and have about 7 tokens of the word "because". The corpus "UD-EWT"
has about 17 tokens of the word "because" and has about 15 tokens of the
word "since". The word "since" marks the time in a big share of the tokens.
The corpus "UD-EWT" has about 60 tokens of the word "so". The tokens of "so"
have different senses.

English offers 2 orders. The phrase "A, so B" puts the reason at the front,
and the phrase "because A, B" puts the reason at the front. The phrase "B
because A" puts the result at the front. The 2 orders differ in the
structure of the information. One clause is old, and one clause is new. The
order "old before new" helps the comprehension. The decision "0006" adopts
the order. The common advice of the style adopts the order. If the result
sits at the front, then the sentence can have an ambiguous attachment. The
phrase "B because A and C" is one example.

The decision "0007" fixed one order for the Conditional. The decision "0007"
is not a precedent for a causal sentence, because the 2 Sentence Shapes
differ. If the consequent of a Conditional sits at the front, then the
reader holds the consequent in the memory. The consequent is a hypothesis,
so the reader cannot judge the consequent before the condition. A causal
sentence asserts the 2 clauses, so the reader does not hold a hypothesis in
the memory.

## Decision

The language has 2 causal constructions:
- the phrase "<clause>, so <clause>"
- the phrase "<clause>, because <clause>"

Every construction has one fiat meaning. A causal sentence needs the comma
at the seam. The word "so" marks a result. The reason is old, and the result
is new. The word "because" marks a reason. The result is old, and the reason
is new. A clause of a causal sentence does not contain a Coordination, so
the Grammar fixes the scope of the connective. The Conditional has the rule.

The 2 connectives cannot open a sentence, because the First Token of a
sentence announces the Sentence Shape. The decision "0014" names the rule.
If the reason sits in a prior sentence, then the writer merges the 2
sentences. A causal verb is the alternative and puts the topic at the front.
The sentence "the expense explains the decision" uses the causal verb
"explain".

The phrase "so big" marks a degree, and the phrase "so that" marks a purpose.
The Grammar gives one sense to the word "so", so the language cannot say the
2 phrases. The language bans 5 words:
- "since"
- "hence"
- "therefore"
- "thus"
- "as"

The Linter explains the Ban and names the 2 constructions.

The maintainers deferred 4 questions. The first question is an Imperative
inside a causal sentence. The second question is a Conditional inside a
causal sentence. The third question is the causal chain "A, so B, so C". The
4th question is the purpose "in order to".

## Consequences

- The decision "0001" says the causal claim with the sentence "the expense
  of a reversal grows with the seed, so the maintainers record the
  decision".
- The Linter names 3 repairs. The first repair adds the comma at the seam.
  The second repair moves the word "because" from the front. The third
  repair moves the word "so" from the front.
- The correct connective depends on the prior sentence, so the Linter cannot
  check the choice in one sentence. The tool "lint-file" measures the
  continuity of the topic in the whole document, so the tool "lint-file" can
  check the choice.
