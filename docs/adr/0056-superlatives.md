# 0056 — Superlatives: an inflected or "most"-marked adjective before a noun

Date: 2026-09-06
Status: proposed (tentative). Closes a deferral of ADR 0029 and ADR 0030.

## Context

The decision "0029" deferred the superlative. The decision "0030" deferred
the superlative. A rewrite of the corpus dropped a claim, because the
Grammar did not have a shape for the superlative.

The decision "0029" added the Ordinal. The Ordinal follows a determiner
and precedes a noun. The Ordinal takes a Prepositional Phrase. The
Prepositional Phrase names the set of the Ordinal.

The decision "0030" made 2 classes of the adjective. A short adjective
inflects, and a long adjective takes the word "more".

## Decision

The decision "0056" repeats the rule of the decision "0030" for the
superlative. A short adjective gains a Surface Form of the superlative. A
long adjective does not gain the Surface Form.

The Grammar gains 2 alternatives of the rule "NP". The decision "0029"
added the Ordinal in the identical position. The sentence "the mechanism
deletes the biggest file" is one example. The sentence "the mechanism
deletes the most transparent file" is a second example.

The superlative takes the identical Prepositional Phrase. The sentence
"the mechanism deletes the biggest file of the 3 files" is a third
example.

The word "most" is not a Ban. The word "most" becomes a Function Word.
The word "shortest" was a Ban, and the decision "0056" removes the Ban.

## Consequences

- Lexgen gains the rule of one inflection. The rule adds the suffix "est"
  to a short adjective.
- The maintainers checked 9 generated words against the data. The Seed
  gains new entries. The entries name the exceptions of the general rule.
- The maintainers can revert the rewrite of a sentence with a claim.
