//! 正音(훈민정음) front end — local prototype for 《正音으로 듣고쓰다》's notation.
//!
//! 正音 is a phonetic notation and a superset of Hangul: a word is written as it sounds, so this
//! front end reads every Hangul word literally (국물 = [국.물]; spoken smoothly it is written 궁물).
//! Turning morphological spelling (먹어) into sound (머거) belongs to a separate Hangul→正音 converter,
//! not here. Old letters (ㆍ ㆆ ㆁ ㅿ ᄫ ᄛ …) and the combined vowels (ᆜ ᆕ ힼ ᆐ ᆛ ᆟ ᆗ ힹ ힺ ힻ) come in as
//! conjoining jamo; a compatibility ㆆ ㅿ ㆁ ㅱ right after a syllable is read as its coda (간ㆆ = 간ᇹ),
//! since IMEs cannot type those codas.
//!
//! Rules (as decided with the notation's author, 2026-11):
//! - The vowel after a sonorant onset (ㄴ ㄹ ㅁ ㆁ, and no onset at all) sits one step lower:
//!   ㅡ ɯ → ɤ, ㅓ ɤ̞ → ʌ, ㆍ ə → ɑ. ㆆ is the glottal stop [ʔ] and is not a sonorant.
//! - ㅔ e̞, ㅚ ø̞, ㅗ o̞ (true-mid vowels); ᆜ ɿ (apical, z c s); ㅢ after ㅈ ㅊ ㅅ is ʅ and turns the
//!   onset retroflex (즤 [tʂʅ]); a coda ㅿ there only confirms it. ᅟᅥᇫ is ɝ. ᅟᆕ ʉ, ᅟힼ ɵ.
//! - Vowel values: the Seeing Speech cardinal vowels (Janet Beck) measured and calibrated as the
//!   back keys ɯ̠ ɤ̠ ʌ̠ ɑ̠ and the midpoints ɤ̞ e̞ ø̞ o̞ (vowel_data.rs).
//!
//! - Consonants: see `onset`, `coda` and the rules in `syllable` / `word_syllables` (palatals by
//!   C+j, uvulars by ㄲ ㄱ ㅋ ㅎ + ㅡ-compound, ᄒᆞᇹ ħ / ᅙᆞᇂ ʕ, coda ㅿ marks, ㄹ|ㅎ ʀ, ㅎ|ㄹ ʁ).
//!
//! Not here: the tone layer of the Mandarin notation (a 1st-tone syllable written as two blocks, ㆁ as
//! the rising tone, ㆆ falling in the 3rd tone) — pinyin goes through jeongeum_pinyin.py /
//! jeongeum_sing.py, which set pitch per syllable. Every block is one sung syllable here.

use super::mandarin::{Mandarin, MandarinLexicon, is_han};
use super::syllables::FrontEnd;
use super::{Bend, Edge, Segment, Syllable};
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct Jeongeum;

const BASE: u32 = 0xAC00;

fn is_block(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c)
}
fn is_conjoining(c: char) -> bool {
    matches!(c as u32, 0x1100..=0x11FF | 0xA960..=0xA97F | 0xD7B0..=0xD7FF)
}
fn is_lead(c: char) -> bool {
    matches!(c as u32, 0x1100..=0x115F | 0xA960..=0xA97F)
}
fn is_vowel_jamo(c: char) -> bool {
    matches!(c as u32, 0x1160..=0x11A7 | 0xD7B0..=0xD7C6)
}
fn is_tail(c: char) -> bool {
    matches!(c as u32, 0x11A8..=0x11FF | 0xD7CB..=0xD7FB)
}

/// Every syllable block as its jamo (L V T), the rest as they are.
fn jamo(word: &str) -> Vec<char> {
    let mut out = Vec::new();
    for c in word.chars() {
        if is_block(c) {
            let i = c as u32 - BASE;
            out.push(char::from_u32(0x1100 + i / 588).unwrap());
            out.push(char::from_u32(0x1161 + (i % 588) / 28).unwrap());
            if i % 28 != 0 {
                out.push(char::from_u32(0x11A7 + i % 28).unwrap());
            }
        } else if let Some(t) = compat_tail(c).filter(|_| out.last().is_some_and(|&p| is_vowel_jamo(p) || is_tail(p))) {
            // 입력기로 칠 수 없는 받침의 대체 입력: 음절 바로 뒤의 낱자 ㆆ ㅿ ㆁ ㅱ 은 받침으로 읽는다 (간ㆆ = 간ᇹ)
            out.push(t);
        } else {
            out.push(c);
        }
    }
    out
}

