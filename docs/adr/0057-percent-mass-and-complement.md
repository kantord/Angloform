# 0057 — Percent: a singular noun, and a bare share as a value

Date: 2026-09-06
Status: proposed (tentative). Closes 2 deferrals of ADR 0024.

## Context

The decision "0024" decided a share of a plural noun. The decision "0024"
deferred 3 questions. The first question covers a share of a singular
noun. The second question covers a share in the position of the
Complement. The third question covers the decimals.

The decision "0029" already answers the third question. A decimal is a
quantity in digits, and the decision "0029" already allows the digits
after the word "percent".

The decision "0057" answers the first question. The decision "0057"
answers the second question.

## Decision

The Grammar gains one alternative of the rule "NP". The share of a
singular noun takes the singular verb. The sentence "50 percent of the
file is the text" is one example.

The Grammar gains one alternative of the rule "Compl". A bare share is a
value. The value is not a Noun Phrase of a set, so the Ban of the decision
"0024" does not cover the value. The sentence "the load is 43 percent" is
a second example.

The bare share stays inside the Complement. The Ban of the decision
"0024" stays for the subject and stays for the object.

## Consequences

- The maintainers can revert the rewrite of a sentence with a share of a
  singular noun.
- The maintainers can revert the rewrite of a sentence with a bare share.
- Every question of the decision "0024" is closed.
