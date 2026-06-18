import { hexToRgba } from '../../lib/color'

interface StatusDotProps {
  color: string
  size?: number
  className?: string
}

export function StatusDot({ color, size = 7, className = '' }: StatusDotProps) {
  return (
    <span
      className={`relative inline-block shrink-0 rounded-full ${className}`}
      style={{
        width: size,
        height: size,
        backgroundColor: color,
        boxShadow: `0 0 0 ${size * 0.4}px ${hexToRgba(color, 0.2)}`,
      }}
    >
      <span
        className="absolute rounded-full bg-white/70"
        style={{
          width: size * 0.22,
          height: size * 0.22,
          left: size * 0.16,
          top: size * 0.16,
        }}
      />
    </span>
  )
}
