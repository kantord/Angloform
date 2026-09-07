//! angloform linter compiled to wasm32-unknown-unknown for the static web
//! playground (see `web/`). The lexicon is embedded at compile time, so no
//! filesystem access is needed at runtime.

use diagnose::{diagnose, Diagnosis};
use grammar::{cases, parse_text, Case, Lexicon, Metrics, Tok, Tree};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::LazyLock;
use wasm_bindgen::prelude::*;

static LEXICON: LazyLock<Lexicon> = LazyLock::new(|| {
    Lexicon::from_tsv(include_str!("../../../lexicon.tsv")).expect("embedded lexicon.tsv")
});

/// Lint one sentence (or Enumeration / Step Block) and return a JSON
/// document:
///
/// ```json
/// {
///   "kind": "Clean" | "Word" | "Style" | "Ambiguous" | "Unknown",
///   "messages": ["..."],
///   "readings": 3,
///   "metrics": { ... },
///   "tree": { "nodes": [...] },
///   "tokens": [{ "pos": 0, "word": "the", "tag": "DET" }]
/// }
/// ```
#[wasm_bindgen]
pub fn diagnose_sentence(sentence: &str) -> String {
    let tokens = tokenize_json(sentence);
    let payload = match diagnose(&LEXICON, sentence) {
        Diagnosis::Clean(metrics) => {
            let tree = parse_text(&LEXICON, sentence).ok();
            json!({
                "kind": "Clean",
                "metrics": metrics_json(metrics),
                "tree": tree.map(|t| tree_json(&t)),
            })
        }
        Diagnosis::Word(msg) => json!({ "kind": "Word", "messages": vec![msg] }),
        Diagnosis::Style(findings) => json!({ "kind": "Style", "messages": findings }),
        Diagnosis::Ambiguous { readings, findings } => json!({
            "kind": "Ambiguous",
            "readings": readings,
            "messages": findings,
        }),
        Diagnosis::Unknown => json!({ "kind": "Unknown", "messages": [] }),
    };
    let mut out = payload.as_object().cloned().unwrap_or_default();
    out.insert("tokens".to_string(), tokens);
    serde_json::to_string(&Value::Object(out)).expect("json")
}

/// Split a sentence into tokens and return them as JSON:
/// `[{ "pos": 0, "word": "the", "tag": "DET" }, ...]`。
 /// A token that the lexicon does not know is returned with the tag `"?"`。
#[wasm_bindgen]
pub fn tokenize(sentence: &str) -> String {
    serde_json::to_string(&tokenize_json(sentence)).expect("json")
}

fn tokenize_json(sentence: &str) -> Value {
    let toks = LEXICON.tokenize(sentence);
    match toks {
        Ok(toks) => {
            let arr: Vec<Value> = toks
                .into_iter()
                .map(|(pos, t)| json!({ "pos": pos, "word": word_of(&t), "tag": tag_of(&t) }))
                .collect();
            Value::Array(arr)
        }
        Err(e) => Value::Array(vec![json!({ "pos": e.position, "word": e.word, "tag": "?" })]),
    }
}

fn word_of(t: &Tok) -> &str {
    use Tok::*;
    match t {
        Det(w) | DetSg(w) | Adj(w) | NounSg(w) | NounPl(w) | VtBase(w) | Vt3(w) | VtEd(w)
        | VtIng(w) | ViBase(w) | Vi3(w) | ViEd(w) | ViIng(w) | PrepN(w) | PrepV(w)
        | Pron1(w) | Pron2(w) | Poss(w) | CopSg(w) | CopPl(w) | CopSgPast(w)
        | CopPlPast(w) | Cop1Sg(w) | Conj(w) | Neg(w) | TempAdv(w) | TimeAdv(w) | Yet(w) | Focus(w) | Other(w) | DoBase(w) | Do3(w) | DoPast(w)
        | ModalMust(w) | ModalCan(w) | ModalCannot(w) | If(w) | Then(w) | Every(w)
        | No(w) | Num(w) | NumPl(w) | Percent(w) | Approx(w) | So(w) | Because(w) | Namely(w) | Which(w) | Who(w)
        | Ord(w) | Than(w) | More(w) | Scale(w) | AdjCmp(w) | AdjLong(w) | AdjSup(w) | Most(w) | NumVal(w) | Be(w)
        | BecomeSg(w) | BecomePl(w) | BecomePast(w) | Some_(w) | Name(w) => w,
        Comma => ",",
        Colon => ":",
    }
}