/// 낱자(호환용 자모) → 받침 자모: 입력기에서 옛 받침을 칠 수 없을 때 음절 뒤에 붙여 쓰는 대체 입력.
fn compat_tail(c: char) -> Option<char> {
    match c {
        'ㆆ' => Some('ᇹ'),
        'ㅿ' => Some('ᇫ'),
        'ㆁ' => Some('ᇰ'),
        'ㅱ' => Some('ᇢ'),
        _ => None,
    }
}

/// One written syllable: onset jamo (None: ㅇ or the filler), vowel jamo, tail jamo.
struct Written {
    text: String,
    lead: Option<char>,
    vowel: Vec<char>,
    tail: Vec<char>,
}

fn split(word: &str) -> Vec<Written> {
    let j = jamo(word);
    let mut out: Vec<Written> = Vec::new();
    let mut i = 0;
    while i < j.len() {
        let start = i;
        let mut lead = None;
        while i < j.len() && is_lead(j[i]) {
            if j[i] != '\u{110B}' && j[i] != '\u{115F}' {
                lead = Some(j[i]);
            }
            i += 1;
        }
        let mut vowel = Vec::new();
        while i < j.len() && is_vowel_jamo(j[i]) {
            if j[i] != '\u{1160}' {
                vowel.push(j[i]);
            }
            i += 1;
        }
        let mut tail = Vec::new();
        while i < j.len() && is_tail(j[i]) {
            tail.push(j[i]);
            i += 1;
        }
        if i == start {
            i += 1; // not a jamo: skip it
            continue;
        }
        let text: String = j[start..i].iter().collect();
        out.push(Written { text, lead, vowel, tail });
    }
    out
}

/// Onset → (IPA, aspirated, VOT). Plain ㄱ ㄷ ㅂ ㅈ aspirate briefly (Korean lax), the doubled ones
/// not at all (Mandarin b d g are written ㅃ ㄸ ㄲ), ㅋ ㅌ ㅍ ㅊ long.
fn onset(c: char) -> Option<(&'static str, Option<bool>, Option<f64>)> {
    const LAX: (Option<bool>, Option<f64>) = (Some(true), Some(0.03));
    const TENSE: (Option<bool>, Option<f64>) = (Some(false), None);
    const ASP: (Option<bool>, Option<f64>) = (Some(true), Some(0.08));
    let (ipa, (a, v)) = match c {
        'ᄀ' => ("k", LAX),
        'ᄁ' => ("k", TENSE),
        'ᄏ' => ("k", ASP),
        'ᄃ' => ("t", LAX),
        'ᄄ' => ("t", TENSE),
        'ᄐ' => ("t", ASP),
        'ᄇ' => ("p", LAX),
        'ᄈ' => ("p", TENSE),
        'ᄑ' => ("p", ASP),
        'ᄌ' => ("tɕ", LAX),
        'ᄍ' => ("tɕ", TENSE),
        'ᄎ' => ("tɕ", ASP), // ㅊ = tɕʰ
        'ᄉ' | 'ᄊ' => ("s", (None, None)),
        'ᄒ' => ("h", (None, None)),
        'ᄂ' => ("n", (None, None)),
        'ᄅ' => ("ɾ", (None, None)),
        'ᄆ' => ("m", (None, None)),
        'ᅌ' => ("ŋ", (None, None)),
        'ᅙ' => ("ʔ", (None, None)),
        'ᅀ' => ("z", (None, None)),
        'ᄫ' => ("β", (None, None)),
        'ᅗ' => ("f", (None, None)),
        'ᄛ' => ("r", (None, None)), // 가벼운 ㄹ: 초성 전동 r
        'ᄙ' => ("r", (None, None)), // ㄹㄹ: 전동 r (ɼ 기호는 엔진에 없어 치조 전동음 r)
        'ᄬ' => ("ʙ", (None, None)), // ㅹ: 양순 전동음
        'ᅘ' => ("x", (None, None)), // ㆅ: 연구개 마찰음 (j 앞에서는 ç: `syllable`)
        _ => return None,
    };
    Some((ipa, a, v))
}

