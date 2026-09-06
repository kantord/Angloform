//! Tier-2 diagnosis: when tier-1 rejects a sentence, explain why.
//!
//! A deliberately loose superset grammar (agreement ignored, transitivity
//! ignored, banned attachments and orders admitted) is parsed with an
//! all-parses counter:
//!   count > 1 → AMBIGUOUS — the sentence has several readings, and the
//!               count itself is the red flag (see CONTEXT.md "Rejection");
//!   count = 1 → STYLE — a banned construction, named by pattern checks;
//!   count = 0 → not recognizable as an English-like clause at all.
//! Pattern checks over the token stream attach specific, actionable names
//! (missing *then*, quantifier+negation, reduced relative, …).

use grammar::{is_enumeration, is_mapping, is_step_block, metrics, parse, parse_text, Lexicon, Metrics, Tok, Tree};
use std::collections::BTreeMap;

// ------------------------------------------------------------ diagnosis --

#[derive(Debug)]
pub enum Diagnosis {
    /// Tier-1 accepts: unique parse, with its load metrics.
    Clean(Metrics),
    /// A word-level rejection (unknown word / rejected sense), with advice.
    Word(String),
    /// Banned structure with exactly one tier-2 reading.
    Style(Vec<String>),
    /// Multiple tier-2 readings: structurally ambiguous.
    Ambiguous { readings: u64, findings: Vec<String> },
    /// Tier-2 cannot recognize it either.
    Unknown,
}

pub fn diagnose(lexicon: &Lexicon, sentence: &str) -> Diagnosis {
    // an Enumeration block (ADR 0028) is one unit; its errors are already explained
    if is_enumeration(sentence) || is_step_block(sentence) || is_mapping(sentence) {
        return match parse_text(lexicon, sentence) {
            Ok(tree) => Diagnosis::Clean(metrics(&tree)),
            Err(e) if e.contains("not an angloform word") || e.contains("is banned in angloform") => Diagnosis::Word(e),
            Err(e) => Diagnosis::Style(vec![e]),
        };
    }
    match parse(lexicon, sentence) {
        Ok(tree) => {
            if let Some(msg) = same_verb_coordination(lexicon, &tree) {
                return Diagnosis::Style(vec![msg]);
            }
            if let Some(msg) = vague_appositive(lexicon, &tree) {
                return Diagnosis::Style(vec![msg]);
            }
            if let Some(msg) = other_domain_membership(lexicon, &tree) {
                return Diagnosis::Style(vec![msg]);
            }
            return Diagnosis::Clean(metrics(&tree));
        }
        Err(e) if e.contains("not an angloform word")
            || e.contains("is banned in angloform")
            || e.contains("not yet usable") => {
            return Diagnosis::Word(e)
        }
        Err(_) => {}
    }
    let Ok(toks) = lexicon.tokenize(sentence) else {
        return Diagnosis::Unknown;
    };
    let toks: Vec<Tok> = toks.into_iter().map(|(_, t)| t).collect();
    let mut findings = pattern_findings(&toks);
    findings.extend(slot_findings(lexicon, &toks));
    suppress_superseded(&mut findings);
    findings.sort();
    findings.dedup();
    // crates/antiparse: when the hand-written checks above found nothing,
    // try the antiparser prototype before falling back to a fully generic
    // message — a structural match carries real certainty (it's a real
    // parse of the bad shape, not a token-window guess), ranked by
    // proximity to where Tier-1 actually failed.
    if findings.is_empty() {
        if let Some(msg) = antiparser_fallback(lexicon, &toks) {
            findings.push(msg);
        }
    }
    let readings = Tier2::new(&toks).count();
    match readings {
        0 => {
            if findings.is_empty() {
                Diagnosis::Unknown
            } else {
                Diagnosis::Style(findings)
            }
        }
        1 => {
            if findings.is_empty() {
                findings.push(
                    "this structure is outside the sanctioned sentence shapes — \
                     restructure into one of the angloform templates"
                        .to_string(),
                );
            }
            Diagnosis::Style(findings)
        }
        n => {
            findings.push(
                "split the sentence, or move the prepositional phrase so each \
                 attachment is explicit"
                    .to_string(),
            );
            Diagnosis::Ambiguous { readings: n, findings }
        }
    }
}

/// The top antiparser match, formatted, or None if none fired. Re-parses
/// the (already known to fail) token stream to recover the raw
/// `ParseError` and its failure position for ranking — cheap, and keeps
/// `parse()`'s public signature (a formatted string) unchanged for every
/// other caller.
fn antiparser_fallback(lexicon: &Lexicon, toks: &[Tok]) -> Option<String> {
    let mut findings = antiparse::scan(lexicon, toks);
    if findings.is_empty() {
        return None;
    }
    if let Err(e) = grammar::parse_tokens(toks) {
        if let Some(pos) = grammar::failure_position(&e) {
            antiparse::rank_by_failure_position(&mut findings, pos);
        }
    }
    Some(antiparse::format_finding(&findings[0]))
}

// ---------------------------------------------------- success-time checks --

/// ADR 0048: same-subject VP coordination repeating the identical verb
/// lemma ("stores a word and stores a message") is banned in favor of the
/// colon-list construction (ADR 0041) — the sole canonical form for this
/// meaning now (grilled design: angloform converges to one construction
/// per meaning by default; coexistence needs empirical evidence, and none
/// exists for this pair — this session's own earlier analysis already
/// found the repeat form reads worse). This cannot be a grammar rule: a
/// CFG has no way to compare two terminals' string payloads against each
/// other, only their category, so a LALRPOP production can't express "the
/// second verb token must differ from the first." It's a semantic check
/// on an already-successful Tier-1 parse — the first check in this file
/// that can turn `Clean` into `Style` (a deliberately narrow special
/// case, not a general new pipeline stage; see docs/ideas.md).
fn same_verb_coordination(lexicon: &Lexicon, tree: &Tree) -> Option<String> {
    if let Tree::Node { label: "S" | "Clause", children, .. } = tree {
        if children.len() == 3 {
            if let (Some(pred), Tree::Node { label: "CoordPred", children: cp, .. }) =
                (children.get(1), &children[2])
            {
                if let (Some(v1), Some(v2)) = (plain_vp_verb(pred), cp.get(1).and_then(plain_vp_verb)) {
                    let l1 = lexicon.lemma_of(v1).unwrap_or(v1);
                    let l2 = lexicon.lemma_of(v2).unwrap_or(v2);
                    if l1 == l2 {
                        return Some(format!(
                            "\"{v1}\" repeats across the coordination — write the colon-list \
                             instead: \"{v1}: <object> and <object>\" (ADR 0041, ADR 0048)"
                        ));
                    }
                }
            }
        }
    }
    if let Tree::Node { children, .. } = tree {
        for c in children {
            if let Some(m) = same_verb_coordination(lexicon, c) {
                return Some(m);
            }
        }
    }
    None
}

/// A generic placeholder noun ("term", "thing", ...) that reads as a
/// vague auto-inserted tag when it is the ENTIRE content of an
/// Appositive (ADR 0054), with no adjective and no "of"-PP to give it
/// real content. A direct A/B test found this exact contrast: "The
/// Redirect, a term, appears in the Lexicon" scored 2/5 (the judge:
/// "vague filler... auto-inserted gloss tag"), while the identical
/// shape with a modifier ("a term of the project") scored 4/5. A CFG
/// cannot single out "term" from a specific noun like "Function Word"
/// — both are just LNounSg — so this is a closed, curated word list, a
/// semantic check in the same spirit as ADR 0048's same-verb check
/// just above.
const VAGUE_APPOS_NOUNS: &[&str] = &[
    "term", "thing", "kind", "sort", "item", "entity", "aspect", "element", "factor", "part", "piece",
];

