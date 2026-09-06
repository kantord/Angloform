//! ADR 0058: a bare digit as a measurement value in Complement position —
//! "the value is 3", "the value is 0" — closing ADR 0022's deferral. "0"
//! tokenizes as its own NUM_VAL terminal, distinct from the NUM_PL count
//! slot, so it is legal here but still cannot reach a count position.

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
fn a_nonzero_value_is_a_valid_complement() {
    ok("the value is 3.");
}

#[test]
fn zero_is_a_valid_complement() {
    ok("the value is 0.");
}

#[test]
fn a_value_can_be_approximate() {
    ok("the value is about 4.");
}

#[test]
fn zero_still_cannot_reach_a_count_position() {
    // "0" tokenizes as NUM_VAL, never NUM_PL — NPNum needs NUM_PL, so this
    // stays unparseable exactly as ADR 0022 intended (redirect to "no").
    rejects("the agent deleted 0 files.");
}
