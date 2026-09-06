import { beforeAll, describe, expect, it } from 'vitest'
import { readFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import init, { diagnose_sentence, tokenize } from '../lib/wasm/angloform_wasm.js'
import type { DiagnosisResult, TreeNode } from '../lib/types'

async function loadWasmForTest() {
  const bytes = await readFile(resolve(process.cwd(), 'src/lib/wasm/angloform_wasm_bg.wasm'))
  await init({ module_or_path: bytes })
}

function lint(sentence: string): DiagnosisResult {
  return JSON.parse(diagnose_sentence(sentence)) as DiagnosisResult
}

beforeAll(async () => {
  await loadWasmForTest()
})

describe('wasm linter', () => {
  it('accepts a clean sentence with metrics and a parse tree', () => {
    const result = lint('the agent reads the file')
    expect(result.kind).toBe('Clean')
    expect(result.metrics).toMatchObject({
      peak_open_deps: 2,
      max_dep_len: 2,
      embedding_depth: 1,
    })
    expect(result.tree).toBeDefined()
    const nodes = result.tree!.nodes
    expect(nodes[0]).toMatchObject({ name: 'Statement', kind: 'node', parentId: null })
    const words = nodes.filter((n: TreeNode) => n.kind === 'word')
    expect(words.map((n) => n.name)).toEqual(['the', 'agent', 'reads', 'the', 'file'])
    expect(words.map((n) => n.lemma)).toEqual(['the', 'agent', 'read', 'the', 'file'])
    for (const w of words) {
      expect(w.gloss).toBeTruthy()
    }
const parentIds = new Set(nodes.map((n) => n.parentId).filter(Boolean))
    for (const n of nodes.filter((n) => n.kind === 'node')) {
      expect(parentIds.has(n.id)).toBe(true)
    }
  })

  it('rejects a banned pronoun with a suggestion', () => {
    const result = lint('it fails')
    expect(result.kind).toBe('Word')
    expect(result.messages.join(' ')).toContain('banned')
    expect(result.messages.join(' ')).toContain('repeat the noun')
  })

  it('reports ambiguity', () => {
    const result = lint('a copy of the report is stored in the database by the system')
    expect(result.kind).toBe('Ambiguous')
    expect(result.readings).toBeGreaterThan(1)
  })

  it('tokenizes with form tags', () => {
    const tokens = JSON.parse(tokenize('the agent reads the file')) as DiagnosisResult['tokens']
    expect(tokens).toHaveLength(5)
    expect(tokens[1]).toEqual({ pos: 1, word: 'agent', tag: 'NOUN_SG' })
  })
})

describe('examples', () => {
  it('has unique labels and non-empty sentences', async () => {
    const { EXAMPLES } = await import('../lib/examples')
    expect(new Set(EXAMPLES.map((e) => e.label)).size).toBe(EXAMPLES.length)
    for (const ex of EXAMPLES) {
      expect(ex.sentence.trim().length).toBeGreaterThan(0)
    }
  })

  it('covers both clean and rejected sentences', async () => {
    const { EXAMPLES } = await import('../lib/examples')
    const kinds = new Set(EXAMPLES.map((ex) => lint(ex.sentence).kind))
    expect(kinds.has('Clean')).toBe(true)
    expect([...kinds].some((k) => k !== 'Clean')).toBe(true)
  })
})