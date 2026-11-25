/**
 * Sing's tunes, named for how they sound: melodies anyone's words are fitted to (fit.ts). All are
 * written for this instrument except Twinkle (traditional). Each phrase is one line of lyrics; a song longer than the tune goes round again.
 * Notes are [midi, beats], at the female voice's pitch; `octave` shifts the other bodies, as the
 * demo songs do. Phrases end on a long note: the breath before the next line comes out of it.
 * After the moods come the engine's other demo songs (contributed ones among them), by name and
 * with their own words.
 */
import { DEMO_SONGS } from "../wasm/engine.ts";

export interface Tune {
  id: string;
  name: string;
  bpm: number;
  octave: { male: number; female: number; child: number };
  phrases: [number, number][][];
  /** a demo song's own words: choosing it puts them in the lyrics, and Sing sings them a syllable to a note */
  lyrics?: string;
  /**
   * 正音: 읽기용 곡. 가사를 악구에 끼워 맞추지 않고 음절마다 같은 길이의 음 하나를 준다 (fit.ts `readSong`):
   * 줄 안에서 `fall` 반음을 서서히 내려가고 줄 끝 음절은 `end` 반음 더 내려가며, 비브라토는 없다.
   * `phrases` 의 첫 음이 읽는 높이.
   */
  read?: { fall: number; end: number };
}

/** the demo songs the moods take their melodies from */
const borrowed = new Set<string>();

/** a demo song's melody (the engine's catalog, shared with `chimera render song`) cut into phrases of `sizes` notes */
function fromDemo(id: string, name: string, demo: string, sizes: number[]): Tune {
  const d = DEMO_SONGS.find((s) => s.name === demo)!;
  borrowed.add(demo);
  let at = 0;
  const phrases = sizes.map((n) => d.notes.slice(at, (at += n)).map(([m, b]): [number, number] => [m, b]));
  return { id, name, bpm: d.bpm, octave: d.octave, phrases };
}

/** 읽기용 곡의 한 줄 (가사가 없을 때 "la" 로 읽는 길이): 라3, 음절마다 한 박, 줄 끝은 길게 */
const READ_LINE: [number, number][] = [...Array.from({ length: 7 }, (): [number, number] => [57, 1]), [57, 2.2]];

const MOODS: Tune[] = [
  // "Made of air", the instrument's own: minor, unhurried
  fromDemo("dreamy", "Dreamy", "Made of air", [9, 10]),
  {
    // major, bright, climbing
    id: "joyful",
    name: "Joyful",
    bpm: 104,
    octave: { male: -1, female: 0, child: 0 },
    phrases: [
      [[64, 1], [64, 1], [67, 1], [69, 1], [67, 1.5], [64, 0.5], [62, 2]],
      [[60, 1], [62, 1], [64, 1], [67, 1], [64, 1.5], [62, 0.5], [60, 2]],
      [[69, 1], [69, 1], [72, 1], [69, 1], [67, 1.5], [64, 0.5], [67, 2]],
      [[65, 1], [64, 1], [62, 1], [64, 1], [62, 1], [59, 1], [60, 2]],
    ],
  },
  {
    // minor, slow, falling back home
    id: "somber",
    name: "Somber",
    bpm: 76,
    octave: { male: -1, female: 0, child: 0 },
    phrases: [
      [[57, 1], [60, 1], [64, 1.5], [62, 0.5], [60, 1], [59, 1], [57, 2]],
      [[57, 1], [60, 1], [64, 1], [67, 1], [65, 1.5], [64, 0.5], [64, 2]],
      [[65, 1], [64, 1], [62, 1], [60, 1], [62, 1.5], [60, 0.5], [59, 2]],
      [[57, 1], [59, 1], [60, 1], [62, 1], [64, 1], [59, 1], [57, 2]],
    ],
  },
  {
    // quick, short notes, room for many words
    id: "playful",
    name: "Playful",
    bpm: 120,
    octave: { male: -1, female: 0, child: 0 },
    phrases: [
      [[67, 0.5], [67, 0.5], [64, 0.5], [67, 0.5], [69, 1], [67, 1], [64, 0.5], [62, 0.5], [60, 1], [62, 2]],
      [[60, 0.5], [60, 0.5], [62, 0.5], [64, 0.5], [67, 1], [64, 1], [62, 0.5], [60, 0.5], [62, 1], [64, 2]],
      [[67, 0.5], [67, 0.5], [69, 0.5], [71, 0.5], [72, 1], [71, 1], [69, 0.5], [67, 0.5], [64, 1], [67, 2]],
      [[69, 0.5], [67, 0.5], [64, 0.5], [62, 0.5], [60, 1], [62, 1], [64, 0.5], [62, 0.5], [60, 1], [60, 2]],
    ],
  },
  {
    // a slow waltz, long notes
    id: "gentle",
    name: "Gentle",
    bpm: 96,
    octave: { male: -1, female: 0, child: 0 },
    phrases: [
      [[65, 2], [69, 1], [72, 2], [69, 1], [67, 2], [65, 1], [67, 3]],
      [[65, 2], [69, 1], [72, 2], [74, 1], [72, 2], [69, 1], [67, 3]],
      [[70, 2], [69, 1], [67, 2], [65, 1], [64, 2], [65, 1], [67, 3]],
      [[65, 2], [69, 1], [67, 2], [64, 1], [62, 2], [64, 1], [65, 3]],
    ],
  },
  fromDemo("twinkle", "Twinkle, twinkle", "Twinkle, twinkle", [7, 7]),
  // 正音: 문장 읽기. 한 음으로 평평하게 / 줄 안에서 조금씩 내려가고 끝을 내리는 말투
  { id: "read-flat", name: "Reading (flat)", bpm: 280, octave: { male: -1, female: 0, child: 0 }, phrases: [READ_LINE], read: { fall: 0, end: 0 } },
  { id: "read-fall", name: "Reading (falling)", bpm: 280, octave: { male: -1, female: 0, child: 0 }, phrases: [READ_LINE], read: { fall: 2, end: 2 } },
];

/** every other demo song as written: one phrase, its own words (a song added to the engine's catalog shows up here) */
const SONGS: Tune[] = DEMO_SONGS.filter((d) => !borrowed.has(d.name)).map((d) => ({
  id: `song:${d.name}`,
  name: d.name,
  bpm: d.bpm,
  octave: d.octave,
  phrases: [d.notes.map(([m, b]): [number, number] => [m, b])],
  lyrics: d.lyrics,
}));

export const TUNES: readonly Tune[] = [...MOODS, ...SONGS];
