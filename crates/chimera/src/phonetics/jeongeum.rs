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
//! - ㅔ e̞, ㅚ ø̞, ㅗ o̞ (true-mid vowels); ᆜ breathy i̤ (ㅣ + breath; ᅙᆜ = ʔɨ); ㅢ after ㅈ ㅊ ㅅ is ʅ and turns the
//!   onset retroflex (즤 [tʂʅ]); a coda ㅿ there only confirms it. ᅟᅥᇫ is ɝ. ᅟᆕ ʉ, ᅟힼ ɵ.
//! - Vowel values: the Seeing Speech cardinal vowels (Janet Beck) measured and calibrated as the
//!   back keys ɯ̠ ɤ̠ ʌ̠ ɑ̠ and the midpoints ɤ̞ e̞ ø̞ o̞ (vowel_data.rs).
//!
//! - Lax ㄱ ㄷ ㅂ ㅈ are voiced (and ㅎ is ɦ) between voiced sounds, as the Korean front end does.
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
        ['ᆜ'] => ("", "i"),   // ㅣ→ㅡ: ㅣ + 숨 (숨 섞인 i̤), ᅙᆜ 는 ɨ — `syllable`
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
        'ᆼ' => "ŋ", // 현대 한글의 받침 ㅇ = [ŋ]. 옛 받침 ㆁ(ᇰ)은 비모음 표지: `syllable`
        'ᆺ' => "s",
        'ᆩ' | 'ᆿ' => "k",
        'ᇀ' | 'ᆽ' | 'ᆾ' | 'ᆻ' => "t",
        'ᇁ' => "p",
        // 한글 겹받침 (사용자 지정): 앞 자음이 나는 ㄳ ㄵ ㄼ ㄽ ㄾ ㅄ, 뒤 자음이 나는 ㄺ ㄻ ㄿ.
        // ㄶ(ɴ)·ㅀ(ʀ)은 正音의 옛 받침 값으로 따로 둔다
        'ᆪ' => "k",
        'ᆬ' => "n",
        'ᆲ' | 'ᆳ' | 'ᆴ' => "l",
        'ᆹ' => "p",
        'ᆰ' => "k",
        'ᆱ' => "m",
        'ᆵ' => "p",
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

/// 받침 ㆁ(ᇰ)은 자음이 아니라 앞 모음을 비모음으로 만든다 (프랑스어 비모음: 모음 + 받침 ㆁ). 현대 한글의
/// 받침 ㅇ(ᆼ)은 [ŋ].
/// 프랑스어 비모음은 모음 + 받침 ㆁ 으로 적되, 모음 글자는 비모음의 소리에 맞춰 고른다 (사용자 지정):
///   ᅟힺᇰ [ã] · ᅟᅡᇰ [ɛ̃](북부) / ᅟᅢᇰ [ɛ̃](남부) · ᅟᅩᇰ [ɔ̃] · ᅟᆟᇰ [œ̃] · ᅟᅪᇰ [wɛ̃].
/// 그 밖의 모음 + ㆁ 은 그 모음을 그대로 콧소리로 낸다.
fn nasal_vowel(v: &[char]) -> Option<(char, &'static str)> {
    // (자음 규칙을 받지 않을 대리 모음 글자, 비모음의 바탕 모음)
    Some(match v {
        ['ힺ'] => ('ᅡ', "a"), // ㅡ결합이지만 목젖음 규칙(ᄏힺ = qʰ)을 받지 않는다: 프랑스어 quand [kã]
        ['ᅡ'] => ('ᅡ', "ɛ"),
        ['ᅢ'] => ('ᅢ', "ɛ"),
        ['ᅩ'] => ('ᅩ', "ɔ"),
        ['ᆟ'] => ('ᅩ', "œ"),
        ['ᅪ'] => ('ᅪ', "ɛ"),
        _ => return None,
    })
}

