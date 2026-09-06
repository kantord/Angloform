# 0002 — Pronouns: third person banned, indexicals allowed

Date: 2026-08-31
Status: proposed (tentative — revisit once real corpus experience exists; rewritten in angloform 2026-09-02, see docs/dogfood-adr-0002.md)

## Context

Triage counted the unknown tokens of the corpus "UD-EWT". The file
"docs/triage-report.md" records the result. About 2200 unknown tokens are
Pronouns, so the Pronouns are a big Gap in the Coverage.

Every Pronoun of the third person is an Anaphoric Pronoun. An Anaphoric
Pronoun refers to a noun of a prior sentence, so the reader must find the
noun. The sentence "it
failed" needs a Discourse Layer, because the sentence does not name the
noun of "it". A writer can split a long sentence into 2 sentences, but the
second sentence still needs the noun of the first sentence. The file
"docs/research/cnl-design-findings.md" describes the problem.

An Indexical Pronoun refers to the speaker or refers to the hearer. The
reference is not anaphoric, so an Indexical Pronoun does not cause a
Reference Ambiguity. Instructions use Indexical Pronouns.

## Decision

The language bans every Anaphoric Pronoun:
- "it"
- "its"
- "they"
- "them"
- "their"
- "he"
- "him"
- "his"
- "she"
- "her"
- "this"
- "that"
- "these"
- "those"

If a writer needs an Anaphoric Pronoun, then the writer repeats the noun.

The language allows 4 Indexical Pronouns:
- "I"
- "you"
- "my"
- "your"

Every word has one Form Tag:

| word | Form Tag |
|---|---|
| "I" | "PRON_1SG" |
| "you" | "PRON_2" |
| "my" | "POSS_1SG" |
| "your" | "POSS_2" |

The maintainers did not build a Discourse Layer for the initial version.

## Consequences

- The construction of the language removes every Reference Ambiguity, so
  the maintainers do not need a Discourse Layer.
- The prose repeats the nouns, so the prose loses the naturalness. The
  project accepts the loss, because the project prefers the clarity to
  the naturalness.
- The current Lexicon does not contain the Anaphoric Pronouns. The Linter
  finds an unknown word and does not explain the Ban. A writer needs the
  message "repeat the noun you mean", but the message needs a new
  mechanism. The maintainers deferred the mechanism.