/// The Form Tag of a token, e.g. `"NOUN_SG"` (matches the Lexicon).
fn tag_of(t: &Tok) -> &'static str {
    use Tok::*;
    match t {
        Det(_) => "DET",
        DetSg(_) => "DET_SG",
        Adj(_) => "ADJ",
        NounSg(_) => "NOUN_SG",
        NounPl(_) => "NOUN_PL",
        VtBase(_) => "VERB_TRANS_BASE",
        Vt3(_) => "VERB_TRANS_3SG",
        VtEd(_) => "VERB_TRANS_ED",
        VtIng(_) => "VERB_TRANS_ING",
        ViBase(_) => "VERB_INTRANS_BASE",
        Vi3(_) => "VERB_INTRANS_3SG",
        ViEd(_) => "VERB_INTRANS_ED",
        ViIng(_) => "VERB_INTRANS_ING",
        PrepN(_) => "PREP_N",
        PrepV(_) => "PREP_V",
        Pron1(_) => "PRON_1",
        Pron2(_) => "PRON_2",
        Poss(_) => "POSS",
        CopSg(_) => "COPULA_SG",
        CopPl(_) => "COPULA_PL",
        CopSgPast(_) => "COPULA_SG_PAST",
        CopPlPast(_) => "COPULA_PL_PAST",
        Cop1Sg(_) => "COPULA_1SG",
        Conj(_) => "CONJ",
        Neg(_) => "NEG",
        TempAdv(_) => "TEMP_ADV",
        TimeAdv(_) => "TIME_ADV",
        Yet(_) => "YET",
        Focus(_) => "FOCUS",
        Other(_) => "OTHER",
        DoBase(_) => "NEG_AUX_BASE",
        Do3(_) => "NEG_AUX_3SG",
        DoPast(_) => "NEG_AUX_PAST",
        ModalMust(_) => "MODAL_MUST",
        ModalCan(_) => "MODAL_CAN",
        ModalCannot(_) => "MODAL_CAN_NEG",
        If(_) => "SCONJ_COND",
        Then(_) => "THEN",
        Every(_) => "QUANT_UNIV",
        No(_) => "QUANT_NEG",
        Num(_) => "NUM_SG",
        NumPl(_) => "NUM_PL",
        Percent(_) => "PERCENT",
        Approx(_) => "APPROX",
        So(_) => "RESULT",
        Because(_) => "REASON",
        Namely(_) => "NAMELY",
        Which(_) => "WHICH",
        Who(_) => "WHO",
        Ord(_) => "ORD",
        Than(_) => "THAN",
        More(_) => "MORE",
        Scale(_) => "SCALE",
        AdjCmp(_) => "ADJ_CMP",
        AdjLong(_) => "ADJ_LONG",
        AdjSup(_) => "ADJ_SUP",
        Most(_) => "MOST",
        NumVal(_) => "NUM_VAL",
        Be(_) => "BE",
        BecomeSg(_) => "BECOME_SG",
        BecomePl(_) => "BECOME_PL",
        BecomePast(_) => "BECOME_PAST",
        Some_(_) => "QUANT_EXIST",
        Name(_) => "NAME",
        Comma => "COMMA",
        Colon => "COLON",
    }
}

fn metrics_json(m: Metrics) -> Value {
    json!({
        "peak_open_deps": m.peak_open_deps,
        "max_dep_len": m.max_dep_len,
        "embedding_depth": m.embedding_depth,
        "right_branching": m.right_branching,
        "fronted": m.fronted,
    })
}

/// Flatten a parse tree into rows for the tree chart:
/// `{ "nodes": [{ "id", "parentId", "name", "kind", "head" }] }`。
 /// `kind` is `"node"` for a phrase/sentence and `"word"` for a leaf; `head`
/// marks the child that is its parent's syntactic head.
/// Flatten a parse tree into rows for the tree chart:
/// `{ "nodes": [{ "id", "parentId", "name", "kind", "head", "lemma", "gloss", "full" }] }`。
/// `kind` is `"node"` for a phrase/sentence and `"word"` for a leaf; `head`
/// marks the child that is its parent's syntactic head. Node names are full
/// phrase names (no abbreviations); word leaves carry the lemma, a short
/// gloss (the definition of its meaning, when the domain model has one),
/// and the full definition for tooltips.

