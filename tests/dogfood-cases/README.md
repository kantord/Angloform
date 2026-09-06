# Dogfood lint cases

The standard dogfooding flow for our own documents (ADRs, docs):

1. **Lint** the text (`cargo run -p diagnose -- "<sentence>"`).
2. **Examine** each rejection: is the linter's flag *useful* — is it right
   about the prose? Check it against `docs/review-checklist.md`.
3. **If useful → act**: rewrite the source text **in place** (sanctioned for
   ADRs when meaning is preserved). Record the case as `resolution: rewritten`
   with the applied text.
4. **If it exposes an angloform gap → track it**: `resolution: gap` with the
   task. A gap claim requires a *well-written* sentence being rejected —
   coverage of badly-written prose is never a goal.
5. Cases awaiting judgment: `resolution: pending`.

One YAML per finding. Fields: `source` (file), `sentence` (the flagged
original), `diagnosis` (linter output at time of filing), `resolution`,
and `rewrite` (applied replacement) or `gap` (tracked task).
