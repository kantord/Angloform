//! Tier-1 angloform grammar: LALRPOP over form-tags, lexicon-backed lexer,
//! and cognitive-load metrics derived from head-annotated parse trees.
//!
//! The LR(1) build is the ambiguity gate: a grammar conflict is a compile
//! error, so every accepted sentence has exactly one parse by construction.
//! Metrics follow ADR 0006: report peak open dependencies, dependency
//! length, embedding depth, branching direction, and fronted weight —
//! measurement only, no gating yet.

use lalrpop_util::lalrpop_mod;
use std::collections::BTreeMap;

lalrpop_mod!(pub angloform);

// ---------------------------------------------------------------- tokens --

#[derive(Clone, Debug)]
pub enum Tok {
    Det(String),
    DetSg(String),
    Adj(String),
    NounSg(String),
    NounPl(String),
    VtBase(String),
    Vt3(String),
    VtEd(String),
    VtIng(String),
    ViBase(String),
    Vi3(String),
    ViEd(String),
    ViIng(String),
    PrepN(String),
    PrepV(String),
    Pron1(String),
    Pron2(String),
    Poss(String),
    CopSg(String),
    CopPl(String),
    CopSgPast(String),
    CopPlPast(String),
    Cop1Sg(String),
    Conj(String),
    Neg(String),
    TempAdv(String),
    TimeAdv(String),
    Yet(String),
    Focus(String),
    Other(String),
    DoBase(String),
    Do3(String),
    DoPast(String),
    ModalMust(String),
    ModalCan(String),
    ModalCannot(String),
    If(String),
    Then(String),
    Every(String),
    No(String),
    Num(String),
    NumPl(String),
    Percent(String),
    Approx(String),
    So(String),
    Because(String),
    Namely(String),
    Which(String),
    Who(String),
    Ord(String),
    Than(String),
    More(String),
    Scale(String),
    AdjCmp(String),
    AdjLong(String),
    AdjSup(String),
    Most(String),
    NumVal(String),
    Be(String),
    BecomeSg(String),
    BecomePl(String),
    BecomePast(String),
    Some_(String),
    Name(String),
    Comma,
    Colon,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexError {
    pub word: String,
    pub position: usize,
    /// From the lexicon's reject table, when the unknown word is a known
    /// rejected use: "use \"X\" instead".
    pub suggestion: Option<String>,
    /// The word is deliberately banned (ban table), not merely unknown.
    pub banned: bool,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.banned {
            write!(f, "\"{}\" is banned in angloform", self.word)?;
        } else {
            write!(f, "\"{}\" is not an angloform word", self.word)?;
        }
        if let Some(s) = &self.suggestion {
            write!(f, " — {s}")?;
        }
        Ok(())
    }
}

// --------------------------------------------------------------- lexicon --

/// A rejected-sense redirect: either a suggested replacement word, or
/// free-text advice for a collision with no natural word substitute (ADR
/// 0060 — replaces the old silent `waive`; every excluded sense explains
/// itself to the writer who hits it).
#[derive(Clone, Debug)]
pub enum Redirect {
    Word(String),
    Advice(String),
}

impl Redirect {
    /// Render as the message fragment shown after "is a {pos} in angloform".
    pub fn render(&self, pos: &str) -> String {
        match self {
            Redirect::Word(w) => format!("as a {pos} use \"{w}\""),
            Redirect::Advice(a) => a.clone(),
        }
    }
}

pub struct Lexicon {
    forms: BTreeMap<String, String>,
    lemmas: BTreeMap<String, String>,
    /// lowercase spelling of a defined term → its Capitalized form (ADR 0027)
    terms: BTreeMap<String, String>,
    /// defined proper names (domain model NAME entries): may start a sentence
    names: std::collections::BTreeSet<String>,
    /// adjective lemma → its inflected comparative surface (ADR 0030)
    comparatives: BTreeMap<String, String>,
    /// adjective lemma → its inflected superlative surface (ADR 0056)
    superlatives: BTreeMap<String, String>,
    rejects: BTreeMap<String, Vec<(String, Redirect)>>,
    bans: BTreeMap<String, String>,
    /// Capitalized term → its Capitalized parent category (ADR 0027's
    /// `member_of`, ADR 0049): the domain model's own is-a graph, made
    /// available at runtime.
    member_of: BTreeMap<String, String>,
}

impl Lexicon {
    /// Parse the TSV contents of a lexicon file (used by `load`, and by the
    /// wasm build, which embeds the file with `include_str!`).
    pub fn from_tsv(text: &str) -> Result<Lexicon, String> {
        let mut forms = BTreeMap::new();
        let mut lemmas = BTreeMap::new();
        let mut rejects: BTreeMap<String, Vec<(String, Redirect)>> = BTreeMap::new();
        let mut bans: BTreeMap<String, String> = BTreeMap::new();
        let mut terms: BTreeMap<String, String> = BTreeMap::new();
        let mut names = std::collections::BTreeSet::new();
        let mut comparatives: BTreeMap<String, String> = BTreeMap::new();
        let mut superlatives: BTreeMap<String, String> = BTreeMap::new();
        let mut member_of: BTreeMap<String, String> = BTreeMap::new();
        for line in text.lines().filter(|l| !l.starts_with('#')) {
            let f: Vec<&str> = line.split('\t').collect();
            let [surface, kind, tag, value] = f[..] else { continue };
            match kind {
                "form" => {
                    forms.insert(surface.to_string(), tag.to_string());
                    lemmas.insert(surface.to_string(), value.to_string());
                    if tag == "ADJ_CMP" {
                        comparatives.insert(value.to_string(), surface.to_string());
                    }
                    if tag == "ADJ_SUP" {
                        superlatives.insert(value.to_string(), surface.to_string());
                    }
                }
                "reject" => rejects
                    .entry(surface.to_string())
                    .or_default()
                    .push((tag.to_string(), Redirect::Word(value.to_string()))),
                "reject_advice" => rejects
                    .entry(surface.to_string())
                    .or_default()
                    .push((tag.to_string(), Redirect::Advice(value.to_string()))),
                "ban" => {
                    bans.insert(surface.to_string(), value.to_string());
                }
                "term" => {
                    terms.insert(surface.to_string(), value.to_string());
                }
                "name" => {
                    names.insert(surface.to_string());
                }
                "member_of" => {
                    member_of.insert(surface.to_string(), value.to_string());
                }
                _ => {}
            }
        }
        Ok(Lexicon { forms, lemmas, terms, names, comparatives, superlatives, rejects, bans, member_of })
    }