/// The retroflex counterpart of ㅈ ㅊ ㅅ, before ㅢ (즤 츼 싀: zhi chi shi).
fn retroflex(ipa: &str) -> Option<&'static str> {
    match ipa {
        "tɕ" | "ts" => Some("tʂ"),
        "s" => Some("ʂ"),
        _ => None,
    }
}

fn sonorant(lead: Option<char>) -> bool {
    match lead {
        None => true, // no onset: 어 = ʌ, 으 = ɤ
        Some(c) => matches!(c, 'ᄂ' | 'ᄅ' | 'ᄆ' | 'ᅌ'),
    }
}

/// Vowel jamo → (glide, vowel). The rule-bound ones are resolved by `son`.
fn vowel(v: &[char], son: bool) -> Option<(&'static str, &'static str)> {
    let eo = if son { "ʌ̠" } else { "ɤ̞" };
    let eu = if son { "ɤ̠" } else { "ɯ̠" };
    let arae = if son { "ɑ̠" } else { "ə" };
    Some(match v {
        ['ᅡ'] => ("", "a"),
        ['ᅢ'] => ("", "ɛ"),
        ['ᅣ'] => ("j", "a"),
        ['ᅤ'] => ("j", "ɛ"),
        ['ᅥ'] => ("", eo),
        ['ᅦ'] => ("", "e̞"),
        ['ᅧ'] => ("j", eo),
        ['ᅨ'] => ("j", "e̞"),
        ['ᅩ'] => ("", "o̞"),
        ['ᅪ'] => ("w", "a"),
        ['ᅫ'] => ("w", "ɛ"),
        ['ᅬ'] => ("", "ø̞"),
        ['ᅭ'] => ("j", "o̞"),
        ['ᅮ'] => ("", "u"),
        ['ᅯ'] => ("w", eo),
        ['ᅰ'] => ("w", "e̞"),
        ['ᅱ'] => ("w", "i"),
        ['ᅲ'] => ("j", "u"),
        ['ᅳ'] => ("", eu),
        ['ᅴ'] => ("ɰ", "i"), // after a consonant: see `syllable`
        ['ᅵ'] => ("", "i"),
        ['ᆞ'] => ("", arae),
        ['ᆜ'] => ("", "ɿ"),   // ㅣ+ㅡ: the apical vowel of z c s
        ['ᆕ'] => ("", "ʉ"),   // ㅡ+ㅜ
        ['ힼ'] => ("", "ɵ"),   // ㅡ+ㅗ
        ['ᆐ'] => ("", "yɛ"),  // ㅠ+ㅔ (jue4 ᄌᆐ)
        ['ᆛ'] => ("", "y"),   // ㅣ+ㅜ: ü
        ['ᆟ'] => ("", "ɔ"),   // ㆍ+ㅓ: pinyin o, ou
        ['ᆗ'] => ("", "u̠"),  // ㅢ+ㅜ: after zh ch sh (ʊ)
        ['ힹ'] => ("", "ɨa"),  // ㅡ+ㅏ: the 3rd tone's glide
        ['ힺ'] => ("", "ɨə"),  // ㅡ+ㅓ
        ['ힻ'] => ("", "ɨe"),  // ㅡ+ㅔ
        _ => return None,
    })
}

