interface SidePillProps {
  label: string
  visible: boolean
  side: 'left' | 'right'
  fontSize?: number
  onClick?: () => void
  ariaLabel?: string
}

export function SidePill({ label, visible, side, fontSize = 8, onClick, ariaLabel }: SidePillProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      tabIndex={visible ? 0 : -1}
      aria-hidden={!visible}
      aria-label={ariaLabel ?? label}
      style={{ transformOrigin: side === 'left' ? 'right center' : 'left center', fontSize }}
      className={[
        'flex h-[20px] w-[52px] shrink-0 items-center justify-center rounded-full',
        'border border-[rgba(255,255,255,0.33)] bg-[rgba(10,10,11,0.86)] text-white/90',
        'transition-all duration-200 ease-snap active:scale-[0.97]',
        'focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record',
        visible ? 'opacity-100 scale-100' : 'pointer-events-none scale-90 opacity-0',
      ].join(' ')}
    >
      {label}
    </button>
  )
}
