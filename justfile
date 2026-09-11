# angloform task runner — `just <target>`

# THE single entry point: every validation in the repo, Rust and web
# alike (tests, regeneration, drift check, homophones, web typecheck/
# unit/e2e). Run before every commit; CI runs exactly this, nothing else.
check:
    ./scripts/check.sh

# fetch/derive the non-vendored reference data
fetch-data:
    ./scripts/fetch-data.sh

# full homophone report across the whole enabled lexicon (informational —
# needs espeak-ng; `just check` runs the enforced seed/definitions-only gate)
homophones:
    python3 scripts/homophone-check.py

# regenerate web/src/lib/dictionary-data.json from seed/definitions/*.yaml
# (web/package.json's build/dev scripts already run this automatically)
dictionary-data:
    python3 scripts/build-dictionary-data.py

# regenerate the linter showcase
showcase:
    ./scripts/showcase.sh

# lint one or more sentences (quote each one: just lint "the agent reads the file")
[positional-arguments]
lint +SENTENCES:
    -cargo run -q -p diagnose --bin diagnose -- "$@"

# real-usage frequency of diagnose() outcomes and STYLE finding kinds
finding-frequency:
    cargo run -q -p diagnose --bin finding-frequency

# document-level lint of one markdown file (parse rate, topic continuity, relation inventory)
lint-file FILE:
    python3 scripts/lint-file.py {{FILE}}

# regenerate docs/coherence-report.md over every ADR
coherence:
    ./scripts/coherence.sh

# run the wasm linter playground (static site, web/)
web:
    cd web && pnpm dev

# build the playground (wasm + vite), ready for GitHub Pages
web-build:
    cd web && pnpm build

# unit + e2e tests of the playground only — a fast subset for iterating
# on web/ alone; `just check` is the full gate and already includes this
web-test:
    cd web && pnpm build && pnpm test && pnpm test:e2e

# MILESTONE: run the LLM repair-loop harness (needs OPENROUTER_API_KEY;
# see tests/agent-cases/README.md — results need human review)
agenttest concurrency="64":
    ANGLOFORM_TEST_CONCURRENCY={{concurrency}} cargo run -p agenttest

# MILESTONE: paragraph-level proposals for a markdown file (needs
# OPENROUTER_API_KEY; cases in tests/paragraph-cases/, report needs ADR 0012
# review). Add --dry-run to measure only, no API calls.
autofix-paragraphs file out="docs/paragraph-report.md" *FLAGS:
    cargo run -p agenttest -- paragraphs {{file}} {{out}} {{FLAGS}}

# print a domain term's definition: just define "Anaphoric Pronoun"
define +TERM:
    python3 scripts/define.py {{TERM}}

# pre-authoring check for a word already in seed/definitions/: every
# attested-but-not-own category must have a real "redirects" entry (no
# "advice"); shows the WordNet gloss + already-enabled candidates for each
word-check LEMMA:
    python3 scripts/word-check.py {{LEMMA}}

# review pending word definitions: diffs of changed seed/definitions/*.yaml,
# then the full content of any new (untracked) ones — colored, syntax-highlighted
word-review:
    git diff --color=always -- seed/definitions/*.yaml
    @echo "=== new (untracked) definitions ==="
    @git status --porcelain --untracked-files=all -- seed/definitions/ \
        | awk '$1 == "??" {print $2}' \
        | while read -r f; do bat --language=yaml --style=header --color=always "$f"; done

# no-LLM proxy: re-validate every stored paragraph proposal against the current
# linter and report valid counts, no-advice rejections, blocking words (seconds)
replay:
    python3 scripts/replay-stats.py

# review paragraph cases: all (one screen each), one in detail, or set a verdict
review-paragraphs *ARGS:
    python3 scripts/paragraph-review.py {{ARGS}}

# pre-judge cases with blind sub-agents (docs/prejudge.md): bundle the cases
# with a current best proposal, record the judgements, or rebuild the report
prejudge *ARGS:
    python3 scripts/prejudge.py {{ARGS}}

# judge every ADR and the domain model with blind sub-agents (docs/prejudge.md,
# "Whole documents"): bundle, record, report, or write rewrite briefs
judge-docs *ARGS:
    python3 scripts/docjudge.py {{ARGS}}

# set a verdict on paragraph case N: just verdict 3 needs-fix "why"
verdict N VERDICT *NOTE:
    python3 scripts/paragraph-review.py {{N}} {{VERDICT}} {{NOTE}}

# MILESTONE: propose angloform fixes for a markdown file's prose (never edits
# the source; output needs ADR 0012 meaning review)
autofix file out="docs/autofix-report.md":
    python3 scripts/extract-sentences.py {{file}} > /tmp/autofix-sentences.txt
    cargo run -p agenttest -- fix /tmp/autofix-sentences.txt {{out}}
