import { Download } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { Section } from '../components/ui/Section'
import { GITHUB_RELEASES_URL } from '../lib/constants'

export function DownloadCTA() {
  return (
    <Section id="download" className="py-24">
      <div className="rounded-2xl border border-line bg-[radial-gradient(circle_at_50%_0%,rgba(255,59,48,0.08),transparent_60%)] bg-surface-raised/60 px-8 py-16 text-center">
        <h2 className="font-display text-3xl font-semibold text-zinc-50 sm:text-4xl">Get NPUtella</h2>
        <p className="mx-auto mt-3 max-w-md text-sm text-zinc-400 sm:text-base">
          One binary, one key. Latest release, straight from GitHub.
        </p>
        <div className="mt-8 flex justify-center">
          <Button href={GITHUB_RELEASES_URL} variant="primary" icon={<Download className="h-4 w-4" />}>
            Download for Windows
          </Button>
        </div>
        <p className="mx-auto mt-5 max-w-md font-mono text-xs text-muted">
          Windows on Snapdragon X Plus &middot; falls back to CPU if the Qualcomm NPU isn&rsquo;t available &middot;
          free and open source
        </p>
      </div>
    </Section>
  )
}
