import { useInView } from '../../hooks/useInView'
import type { DictionaryEntryData } from './types'

interface EntryListProps {
  entries: DictionaryEntryData[]
  selectedId: string | null
  onSelect: (id: string) => void
  onAdd: () => void
}

export function EntryList({ entries, selectedId, onSelect, onAdd }: EntryListProps) {
  const { ref, visible } = useInView<HTMLDivElement>()

  return (
    <div className="flex h-full flex-col border-r border-line pr-4 sm:w-[220px] sm:shrink-0">
      <div className="flex items-center justify-between">
        <span className="font-semibold text-zinc-100">Entries</span>
        <button
          type="button"
          onClick={onAdd}
          className="rounded-md px-1.5 py-0.5 text-sm text-zinc-300 transition-colors duration-150 ease-snap hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
        >
          Add
        </button>
      </div>
      <div ref={ref} className="mt-3 flex max-h-[260px] flex-col gap-1 overflow-y-auto sm:max-h-[310px]">
        {entries.map((entry, i) => {
          const selected = entry.id === selectedId
          return (
            <button
              key={entry.id}
              type="button"
              onClick={() => onSelect(entry.id)}
              aria-current={selected}
              style={visible ? { animationDelay: `${i * 50}ms` } : undefined}
              className={[
                'truncate rounded px-2 py-1.5 text-left text-sm transition-colors duration-150 ease-snap',
                'focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white',
                visible ? 'animate-rise-in' : 'opacity-0',
                selected ? 'bg-white text-black' : 'text-zinc-300 hover:bg-white/[0.06]',
              ].join(' ')}
            >
              {entry.written.trim() || 'Untitled'}
            </button>
          )
        })}
      </div>
    </div>
  )
}
