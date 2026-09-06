//! triage — run the angloform lexicon against a pre-tagged corpus (CoNLL-U)
//! and report how far real English sentences are from being expressible.
//!
//! Token verdicts:
//!   OK          surface is enabled and its gold UPOS matches the enabled category
//!   REDIRECT    gold UPOS is a rejected use of an enabled lemma — the writer
//!               would get a precise "use X instead" error (the designed case)
//!   MISMATCH    gold UPOS conflicts with the enabled category and no redirect
//!               covers it — a hole in the redirect table
//!   PROPN       proper noun — out of scope for the lexicon by design
//!   OOV         not in the lexicon at all (bucketed by gold UPOS)
//!
//! Usage: cargo run -p triage [-- path/to/corpus.conllu]

use std::collections::BTreeMap;

const LEXICON_PATH: &str = "lexicon.tsv";
const DEFAULT_CORPUS: &str = "data/ud/en_ewt-ud-test.conllu";
const REPORT_PATH: &str = "docs/triage-report.md";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Verdict {
    Ok,
    Redirect,
    Mismatch,
    Propn,
    Oov,
}

struct Token {
    form: String,
    lemma: String,
    upos: String,
}

struct Sentence {
    text: String,
    tokens: Vec<Token>,
}

struct Lexicon {
    /// surface form → lexicon POS family ("NOUN", "VERB", "ADJ", "ADP", "DET", …)
    surface_pos: BTreeMap<String, String>,
    /// (lemma, rejected POS) → suggestion
    rejects: BTreeMap<(String, String), String>,
}

fn main() {
    let corpus_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CORPUS.to_string());
    let lexicon = load_lexicon(LEXICON_PATH);
    let sentences = load_conllu(&corpus_path);

    let mut token_counts: BTreeMap<Verdict, usize> = BTreeMap::new();
    let mut oov_by_upos: BTreeMap<String, usize> = BTreeMap::new();
    let mut missing_lemmas: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut redirect_hits: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut mismatches: BTreeMap<(String, String), usize> = BTreeMap::new();

    // sentence buckets by number of problem tokens (everything not OK)
    let mut perfect: Vec<&Sentence> = Vec::new();
    let mut close: Vec<(&Sentence, Vec<String>)> = Vec::new(); // ≤2 problems
    let mut far = 0usize;

    for s in &sentences {
        let mut problems: Vec<String> = Vec::new();
        for t in &s.tokens {
            let v = judge(t, &lexicon);
            *token_counts.entry(v).or_default() += 1;
            match v {
                Verdict::Ok => {}
                Verdict::Redirect => {
                    let key = (t.lemma.clone(), t.upos.clone());
                    *redirect_hits.entry(key).or_default() += 1;
                    let sugg = lexicon
                        .rejects
                        .get(&(t.lemma.clone(), reject_key(&t.upos).to_string()))
                        .cloned()
                        .unwrap_or_default();
                    problems.push(format!("{} as {} → \"{sugg}\"", t.form, t.upos));
                }
                Verdict::Mismatch => {
                    *mismatches
                        .entry((t.lemma.clone(), t.upos.clone()))
                        .or_default() += 1;
                    problems.push(format!("{} as {} (no redirect)", t.form, t.upos));
                }
                Verdict::Propn => problems.push(format!("{} (proper noun)", t.form)),
                Verdict::Oov => {
                    *oov_by_upos.entry(t.upos.clone()).or_default() += 1;
                    *missing_lemmas
                        .entry((t.lemma.clone(), t.upos.clone()))
                        .or_default() += 1;
                    problems.push(format!("{} ({}, OOV)", t.form, t.upos));
                }
            }
        }
        match problems.len() {
            0 => perfect.push(s),
            1..=2 => close.push((s, problems)),
            _ => far += 1,
        }
    }

    let report = render(
        &corpus_path,
        &sentences,
        &token_counts,
        &oov_by_upos,
        &missing_lemmas,
        &redirect_hits,
        &mismatches,
        &perfect,
        &close,
        far,
    );
    std::fs::write(REPORT_PATH, &report).expect("write report");
    println!(
        "triage: {} sentences → {} perfect, {} close (≤2 problems), {} far; report in {REPORT_PATH}",
        sentences.len(),
        perfect.len(),
        close.len(),
        far
    );
}

