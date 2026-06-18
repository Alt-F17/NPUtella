import { useEffect, useRef } from 'react'

const BAR_COUNT = 15
const BAR_W = 2
const BAR_GAP = 2
const BAR_MAX_H = 16
const BAR_MIN_H = 2

interface AudioBarsProps {
  active: boolean
}

export function AudioBars({ active }: AudioBarsProps) {
  const barRefs = useRef<(HTMLSpanElement | null)[]>([])
  const frame = useRef<number>(0)

  useEffect(() => {
    if (!active) return
    const start = performance.now()

    const tick = (now: number) => {
      const t = (now - start) / 1000
      for (let i = 0; i < BAR_COUNT; i++) {
        const el = barRefs.current[i]
        if (!el) continue
        const sensitivity = 0.8 + 0.4 * Math.sin(i * 1.3)
        const simulatedLevel = 0.45 + 0.4 * Math.sin(t * 4 + i * 0.5)
        const val = Math.min(Math.max(simulatedLevel * sensitivity, 0), 1)
        const jitter = Math.sin(t * 8 + i * 0.72) * 0.7
        const h = Math.max(BAR_MIN_H, BAR_MIN_H + (BAR_MAX_H - BAR_MIN_H) * val + jitter)
        el.style.height = `${h}px`
        const dist = Math.abs(i - BAR_COUNT / 2) / (BAR_COUNT / 2)
        el.style.opacity = `${1 - dist * 0.35}`
      }
      frame.current = requestAnimationFrame(tick)
    }

    frame.current = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(frame.current)
  }, [active])

  return (
    <span className="flex items-center" style={{ gap: BAR_GAP }} aria-hidden="true">
      {Array.from({ length: BAR_COUNT }).map((_, i) => (
        <span
          key={i}
          ref={(el) => {
            barRefs.current[i] = el
          }}
          className="rounded-[1px] bg-accent-record"
          style={{ width: BAR_W, height: BAR_MIN_H }}
        />
      ))}
    </span>
  )
}
