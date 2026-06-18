# NPUtella website

Marketing site for [NPUtella](https://github.com/Alt-F17/NPUtella), recreating the app's pill overlay and dictionary
manager as live React components instead of screenshots.

## Stack

Vite + React + TypeScript + Tailwind CSS. Self-hosted fonts via `@fontsource` (Bricolage Grotesque, IBM Plex Sans,
IBM Plex Mono). No backend, no router — single page.

## Development

```bash
npm install
npm run dev       # local dev server
npm run build     # tsc -b && vite build
npm run preview   # serve the production build locally
npm run lint
```

## Structure

```
src/
  components/
    ui/          buttons, badges, the scroll-reveal Section wrapper
    pill/        the floating pill overlay (idle/recording/transcribing/done/error/loading)
    dictionary/  the dictionary manager window
    decor/       ambient waveform background
    icons/       hand-built logo mark
  sections/      the page, top to bottom (Hero, Features, Showcase, HowItWorks, DownloadCTA, Footer)
  hooks/         usePillDemo (auto-cycling phase demo), useInView (scroll-triggered stagger)
  lib/           constants (GitHub URLs) and small helpers
```

## Deployment

Static Vite SPA, deployed on Vercel using the automatic Vite framework preset (no custom `vercel.json` needed):
install `npm install`, build `vite build`, output `dist`.

---

Need a website for **your** business? Visit [felixegan.me/studio](https://felixegan.me/studio).