    pub fn load(path: &str) -> Result<Lexicon, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        Self::from_tsv(&text)
    }

    /// The form-tag of an enabled surface form, if any.
    pub fn tag_of(&self, word: &str) -> Option<&str> {
        self.forms.get(word).map(String::as_str)
    }

    /// The Capitalized form of a defined term written in lowercase
    /// ("reference ambiguity" → "Reference Ambiguity"), ADR 0027.
    pub fn term(&self, lowercase: &str) -> Option<&str> {
        self.terms.get(lowercase).map(String::as_str)
    }

    /// The inflected comparative of an adjective, if it has one (ADR 0030).
    pub fn comparative(&self, adj: &str) -> Option<&str> {
        self.comparatives.get(adj).map(String::as_str)
    }

    /// The inflected superlative of an adjective, if it has one (ADR 0056).
    pub fn superlative(&self, adj: &str) -> Option<&str> {
        self.superlatives.get(adj).map(String::as_str)
    }

    /// The Capitalized parent category of a Capitalized domain term, if
    /// the domain model declares one (ADR 0027's `member_of`, ADR 0049).
    pub fn member_of(&self, capitalized_term: &str) -> Option<&str> {
        self.member_of.get(capitalized_term).map(String::as_str)
    }

    /// The lemma of an enabled surface form.
    pub fn lemma_of(&self, word: &str) -> Option<&str> {
        self.lemmas.get(word).map(String::as_str)
    }

    /// The redirect suggestion for a rejected (POS) use of a word, looked up
    /// by its lemma (the reject table is keyed by lemma).
    pub fn redirect(&self, word: &str, pos: &str) -> Option<&Redirect> {
        let key = self.lemmas.get(word).map(String::as_str).unwrap_or(word);
        self.rejects
            .get(key)
            .and_then(|rs| rs.iter().find(|(p, _)| p == pos).map(|(_, r)| r))
    }

    /// Tokenize a sentence. Commas become COMMA tokens; a trailing period is
    /// dropped; a double-quoted span is a NAME (verbatim identifier);
    /// capitalized words are NAMEs unless they fold to a lexicon word at
    /// sentence start (ADR 0018); anything else not in the lexicon is a
    /// LexError with a redirect suggestion when the reject table has one.
    pub fn tokenize(&self, sentence: &str) -> Result<Vec<(usize, Tok)>, LexError> {
        let mut out: Vec<(usize, Tok)> = Vec::new();
        // quoted spans become single NAME tokens; even-indexed pieces are
        // ordinary text
        let pieces: Vec<&str> = sentence.split('"').collect();
        // fail loud on an unterminated quote (ADR 0018): never silently
        // quote to the end of the line
        if pieces.len() % 2 == 0 {
            return Err(LexError {
                word: "\"".to_string(),
                position: 0,
                suggestion: Some("a quote is not closed — quotes come in pairs".to_string()),
                banned: false,
            });
        }
        for (pi, piece) in pieces.iter().enumerate() {
            if pi % 2 == 1 {
                let pos = out.len();
                out.push((pos, Tok::Name(piece.trim().to_string())));
                continue;
            }
            self.tokenize_plain(piece, &mut out)?;
        }
        // merge consecutive capitalized NAMEs ("Visual Studio Code")
        let mut merged: Vec<(usize, Tok)> = Vec::new();
        for (_, t) in out {
            if let (Some((_, Tok::Name(prev))), Tok::Name(w)) = (merged.last_mut(), &t) {
                prev.push(' ');
                prev.push_str(w);
                continue;
            }
            let pos = merged.len();
            merged.push((pos, t));
        }
        Ok(merged.into_iter().enumerate().map(|(i, (_, t))| (i, t)).collect())
    }

    fn tokenize_plain(
        &self,
        piece: &str,
        out: &mut Vec<(usize, Tok)>,
    ) -> Result<(), LexError> {
        let raws: Vec<&str> = piece.split_whitespace().collect();
        let mut i = 0;
        while i < raws.len() {
            // ADR 0027: a multi-word defined term ("Anaphoric Pronouns") is the
            // longest run of capitalized words whose joined form is a lexicon
            // surface; punctuation may only trail the last word
            if raws[i].chars().next().is_some_and(|c| c.is_uppercase()) {
                let mut matched = None;
                for k in (2..=4).rev() {
                    if i + k > raws.len() {
                        continue;
                    }
                    let inner_clean = raws[i..i + k - 1].iter().all(|w| clean(w) == (*w, false));
                    let (last_raw, colon) = match raws[i + k - 1].strip_suffix(':') {
                        Some(w) => (w, true),
                        None => (raws[i + k - 1], false),
                    };
                    let (last, comma) = clean(last_raw);
                    let joined = format!("{} {last}", raws[i..i + k - 1].join(" "));
                    if inner_clean && self.forms.contains_key(&joined) {
                        matched = Some((k, joined, comma, colon));
                        break;
                    }
                }
                if let Some((k, joined, comma, colon)) = matched {
                    let pos = out.len();
                    let tag = &self.forms[&joined];
                    let tok = tag_to_tok(tag, &joined).expect("term tag");
                    out.push((pos, tok));
                    if comma {
                        let pos = out.len();
                        out.push((pos, Tok::Comma));
                    }
                    if colon {
                        let pos = out.len();
                        out.push((pos, Tok::Colon));
                    }
                    i += k;
                    continue;
                }
            }
            let (word, had_colon) = match raws[i].strip_suffix(':') {
                Some(w) => (w, true),
                None => (raws[i], false),
            };
            let (word, had_comma) = clean(word);
            i += 1;
            if !word.is_empty() {
                self.tokenize_word(word, out)?;
            }
            if had_comma {
                let pos = out.len();
                out.push((pos, Tok::Comma));
            }
            if had_colon {
                let pos = out.len();
                out.push((pos, Tok::Colon));
            }
        }
        Ok(())
    }

    fn tokenize_word(&self, word: &str, out: &mut Vec<(usize, Tok)>) -> Result<(), LexError> {
        let pos = out.len();
        // ADR 0022: digits are a lexer class, not lexicon words;
        // ADR 0025: a "~" prefix is the symbol form of "about"
        let (approx, word) = match word.strip_prefix('~') {
            Some(rest) if !rest.is_empty() => (true, rest),
            _ => (false, word),
        };
        if approx {
            out.push((pos, Tok::Approx("~".to_string())));
        }
        let pos = out.len();
        if let Some(numeral) = number_token(word) {
            let tok = numeral.map_err(|suggestion| LexError {
                word: word.to_string(),
                position: pos,
                suggestion: Some(suggestion),
                banned: false,
            })?;
            out.push((pos, tok));
            return Ok(());
        }
        let capitalized = word.chars().next().is_some_and(|c| c.is_uppercase());
        // ADR 0018, fail-loud: unquoted NAME only when capitalized,
        // mid-sentence, and not a lexicon word in lowercase. A
        // sentence-initial capital folds to the lexicon or errors —
        // never silently becomes a name (typos must stay loud).
        let folded;
        let word = if !self.forms.contains_key(word) {
            folded = word.to_lowercase();
            let known_lower = self.forms.contains_key(&folded);
            if known_lower && (pos == 0 || word == "I") {
                folded.as_str()
            } else if capitalized && known_lower {
                return Err(LexError {
                    word: word.to_string(),
                    position: pos,
                    suggestion: Some(format!(
                        "angloform words are lowercase (\"{folded}\"); a name \
                         that equals a word needs quotes"
                    )),
                    banned: false,
                });
            } else if capitalized && (pos > 0 || self.names.contains(word)) {
                out.push((pos, Tok::Name(word.to_string())));
                return Ok(());
            } else if capitalized {
                return Err(LexError {
                    word: word.to_string(),
                    position: pos,
                    suggestion: Some(
                        "a name cannot start a sentence — introduce it \
                         (\"the tool Lexgen …\") or quote it; or if this \
                         is a command, use an angloform verb in lowercase \
                         (\"delete the file\")"
                            .to_string(),
                    ),
                    banned: false,
                });
            } else {
                word
            }
        } else {
            word
        };
        // ADR 0027: a defined term written in lowercase
        if let Some(cap) = self.terms.get(word) {
            if !self.forms.contains_key(word) {
                return Err(LexError {
                    word: word.to_string(),
                    position: pos,
                    suggestion: Some(format!(
                        "\"{cap}\" is a defined term — write it capitalized (see CONTEXT.md)"
                    )),
                    banned: false,
                });
            }
        }
        if let Some(advice) = self.bans.get(word) {
            return Err(LexError {
                word: word.to_string(),
                position: pos,
                suggestion: Some(advice.clone()),
                banned: true,
            });
        }
        let tag = self.forms.get(word).ok_or_else(|| LexError {
            word: word.to_string(),
            position: pos,
            suggestion: self.rejects.get(word).map(|rs| {
                rs.iter()
                    .map(|(pos, r)| r.render(pos))
                    .collect::<Vec<_>>()
                    .join("; ")
            }),
            banned: false,
        })?;
        let tok = tag_to_tok(tag, word).ok_or_else(|| LexError {
            word: word.to_string(),
            position: pos,
            suggestion: Some(format!("its category {tag} is not yet usable in any sentence structure")),
            banned: false,
        })?;
        out.push((pos, tok));
        Ok(())
    }
}