fn vague_appositive(lexicon: &Lexicon, tree: &Tree) -> Option<String> {
    if let Tree::Node { label: "Appos", children, .. } = tree {
        // children: [subject, comma, "namely", content NP, comma] since
        // the 2026-09-06 revision made "namely" mandatory (ADR 0054) —
        // content moved from index 2 to index 3.
        if let Some(Tree::Node { label: "NP", children: np, .. }) = children.get(3) {
            if np.len() == 2 {
                if let Tree::Leaf { word, .. } = &np[1] {
                    let lemma = lexicon.lemma_of(word).unwrap_or(word);
                    if VAGUE_APPOS_NOUNS.contains(&lemma) {
                        return Some(format!(
                            "\"{word}\" alone in an Appositive reads as a vague placeholder — \
                             add a modifier (\"a {word} of <thing>\") or use a more specific noun (ADR 0054)"
                        ));
                    }
                }
            }
        }
    }
    if let Tree::Node { children, .. } = tree {
        for c in children {
            if let Some(m) = vague_appositive(lexicon, c) {
                return Some(m);
            }
        }
    }
    None
}

/// The verb word of a VP whose shape the colon-list construction (ADR
/// 0041) can actually replace: a transitive verb with a plain NP object
/// and *nothing else* — unwraps the ADR 0044 medial-adverb wrapper if
/// present. Anything else has no valid colon-list equivalent, so it's
/// deliberately excluded, not a false negative: an intransitive verb's
/// PP complement ("refers to the writer") — 2 children, but the second
/// is a PP, not an NP; a transitive verb with a trailing PPv ("checks
/// the word against the Lexicon") — the colon-list production has no PPv
/// slot at all, so rewriting would silently drop that PP's meaning.
fn plain_vp_verb(t: &Tree) -> Option<&str> {
    let t = match t {
        Tree::Node { label: "PredAdv", children, .. } => children.last()?,
        _ => t,
    };
    let Tree::Node { label: "VP", children, .. } = t else { return None };
    let [Tree::Leaf { word, .. }, obj] = children.as_slice() else { return None };
    is_np_like(obj).then_some(word)
}

fn is_np_like(t: &Tree) -> bool {
    match t {
        Tree::Leaf { .. } => true, // a bare Name as object
        Tree::Node { label, .. } => matches!(*label, "NP" | "NPAppos" | "NPGen" | "NPPct" | "NPOnly"),
    }
}

/// ADR 0049: "other" is grammatically confined to a colon-list's second
/// conjunct, but that alone doesn't make "the Ban and other Rejections"
/// well-formed — it also needs the first conjunct to actually *be* a
/// member of the category the plural noun names. That's a fact no CFG
/// can check (it's not even in the sentence — it's in the domain
/// model's own `member_of` graph, loaded into the Lexicon at lexgen
/// time, ADR 0027). Unverifiable relationships (either side isn't a
/// known domain term, or no membership is declared) are rejected, not
/// guessed — same discipline as ADR 0048's semantic check.
fn other_domain_membership(lexicon: &Lexicon, tree: &Tree) -> Option<String> {
    if let Tree::Node { label: "VP", children, .. } = tree {
        if let [_, _, n1, _, n2] = children.as_slice() {
            // NPOther children: [every|some, other, plural-noun] (head 2)
            if let Tree::Node { label: "NPOther", children: on, .. } = n2 {
                if let (Some(Tree::Leaf { word: quant, .. }), Some(Tree::Leaf { word: plural, .. })) =
                    (on.first(), on.get(2))
                {
                    let anchor = head_word(n1);
                    // path A (ADR 0049): the anchor is a Capitalized domain
                    // term that is a real member of the category the
                    // plural noun names.
                    let category = lexicon.lemma_of(plural).map(capitalize).unwrap_or_else(|| plural.clone());
                    let domain_ok = anchor.and_then(|a| lexicon.member_of(a)).is_some_and(|m| m == category);
                    // path B (ADR 0049 revision): a set-complement — the
                    // anchor's own head noun is the *same* lemma as the
                    // "other"-marked plural (no domain model needed, any
                    // common noun qualifies: "the table and every other
                    // table"). Verified by lemma equality, same technique
                    // ADR 0048 uses for the same-verb check.
                    let same_noun_ok = anchor
                        .and_then(|a| lexicon.lemma_of(a))
                        .zip(lexicon.lemma_of(plural))
                        .is_some_and(|(a, p)| a == p);
                    if !domain_ok && !same_noun_ok {
                        return Some(format!(
                            "\"{quant} other {plural}\" needs either \"{a}\" to be a member of \
                             \"{category}\" in the domain model, or \"{a}\" to name the same kind of \
                             thing as \"{plural}\" — neither holds, so this cannot be verified (ADR 0049)",
                            a = anchor.unwrap_or("?")
                        ));
                    }
                }
            }
        }
    }
    if let Tree::Node { children, .. } = tree {
        for c in children {
            if let Some(m) = other_domain_membership(lexicon, c) {
                return Some(m);
            }
        }
    }
    None
}

fn head_word(t: &Tree) -> Option<&str> {
    match t {
        Tree::Leaf { word, .. } => Some(word),
        Tree::Node { head, children, .. } => head_word(children.get(*head)?),
    }
}

