export function Logo({ className = '' }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 32 32"
      className={className}
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <rect x="2" y="2" width="28" height="28" rx="7" stroke="currentColor" strokeOpacity="0.25" strokeWidth="1.5" />
      <path
        d="M7 17h2.6l1.8-8 3.6 16 3-12 1.6 4H23"
        stroke="#FF3B30"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}
