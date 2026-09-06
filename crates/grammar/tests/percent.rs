//! ADR 0057: percent of a singular noun (singular agreement, unlike ADR
//! 0024's percent of a counted plural set), and a bare share as a value in
//! Complement position — closing ADR 0024's 2 remaining deferrals (its
//! 3rd, decimals, was already closed by ADR 0029's general decimal digits).

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
fn percent_of_a_plural_set_still_takes_plural_agreement() {
    ok("43 percent of the swaps did not reduce the ambiguity.");
}

#[test]
fn percent_of_a_singular_noun_takes_singular_agreement() {
    ok("50 percent of the file is the text.");
}

#[test]
fn percent_of_a_singular_noun_rejects_plural_agreement() {
    rejects("50 percent of the file are the queues.");
}

#[test]
fn a_bare_share_is_a_valid_complement() {
    ok("the load is 43 percent.");
}

#[test]
fn decimal_percent_already_works_via_adr_0029() {
    ok("43.5 percent of the swaps did not reduce the ambiguity.");
}

#[test]
fn a_bare_share_still_cannot_stand_as_the_subject() {
    // ADR 0024's anaphora ban: a bare share names no set, so it cannot
    // stand where a real noun phrase would — only in Complement position
    // does a bare share instead read as a value (ADR 0057).
    rejects("43 percent did not reduce the ambiguity.");
}