fn coda(c: char) -> Option<&'static str> {
    Some(match c {
        'ᆨ' => "k",
        'ᆫ' => "n",
        'ᆮ' => "t",
        'ᆯ' => "l",
        'ᇙ' => "ɫ", // 종성 ㅭ (ㄹ+ㆆ, 以影補來): 연구개화 설측음
        'ᆷ' => "m",
        'ᆸ' => "p",
        'ᆼ' | 'ᇰ' => "ŋ",
        'ᆺ' => "s",
        'ᆩ' | 'ᆿ' => "k",
        'ᇀ' | 'ᆽ' | 'ᆾ' | 'ᆻ' => "t",
        'ᇁ' => "p",
        'ᇢ' => "ʋ", // 종성 ㅱ (가벼운 미음)
        'ᆭ' => "ɴ", // 종성 ㄶ: 목젖 비음
        'ᆶ' => "ʀ", // 종성 ㅀ: 목젖 전동음
        'ᇐ' => "r", // 종성 ㄹㄹ: 전동 r
        'ퟛ' => "r", // 종성 가벼운 ㄹ (ᅟᆞퟛ): 전동 r
        'ᇂ' => "h", // 종성 ㅎ: 무성 h. 모음 사이면 유성 ɦ (`word_syllables`)
        'ᇹ' => "ʔ", // 종성 ㆆ: 초성과 같이 성문 폐쇄 (ㄴ 뒤 겹받침 ㄴㆆ 는 ɲ: `syllable`)
        _ => return None,
    })
}

/// ㅡ 결합 모음 → ㅡ를 뺀 모음 (ힹ → ㅏ): 앞 자음이 ㅡ를 흡수하는 표기(목젖음 ᄁힹ, 치 마찰음 ᄃힹᇫ)에 쓴다.
fn eu_base(c: char) -> Option<char> {
    match c {
        'ힹ' => Some('ᅡ'),
        'ힺ' => Some('ᅥ'),
        'ힻ' => Some('ᅦ'),
        'ힼ' => Some('ᅩ'),
        'ᆕ' => Some('ᅮ'),
        'ᅴ' => Some('ᅵ'),
        _ => None,
    }
}

