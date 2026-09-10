# Reference data

Used by `lexgen`/`triage` for **checking only** — never for choosing words
(see ADR 0001). Pinned by checksum in `checksums.sha256`; verify with
`sha256sum -c checksums.sha256` from this directory.

Permissively licensed files (WordNet, moby) are vendored in git. The
**CC BY-SA 4.0** files (`ud/`, `freq/`) are NOT vendored — run
`scripts/fetch-data.sh` once after cloning; it downloads/derives them from
pinned upstream versions and verifies these checksums, keeping the
repository itself entirely MIT/Apache.

| Path | Source | License |
|---|---|---|
| `wordnet/index.{noun,verb,adj,adv}` | WordNet 3.0, Princeton University (<https://wordnetcode.princeton.edu/3.0/WNdb-3.0.tar.gz>) | WordNet License (BSD-style permissive) — see `wordnet/LICENSE` |
| `wordnet/{noun,verb,adj,adv}.exc` | WordNet 3.0, Princeton University — the full package (<https://wordnetcode.princeton.edu/3.0/WordNet-3.0.tar.gz>, a different tarball from `index.*` above; same project, same license text) — morphological exceptions (irregular inflected form → lemma) | WordNet License (BSD-style permissive) — identical text to `wordnet/LICENSE` |
| `wordnet/data.{noun,verb,adj,adv}` | WordNet 3.0, Princeton University — same full-package tarball as the `.exc` files above (`dict/data.*`) — full synset membership and glosses (a lemma's actual sense text), used to show a rejected sense's real meaning during word authoring, never used to pick words | WordNet License (BSD-style permissive) — identical text to `wordnet/LICENSE` |
| `moby/mobypos.txt` | Moby Part-of-Speech II, Grady Ward, via Project Gutenberg #3203 | Public domain |
| `ud/en_ewt-ud-test.conllu` (fetched) | UD_English-EWT **r2.16** test split (<https://github.com/UniversalDependencies/UD_English-EWT>) | **CC BY-SA 4.0** — gold-tagged evaluation corpus for `triage`; data only, never enters generated lexicon |
| `freq/en_zipf.tsv` (derived at fetch time) | Derived from the `wordfreq` 3.1.1 Python package (Robyn Speer), English "best" wordlist, top 100k words | **CC BY-SA 4.0** (wordfreq's data license). The derived TSV remains CC BY-SA 4.0; attribution: wordfreq, <https://github.com/rspeer/wordfreq> |

The CC BY-SA file is data, kept separate from the project's code license; it
is only read to print frequency warnings in the lexicon report.

## Format notes

- WordNet index files: license header lines start with two spaces; data lines
  are `lemma pos synset_cnt ...` (space-separated, lemma uses `_` for spaces).
- WordNet `.exc` files: `inflected_form base_form[ base_form2 ...]`,
  space-separated, one inflected form per line (a form can map to more than
  one base, e.g. irregular plurals with two valid singulars).
- WordNet `data.*` files: license header lines start with two spaces; data
  lines are `synset_offset lex_filenum pos w_cnt word lex_id [word lex_id ...]
  p_cnt [pointer...] | gloss` — the text after `|` is the synset's gloss
  (definition + usage examples), the only reason this file is vendored.
- mobypos: `word\CODES` per line (backslash separator), CRLF line endings.
  Codes: N noun, p plural, h noun phrase, V verb (participle), t transitive
  verb, i intransitive verb, A adjective, v adverb, C conjunction,
  P preposition, ! interjection, r pronoun, D/I articles, o nominative.
- en_zipf.tsv: `word\tzipf`, one header comment line.
