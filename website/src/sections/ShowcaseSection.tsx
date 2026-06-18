import { Badge } from '../components/ui/Badge'
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
    <Section id="showcase" className="py-24">
      <div className="text-center">
        <Badge>Recreated from the app&rsquo;s source, not a screenshot</Badge>
        <h2 className="mt-5 font-display text-3xl font-semibold text-zinc-50 sm:text-4xl">See it in action</h2>
        <p className="mx-auto mt-3 max-w-lg text-sm text-zinc-400 sm:text-base">
          These are the same components, rebuilt in React straight from the app&rsquo;s Rust source &mdash; not a
          video or a still image.
        </p>
      </div>

      <div className="mt-16 grid grid-cols-1 gap-10 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)] lg:items-start">
        <div className="flex flex-col">
          <h3 className="font-display text-xl text-zinc-50">The overlay</h3>
          <p className="mt-2 text-sm leading-relaxed text-zinc-400">
            Idle, recording, transcribing, and done states. Hover to reveal the dictionary and language side pills,
            exactly like the always-on-top native overlay.
          </p>
          <div className="relative mt-8 flex flex-1 items-center justify-center rounded-2xl border border-line bg-surface-raised/60 px-8 py-16">
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
          <h3 className="font-display text-xl text-zinc-50">The dictionary manager</h3>
          <p className="mt-2 text-sm leading-relaxed text-zinc-400">
            Custom written forms, comma-separated aliases, phonetic matching, and per-entry priority &mdash; try it,
            it&rsquo;s fully interactive.
          </p>
          <DictionaryManager className="mt-8" />
        </div>
      </div>
    </Section>
  )
}
