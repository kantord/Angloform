# Dogfood: ADR 0005 in angloform

One model run (cold start, 6 of 9 paragraphs with a valid proposal on the
first run; all six judged needs-fix), then rewritten by hand 2026-09-02.
43/43 units parse on the first lint.

| Run finding | Result |
|---|---|
| jargon: prohibition, negation, auxiliary, constituent, scope ambiguity | five terms with definitions |
| "permits", "expresses", "negated", "interrogative", "only" | bans with advice (`only` stays a queue item) |
| a hallucination from a badly written source bullet ("did later added by ADR 0010") | the model now sees the whole document as context |
| "The Grammar bans <phrase>" read as an exhaustive list | examples are written "the phrase X is one example" until an exemplification form exists |

Declared losses in corpus/dogfood-pairs.tsv.
