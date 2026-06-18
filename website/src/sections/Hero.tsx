import { Download, Github } from 'lucide-react'
import { Badge } from '../components/ui/Badge'
import { Button } from '../components/ui/Button'
import { Section } from '../components/ui/Section'
import { PillWidget } from '../components/pill/PillWidget'
import { usePillDemo } from '../hooks/usePillDemo'
import { GITHUB_RELEASES_URL, GITHUB_REPO_URL } from '../lib/constants'

export function Hero() {
  const { phase, resultText, language, cycleLanguage, setPaused } = usePillDemo()

  function scrollToShowcase() {
    document.getElementById('showcase')?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }

  return (
    <Section id="hero" className="relative min-h-[calc(100dvh-57px)] py-16 sm:py-20 lg:py-24">
      <div className="grid min-h-[calc(100dvh-12rem)] grid-cols-1 items-center gap-12 lg:grid-cols-[minmax(0,0.92fr)_minmax(0,1.08fr)]">
        <div className="relative z-10 max-w-2xl">
          <Badge>Snapdragon X Plus / local Whisper STT</Badge>
          <h1 className="mt-6 text-balance text-5xl font-semibold leading-[0.95] tracking-[-0.04em] text-zinc-50 sm:text-6xl lg:text-7xl">
            Dictation that stays on your machine.
          </h1>
          <p className="mt-6 max-w-xl text-base leading-7 text-zinc-300 sm:text-lg">
            Hold Ctrl, then Win, speak, release. NPUtella runs Whisper through Qualcomm&rsquo;s NPU, applies
            your dictionary, and pastes the result wherever your cursor already is.
          </p>
          <div className="mt-8 flex flex-wrap items-center gap-3">
            <Button href={GITHUB_RELEASES_URL} variant="primary" icon={<Download className="h-4 w-4" />}>
              Download for Windows
            </Button>
            <Button href={GITHUB_REPO_URL} variant="secondary" icon={<Github className="h-4 w-4" />}>
              GitHub
            </Button>
          </div>
          <dl className="mt-10 grid max-w-lg grid-cols-3 gap-4 border-t border-line pt-6">
            {[
              ['0', 'cloud calls'],
              ['Ctrl+Win', 'capture flow'],
              ['NPU', 'accelerated'],
            ].map(([value, label]) => (
              <div key={label}>
                <dt className="font-mono text-sm text-zinc-50">{value}</dt>
                <dd className="mt-1 text-xs text-zinc-500">{label}</dd>
              </div>
            ))}
          </dl>
        </div>

        <div className="relative">
          <div className="absolute inset-x-6 -top-10 h-40 rounded-full bg-white/[0.05] blur-3xl" aria-hidden="true" />
          <div className="surface-glow relative overflow-hidden rounded-lg border border-line bg-surface-raised shadow-2xl shadow-black">
            <div className="flex items-center justify-between border-b border-line px-4 py-3">
              <div className="flex items-center gap-2">
                <span className="h-2.5 w-2.5 rounded-full bg-zinc-600" />
                <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
                <span className="h-2.5 w-2.5 rounded-full bg-zinc-800" />
              </div>
              <span className="font-mono text-xs text-zinc-500">live overlay</span>
            </div>
            <div className="relative min-h-[410px] px-5 py-6 sm:px-8">
              <div className="grid gap-3 text-sm text-zinc-500">
                <div className="h-3 w-3/4 rounded bg-white/[0.06]" />
                <div className="h-3 w-1/2 rounded bg-white/[0.04]" />
                <div className="mt-6 h-24 rounded-md border border-line bg-black/40" />
                <div className="grid grid-cols-2 gap-3">
                  <div className="h-24 rounded-md border border-line bg-black/35" />
                  <div className="h-24 rounded-md border border-line bg-black/35" />
                </div>
              </div>
              <div className="absolute bottom-10 left-1/2 -translate-x-1/2">
                <PillWidget
                  phase={phase}
                  resultText={resultText}
                  language={language}
                  interactive
                  onLanguageClick={cycleLanguage}
                  onDictionaryClick={scrollToShowcase}
                  onHoverChange={setPaused}
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </Section>
  )
}
