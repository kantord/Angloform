//! Prototype: "antiparsers" — small, independent grammars that recognize
//! known-*invalid*-angloform constructions structurally (not by token-window
//! heuristic), so a match carries real certainty, not a guess. Each lives
//! in its own `.lalrpop` file with its own `extern` token block and shares
//! no nonterminal with `angloform.lalrpop` or with each other — the same
//! confinement principle ADR 0043–0047 used for words, applied here to
//! grammars. Evaluates: does that scale (do N independent grammars stay
//! conflict-free), and can a match be mapped to a fixed repair.
//!
//! Scope, found empirically while building this: banned *words* (pronouns,
//! epistemic hedges, "may") never reach this pipeline at all — the lexer
//! rejects them before a token stream exists (see `Lexicon::tokenize`'s
//! `bans` check). Antiparsers are relevant only to *structural* mistakes:
//! every word tokenizes fine, the arrangement is what's rejected.

use grammar::{Lexicon, Tok};
use lalrpop_util::lalrpop_mod;

pub mod anti;
use anti::{Repair};

lalrpop_mod!(pub anti_bare_coord);
lalrpop_mod!(pub anti_noun_verb);
lalrpop_mod!(pub anti_free_only);
lalrpop_mod!(pub anti_clause_object);

/// One antiparser's match against a span of the token stream.
pub struct AntiFinding {
    pub name: &'static str,
    /// Token-index span in the original stream (for ranking against the
    /// Tier-1 failure position).
    pub span: (usize, usize),
    pub repair: Repair,
}

/// Try every antiparser at every span of `toks`. Cheap: sentences are
/// short (rarely >20 tokens), so the O(n^2) span search per antiparser is
/// negligible — a real deployment would bound span length per antiparser
/// instead of searching everything, but the prototype favors correctness
/// over that optimization.
pub fn scan(lexicon: &Lexicon, toks: &[Tok]) -> Vec<AntiFinding> {
    let mut out = Vec::new();
    let n = toks.len();
    for start in 0..n {
        for end in (start + 1)..=n {
            if let Some(m) = try_bare_coord(&toks[start..end]) {
                out.push(AntiFinding { name: "AntiBareCoordObject", span: (start, end), repair: m.repair() });
            }
            if let Some(m) = try_noun_verb(&toks[start..end]) {
                let redirect = lexicon.redirect(&m.word, "VERB");
                out.push(AntiFinding { name: "AntiNounVerbSlot", span: (start, end), repair: m.repair(redirect) });
            }
            if let Some(m) = try_free_only(&toks[start..end]) {
                out.push(AntiFinding { name: "AntiFreeOnly", span: (start, end), repair: m.repair() });
            }
            if let Some(m) = try_clause_object(&toks[start..end]) {
                out.push(AntiFinding { name: "AntiClauseObject", span: (start, end), repair: m.repair() });
            }
        }
    }
    out
}

/// Rank findings by proximity to the Tier-1 failure position — the
/// cheap, nearly-free heuristic from the design discussion: a match whose
/// span covers or touches the exact token where Tier-1 choked is almost
/// certainly the real explanation, far more likely than one matching some
/// unrelated span elsewhere in the sentence.
pub fn rank_by_failure_position(findings: &mut [AntiFinding], failure_pos: usize) {
    findings.sort_by_key(|f| distance(f.span, failure_pos));
}

fn distance(span: (usize, usize), pos: usize) -> usize {
    let (s, e) = span;
    if pos < s {
        s - pos
    } else if pos >= e {
        pos - e + 1
    } else {
        0 // failure position falls inside the span: strongest possible signal
    }
}

fn try_bare_coord(slice: &[Tok]) -> Option<anti::bare_coord::BareCoordMatch> {
    let iter = to_iter(slice);
    anti_bare_coord::StartParser::new().parse(iter).ok()
}

fn try_noun_verb(slice: &[Tok]) -> Option<anti::noun_verb::NounVerbMatch> {
    let iter = to_iter(slice);
    anti_noun_verb::StartParser::new().parse(iter).ok()
}

fn try_free_only(slice: &[Tok]) -> Option<anti::free_only::FreeOnlyMatch> {
    let iter = to_iter(slice);
    anti_free_only::StartParser::new().parse(iter).ok()
}

fn try_clause_object(slice: &[Tok]) -> Option<anti::clause_object::ClauseObjectMatch> {
    let iter = to_iter(slice);
    anti_clause_object::StartParser::new().parse(iter).ok()
}

type TokTriple = Result<(usize, Tok, usize), grammar::LexError>;

fn to_iter(slice: &[Tok]) -> impl Iterator<Item = TokTriple> + '_ {
    slice.iter().enumerate().map(|(i, t)| Ok::<(usize, Tok, usize), grammar::LexError>((i, t.clone(), i + 1)))
}

/// Render one finding as writer-facing text, matching the project's
/// existing advice voice.
pub fn format_finding(f: &AntiFinding) -> String {
    match &f.repair {
        Repair::Single(fix) => format!("[{}] try: \"{fix}\"", f.name),
        Repair::Menu(options) => {
            format!("[{}] ambiguous — pick one: {}", f.name, options.join("; "))
        }
        Repair::None(why) => format!("[{}] {why}", f.name),
    }
}
