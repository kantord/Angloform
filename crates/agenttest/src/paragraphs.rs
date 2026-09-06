//! Paragraph repair: fix-and-compare in context (docs/ideas.md, "Paragraph
//! repair"). The unit of repair is the paragraph, because a sentence-level
//! fix can be right alone and wrong for the paragraph (given-before-new,
//! topic continuity).
//!
//! For every paragraph with a rejected sentence: the model gets the
//! paragraph, the per-sentence linter verdicts, the neighbouring paragraphs,
//! and the skill; it returns a rewritten paragraph plus a `drops:` line
//! (ADR 0012 contract). Every proposal is linted sentence by sentence and
//! measured (parse rate, topic continuity, surprisal cost); one YAML per
//! paragraph lands in tests/paragraph-cases/ with a human `verdict`, and
//! docs/paragraph-report.md shows original and proposals side by side.
//! Ranking is display order only — never a gate. Nothing edits the source.
//!
//! `--dry-run`: no API calls; measures the originals and writes the cases,
//! so the pipeline can be checked without spend.

use diagnose::{diagnose, Diagnosis};
use grammar::Lexicon;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const CASES_DIR: &str = "tests/paragraph-cases";
const ZIPF_PATH: &str = "data/freq/en_zipf.tsv";
const MAX_SURPRISAL: f64 = 9.0;

#[derive(Serialize, Deserialize, Default)]
pub struct ParaCase {
    source: String,
    index: usize,
    original: String,
    #[serde(default)]
    context_before: String,
    #[serde(default)]
    context_after: String,
    /// Metrics of the original paragraph.
    #[serde(default)]
    original_metrics: Metrics,
    /// Human judgment of `best`: ideal | needs-fix | unreviewed (auto-reset
    /// whenever `best` changes).
    #[serde(default = "unreviewed")]
    verdict: String,
    /// Free-text reviewer note (set with scripts/paragraph-review.py).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    note: String,
    /// The highest-ranked valid proposal (parse → continuity → cost).
    #[serde(default)]
    best: String,
    /// Every distinct proposal ever seen, failures included.
    #[serde(default)]
    proposals: Vec<Proposal>,
}