fn syllable(w: &Written) -> Option<Syllable> {
    // 모음 없이 종성만 있는 음절: 성절 자음 (ᅟᅠᆫ = 嗯 n, ᅟᅠᆼ = ng, ᅟᅠᇢ = 呣 m 을 ㅱ=[ʋ]로)
    if w.vowel.is_empty() && let Some((first, rest)) = w.tail.split_first() {
        let nucleus = coda(*first)?;
        let on: Vec<Segment> = w.lead.and_then(onset).map(|(ipa, a, v)| Segment { aspirated: a, vot: v, ..Segment::new(ipa) }).into_iter().collect();
        let cd = rest.iter().filter_map(|&t| coda(t)).map(Segment::new).collect();
        return Some(Syllable { text: w.text.clone(), onset: on, nucleus: Segment::new(nucleus), coda: cd, ..Default::default() });
    }
    // 인두 마찰음 표기 (주로 아랍어): ᄒᆞᇹ = ħ, ᅙᆞᇂ = ʕ. 중성 ㆍ는 자리표시가 아니라 이 표기의 구성요소이므로,
    // ᅙᆞ = ə 처럼 초성·ㆍ·종성 세 자를 한 단위로 읽고 따로 음가를 매기지 않는다.
    // 자음만인 음절이라 다음 음절 첫소리(또는 앞 음절 받침)로 붙는다: `word_syllables`
    let whole = match (w.lead, w.vowel.as_slice(), w.tail.as_slice()) {
        (Some('ᄒ'), ['ᆞ'], ['ᇹ']) => Some("ħ"),
        (Some('ᅙ'), ['ᆞ'], ['ᇂ']) => Some("ʕ"),
        _ => None,
    };
    if let Some(c) = whole {
        return Some(Syllable { text: w.text.clone(), onset: vec![Segment::new(c)], nucleus: Segment::new(""), ..Default::default() });
    }
    // ㄲ ㄱ ㅋ ㅎ + ㅡ 결합 모음 (ᄁힹ = [qa]): ㅡ는 자음에 흡수되어 목젖 자음 q ɢ qʰ χ 가 되고 나머지 모음이 남는다.
    //   ힹ→ㅏ ힺ→ㅓ ힻ→ㅔ ힼ→ㅗ ᆕ→ㅜ ㅢ→ㅣ. ㅡ 단독(그 끄)은 목젖 규칙을 받지 않는다 (그 = [kɯ]).
    if let Some(l @ ('ᄁ' | 'ᄀ' | 'ᄏ' | 'ᄒ')) = w.lead
        && let [first, rest @ ..] = w.vowel.as_slice()
        && let Some(base) = eu_base(*first)
    {
        let mut v = vec![base];
        v.extend_from_slice(rest);
        let (glide, nucleus) = vowel(&v, false)?;
        let uvular = match l {
            'ᄁ' => Segment { aspirated: Some(false), ..Segment::new("q") },
            'ᄀ' => Segment::new("ɢ"),
            'ᄏ' => Segment { aspirated: Some(true), vot: Some(0.08), ..Segment::new("q") },
            _ => Segment::new("χ"),
        };
        let mut on = vec![uvular];
        if !glide.is_empty() {
            on.push(Segment::new(glide));
        }
        let cd = w.tail.iter().filter_map(|&t| coda(t)).map(Segment::new).collect();
        return Some(Syllable { text: w.text.clone(), onset: on, nucleus: Segment::new(nucleus), coda: cd, ..Default::default() });
    }
    // ㄷ ㅌ + ㅡ 결합 모음 + 종성 ㅿ (ᄃힹᇫ = [ða]): ㅡ는 치 마찰음에 흡수되고 나머지 모음이 남는다.
    //   드ᇫ = ð 단독, 트ᇫ = θ 단독과 같은 원리. 종성 ㅿ는 표지이므로 소리 나지 않는다.
    if let Some(l @ ('ᄃ' | 'ᄐ')) = w.lead
        && w.tail.contains(&'ᇫ')
        && let [first, rest @ ..] = w.vowel.as_slice()
        && let Some(base) = eu_base(*first)
    {
        let mut v = vec![base];
        v.extend_from_slice(rest);
        let (glide, nucleus) = vowel(&v, false)?;
        let mut on = vec![Segment::new(if l == 'ᄃ' { "ð" } else { "θ" })];
        if !glide.is_empty() {
            on.push(Segment::new(glide));
        }
        let cd = w.tail.iter().filter(|&&t| t != 'ᇫ').filter_map(|&t| coda(t)).map(Segment::new).collect();
        return Some(Syllable { text: w.text.clone(), onset: on, nucleus: Segment::new(nucleus), coda: cd, ..Default::default() });
    }
    let son = sonorant(w.lead);
    let (mut glide, mut nucleus) = vowel(&w.vowel, son)?;
    let mut on: Vec<Segment> = Vec::new();
    // ᅙᆞ 는 그 자체가 [ə] 를 나타내는 한 단위 (성절 자음 표기 ᅙᆞᆫ ᅙᆞᆼ ᅙᆞᇢ = ən əŋ əʋ):
    // ㆆ 와 ㆍ 로 나눠 각각 음가를 매기지 않는다
    let silent_hieut = w.lead == Some('ᅙ') && w.vowel == ['ᆞ'];
    if let Some(l) = w.lead.filter(|_| !silent_hieut)
        && let Some((ipa, aspirated, vot)) = onset(l)
    {
        let (mut ipa, mut aspirated, mut vot) = (ipa, aspirated, vot);
        // ㅢ (and ㅢ+ㅜ) after ㅈ ㅊ ㅅ: the retroflex onset and the apical ʅ (즤 = zhi)
        if matches!(w.vowel.as_slice(), ['ᅴ'] | ['ᆗ']) && let Some(r) = retroflex(ipa) {
            ipa = r;
            if w.vowel == ['ᅴ'] {
                nucleus = "ʅ";
            }
            glide = "";
        } else if w.vowel == ['ᅴ'] {
            glide = ""; // ㅢ after another consonant is [i] (희 → 히)
        }
        // 자음 + j 활음(ㅑ ㅕ ㅛ ㅠ ㅒ ㅖ) = 경구개 자음: 활음은 그 자음에 흡수된다
        //   ㆅ+j = ç, ㄲ+j = c, ㄱ+j = ɟ, ㅋ+j = cʰ, ㄴ+j = ɲ, ㄹ+j = ʎ
        if glide == "j" {
            let pal: Option<(&'static str, Option<bool>, Option<f64>)> = match l {
                'ᅘ' => Some(("ç", None, None)),
                'ᄁ' => Some(("c", Some(false), None)),
                'ᄀ' => Some(("ɟ", None, None)),
                'ᄏ' => Some(("c", Some(true), Some(0.08))),
                'ᄂ' => Some(("ɲ", None, None)),
                'ᄅ' => Some(("ʎ", None, None)),
                'ᄉ' | 'ᄊ' => Some(("ʃ", None, None)),
                'ᅀ' => Some(("ʒ", None, None)),
                _ => None,
            };
            if let Some((p, a, v)) = pal {
                (ipa, aspirated, vot) = (p, a, v);
                glide = "";
            }
        }
        // ㅅ ㅆ 은 ㅣ·j 앞에서 ɕ (시 = 중국어 xi)
        if matches!(l, 'ᄉ' | 'ᄊ') && (glide == "j" || nucleus == "i") && ipa == "s" {
            ipa = "ɕ";
        }
        // 초성 ㄹ + 종성 ㅿ = ɽ (권설 탄음): 종성 ㅿ는 초성의 권설을 표시하고 따로 소리 나지 않는다
        if l == 'ᄅ' && w.tail.contains(&'ᇫ') {
            ipa = "ɽ";
        }
        // 초성 ㅌ ㄸ ㄷ + 종성 ㅿ: 종성 ㅿ는 초성을 바꾸는 표지이고 따로 소리 나지 않는다.
        //   ㅌ+ㅿ = θ (치 마찰음), ㄸ+ㅿ = ʈ (권설 파열음), ㄷ+ㅿ = ɖ (권설 유성 파열음).
        //   자음만 나타내는 표기: 트ᇫ = θ, 드ᇫ = ð (모음 ㅡ), ᄃᆞᇫ = ɖ, ᄄᆞᇫ = ʈ (모음 ㆍ, ㆍ는 표기의 구성요소).
        //   뜨ᇫ 도 ʈ 단독으로 읽는다.
        if matches!(l, 'ᄐ' | 'ᄄ' | 'ᄃ') && w.tail.contains(&'ᇫ') {
            let eu = w.vowel == ['ᅳ'];
            let arae = w.vowel == ['ᆞ'] && matches!(l, 'ᄃ' | 'ᄄ');
            (ipa, aspirated, vot) = match l {
                'ᄐ' => ("θ", None, None),
                'ᄄ' => ("ʈ", Some(false), None),
                _ if eu => ("ð", None, None),
                _ => ("ɖ", None, None),
            };
            if eu || arae {
                nucleus = "";
            }
        }
        // ㅿ + ㅢ = 권설 [ʐ] + ʅ (중국어 r 이 [ʐ]로 나는 경우: ᅀᅴ = ri), ㅈ ㅊ ㅅ + ㅢ 와 같은 원리
        if l == 'ᅀ' && w.vowel == ['ᅴ'] {
            ipa = "ʐ";
            nucleus = "ʅ";
            glide = "";
        }
        // 전동 r 의 세기: ᄙ = 강한 떨림(스페인어 rr), ᄛ = 약한 떨림
        let dur = match l {
            'ᄙ' => Some(0.2),
            'ᄛ' => Some(0.06),
            _ => None,
        };
        on.push(Segment { aspirated, vot, dur, ..Segment::new(ipa) });
    }
    if !glide.is_empty() {
        on.push(Segment::new(glide));
    }
    let mut cd: Vec<Segment> = Vec::new();
    let mut tails = w.tail.iter().copied().peekable();
    while let Some(t) = tails.next() {
        // 겹받침 ㄴㆆ (ᆫ + ᇹ) = ɲ (초성 자리의 ㄴ+j 에 대응)
        if t == 'ᆫ' && tails.peek() == Some(&'ᇹ') {
            tails.next();
            cd.push(Segment::new("ɲ"));
            continue;
        }
        if t == 'ᇫ' {
            // coda ㅿ: r-colouring after ㅓ (ᅟᅥᇫ = ɝ), a retroflex confirmation after ʅ or on ㄹ (ɽ), else nothing
            if !matches!(w.lead, Some('ᄅ' | 'ᄐ' | 'ᄄ' | 'ᄃ')) && matches!(nucleus, "ʌ̠" | "ɤ̞") {
                nucleus = "ɝ";
            }
            continue;
        }
        if let Some(c) = coda(t) {
            let dur = match t {
                'ퟛ' => Some(0.06),
                'ᇐ' => Some(0.2),
                _ => None,
            };
            cd.push(Segment { dur, ..Segment::new(c) });
        }
    }
    Some(Syllable { text: w.text.clone(), onset: on, nucleus: Segment::new(nucleus), coda: cd, ..Default::default() })
}

