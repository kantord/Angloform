# 0006 — Syntax principle: comprehension-first branching, minimal context need

Date: 2026-08-31
Status: accepted (governing principle for all grammar-tier decisions; rewritten in angloform 2026-09-02)

## Context

The decisions of the Grammar need a rule, because 2 structures can tie.
The file `docs/research/cnl-design-findings.md` records the research. The
research grounds the difficulty of a sentence in the Cognitive Load. The
Cognitive Load has 3 metrics:
- the Open Dependencies
- the Dependency Length
- the Embedding Depth

The comfortable bound is 4 Open Dependencies. The 3 metrics predict the
difficulty of a sentence and beat every rule of the style.

A categorical Ban of a shape overshoots. One sentence of the research
carried a heavy branch at the front, but the sentence was effortless. The
load of the sentence was small, so the correct constraint is a bound on
the load. A Ban of a shape is the wrong constraint.

The profile of a person matches the profile of a Language Model. The 2
readers integrate a sentence in the order of the words and have a small
memory.

## Decision

The maintainers weigh 5 criteria for every decision of the syntax. A
criterion outranks every later criterion.

The first criterion is the Cognitive Load. The maintainers prefer the
structure with a small Cognitive Load. The rules of the branching help a
person and help a Language Model. The results of the science ground the
rules, so the rules do not follow a taste. A tradition does not ground a
rule.

The second criterion is the familiarity. If the ideal structure is not
available, then the maintainers choose the common construction of
English. A familiar construction helps the reader, but a surprise costs the
reader.

The third criterion is the Context Need. A sentence must have a small
Context Need, so the language bans the Anaphoric Pronouns. The criterion
explains the repetition of the nouns. The Grammar pins the scope of a
word, because the scope must not depend on the context.

The 4th criterion is the density. The target is a dense text. A dense text
carries a precise meaning and is cheap. The maintainers prefer the short
formulation inside the bounds of the load. A verbose text is not free,
because the noise of the padding causes an ambiguity. If a rule adds
words, then the rule must remove an ambiguity. The repetition of the nouns
earns the words, and the mandatory word "then" earns the word. If a rule
adds the padding, then the rule violates the 4th criterion.

The 5th criterion is the expressiveness. A natural text is a real goal. A
text of Angloform must be pleasant and must not be robotic. The 4 prior
criteria outrank the expressiveness. The comparison of 2 valid
formulations uses 3 criteria:
- the Cognitive Load
- the density
- the Context Need

If the 2 formulations tie, then the maintainers choose the expressive
formulation. The maintainers do not trade the precision for the charisma.
The clarity outranks the charisma.

A constraint is a bound on the load. A constraint is not a Ban of a shape. The
Grammar limits the load at the front of a sentence and does not ban a
shape. If a rule requires a long text, then the rule must earn the words.
The length is cheaper than the load but is not free.

The Sentence Shapes are the primary tool. The Sentence Shapes put the head
at the front of a sentence, so a bad shape does not exist in the language.
The Grammar excludes a bad shape and does not weigh the bad shape. The
score of a sentence is the secondary tool. The score verifies the claim of
the Sentence Shapes and monitors the load of a text. The score gathers the
evidence for a future bound but does not protect the readability of a
text.

## Consequences

- The maintainers must justify every future decision of the Grammar
  against the 5 criteria. The order of the criteria is strict.
- The research defines the 3 metrics of the Cognitive Load. A future tool
  can measure the 3 metrics on the Parse of a sentence.
- If a natural formulation conflicts with an easy formulation, then the
  easy formulation wins. The second criterion limits the loss of the
  naturalness.
