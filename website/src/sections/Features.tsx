import { BookOpen, Keyboard, Languages, Lock } from 'lucide-react'
import { Section } from '../components/ui/Section'
import { useInView } from '../hooks/useInView'

const FEATURES = [
  {
    icon: Lock,
    title: '100% Local',
    description:
      'Audio never leaves your machine. Whisper-Base runs on-device through the Snapdragon NPU — no network calls, no cloud inference.',
  },
  {
    icon: Keyboard,
    title: 'One Key, Zero Friction',
    description:
      'Hold Right Alt, speak, release. The transcript pastes wherever your cursor is. No app switching, no clicking record.',
  },
  {
    icon: Languages,
    title: 'Multilingual',
    description: 'Auto-detect, or force English or French. Cycle languages right from the overlay with a single click.',
  },
  {
    icon: BookOpen,
    title: 'Custom Dictionary',
    description:
      'Teach it your names, jargon, and snippets. Phonetic matching and high-priority entries fix words Whisper gets wrong by default.',
  },
]

export function Features() {
  const { ref, visible } = useInView<HTMLDivElement>()

  return (
    <Section id="features" className="py-24">
      <div className="text-center">
        <span className="font-mono text-[11px] uppercase tracking-[0.12em] text-muted">Capabilities</span>
        <h2 className="mt-3 font-display text-3xl font-semibold text-zinc-50 sm:text-4xl">
          Built for the device, not the cloud
        </h2>
      </div>
      <div ref={ref} className="mt-12 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {FEATURES.map(({ icon: Icon, title, description }, i) => (
          <div
            key={title}
            style={visible ? { animationDelay: `${i * 70}ms` } : undefined}
            className={`rounded-xl border border-line bg-surface-raised/60 p-6 ${visible ? 'animate-rise-in' : 'opacity-0'}`}
          >
            <Icon className="h-5 w-5 text-accent-record" aria-hidden="true" />
            <h3 className="mt-4 font-display text-lg text-zinc-50">{title}</h3>
            <p className="mt-2 text-sm leading-relaxed text-zinc-400">{description}</p>
          </div>
        ))}
      </div>
    </Section>
  )
}
