/**
 * 正音 Mood box fitted to this CHIMERA fork. The only file of src/mood/ that knows the host: on a new fork,
 * rewrite this against the new engine (src/mood/README.md).
 */
import { engine } from "../audio/AudioEngine.ts";
import { inPlugin } from "../audio/pluginBridge.ts";
import { setJeongeumMarks } from "../wasm/engine.ts";
import { setMoodHost } from "./host.ts";

setMoodHost({
  inPlugin,
  setRoom: (mix) => engine.setRoom(mix),
  setRoomShape: (size, decay) => engine.setRoomShape(size, decay),
  setReadingMarks: (bits) => setJeongeumMarks(bits),
});

export { resendMood } from "./store.ts";
