import { Check } from 'lucide-react'

interface CheckboxProps {
  checked: boolean
  onChange: (checked: boolean) => void
  label: string
}

export function Checkbox({ checked, onChange, label }: CheckboxProps) {
  return (
    <label className="flex cursor-pointer items-center gap-2 text-sm text-zinc-300 select-none">
      <span className="relative inline-flex h-4 w-4 shrink-0 items-center justify-center">
        <input
          type="checkbox"
          checked={checked}
          onChange={(e) => onChange(e.target.checked)}
          className="peer absolute inset-0 h-full w-full cursor-pointer appearance-none rounded-[3px] border border-white/30 bg-black/35 transition-colors duration-150 ease-snap checked:border-white checked:bg-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
        />
        <Check
          className="pointer-events-none relative h-3 w-3 scale-0 text-black transition-transform duration-150 ease-snap peer-checked:scale-100"
          strokeWidth={3}
        />
      </span>
      {label}
    </label>
  )
}
