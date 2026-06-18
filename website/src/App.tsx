import { Github } from 'lucide-react'
import { Logo } from './components/icons/Logo'
import { Hero } from './sections/Hero'
import { Features } from './sections/Features'
import { ShowcaseSection } from './sections/ShowcaseSection'
import { HowItWorks } from './sections/HowItWorks'
import { DownloadCTA } from './sections/DownloadCTA'
import { Footer } from './sections/Footer'
import { GITHUB_REPO_URL } from './lib/constants'

const NAV_LINKS = [
  { label: 'Features', href: '#features' },
  { label: 'Showcase', href: '#showcase' },
  { label: 'How it works', href: '#how-it-works' },
]

function Header() {
  return (
    <header className="sticky top-0 z-50 border-b border-line bg-surface/80 backdrop-blur">
      <div className="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
        <a
          href="#hero"
          className="flex items-center gap-2 rounded focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record"
        >
          <Logo className="h-6 w-6 text-zinc-300" />
          <span className="font-display text-lg text-zinc-50">NPUtella</span>
        </a>
        <nav aria-label="Primary" className="hidden items-center gap-6 sm:flex">
          {NAV_LINKS.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="rounded text-sm text-zinc-400 transition-colors duration-150 ease-snap hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record"
            >
              {link.label}
            </a>
          ))}
        </nav>
        <a
          href={GITHUB_REPO_URL}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-2 rounded-full border border-white/15 px-3 py-1.5 text-sm text-zinc-200 transition-all duration-200 ease-snap hover:border-white/30 hover:bg-white/5 active:scale-[0.97] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record"
        >
          <Github className="h-4 w-4" aria-hidden="true" />
          <span className="hidden sm:inline">GitHub</span>
        </a>
      </div>
    </header>
  )
}

export default function App() {
  return (
    <div className="flex min-h-screen flex-col overflow-x-hidden">
      <Header />
      <main className="flex flex-1 flex-col">
        <Hero />
        <Features />
        <ShowcaseSection />
        <HowItWorks />
        <DownloadCTA />
      </main>
      <Footer />
    </div>
  )
}
