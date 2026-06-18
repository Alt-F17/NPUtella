import type { ReactNode } from 'react'
import { Section } from '../components/ui/Section'

const USAGE_ROWS: { action: ReactNode; result: ReactNode }[] = [
  { action: <>Hold <Kbd>Right Alt</Kbd></>, result: 'Starts recording and shows red audio bars' },
  { action: <>Release <Kbd>Right Alt</Kbd></>, result: <>Transcribes locally and pastes with <Kbd>Ctrl+V</Kbd></> },
  { action: 'Click idle overlay', result: 'Toggles recording' },
  { action: <>Hover and click <Kbd>dict</Kbd></>, result: 'Opens the dictionary manager' },
  {
    action: <>Hover and click <Kbd>bi</Kbd>/<Kbd>fr</Kbd>/<Kbd>en</Kbd></>,
    result: 'Cycles transcription language',
  },
  { action: 'Tap under 300 ms', result: 'Ignored to prevent accidental triggers' },
]

const SIGNAL_STEPS = [
  {
    title: 'Capture',
    description: 'Hold Right Alt. The native recorder captures 16 kHz mono audio while you speak.',
  },
  {
    title: 'Log-mel features',
    description: 'On release, audio is converted into Whisper-compatible log-mel features.',
  },
  {
    title: 'Encoder / decoder via QNN',
    description:
      'encoder.onnx and decoder.onnx run through ONNX Runtime’s QNN Execution Provider on the Snapdragon NPU, greedily decoding tokens.',
  },
  {
    title: 'Dictionary & formatting',
    description: 'The decoded text passes through your custom dictionary, snippet expansion, and smart formatting.',
  },
  {
    title: 'Clipboard + paste',
    description: 'The result is written to the clipboard and pasted into the focused app with a synthetic Ctrl+V.',
  },
]

function Kbd({ children }: { children: ReactNode }) {
  return (
    <kbd className="rounded border border-white/15 bg-white/[0.04] px-1.5 py-0.5 font-mono text-[0.8em] text-zinc-200">
      {children}
    </kbd>
  )
}

export function HowItWorks() {
  return (
    <Section id="how-it-works" className="py-24">
      <div className="text-center">
        <span className="font-mono text-[11px] uppercase tracking-[0.12em] text-muted">How it works</span>
        <h2 className="mt-3 font-display text-3xl font-semibold text-zinc-50 sm:text-4xl">
          One key, five steps, zero network calls
        </h2>
      </div>

      <div className="mt-12 grid grid-cols-1 gap-12 lg:grid-cols-2">
        <div className="overflow-x-auto rounded-xl border border-line">
          <table className="w-full border-collapse text-left text-sm">
            <thead>
              <tr className="border-b border-line bg-white/[0.02]">
                <th scope="col" className="px-4 py-3 font-display font-medium text-zinc-50">
                  Action
                </th>
                <th scope="col" className="px-4 py-3 font-display font-medium text-zinc-50">
                  Result
                </th>
              </tr>
            </thead>
            <tbody>
              {USAGE_ROWS.map((row, i) => (
                <tr key={i} className="border-b border-line last:border-b-0">
                  <td className="px-4 py-3 text-zinc-200">{row.action}</td>
                  <td className="px-4 py-3 text-zinc-400">{row.result}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <ol className="flex flex-col gap-5">
          {SIGNAL_STEPS.map((step, i) => (
            <li key={step.title} className="flex gap-4">
              <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-accent-record/40 font-mono text-xs text-accent-record">
                {i + 1}
              </span>
              <div>
                <h3 className="font-display text-base text-zinc-50">{step.title}</h3>
                <p className="mt-1 text-sm leading-relaxed text-zinc-400">{step.description}</p>
              </div>
            </li>
          ))}
        </ol>
      </div>
    </Section>
  )
}
