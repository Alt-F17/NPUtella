import { Download } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { Section } from '../components/ui/Section'
import { GITHUB_RELEASES_URL } from '../lib/constants'

export function DownloadCTA() {
  return (
    <Section id="download" className="py-20 sm:py-24">
      <div className="surface-glow flex flex-col items-start justify-between gap-8 rounded-lg border border-line bg-surface-raised px-6 py-8 sm:px-8 lg:flex-row lg:items-center">
        <div>
          <h2 className="text-3xl font-semibold tracking-[-0.03em] text-zinc-50 sm:text-4xl">Get NPUtella</h2>
          <p className="mt-3 max-w-md text-sm leading-6 text-zinc-400 sm:text-base">
          One binary, one shortcut. Latest release, straight from GitHub.
          </p>
          <p className="mt-5 max-w-xl font-mono text-xs text-muted">
            Windows on Snapdragon X Plus / falls back to CPU if the Qualcomm NPU is not available / free and open
            source
          </p>
        </div>
        <div className="shrink-0">
          <Button href={GITHUB_RELEASES_URL} variant="primary" icon={<Download className="h-4 w-4" />}>
            Download for Windows
          </Button>
        </div>
      </div>
    </Section>
  )
}
