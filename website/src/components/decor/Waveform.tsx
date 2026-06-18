function buildWavePath() {
  const points: string[] = []
  const width = 800
  const step = 8
  for (let x = 0; x <= width; x += step) {
    const y =
      60 +
      Math.sin(x * 0.04) * 22 +
      Math.sin(x * 0.013 + 1.3) * 14 +
      Math.sin(x * 0.21) * 4
    points.push(`${x},${y.toFixed(1)}`)
  }
  return `M${points.join(' L')}`
}

const WAVE_PATH = buildWavePath()

function WaveLine() {
  return (
    <svg viewBox="0 0 800 120" preserveAspectRatio="none" className="h-full w-1/2 shrink-0">
      <path d={WAVE_PATH} fill="none" stroke="#FF3B30" strokeWidth="1.5" />
    </svg>
  )
}

export function Waveform({ className = '' }: { className?: string }) {
  return (
    <div className={`pointer-events-none absolute overflow-hidden opacity-[0.12] ${className}`} aria-hidden="true">
      <div className="flex h-full w-[200%] animate-drift">
        <WaveLine />
        <WaveLine />
      </div>
    </div>
  )
}
