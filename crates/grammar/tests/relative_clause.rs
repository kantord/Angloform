//! ADR 0059 (revises ADR 0010): a non-restrictive relative clause,
//! attached at the Sentence level (`Clause , which/who <predicate>`),
//! not embedded inside the object NP — an earlier embedded design hit a
//! real, confirmed LALR(1) conflict (see
//! docs/relative-clause-design-2026-09-06.md). "who" for a human/agent
//! antecedent, "which" otherwise; both are grammar-interchangeable —
//! correctness (who vs. which, and number agreement) is a documented,
//! not-yet-built Linter check (diagnose::relative_clause_checks), never
//! a grammar-level rejection.

use grammar::{parse_text, Lexicon};

fn repo(path: &str) -> String {
    format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))
}

fn lexicon() -> Lexicon {
    Lexicon::load(&repo("lexicon.tsv")).unwrap()
}

fn ok(s: &str) {
    let lex = lexicon();
    parse_text(&lex, s).unwrap_or_else(|e| panic!("expected to parse: {s} — {e}"));
}

fn rejects(s: &str) {
    let lex = lexicon();
    assert!(parse_text(&lex, s).is_err(), "expected a rejection: {s}");
}

#[test]
fn a_relative_clause_can_follow_a_transitive_clause() {
    ok("the Lexer produces the token, which helps the writer.");
}

#[test]
fn who_is_legal_for_a_plural_antecedent_too() {
    ok("the tool trusts the maintainers, who write the report.");
}

#[test]
fn who_and_which_are_grammar_interchangeable() {
    // Correct antecedent choice (who vs. which) is a Linter check, not
    // built yet (diagnose::relative_clause_checks) — the grammar
    // accepts either relativizer regardless of the real antecedent's
    // animacy. This test documents today's true behavior, not the
    // eventual naturalness-correct one.
    ok("the Lexer produces the token, who helps the writer.");
    ok("the tool trusts the maintainers, which write the report.");
}

#[test]
fn a_relative_clause_can_follow_an_intransitive_clause() {
    // Known, documented gap (see docs/relative-clause-design-
    // 2026-09-06.md): the grammar cannot tell whether the antecedent is
    // the subject or the object once Clause has already reduced. An
    // intransitive Clause leaves only the subject NP available, so this
    // is really subject-attachment slipping in through the back door —
    // v2's territory, gated on testing, per the design doc. Grammar
    // accepts it today; a Linter check would need to reject it.
    ok("the Lexer sits, which helps the writer.");
}

#[test]
fn that_stays_banned_as_a_relativizer() {
    rejects("the Lexer produces the token, that helps the writer.");
}

#[test]
fn object_relative_is_not_expressible() {
    // The embedded predicate always starts with a verb (PredRel's every
    // alternative opens on Vt/Vi/negation/modal/copula) — "which" can
    // never be followed by a subject NP, so an object-relative reading
    // ("which the tool accepts") has no derivation at all, not merely a
    // rejected one.
    rejects("the Lexer produces the token, which the tool accepts.");
}

#[test]
fn a_bare_relative_clause_cannot_open_a_sentence() {
    // RelSentence always requires a complete Clause (Subj + Predicate)
    // before the comma — "the lexer" alone has no predicate, so this
    // never reaches a state where a relative clause could attach
    // directly to the subject mid-sentence (the shape the original,
    // embedded design would have needed to explicitly forbid).
    rejects("the Lexer, which produces the token, helps the writer.");
}

#[test]
fn embedded_predicate_coordination_check() {
    ok("the Lexer produces the token, which sits in the position of a determiner.");
    ok("the Lexer produces the token, which sits in the position of a determiner and takes a plural noun.");
}

#[test]
fn real_corpus_rewrite_examples_parse() {
    ok("the Lexicon does not produce the token, which sits in the position of a determiner and takes a plural noun.");
    ok("the folder holds the program, which expands the Paradigms with about 10 rules.");
}