/// Strip a trailing period and comma: ("word,", true) for "word,." / "word,".
fn clean(raw: &str) -> (&str, bool) {
    let w = raw.trim_end_matches('.');
    match w.strip_suffix(',') {
        Some(s) => (s, true),
        None => (w, false),
    }
}

/// Lines of a text grouped into units: an Enumeration (a line ending in a
/// colon plus the "- item" lines after it) is one unit; every other
/// non-empty, non-comment line is one unit (ADR 0028).
pub fn units(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        if l.is_empty() || l.starts_with('#') {
            i += 1;
            continue;
        }
        // a Step Block (ADR 0034): consecutive header/step lines are one unit
        if is_step_line(l) {
            let mut block = Vec::new();
            let mut j = i;
            while j < lines.len() && is_step_line(lines[j]) {
                block.push(lines[j].to_string());
                j += 1;
            }
            out.push(block.join("\n"));
            i = j;
            continue;
        }
        if l.ends_with(':') {
            // the intro is the LAST sentence on the line; prose before it is
            // its own unit (a reply may put both on one line)
            let (prose, intro) = match l.rfind(". ") {
                Some(cut) => (&l[..cut + 1], l[cut + 2..].trim()),
                None => ("", l),
            };
            if !prose.trim().is_empty() {
                out.push(prose.trim().to_string());
            }
            let mut block = vec![intro.to_string()];
            let mut j = i + 1;
            while j < lines.len() && lines[j].starts_with("- ") {
                block.push(lines[j].to_string());
                j += 1;
            }
            out.push(block.join("\n"));
            i = j;
        } else {
            out.push(l.to_string());
            i += 1;
        }
    }
    out
}

