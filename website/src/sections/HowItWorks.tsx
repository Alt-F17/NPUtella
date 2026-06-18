import type { ReactNode } from 'react'
import { Section } from '../components/ui/Section'

const USAGE_ROWS: { action: ReactNode; result: ReactNode }[] = [
  { action: <>Hold <Kbd>Ctrl</Kbd>, then <Kbd>Win</Kbd></>, result: 'Starts recording and shows red audio bars' },
  { action: <>Release <Kbd>Ctrl+Win</Kbd></>, result: <>Transcribes locally and pastes with <Kbd>Ctrl+V</Kbd></> },
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
    description: 'Hold Ctrl, then Win. The native recorder captures 16 kHz mono audio while you speak.',
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
    <kbd className="rounded-md border border-white/15 bg-white/[0.05] px-1.5 py-0.5 font-mono text-[0.8em] text-zinc-200">
      {children}
    </kbd>
  )
}

export function HowItWorks() {
  return (
    <Section id="how-it-works" className="py-20 sm:py-24">
      <div className="max-w-2xl">
        <h2 className="text-balance text-3xl font-semibold tracking-[-0.03em] text-zinc-50 sm:text-4xl">
          Ctrl + Win, five steps, zero network calls
        </h2>
      </div>

      <div className="mt-10 grid grid-cols-1 gap-6 lg:grid-cols-2">
        <div className="overflow-x-auto rounded-lg border border-line bg-surface-raised">
          <table className="w-full border-collapse text-left text-sm">
            <thead>
              <tr className="border-b border-line bg-white/[0.03]">
                <th scope="col" className="px-4 py-3 font-medium text-zinc-50">
                  Action
                </th>
                <th scope="col" className="px-4 py-3 font-medium text-zinc-50">
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

        <ol className="flex flex-col overflow-hidden rounded-lg border border-line bg-surface-raised">
          {SIGNAL_STEPS.map((step, i) => (
            <li key={step.title} className="flex gap-4 border-b border-line p-4 last:border-b-0">
              <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-white/20 font-mono text-xs text-zinc-200">
                {i + 1}
              </span>
              <div>
                <h3 className="text-base font-medium text-zinc-50">{step.title}</h3>
                <p className="mt-1 text-sm leading-6 text-zinc-400">{step.description}</p>
              </div>
            </li>
          ))}
        </ol>
      </div>
    </Section>
  )
}