fn syllable(w: &Written) -> Option<Syllable> {
    let nasal = w.tail.contains(&'ᇰ');
    let french = if nasal { nasal_vowel(&w.vowel) } else { None };
    let mut s = match french {
        Some((proxy, _)) => syllable_oral(&Written { text: w.text.clone(), lead: w.lead, vowel: vec![proxy], tail: w.tail.clone() })?,
        None => syllable_oral(w)?,
    };
    if let Some((_, v)) = french {
        s.nucleus.ipa = v.to_string();
    }
    if nasal {
        s.nucleus.nasalized = Some(true);
    }
    // ᆜ (ㅣ→ㅡ 를 잇따라): ㅡ 는 기식(숨)이라 숨 섞인 ㅣ [i̤] (산스크리트 dhi ᄃᆜ = di̤, ghi ᄀᆜ = gi̤).
    // ㆆ 를 앞세운 ᅙᆜ 는 중설 고모음 [ʔɨ] (폴란드어 y): ㆆ 의 [ʔ] 가 날카로움을 더한다 (사용자 지정)
    if w.vowel == ['ᆜ'] {
        if w.lead == Some('ᅙ') {
            s.nucleus.ipa = "ɨ".into();
        } else if let Some(l @ ('ᄌ' | 'ᄍ' | 'ᄎ' | 'ᄉ' | 'ᄊ')) = w.lead {
            // ㅈ ㅊ ㅅ 는 ㆆ 를 품은 글자라 (사용자 지정) ㅈ ㅊ ㅅ + ᆜ 는 ᅙᆜ 처럼 날카로운 [ɨ]: 중국어 z c s,
            // 폴란드어 cy sy. 자음은 혀끝 치조 파찰·마찰음 [ts tsʰ s] (ㅣ 앞의 구개음화 ɕ tɕ 가 아님)
            s.nucleus.ipa = "ɨ".into();
            if let Some(first) = s.onset.first_mut() {
                let (ipa, asp, vot) = match l {
                    'ᄌ' => ("ts", Some(true), Some(0.03)),
                    'ᄍ' => ("ts", Some(false), None),
                    'ᄎ' => ("ts", Some(true), Some(0.08)),
                    _ => ("s", None, None),
                };
                *first = Segment { aspirated: asp, vot, dur: first.dur, ..Segment::new(ipa) };
            }
        } else {
            s.nucleus.breathy = Some(true);
            // ㄱ ㄷ ㅂ ㅈ + ᆜ: ㅡ 가 유성 기식이라 유성음 (산스크리트 ghi dhi bhi jhi = ᄀᆜ ᄃᆜ ᄇᆜ ᄌᆜ → gʱi dʱi bʱi dʑʱi)
            if let Some(first) = s.onset.first_mut() {
                let voiced = match (w.lead, first.ipa.as_str()) {
                    (Some('ᄀ'), "k") => Some("g"),
                    (Some('ᄃ'), "t") => Some("d"),
                    (Some('ᄇ'), "p") => Some("b"),
                    (Some('ᄌ'), "tɕ") => Some("dʑ"),
                    _ => None,
                };
                if let Some(v) = voiced {
                    *first = Segment { dur: first.dur, ..Segment::new(v) };
                }
            }
        }
    }
    Some(s)
}