pub fn word_syllables(word: &str) -> Vec<Syllable> {
    let ws = split(word);
    let mut out: Vec<(usize, Syllable)> = ws.iter().enumerate().filter_map(|(i, w)| syllable(w).map(|s| (i, s))).collect();
    // 음절 경계를 넘는 결합: 앞 음절 종성 + 다음 음절 초성이 한 소리가 된다 (다음 음절 첫소리로 낸다)
    //   종성 ㄹ + 초성 ㅎ = ʀ (목젖 전동음), 종성 ㅎ + 초성 ㄹ = ʁ (목젖 마찰음)
    for k in 0..out.len().saturating_sub(1) {
        let (i, j) = (out[k].0, out[k + 1].0);
        if j != i + 1 {
            continue;
        }
        let pair = (ws[i].tail.last().copied(), ws[j].lead);
        let merged = match pair {
            (Some('ᆯ'), Some('ᄒ')) => Some("ʀ"),
            (Some('ᇂ'), Some('ᄅ')) => Some("ʁ"),
            _ => None,
        };
        if let Some(m) = merged {
            out[k].1.coda.pop(); // 앞 음절 받침 ㄹ·ㅎ 은 합쳐진 소리로 넘어간다
            if let Some(first) = out[k + 1].1.onset.first_mut() {
                *first = Segment::new(m);
            }
        }
    }
    // 모음 사이의 받침 ㅎ = 유성 ɦ (다음 음절이 초성 없이 모음으로 시작할 때: 아랍어·프랑스어 표기). 그 밖의 받침 ㅎ = h
    for k in 0..out.len().saturating_sub(1) {
        let (i, j) = (out[k].0, out[k + 1].0);
        if j == i + 1 && ws[i].tail.last() == Some(&'ᇂ') && ws[j].lead.is_none() && !ws[j].vowel.is_empty()
            && let Some(last) = out[k].1.coda.last_mut()
            && last.ipa == "h"
        {
            *last = Segment::new("ɦ");
        }
    }
    // 자음만 나타내는 음절(트ᇫ = θ): 다음 음절의 첫소리에 붙이고, 단어 끝이면 앞 음절 받침으로 붙인다
    let mut syls: Vec<Syllable> = out.into_iter().map(|(_, s)| s).collect();
    let mut i = 0;
    while i < syls.len() {
        if syls[i].nucleus.ipa.is_empty() {
            let cons = std::mem::take(&mut syls[i].onset);
            if i + 1 < syls.len() {
                let mut on = cons;
                on.append(&mut syls[i + 1].onset);
                syls[i + 1].onset = on;
                syls[i + 1].text = format!("{}{}", syls[i].text, syls[i + 1].text);
                syls.remove(i);
                continue;
            } else if i > 0 {
                let t = syls[i].text.clone();
                syls[i - 1].coda.extend(cons);
                syls[i - 1].text.push_str(&t);
                syls.remove(i);
                continue;
            } else {
                // 홀로 쓰인 자음: 자음 자체를 길게 낸다
                syls[i].nucleus = cons.first().cloned().unwrap_or_else(|| Segment::new("ə"));
            }
        }
        i += 1;
    }
    syls
}

