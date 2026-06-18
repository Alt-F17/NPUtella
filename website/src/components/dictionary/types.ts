export type DictionaryLanguage = 'any' | 'english' | 'french'

export interface DictionaryEntryData {
  id: string
  written: string
  aliases: string
  phonetic: boolean
  highPriority: boolean
  language: DictionaryLanguage
}

export const SEED_ENTRIES: DictionaryEntryData[] = [
  {
    id: 'nputella',
    written: 'NPUtella',
    aliases: 'n p u tella, npu tella',
    phonetic: true,
    highPriority: true,
    language: 'any',
  },
  {
    id: 'nixos',
    written: 'NixOS',
    aliases: 'nix os, nix o s, nicsos, nicks os',
    phonetic: true,
    highPriority: true,
    language: 'any',
  },
  {
    id: 'qualcomm',
    written: 'Qualcomm',
    aliases: 'qualcom, qual com, kwall com',
    phonetic: true,
    highPriority: false,
    language: 'any',
  },
]

export const DICTIONARY_PATH = String.raw`C:\Users\felix\AppData\Roaming\NPUtella\dictionary.toml`
