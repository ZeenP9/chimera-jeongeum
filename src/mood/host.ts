/**
 * 正音 Mood box: what it needs from the CHIMERA it is fitted to. The box never imports the host's engine or
 * store; the host hands it these with `setMoodHost` (in this fork: src/mood/chimera.ts). On a new fork, write
 * that one file against the new CHIMERA and the rest of src/mood/ moves over unchanged (src/mood/README.md).
 */
export interface MoodHost {
  /** true inside the CLAP/VST3/AU plugin: the DAW adds its own room, so the Mood page is not shown */
  inPlugin: boolean;
  /** the room after the voice: how much is mixed in (0 to 1) */
  setRoom(mix: number): void;
  /** the room's size and liveness (0 to 1 each) */
  setRoomShape(size: number, decay: number): void;
  /** the 正音 reader's reading switches, as `READ_*` bits (src/mood/store.ts), from the next text read */
  setReadingMarks(bits: number): void;
}

const NONE: MoodHost = {
  inPlugin: false,
  setRoom() {},
  setRoomShape() {},
  setReadingMarks() {},
};
let host: MoodHost = NONE;

/** Fit the box to a host. Call once, before the first sound. */
export function setMoodHost(h: MoodHost): void {
  host = h;
}

export function moodHost(): MoodHost {
  return host;
}