impl FrontEnd for Jeongeum {
    fn starts(&self, c: char) -> bool {
        is_block(c) || is_conjoining(c)
    }
    fn continues(&self, c: char) -> bool {
        is_block(c) || is_conjoining(c) || compat_tail(c).is_some()
    }
    /// 정음은 한글의 상위호환이고 표음 표기: 모든 한글을 쓴 대로 읽는다 (국물 = [국.물], 이어 읽으면 궁물로 적음).
    /// 형태주의 맞춤법(먹어 → 머거)을 발음으로 바꾸는 일은 이 처리기가 아니라 한글→정음 변환의 몫이다.
    fn word(&self, word: &str) -> Vec<Syllable> {
        word_syllables(word)
    }
}

// ── 한자(중국어) → 병음·성조 → 正音 ─────────────────────────────────────────────────────────────

/// 병음(성조 숫자) → 正音 표기, 《正音으로 듣고쓰다》 pp.35–37 규칙으로 만든 표 (jeongeum_pinyin.py 가 생성).
static PINYIN_TABLE: &str = include_str!("jeongeum_pinyin.tsv");

/// zhong1 → 즤옹. 표에 없는 음절은 None.
pub fn pinyin_to_jeongeum(py: &str) -> Option<&'static str> {
    static TABLE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    TABLE
        .get_or_init(|| PINYIN_TABLE.lines().filter(|l| !l.starts_with('#')).filter_map(|l| l.split_once('\t')).collect())
        .get(py)
        .copied()
}

