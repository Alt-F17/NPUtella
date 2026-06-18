import { useEffect, useRef, useState, type ReactNode } from 'react'

interface SectionProps {
  id?: string
  className?: string
  children: ReactNode
}

export function Section({ id, className = '', children }: SectionProps) {
  const ref = useRef<HTMLElement>(null)
  const [visible, setVisible] = useState(false)

  useEffect(() => {
    const el = ref.current
    if (!el) return
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setVisible(true)
          observer.disconnect()
        }
      },
      { threshold: 0.15 },
    )
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  return (
    <section
      id={id}
      ref={ref}
      className={`mx-auto w-full max-w-6xl px-6 ${visible ? 'animate-rise-in' : 'opacity-0'} ${className}`}
    >
      {children}
    </section>
  )
}
