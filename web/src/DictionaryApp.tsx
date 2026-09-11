import { useMemo } from 'react'
import DATA from '@/lib/dictionary-data.json'
import type { DictionaryEntry } from '@/lib/dictionary-types'

const POS_LABEL: Record<string, string> = {
  NOUN: 'n.',
  VERB_TRANS: 'v.t.',
  VERB_INTRANS: 'v.i.',
  ADJ: 'adj.',
}

const LETTERS = 'abcdefghijklmnopqrstuvwxyz'.split('')

function renderDefinition(entry: DictionaryEntry) {
  const words = entry.definition.split(' ')
  if (entry.category === 'ADJ') {
    if (words[0] === 'not') {
      return (
        <>
          <em className="not-italic font-semibold text-primary">not</em>{' '}
          {words.slice(1).join(' ')}
        </>
      )
    }
    return entry.definition
  }
  if (entry.category.startsWith('VERB')) {
    return (
      <>
        <em className="italic">{words[0]}</em> {words.slice(1).join(' ')}
      </>
    )
  }
  const m = entry.definition.match(/^(an?)\s+(\S+)(,.*)$/)
  if (m) {
    return (
      <>
        {m[1]} <span className="font-semibold text-primary">{m[2]}</span>
        {m[3]}
      </>
    )
  }
  return entry.definition
}

export default function DictionaryApp() {
  const entries = DATA as DictionaryEntry[]

  const counts = useMemo(() => {
    const c: Record<string, number> = {}
    for (const e of entries) c[e.category] = (c[e.category] ?? 0) + 1
    return c
  }, [entries])

  const byLetter = useMemo(() => {
    const g: Record<string, DictionaryEntry[]> = {}
    for (const e of entries) {
      const l = e.lemma[0].toLowerCase()
      ;(g[l] ??= []).push(e)
    }
    return g
  }, [entries])

  return (
    <div className="mx-auto flex max-w-3xl flex-col gap-9 px-4 py-8">
      <header className="flex flex-col gap-2 border-b border-border pb-7">
        <div className="flex items-center justify-between gap-3">
          <span className="font-mono text-[0.68rem] tracking-wide text-primary uppercase">
            seed/definitions/*.yaml &middot; ADR 0061
          </span>
          <a
            href="./index.html"
            className="text-xs text-muted-foreground underline-offset-4 hover:text-foreground hover:underline"
          >
            &larr; linter playground
          </a>
        </div>
        <h1
          className="text-4xl font-semibold tracking-tight text-balance"
          style={{ fontFamily: "'Fraunces Variable', var(--font-heading)" }}
        >
          Angloform Dictionary
        </h1>
        <p className="max-w-[62ch] text-sm leading-relaxed text-muted-foreground">
          Every word with a checked definition — parsed by Angloform's own
          LALR(1) grammar (<code className="rounded bg-primary/10 px-1 py-0.5 text-[0.85em] text-primary">parse_definition</code>),
          not just typed in. Each entry follows the same shape: a genus and a
          differentia (<em>"a thing, which…"</em>), or a bare antonym
          (<em>"not…"</em>). This excludes most of the lexicon — the rest
          still lives in <code className="rounded bg-primary/10 px-1 py-0.5 text-[0.85em] text-primary">seed.json</code> with
          no definition text at all.
        </p>
        <div className="mt-2 flex flex-wrap gap-6">
          <Stat n={entries.length} label="words" />
          <Stat n={counts.NOUN ?? 0} label="nouns" />
          <Stat n={counts.ADJ ?? 0} label="adjectives" />
          <Stat n={counts.VERB_TRANS ?? 0} label="verbs" />
        </div>
      </header>

      <nav aria-label="Jump to letter" className="flex flex-wrap gap-1 font-mono text-sm">
        {LETTERS.map((l) =>
          byLetter[l] ? (
            <a
              key={l}
              href={`#letter-${l}`}
              className="flex size-6 items-center justify-center rounded text-muted-foreground uppercase hover:bg-primary/10 hover:text-primary"
            >
              {l}
            </a>
          ) : (
            <span
              key={l}
              className="flex size-6 items-center justify-center text-muted-foreground/30 uppercase"
            >
              {l}
            </span>
          )
        )}
      </nav>

      <main className="flex flex-col gap-6">
        {LETTERS.filter((l) => byLetter[l]).map((l) => (
          <section key={l} className="flex flex-col gap-0.5">
            <h2
              id={`letter-${l}`}
              className="scroll-mt-2 border-b border-border pb-1.5 text-lg font-semibold text-primary"
              style={{ fontFamily: "'Fraunces Variable', var(--font-heading)" }}
            >
              {l.toUpperCase()}
            </h2>
            <dl className="flex flex-col">
              {byLetter[l].map((e) => (
                <div
                  key={e.lemma}
                  className="grid grid-cols-1 gap-1 border-b border-border py-2.5 last:border-b-0 sm:grid-cols-[minmax(140px,220px)_1fr] sm:items-baseline sm:gap-4"
                >
                  <div className="flex flex-wrap items-baseline gap-2">
                    <dt
                      className="text-[1.05rem] font-semibold"
                      style={{ fontFamily: "'Fraunces Variable', var(--font-heading)" }}
                    >
                      {e.lemma}
                    </dt>
                    <span className="rounded bg-primary/10 px-1.5 py-px font-mono text-[0.65rem] text-primary">
                      {POS_LABEL[e.category] ?? e.category}
                    </span>
                  </div>
                  <dd className="text-sm leading-relaxed">{renderDefinition(e)}</dd>
                </div>
              ))}
            </dl>
          </section>
        ))}
      </main>

      <footer className="border-t border-border pt-5 text-xs leading-relaxed text-muted-foreground">
        Generated at build time from{' '}
        <code className="font-mono">seed/definitions/*.yaml</code> — see{' '}
        <code className="font-mono">scripts/build-dictionary-data.py</code>{' '}
        and <code className="font-mono">docs/vocab-ratchet-findings-2026-09-11.md</code>{' '}
        for how this word list was derived.
      </footer>
    </div>
  )
}

function Stat({ n, label }: { n: number; label: string }) {
  return (
    <div className="flex flex-col gap-0.5">
      <span
        className="text-2xl leading-none font-semibold text-primary tabular-nums"
        style={{ fontFamily: "'Fraunces Variable', var(--font-heading)" }}
      >
        {n}
      </span>
      <span className="text-[0.68rem] tracking-wide text-muted-foreground uppercase">
        {label}
      </span>
    </div>
  )
}