fn syllable_oral(w: &Written) -> Option<Syllable> {
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
    // 떨어진 ㅡ: ㅡ 와 다음 모음을 두 글자로 이어 쓴 것(ᄀ+ᅳ+ᅡ)은 ㅡ 가 앞 자음의 유성 표지다 — ᄀ g · ᄃ d · ᄇ b
    //   (사용자 지정: 획이 닿지 않은 ㅢ 는 [그+ㅣ] = [gi], 닿은 결합모음 긔·ᄀힹ 은 목젖음 [ɢi]·[ɢa]).
    //   일본어 탁음 が ᄀ+ᅳ+ᅡ [ga], ぎ ᄀ+ᅳ+ᅵ [gi], だ ᄃ+ᅳ+ᅡ [da].
    if let Some(l @ ('ᄀ' | 'ᄃ' | 'ᄇ')) = w.lead
        && let ['ᅳ', rest @ ..] = w.vowel.as_slice()
        && !rest.is_empty()
    {
        let (glide, nucleus) = vowel(rest, false)?;
        let mut on = vec![Segment::new(match l {
            'ᄀ' => "g",
            'ᄃ' => "d",
            _ => "b",
        })];
        if !glide.is_empty() {
            on.push(Segment::new(glide));
        }
        let cd = w.tail.iter().filter_map(|&t| coda(t)).map(Segment::new).collect();
        return Some(Syllable { text: w.text.clone(), onset: on, nucleus: Segment::new(nucleus), coda: cd, ..Default::default() });
    }
    // ㄲ ㄱ ㅋ ㅎ + ㅡ 결합 모음 (ᄁힹ = [qa]): ㅡ는 자음에 흡수되어 목젖 자음 q ɢ qʰ χ 가 되고 나머지 모음이 남는다.
    //   ힹ→ㅏ ힺ→ㅓ ힻ→ㅔ ힼ→ㅗ ㅢ→ㅣ. ㅡ 단독(그 끄)과 ᆕ(= ʉ)는 목젖 규칙을 받지 않는다 (그 = [kɯ]; ᄀᆕ 는 위에서 [gʉ]).
    // ᄀ·ᄃ·ᄇ + ᆕ = [gʉ]·[dʉ]·[bʉ]: ᆕ 속의 ㅡ 가 앞 자음을 유성으로 (일본어 ぐ ぶ, 사용자 지정).
    // ᄁᆕ [k˭ʉ] · ᄏᆕ [kʰʉ] 는 무성 그대로. 받침 ㅿ 가 붙은 ᄃ 는 치 마찰음 규칙(ᄃᆕᇫ = ðʉ)으로
    if let Some(l @ ('ᄀ' | 'ᄃ' | 'ᄇ')) = w.lead
        && w.vowel == ['ᆕ']
        && !(l == 'ᄃ' && w.tail.contains(&'ᇫ'))
    {
        let cd = w.tail.iter().filter_map(|&t| coda(t)).map(Segment::new).collect();
        let c = match l {
            'ᄀ' => "g",
            'ᄃ' => "d",
            _ => "b",
        };
        return Some(Syllable { text: w.text.clone(), onset: vec![Segment::new(c)], nucleus: Segment::new("ʉ"), coda: cd, ..Default::default() });
    }
    // ᄃ·ᄇ + 붙은 ㅡ결합모음 (ᄃힹ ᄃힻ ᄃힼ, ᄇힹ ᄇힻ ᄇힼ) = 유성 [d]·[b] + 남은 모음: ㄷ ㅂ 에는 목젖 짝이 없어 ㅡ 는
    // 유성 표지로만 읽는다 (책: 그 g · 드 d · 브 b; 일본어 だ で ど ば べ ぼ, 사용자 지정). 떨어진 ㅡ(ᄃ+ᅳ+ᅡ)와 같은 소리.
    // び 는 떨어진 [브+ㅣ](ᄇ+ᅳ+ᅵ)로 적는다 — 붙은 ㅂ+ㅢ 는 다른 소리라 뺀다 (사용자 지정; ㄷ+ㅢ 도 같이 뺀다).
    // 받침 ㅿ 가 붙으면 ᄃ 는 치 마찰음 규칙(ᄃힹᇫ = ða)을 따른다.
    if let Some(l @ ('ᄃ' | 'ᄇ')) = w.lead
        && !(l == 'ᄃ' && w.tail.contains(&'ᇫ'))
        && let [first, rest @ ..] = w.vowel.as_slice()
        && *first != 'ᅴ'
        && let Some(base) = eu_base(*first)
    {
        let mut v = vec![base];
        v.extend_from_slice(rest);
        let (glide, nucleus) = vowel(&v, false)?;
        let mut on = vec![Segment::new(if l == 'ᄃ' { "d" } else { "b" })];
        if !glide.is_empty() {
            on.push(Segment::new(glide));
        }
        let cd = w.tail.iter().filter_map(|&t| coda(t)).map(Segment::new).collect();
        return Some(Syllable { text: w.text.clone(), onset: on, nucleus: Segment::new(nucleus), coda: cd, ..Default::default() });
    }
    // ᆕ(ㅡ+ㅜ = ʉ)는 예외: 그 자체가 모음 [ʉ]라 목젖음 규칙을 받지 않는다 (ᄀᆕ ᄏᆕ ᄒᆕ = 일본어 ぐ く ふ, 사용자 지정)
    if let Some(l @ ('ᄁ' | 'ᄀ' | 'ᄏ' | 'ᄒ')) = w.lead
        && let [first, rest @ ..] = w.vowel.as_slice()
        && *first != 'ᆕ'
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

/// 한글 겹받침의 두 자음: (받침에 남는 자음, 다음 음절 첫소리로 옮겨 갈 자음).
fn double_coda(t: char) -> Option<(char, char)> {
    Some(match t {
        'ᆪ' => ('ᆨ', 'ᄉ'), // ㄳ
        'ᆬ' => ('ᆫ', 'ᄌ'), // ㄵ
        'ᆲ' => ('ᆯ', 'ᄇ'), // ㄼ
        'ᆳ' => ('ᆯ', 'ᄉ'), // ㄽ
        'ᆴ' => ('ᆯ', 'ᄐ'), // ㄾ
        'ᆹ' => ('ᆸ', 'ᄉ'), // ㅄ
        'ᆰ' => ('ᆯ', 'ᄀ'), // ㄺ
        'ᆱ' => ('ᆯ', 'ᄆ'), // ㄻ
        'ᆵ' => ('ᆯ', 'ᄑ'), // ㄿ
        _ => return None,
    })
}

pub fn word_syllables(word: &str) -> Vec<Syllable> {
    let mut ws = split(word);
    // 겹받침 + 초성 없는 음절: 뒤 자음이 다음 음절 첫소리로 옮겨 간다 (앉아 [안자], 넓어 [널버], 값이 [갑시],
    // 닭이 [달기]: 표준 발음법 제14항). 홀로 또는 자음 앞에서는 한 자음만 난다 (`coda`)
    for i in 0..ws.len().saturating_sub(1) {
        if ws[i + 1].lead.is_none()
            && !ws[i + 1].vowel.is_empty()
            && let Some(&t) = ws[i].tail.last()
            && let Some((stay, moved)) = double_coda(t)
        {
            *ws[i].tail.last_mut().unwrap() = stay;
            ws[i + 1].lead = Some(moved);
        }
    }
    let ws = ws;
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
    // 유성음 사이의 평음 ㄱ ㄷ ㅂ ㅈ 은 유성 [g d b dʑ], ㅎ 은 [ɦ] (한국어 처리기와 같은 규칙, 사용자 지정):
    // 앞 음절이 모음이나 유성 받침(ㄴ ㄹ ㅁ ㅇ, 옛 ㆁ ㅱ ㅭ ㄶ)으로 끝날 때 (가다 [kada], 안다 [anda])
    for k in 1..out.len() {
        let (i, j) = (out[k - 1].0, out[k].0);
        if j != i + 1 {
            continue;
        }
        let voiced_before = match ws[i].tail.last() {
            None => true,
            Some(t) => matches!(t, 'ᆫ' | 'ᆯ' | 'ᆷ' | 'ᆼ' | 'ᇰ' | 'ᇢ' | 'ᇙ' | 'ᆭ' | 'ᆬ' | 'ᆲ' | 'ᆳ' | 'ᆴ' | 'ᆱ'),
        };
        if !voiced_before {
            continue;
        }
        let Some(first) = out[k].1.onset.first_mut() else { continue };
        let voiced = match (ws[j].lead, first.ipa.as_str(), first.aspirated) {
            (Some('ᄀ'), "k", Some(true)) => "g",
            (Some('ᄃ'), "t", Some(true)) => "d",
            (Some('ᄇ'), "p", Some(true)) => "b",
            (Some('ᄌ'), "tɕ", Some(true)) => "dʑ",
            (Some('ᄒ'), "h", _) => "ɦ",
            _ => continue,
        };
        *first = Segment { dur: first.dur, ..Segment::new(voiced) };
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

/// 3성 ㆆ형: ㆆ가 둘째 음절 이후의 첫소리로 쓰였다 (마ᅙᅡ). 첫 음절의 ᅙᆞ·ᅙᅳ(성절 자음 ᅙᅳᄋᆞᆼ)는 아니다.
fn hieut_inside(spelling: &str) -> bool {
    spelling.char_indices().any(|(i, c)| i > 0 && c == 'ᅙ')
}

/// 첫머리 ㆆ = [ʔ] (二 ᅙᅥᆯ). 단 ᅙᆞ(= ə)·ᅙᅳ 는 성절 자음 표기의 한 단위라 [ʔ]가 아니다 (嗯 ᅙᆞᆼ).
fn glottal_start(spelling: &str) -> bool {
    let mut cs = spelling.chars();
    cs.next() == Some('ᅙ') && !matches!(cs.next(), Some('ᆞ' | 'ᅳ'))
}

/// 성조를 正音 표기대로 노래의 음높이 굽이로 싣는다 (멜로디 음에서 반음 단위로):
///   1성(경성 포함) 평평 · 2성 ㆁ = 아래에서 올라감 · 4성 = 크게 하강, 첫머리 ㆆ(二 ᅙᅥᆯ)는 [ʔ].
///   3성은 표기는 하나지만 소리는 자리에 따라: 구 끝(`last`)이면 온3성 214 — ㅡ첨가형은 ㅢ식 굴곡(ɨ 활음,
///   내려갔다 오름), ㆆ형은 하강 — 이고, 구 안이면 반3성 21(내려가기만). 3성+3성 → 2성은 사전이 이미 바꿔 온다.
/// [ʔ]는 正音 표기에 ㆆ가 있을 때만: 중국어 처리기가 초성 없는 음절에 붙이는 기본 [ʔ]는 뗀다.
fn apply_tone(s: &mut Syllable, tone: u8, spelling: &str, last: bool) {
    s.onset.retain(|x| x.ipa != "ʔ");
    let bend = |at, len, max, st| Bend { at, len, max, st };
    match tone {
        2 => s.pitch = vec![bend(Edge::Start, 0.75, 0.6, -4.0)],
        3 if !last || hieut_inside(spelling) => s.pitch = vec![bend(Edge::End, 0.8, 0.5, -3.0)],
        3 => {
            let glide = format!("ɨ{}", s.nucleus.ipa);
            if super::planner::expand(&glide).is_some() {
                s.nucleus.ipa = glide;
            }
            s.pitch = vec![bend(Edge::Middle, 0.4, 0.35, -2.0), bend(Edge::End, 0.35, 0.3, 3.0)];
        }
        4 => {
            if glottal_start(spelling) {
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
            // 嗯: 사전은 옛 표기 n2(물음) 하나로 읽는다. 현대 표준(《现代汉语词典》)대로 ng 로 읽고, 성조는 가장 흔한
            // 4성 대답 ǹg ᅙᆞᆼ("응.")을 기본으로 한다 (물음 ńg ᅙᆞᇰ, 놀람 ňg ᅙᅳᆼ 은 正音으로 직접 적는다)
            let std_py;
            let py = if py.trim_end_matches(|c: char| c.is_ascii_digit()) == "n" {
                std_py = "ng4".to_string();
                &std_py
            } else {
                py
            };
            let n = s.text.chars().count();
            let tone = py.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(5) as u8;
            let spelling = pinyin_to_jeongeum(py).unwrap_or("");
            // 儿화(块儿)의 正音 표기는 아직 정하지 않아 한자 그대로 둔다
            if n == 1 && !spelling.is_empty() {
                s.text = spelling.to_string();
            }
            apply_tone(s, tone, spelling, i + n >= cs.len());
            // 성절 자음: 중국어 처리기는 n·ng·m 을 모두 [ən]으로 근사한다. 正音 표기대로 받침을 낸다
            // (ᅙᆞᆫ ən · ᅙᆞᆼ əŋ · ᅙᆞᇢ əʋ)
            let base = py.trim_end_matches(|c: char| c.is_ascii_digit());
            if matches!(base, "n" | "ng" | "m" | "hm" | "hng") {
                let coda = if base.ends_with("ng") { "ŋ" } else if base.ends_with('m') { "ʋ" } else { "n" };
                s.coda = vec![Segment::new(coda)];
            }
            i += n;
        }
        syls
    }
}
