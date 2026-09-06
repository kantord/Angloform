import { useEffect, useState } from 'react'
import { Spinner } from '@/components/ui/spinner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Textarea } from '@/components/ui/textarea'
import { Separator } from '@/components/ui/separator'
import { ResultPanel } from '@/components/ResultPanel'
import { EXAMPLES } from '@/lib/examples'
import { lintSentence, loadWasm } from '@/lib/wasm'
import type { DiagnosisResult } from '@/lib/types'

const DEFAULT_SENTENCE = 'the agent reads the file'

export default function App() {
  const [sentence, setSentence] = useState(DEFAULT_SENTENCE)

  const [result, setResult] = useState<DiagnosisResult | null>(null)
  const [ready, setReady] = useState(false)
  const [failed, setFailed] = useState(false)

  useEffect(() => {
    let cancelled = false
    loadWasm()
      .then(() => {
        if (cancelled) return
        setReady(true)
      })
      .catch(() => {
        if (cancelled) return
        setFailed(true)
      })
    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    if (ready) {
      setResult(lintSentence(DEFAULT_SENTENCE))
    }
  }, [ready])

  function run(value: string) {
    const trimmed = value.trim()
    if (!trimmed) return
    setSentence(trimmed)
    setResult(lintSentence(trimmed))
  }

  return (
    <div className="mx-auto flex max-w-5xl flex-col gap-6 px-4 py-8">
      <header className="flex flex-col gap-1">
        <h1 className="font-heading text-3xl font-semibold tracking-tight">angloform</h1>
        <p className="text-sm text-muted-foreground">
          the linter of the project, compiled to wasm — type a sentence, pick an example,
          and see the Parse and the Rejections of the Linter.

        </p>
      </header>

      <Card>
        <CardHeader>
          <CardTitle>Sentence</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <Textarea
            value={sentence}
            onChange={(e) => setSentence(e.target.value)}
            placeholder="type a sentence of angloform…"
            rows={2}
            onKeyDown={(e) => {
              if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                run(sentence)
              }
            }}
          />
          <div className="flex items-center gap-2">
            <Button onClick={() => run(sentence)} disabled={!ready} size="sm">
              {!ready ? <Spinner /> : 'Lint'}
            </Button>
            <span className="text-xs text-muted-foreground">
              Ctrl+Enter to lint
            </span>
          </div>
        </CardContent>
      </Card>

      <div className="flex flex-col gap-2" aria-label="Examples">
        <span className="text-xs font-medium text-muted-foreground">Examples</span>
        <div className="flex flex-wrap gap-1.5">
          {EXAMPLES.map((ex) => (
            <Button
              key={ex.label}
              variant="outline"
              size="sm"
              onClick={() => run(ex.sentence)}
              title={ex.note}
            >
              {ex.label}
            </Button>
          ))}
        </div>
      </div>

      {failed && (
        <p className="text-sm text-destructive">
          failed to load the wasm module — check the console.
        </p>
      )}

      {result && (
        <>
          <Separator />
          <ResultPanel result={result} />
        </>
      )}
    </div>
  )
}