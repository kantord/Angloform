# Dogfood: ADR 0003 in angloform

Rewritten in place 2026-09-02 with no model run: `just lint-file` on the
source, then fix a gap or follow the advice per sentence. 34/34 units parse.

| Blocker | Choice | Result |
|---|---|---|
| grammar jargon (copula, complement, participle, passive, progressive, tense, reduced relative) | gap — seven terms with definitions in the domain model | Capitalized terms |
| "the maintainers did not decide 4 questions" as a list | Enumeration (ADR 0028); negated intros needed the harness to look through NegVP | fixed |
| "A Copula with a Participle …" (noun + with) | advice — restructured as a conditional | rewritten |
| superlatives ("single largest", "nastiest") | advice — declared as `ranking` drops | pairs |
| stale claim ("documentation-only until the grammar tier") | rewritten to the current facts | `update` drop |

Declared losses are in corpus/dogfood-pairs.tsv. The intro-line rule for
Enumerations (one line, own paragraph) was made explicit for the tools.
