import { Github } from 'lucide-react'
import { AppBackground } from './components/decor/AppBackground'
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
    <header className="sticky top-0 z-50 border-b border-line bg-black/75 backdrop-blur-xl">
      <div className="mx-auto flex max-w-7xl items-center justify-between px-5 py-3 sm:px-6">
        <a
          href="#hero"
          className="flex items-center gap-2 rounded-md focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
        >
          <Logo className="h-6 w-6 text-zinc-50" />
          <span className="text-sm font-medium text-zinc-50">NPUtella</span>
        </a>
        <nav aria-label="Primary" className="hidden items-center gap-6 sm:flex">
          {NAV_LINKS.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="rounded-md text-sm text-zinc-400 transition-colors duration-150 ease-snap hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
            >
              {link.label}
            </a>
          ))}
        </nav>
        <a
          href={GITHUB_REPO_URL}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-2 rounded-full border border-white/15 px-3 py-1.5 text-sm text-zinc-200 transition-[background-color,border-color,transform] duration-200 ease-snap hover:border-white/30 hover:bg-white/[0.06] active:scale-[0.97] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
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
    <div className="relative flex min-h-screen flex-col overflow-x-hidden bg-black">
      <AppBackground />
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
