import { useState } from 'react'
import { AudioBars } from './AudioBars'
import { StatusDot } from './StatusDot'
import { SidePill } from './SidePill'

export type PillPhase = 'idle' | 'recording' | 'transcribing' | 'done' | 'error' | 'loading'
export type PillLanguage = 'bi' | 'fr' | 'en'

interface PillWidgetProps {
  phase: PillPhase
  resultText?: string
  errorText?: string
  language: PillLanguage
  interactive?: boolean
  onLanguageClick?: () => void
  onDictionaryClick?: () => void
  onHoverChange?: (hovered: boolean) => void
  className?: string
}

const IDLE_SIZE = { w: 46, h: 18 }
const IDLE_HOVER_SIZE = { w: 50, h: 20 }
const ACTIVE_SIZE = { w: 128, h: 30 }

const SPINNER_DOTS: [number, number][] = [
  [6, 0],
  [4.24, 2.12],
  [0, 3],
  [-4.24, 2.12],
  [-6, 0],
  [-4.24, -2.12],
  [0, -3],
  [4.24, -2.12],
]

function truncate(text: string, max: number) {
  const chars = Array.from(text)
  return chars.length > max ? chars.slice(0, max).join('') + '...' : text
}

function describePhase(phase: PillPhase, resultText: string, errorText: string) {
  switch (phase) {
    case 'idle':
      return 'App status: idle'
    case 'recording':
      return 'App status: recording'
    case 'transcribing':
      return 'App status: transcribing'
    case 'done':
      return `App status: done. ${resultText || 'ready'}`
    case 'error':
      return `App status: error. ${errorText}`
    case 'loading':
      return 'App status: loading'
  }
}

export function PillWidget({
  phase,
  resultText = '',
  errorText = '',
  language,
  interactive = false,
  onLanguageClick,
  onDictionaryClick,
  onHoverChange,
  className = '',
}: PillWidgetProps) {
  const [hovered, setHovered] = useState(false)

  const isIdle = phase === 'idle'
  const showHoverGrow = isIdle && hovered && interactive
  const size = isIdle ? (showHoverGrow ? IDLE_HOVER_SIZE : IDLE_SIZE) : ACTIVE_SIZE
  const sidePillsVisible = isIdle && hovered && interactive

  const updateHover = (value: boolean) => {
    if (!interactive) return
    setHovered(value)
    onHoverChange?.(value)
  }

  return (
    <div
      className={`relative flex items-center justify-center gap-2 ${className}`}
      onMouseEnter={() => updateHover(true)}
      onMouseLeave={() => updateHover(false)}
    >
      <SidePill
        label="dict"
        side="left"
        visible={sidePillsVisible}
        fontSize={8}
        onClick={onDictionaryClick}
        ariaLabel="Open dictionary manager"
      />

      <div
        className="relative flex shrink-0 items-center justify-center overflow-hidden rounded-full border transition-[width,height,border-color,background-color,transform] duration-200 ease-snap"
        style={{
          width: size.w,
          height: size.h,
          borderColor: isIdle ? 'rgba(255,255,255,0.27)' : 'rgba(255,255,255,0.42)',
          backgroundColor: isIdle ? 'rgba(12,12,12,0.88)' : 'rgba(7,7,8,0.93)',
        }}
      >
        <span className="sr-only" role="status">
          {describePhase(phase, resultText, errorText)}
        </span>

        {isIdle && <span className="h-[7px] w-[7px] rounded-full bg-idle animate-breathe" aria-hidden="true" />}

        {phase === 'recording' && (
          <div key="recording" className="flex items-center gap-[6px] animate-fade-in" aria-hidden="true">
            <StatusDot color="#ff453a" />
            <AudioBars active />
          </div>
        )}

        {phase === 'transcribing' && (
          <div key="transcribing" className="relative h-3 w-3 animate-fade-in" aria-hidden="true">
            <Spinner />
          </div>
        )}

        {phase === 'done' && (
          <div key="done" className="flex items-center gap-2 px-3 animate-fade-in" aria-hidden="true">
            <StatusDot color="#32d74b" />
            <span className="truncate text-[9px] text-white">{truncate(resultText, 16) || 'ready'}</span>
          </div>
        )}

        {phase === 'error' && (
          <div key="error" className="px-3 animate-fade-in" aria-hidden="true">
            <span className="text-[8px] text-accent-error">! {truncate(errorText, 22)}</span>
          </div>
        )}

        {phase === 'loading' && (
          <div key="loading" className="flex items-center gap-3 px-3 animate-fade-in" aria-hidden="true">
            <LoadingDots />
            <span className="text-[8px] text-muted">loading...</span>
          </div>
        )}
      </div>

      <SidePill
        label={language}
        side="right"
        visible={sidePillsVisible}
        fontSize={9}
        onClick={onLanguageClick}
        ariaLabel={`Switch language, currently ${language}`}
      />
    </div>
  )
}

function Spinner() {
  return (
    <div className="absolute inset-0">
      {SPINNER_DOTS.map(([x, y], i) => (
        <span
          key={i}
          className="absolute h-[3.6px] w-[3.6px] rounded-full bg-zinc-300 animate-breathe"
          style={{
            left: `calc(50% + ${x}px - 1.8px)`,
            top: `calc(50% + ${y}px - 1.8px)`,
            animationDelay: `${i * 130}ms`,
          }}
        />
      ))}
    </div>
  )
}

function LoadingDots() {
  return (
    <div className="flex items-center gap-[6px]">
      {[0, 1, 2].map((i) => (
        <span
          key={i}
          className="h-[5px] w-[5px] rounded-full bg-zinc-400 animate-bounce-dot"
          style={{ animationDelay: `${i * 140}ms` }}
        />
      ))}
    </div>
  )
}
