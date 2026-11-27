import { V } from "../colors.ts";
import { useStore } from "../store.ts";

const R = 30;
const D = R * Math.SQRT1_2;
type Key = "i" | "arae" | "eu" | "hieut";

/** The four slices of the circle cut by an X: left, top, right, bottom. */
const SLICES: { key: Key; name: string; d: string }[] = [
  { key: "i", name: "ㅣ as ɨ (Polish, Russian)", d: `M0 0L${-D} ${-D}A${R} ${R} 0 0 0 ${-D} ${D}Z` },
  { key: "arae", name: "ㆍ (no function yet)", d: `M0 0L${-D} ${-D}A${R} ${R} 0 0 1 ${D} ${-D}Z` },
  { key: "eu", name: "ㅡ → ḥ : Sanskrit · Hindi reading", d: `M0 0L${D} ${-D}A${R} ${R} 0 0 1 ${D} ${D}Z` },
  { key: "hieut", name: "ㆆ ʕ (no function yet)", d: `M0 0L${D} ${D}A${R} ${R} 0 0 1 ${-D} ${D}Z` },
];

/**
 * The signs, drawn here as strokes around (0, 0) so they look the same whatever font the page has:
 * thin, even, round-ended lines, like the lit legends on the glass of a 1970s tuner. `dots` are filled.
 */
const SIGN: Record<string, { d: string; dots?: [number, number, number][] }> = {
  "ㅣ": { d: "M0 -5.4V5.4" },
  "ㅡ": { d: "M-5.4 0H5.4" },
  // ㆆ 은 제 꼴대로: 가로획 바로 아래에 동그라미 (사이에 세로획을 넣지 않는다)
  "ㆆ": { d: "M-4.6 -4.4H4.6M3.6 1.6A3.6 3.6 0 1 1 -3.6 1.6A3.6 3.6 0 1 1 3.6 1.6Z" },
  "ɨ": { d: "M0 -1.8V5.6M-2.8 1H2.8", dots: [[0, -4.7, 0.95]] },
  "ḥ": { d: "M-2.8 -5.6V3.4M-2.8 0.2C-2.8 -3.2 2.8 -3.2 2.8 0.2V3.4", dots: [[0, 5.9, 0.95]] },
  "ʕ": { d: "M3 -3.2C1.8 -6 -3.2 -5.6 -3.2 -2.6C-3.2 0.2 0 0 0 2.4V5.6" },
};

/** One sign at (x, y), scaled and turned (degrees clockwise); its stroke keeps its weight when scaled. */
function Sign(props: { ch: string; x: number; y: number; scale?: number; turn?: number; className: string }) {
  const s = SIGN[props.ch], k = props.scale ?? 1;
  return (
    <g transform={`translate(${props.x} ${props.y}) rotate(${props.turn ?? 0}) scale(${k})`} className={props.className}>
      <path d={s.d} className="jm-stroke" strokeWidth={1.25 / k} />
      {s.dots?.map(([cx, cy, r], i) => <circle key={i} cx={cx} cy={cy} r={r} className="jm-fill" />)}
    </g>
  );
}

/**
 * 正音: 네 조각 스위치 (사용자 지정 2026-11-26). X 로 가른 원의 네 조각이 저마다 켜지고 꺼진다:
 *   왼쪽 ɨ ㅣ : ᆜ 를 [ɨ] 로 (폴란드어·러시아어).
 *   오른쪽 ㅡ ḥ : 산스크리트·힌디어 읽기 (jeongeum.rs `MARK_EU`).
 *   위 ⊙ (○ 안의 ㆍ) · 아래 ㆆ ʕ : 자리만 잡아 두었다 (켜지고 꺼지기만 한다; 기능은 미정)
 * 정음 글자 ㅣ ㅡ ㆆ (호박색)와 ㆍ (빨강)은 늘 보이고, IPA 쪽 ɨ ○ ḥ ʕ 는 켜면 희게 밝아지고 끄면 어두워진다.
 * 꺼져 있을 때는 정음 글자만 십자로 가지런히 보이고, 켜면 그 바깥의 IPA 기호에 불이 들어온다 (ʕ 만 시계 방향으로
 * 90° 눕힌다). 글자는 글꼴이 아니라 가늘고 고른 획으로 직접 그린다: 1970년대 튜너 유리판의 불 들어온 글씨처럼.
 * 테두리는 Ring 노브처럼 흰색.
 */
export function JeongeumSwitch() {
  const marks = useStore((s) => s.marks);
  const toggle = useStore((s) => s.toggleMark);
  const on = (k: Key) => (marks[k] ? " on" : "");
  return (
    <div className="encoder jm-switch" style={{ "--slot": V.white } as React.CSSProperties}>
      <span className="enc-label">正音</span>
      <div className="jm-dial">
        <svg viewBox="-42 -42 84 84" role="group" aria-label="正音 readings">
          {SLICES.map((s) => (
            <path
              key={s.key}
              d={s.d}
              className={`jm-slice${on(s.key)}`}
              role="switch"
              aria-checked={marks[s.key]}
              aria-label={s.name}
              tabIndex={0}
              onClick={() => toggle(s.key)}
              onKeyDown={(e) => {
                if (e.key === " " || e.key === "Enter") {
                  e.preventDefault();
                  toggle(s.key);
                }
              }}
            />
          ))}
          <circle r={R} className="jm-ring" />
          <line x1={-D} y1={-D} x2={D} y2={D} className="jm-cut" />
          <line x1={-D} y1={D} x2={D} y2={-D} className="jm-cut" />
          <g aria-hidden="true" className="jm-signs">
            {/* 정음 글자는 가운데 둘레에 십자로 가지런히, IPA 기호는 그 바깥쪽 같은 줄에. ʕ 만 시계 방향으로 90° 눕힌다 */}
            {/* the glow is on the group: a lone straight stroke has an empty box, and a filter on it alone hides it */}
            <g className="jm-jamo">
              <Sign ch="ㅣ" x={-12.5} y={0} className="" />
              <Sign ch="ㅡ" x={12} y={0} scale={0.85} className="" />
              <Sign ch="ㆆ" x={0} y={12.5} scale={0.9} className="" />
            </g>
            <circle cx={0} cy={-16} r={1.2} className="jm-dot" />
            <Sign ch="ɨ" x={-23.5} y={0} className={`jm-ipa${on("i")}`} />
            <circle cx={0} cy={-16} r={6} className={`jm-o${on("arae")}`} />
            <Sign ch="ḥ" x={23.5} y={-0.4} className={`jm-ipa${on("eu")}`} />
            <Sign ch="ʕ" x={0.6} y={23.6} turn={90} className={`jm-ipa${on("hieut")}`} />
          </g>
        </svg>
      </div>
      <span className="enc-value" aria-hidden="true">
        {"\u00a0"}
      </span>
      <span className="enc-unit" aria-hidden="true">
        {"\u00a0"}
      </span>
    </div>
  );
}