/// Grammatical case (ADR: `cases()` is a pure derived view of the tree —
/// position already determines role, so this is display only, not new
/// grammar) — Nominative/Accusative/Genitive/Oblique/Complement.
fn case_name(c: Case) -> &'static str {
    match c {
        Case::Nominative => "Nominative",
        Case::Accusative => "Accusative",
        Case::Genitive => "Genitive",
        Case::Oblique => "Oblique",
        Case::Complement => "Complement",
    }
}

fn tree_json(tree: &Tree) -> Value {
    let case_map: BTreeMap<usize, Case> = cases(tree).into_iter().collect();
    let mut nodes: Vec<Value> = Vec::new();
    fn walk(tree: &Tree, parent: Option<&str>, head: bool, case_map: &BTreeMap<usize, Case>, nodes: &mut Vec<Value>) -> String {
        let id = format!("n{}", nodes.len());
        match tree {
            Tree::Leaf { pos, word } => {
                let (gloss, full) = gloss_for(word);
                let lemma = LEXICON.lemma_of(word).unwrap_or(word).to_lowercase();
                let case = case_map.get(pos).map(|c| case_name(*c));
                nodes.push(json!({
                    "id": id, "parentId": parent, "name": word, "kind": "word", "head": head,
                    "lemma": lemma, "tag": tag_of_lex(word), "gloss": gloss, "full": full,
                    "case": case,
                }));
            }
            Tree::Node { label, head: h, children } => {
                nodes.push(json!({
                    "id": id, "parentId": parent, "name": label_name(label), "kind": "node", "head": head,
                }));
                for (i, c)in children.iter().enumerate() {
                    walk(c, Some(&id), i == *h, case_map, nodes);
                }
            }
        }
        id
    }
    walk(tree, None, false, &case_map, &mut nodes);
    json!({ "nodes": nodes })
}

/// Full descriptive phrase name for a grammar node label (no abbreviations).
fn label_name(label: &str) -> &str {
    match label {
        "S" => "Statement",
        "Clause" => "Clause",
        "Cond" => "Conditional",
        "Causal" => "Causal",
        "Prohib" => "Prohibition",
        "Imp" => "Imperative",
        "NP" => "Noun Phrase",
        "NPGen" => "Bare Plural",
        "NPOnly" => "Noun Phrase with \"only\"",
        "NPOther" => "Noun Phrase with \"other\"",
        "NPAppos" => "Noun Phrase with Name",
        "NPPct" => "Percent Share",
        "Value" => "Measurement Value",
        "OfPP" => "of Phrase",
        "PP" => "Prepositional Phrase",
        "VP" => "Verb Phrase",
        "NegVP" => "Negated Verb Phrase",
        "ModalVP" => "Modal Verb Phrase",
        "ModalCop" => "Modal Copula",
        "CopPred" => "Copular Predicate",
        "PredAdv" => "Predicate with Adverb",
        "ComplPP" => "Complement",
        "Cmp" => "Comparative",
        "CoordPred" => "Predicate Coordination",
        "CoordClause" => "Clause Coordination",
        "RelS" => "Relative Clause",
        "Intro" => "Enumeration Intro",
        "Enum" => "Enumeration",
        "Steps" => "Step Block",
        _ => label,
    }
}

fn tag_of_lex(word: &str) -> &str {
    LEXICON.tag_of(word).unwrap_or("")
}

/// Short display gloss for a surface form (the definition of its meaning,
/// when the domain model has one), plus the full definition for tooltips.


fn gloss_for(form: &str) -> (String, Option<String>) {
    let lemma = LEXICON.lemma_of(form).unwrap_or(form).to_lowercase();
    if let Some(full) = DOMAIN_DEFS.get(&lemma) {
        return (truncate(full, 56), Some(full.clone()));
    }
    let cat = SEED_CATS.get(&lemma).cloned().unwrap_or_else(|| tag_category(tag_of_lex(form)));
    let mut gloss = cat_phrase(&cat).to_string();
    if lemma != form.to_lowercase() {
        gloss.push_str(&format!(" ({lemma})"));
    }
    if let Some(note) = SEED_NOTES.get(&lemma) {
        gloss.push_str(&format!(" — {}", truncate(note, 32)));
    }
    (truncate(&gloss, 64), None)
}

