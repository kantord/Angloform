//! ADR 0047: "only" confined inside an NP (NPSG/NPPL) — never a free
//! pre-predicate adverb — so its scope is exactly the NP it wraps, never
//! ambiguous between the verb, the object, or the whole clause the way
//! bare English "only" famously is (Rooth 1992's focus-association problem).

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
fn scopes_the_object() {
    ok("the mechanism stores only the report.");
}

#[test]
fn scopes_the_subject() {
    ok("only the mechanism stores the report.");
}

#[test]
fn scopes_a_pp_object() {
    ok("the agent gives only the report to the author.");
}

#[test]
fn combines_with_negation() {
    ok("the agent does not save only the report.");
}

#[test]
fn scopes_a_counted_plural() {
    ok("the agent stores only 3 reports.");
}

#[test]
fn free_pre_predicate_position_is_impossible() {
    // The classic ambiguous English shape ("I only introduced Sue to
    // John") — banned by construction, not by a linter rule.
    rejects("the mechanism only stores the report.");
}

#[test]
fn enumeration_intro_still_finds_its_noun_phrase() {
    ok("the agent stores only 3 reports:\n- \"a\"\n- \"b\"\n- \"c\"");
}
