import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import DictionaryApp from './DictionaryApp'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <DictionaryApp />
  </StrictMode>,
)
