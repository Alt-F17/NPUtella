import { Section } from '../components/ui/Section'
import { DictionaryManager } from '../components/dictionary/DictionaryManager'
import { PillWidget } from '../components/pill/PillWidget'
import { usePillDemo } from '../hooks/usePillDemo'

export function ShowcaseSection() {
  const { phase, resultText, language, cycleLanguage, setPaused } = usePillDemo()

  function scrollToDictionary() {
    document.getElementById('dictionary-demo')?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }

  return (
    <Section id="showcase" className="py-20 sm:py-24">
      <div className="max-w-2xl">
        <h2 className="text-balance text-3xl font-semibold tracking-[-0.03em] text-zinc-50 sm:text-4xl">
          The native surfaces, rebuilt live.
        </h2>
        <p className="mt-3 max-w-xl text-sm leading-6 text-zinc-400 sm:text-base">
          These are the same components, rebuilt in React straight from the app&rsquo;s Rust source - not a
          video or a still image.
        </p>
      </div>

      <div className="mt-12 grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,0.82fr)_minmax(0,1.18fr)] lg:items-start">
        <div className="flex flex-col">
          <h3 className="text-lg font-medium text-zinc-50">Overlay</h3>
          <p className="mt-2 text-sm leading-6 text-zinc-400">
            Idle, recording, transcribing, and done states. Hover to reveal the dictionary and language side pills,
            exactly like the always-on-top native overlay.
          </p>
          <div className="surface-glow relative mt-6 flex min-h-56 flex-1 items-center justify-center rounded-lg border border-line bg-surface-raised px-8 py-16">
            <PillWidget
              phase={phase}
              resultText={resultText}
              language={language}
              interactive
              onLanguageClick={cycleLanguage}
              onDictionaryClick={scrollToDictionary}
              onHoverChange={setPaused}
            />
          </div>
        </div>

        <div id="dictionary-demo" className="flex flex-col">
          <h3 className="text-lg font-medium text-zinc-50">Dictionary manager</h3>
          <p className="mt-2 text-sm leading-6 text-zinc-400">
            Custom written forms, comma-separated aliases, phonetic matching, and per-entry priority - try it,
            it&rsquo;s fully interactive.
          </p>
          <DictionaryManager className="mt-6" />
        </div>
      </div>
    </Section>
  )
}
