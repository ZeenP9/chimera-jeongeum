import { create } from "zustand";
import { moodHost } from "./host.ts";

/** The four slices of the 正音 switch: left ɨㅣ, top ⊙, right ㅡḥ, bottom ㆆع. */
export type Slice = "i" | "arae" | "eu" | "hieut";
export type Marks = Record<Slice, boolean>;

/** Reading bits the 正音 reader understands (crates/chimera/src/phonetics/jeongeum.rs `MARK_*`). */
export const READ_I = 1; // ᆜ as [ɨ]: Polish, Russian
export const READ_EU = 2; // Sanskrit, Hindi

/**
 * What the switch asks of the reader. All off is the default (neutral) reading. Left + right together is a
 * mode of its own (French written in 正音), not the two readings at once; its rules are not set yet, so it
 * reads as neutral. The top and bottom slices send nothing yet (see `route`).
 */
export function readingBits(m: Marks): number {
  if (m.i && m.eu) return 0;
  return (m.i ? READ_I : 0) | (m.eu ? READ_EU : 0);
}

/**
 * The path text takes to the voice (the top slice, 사용자 지정 2026-11-28):
 *   "ipa"      (off, the default): the input's own front end makes IPA, the engine sings that, and 正音 only
 *              shows it on the canvas (the back dancer);
 *   "jeongeum" (on): the input is first written in 正音, and that 正音 is read to IPA for the engine.
 * Not wired yet: the host reads this when it routes lyrics (src/mood/README.md).
 */
export function route(m: Marks): "ipa" | "jeongeum" {
  return m.arae ? "jeongeum" : "ipa";
}

interface MoodState {
  /** how much of the room is mixed in (0 = dry, as the engine makes it) */
  room: number;
  /** the room's size (0 small room, 1 large hall) and how live it is for that size (0 to 1) */
  roomSize: number;
  roomDecay: number;
  marks: Marks;
  setRoom: (v: number) => void;
  setRoomShape: (shape: { roomSize?: number; roomDecay?: number }) => void;
  toggleMark: (k: Slice) => void;
}

export const useMood = create<MoodState>((set, get) => ({
  // 19 %: the room 14 dB under the voice
  room: 0.19,
  // a small room, 0.30 s
  roomSize: 0,
  roomDecay: 0.5,
  marks: { i: false, arae: false, eu: false, hieut: false },
  setRoom: (v) => {
    set({ room: v });
    moodHost().setRoom(v);
  },
  setRoomShape: (shape) => {
    set(shape);
    const st = get();
    moodHost().setRoomShape(st.roomSize, st.roomDecay);
  },
  toggleMark: (k) => {
    set((s) => ({ marks: { ...s.marks, [k]: !s.marks[k] } }));
    moodHost().setReadingMarks(readingBits(get().marks));
  },
}));

/** Send everything the box holds to the host again: call it whenever the host has a new engine. */
export function resendMood(): void {
  const s = useMood.getState(),
    h = moodHost();
  h.setRoom(s.room);
  h.setRoomShape(s.roomSize, s.roomDecay);
  h.setReadingMarks(readingBits(s.marks));
}
