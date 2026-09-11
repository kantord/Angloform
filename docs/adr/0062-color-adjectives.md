# 0062 — Color adjectives, and the noun phrase "the color X"

Date: 2026-09-12
Status: proposed. Extends ADR 0001.

## Context

A color is a common adjective, but the Lexicon had no color before this
ADR. A basic color has no synonym and no genus: "red" cannot borrow the
definition pattern of an ordinary adjective ("huge: big"), because no
already-enabled word names the same shade. A speaker also needs a way to
name a color itself, not only to describe a thing with it: "the color
red", not only "the red file".

## Decision

The maintainers add a Category: "COLOR_ADJ". A word of the Category
"COLOR_ADJ" gets its own Form Tag, but composes into every existing
adjective construction: the attributive position ("the red file"), the
predicate position ("the file is red"), and every other position the
Category "ADJ" already reaches. The dedicated Form Tag lets the Grammar
require a color specifically in one new construction, instead of
accepting any adjective there.

The maintainers add one more Category, "COLOR", for the single word
"color". This word starts a new Noun Phrase. The color adjective after it
is optional, so 3 shapes all parse: "the color" alone, "the color red",
and "the color of the file" (an of-PP names the color instead).

A basic color's definition reuses this same Noun Phrase, instead of the
usual genus-and-differentia pattern: "red" is defined as "the color of a
blood". No other Category may use this pattern — only "COLOR_ADJ" points
to a Noun Phrase built around the word "color" as its own definition.

## Consequences

11 basic colors and 7 prototype nouns (blood, fire, grass, sky, grape,
wood, stone) enter the Lexicon together, each prototype needed by at
least one color's definition. Only "pink" needed no new prototype: its
definition is "red" itself, a genuine shade of it (WordNet: "a light
shade of red"). Every color gets one Rejected Sense per competing
WordNet category, mostly a Gap — no word already covers "the color of
clothing worn for mourning", the competing noun sense of "black".
