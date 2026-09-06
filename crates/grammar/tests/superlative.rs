//! ADR 0056: attributive superlatives — "the biggest file", "the most
//! transparent compound" — closing ADR 0029/0030's superlative deferral.
//! Same short/long split as ADR 0030's comparative, and the same optional
//! of-PP slot as ADR 0029's Ordinal.

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
fn inflected_short_form() {
    ok("the mechanism deletes the biggest file.");
}

#[test]
fn periphrastic_long_form() {
    ok("the mechanism deletes the most transparent file.");
}

#[test]
fn names_its_set_with_an_of_pp() {
    ok("the mechanism deletes the biggest file of the 3 files.");
}

#[test]
fn set_is_optional_like_the_ordinal() {
    ok("the report is the shortest document.");
}

#[test]
fn bare_most_before_a_short_adjective_is_impossible() {
    // short adjectives inflect; "most" is reserved for long ones (ADR 0056,
    // mirroring ADR 0030's "more big" vs "bigger" split)
    rejects("the mechanism deletes the most big file.");
}