/// Capitalizes every word, not just the first — a multi-word domain term
/// ("function word") capitalizes as "Function Word" (ADR 0027), not
/// "Function word". Bug found empirically: without this, real member_of
/// relationships on multi-word categories ("Copula" -> "Function Word")
/// were rejected as unverifiable, because the computed category name
/// didn't match what the lexicon actually stores.
fn capitalize(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ------------------------------------------------------ pattern findings --

/// When a word has a specific "X is a noun in angloform — as a verb use Y"
/// (or the mirror "is a verb ... as a noun use Y") redirect finding, drop
/// every other finding that also names that exact quoted word — those are
/// generic checks re-diagnosing the same token with worse, sometimes
/// contradictory advice (advice gap #2: the redirect is correct, the other
/// checks just don't know about it).
fn suppress_superseded(findings: &mut Vec<String>) {
    // the redirected word may sit alone in quotes ("file") or as part of a
    // longer quoted span ("file the") — match it at a quote/space boundary
    // either way, not just as a fully standalone quoted token.
    let redirected: Vec<String> = findings
        .iter()
        .filter(|f| f.contains("in angloform — as a "))
        .filter_map(|f| f.split('"').nth(1).map(str::to_string))
        .collect();
    if redirected.is_empty() {
        return;
    }
    findings.retain(|f| {
        f.contains("in angloform — as a ")
            || !redirected.iter().any(|w| {
                f.contains(&format!("\"{w}\"")) || f.contains(&format!("\"{w} ")) || f.contains(&format!(" {w}\""))
            })
    });
}

fn word(t: &Tok) -> &str {
    match t {
        Tok::Det(w) | Tok::DetSg(w) | Tok::Adj(w) | Tok::NounSg(w) | Tok::NounPl(w)
        | Tok::VtBase(w) | Tok::Vt3(w) | Tok::VtEd(w) | Tok::VtIng(w) | Tok::ViBase(w)
        | Tok::Vi3(w) | Tok::ViEd(w) | Tok::ViIng(w) | Tok::PrepN(w) | Tok::PrepV(w)
        | Tok::Pron1(w) | Tok::Pron2(w) | Tok::Poss(w) | Tok::CopSg(w) | Tok::CopPl(w)
        | Tok::CopSgPast(w) | Tok::CopPlPast(w) | Tok::Cop1Sg(w) | Tok::Conj(w) | Tok::Neg(w) | Tok::TempAdv(w) | Tok::TimeAdv(w) | Tok::Yet(w) | Tok::Focus(w) | Tok::Other(w)
        | Tok::DoBase(w) | Tok::Do3(w) | Tok::DoPast(w) | Tok::ModalMust(w)
        | Tok::ModalCan(w) | Tok::ModalCannot(w) | Tok::If(w) | Tok::Then(w)
        | Tok::Every(w) | Tok::No(w) | Tok::Num(w) | Tok::NumPl(w) | Tok::Percent(w)
        | Tok::Approx(w) | Tok::So(w) | Tok::Because(w) | Tok::Namely(w) | Tok::Some_(w) | Tok::Name(w)
        | Tok::Ord(w) | Tok::Than(w) | Tok::More(w) | Tok::Scale(w) | Tok::AdjCmp(w) | Tok::AdjLong(w)
        | Tok::AdjSup(w) | Tok::Most(w) | Tok::NumVal(w) | Tok::Be(w) | Tok::BecomeSg(w) | Tok::BecomePl(w) | Tok::BecomePast(w) => w,
        Tok::Comma => ",",
        Tok::Colon => ":",
    }
}

fn is_finite_verb(t: &Tok) -> bool {
    matches!(
        t,
        Tok::VtBase(_) | Tok::Vt3(_) | Tok::VtEd(_) | Tok::ViBase(_) | Tok::Vi3(_) | Tok::ViEd(_)
    )
}

fn pattern_findings(toks: &[Tok]) -> Vec<String> {
    let mut out = Vec::new();
    let has = |f: &dyn Fn(&Tok) -> bool| toks.iter().any(|t| f(t));

    // conditionals: missing then / consequent-first
    if matches!(toks.first(), Some(Tok::If(_))) && !has(&|t| matches!(t, Tok::Then(_))) {
        out.push(
            "conditional without \"then\" — write: if <clause>, then <clause> (ADR 0007)"
                .to_string(),
        );
    }
    if toks.iter().skip(1).any(|t| matches!(t, Tok::If(_))) {
        out.push(
            "condition must come first — write: if <clause>, then <clause> (ADR 0007)"
                .to_string(),
        );
    }
    // quantifier + negation
    if matches!(toks.first(), Some(Tok::Every(_)))
        && has(&|t| matches!(t, Tok::Neg(_) | Tok::ModalCannot(_)))
    {
        out.push(
            "\"every … not\" is scope-ambiguous — for none write \"no <noun> …\"; \
             for not-all write \"some <nouns> do not …\" (ADR 0014)"
                .to_string(),
        );
    }
    if matches!(toks.first(), Some(Tok::No(_)))
        && has(&|t| matches!(t, Tok::Neg(_) | Tok::ModalCannot(_)))
    {
        out.push("\"no …\" already negates — drop the second negation (ADR 0014)".to_string());
    }
    if matches!(toks.first(), Some(Tok::No(_))) && has(&|t| matches!(t, Tok::ModalMust(_))) {
        out.push(
            "\"no … must\" is ambiguous — for prohibition write, for example, \
             \"agents must not check the input\" (bare plural + must not, ADR 0014)"
                .to_string(),
        );
    }
    // "Then" opens a sentence only inside a Step Block (ADR 0034)
    if matches!(toks.first(), Some(Tok::Then(_))) {
        out.push(
            "\"then\" opens a line only inside a Step Block (Given / When / Then / And lines, ADR 0034); \
             in prose write the step as a plain sentence"
                .to_string(),
        );
    }
    // causal connectives (ADR 0026): never sentence-initial; comma mandatory
    if matches!(toks.first(), Some(Tok::Because(_))) {
        out.push(
            "\"because\" cannot start a sentence — write the result first: \
             \"<result>, because <reason>\"; or cause first: \"<cause>, so <result>\" (ADR 0026)"
                .to_string(),
        );
    }
    if matches!(toks.first(), Some(Tok::So(_))) {
        out.push(
            "\"so\" cannot start a sentence — join the cause in the same sentence: \
             \"<cause>, so <result>\" (ADR 0026)"
                .to_string(),
        );
    }
    for w in toks.windows(2) {
        if matches!(w[1], Tok::So(_) | Tok::Because(_)) && !matches!(w[0], Tok::Comma) {
            out.push(format!(
                "a comma before \"{}\" is mandatory — \"<clause>, {} <clause>\" (ADR 0026)",
                word(&w[1]),
                word(&w[1])
            ));
        }
    }
    // full-clause coordination (ADR 0037): a comma is mandatory when
    // "but"/"and"/"or" opens a new clause (a new subject follows the
    // conjunction) rather than joining a predicate under the same
    // subject, which stays comma-free.
    for i in 1..toks.len() {
        if let Tok::Conj(cj) = &toks[i] {
            if !matches!(toks[i - 1], Tok::Comma) && toks.get(i + 1).is_some_and(|n| is_det(n) || is_noun_head(n)) {
                out.push(format!(
                    "a comma before \"{cj}\" is mandatory when a new clause follows — \"<clause>, {cj} <clause>\"; \
                     no comma when \"{cj}\" only joins a predicate under the same subject (ADR 0037)"
                ));
            }
        }
    }
    // no / some misplacement
    if toks.iter().skip(1).any(|t| matches!(t, Tok::No(_))) {
        out.push(
            "\"no\" is subject-only — write \"… does not <verb> <nouns>\" instead (ADR 0014)"
                .to_string(),
        );
    }
    for w in toks.windows(2) {
        if matches!(w[0], Tok::Some_(_)) && matches!(w[1], Tok::NounSg(_) | Tok::Adj(_) | Tok::AdjLong(_)) {
            if let [Tok::Some_(_), Tok::NounSg(_)] = w {
                out.push("\"some\" takes a plural noun (ADR 0017)".to_string());
            }
        }
        // determiner–number agreement
        if matches!(w[0], Tok::DetSg(_)) && matches!(w[1], Tok::NounPl(_)) {
            out.push(format!(
                "\"{} {}\" — a/an take a singular noun",
                word(&w[0]),
                word(&w[1])
            ));
        }
        if matches!(w[0], Tok::Num(_)) && matches!(w[1], Tok::NounPl(_)) {
            out.push(format!(
                "\"{} {}\" — one takes a singular noun (ADR 0016)",
                word(&w[0]),
                word(&w[1])
            ));
        }
        if matches!(w[0], Tok::NumPl(_)) && matches!(w[1], Tok::NounSg(_)) {
            out.push(format!(
                "\"{} {}\" — a number takes a plural noun (ADR 0022)",
                word(&w[0]),
                word(&w[1])
            ));
        }
        // perfect aspect via have
        if matches!(&w[0], Tok::Vt3(v) | Tok::VtBase(v) | Tok::VtEd(v) if ["have","has","had"].contains(&v.as_str()))
            && matches!(w[1], Tok::VtEd(_) | Tok::ViEd(_))
        {
            out.push(
                "perfect aspect (\"has <verb>-ed\") is not in angloform — use the simple past \
                 (ADR 0016)"
                    .to_string(),
            );
        }
        // progressive / passive after copula
        if matches!(w[0], Tok::CopSg(_) | Tok::CopPl(_) | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::Cop1Sg(_)) {
            match &w[1] {
                Tok::VtIng(_) | Tok::ViIng(_) => out.push(
                    "progressive (\"is <verb>-ing\") is not in angloform — use the simple form \
                     (ADR 0003)"
                        .to_string(),
                ),
                Tok::VtEd(_) => out.push(
                    "passive (\"is <verb>-ed\") is not in angloform — name the doer and use \
                     active voice (ADR 0003)"
                        .to_string(),
                ),
                _ => {}
            }
        }
    }
    // reduced relative: noun + -ed verb + … + second finite verb, no conjunction
    if !has(&|t| matches!(t, Tok::Conj(_) | Tok::If(_))) {
        let verbs: Vec<usize> = toks
            .iter()
            .enumerate()
            .filter(|(_, t)| is_finite_verb(t) || matches!(t, Tok::CopSg(_) | Tok::CopPl(_) | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::Cop1Sg(_)))
            .map(|(i, _)| i)
            .collect();
        if verbs.len() >= 2 {
            if let Tok::VtEd(_) | Tok::ViEd(_) = toks[verbs[0]] {
                if verbs[0] >= 1 && matches!(toks[verbs[0] - 1], Tok::NounSg(_) | Tok::NounPl(_)) {
                    out.push(
                        "this reads as a reduced relative (\"the file stored …\") — angloform \
                         has no relative clauses yet; split into two sentences (ADR 0010)"
                            .to_string(),
                    );
                }
            }
        }
    }
    // a word mentioned as a word must be quoted (ADR 0018)
    for w in toks.windows(2) {
        let function_word = matches!(
            w[1],
            Tok::Pron1(_) | Tok::Pron2(_) | Tok::Poss(_) | Tok::Det(_) | Tok::DetSg(_)
                | Tok::If(_) | Tok::Then(_) | Tok::No(_) | Tok::Every(_) | Tok::Some_(_)
        );
        if matches!(w[0], Tok::NounSg(_)) && function_word {
            out.push(format!(
                "\"{} {}\" — a word mentioned as a word must be quoted: \"{} \\\"{}\\\"\" (ADR 0018)",
                word(&w[0]), word(&w[1]), word(&w[0]), word(&w[1])
            ));
        }
    }
    // "the i Pronoun": a word used as a name goes after the noun, in quotes
    for w in toks.windows(2) {
        if matches!(w[0], Tok::Pron1(_) | Tok::Pron2(_) | Tok::Poss(_)) && matches!(w[1], Tok::NounSg(_) | Tok::NounPl(_)) {
            out.push(format!(
                "\"{} {}\" — a word used as a name follows the noun in quotes: \"the {} \\\"{}\\\"\" (ADR 0018)",
                word(&w[0]), word(&w[1]), word(&w[1]), word(&w[0])
            ));
        }
    }
    // of-chains (ADR 0011): "the point of the split of the sentences"
    {
        let ofs: Vec<usize> = toks.iter().enumerate().filter(|(_, t)| matches!(t, Tok::PrepN(_))).map(|(i, _)| i).collect();
        for w in ofs.windows(2) {
            let between = &toks[w[0] + 1..w[1]];
            if !between.iter().any(|t| is_finite_verb(t) || matches!(t, Tok::CopSg(_) | Tok::CopPl(_) | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::Cop1Sg(_) | Tok::Conj(_) | Tok::Comma | Tok::ModalMust(_) | Tok::ModalCan(_) | Tok::ModalCannot(_) | Tok::Do3(_) | Tok::DoBase(_) | Tok::DoPast(_))) {
                out.push(
                    "\"of … of\" — \"of\" does not chain; name the inner thing in its own sentence, \
                     or drop one level (ADR 0011)"
                        .to_string(),
                );
                break;
            }
        }
    }
    if let [first, Tok::CopSg(_) | Tok::CopPl(_), ..] = toks {
        if matches!(first, Tok::Pron2(_) | Tok::Poss(_) | Tok::Det(_) | Tok::DetSg(_)) {
            out.push(format!(
                "\"{}\" used as a word must be quoted: \"\\\"{}\\\" is …\" (ADR 0018)",
                word(first), word(first)
            ));
        }
    }
    // bare singular noun: "prefers clarity" — every singular noun takes a
    // determiner (mass nouns take "the")
    for (i, t) in toks.iter().enumerate() {
        if let Tok::NounSg(n) = t {
            // look back through adjectives to the word before the noun phrase
            let mut j = i;
            while j > 0 && matches!(toks[j - 1], Tok::Adj(_) | Tok::AdjLong(_)) {
                j -= 1;
            }
            let prev = j.checked_sub(1).map(|k| &toks[k]);
            let determined = prev.is_some_and(|p| {
                is_det(p) || matches!(p, Tok::NounSg(_) | Tok::Approx(_) | Tok::Percent(_)
                    | Tok::Pron1(_) | Tok::Pron2(_) | Tok::Poss(_))
            });
            let mentioned = toks.get(i + 1).is_some_and(|nx| matches!(nx, Tok::Name(_)));
            if !determined && !mentioned {
                out.push(format!(
                    "\"{n}\" — a singular noun needs a determiner: \"the {n}\" (mass nouns take \"the\")"
                ));
            }
        }
    }
    // clause as object ("the report shows the Pronouns are banned") used to
    // be a token-window heuristic here ("two verb-ish tokens, no
    // connective") — it misfired on ordinary predicates (do-support
    // negation, modal + verb, copula + passive participle: each is
    // exactly two verb-ish tokens with nothing between them). Replaced by
    // `antiparse`'s `AntiClauseObject`, which requires a real subject-like
    // NP wedged between the two verbs — the actual structural signal that
    // a second clause has started (see crates/antiparse/src/
    // anti_clause_object.lalrpop). It runs as an antiparser fallback, not
    // here, so it no longer fires alongside an unrelated correct finding.
    // "be" outside "must be / can be / cannot be" (ADR 0032)
    for (i, t) in toks.iter().enumerate() {
        if matches!(t, Tok::Be(_)) {
            let after_modal = i > 0 && matches!(toks[i - 1], Tok::ModalMust(_) | Tok::ModalCan(_) | Tok::ModalCannot(_) | Tok::Neg(_));
            if !after_modal {
                out.push("\"be\" exists only after a modal: \"must be <adjective>\", \"can be <noun phrase>\"; elsewhere write \"is\" or \"are\" (ADR 0032)".to_string());
            }
        }
    }
    // "about" as a preposition (APPROX sits only before digits, ADR 0025)
    for (i, t) in toks.iter().enumerate() {
        if matches!(t, Tok::Approx(_)) && !matches!(toks.get(i + 1), Some(Tok::NumPl(_))) {
            out.push("\"about\" exists only before digits (\"about 10 rules\"); for a topic write \"of\": \"a decision of the Grammar\", or restructure (ADR 0025)".to_string());
        }
    }
    // resultative: verb + object + adjective ("keeps the loss small") — no such shape
    for (i, t) in toks.iter().enumerate() {
        if !is_finite_verb(t) {
            continue;
        }
        // object = [det] [adj] noun; then an adjective right after it
        let mut j = i + 1;
        if toks.get(j).is_some_and(is_det) { j += 1; }
        while matches!(toks.get(j), Some(Tok::Adj(_) | Tok::AdjLong(_))) { j += 1; }
        let mut k = j;
        if matches!(toks.get(k + 1), Some(Tok::PrepN(_))) {
            // skip "of <det> [adj] <noun>"
            k += 2;
            if toks.get(k).is_some_and(is_det) { k += 1; }
            while matches!(toks.get(k), Some(Tok::Adj(_) | Tok::AdjLong(_))) { k += 1; }
        }
        if matches!(toks.get(j), Some(Tok::NounSg(_) | Tok::NounPl(_) | Tok::Name(_)))
            && matches!(toks.get(k), Some(Tok::NounSg(_) | Tok::NounPl(_) | Tok::Name(_)))
            && matches!(toks.get(k + 1), Some(Tok::Adj(_) | Tok::AdjLong(_)))
            && toks.get(k + 2).is_none()
        {
            out.push(format!(
                "\"{} … {} {}\" — a verb cannot take an object and an adjective; use a verb that carries the result (\"limits the loss\"), or 2 sentences",
                word(t), word(&toks[j]), word(&toks[j + 1])
            ));
        }
    }
    // "0" is a measurement value (ADR 0058), never a count: it tokenizes
    // as NUM_VAL, not NUM_PL, so it can never reach a count position
    // (NPNum needs NUM_PL) — give the same redirect the old lex-level ban
    // on "0" gave, now as a named finding instead of an unnamed rejection.
    for (i, t) in toks.iter().enumerate() {
        if matches!(t, Tok::NumVal(_)) && matches!(toks.get(i + 1), Some(Tok::NounPl(_))) {
            out.push("\"0\" is a value, not a count — for none write \"no <noun> …\" as the subject, or \"… does not <verb> <nouns>\" (ADR 0022)".to_string());
        }
    }
    // a quantifier + noun + Name: "every Grammar ADR" — the appositive needs the/a
    for w in toks.windows(3) {
        if matches!(w[0], Tok::Every(_) | Tok::No(_) | Tok::Some_(_) | Tok::NumPl(_)) && matches!(w[1], Tok::NounSg(_) | Tok::NounPl(_)) && matches!(w[2], Tok::Name(_)) {
            out.push(format!(
                "\"{} {} {}\" — a Name follows a noun only after \"the\" or \"a\"; write \"every {} of {}\" or restructure (ADR 0018)",
                word(&w[0]), word(&w[1]), word(&w[2]), word(&w[1]), word(&w[2])
            ));
        }
    }
    // a noun + Name inside an of-phrase or as a modifier: "standard English"
    for w in toks.windows(2) {
        if matches!(w[0], Tok::PrepN(_) | Tok::PrepV(_)) && matches!(w[1], Tok::Adj(_) | Tok::AdjLong(_) | Tok::NounSg(_)) {
            // handled by the bare-singular / adjective rules
        }
        if matches!(w[0], Tok::Adj(_) | Tok::AdjLong(_)) && matches!(w[1], Tok::Name(_)) {
            out.push(format!(
                "\"{} {}\" — an adjective cannot modify a Name; write \"{}\" alone, or \"the {} <noun> {}\" (ADR 0018)",
                word(&w[0]), word(&w[1]), word(&w[1]), word(&w[0]), word(&w[1])
            ));
        }
    }
    // "one" as a pronoun: "the more expressive one", "a good one"
    for (i, t) in toks.iter().enumerate() {
        if matches!(t, Tok::Num(_)) && i > 0 && matches!(toks[i - 1], Tok::Adj(_) | Tok::AdjLong(_) | Tok::AdjCmp(_))
            && !matches!(toks.get(i + 1), Some(Tok::NounSg(_)))
        {
            out.push("\"one\" is not a Pronoun — repeat the noun: \"the expressive formulation\" (ADR 0016)".to_string());
        }
    }
    // an ordinal alone: "the criterion is first"
    for (i, t) in toks.iter().enumerate() {
        if matches!(t, Tok::Ord(_)) && !matches!(toks.get(i + 1), Some(Tok::NounSg(_) | Tok::Adj(_) | Tok::AdjLong(_))) {
            out.push(format!("\"{}\" needs its noun: \"the {} criterion\" (ADR 0029)", word(t), word(t)));
        }
    }
    // "more" before a noun: a comparative of quantity
    for w in toks.windows(2) {
        if matches!(w[0], Tok::More(_)) && matches!(w[1], Tok::NounSg(_) | Tok::NounPl(_)) {
            out.push("\"more <noun>\" — comparatives of a quantity are deferred (ADR 0030); write \"a bigger number of <nouns>\", or restructure".to_string());
        }
    }
    // ditransitive: verb + NP + NP ("gives every sentence one Parse")
    for (i, t) in toks.iter().enumerate() {
        if !is_finite_verb(t) { continue; }
        let mut j = i + 1;
        if toks.get(j).is_some_and(is_det) { j += 1; }
        while matches!(toks.get(j), Some(Tok::Adj(_) | Tok::AdjLong(_))) { j += 1; }
        if matches!(toks.get(j), Some(Tok::NounSg(_) | Tok::NounPl(_) | Tok::Name(_))) && toks.get(j + 1).is_some_and(is_det) {
            let mut k = j + 2;
            while matches!(toks.get(k), Some(Tok::Adj(_) | Tok::AdjLong(_))) { k += 1; }
            if matches!(toks.get(k), Some(Tok::NounSg(_) | Tok::NounPl(_) | Tok::Name(_))) {
                out.push(format!(
                    "\"{} {} … {}\" — a verb takes one object; write \"{} <thing> to <receiver>\" (no ditransitives)",
                    word(t), word(&toks[j]), word(&toks[k]), word(t)
                ));
            }
        }
    }
    // an adjective as an adverb at the end: "arrives later"
    if let [.., v, Tok::Adj(a) | Tok::AdjLong(a)] = toks {
        if matches!(v, Tok::Vi3(_) | Tok::ViEd(_) | Tok::ViBase(_)) {
            out.push(format!("\"{}\" — no adverbs; name the time or the way with a phrase: \"after the decision\", \"in a {} way\"", a, a));
        }
    }
    // a determiner before a Name: "the Triage" (a Name takes no determiner)
    for w in toks.windows(2) {
        if is_det(&w[0]) && matches!(w[1], Tok::Name(_)) && !matches!(w[0], Tok::NumPl(_)) {
            out.push(format!(
                "\"{} {}\" — a Name takes no determiner: write \"{}\", or introduce it with a noun: \"the tool {}\" (ADR 0018)",
                word(&w[0]), word(&w[1]), word(&w[1]), word(&w[1])
            ));
        }
    }
    // a stray "not" inside a noun phrase: "a person not the speaker"
    for (i, t) in toks.iter().enumerate() {
        if matches!(t, Tok::Neg(_)) && i > 0
            && !matches!(toks[i - 1], Tok::CopSg(_) | Tok::CopPl(_) | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::Cop1Sg(_) | Tok::DoBase(_) | Tok::Do3(_) | Tok::DoPast(_) | Tok::ModalMust(_))
        {
            out.push("\"not\" negates the verb only — write \"does not <verb>\" or \"is not <adjective>\"; a noun phrase cannot carry \"not\" (ADR 0005)".to_string());
        }
    }
    // comma splice: a comma between two clauses with no connective after it
    for (i, t) in toks.iter().enumerate() {
        if matches!(t, Tok::Comma) && !matches!(toks.get(i + 1), Some(Tok::So(_) | Tok::Because(_) | Tok::Then(_) | Tok::Conj(_)))
            && !matches!(toks.first(), Some(Tok::If(_)))
            && toks[..i].iter().any(|t| is_finite_verb(t) || matches!(t, Tok::CopSg(_) | Tok::CopPl(_) | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::Cop1Sg(_)))
            && toks[i + 1..].iter().any(|t| is_finite_verb(t) || matches!(t, Tok::CopSg(_) | Tok::CopPl(_) | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::Cop1Sg(_)))
        {
            out.push("a comma cannot join 2 clauses — write 2 sentences, or \"<clause>, so <clause>\" / \"<clause>, because <clause>\" (ADR 0026)".to_string());
        }
    }
    // (b) adjective used as a noun: "a possessive", "the future" at the end
    for (i, t) in toks.iter().enumerate() {
        if let Tok::Adj(a) | Tok::AdjLong(a) = t {
            let after_det = i > 0 && is_det(&toks[i - 1]);
            let no_noun = !matches!(toks.get(i + 1), Some(Tok::Adj(_) | Tok::AdjLong(_) | Tok::NounSg(_) | Tok::NounPl(_) | Tok::Name(_)));
            if after_det && no_noun {
                out.push(format!("\"{a}\" is an adjective — add the noun it describes: \"a {a} <noun>\""));
            }
        }
    }
    // (c) a count without its noun: "2200 of the tokens"
    for w in toks.windows(2) {
        if matches!(w[0], Tok::NumPl(_)) && matches!(w[1], Tok::PrepN(_)) {
            out.push(format!(
                "\"{} of …\" — a count needs its noun: \"{} Pronouns\"; for a share write \"<digits> percent of …\" (ADR 0022, 0024)",
                word(&w[0]), word(&w[0])
            ));
        }
    }
    // (d) -ing / -ed verb form after a determiner: "the finding"
    for w in toks.windows(2) {
        if is_det(&w[0]) && matches!(w[1], Tok::VtIng(_) | Tok::ViIng(_)) {
            out.push(format!(
                "\"{}\" is a verb form in angloform and cannot follow a determiner — name the thing with a noun",
                word(&w[1])
            ));
        }
    }
    // (e) a verb form as the subject: "Resolving the Pronoun requires …"
    if let Some(Tok::VtIng(v) | Tok::ViIng(v)) = toks.first() {
        out.push(format!(
            "\"{v} …\" — a verb form cannot be the subject; name the doer: \"the Discourse Layer resolves the Pronoun\""
        ));
    }
    // (f) "of every": every is subject/object only (ADR 0014)
    for w in toks.windows(2) {
        if matches!(w[0], Tok::PrepN(_)) && matches!(w[1], Tok::Every(_)) {
            out.push("\"of every\" — \"every\" cannot follow \"of\"; write \"of a <noun>\", or make the every-phrase the subject (ADR 0014)".to_string());
        }
    }
    // (g) name before its noun: "UD-EWT corpora"
    for w in toks.windows(2) {
        if let (Tok::Name(n), Tok::NounSg(h) | Tok::NounPl(h)) = (&w[0], &w[1]) {
            out.push(format!("\"{n} {h}\" — the name follows its noun: \"the {h} {n}\" (ADR 0018)"));
        }
    }
    // (h) a pronoun mentioned as a word at the end or in a list: "allows my"
    for (i, t) in toks.iter().enumerate() {
        if matches!(t, Tok::Pron1(_) | Tok::Pron2(_) | Tok::Poss(_)) {
            let last = i + 1 == toks.len();
            let listed = matches!(toks.get(i + 1), Some(Tok::Conj(_))) && i > 0 && !matches!(toks[i - 1], Tok::Conj(_)) && i > 1;
            if (last && i > 0) || listed {
                out.push(format!("\"{}\" used as a word must be quoted: \"\\\"{}\\\"\" (ADR 0018)", word(t), word(t)));
            }
        }
    }
    // (i) coordinated prepositional phrases: "of the speaker or of the hearer"
    for w in toks.windows(2) {
        if matches!(w[0], Tok::Conj(_)) && matches!(w[1], Tok::PrepN(_) | Tok::PrepV(_)) {
            out.push("phrases cannot be coordinated — split the sentence, one phrase each (ADR 0004)".to_string());
        }
    }
    // participle as a noun modifier: "the banned Pronouns" (no participial
    // adjectives; the verb keeps the form)
    for w in toks.windows(3) {
        if is_det(&w[0]) && matches!(w[1], Tok::VtEd(_) | Tok::VtIng(_) | Tok::ViEd(_) | Tok::ViIng(_))
            && matches!(w[2], Tok::NounSg(_) | Tok::NounPl(_))
        {
            out.push(format!(
                "\"{} {} {}\" — a verb form cannot modify a noun; say who does it: \
                 \"the Linter bans the Pronouns\", or split the sentence",
                word(&w[0]), word(&w[1]), word(&w[2])
            ));
        }
    }
    // a preposition other than "of" right after a subject noun attaches to
    // the verb, not the noun (ADR 0011): "Pronouns for the speaker are …"
    {
        let first_verb = toks.iter().position(|t| {
            is_finite_verb(t)
                || matches!(t, Tok::CopSg(_) | Tok::CopPl(_) | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::Cop1Sg(_)
                    | Tok::ModalMust(_) | Tok::ModalCan(_) | Tok::ModalCannot(_) | Tok::Do3(_) | Tok::DoBase(_) | Tok::DoPast(_))
        });
        if let Some(v) = first_verb {
            for i in 1..v {
                if matches!(toks[i - 1], Tok::NounSg(_) | Tok::NounPl(_)) {
                    if let Tok::PrepV(p) = &toks[i] {
                        out.push(format!(
                            "\"{} {p} …\" — only \"of\" attaches to a noun; \"{p}\" attaches to the verb. \
                             Write \"the {} of …\", or move the phrase after the verb (ADR 0011)",
                            word(&toks[i - 1]), word(&toks[i - 1])
                        ));
                    }
                }
            }
        }
    }
    // copula + prepositional phrase (ADR 0003) — a bare PP still has no
    // attachment; an adjective + PP is legal now (ADR 0055) so is not
    // flagged here.
    for (i, t) in toks.iter().enumerate() {
        if !matches!(t, Tok::CopSg(_) | Tok::CopPl(_) | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::Cop1Sg(_)) {
            continue;
        }
        let mut j = i + 1;
        if matches!(toks.get(j), Some(Tok::Neg(_))) {
            j += 1;
        }
        if let Some(Tok::PrepV(p) | Tok::PrepN(p)) = toks.get(j) {
            out.push(format!(
                "\"{} {p} …\" — the copula takes an adjective or a noun phrase, not a \
                 prepositional phrase; use a verb: \"the Lexicon contains the Pronouns\" (ADR 0003)",
                word(t)
            ));
        }
    }
    // noun-phrase coordination (ADR 0004: coordinate predicates or clauses)
    for (i, t) in toks.iter().enumerate() {
        if !matches!(t, Tok::Conj(_)) || i == 0 || !matches!(toks[i - 1], Tok::NounSg(_) | Tok::NounPl(_) | Tok::Name(_)) {
            continue;
        }
        let mut j = i + 1;
        if toks.get(j).is_some_and(is_det) {
            j += 1;
        }
        while matches!(toks.get(j), Some(Tok::Adj(_) | Tok::AdjLong(_))) {
            j += 1;
        }
        let verb_before = toks[..i].iter().any(|t| {
            is_finite_verb(t)
                || matches!(t, Tok::CopSg(_) | Tok::CopPl(_) | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::Cop1Sg(_)
                    | Tok::ModalMust(_) | Tok::ModalCan(_) | Tok::ModalCannot(_) | Tok::Do3(_) | Tok::DoBase(_) | Tok::DoPast(_))
        });
        if matches!(toks.get(j), Some(Tok::NounSg(_) | Tok::NounPl(_) | Tok::Name(_)))
            && (!verb_before || toks.get(j + 1).is_none_or(|n| matches!(n, Tok::PrepV(_) | Tok::PrepN(_) | Tok::Comma | Tok::Conj(_))))
        {
            out.push(
                "noun phrases cannot be coordinated — write the colon-list: \"the mechanism \
                 stores: a word and a message\" (ADR 0041), or split the sentence (ADR 0004). \
                 Repeating the verb does not help here: the same verb twice is itself banned \
                 (ADR 0048)"
                    .to_string(),
            );
        }
    }
    // transitive verb with no object
    for (i, t) in toks.iter().enumerate() {
        if matches!(t, Tok::VtBase(_) | Tok::Vt3(_) | Tok::VtEd(_)) {
            let next = toks.get(i + 1);
            let object_start = matches!(
                next,
                Some(Tok::Det(_) | Tok::DetSg(_) | Tok::Poss(_) | Tok::Num(_) | Tok::NumPl(_) | Tok::Approx(_) | Tok::Every(_)
                    | Tok::Adj(_) | Tok::AdjLong(_) | Tok::NounSg(_) | Tok::NounPl(_))
            );
            let have_like = ["have", "has", "had"].contains(&word(t));
            if !object_start && !have_like && (next.is_none() || matches!(next, Some(Tok::PrepV(_)))) {
                out.push(format!(
                    "\"{}\" is transitive in angloform and needs an object",
                    word(t)
                ));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

// ------------------------------------------------ slot (redirect) findings --

fn is_det(t: &Tok) -> bool {
    matches!(t, Tok::Det(_) | Tok::DetSg(_) | Tok::Poss(_) | Tok::Num(_) | Tok::NumPl(_) | Tok::Every(_) | Tok::No(_) | Tok::Some_(_) | Tok::Ord(_) | Tok::Scale(_))
}

fn is_noun_head(t: &Tok) -> bool {
    matches!(t, Tok::NounSg(_) | Tok::NounPl(_) | Tok::Name(_) | Tok::Pron1(_) | Tok::Pron2(_))
}

/// A token after which the grammar expects a bare verb form (VBaseP):
/// negation, "yet" (ADR 0046), or a modal — the do-support/Prohibition/
/// ModalVP/Imperative positions, beyond the plain-SVO subject-head case.
fn introduces_bare_verb(t: &Tok) -> bool {
    matches!(t, Tok::Neg(_) | Tok::Yet(_) | Tok::ModalMust(_) | Tok::ModalCan(_) | Tok::ModalCannot(_))
}

/// A word in the wrong slot whose rejected sense has a redirect (ideas,
/// advice gap 2): "the cost" (cost is a verb; noun → expense), "the agent
/// files the report" (files is a noun; verb → submit).
fn slot_findings(lexicon: &Lexicon, toks: &[Tok]) -> Vec<String> {
    let mut out = Vec::new();
    // comparatives (ADR 0030): "more big" when an inflected form exists;
    // a comparative needs its standard ("than …")
    for (i, t) in toks.iter().enumerate() {
        if let Tok::More(_) = t {
            if let Some(Tok::Adj(a) | Tok::AdjLong(a)) = toks.get(i + 1) {
                if let Some(c) = lexicon.comparative(a) {
                    out.push(format!("\"more {a}\" — write \"{c}\" (short adjectives inflect, ADR 0030)"));
                }
            }
        }
        if matches!(t, Tok::AdjCmp(_)) && !matches!(toks.get(i + 1), Some(Tok::Than(_))) {
            out.push(format!("\"{}\" needs the standard: \"{} than <noun phrase>\" (ADR 0030)", word(t), word(t)));
        }
        // superlatives (ADR 0056): "most big" when an inflected form exists
        if let Tok::Most(_) = t {
            if let Some(Tok::Adj(a) | Tok::AdjLong(a)) = toks.get(i + 1) {
                if let Some(s) = lexicon.superlative(a) {
                    out.push(format!("\"most {a}\" — write \"{s}\" (short adjectives inflect, ADR 0056)"));
                }
            }
        }
        if matches!(t, Tok::Scale(_)) && !(i > 0 && matches!(toks[i - 1], Tok::NumPl(_))) {
            out.push(format!("\"{}\" follows digits: \"20 {}\" (ADR 0029)", word(t), word(t)));
        }
    }

    for (i, t) in toks.iter().enumerate() {
        let prev = i.checked_sub(1).map(|j| &toks[j]);
        let next = toks.get(i + 1);
        match t {
            // verb form in a noun slot: right after a determiner
            Tok::VtBase(w) | Tok::ViBase(w) | Tok::Vt3(w) | Tok::Vi3(w)
                if prev.is_some_and(is_det) =>
            {
                out.push(match lexicon.redirect(w, "NOUN") {
                    Some(r) => format!("\"{w}\" is a verb in angloform — {}", r.render("noun")),
                    None => format!("\"{w}\" is a verb in angloform and cannot follow a determiner"),
                });
            }
            // noun form in a verb slot: after a subject head (plain SVO), or
            // anywhere a bare verb form is grammatically expected — sentence
            // start (Imperative), after "not"/"yet", or after a modal — before
            // an object start. Broadened from subject-head-only (advice gap
            // #2): the redirect lookup itself is already position-independent
            // (keyed by lemma), only this trigger was too narrow.
            Tok::NounSg(w) | Tok::NounPl(w)
                if (prev.is_none() || prev.is_some_and(|p| is_noun_head(p) || introduces_bare_verb(p)))
                    && next.is_some_and(|n| is_det(n) || matches!(n, Tok::Adj(_) | Tok::AdjLong(_))) =>
            {
                if let Some(r) = lexicon.redirect(w, "VERB") {
                    out.push(format!("\"{w}\" is a noun in angloform — {}", r.render("verb")));
                }
            }
            _ => {}
        }
    }
    // inline list: "X are "it", "they", and "those"" — a comma-separated run
    // of noun phrases; lists are Enumeration blocks (ADR 0028)
    {
        let commas = toks.iter().filter(|t| matches!(t, Tok::Comma)).count();
        let comma_and = toks.windows(2).any(|w| matches!(w[0], Tok::Comma) && matches!(w[1], Tok::Conj(_)));
        if commas >= 2 || comma_and {
            out.push(
                "this is an inline list — write an Enumeration block: a statement ending in \
                 \":\" whose last noun phrase is plural or counted, then one \"- item\" per line \
                 (ADR 0028)"
                    .to_string(),
            );
        }
    }
    // a defined multi-word term written in lowercase ("reference ambiguity")
    // beats the compound rule below (ADR 0027)
    let mut term_spans: Vec<(usize, usize)> = Vec::new();
    for len in [3usize, 2] {
        for i in 0..toks.len().saturating_sub(len - 1) {
            if term_spans.iter().any(|&(s, e)| i < e && i + len > s) {
                continue;
            }
            let ws: Vec<&str> = toks[i..i + len].iter().map(word).collect();
            if ws.iter().any(|w| *w == "," || w.starts_with('"')) {
                continue;
            }
            let joined = ws.join(" ").to_lowercase();
            if let Some(cap) = lexicon.term(&joined) {
                out.push(format!("\"{joined}\" is a defined term — write \"{cap}\" (see CONTEXT.md)"));
                term_spans.push((i, i + len));
            }
        }
    }
    // noun-noun compound (ADR 0015): two nouns in a row, unless the second
    // is a noun form used in the verb slot (handled above) or the pair is a
    // defined term (handled just above)
    for (i, w) in toks.windows(2).enumerate() {
        if let (Tok::NounSg(a), Tok::NounSg(b) | Tok::NounPl(b)) = (&w[0], &w[1]) {
            if term_spans.iter().any(|&(s, e)| i >= s && i + 1 < e) {
                continue;
            }
            let verb_slot = toks.get(i + 2).is_some_and(|n| is_det(n) || matches!(n, Tok::Adj(_) | Tok::AdjLong(_)))
                && lexicon.redirect(b, "VERB").is_some();
            if !verb_slot {
                out.push(format!(
                    "\"{a} {b}\" — noun-noun compounds are not angloform; write \"the {b} of the {a}\", \
                     or one transparent word (ADR 0015)"
                ));
            }
        }
    }
    out
}

// --------------------------------------------------- tier-2 loose parser --

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Term {
    Det, DetSg, Poss, Every, No, Some_, Num, Adj, NSg, NPl,
    VAny, PrepN, PrepV, Pron, CopAny, Conj, Neg, TempAdv, DoAny, ModAny, If, Then, Comma,
    Ing, Ed, NameT, Pct, Approx, So, Because, Namely, Ord, Than,
}

fn term_of(t: &Tok) -> Vec<Term> {
    match t {
        Tok::Det(_) => vec![Term::Det],
        Tok::DetSg(_) => vec![Term::DetSg],
        Tok::Poss(_) => vec![Term::Poss],
        Tok::Every(_) => vec![Term::Every],
        Tok::No(_) => vec![Term::No],
        Tok::Some_(_) => vec![Term::Some_],
        Tok::Num(_) | Tok::NumPl(_) | Tok::NumVal(_) => vec![Term::Num],
        Tok::Percent(_) => vec![Term::Pct],
        Tok::Approx(_) => vec![Term::Approx],
        Tok::So(_) => vec![Term::So],
        Tok::Because(_) => vec![Term::Because],
        Tok::Namely(_) => vec![Term::Namely],
        Tok::Ord(_) => vec![Term::Ord],
        Tok::Be(_) | Tok::BecomeSg(_) | Tok::BecomePl(_) | Tok::BecomePast(_) => vec![Term::CopAny],
        Tok::Than(_) => vec![Term::Than],
        Tok::More(_) | Tok::Scale(_) | Tok::AdjCmp(_) | Tok::AdjLong(_) | Tok::Adj(_)
        | Tok::AdjSup(_) | Tok::Most(_) => vec![Term::Adj],
        Tok::NounSg(_) => vec![Term::NSg],
        Tok::NounPl(_) => vec![Term::NPl],
        Tok::VtBase(_) | Tok::Vt3(_) | Tok::ViBase(_) | Tok::Vi3(_) => vec![Term::VAny],
        Tok::VtEd(_) | Tok::ViEd(_) => vec![Term::VAny, Term::Ed],
        Tok::VtIng(_) | Tok::ViIng(_) => vec![Term::Ing],
        Tok::PrepN(_) => vec![Term::PrepN],
        Tok::PrepV(_) => vec![Term::PrepV],
        Tok::Pron1(_) | Tok::Pron2(_) => vec![Term::Pron],
        Tok::CopSg(_) | Tok::CopPl(_) | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::Cop1Sg(_) => vec![Term::CopAny],
        Tok::Conj(_) => vec![Term::Conj],
        Tok::Neg(_) => vec![Term::Neg],
        Tok::TempAdv(_) => vec![Term::TempAdv],
        Tok::TimeAdv(_) => vec![Term::PrepV],
        Tok::Yet(_) => vec![Term::Neg],
        Tok::Focus(_) => vec![Term::Det],
        Tok::Other(_) => vec![Term::Det],
        Tok::DoBase(_) | Tok::Do3(_) | Tok::DoPast(_) => vec![Term::DoAny],
        Tok::ModalMust(_) | Tok::ModalCan(_) | Tok::ModalCannot(_) => vec![Term::ModAny],
        Tok::If(_) => vec![Term::If],
        Tok::Then(_) => vec![Term::Then],
        Tok::Name(_) => vec![Term::NameT],
        Tok::Comma => vec![Term::Comma],
        Tok::Colon => vec![Term::Comma],
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Sym {
    T(Term),
    N(usize),
}

// Nonterminal ids
const S2: usize = 0;
const CL: usize = 1;
const NPS: usize = 2;
const DETX: usize = 3;
const ADJS: usize = 4;
const NH: usize = 5;
const PPS: usize = 6;
const PP: usize = 7;
const PRED: usize = 8;
const VPX: usize = 9;
const COMPL: usize = 10;
const N_NONTERMS: usize = 11;

fn productions() -> Vec<Vec<Vec<Sym>>> {
    use Sym::{N, T};
    let mut p = vec![Vec::new(); N_NONTERMS];
    p[S2] = vec![
        vec![N(CL)],
        // full-clause coordination (ADR 0037): comma mandatory; the
        // comma-free shape is kept too as the "dirty" reading a writer
        // reaches for, so it still gets a named Style advice instead of
        // falling through as Unknown.
        vec![N(CL), T(Term::Comma), T(Term::Conj), N(CL)],
        vec![N(CL), T(Term::Conj), N(CL)],
        vec![N(CL), T(Term::Conj), N(VPX)],
        vec![T(Term::If), N(CL), T(Term::Comma), T(Term::Then), N(CL)],
        vec![T(Term::If), N(CL), T(Term::Then), N(CL)],
        vec![T(Term::If), N(CL), T(Term::Comma), N(CL)],
        vec![N(CL), T(Term::If), N(CL)],
        vec![T(Term::DoAny), T(Term::Neg), N(VPX)],
        vec![N(VPX)], // bare imperative
        // causal (ADR 0026) and its dirty shapes: missing comma, fronted because
        vec![N(CL), T(Term::Comma), T(Term::So), N(CL)],
        vec![N(CL), T(Term::Comma), T(Term::Because), N(CL)],
        vec![N(CL), T(Term::So), N(CL)],
        vec![N(CL), T(Term::Because), N(CL)],
        vec![T(Term::Because), N(CL), T(Term::Comma), N(CL)],
        vec![T(Term::So), N(CL)],
    ];
    p[CL] = vec![vec![N(NPS), N(PRED)]];
    p[NPS] = vec![
        vec![N(DETX), N(ADJS), N(NH), N(PPS)],
        vec![T(Term::Num), T(Term::Pct), N(PPS)], // 43 percent of the swaps
        vec![T(Term::Approx), T(Term::Num), T(Term::Pct), N(PPS)],
        vec![N(ADJS), T(Term::NPl), N(PPS)],
        vec![T(Term::NSg), N(PPS)], // bare singular (dirty)
        vec![T(Term::Pron)],
        vec![T(Term::NameT)],
        vec![N(DETX), N(ADJS), N(NH), T(Term::NameT), N(PPS)],
    ];
    p[DETX] = vec![
        vec![T(Term::Det)], vec![T(Term::DetSg)], vec![T(Term::Poss)],
        vec![T(Term::Every)], vec![T(Term::No)], vec![T(Term::Some_)], vec![T(Term::Num)],
        vec![T(Term::Approx), T(Term::Num)], vec![T(Term::Det), T(Term::Ord)],
    ];
    p[ADJS] = vec![vec![], vec![T(Term::Adj), N(ADJS)]];
    p[NH] = vec![vec![T(Term::NSg)], vec![T(Term::NPl)]];
    p[PPS] = vec![vec![], vec![N(PP), N(PPS)]];
    p[PP] = vec![
        vec![T(Term::PrepN), N(NPS)],
        vec![T(Term::PrepV), N(NPS)], // under NP this is the classic dirty attachment
    ];
    p[PRED] = vec![
        vec![T(Term::VAny), N(PPS)],
        vec![T(Term::VAny), N(NPS), N(PPS)],
        vec![T(Term::CopAny), N(COMPL)],
        vec![T(Term::CopAny), T(Term::Neg), N(COMPL)],
        vec![T(Term::DoAny), T(Term::Neg), N(VPX)],
        vec![T(Term::ModAny), N(VPX)],
        vec![T(Term::ModAny), T(Term::Neg), N(VPX)],
        vec![T(Term::ModAny), T(Term::CopAny), N(COMPL)],
        vec![T(Term::ModAny), T(Term::Neg), T(Term::CopAny), N(COMPL)],
    ];
    p[VPX] = vec![
        vec![T(Term::VAny), N(PPS)],
        vec![T(Term::VAny), N(NPS), N(PPS)],
        vec![T(Term::Ing), N(PPS)],
        vec![T(Term::Ing), N(NPS), N(PPS)],
    ];
    p[COMPL] = vec![
        vec![T(Term::Adj)],
        vec![T(Term::Adj), T(Term::Than), N(NPS)], // comparative + standard
        vec![T(Term::Adj), T(Term::Adj), T(Term::Than), N(NPS)], // more <adj> than
        vec![N(NPS)],
        vec![T(Term::Ed), N(PPS)],  // passive shape (dirty)
        vec![T(Term::Ing), N(PPS)], // progressive shape (dirty)
        vec![T(Term::Ing), N(NPS), N(PPS)],
    ];
    p
}

pub struct Tier2 {
    terms: Vec<Vec<Term>>,
    prods: Vec<Vec<Vec<Sym>>>,
    memo: BTreeMap<(usize, usize), BTreeMap<usize, u64>>,
}

impl Tier2 {
    pub fn new(toks: &[Tok]) -> Tier2 {
        Tier2 {
            terms: toks.iter().map(term_of).collect(),
            prods: productions(),
            memo: BTreeMap::new(),
        }
    }

    /// Number of distinct tier-2 parse trees for the whole token stream.
    pub fn count(&mut self) -> u64 {
        let n = self.terms.len();
        if n == 0 {
            return 0;
        }
        *self.nonterm(S2, 0).get(&n).unwrap_or(&0)
    }

    /// end position → number of derivations of `nt` starting at `i`.
    fn nonterm(&mut self, nt: usize, i: usize) -> BTreeMap<usize, u64> {
        if let Some(m) = self.memo.get(&(nt, i)) {
            return m.clone();
        }
        // no left recursion in the grammar, so seeding with empty is sound
        self.memo.insert((nt, i), BTreeMap::new());
        let mut result: BTreeMap<usize, u64> = BTreeMap::new();
        let prods = self.prods[nt].clone();
        for prod in prods {
            let mut ends: BTreeMap<usize, u64> = BTreeMap::from([(i, 1)]);
            for sym in prod {
                let mut next: BTreeMap<usize, u64> = BTreeMap::new();
                for (&pos, &cnt) in &ends {
                    match sym {
                        Sym::T(term) => {
                            if pos < self.terms.len() && self.terms[pos].contains(&term) {
                                *next.entry(pos + 1).or_default() += cnt;
                            }
                        }
                        Sym::N(sub) => {
                            for (&end, &c2) in &self.nonterm(sub, pos) {
                                *next.entry(end).or_default() += cnt * c2;
                            }
                        }
                    }
                }
                ends = next;
                if ends.is_empty() {
                    break;
                }
            }
            for (end, cnt) in ends {
                *result.entry(end).or_default() += cnt;
            }
        }
        self.memo.insert((nt, i), result.clone());
        result
    }
}
