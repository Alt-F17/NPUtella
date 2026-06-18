import type { Config } from 'tailwindcss'

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: '#000000',
          raised: '#0a0a0a',
          panel: '#111111',
          inset: '#050505',
        },
        accent: {
          record: '#ff453a',
          done: '#32d74b',
          error: '#ffb340',
          warn: '#ffd60a',
          select: '#ffffff',
        },
        idle: '#737373',
        muted: '#a1a1aa',
        line: 'rgba(255,255,255,0.12)',
      },
      fontFamily: {
        display: ['"Geist"', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        sans: ['"Geist"', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        mono: ['"Geist Mono"', 'ui-monospace', 'SFMono-Regular', 'monospace'],
      },
      transitionTimingFunction: {
        snap: 'cubic-bezier(0.23, 1, 0.32, 1)',
        reveal: 'cubic-bezier(0.16, 1, 0.3, 1)',
      },
      keyframes: {
        breathe: {
          '0%, 100%': { opacity: '0.4' },
          '50%': { opacity: '1' },
        },
        orbit: {
          '0%': { transform: 'rotate(0deg)' },
          '100%': { transform: 'rotate(360deg)' },
        },
        bounceDot: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-4px)' },
        },
        drift: {
          '0%': { transform: 'translateX(0)' },
          '100%': { transform: 'translateX(-50%)' },
        },
        grainShift: {
          '0%': { transform: 'translate(0, 0)' },
          '100%': { transform: 'translate(-4%, 3%)' },
        },
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        riseIn: {
          '0%': { opacity: '0', transform: 'translateY(8px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
      },
      animation: {
        breathe: 'breathe 2.4s ease-in-out infinite',
        orbit: 'orbit 1.6s linear infinite',
        'bounce-dot': 'bounceDot 1.1s ease-in-out infinite',
        drift: 'drift 18s linear infinite',
        grain: 'grainShift 1.4s steps(2) infinite',
        'fade-in': 'fadeIn 180ms cubic-bezier(0.4, 0, 0.2, 1) both',
        'rise-in': 'riseIn 360ms cubic-bezier(0.16, 1, 0.3, 1) both',
      },
    },
  },
  plugins: [],
} satisfies Config