/// Gherkin keywords that open a step line (ADR 0034).
pub const STEP_KEYWORDS: [&str; 4] = ["Given ", "When ", "Then ", "And "];
pub const HEADER_KEYWORDS: [&str; 2] = ["Feature:", "Scenario:"];

fn is_step_line(l: &str) -> bool {
    STEP_KEYWORDS.iter().any(|k| l.starts_with(k)) || HEADER_KEYWORDS.iter().any(|k| l.starts_with(k))
}

/// True when a text has the Step Block shape: every non-empty line is a
/// Gherkin header or step, and at least one step is present.
pub fn is_step_block(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    // opens with a header, "Given" or "When": a lone "Then …" line is prose
    // with a misplaced "then", not a block
    let opens = lines.first().is_some_and(|l| {
        HEADER_KEYWORDS.iter().any(|k| l.starts_with(k)) || l.starts_with("Given ") || l.starts_with("When ")
    });
    opens && lines.iter().all(|l| is_step_line(l))
}

/// True when a text has the Enumeration block shape.
pub fn is_enumeration(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    lines.len() > 1 && lines[0].ends_with(':') && lines[1..].iter().all(|l| l.starts_with("- "))
}

/// True when a text has the Mapping block shape (ADR 0051): an intro
/// ending in ":", then 2-column markdown table rows (`scripts/mdblocks.py`
/// has already dropped the header row and the `|---|---|` divider).
pub fn is_mapping(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    lines.len() > 1
        && lines[0].ends_with(':')
        && lines[1..]
            .iter()
            .all(|l| l.starts_with('|') && l.ends_with('|') && l.matches('|').count() == 3)
}

/// Digit strings (ADR 0022): `2` and up are NUM_PL; `0` and `1` are
/// redirected (*no* / *one*), leading zeros are rejected. Non-digit words
/// return None.
fn number_token(word: &str) -> Option<Result<Tok, String>> {
    // ADR 0029: digits + st/nd/rd/th is an ordinal; 1st–3rd are word-form
    for suf in ["st", "nd", "rd", "th"] {
        if let Some(n) = word.strip_suffix(suf) {
            if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) {
                return Some(match n {
                    "1" => Err("write \"first\" — the first three ordinals are words (ADR 0029)".to_string()),
                    "2" => Err("write \"second\" — the first three ordinals are words (ADR 0029)".to_string()),
                    "3" => Err("write \"third\" — the first three ordinals are words (ADR 0029)".to_string()),
                    _ => Ok(Tok::Ord(word.to_string())),
                });
            }
        }
    }
    // ADR 0029: a decimal (digits . digits) is a quantity written in digits
    if let Some((a, b)) = word.split_once('.') {
        if !a.is_empty() && !b.is_empty() && a.chars().all(|c| c.is_ascii_digit()) && b.chars().all(|c| c.is_ascii_digit()) {
            return Some(if a.starts_with('0') && a != "0" {
                Err("a number does not start with 0 (ADR 0022)".to_string())
            } else {
                Ok(Tok::NumPl(word.to_string()))
            });
        }
    }
    if word.is_empty() || !word.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(match word {
        // ADR 0058: "0" is a measurement value (a NUM_VAL, not a NUM_PL
        // count) — the Grammar confines it to the value position, so it
        // still cannot reach a count position ("0 files"); the Linter
        // gives the same "no <noun>" redirect there (ADR 0022) as before,
        // now as a named finding instead of a lex-level ban.
        "0" => Ok(Tok::NumVal(word.to_string())),
        "1" => Err("write \"one\" — exactly one stays a word (ADR 0016)".to_string()),
        _ if word.starts_with('0') => {
            Err("a number does not start with 0 (ADR 0022)".to_string())
        }
        _ => Ok(Tok::NumPl(word.to_string())),
    })
}

