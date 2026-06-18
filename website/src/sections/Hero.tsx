import { Download, Github } from 'lucide-react'
import { Badge } from '../components/ui/Badge'
import { Button } from '../components/ui/Button'
import { Section } from '../components/ui/Section'
import { Waveform } from '../components/decor/Waveform'
import { PillWidget } from '../components/pill/PillWidget'
import { usePillDemo } from '../hooks/usePillDemo'
import { GITHUB_RELEASES_URL, GITHUB_REPO_URL } from '../lib/constants'

export function Hero() {
  const { phase, resultText, language, cycleLanguage, setPaused } = usePillDemo()

  function scrollToShowcase() {
    document.getElementById('showcase')?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }

  return (
    <Section id="hero" className="relative pt-32 pb-24">
      <Waveform className="left-1/2 top-16 h-48 w-[640px] -translate-x-1/2" />

      <div className="relative mx-auto max-w-2xl text-center">
        <Badge>Snapdragon X Plus &middot; NPU-accelerated</Badge>
        <h1 className="mt-6 font-display text-6xl font-semibold tracking-tight text-zinc-50 sm:text-7xl">
          NPUtella
        </h1>
        <p className="mt-5 text-lg text-zinc-200 sm:text-xl">
          Local NPU Whisper STT &mdash; dictation that never leaves your machine.
        </p>
        <p className="mx-auto mt-3 max-w-lg text-sm text-zinc-400 sm:text-base">
          Hold a key, speak, release. NPUtella transcribes on-device with Qualcomm&rsquo;s NPU and pastes the
          result wherever your cursor is &mdash; no network, no cloud, no leaked audio.
        </p>
        <div className="mt-9 flex flex-wrap items-center justify-center gap-4">
          <Button href={GITHUB_RELEASES_URL} variant="primary" icon={<Download className="h-4 w-4" />}>
            Download for Windows
          </Button>
          <Button href={GITHUB_REPO_URL} variant="secondary" icon={<Github className="h-4 w-4" />}>
            View on GitHub
          </Button>
        </div>
      </div>

      <div className="relative mx-auto mt-20 w-full max-w-xl">
        <div className="relative overflow-hidden rounded-2xl border border-line bg-[radial-gradient(circle_at_50%_0%,rgba(255,59,48,0.07),transparent_60%)] bg-surface-raised/60 px-8 py-16">
          <span className="absolute left-6 top-5 font-mono text-[10px] uppercase tracking-widest text-muted">
            Your screen
          </span>
          <span className="absolute right-6 top-5 font-mono text-[10px] uppercase tracking-widest text-muted">
            live demo
          </span>
          <div className="absolute bottom-0 left-1/2 translate-y-1/2 -translate-x-1/2">
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
    </Section>
  )
}