fn judge(t: &Token, lex: &Lexicon) -> Verdict {
    let form = t.form.to_lowercase();
    let lemma = t.lemma.to_lowercase();
    if t.upos == "PROPN" {
        return Verdict::Propn;
    }
    if let Some(enabled) = lex.surface_pos.get(&form) {
        if upos_family(&t.upos) == Some(enabled.as_str()) {
            return Verdict::Ok;
        }
        // UD lumps personal and possessive pronouns under PRON
        if t.upos == "PRON" && (enabled.starts_with("PRON_") || enabled.starts_with("POSS_")) {
            return Verdict::Ok;
        }
        // UD tags copular/auxiliary be as AUX; token-level checking cannot
        // distinguish the (allowed) copula from the (banned) aux construction
        if t.upos == "AUX"
            && (enabled.starts_with("COPULA")
                || enabled.starts_with("NEG_AUX")
                || enabled.starts_with("MODAL"))
        {
            return Verdict::Ok;
        }
        if t.upos == "CCONJ" && enabled == "CONJ" {
            return Verdict::Ok;
        }
        if t.upos == "PART" && enabled == "NEG" {
            return Verdict::Ok;
        }
        if t.upos == "SCONJ" && enabled == "SCONJ_COND" {
            return Verdict::Ok;
        }
        if t.upos == "ADV" && enabled == "THEN" {
            return Verdict::Ok;
        }
        if t.upos == "DET" && enabled.starts_with("QUANT") {
            return Verdict::Ok;
        }
    }
    // not usable as-is: is this a rejected use of an enabled lemma?
    let rk = reject_key(&t.upos);
    if !rk.is_empty() && lex.rejects.contains_key(&(lemma.clone(), rk.to_string())) {
        return Verdict::Redirect;
    }
    // enabled surface or lemma used in a conflicting, uncovered POS
    if lex.surface_pos.contains_key(&form) || lex.surface_pos.contains_key(&lemma) {
        return Verdict::Mismatch;
    }
    Verdict::Oov
}

/// Map a UD UPOS to the lexicon's POS family it would have to match.
fn upos_family(upos: &str) -> Option<&'static str> {
    match upos {
        "NOUN" => Some("NOUN"),
        "VERB" => Some("VERB"),
        "ADJ" => Some("ADJ"),
        "ADP" => Some("ADP"),
        "DET" => Some("DET"),
        _ => None,
    }
}

/// Map a UD UPOS to the reject-table key used in seed.json.
fn reject_key(upos: &str) -> &'static str {
    match upos {
        "NOUN" => "NOUN",
        "VERB" => "VERB",
        "ADJ" => "ADJ",
        "ADV" => "ADV",
        _ => "",
    }
}

fn load_lexicon(path: &str) -> Lexicon {
    let text = std::fs::read_to_string(path).expect("lexicon.tsv — run lexgen first");
    let mut lex = Lexicon {
        surface_pos: BTreeMap::new(),
        rejects: BTreeMap::new(),
    };
    for line in text.lines() {
        if line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        let [surface, kind, tag, value] = f[..] else { continue };
        match kind {
            "form" => {
                let family = if tag.starts_with("NOUN") {
                    "NOUN"
                } else if tag.starts_with("VERB") {
                    "VERB"
                } else if tag == "ADJ" {
                    "ADJ"
                } else if tag.starts_with("PREP") {
                    "ADP"
                } else if tag.starts_with("DET") {
                    "DET"
                } else {
                    tag
                };
                lex.surface_pos.insert(surface.to_string(), family.to_string());
            }
            "reject" => {
                lex.rejects.insert(
                    (surface.to_string(), tag.to_string()),
                    value.to_string(),
                );
            }
            _ => {}
        }
    }
    lex
}

