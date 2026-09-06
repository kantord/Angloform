# 0058 — A bare digit as a measurement value

Date: 2026-09-06
Status: proposed (tentative). Answers one question of ADR 0022's deferral.

## Context

The decision "0022" deferred the measurement. The sentence "the exit code
is 0" has a measurement. A measurement is a value, and a measurement does
not count things.

The digit "0" is a Ban in the position of a quantity. The digit "0" is
the future home of a measurement. The maintainers deferred 6 questions in
the decision "0022".

## Decision

The Lexer produces a new token for the digit "0". The token has the Form
Tag "NUM_VAL". The Grammar keeps the Ban of a quantity for the digit "0",
because the Grammar does not use the token "NUM_VAL" in the position of a
quantity.

The Grammar gains one alternative of the rule "Compl". A value stays
inside the Complement. A value can take the word "about". The sentence
"the value is 3" is one example. The sentence "the value is 0" is a
second example. The sentence "the value is about 4" is a third example.

The Linter keeps the Redirect of the decision "0022" for the digit "0".
The Redirect names the word "no" for the subject. The Redirect names the
Negation for the object. The old Ban of the Lexer becomes a Ban of the
Linter, because the Grammar rejects the digit "0" in the position of a
quantity.

The decision "0058" answers one question of the 6 questions.

## Consequences

- The maintainers can revert the rewrite of a sentence with a bare
  value.
- The digit "0" cannot enter the position of a quantity, because the
  digit "0" does not have the Form Tag of the quantity. The Ban of the
  phrase "0 files" stays.
- The decision "0022" defers 3 questions:
  - the units
  - the separators
  - the negative numbers
