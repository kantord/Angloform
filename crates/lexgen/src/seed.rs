//! The curated seed list: the only hand-edited input (ADR 0001).

use serde::Deserialize;
use std::collections::BTreeMap;

/// A rejected-sense entry: either a suggested replacement word, or free-text
/// advice for a collision with no natural word substitute. Replaces the old
/// silent `waive` field (see docs/adr/0060) — every excluded sense must
/// explain itself to the writer who hits it; nothing is silently dropped.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum RejectTarget {
    /// A suggested replacement word.
    Word(String),
    /// Free-text advice, shown verbatim by the Linter.
    Advice { advice: String },
    /// A real, structural absence — no already-enabled word covers this
    /// attested sense yet (docs/lexicon-authoring-format-2026-09-07.md's
    /// "Gap" concept). Renders as a fixed, templated message, never
    /// hand-authored prose. Only ever constructed from
    /// seed/definitions/*.yaml's `gap: true` (definitions::to_seed_entries)
    /// — seed.json has no equivalent field and never deserializes to this.
    Gap,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedEntry {
    pub lemma: String,
    pub category: String,
    /// Irregular-form overrides, keyed by slot name (see Category::slots).
    #[serde(default)]
    pub forms: BTreeMap<String, String>,
    /// Rejected uses: attested-but-banned POS → a redirect word or advice.
    #[serde(default)]
    pub reject: BTreeMap<String, RejectTarget>,
    /// Writer-facing advice for BANNED words (shown by the linter).
    #[serde(default)]
    pub advice: String,
    /// Domain-model entries only (ADR 0027): the term's meaning, in
    /// angloform — self-linted, shown to writers and to the repair model.
    #[serde(default)]
    pub definition: String,
    /// Domain-model entries (ADR 0036): "unique" (one thing: Lexgen, the Seed)
    /// or "category" (a kind of thing: Anaphoric Pronoun). Nouns and names.
    #[serde(default)]
    pub kind: String,
    /// Members or angloform sentences that show the thing; required for a
    /// category (imaginability: a beginner sees an example before a rule).
    #[serde(default)]
    pub examples: Vec<String>,
    /// The category this term is a member of (is-a), by lemma.
    #[serde(default)]
    pub member_of: String,
    /// True for entries loaded from domain/model.json (never serialized).
    #[serde(skip)]
    pub domain: bool,
    /// The pack this entry came from (ADR 0053), e.g. "narrative" — never
    /// serialized, set by `load_pack`. `None` for core seed and domain
    /// entries.
    #[serde(skip)]
    pub pack: Option<String>,
    /// Curation rationale. Free text, never machine-read (hence dead_code).
    #[serde(default)]
    #[allow(dead_code)]
    pub note: String,
}

pub enum Category {
    Noun,
    VerbTrans,
    VerbIntrans,
    Adj,
    Prep,
    Det,
    /// A banned surface form with writer-facing advice (ADR 0008).
    Banned,
    /// A proper name with a definition (domain model only): no forms — the
    /// lexer already treats capitalized words as names (ADR 0018).
    Name,
    /// Function words: any other UPPERCASE tag, by fiat (CONJ, NEG, …).
    Closed(String),
}

impl SeedEntry {
    pub fn cat(&self) -> Category {
        match self.category.as_str() {
            "NOUN" => Category::Noun,
            "VERB_TRANS" => Category::VerbTrans,
            "VERB_INTRANS" => Category::VerbIntrans,
            "ADJ" => Category::Adj,
            "PREP" => Category::Prep,
            "DET" => Category::Det,
            "BANNED" => Category::Banned,
            "NAME" => Category::Name,
            other => Category::Closed(other.to_string()),
        }
    }
}

impl Category {
    /// The WordNet POS letter this category occupies, if it is open-class.
    pub fn wordnet_pos(&self) -> Option<char> {
        match self {
            Category::Noun => Some('n'),
            Category::VerbTrans | Category::VerbIntrans => Some('v'),
            Category::Adj => Some('a'),
            _ => None,
        }
    }

    /// Valid `forms` override slots.
    pub fn slots(&self) -> &'static [&'static str] {
        match self {
            Category::Noun => &["plural"],
            // ADR 0030/0056: short adjectives inflect (-er/-est); "none" opts out
            Category::Adj => &["comparative", "superlative"],
            Category::VerbTrans | Category::VerbIntrans => {
                &["third", "past", "ppart", "ing"]
            }
            // a NAME may spell its capitalization ("WordNet")
            Category::Name => &["name"],
            _ => &[],
        }
    }
}

pub fn load(path: &str) -> Result<Vec<SeedEntry>, String> {
    load_with(path, false)
}

/// A vocabulary pack (ADR 0053): plain seed-shaped entries (same rules as
/// the core seed — single lowercase lemma, no domain-model fields), tagged
/// with the pack's name so the caller can enforce "packs add, never
/// override" without re-deriving which entries came from which file.
pub fn load_pack(path: &str, pack_name: &str) -> Result<Vec<SeedEntry>, String> {
    let mut entries = load(path)?;
    for e in &mut entries {
        e.pack = Some(pack_name.to_string());
    }
    Ok(entries)
}

/// Load a seed-shaped file. Domain-model entries may have multi-word lemmas
/// ("anaphoric pronoun") and must carry a definition.
pub fn load_with(path: &str, domain: bool) -> Result<Vec<SeedEntry>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut entries: Vec<SeedEntry> =
        serde_json::from_str(&text).map_err(|e| e.to_string())?;
    for e in &mut entries {
        e.domain = domain;
        let ok = !e.lemma.is_empty()
            && e.lemma
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '-' || (domain && c == ' '));
        if !ok {
            return Err(format!(
                "lemma \"{}\" must be lowercase ascii (hyphens{} allowed)",
                e.lemma,
                if domain { " and spaces" } else { "" }
            ));
        }
        if domain && e.definition.trim().is_empty() {
            return Err(format!("domain term \"{}\" needs a `definition`", e.lemma));
        }
        if domain && matches!(e.category.as_str(), "NOUN" | "NAME") {
            match e.kind.as_str() {
                "unique" | "category" => {}
                "" => return Err(format!("domain term \"{}\" needs `kind`: \"unique\" or \"category\" (ADR 0036)", e.lemma)),
                other => return Err(format!("domain term \"{}\": unknown kind \"{other}\"", e.lemma)),
            }
            if e.kind == "category" && e.examples.is_empty() {
                return Err(format!("category \"{}\" needs at least one example (ADR 0036)", e.lemma));
            }
        }
        if !domain && (!e.kind.is_empty() || !e.examples.is_empty() || !e.member_of.is_empty()) {
            return Err(format!("\"{}\": `kind`/`examples`/`member_of` belong in domain/model.json", e.lemma));
        }
        if !domain && !e.definition.is_empty() {
            return Err(format!("\"{}\": `definition` belongs in domain/model.json", e.lemma));
        }
    }
    Ok(entries)
}

/// The written form of a NAME entry: its `name` override or the capitalized lemma.
pub fn shown_name(e: &SeedEntry) -> String {
    e.forms.get("name").cloned().unwrap_or_else(|| capitalize(&e.lemma))
}

/// "anaphoric pronoun" → "Anaphoric Pronoun" (the written form of a term).
pub fn capitalize(lemma: &str) -> String {
    lemma
        .split(' ')
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