/// Form Tag → a short category phrase (e.g. "VERB_TRANS_3SG" → "a
/// transitive verb"), used when the seed does not know the lemma.


fn tag_category(tag: &str) -> String {
    if tag.starts_with("VERB_TRANS") {
        "VERB_TRANS".to_string()
    } else if tag.starts_with("VERB_INTRANS") {
        "VERB_INTRANS".to_string()
    } else if tag.starts_with("COPULA") {
        "COPULA".to_string()
    } else if tag.starts_with("NEG_AUX") {
        "NEG_AUX".to_string()
    } else if tag.starts_with("MODAL") {
        tag.to_string()
    } else {
        tag.to_string()
    }
}

fn cat_phrase(cat: &str) -> &str {
    match cat {
        "DET" => "a determiner",
        "DET_SG" => "a determiner (singular)",
        "ADJ" => "an adjective",
        "NOUN" => "a noun",
        "VERB_TRANS" => "a transitive verb",
        "VERB_INTRANS" => "an intransitive verb",
        "PREP_N" => "a noun preposition",
        "PREP_V" => "a verb preposition",
        "PRON_1SG" => "a Pronoun (writer)",
        "PRON_2" => "a Pronoun (reader)",
        "POSS_1SG" => "a possessive",
        "POSS_2" => "a possessive",
        "COPULA" => "a Copula",
        "CONJ" => "a conjunction",
        "NEG" => "a negation",
        "NEG_AUX" => "an Auxiliary",
        "MODAL_MUST" => "a modal (must)",
        "MODAL_CAN" => "a modal (can)",
        "MODAL_CAN_NEG" => "a modal (cannot)",
        "SCONJ_COND" => "a function word (if)",
        "THEN" => "a function word (then)",
        "QUANT_UNIV" => "a quantifier (every)",
        "QUANT_NEG" => "a quantifier (no)",
        "QUANT_EXIST" => "a quantifier (some)",
        "NUM_SG" => "a number word (one)",
        "PERCENT" => "a percent sign",
        "APPROX" => "an approximator",
        "RESULT" => "a causal connective (so)",
        "REASON" => "a causal connective (because)",
        "ORD" => "an ordinal",
        "THAN" => "a comparative marker (than)",
        "MORE" => "a comparative marker (more)",
        "SCALE" => "a scale word",
        "ADJ_CMP" => "an adjective (comparative)",
        "ADJ_LONG" => "an adjective",
        "BE" => "a Copula (be)",
        "BECOME_SG" => "a Copula (become)",
        "BECOME_PL" => "a Copula (become)",
        "BECOME_PAST" => "a Copula (become)",
        "NAME" => "a Name",
        "COMMA" => "a comma",
        "COLON" => "a colon",
        _ => "a word",
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut cut = max;
        while cut > 0 && !s.is_char_boundary(cut) { cut -= 1; }
        let mut out: String = s[..cut].to_string();
        out.push('…');
        out
    }
}

static DOMAIN_DEFS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let model: Value = serde_json::from_str(include_str!("../../../domain/model.json")).expect("domain/model.json");
    let mut defs = BTreeMap::new();
    for e in model.as_array().into_iter().flatten() {
        if let (Some(lemma), Some(def)) = (e["lemma"].as_str(), e["definition"].as_str()) {
            defs.insert(lemma.to_lowercase(), def.to_string());
        }
    }
    defs
});

static SEED_CATS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let seed: Value = serde_json::from_str(include_str!("../../../seed/seed.json")).expect("seed/seed.json");
    let mut cats = BTreeMap::new();
    for e in seed.as_array().into_iter().flatten() {
        if let Some(lemma) = e["lemma"].as_str() {
            if let Some(cat) = e["category"].as_str() {
                cats.insert(lemma.to_lowercase(), cat.to_string());
            }
        }
    }
    cats
});

static SEED_NOTES: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let seed: Value = serde_json::from_str(include_str!("../../../seed/seed.json")).expect("seed/seed.json");
    let mut notes = BTreeMap::new();
    for e in seed.as_array().into_iter().flatten() {
        if let (Some(lemma), Some(note)) = (e["lemma"].as_str(), e["note"].as_str()) {
            notes.insert(lemma.to_lowercase(), note.to_string());
        }
    }
    notes
});