/// 성조를 正音 표기대로 노래의 음높이 굽이로 싣는다 (멜로디 음에서 반음 단위로):
///   1성(경성 포함) 평평 · 2성 ㆁ = 아래에서 올라감 · 3성 ㅡ첨가 = ㅢ식 굴곡(ɨ 활음, 내려갔다 오름),
///   3성 ㆆ = 하강 · 4성 = 크게 하강, 첫머리 ㆆ(二 ᅙᅥᆯ)는 [ʔ].
/// [ʔ]는 正音 표기에 ㆆ가 있을 때만: 중국어 처리기가 초성 없는 음절에 붙이는 기본 [ʔ]는 뗀다.
fn apply_tone(s: &mut Syllable, tone: u8, spelling: &str) {
    s.onset.retain(|x| x.ipa != "ʔ");
    let bend = |at, len, max, st| Bend { at, len, max, st };
    match tone {
        2 => s.pitch = vec![bend(Edge::Start, 0.75, 0.6, -4.0)],
        3 if spelling.contains('ᅙ') => s.pitch = vec![bend(Edge::End, 0.8, 0.5, -3.0)],
        3 => {
            let glide = format!("ɨ{}", s.nucleus.ipa);
            if super::planner::expand(&glide).is_some() {
                s.nucleus.ipa = glide;
            }
            s.pitch = vec![bend(Edge::Middle, 0.4, 0.35, -2.0), bend(Edge::End, 0.35, 0.3, 3.0)];
        }
        4 => {
            if spelling.starts_with('ᅙ') {
                s.onset.insert(0, Segment::new("ʔ"));
            }
            s.pitch = vec![bend(Edge::End, 0.85, 0.5, -8.0)];
        }
        _ => {}
    }
}

/// 한자(간체) → 중국어 독음을 正音으로: 사전으로 병음·성조를 읽고(어구 우선, 성조 변화 포함), 소리는 중국어
/// 처리기의 음가로 내되 음절 글자는 正音 표기로 바꾸고 성조는 正音 표기대로 싣는다. 중국어 처리기의 상위호환.
pub struct JeongeumHan<'a>(pub &'a MandarinLexicon);

impl FrontEnd for JeongeumHan<'_> {
    fn starts(&self, c: char) -> bool {
        is_han(c)
    }
    fn word(&self, run: &str) -> Vec<Syllable> {
        let cs: Vec<char> = run.chars().collect();
        let pys = self.0.read(run);
        let mut syls = Mandarin(self.0).word(run);
        // 음절 ↔ 글자 맞추기: 음절 글자는 한 글자(儿화는 두 글자), 읽지 못한 글자는 음절이 없다
        let mut i = 0;
        for s in &mut syls {
            let first = s.text.chars().next();
            while i < cs.len() && Some(cs[i]) != first {
                i += 1;
            }
            let Some(py) = pys.get(i) else { break };
            let n = s.text.chars().count();
            let tone = py.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(5) as u8;
            let spelling = pinyin_to_jeongeum(py).unwrap_or("");
            // 儿화(块儿)의 正音 표기는 아직 정하지 않아 한자 그대로 둔다
            if n == 1 && !spelling.is_empty() {
                s.text = spelling.to_string();
            }
            apply_tone(s, tone, spelling);
            i += n;
        }
        syls
    }
}
