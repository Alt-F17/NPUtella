import type { ReactNode } from 'react'

interface ButtonProps {
  href: string
  variant?: 'primary' | 'secondary'
  icon?: ReactNode
  children: ReactNode
  className?: string
}

const VARIANTS = {
  primary: 'bg-accent-record text-white shadow-lg shadow-accent-record/20 hover:bg-[#ff5147]',
  secondary: 'border border-white/15 text-zinc-100 hover:border-white/30 hover:bg-white/5',
}

export function Button({ href, variant = 'primary', icon, children, className = '' }: ButtonProps) {
  const isExternal = href.startsWith('http')
  return (
    <a
      href={href}
      className={[
        'inline-flex items-center justify-center gap-2 rounded-full px-6 py-3 text-sm font-medium',
        'transition-all duration-200 ease-snap active:scale-[0.97]',
        'focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record',
        VARIANTS[variant],
        className,
      ].join(' ')}
      {...(isExternal ? { target: '_blank', rel: 'noopener noreferrer' } : {})}
    >
      {icon}
      {children}
    </a>
  )
}
