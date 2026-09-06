# Dogfood: ADR 0002 in angloform

Rewritten in place 2026-09-02 after nine paragraph-repair runs (tests/
paragraph-cases/, docs/paragraph-report.md). All six model proposals were
judged needs-fix for unnatural wording; the runs' value was in what they
forced into the system, not in their text:

| Run finding | Result |
|---|---|
| Project concepts decomposed into general words ("the ambiguity of the reference") | ADR 0027: domain model, Capitalized terms with definitions |
| Lists rendered as one sentence per item | ADR 0028: Enumeration blocks |
| 115, then 57, rejections with no advice | 20+ named linter findings; `just replay` proxy |
| Stale claim in the source (ban messages "deferred") | Rewritten to the current facts |

Every sentence and block of the rewritten ADR parses (29/29, `just
lint-file`). Declared losses per ADR 0012 are in corpus/dogfood-pairs.tsv:
the "second-largest" ranking and the citation form (pair 1), the "exactly
where"/"constantly" emphasis (pairs 2, 3), the example and register of the
consequences (pair 6). The full banned list was expanded from "and their
object/possessive forms" to the explicit forms — a gain in precision, not
a loss. The Form Tags of the allowed pronouns are kept as four statements
after the Enumeration, because items cannot carry a second phrase.
