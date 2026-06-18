import { Logo } from '../components/icons/Logo'
import { GITHUB_ISSUES_URL, GITHUB_RELEASES_URL, GITHUB_REPO_URL } from '../lib/constants'

const LINKS = [
  { label: 'GitHub', href: GITHUB_REPO_URL },
  { label: 'Releases', href: GITHUB_RELEASES_URL },
  { label: 'Issues', href: GITHUB_ISSUES_URL },
]

export function Footer() {
  return (
    <footer className="mx-auto w-full max-w-7xl px-5 py-12 sm:px-6">
      <div className="flex flex-col items-center gap-6 border-t border-line pt-10 sm:flex-row sm:items-start sm:justify-between">
        <div className="flex flex-col items-center gap-2 sm:items-start">
          <div className="flex items-center gap-2">
            <Logo className="h-6 w-6 text-zinc-400" />
            <span className="text-sm font-medium text-zinc-200">NPUtella</span>
          </div>
          <p className="font-mono text-xs text-muted">Local NPU Whisper STT for Snapdragon X Plus</p>
        </div>
        <nav aria-label="Footer" className="flex items-center gap-6">
          {LINKS.map((link) => (
            <a
              key={link.label}
              href={link.href}
              target="_blank"
              rel="noopener noreferrer"
              className="rounded-md text-sm text-zinc-400 transition-colors duration-150 ease-snap hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
            >
              {link.label}
            </a>
          ))}
        </nav>
      </div>
      <p className="mt-8 text-center font-mono text-xs text-muted sm:text-left">
        &copy; {new Date().getFullYear()} NPUtella
      </p>
    </footer>
  )
}