fn load_conllu(path: &str) -> Vec<Sentence> {
    let text = std::fs::read_to_string(path).expect("corpus file");
    let mut out = Vec::new();
    let mut cur: Vec<Token> = Vec::new();
    let mut cur_text = String::new();
    for line in text.lines() {
        if line.is_empty() {
            if !cur.is_empty() {
                out.push(Sentence {
                    text: std::mem::take(&mut cur_text),
                    tokens: std::mem::take(&mut cur),
                });
            }
            continue;
        }
        if let Some(t) = line.strip_prefix("# text = ") {
            cur_text = t.to_string();
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 || f[0].contains('-') || f[0].contains('.') {
            continue; // multi-word ranges and empty nodes
        }
        let upos = f[3];
        if upos == "PUNCT" || upos == "SYM" || upos == "X" {
            continue;
        }
        cur.push(Token {
            form: f[1].to_string(),
            lemma: f[2].to_string(),
            upos: upos.to_string(),
        });
    }
    if !cur.is_empty() {
        out.push(Sentence {
            text: cur_text,
            tokens: cur,
        });
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn render(
    corpus_path: &str,
    sentences: &[Sentence],
    token_counts: &BTreeMap<Verdict, usize>,
    oov_by_upos: &BTreeMap<String, usize>,
    missing_lemmas: &BTreeMap<(String, String), usize>,
    redirect_hits: &BTreeMap<(String, String), usize>,
    mismatches: &BTreeMap<(String, String), usize>,
    perfect: &[&Sentence],
    close: &[(&Sentence, Vec<String>)],
    far: usize,
) -> String {
    let total_tokens: usize = token_counts.values().sum();
    let pct = |n: usize, d: usize| format!("{:.1}%", 100.0 * n as f64 / d.max(1) as f64);
    let count = |v: Verdict| *token_counts.get(&v).unwrap_or(&0);

    let mut out = String::from(
        "# Triage report\n\n*Generated by triage — do not edit.*\n\n\
         *Sample sentences quoted below are from UD_English-EWT (r2.16), \
         © the UD English-EWT contributors, licensed CC BY-SA 4.0 \
         (<https://github.com/UniversalDependencies/UD_English-EWT>); \
         quoted unmodified.*\n\n",
    );
    out.push_str(&format!(
        "Corpus: `{corpus_path}` — {} sentences, {} countable tokens \
         (punctuation/symbols excluded).\n\n",
        sentences.len(),
        total_tokens
    ));

    out.push_str("## Sentence buckets\n\n");
    out.push_str(&format!(
        "- **Perfect** (every token usable as-is): {} ({})\n\
         - **Close** (1–2 problem tokens): {} ({})\n\
         - **Far** (3+ problem tokens): {} ({})\n\n",
        perfect.len(),
        pct(perfect.len(), sentences.len()),
        close.len(),
        pct(close.len(), sentences.len()),
        far,
        pct(far, sentences.len()),
    ));

    out.push_str("## Token verdicts\n\n");
    out.push_str(&format!(
        "| verdict | tokens | share |\n|---|---|---|\n\
         | OK | {} | {} |\n\
         | REDIRECT (designed rejection with suggestion) | {} | {} |\n\
         | MISMATCH (enabled word, uncovered POS) | {} | {} |\n\
         | PROPN (out of scope) | {} | {} |\n\
         | OOV | {} | {} |\n\n",
        count(Verdict::Ok),
        pct(count(Verdict::Ok), total_tokens),
        count(Verdict::Redirect),
        pct(count(Verdict::Redirect), total_tokens),
        count(Verdict::Mismatch),
        pct(count(Verdict::Mismatch), total_tokens),
        count(Verdict::Propn),
        pct(count(Verdict::Propn), total_tokens),
        count(Verdict::Oov),
        pct(count(Verdict::Oov), total_tokens),
    ));

    out.push_str("## OOV by gold UPOS\n\n");
    let mut oov: Vec<(&String, &usize)> = oov_by_upos.iter().collect();
    oov.sort_by(|a, b| b.1.cmp(a.1));
    for (upos, n) in oov {
        out.push_str(&format!("- {upos}: {n}\n"));
    }
    out.push('\n');

    out.push_str("## Top missing lemmas (curation candidates — human decides)\n\n");
    let mut miss: Vec<(&(String, String), &usize)> = missing_lemmas.iter().collect();
    miss.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    let top: Vec<String> = miss
        .iter()
        .take(40)
        .map(|((lemma, upos), n)| format!("{lemma}/{upos} (×{n})"))
        .collect();
    out.push_str(&top.join(", "));
    out.push_str("\n\n");

    out.push_str("## Redirect hits (the rejection rules doing their job)\n\n");
    let mut hits: Vec<(&(String, String), &usize)> = redirect_hits.iter().collect();
    hits.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    if hits.is_empty() {
        out.push_str("None in this corpus.\n\n");
    } else {
        for ((lemma, upos), n) in hits.iter().take(15) {
            out.push_str(&format!("- {lemma} used as {upos} ×{n}\n"));
        }
        out.push('\n');
    }

    out.push_str("## Uncovered POS mismatches (holes in the redirect table)\n\n");
    let mut mm: Vec<(&(String, String), &usize)> = mismatches.iter().collect();
    mm.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    if mm.is_empty() {
        out.push_str("None. ✓\n\n");
    } else {
        for ((lemma, upos), n) in mm.iter().take(15) {
            out.push_str(&format!("- {lemma} used as {upos} ×{n}\n"));
        }
        out.push('\n');
    }

    out.push_str("## Samples for manual review\n\n### Perfect\n\n");
    for s in perfect.iter().take(10) {
        out.push_str(&format!("- {}\n", s.text));
    }
    out.push_str("\n### Close (with their problems)\n\n");
    for (s, problems) in close.iter().take(10) {
        out.push_str(&format!("- {} — *{}*\n", s.text, problems.join("; ")));
    }
    out.push('\n');
    out
}
