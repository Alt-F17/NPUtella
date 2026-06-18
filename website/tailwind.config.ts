import type { Config } from 'tailwindcss'

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: '#0a0a0a',
          raised: '#0c0c0c',
          panel: '#070708',
        },
        accent: {
          record: '#FF3B30',
          done: '#30D158',
          error: '#FF9F0A',
          warn: '#E6AF4B',
          select: '#1F7A99',
        },
        idle: '#4E4E4E',
        muted: '#888888',
        line: 'rgba(255,255,255,0.08)',
      },
      fontFamily: {
        display: ['"Bricolage Grotesque"', 'sans-serif'],
        sans: ['"IBM Plex Sans"', 'sans-serif'],
        mono: ['"IBM Plex Mono"', 'monospace'],
      },
      transitionTimingFunction: {
        snap: 'cubic-bezier(0.4, 0, 0.2, 1)',
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
          '0%': { opacity: '0', transform: 'translateY(10px)' },
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
        'rise-in': 'riseIn 500ms cubic-bezier(0.16, 1, 0.3, 1) both',
      },
    },
  },
  plugins: [],
} satisfies Config
