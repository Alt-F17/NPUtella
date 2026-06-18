import { useCallback, useEffect, useState } from 'react'
import type { PillLanguage, PillPhase } from '../components/pill/PillWidget'

interface DemoStep {
  phase: PillPhase
  duration: number
  resultText?: string
}

const PHASE_SEQUENCE: DemoStep[] = [
  { phase: 'idle', duration: 3200 },
  { phase: 'recording', duration: 2600 },
  { phase: 'transcribing', duration: 1300 },
  { phase: 'done', duration: 2200, resultText: 'NPU ready' },
  { phase: 'idle', duration: 2800 },
  { phase: 'recording', duration: 2600 },
  { phase: 'transcribing', duration: 1300 },
  { phase: 'done', duration: 2400, resultText: 'Meeting notes pasted' },
]

const LANGUAGES: PillLanguage[] = ['bi', 'fr', 'en']

export function usePillDemo() {
  const [step, setStep] = useState(0)
  const [language, setLanguage] = useState<PillLanguage>('bi')
  const [paused, setPaused] = useState(false)

  const current = PHASE_SEQUENCE[step % PHASE_SEQUENCE.length]

  useEffect(() => {
    if (paused) return
    const id = window.setTimeout(() => setStep((s) => s + 1), current.duration)
    return () => window.clearTimeout(id)
  }, [step, paused, current.duration])

  const cycleLanguage = useCallback(() => {
    setLanguage((lang) => LANGUAGES[(LANGUAGES.indexOf(lang) + 1) % LANGUAGES.length])
  }, [])

  return {
    phase: current.phase,
    resultText: current.resultText ?? '',
    language,
    cycleLanguage,
    setPaused,
  }
}