fn tag_to_tok(tag: &str, word: &str) -> Option<Tok> {
    let w = word.to_string();
    Some(match tag {
        "DET" => Tok::Det(w),
        "DET_SG" => Tok::DetSg(w),
        "ADJ" => Tok::Adj(w),
        "NOUN_SG" => Tok::NounSg(w),
        "NOUN_PL" => Tok::NounPl(w),
        "VERB_TRANS_BASE" => Tok::VtBase(w),
        "VERB_TRANS_3SG" => Tok::Vt3(w),
        "VERB_TRANS_ED" => Tok::VtEd(w),
        "VERB_TRANS_ING" => Tok::VtIng(w),
        "VERB_INTRANS_BASE" => Tok::ViBase(w),
        "VERB_INTRANS_3SG" => Tok::Vi3(w),
        "VERB_INTRANS_ED" => Tok::ViEd(w),
        "VERB_INTRANS_ING" => Tok::ViIng(w),
        "PREP_N" => Tok::PrepN(w),
        "PREP_V" => Tok::PrepV(w),
        "PRON_1SG" => Tok::Pron1(w),
        "PRON_2" => Tok::Pron2(w),
        "POSS_1SG" | "POSS_2" => Tok::Poss(w),
        "COPULA_SG" => Tok::CopSg(w),
        "COPULA_PL" => Tok::CopPl(w),
        "COPULA_SG_PAST" => Tok::CopSgPast(w),
        "COPULA_PL_PAST" => Tok::CopPlPast(w),
        "COPULA_1SG" => Tok::Cop1Sg(w),
        "CONJ" => Tok::Conj(w),
        "NEG" => Tok::Neg(w),
        "TEMP_ADV" => Tok::TempAdv(w),
        "TIME_ADV" => Tok::TimeAdv(w),
        "YET" => Tok::Yet(w),
        "FOCUS" => Tok::Focus(w),
        "OTHER" => Tok::Other(w),
        "NEG_AUX_BASE" => Tok::DoBase(w),
        "NEG_AUX_3SG" => Tok::Do3(w),
        "NEG_AUX_PAST" => Tok::DoPast(w),
        "MODAL_MUST" => Tok::ModalMust(w),
        "MODAL_CAN" => Tok::ModalCan(w),
        "MODAL_CAN_NEG" => Tok::ModalCannot(w),
        "SCONJ_COND" => Tok::If(w),
        "THEN" => Tok::Then(w),
        "QUANT_UNIV" => Tok::Every(w),
        "QUANT_NEG" => Tok::No(w),
        "NUM_SG" => Tok::Num(w),
        "QUANT_EXIST" => Tok::Some_(w),
        "PERCENT" => Tok::Percent(w),
        "APPROX" => Tok::Approx(w),
        "RESULT" => Tok::So(w),
        "REASON" => Tok::Because(w),
        "NAMELY" => Tok::Namely(w),
        "WHICH" => Tok::Which(w),
        "WHO" => Tok::Who(w),
        "ORD" => Tok::Ord(w),
        "THAN" => Tok::Than(w),
        "MORE" => Tok::More(w),
        "SCALE" => Tok::Scale(w),
        "ADJ_CMP" => Tok::AdjCmp(w),
        "ADJ_LONG" => Tok::AdjLong(w),
        "ADJ_SUP" => Tok::AdjSup(w),
        "MOST" => Tok::Most(w),
        "BE" => Tok::Be(w),
        "BECOME_SG" => Tok::BecomeSg(w),
        "BECOME_PL" => Tok::BecomePl(w),
        "BECOME_PAST" => Tok::BecomePast(w),
        // NAME is produced directly by the tokenizer, never from the lexicon
        _ => return None,
    })
}

// ------------------------------------------------------------------ tree --

#[derive(Clone, Debug)]
pub enum Tree {
    Leaf { pos: usize, word: String },
    Node { label: &'static str, head: usize, children: Vec<Tree> },
}

pub fn leaf(pos: usize, word: String) -> Tree {
    Tree::Leaf { pos, word }
}

/// Build a node from fixed slots, dropping absent optionals. `head_slot`
/// indexes the *slot* list; it is re-based onto the surviving children.
pub fn nd(label: &'static str, head_slot: usize, slots: Vec<Option<Tree>>) -> Tree {
    let head = slots[..head_slot].iter().filter(|s| s.is_some()).count();
    let children: Vec<Tree> = slots.into_iter().flatten().collect();
    assert!(head < children.len(), "head slot must be present");
    Tree::Node { label, head, children }
}

impl Tree {
    pub fn head_word_pos(&self) -> usize {
        match self {
            Tree::Leaf { pos, .. } => *pos,
            Tree::Node { head, children, .. } => children[*head].head_word_pos(),
        }
    }

    pub fn render(&self, out: &mut String, indent: usize) {
        match self {
            Tree::Leaf { word, .. } => {
                out.push_str(&format!("{}{}\n", "  ".repeat(indent), word))
            }
            Tree::Node { label, head, children, .. } => {
                out.push_str(&format!("{}({label} h={head}\n", "  ".repeat(indent)));
                for c in children {
                    c.render(out, indent + 1);
                }
                out.push_str(&format!("{})\n", "  ".repeat(indent)));
            }
        }
    }
}

// --------------------------------------------------------------- metrics --

/// Cognitive-load metrics per ADR 0006, derived from the head-annotated tree.
#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    /// Peak number of dependency arcs spanning any inter-word gap
    /// (comfortable ceiling ≈ 4 per the research findings).
    pub peak_open_deps: usize,
    /// Longest dependency in words.
    pub max_dep_len: usize,
    /// Deepest nesting of clause-level nodes.
    pub embedding_depth: usize,
    /// Share of dependencies whose dependent follows its head
    /// (head-initial = right-branching).
    pub right_branching: f64,
    /// Tokens before the sentence's main head word (fronted material).
    pub fronted: usize,
}

