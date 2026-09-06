import init, { diagnose_sentence, tokenize } from './wasm/angloform_wasm.js'
import wasmUrl from './wasm/angloform_wasm_bg.wasm?url'
import type { DiagnosisResult, TokenRow } from './types'

let ready: Promise<void> | null = null

export function loadWasm(): Promise<void> {
  if (!ready) {
    ready = init(wasmUrl).then(() => undefined).catch((e) => {
      ready = null
      throw e
    })
  }
  return ready
}

export function lintSentence(sentence: string): DiagnosisResult {
  return JSON.parse(diagnose_sentence(sentence)) as DiagnosisResult
}

export function tokenizeSentence(sentence: string): TokenRow[] {
  return JSON.parse(tokenize(sentence)) as TokenRow[]
}