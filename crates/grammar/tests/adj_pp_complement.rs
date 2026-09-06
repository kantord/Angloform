//! ADR 0055: a prepositional phrase after an adjective complement —
//! "the copies are identical to the report" — the named-standard form ADR
//! 0023 deferred. Same attachment as ADR 0031's noun-phrase complement PP:
//! the copula (or a modal + "be") has no object to compete for it.

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

#[test]
fn named_standard_form() {
    ok("the copies are identical to the report.");
}

#[test]
fn bare_adjective_still_parses() {
    ok("the copies are identical.");
}

#[test]
fn works_after_a_modal() {
    ok("the report must be safe from the error.");
}

#[test]
fn works_with_other_verb_prepositions() {
    ok("the design is consistent with the project.");
}