pub fn metrics(tree: &Tree) -> Metrics {
    let mut arcs: Vec<(usize, usize)> = Vec::new(); // (dependent, head)
    collect_arcs(tree, &mut arcs);
    let max_pos = arcs
        .iter()
        .flat_map(|&(a, b)| [a, b])
        .max()
        .unwrap_or(0);
    let mut peak = 0usize;
    for gap in 0..max_pos {
        // arcs spanning the gap between word `gap` and `gap+1`
        let open = arcs
            .iter()
            .filter(|&&(a, b)| a.min(b) <= gap && a.max(b) > gap)
            .count();
        peak = peak.max(open);
    }
    let max_dep_len = arcs.iter().map(|&(a, b)| a.abs_diff(b)).max().unwrap_or(0);
    let right = arcs.iter().filter(|&&(dep, head)| dep > head).count();
    Metrics {
        peak_open_deps: peak,
        max_dep_len,
        embedding_depth: clause_depth(tree),
        right_branching: if arcs.is_empty() {
            1.0
        } else {
            right as f64 / arcs.len() as f64
        },
        fronted: tree.head_word_pos(),
    }
}

fn collect_arcs(tree: &Tree, arcs: &mut Vec<(usize, usize)>) {
    if let Tree::Node { head, children, .. } = tree {
        let h = children[*head].head_word_pos();
        for (i, c) in children.iter().enumerate() {
            if i != *head {
                arcs.push((c.head_word_pos(), h));
            }
            collect_arcs(c, arcs);
        }
    }
}

fn clause_depth(tree: &Tree) -> usize {
    match tree {
        Tree::Leaf { .. } => 0,
        Tree::Node { label, children, .. } => {
            let inner = children.iter().map(clause_depth).max().unwrap_or(0);
            let is_clause = matches!(*label, "S" | "Clause" | "Cond" | "Causal" | "Prohib" | "Imp" | "Steps");
            inner + usize::from(is_clause)
        }
    }
}

// ------------------------------------------------------------------ case --

/// Grammatical case, assigned positionally (like English): the grammar
/// already uses a distinct nonterminal per role (subject, VP object,
/// PP object, of-PP possessor, copula complement), so every argument's
/// role is fully determined by *where* it sits in the tree — this is a
/// pure derived view, not new grammar. Keyed by the argument's head word
/// position (`head_word_pos`), one entry per argument NP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    /// The subject of a clause.
    Nominative,
    /// The direct object of a verb phrase.
    Accusative,
    /// The object of an of-PP (possessor/whole, ADR 0011).
    Genitive,
    /// The object of any other PP (a PREP_N or PREP_V attachment).
    Oblique,
    /// A copula's complement — not a case-bearing argument in English,
    /// but a distinct role worth keeping separate from Nominative/Accusative.
    Complement,
}

const NP_LABELS: [&str; 9] = ["NP", "NPAppos", "NPGen", "NPPct", "Cmp", "ComplPP", "NPOnly", "NPOther", "Value"];

/// Tag `t` with `case` if it stands directly as an argument (an NP-family
/// node, or a bare name leaf) — everything else (a PP, an of-PP) is not
/// itself case-bearing; its own case comes from recursing into it.
fn tag_arg(t: &Tree, case: Case, out: &mut Vec<(usize, Case)>) {
    match t {
        Tree::Leaf { pos, .. } => out.push((*pos, case)),
        Tree::Node { label, .. } if NP_LABELS.contains(label) => {
            out.push((t.head_word_pos(), case));
        }
        _ => {}
    }
}

/// Every argument's grammatical case, derived from the parse tree alone.
pub fn cases(tree: &Tree) -> Vec<(usize, Case)> {
    let mut out = Vec::new();
    walk_cases(tree, &mut out);
    out.sort_by_key(|&(pos, _)| pos);
    out
}

fn walk_cases(t: &Tree, out: &mut Vec<(usize, Case)>) {
    let Tree::Node { label, children, .. } = t else { return };
    match *label {
        "S" | "Clause" => {
            if let Some(subj) = children.first() {
                tag_arg(subj, Case::Nominative, out);
                walk_cases(subj, out);
            }
            for c in &children[1..] {
                walk_cases(c, out);
            }
        }
        "VP" => {
            for c in children.iter().skip(1) {
                tag_arg(c, Case::Accusative, out);
                walk_cases(c, out);
            }
        }
        "PP" => {
            if let Some(np) = children.get(1) {
                tag_arg(np, Case::Oblique, out);
                walk_cases(np, out);
            }
        }
        "OfPP" => {
            if let Some(np) = children.get(1) {
                tag_arg(np, Case::Genitive, out);
                walk_cases(np, out);
            }
        }
        "CopPred" => {
            if let Some(cm) = children.last() {
                tag_arg(cm, Case::Complement, out);
                walk_cases(cm, out);
            }
            for c in &children[..children.len().saturating_sub(1)] {
                walk_cases(c, out);
            }
        }
        _ => {
            for c in children {
                walk_cases(c, out);
            }
        }
    }
}

// ----------------------------------------------------------------- parse --

pub type ParseError =
    lalrpop_util::ParseError<usize, Tok, LexError>;

