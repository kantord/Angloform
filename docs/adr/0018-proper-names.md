# 0018 — Proper names: capitalization mid-sentence, quotes for identifiers

Date: 2026-09-01
Status: proposed (tentative)

## Context

The Dogfood needs Names, because the documents of the project name things.
The documents name 4 things:
- the language Angloform
- the file "seed.json"
- the tool Lexgen
- the database WordNet

The Names are not a Closed Class, so the Lexicon is the wrong tool for
the Names. A quoted span can hold every Name but is heavier than the
convention of English.

## Decision

A Name has 2 shapes:
- a capitalized word
- a quoted span

A capitalized word is a Name. The rule follows the convention of English
but is strict. An unquoted Name must have a capital and must not open the
sentence. The lowercase of an unquoted Name must not be a word of the
Lexicon. If a capitalized word follows a capitalized word, then the Linter
merges the 2 words into one Name. If a capitalized word opens a sentence,
then the Linter checks the lowercase of the word. If the Lexicon has the
lowercase, then the Linter reads the lowercase. If the Lexicon does not
have the lowercase, then the Linter shows an error. The error suggests an
appositive or suggests a quoted span. The Linter does not turn a typo into
a Name and does not turn an unknown word into a Name. If a capitalized word
does not open the sentence, then the Linter checks the lowercase of the
word. If the Lexicon has the lowercase, then the Linter shows an error. The
error names a wrong capital or names a Collision. If a Name collides with a
word of the Lexicon, then the Name needs a quoted span. The word "I" is the
Pronoun. The word "I" is not a Name.

A quoted span holds an identifier. The identifier keeps the case and keeps
the spelling inside a quoted span. The file "seed.json" is one example. A
string of a program is a second example. A quoted span allows every
character. A quoted span is one opaque Noun Phrase. A quotation mentions a
phrase or mentions a sentence. A quotation is a different construction, so
the decision does not cover a quotation. A quotation needs a future design.
The file "docs/ideas.md" describes the future design. The future design
can parse a sentence inside the quoted span.

A Name is a singular Noun Phrase and can follow a noun in an appositive.
The phrase "the tool Lexgen" is one example. A Name does not inflect. A Name is
opaque, so the writer repeats the Name. The rule matches the decision
"0002", because the decision "0002" bans every Anaphoric Pronoun. The
metric gives a flat expense to every Name, because the frequency of a Name
is meaningless.

## Consequences

The language can describe 3 things:
- the language
- the files
- the tools

The Names unblock rows of the Dogfood.

A Name is not a miss of the Lexicon, so Triage must not count a Name
against the Coverage. The metric must give a flat expense to every Name.

If a future brand does not have a capital, then the brand cannot open a
sentence. The writer must give a capital to the brand or must introduce
the brand with a noun.