fn unreviewed() -> String {
    "unreviewed".to_string()
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Metrics {
    pub sentences: usize,
    pub parsed: usize,
    /// consecutive sentence pairs whose second subject appears in the first
    pub continuity_pairs: usize,
    pub continuity_ok: usize,
    /// Σ unigram surprisal (9 − zipf), as textcost
    pub cost: f64,
    pub words: usize,
    /// defined terms used (ADR 0027): Capitalized term surfaces in the text
    #[serde(default)]
    pub terms: usize,
    /// connectives and blocks used (and, or, but, so, because, if/then, than,
    /// Enumeration, Step Block): a proxy for natural, non-fragmented prose
    #[serde(default)]
    pub connectives: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Proposal {
    text: String,
    #[serde(default)]
    drops: String,
    valid: bool,
    metrics: Metrics,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnosis: Option<String>,
    count: usize,
    model: String,
}

/// Word → (tag, lemma) from lexicon.tsv, for the continuity metric.
struct Words {
    tags: BTreeMap<String, (String, String)>,
    zipf: BTreeMap<String, f64>,
    /// Capitalized surfaces of defined noun terms ("Anaphoric Pronouns")
    terms: Vec<String>,
}

impl Words {
    fn load() -> Words {
        let mut tags = BTreeMap::new();
        let mut terms = Vec::new();
        for line in std::fs::read_to_string(super::LEXICON_PATH).expect("lexicon").lines() {
            let f: Vec<&str> = line.split('\t').collect();
            if let [surface, "form", tag, lemma] = f[..] {
                tags.insert(surface.to_string(), (tag.to_string(), lemma.to_string()));
            }
            if let [_, "term", _, cap] = f[..] {
                terms.push(cap.to_string());
            }
        }
        // longest first, so "Anaphoric Pronouns" is counted once, not as "Pronouns"
        terms.sort_by_key(|t| std::cmp::Reverse(t.len()));
        terms.dedup();
        let mut zipf = BTreeMap::new();
        if let Ok(t) = std::fs::read_to_string(ZIPF_PATH) {
            for line in t.lines().filter(|l| !l.starts_with('#')) {
                if let Some((w, z)) = line.split_once('\t') {
                    if let Ok(z) = z.trim().parse() {
                        zipf.insert(w.to_string(), z);
                    }
                }
            }
        }
        Words { tags, zipf, terms }
    }

    /// (subject, all noun/name lemmas) of one sentence; subject = first noun
    /// or name. Names (quoted or capitalized mid-sentence) count as nouns.
    fn nouns(&self, sentence: &str) -> (Option<String>, BTreeSet<String>) {
        let mut set = BTreeSet::new();
        let mut subj = None;
        let mut i = 0;
        for tok in tokens(sentence) {
            let key = if tok.starts_with('"') {
                Some(tok.trim_matches('"').to_lowercase())
            } else if i > 0 && tok.chars().next().is_some_and(|c| c.is_uppercase()) {
                Some(tok.to_lowercase())
            } else {
                let w = tok.to_lowercase();
                self.tags.get(&w).filter(|(t, _)| t.starts_with("NOUN")).map(|(_, l)| l.clone())
            };
            if let Some(k) = key {
                if subj.is_none() {
                    subj = Some(k.clone());
                }
                set.insert(k);
            }
            i += 1;
        }
        (subj, set)
    }

    /// Every word of a sentence that is neither a lexicon form, a name
    /// (quoted / capitalized mid-sentence), a digit count, nor punctuation.
    fn unknown_words(&self, sentence: &str) -> Vec<String> {
        let mut out = Vec::new();
        for tok in tokens(sentence) {
            if tok.starts_with('"') || tok == "," {
                continue;
            }
            let digits = tok.trim_start_matches('~').chars().all(|c| c.is_ascii_digit());
            // capitalized anywhere: a name or a defined term (ADR 0018, 0027)
            let name = tok.chars().next().is_some_and(|c| c.is_uppercase());
            let w = tok.to_lowercase();
            if !digits && !name && !self.tags.contains_key(&w) && !out.contains(&w) {
                out.push(w);
            }
        }
        out
    }

    /// How many defined-term occurrences a text uses (case-sensitive).
    fn term_uses(&self, text: &str) -> usize {
        let mut rest = text.to_string();
        let mut n = 0;
        for t in &self.terms {
            let count = rest.matches(t.as_str()).count();
            if count > 0 {
                n += count;
                rest = rest.replace(t.as_str(), "§");
            }
        }
        n
    }

    fn cost(&self, text: &str) -> (usize, f64) {
        let toks: Vec<String> = text
            .split_whitespace()
            .map(|t| t.trim_matches(|c: char| c.is_ascii_punctuation() && c != '~').to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        let c = toks
            .iter()
            .map(|t| MAX_SURPRISAL - self.zipf.get(t).copied().unwrap_or(0.0))
            .sum();
        (toks.len(), c)
    }
}

/// Quoted spans stay one token; otherwise whitespace words.
fn tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, piece) in s.split('"').enumerate() {
        if i % 2 == 1 {
            out.push(format!("\"{}\"", piece.trim()));
        } else {
            out.extend(piece.split_whitespace().map(|w| {
                w.trim_matches(|c: char| c.is_ascii_punctuation() && c != '~' && c != '-').to_string()
            }));
        }
    }
    out.into_iter().filter(|t| !t.is_empty() && t != "\"\"").collect()
}

// ------------------------------------------------------------ extraction --

/// Prose paragraphs of a markdown file: headings, Date/Status lines and
/// tables skipped; each bullet is its own paragraph; backticks → quotes.
pub fn paragraphs(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for block in text.split("\n\n") {
        let block = block.trim();
        if block.is_empty()
            || block.starts_with('#')
            || block.starts_with("Date:")
            || block.starts_with("Status:")
            || block.starts_with('|')
            || block.starts_with("<!--")
        {
            continue;
        }
        // a Step Block (ADR 0034) is one paragraph and one unit
        let ls: Vec<&str> = block.lines().map(str::trim).collect();
        if grammar::is_step_block(block) {
            out.push(ls.join("\n"));
            continue;
        }
        // an Enumeration block (ADR 0028) is one unit, whether it is the whole
        // paragraph or ends it; the prose before it is its own paragraph
        let k = (0..ls.len()).find(|&i| ls[i].ends_with(':') && i + 1 < ls.len() && ls[i + 1..].iter().all(|l| l.starts_with("- ")));
        if let Some(k) = k {
            // the intro is the last sentence of the (possibly wrapped) text before the items
            let text = normalize(&ls[..=k].join(" "));
            let (prose, intro) = match text.rfind(". ") {
                Some(cut) => (text[..cut + 1].to_string(), text[cut + 2..].to_string()),
                None => (String::new(), text.clone()),
            };
            if !prose.trim().is_empty() {
                out.push(prose.trim().to_string());
            }
            let mut block = vec![intro];
            block.extend(ls[k + 1..].iter().map(|l| l.to_string()));
            out.push(block.join("\n"));
            continue;
        }
        // split bullets into their own paragraphs
        let mut cur = String::new();
        for line in block.lines() {
            let trimmed = line.trim_start();
            let is_bullet = trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed
                    .split_once(". ")
                    .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
            if is_bullet && !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            let content = if is_bullet {
                trimmed
                    .trim_start_matches(|c: char| c == '-' || c == '*' || c.is_ascii_digit() || c == '.')
                    .trim_start()
            } else {
                trimmed
            };
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(content);
        }
        if !cur.is_empty() {
            out.push(cur);
        }
    }
    // normalize prose only: an Enumeration block keeps its line breaks
    out.into_iter()
        .map(|p| if grammar::is_enumeration(&p) || grammar::is_step_block(&p) { p } else { normalize(&p) })
        .filter(|p| p.split_whitespace().count() >= 3)
        .collect()
}

fn normalize(p: &str) -> String {
    let mut s = String::new();
    let mut in_code = false;
    for c in p.chars() {
        match c {
            '`' => {
                in_code = !in_code;
                s.push('"');
            }
            '*' => {}
            _ => s.push(c),
        }
    }
    let _ = in_code;
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Sentence split with abbreviation protection (as scripts/extract-sentences.py).
pub fn sentences(p: &str) -> Vec<String> {
    if grammar::is_enumeration(p) || grammar::is_step_block(p) {
        return vec![p.to_string()];
    }
    let protected = p.replace("e.g.", "e~g~").replace("i.e.", "i~e~").replace("cf.", "cf~");
    let mut out = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = protected.chars().collect();
    let mut in_quote = false;
    for (i, &c) in chars.iter().enumerate() {
        cur.push(c);
        if c == '"' {
            in_quote = !in_quote;
        }
        if !in_quote && matches!(c, '.' | '!' | '?') && chars.get(i + 1).is_none_or(|n| *n == ' ') {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out.into_iter()
        .map(|s| {
            s.trim()
                .trim_end_matches(|c| matches!(c, '.' | '!' | '?'))
                .trim()
                .replace("e~g~", "e.g.")
                .replace("i~e~", "i.e.")
                .replace("cf~", "cf.")
        })
        .filter(|s| s.split_whitespace().count() >= 3)
        .collect()
}

// --------------------------------------------------------------- metrics --

fn measure(lexicon: &Lexicon, words: &Words, para: &str) -> (Metrics, Vec<(String, bool, String)>) {
    // units: Enumeration blocks stay whole; other lines split into sentences
    let mut sents = Vec::new();
    for u in grammar::units(para) {
        if grammar::is_enumeration(&u) || grammar::is_step_block(&u) {
            sents.push(u);
        } else {
            sents.extend(sentences(&u));
        }
    }
    let mut verdicts = Vec::new();
    let mut m = Metrics { sentences: sents.len(), ..Default::default() };
    let mut prev: Option<BTreeSet<String>> = None;
    for s in &sents {
        let d = diagnose(lexicon, s);
        let ok = matches!(d, Diagnosis::Clean(_));
        if ok {
            m.parsed += 1;
        }
        let mut detail = if ok { String::new() } else { super::diagnosis_text(&d) };
        if !ok {
            let unknown = words.unknown_words(s);
            if unknown.len() > 1 {
                detail.push_str(&format!(" [all unknown words: {}]", unknown.join(", ")));
            }
        }
        verdicts.push((s.clone(), ok, detail));
        let (subj, nouns) = words.nouns(s);
        if let (Some(p), Some(subj)) = (&prev, &subj) {
            m.continuity_pairs += 1;
            if p.contains(subj) {
                m.continuity_ok += 1;
            }
        }
        prev = Some(nouns);
    }
    let (w, c) = words.cost(para);
    m.words = w;
    m.cost = c;
    m.terms = words.term_uses(para);
    m.connectives = para
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| matches!(*w, "and" | "or" | "but" | "so" | "because" | "if" | "than"))
        .count()
        + para.matches("\n- ").count().min(1)
        + usize::from(grammar::is_step_block(para));
    (m, verdicts)
}

/// parse rate → fewer units (less fragmentation) → connectives and blocks
/// used → defined terms → cost. Topic continuity is reported, not ranked:
/// ranking on it rewarded the mechanical "the X … the X …" chains the
/// maintainer rejected. Display order only, never a gate.
fn rank_key(p: &Proposal) -> (bool, i64, i64, i64, i64, i64) {
    let m = &p.metrics;
    let parse = if m.sentences == 0 { 0 } else { (1000 * m.parsed / m.sentences) as i64 };
    (p.valid, parse, -(m.sentences as i64), m.connectives as i64, m.terms as i64, -(m.cost * 10.0) as i64)
}

// ------------------------------------------------------------------ run --

#[allow(clippy::too_many_arguments)]
pub fn run(
    api_key: &str,
    model: &str,
    temperature: f64,
    trials: usize,
    system_prompt: &str,
    lexicon: &Lexicon,
    file: &str,
    out_path: &str,
    dry_run: bool,
) {
    let text = std::fs::read_to_string(file).expect("markdown file");
    let paras = paragraphs(&text);
    let words = Words::load();
    std::fs::create_dir_all(CASES_DIR).expect("cases dir");
    let slug = std::path::Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("doc")
        .chars()
        .take(24)
        .collect::<String>();
    let system_prompt = format!(
        "{system_prompt}\n\nYou rewrite WHOLE PARAGRAPHS. To list things, use an Enumeration block: one \
         statement ending in a colon whose last noun phrase is plural, counted or \
         \"every <noun>\", then one line per item starting with \"- \", each item a \
         noun phrase (a quoted word, a Capitalized term, or \"the <noun>\"). Example:\n\
         The language allows 4 pronouns:\n- \"I\"\n- \"you\"\n- \"my\"\n- \"your\"\n Keep every claim of the \
         original (ADR 0012: never drop or change a proposition). Write natural \
         prose, not a chain of short sentences: keep roughly the sentence count of \
         the original, join related claims with the connectives the language has \
         (and, or, but; \", so\" / \", because\"; \"if …, then …\"; a comparative \
         with \"than\"), and use an Enumeration for a list. Do NOT start consecutive \
         sentences with the same subject when a connective can join them. \
         Reply with the rewritten paragraph (sentences separated by periods, \
         with normal sentence casing: a capital first letter, names and defined terms \
         Capitalized, everything else lowercase; a word mentioned AS A WORD is \
         quoted: the Linter bans \"it\"), then ONE final line `drops: <comma-\
         separated register/affect losses, or none>`. If a meaning-preserving \
         rewrite is impossible with the available words and structures, reply \
         exactly `GAP: <one-line reason>`. No other prose."
    );

    let concurrency: usize = std::env::var("ANGLOFORM_TEST_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16);
    let queue = std::sync::Mutex::new((0..paras.len()).collect::<std::collections::VecDeque<_>>());
    let results: std::sync::Mutex<Vec<Option<ParaCase>>> =
        std::sync::Mutex::new((0..paras.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..concurrency.max(1) {
            scope.spawn(|| loop {
                let i = queue.lock().unwrap().pop_front();
                let Some(i) = i else { break };
                let case = process(
                    &paras, i, file, &slug, api_key, model, temperature, trials, &system_prompt,
                    lexicon, &words, dry_run,
                );
                results.lock().unwrap()[i] = Some(case);
            });
        }
    });
    let cases: Vec<ParaCase> = results.into_inner().unwrap().into_iter().flatten().collect();
    write_report(&cases, file, out_path, dry_run);
    let need = cases.iter().filter(|c| c.original_metrics.parsed < c.original_metrics.sentences).count();
    let solved = cases.iter().filter(|c| !c.best.is_empty()).count();
    println!(
        "paragraphs: {} paragraphs, {need} with rejections, {solved} with a valid proposal → {out_path}",
        cases.len()
    );
}

#[allow(clippy::too_many_arguments)]
fn process(
    paras: &[String],
    i: usize,
    file: &str,
    slug: &str,
    api_key: &str,
    model: &str,
    temperature: f64,
    trials: usize,
    system_prompt: &str,
    lexicon: &Lexicon,
    words: &Words,
    dry_run: bool,
) -> ParaCase {
    let path = format!("{CASES_DIR}/{slug}-{:02}.yaml", i + 1);
    let mut case: ParaCase = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_yaml::from_str(&t).ok())
        .unwrap_or_default();
    if case.verdict.is_empty() {
        case.verdict = unreviewed();
    }
    case.source = file.to_string();
    case.index = i + 1;
    if !case.original.is_empty() && case.original != paras[i] {
        // the source paragraph changed: old proposals stay as replay data,
        // but the best and its verdict no longer apply
        case.best.clear();
        case.verdict = unreviewed();
        case.note = "source paragraph changed; stored proposals are replay data only".to_string();
    }
    case.original = paras[i].clone();
    case.context_before = i.checked_sub(1).map(|j| paras[j].clone()).unwrap_or_default();
    case.context_after = paras.get(i + 1).cloned().unwrap_or_default();
    let (om, verdicts) = measure(lexicon, words, &paras[i]);
    case.original_metrics = om.clone();
    // the language moves under stored proposals: re-validate every one
    // against the current lexicon before ranking
    for p in case.proposals.iter_mut() {
        if p.text.starts_with("GAP:") || p.text.starts_with("API error") {
            continue;
        }
        let (m, v) = measure(lexicon, words, &p.text);
        let bad: Vec<String> = v.iter().filter(|(_, ok, _)| !ok).map(|(s, _, d)| format!("\"{s}\": {d}")).collect();
        p.valid = bad.is_empty();
        p.metrics = m;
        p.diagnosis = (!bad.is_empty()).then(|| bad.join(" | "));
    }
    let clean = om.parsed == om.sentences;
    if clean || dry_run {
        let best = case
            .proposals
            .iter()
            .filter(|p| p.valid)
            .max_by_key(|p| rank_key(p))
            .map(|p| p.text.clone())
            .unwrap_or_default();
        if best != case.best {
            case.best = best;
            case.verdict = "unreviewed".to_string();
        }
        println!(
            "  {} {}: {}/{} parse, {} valid of {} stored",
            if clean { "✓" } else { "·" },
            i + 1,
            om.parsed,
            om.sentences,
            case.proposals.iter().filter(|p| p.valid).count(),
            case.proposals.len()
        );
        std::fs::write(&path, serde_yaml::to_string(&case).unwrap()).expect("write case");
        return case;
    }

    let flags: Vec<String> = verdicts
        .iter()
        .filter(|(_, ok, _)| !ok)
        .map(|(s, _, d)| format!("  - \"{s}\": {d}"))
        .collect();
    // the whole document is the context (ADRs are short); the target is marked
    let document: String = paras
        .iter()
        .enumerate()
        .map(|(j, p)| if j == i { format!(">>> {p} <<<") } else { p.clone() })
        .collect::<Vec<_>>()
        .join("\n\n");
    let user = format!(
        "Rewrite ONLY the paragraph marked >>> <<< in angloform. The rest of the document \
         is context: use it to understand what the paragraph means, do not rewrite it.\n\n\
         DOCUMENT:\n{document}\n\nPARAGRAPH TO REWRITE:\n  {}\n\nLinter rejections:\n{}\n",
        paras[i],
        flags.join("\n")
    );
    // each trial is an independent conversation with up to MAX_ROUNDS repair
    // rounds; every attempt (failures included) becomes a proposal
    let attempts: Vec<Vec<(String, String, bool, Metrics, Option<String>)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..trials)
            .map(|_| {
                scope.spawn(|| {
                    let mut messages = vec![
                        serde_json::json!({"role": "system", "content": system_prompt}),
                        serde_json::json!({"role": "user", "content": user.clone()}),
                    ];
                    let mut out = Vec::new();
                    for round in 0..super::MAX_ROUNDS {
                        let raw = match super::complete_with(api_key, model, temperature, &messages, 700) {
                            Ok(r) => r,
                            Err(e) => {
                                out.push((format!("API error: {e}"), String::new(), false, Metrics::default(), None));
                                break;
                            }
                        };
                        let (text, drops) = split_reply(&raw);
                        let (valid, metrics, diagnosis) = if text.starts_with("GAP:") {
                            (false, Metrics::default(), Some(text.clone()))
                        } else {
                            let (m, v) = measure(lexicon, words, &text);
                            let bad: Vec<String> = v
                                .iter()
                                .filter(|(_, ok, _)| !ok)
                                .map(|(s, _, d)| format!("\"{s}\": {d}"))
                                .collect();
                            (bad.is_empty(), m, (!bad.is_empty()).then(|| bad.join(" | ")))
                        };
                        let gap = text.starts_with("GAP:");
                        out.push((text.clone(), drops, valid, metrics, diagnosis.clone()));
                        if valid || gap || round + 1 == super::MAX_ROUNDS {
                            break;
                        }
                        messages.push(serde_json::json!({"role": "assistant", "content": raw}));
                        messages.push(serde_json::json!({"role": "user", "content": format!(
                            "Still rejected. Linter output per sentence:\n  {}\n\nReply with the whole \
                             corrected paragraph again, then the `drops:` line.",
                            diagnosis.unwrap_or_default().replace(" | ", "\n  ")
                        )}));
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    for (text, drops, valid, metrics, diagnosis) in attempts.into_iter().flatten() {
        if let Some(p) = case.proposals.iter_mut().find(|p| p.text == text) {
            p.count += 1;
        } else {
            case.proposals.push(Proposal { text, drops, valid, metrics, diagnosis, count: 1, model: model.to_string() });
        }
    }
    let best = case
        .proposals
        .iter()
        .filter(|p| p.valid)
        .max_by_key(|p| rank_key(p))
        .map(|p| p.text.clone())
        .unwrap_or_default();
    if best != case.best {
        case.best = best;
        case.verdict = "unreviewed".to_string();
    }
    println!(
        "  {} {}: {}/{} parse, {} proposal(s), {} valid",
        if case.best.is_empty() { "✗" } else { "→" },
        i + 1,
        om.parsed,
        om.sentences,
        case.proposals.len(),
        case.proposals.iter().filter(|p| p.valid).count()
    );
    std::fs::write(&path, serde_yaml::to_string(&case).unwrap()).expect("write case");
    case
}

/// Body and the trailing `drops:` line.
fn split_reply(raw: &str) -> (String, String) {
    let raw = raw.trim().trim_matches('`').trim();
    let mut body = Vec::new();
    let mut drops = String::new();
    for line in raw.lines() {
        let l = line.trim();
        if let Some(d) = l.strip_prefix("drops:") {
            drops = d.trim().to_string();
        } else if !l.is_empty() {
            body.push(l);
        }
    }
    // keep line breaks where they carry structure (Enumeration items)
    if body.iter().any(|l| l.starts_with("- ")) {
        // a block: never strip quotes — the last item may be a quoted word
        return (body.join("\n"), drops);
    }
    let text = body.join(" ");
    // a reply wrapped in one pair of quotes: unwrap it; otherwise keep quotes
    let unwrapped = text.strip_prefix('"').and_then(|t| t.strip_suffix('"')).filter(|t| !t.contains('"'));
    (unwrapped.unwrap_or(&text).trim().to_string(), drops)
}

fn fmt(m: &Metrics) -> String {
    let cont = if m.continuity_pairs == 0 {
        "n/a".to_string()
    } else {
        format!("{}/{}", m.continuity_ok, m.continuity_pairs)
    };
    format!("parse {}/{} · connectives {} · terms {} · continuity {cont} · cost {:.0} ({} words)", m.parsed, m.sentences, m.connectives, m.terms, m.cost, m.words)
}

fn write_report(cases: &[ParaCase], file: &str, out_path: &str, dry_run: bool) {
    let mut out = format!(
        "# Paragraph proposals: {file}\n\n*Generated by `agenttest paragraphs`{} — PROPOSALS \
         ONLY. Apply by hand after ADR 0012 meaning review; record the verdict in \
         tests/paragraph-cases/. Ranking (parse → continuity → cost) is display order, \
         never a gate.*\n\n",
        if dry_run { " (dry run: no API calls)" } else { "" }
    );
    for c in cases {
        let clean = c.original_metrics.parsed == c.original_metrics.sentences;
        out.push_str(&format!("## Paragraph {}{}\n\n", c.index, if clean { " ✓" } else { "" }));
        out.push_str(&format!("**Original** — {}\n\n> {}\n\n", fmt(&c.original_metrics), c.original));
        if clean {
            continue;
        }
        let mut ranked: Vec<&Proposal> = c.proposals.iter().collect();
        ranked.sort_by_key(|p| std::cmp::Reverse(rank_key(p)));
        if let Some(b) = ranked.iter().find(|p| p.text == c.best) {
            out.push_str(&format!(
                "**Best** — {}{}\n\n> {}\n\n",
                fmt(&b.metrics),
                if b.drops.is_empty() { String::new() } else { format!(" · drops: {}", b.drops) },
                b.text
            ));
        } else {
            out.push_str("*(no valid proposal yet)*\n\n");
        }
        out.push_str(&format!("verdict: `{}`{}\n\n", c.verdict, if c.note.is_empty() { String::new() } else { format!(" — {}", c.note) }));
        let others = ranked.iter().filter(|p| p.text != c.best).count();
        if others > 0 {
            out.push_str(&format!("<details><summary>{others} other proposal(s)</summary>\n\n"));
            for p in ranked.iter().filter(|p| p.text != c.best) {
                out.push_str(&format!(
                    "- ({}, ×{}) {}{}\n",
                    if p.valid { "valid" } else { "rejected" },
                    p.count,
                    p.text,
                    p.diagnosis.as_ref().map(|d| format!("\n  - {d}")).unwrap_or_default()
                ));
            }
            out.push_str("\n</details>\n\n");
        }
    }
    std::fs::write(out_path, &out).expect("write report");
}