/// Parse one unit: a sentence, an Enumeration block (ADR 0028), a Step
/// Block (ADR 0034), or a Mapping block (ADR 0051).
pub fn parse_text(lexicon: &Lexicon, text: &str) -> Result<Tree, String> {
    if is_mapping(text) {
        parse_mapping(lexicon, text)
    } else if is_enumeration(text) {
        parse_enumeration(lexicon, text)
    } else if is_step_block(text) {
        parse_step_block(lexicon, text)
    } else {
        parse(lexicon, text)
    }
}

/// Feature:/Scenario: headers carry an angloform sentence or a quoted Name;
/// Given/When/Then/And lines carry one clause each (ADR 0034).
fn parse_step_block(lexicon: &Lexicon, text: &str) -> Result<Tree, String> {
    let mut children = Vec::new();
    let mut seen_step = false;
    for (k, raw) in text.lines().map(str::trim).filter(|l| !l.is_empty()).enumerate() {
        if let Some(kw) = HEADER_KEYWORDS.iter().find(|kw| raw.starts_with(*kw)) {
            let title = raw[kw.len()..].trim();
            if title.is_empty() {
                return Err(format!("line {}: \"{kw}\" needs a title (ADR 0034)", k + 1));
            }
            let tree = parse(lexicon, title).or_else(|_| {
                let toks = lexicon.tokenize(title).map_err(|e| e.to_string())?;
                let iter = toks.into_iter().map(|(i, t)| Ok::<(usize, Tok, usize), LexError>((i, t, i + 1)));
                angloform::ItemParser::new().parse(iter).map_err(|_| String::new())
            }).map_err(|e| format!("line {}: the title \"{title}\" is not an angloform sentence or a quoted Name{}", k + 1, if e.is_empty() { String::new() } else { format!(" — {e}") }))?;
            children.push(tree);
            continue;
        }
        let kw = STEP_KEYWORDS.iter().find(|kw| raw.starts_with(*kw)).expect("step line");
        if kw.trim() == "And" && !seen_step {
            return Err(format!("line {}: \"And\" continues a step — no step comes before it (ADR 0034)", k + 1));
        }
        seen_step = true;
        let clause = raw[kw.len()..].trim().trim_end_matches('.');
        let toks = lexicon.tokenize(clause).map_err(|e| format!("line {}: {e}", k + 1))?;
        let iter = toks.into_iter().map(|(i, t)| Ok::<(usize, Tok, usize), LexError>((i, t, i + 1)));
        let tree = angloform::StepParser::new()
            .parse(iter)
            .map_err(|e| format!("line {} (\"{}{clause}\"): a step is one clause with no coordination — {}", k + 1, kw, format_parse_error(&e)))?;
        children.push(tree);
    }
    Ok(Tree::Node { label: "Steps", head: 0, children })
}

/// intro statement ending in ":" + "- item" lines. The items enumerate the
/// intro's final noun phrase, which must be plural, counted, or "every";
/// each item is one noun phrase; a digit count must match the item count.
fn parse_enumeration(lexicon: &Lexicon, text: &str) -> Result<Tree, String> {
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    let intro_tokens = lexicon.tokenize(lines[0]).map_err(|e| e.to_string())?;
    let iter = intro_tokens
        .into_iter()
        .map(|(i, t)| Ok::<(usize, Tok, usize), LexError>((i, t, i + 1)));
    let intro = angloform::IntroParser::new()
        .parse(iter)
        .map_err(|e| format!("intro: {}", format_parse_error(&e)))?;
    let expected = enumerated_count(&intro)?;
    let mut children = vec![intro];
    for (k, line) in lines[1..].iter().enumerate() {
        let item = line.trim_start_matches("- ").trim();
        let toks = lexicon
            .tokenize(item)
            .map_err(|e| format!("item {}: {e}", k + 1))?;
        let iter = toks
            .into_iter()
            .map(|(i, t)| Ok::<(usize, Tok, usize), LexError>((i, t, i + 1)));
        let tree = angloform::ItemParser::new()
            .parse(iter)
            .map_err(|_| format!("item {} (\"{item}\") is not a noun phrase — an item names one thing (ADR 0028)", k + 1))?;
        children.push(tree);
    }
    let n = lines.len() - 1;
    if let Some(c) = expected {
        if c != n {
            return Err(format!("the intro counts {c} but the list has {n} items (ADR 0028)"));
        }
    }
    Ok(Tree::Node { label: "Enum", head: 0, children })
}

/// intro statement ending in ":" + 2-column table rows (ADR 0051). Each
/// cell is validated independently as one noun phrase (same `Item` rule
/// an Enumeration item uses) — a Mapping states a key -> value fact per
/// row, not a sentence, so there is no cross-cell grammar to check beyond
/// "each cell is a real noun phrase."
fn parse_mapping(lexicon: &Lexicon, text: &str) -> Result<Tree, String> {
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    let intro_tokens = lexicon.tokenize(lines[0]).map_err(|e| e.to_string())?;
    let iter = intro_tokens
        .into_iter()
        .map(|(i, t)| Ok::<(usize, Tok, usize), LexError>((i, t, i + 1)));
    let intro = angloform::IntroParser::new()
        .parse(iter)
        .map_err(|e| format!("intro: {}", format_parse_error(&e)))?;
    let mut children = vec![intro];
    for (k, line) in lines[1..].iter().enumerate() {
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() != 2 {
            return Err(format!("row {}: a Mapping row has 2 cells (ADR 0051)", k + 1));
        }
        for (col, cell) in cells.iter().enumerate() {
            let toks = lexicon
                .tokenize(cell)
                .map_err(|e| format!("row {} cell {}: {e}", k + 1, col + 1))?;
            let iter = toks
                .into_iter()
                .map(|(i, t)| Ok::<(usize, Tok, usize), LexError>((i, t, i + 1)));
            let tree = angloform::ItemParser::new()
                .parse(iter)
                .map_err(|_| format!("row {} cell {} (\"{cell}\") is not a noun phrase — a Mapping cell names one thing (ADR 0051)", k + 1, col + 1))?;
            children.push(tree);
        }
    }
    Ok(Tree::Node { label: "Mapping", head: 0, children })
}

