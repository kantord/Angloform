export type DictionaryCategory = 'NOUN' | 'VERB_TRANS' | 'VERB_INTRANS' | 'ADJ'

export type DictionaryEntry = {
  lemma: string
  category: DictionaryCategory
  definition: string
}
