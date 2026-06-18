import { useState } from 'react'
import { EntryList } from './EntryList'
import { EntryEditor } from './EntryEditor'
import { SEED_ENTRIES, DICTIONARY_PATH } from './types'
import type { DictionaryEntryData } from './types'

interface DictionaryManagerProps {
  className?: string
}

export function DictionaryManager({ className = '' }: DictionaryManagerProps) {
  const [entries, setEntries] = useState<DictionaryEntryData[]>(SEED_ENTRIES)
  const [selectedId, setSelectedId] = useState<string | null>(SEED_ENTRIES[0]?.id ?? null)
  const [dirty, setDirty] = useState(false)
  const [status, setStatus] = useState(`Loaded ${DICTIONARY_PATH}`)
  const [closed, setClosed] = useState(false)

  const selected = entries.find((e) => e.id === selectedId) ?? null

  function patchEntry(patch: Partial<DictionaryEntryData>) {
    if (!selected) return
    setEntries((prev) => prev.map((e) => (e.id === selected.id ? { ...e, ...patch } : e)))
    setDirty(true)
  }

  function addEntry() {
    const id = `entry-${Date.now()}`
    setEntries((prev) => [
      ...prev,
      { id, written: '', aliases: '', phonetic: true, highPriority: true, language: 'any' },
    ])
    setSelectedId(id)
    setDirty(true)
  }

  function deleteEntry() {
    if (!selected) return
    const idx = entries.findIndex((e) => e.id === selected.id)
    const next = entries.filter((e) => e.id !== selected.id)
    setEntries(next)
    setSelectedId(next.length ? next[Math.min(idx, next.length - 1)].id : null)
    setDirty(true)
  }

  function save() {
    setDirty(false)
    setStatus('Saved dictionary')
  }

  function reload() {
    setEntries(SEED_ENTRIES)
    setSelectedId(SEED_ENTRIES[0]?.id ?? null)
    setDirty(false)
    setStatus(`Loaded ${DICTIONARY_PATH}`)
  }

  if (closed) {
    return (
      <div
        className={`flex items-center justify-between rounded-lg border border-line bg-surface-panel px-5 py-4 animate-fade-in ${className}`}
      >
        <span className="text-sm text-muted">Dictionary manager closed.</span>
        <button
          type="button"
          onClick={() => setClosed(false)}
          className="rounded px-2 py-1 text-sm text-zinc-200 transition-colors duration-150 ease-snap hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record"
        >
          Reopen
        </button>
      </div>
    )
  }

  return (
    <div
      className={`animate-fade-in rounded-lg border border-line bg-surface-panel p-5 shadow-2xl shadow-black/60 ${className}`}
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h3 className="font-display text-xl text-zinc-50">Dictionary</h3>
        <div className="flex items-center gap-4 font-mono text-xs uppercase tracking-wide">
          <button
            type="button"
            disabled={!dirty}
            onClick={save}
            className="text-zinc-300 transition-colors duration-150 ease-snap hover:text-white disabled:text-zinc-600 disabled:hover:text-zinc-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record"
          >
            Save
          </button>
          <button
            type="button"
            onClick={reload}
            className="text-zinc-300 transition-colors duration-150 ease-snap hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record"
          >
            Reload
          </button>
          <button
            type="button"
            onClick={() => setClosed(true)}
            className="text-zinc-300 transition-colors duration-150 ease-snap hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-record"
          >
            Close
          </button>
        </div>
      </div>
      <p className="mt-1 truncate font-mono text-xs text-muted">Custom entries saved in {DICTIONARY_PATH}</p>

      <div className="my-4 border-t border-line" />

      <div className="flex flex-col gap-4 sm:flex-row sm:gap-0">
        <EntryList entries={entries} selectedId={selectedId} onSelect={setSelectedId} onAdd={addEntry} />
        <EntryEditor entry={selected} onChange={patchEntry} onDelete={deleteEntry} />
      </div>

      <div className="my-4 border-t border-line" />

      <div className="flex items-center gap-3 text-sm">
        {dirty ? (
          <span className="text-accent-warn">Unsaved changes</span>
        ) : (
          <span className="text-zinc-300">Saved</span>
        )}
        <span className="truncate text-muted">{status}</span>
      </div>
    </div>
  )
}
