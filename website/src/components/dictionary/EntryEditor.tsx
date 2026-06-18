import { Checkbox } from '../ui/Checkbox'
import type { DictionaryEntryData, DictionaryLanguage } from './types'

interface EntryEditorProps {
  entry: DictionaryEntryData | null
  onChange: (patch: Partial<DictionaryEntryData>) => void
  onDelete: () => void
}

const LANGUAGES: { value: DictionaryLanguage; label: string }[] = [
  { value: 'any', label: 'Any' },
  { value: 'english', label: 'English' },
  { value: 'french', label: 'French' },
]

function inputClasses() {
  return 'w-full rounded border border-white/10 bg-white/[0.03] px-2 py-1.5 text-sm text-zinc-100 transition-colors duration-150 ease-snap placeholder:text-zinc-500 hover:border-white/20 focus-visible:border-accent-record focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent-record'
}

export function EntryEditor({ entry, onChange, onDelete }: EntryEditorProps) {
  if (!entry) {
    return (
      <div className="flex h-full flex-1 items-center justify-center text-sm text-muted">
        Add an entry to get started.
      </div>
    )
  }

  return (
    <div className="flex flex-1 flex-col pl-0 pt-4 sm:pl-4 sm:pt-0">
      <div className="flex items-center justify-between">
        <span className="font-semibold text-zinc-100">Entry</span>
        <button
          type="button"
          onClick={onDelete}
          className="rounded px-1.5 py-0.5 text-sm text-zinc-300 transition-colors duration-150 ease-snap hover:text-accent-record focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record"
        >
          Delete
        </button>
      </div>

      <div className="mt-4 flex flex-col gap-3">
        <label className="flex flex-col gap-1 text-sm sm:flex-row sm:items-center sm:gap-3">
          <span className="w-[72px] shrink-0 text-zinc-400">Written</span>
          <input
            type="text"
            value={entry.written}
            onChange={(e) => onChange({ written: e.target.value })}
            className={inputClasses()}
          />
        </label>

        <label className="flex flex-col gap-1 text-sm sm:flex-row sm:items-center sm:gap-3">
          <span className="w-[72px] shrink-0 text-zinc-400">Aliases</span>
          <input
            type="text"
            value={entry.aliases}
            onChange={(e) => onChange({ aliases: e.target.value })}
            className={inputClasses()}
          />
        </label>

        <p className="text-xs text-muted sm:ml-[84px]">
          Use commas between aliases, for example: nix os, nicsos, nicks os
        </p>

        <div className="flex items-center gap-5 sm:ml-[84px]">
          <Checkbox
            checked={entry.phonetic}
            onChange={(checked) => onChange({ phonetic: checked })}
            label="Phonetic"
          />
          <Checkbox
            checked={entry.highPriority}
            onChange={(checked) => onChange({ highPriority: checked })}
            label="High priority"
          />
        </div>

        <div className="flex flex-wrap items-center gap-2 sm:ml-[84px]">
          <span className="text-sm text-zinc-400">Language</span>
          {LANGUAGES.map(({ value, label }) => (
            <button
              key={value}
              type="button"
              onClick={() => onChange({ language: value })}
              aria-pressed={entry.language === value}
              className={[
                'rounded px-2.5 py-1 text-sm transition-colors duration-150 ease-snap',
                'focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record',
                entry.language === value
                  ? 'bg-accent-select text-white'
                  : 'text-zinc-300 hover:bg-white/5',
              ].join(' ')}
            >
              {label}
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
