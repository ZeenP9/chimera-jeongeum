/**
 * 正音 Mood box: the room after the voice. Self-contained Web Audio, with no tie to the engine: the host puts
 * it between its master gain and its outputs with `attach` and forwards the Mood knobs to `setMix` and
 * `setShape`. See src/mood/README.md for what the host needs to do on a new fork.
 */

/** Dry and wet gains for a room amount: the dry voice gives way a little as the room comes in. */
function roomGains(v: number): { dry: number; wet: number } {
  return { dry: 1 - 0.3 * v, wet: v };
}

/** A value that grows evenly in ratio from `a` (at 0) to `b` (at 1). */
const span = (a: number, b: number, u: number) =>
  a * Math.pow(b / a, Math.min(1, Math.max(0, u)));

/**
 * The room's decay time (s, 60 dB) for a size and a liveness, both 0 to 1. A room's decay goes with its
 * volume over its absorption, so each size has its own range: a small room 0.15 to 0.6 s, a large hall
 * 0.8 to 3 s, from heavily furnished (0) to bare stone (1).
 */
export function roomDecaySeconds(size: number, decay: number): number {
  return span(span(0.15, 0.8, size), span(0.6, 3, size), decay);
}

/**
 * A room's impulse response, made here (no file to fetch). `size` (0 small room, 1 large hall) sets how far
 * the walls are: the gap before the first reflection (8 to 50 ms), the spread of the six early
 * reflections (35 to 120 ms) and how slowly the diffuse tail builds. `decay` sets how live the room is
 * for that size (`roomDecaySeconds`). The tail is noise that loses its highs as it goes.
 * Each channel has unit energy, so the wet path is about as loud as the dry one before its gain.
 * The noise is seeded: the same settings give the same room on every load.
 */
export function roomImpulse(
  ctx: BaseAudioContext,
  size: number,
  decay: number,
): AudioBuffer {
  const fs = ctx.sampleRate;
  const rt60 = roomDecaySeconds(size, decay);
  const preS = span(0.008, 0.05, size),
    early = span(0.035, 0.12, size),
    build = span(0.02, 0.08, size);
  const n = Math.round(fs * (rt60 * 1.3 + preS + early + 0.03));
  const buf = ctx.createBuffer(2, n, fs);
  let seed = 0x5eed1234;
  const rand = () => {
    seed ^= seed << 13;
    seed ^= seed >>> 17;
    seed ^= seed << 5;
    return (seed >>> 0) / 4294967296;
  };
  const gauss = () =>
    Math.sqrt(-2 * Math.log(rand() + 1e-12)) * Math.cos(2 * Math.PI * rand());
  const aLow = 1 - Math.exp((-2 * Math.PI * 3500) / fs);
  const aHigh = Math.exp((-2 * Math.PI * 150) / fs);
  const pre = Math.round(fs * preS);
  for (let ch = 0; ch < 2; ch++) {
    const h = buf.getChannelData(ch);
    for (let k = 0; k < 6; k++)
      h[Math.round(fs * (preS * 0.5 + (early - preS * 0.5) * rand()))] +=
        (0.25 + 0.35 * rand()) * (rand() < 0.5 ? -1 : 1);
    let low = 0;
    for (let i = 0; i + pre < n; i++) {
      const t = i / fs;
      const x = gauss() * Math.pow(10, (-3 * t) / rt60);
      low += aLow * (x - low);
      const dark = Math.min(1, t / (rt60 * 0.5));
      h[i + pre] +=
        0.12 * Math.min(1, t / build) * ((1 - dark) * x + dark * low);
    }
    // no rumble below the voice
    let xPrev = 0,
      y = 0,
      energy = 0;
    for (let i = 0; i < n; i++) {
      y = aHigh * (y + h[i] - xPrev);
      xPrev = h[i];
      h[i] = y;
      energy += y * y;
    }
    const g = 1 / Math.sqrt(energy);
    for (let i = 0; i < n; i++) h[i] *= g;
  }
  return buf;
}

/** The room as a Web Audio insert: dry voice plus the voice through the room's impulse response. */
export class Room {
  /** how much of the room is mixed in (0 = the dry voice, as the engine makes it) */
  private mix = 0;
  /** the room's size and liveness (0 to 1 each) */
  private size = 0;
  private decay = 0.5;
  private ctx: BaseAudioContext | null = null;
  private dry: GainNode | null = null;
  private wet: GainNode | null = null;
  private conv: ConvolverNode | null = null;
  private timer: ReturnType<typeof setTimeout> | null = null;

  /** Build the room after `input` in `ctx` and return its output (connect that where `input` went). */
  attach(ctx: BaseAudioContext, input: AudioNode): AudioNode {
    const out = ctx.createGain();
    const dry = ctx.createGain();
    const wet = ctx.createGain();
    const conv = ctx.createConvolver();
    conv.normalize = false;
    conv.buffer = roomImpulse(ctx, this.size, this.decay);
    input.connect(dry).connect(out);
    input.connect(conv).connect(wet).connect(out);
    const g = roomGains(this.mix);
    dry.gain.value = g.dry;
    wet.gain.value = g.wet;
    Object.assign(this, { ctx, dry, wet, conv });
    return out;
  }

  /** How much of the room is mixed into the output: 0 (dry) to 1. */
  setMix(v: number): void {
    this.mix = Math.min(1, Math.max(0, v));
    if (!this.ctx || !this.dry || !this.wet) return;
    const g = roomGains(this.mix),
      t = this.ctx.currentTime;
    this.dry.gain.setTargetAtTime(g.dry, t, 0.02);
    this.wet.gain.setTargetAtTime(g.wet, t, 0.02);
  }

  /**
   * The room's size and how live it is (0 to 1 each). A new impulse response is made a moment after the
   * knobs stop moving, and swapped in with the wet path dipped so the change doesn't click.
   */
  setShape(size: number, decay: number): void {
    const s = Math.min(1, Math.max(0, size)),
      d = Math.min(1, Math.max(0, decay));
    if (s === this.size && d === this.decay) return;
    this.size = s;
    this.decay = d;
    if (!this.ctx || !this.conv || !this.wet) return;
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      const ctx = this.ctx,
        conv = this.conv,
        wet = this.wet;
      if (!ctx || !conv || !wet) return;
      wet.gain.setTargetAtTime(0, ctx.currentTime, 0.01);
      setTimeout(() => {
        conv.buffer = roomImpulse(ctx, this.size, this.decay);
        wet.gain.setTargetAtTime(
          roomGains(this.mix).wet,
          ctx.currentTime,
          0.02,
        );
      }, 60);
    }, 150);
  }
}