/// The intro's enumerated noun phrase: the last child of the predicate must
/// be a plural / every / counted NP (no trailing PP, no coordination tail).
/// Returns the digit count when one is given.
fn enumerated_count(intro: &Tree) -> Result<Option<usize>, String> {
    let Tree::Node { children, .. } = intro else { return Err("intro is not a statement".into()) };
    let Some(Tree::Node { label: "S", children: s, .. }) = children.first() else {
        return Err("the intro of an Enumeration must be a plain statement (ADR 0028)".into());
    };
    if s.len() != 2 {
        return Err("the intro of an Enumeration cannot carry a coordination tail (ADR 0028)".into());
    }
    let mut pred_node = &s[1];
    // negation, modals, and a medial adverb (ADR 0044) wrap the verb phrase:
    // descend to it
    while let Tree::Node { label: "NegVP" | "ModalVP" | "PredAdv", children, .. } = pred_node {
        pred_node = children.last().expect("wrapped VP");
    }
    let Tree::Node { label: pred, children: pc, .. } = pred_node else {
        return Err("intro predicate".into());
    };
    let np = match (*pred, pc.last()) {
        ("VP", Some(t @ Tree::Node { label, .. })) if label.starts_with("NP") => t,
        ("CopPred", Some(t @ Tree::Node { label, .. })) if label.starts_with("NP") => t,
        _ => return Err("the intro must end in the noun phrase the items enumerate — no trailing prepositional phrase or adjective (ADR 0028)".into()),
    };
    // "only" (ADR 0047) wraps the NP without changing its count/shape:
    // descend past it before reading the noun phrase's own shape.
    let np = if let Tree::Node { label: "NPOnly", children, .. } = np {
        children.last().expect("wrapped NP")
    } else {
        np
    };
    let Tree::Node { label, children: nc, .. } = np else { unreachable!() };
    let leaves: Vec<&str> = nc.iter().filter_map(|c| if let Tree::Leaf { word, .. } = c { Some(word.as_str()) } else { None }).collect();
    let first = leaves.first().copied().map(|w| if w == "the" && leaves.len() > 1 { leaves[1] } else { w });
    let last_leaf = nc.iter().rev().find_map(|c| if let Tree::Leaf { word, .. } = c { Some(word.as_str()) } else { None });
    match (*label, first) {
        ("NPGen", _) => Ok(None),
        ("NPPct", _) => Ok(None),
        ("NP", Some(w)) if w == "every" => Ok(None),
        ("NP", Some(w)) if w.chars().all(|c| c.is_ascii_digit()) => Ok(w.parse().ok()),
        ("NP", Some(w)) if w == "about" || w == "~" => Ok(None),
        ("NP", _) if last_leaf.is_some_and(|l| l.ends_with('s')) && nc.len() >= 2 => Ok(None),
        _ => Err("the enumerated noun phrase must be plural, counted (\"3 pronouns\"), or \"every <noun>\" (ADR 0028)".into()),
    }
}

pub fn parse(lexicon: &Lexicon, sentence: &str) -> Result<Tree, String> {
    let tokens = lexicon
        .tokenize(sentence)
        .map_err(|e| e.to_string())?;
    let iter = tokens
        .into_iter()
        .map(|(i, t)| Ok::<(usize, Tok, usize), LexError>((i, t, i + 1)));
    angloform::SentenceParser::new()
        .parse(iter)
        .map_err(|e| format_parse_error(&e))
}

/// Parse an already-tokenized stream, keeping the raw `ParseError` (with
/// its failure position) rather than formatting it to a string — used by
/// `diagnose` to rank antiparser matches by proximity to where Tier-1
/// actually choked, once the caller already has the token stream (e.g.
/// after `Lexicon::tokenize`, for the Tier-2 fallback path).
pub fn parse_tokens(toks: &[Tok]) -> Result<Tree, ParseError> {
    let iter = toks
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, t)| Ok::<(usize, Tok, usize), LexError>((i, t, i + 1)));
    angloform::SentenceParser::new().parse(iter)
}

/// The token position where Tier-1 failed, when the error carries one.
pub fn failure_position(e: &ParseError) -> Option<usize> {
    match e {
        lalrpop_util::ParseError::UnrecognizedToken { token: (pos, ..), .. } => Some(*pos),
        lalrpop_util::ParseError::UnrecognizedEof { location, .. } => Some(*location),
        _ => None,
    }
}

fn format_parse_error(e: &ParseError) -> String {
    match e {
        lalrpop_util::ParseError::UnrecognizedToken { token: (pos, tok, _), .. } => {
            format!("unexpected {tok:?} at word {pos} — no sanctioned sentence structure continues this way")
        }
        lalrpop_util::ParseError::UnrecognizedEof { .. } => {
            "sentence ended before a sanctioned structure was complete".to_string()
        }
        other => format!("{other:?}"),
    }
}
