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
//! - The vowel after a 정음 sonorant onset (ㄴ ㄹ ㅁ ㅿ ㆁ ᄛ, and no onset at all) sits one step lower:
//!   ㅡ ɯ → ɤ, ㅓ ɤ̞ → ʌ, ㅔ e → e̞, ㆍ ə → ɑ. ㆆ is the glottal stop [ʔ] and is not a sonorant.
//! - ㅔ e̞, ㅚ ø̞, ㅗ o̞ (true-mid vowels); ᆜ = ㅣ then a weak breath (ᄉᆜ ɕih; ᅙᆜ = ʔɨ); ㅢ after ㅈ ㅊ ㅅ is ʅ and turns the
//!   onset retroflex (즤 [tʂʅ]); a coda ㅿ there only confirms it. ᅟᅥᇫ is ɝ. ᅟᆕ ʉ, ᅟힼ ɵ.
//! - Vowel values: the Seeing Speech cardinal vowels (Janet Beck) measured and calibrated as the
//!   back keys ɯ ɤ ʌ ɑ and the midpoints ɤ̞ e̞ ø̞ o̞ (vowel_data.rs).
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
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct Jeongeum;

/// 正音 읽기 스위치 (사용자 지정 2026-11-26): 정음 기호가 IPA 보다 적어 한 표기에 겹치는 읽기를 고르는 미세 조정.
/// 화면의 Mood 페이지 네 조각 스위치가 켜고 끈다. 모두 꺼진 것이 正音의 기본이고, 기본만으로 읽기가 온전해야 한다.
/// 지금 기능이 있는 것은 둘이다 (나머지 두 조각은 자리만 잡아 두었다):
///   MARK_I  ㅣ → ɨ : ᆜ 를 [ɨ] 로 (폴란드어·러시아어를 정음으로 적었을 때). 꺼지면 [iɯ]
///   MARK_EU ㅡ → ḥ : 산스크리트·힌디어를 정음으로 적었을 때 (사용자 지정 2026-11-26) —
///     · ᆜ [iɤ̤] · ᅼ [ɐɤ̤] · ᆣ [aɤ̤]: 모음이 숨 섞인 ㅡ 로 풀린다
///     · 겹종성 ㄹㆁ ퟛ = ɽ (पड़ना 뻐ퟛ나) · ㄹㅎ ᆶ = ɽʱ (पढ़ना 뻟나). ㄹㅿ ᇗ = ɭ (ळ) 은 스위치 없이도 그렇다
///     · 초성 ㅎ = ɦ (ह) · 초성 ㄹ = 탄음 ɾ · 초성 ㄱ ㅈ ㄷ ㅂ = 유성 g dʒ d b
///     · 초성 ㅅ = ɕ · 초성 ㅆ = s · 초성 ㅅ + 종성 ㅿ = ʂ
///     · 초성 ㄲ ㄸ ㅃ ㅉ = 기식 없는 k˭ t˭ p˭ tɕ˭ (ㆆ 없이) · 초성 ㅊ + 모음 + 종성 ㆆ = tɕ˭ + 모음 (2026-11-27)
pub const MARK_I: u8 = 1;
pub const MARK_EU: u8 = 2;
thread_local! {
    static MARKS: Cell<u8> = const { Cell::new(0) };
}
/// 읽기 스위치를 정한다 (MARK_* 의 합). 이 스레드에서 다음에 읽는 글부터.
pub fn set_marks(bits: u8) {
    MARKS.with(|m| m.set(bits));
}
pub fn marks() -> u8 {
    MARKS.with(|m| m.get())
}
pub fn mark(bit: u8) -> bool {
    marks() & bit != 0
}

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
        } else if let Some(l) = compat_lead(c) {
            // 호환 자모 낱자 (ㅸ ㄱ, 자판으로 치면 나오는 꼴): 초성으로
            out.push(l);
        } else if let Some(v) = compat_vowel(c) {
            // 호환 자모 모음 (ㆉ ㆍ ㅏ): 중성으로
            out.push(v);
        } else {
            out.push(c);
        }
    }
    out
}

/// 호환 자모 모음 → 중성 자모 (ㅏ…ㅣ, 옛 모음 ㆇ ㆈ ㆉ ㆊ ㆋ ㆌ ㆍ ㆎ)
fn compat_vowel(c: char) -> Option<char> {
    match c {
        'ㅏ'..='ㅣ' => char::from_u32(c as u32 - 0x314F + 0x1161),
        'ㆇ' => Some('ᆄ'),
        'ㆈ' => Some('ᆅ'),
        'ㆉ' => Some('ᆈ'),
        'ㆊ' => Some('ᆑ'),
        'ㆋ' => Some('ᆒ'),
        'ㆌ' => Some('ᆔ'),
        'ㆍ' => Some('ᆞ'),
        'ㆎ' => Some('ᆡ'),
        _ => None,
    }
}

/// 호환 자모 자음 → 초성 자모 (겹받침 꼴 ㄳ ㄵ … 은 초성이 없어 제외)
fn compat_lead(c: char) -> Option<char> {
    let l = match c {
        'ㄱ' => 'ᄀ', 'ㄲ' => 'ᄁ', 'ㄴ' => 'ᄂ', 'ㄷ' => 'ᄃ', 'ㄸ' => 'ᄄ', 'ㄹ' => 'ᄅ', 'ㅁ' => 'ᄆ',
        'ㅂ' => 'ᄇ', 'ㅃ' => 'ᄈ', 'ㅅ' => 'ᄉ', 'ㅆ' => 'ᄊ', 'ㅇ' => 'ᄋ', 'ㅈ' => 'ᄌ', 'ㅉ' => 'ᄍ',
        'ㅊ' => 'ᄎ', 'ㅋ' => 'ᄏ', 'ㅌ' => 'ᄐ', 'ㅍ' => 'ᄑ', 'ㅎ' => 'ᄒ', 'ㅥ' => 'ᄔ', 'ㅱ' => 'ᄝ',
        'ㅸ' => 'ᄫ', 'ㅹ' => 'ᄬ', 'ㅿ' => 'ᅀ', 'ㆁ' => 'ᅌ', 'ㆄ' => 'ᅗ', 'ㆅ' => 'ᅘ', 'ㆆ' => 'ᅙ',
        _ => return None,
    };
    Some(l)
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
                // 초성이 잇따르면 앞의 것은 제 블록을 이루는 낱자다 (ㅅ타 = ㅅ + 타): 뒤의 것에 덮여 사라지지 않게
                if lead.is_some() {
                    break;
                }
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
        // 정음 sonorant (사용자 지정): ㄴ ㄹ ㅁ ㅿ ㆁ ᄛ
        Some(c) => matches!(c, 'ᄂ' | 'ᄅ' | 'ᄆ' | 'ᅀ' | 'ᅌ' | 'ᄛ'),
    }
}

/// 종성이 없거나 정음 sonorant (ㄴ ㄹ ㅁ ㅿ ㆁ, ㆁ 을 이은 ㅇ, 그리고 소리 나는 쪽이 이것들인 겹받침)인가
fn coda_sonorant(tail: &[char]) -> bool {
    tail.iter().all(|t| matches!(t, 'ᆫ' | 'ᇈ' | 'ᆯ' | 'ᇗ' | 'ퟛ' | 'ᆷ' | 'ᆼ' | 'ᇰ' | 'ᇫ' | 'ᇢ' | 'ᇙ' | 'ᆭ' | 'ᆬ' | 'ᆲ' | 'ᆳ' | 'ᆴ' | 'ᆶ' | 'ᆱ'))
}

/// Vowel jamo → (glide, vowel). The rule-bound ones are resolved by the onset (`son`) and the coda
/// (`son_coda`): 정음 모음과 IPA 기호의 배정 (사용자 지정 2026-11-23) —
///   ㅏ [a] (사용자 지정 2026-11-26: [ɐ̞] 789/1506 은 원본의 a 보다 F2 가 227 Hz 낮아 답답하게 들렸다) · ㅐ [ɛ] · ㅔ [e̞] (초성과 상관없이) · ㅚ [ø̞] · ퟅ [ɑ] · ᆟ [ɔ̞] (ɔ 와 ɒ 의 중간)
///   종성이 없거나 정음 sonorant 인 음절 / 그 밖의 종성인 음절:
///   ㅣ [i] / [ɪ] · ㅡ [ɤ] / [ɯ] · ㅓ [ʌ] / [ɤ̞] · ㅗ [o̞] / [o]
///   ㆍ 는 초성으로: 정음 sonorant 뒤 [ɑ], 그 밖 [ə]
fn vowel(v: &[char], son: bool, son_coda: bool) -> Option<(&'static str, &'static str)> {
    let a = "a";
    let eo = if son_coda { "ʌ" } else { "ɤ̞" };
    let eu = if son_coda { "ɤ" } else { "ɯ" };
    let o = if son_coda { "o̞" } else { "o" };
    let i = if son_coda { "i" } else { "ɪ" };
    let e = "e̞";
    // 정음 sonorant 뒤의 ㆍ 는 [ɐ] (사용자 지정 2026-11-27; 그 전에는 [ɑ])
    let arae = if son { "ɐ" } else { "ə" };
    Some(match v {
        ['ᅡ'] => ("", a),
        ['ᅢ'] => ("", "ɛ"),
        ['ᅣ'] => ("j", a),
        ['ᅤ'] => ("j", "ɛ"),
        ['ᅥ'] => ("", eo),
        ['ᅦ'] => ("", e),
        ['ᅧ'] => ("j", eo),
        ['ᅨ'] => ("j", e),
        ['ᅩ'] => ("", o),
        ['ᅪ'] => ("w", a),
        ['ᅫ'] => ("w", "ɛ"),
        ['ᅬ'] => ("", "ø̞"),
        ['ᅭ'] => ("j", o),
        ['ᅮ'] => ("", "u"),
        ['ᅯ'] => ("w", eo),
        ['ᅰ'] => ("w", e),
        ['ᅱ'] => ("w", "i"),
        ['ᅲ'] => ("j", "u"),
        ['ᅳ'] => ("", eu),
        ['ᅴ'] => ("ɰ", "i"), // after a consonant: see `syllable`
        ['ᅵ'] => ("", i),
        ['ᆞ'] => ("", arae),
        ['\u{11A1}'] => ("", "ɑɪ"), // ㆎ (ㆍ+ㅣ): [ɑɪ] (사용자 지정)
        ['ᆜ'] => ("", "i"),   // ㅣ→ㅡ: [iɯ], ᅙᆜ 는 ɨ — `syllable_marked` (여기서는 초성을 ㅣ 앞처럼 읽게 할 뿐)
        ['ᅼ'] => ("", "ɐ"),   // ㅓ→ㅡ: 산스크리트 내재모음 a + 숨 (ᄀᅼ gʱɐ, ᄃᅼ dʱɐ, ᄇᅼ bʱɐ, ᄌᅼ dʑʱɐ)
        ['ᆣ'] => ("", a),     // ㅏ→ㅡ: ā + 숨 (ᄃᆣ dʱa)
        // ᆖ (U+1196, ㅡ+ㅡ) 는 正音키메라에서 쓰지 않는다 (사용자 지정 2026-11-27): 읽지 않는다
        ['ᆕ'] => ("", "ʉ"),   // ㅡ+ㅜ
        ['ힼ'] => ("", "ɵ"),   // ㅡ+ㅗ
        ['ᆐ'] => ("", "yɛ"),  // ㅠ+ㅔ (jue4 ᄌᆐ)
        ['ᆛ'] => ("", "y"),   // ㅣ+ㅜ: ü
        ['ᆟ'] => ("", "ɔ̞"),  // ㆍ+ㅓ: ɔ 와 ɒ 의 중간 (사용자 지정 2026-11-23)
        ['ᆗ'] => ("", "ʊ"),   // ㅢ+ㅜ: [ʊ]; 초성 없이 ᅟᆗ 는 [ʔʊ] — `syllable` (사용자 지정 2026-11-23)
        ['ힹ'] => ("", "ɨa"),  // ㅡ+ㅏ: the 3rd tone's glide
        ['ힺ'] => ("", "ɘ"),   // ㅡ+ㅓ: 중설 반폐 [ɘ] (사용자 지정 2026-11-23)
        ['ᆚ'] => ("", "ø"),   // ㅣ+ㅗ: [ø] (사용자 지정)
        ['ᆈ'] => ("", "œ"),   // ㆉ ㅛ+ㅣ: [œ] (사용자 지정)
        ['\u{D7C5}'] => ("", "ɑ"), // ㆍ+ㅏ: [ɑ] (사용자 지정)
        ['\u{11A0}'] => ("", "aʊ"), // ㆍ+ㅜ: [au] (사용자 지정 2026-11-26)
        // ㆍ+ㅐ: [æ] (사용자 지정). 유니코드에 한 글자가 없어 ㆍ 와 ㅐ 를 이어 쓴다 (ퟅ+ㅣ 도 같게)
        ['ᆞ', 'ᅢ'] | ['\u{D7C5}', 'ᅵ'] => ("", "æ"),
        // ᆜ+ㅜ: [ʏ] (사용자 지정) — ᆗ(ㅢ+ㅜ) = [ʊ] 와 대칭. 한 글자가 없어 ᆜ 와 ㅜ 를 이어 쓴다 (ㅣ ㅡ ㅜ 도 같게)
        ['ᆜ', 'ᅮ'] | ['ᅵ', 'ᅳ', 'ᅮ'] => ("", "ʏ"),
        ['ힻ'] => ("", "ɨe"),  // ㅡ+ㅔ
        _ => return None,
    })
}

fn coda(c: char) -> Option<&'static str> {
    Some(match c {
        'ᆨ' => "k",
        'ᆫ' => "n",
        'ᇈ' => "ɳ", // 종성 ㄴㅿ: 권설 비음 (산스크리트 ण ṇ; jaṇ = 저ᇈ — 사용자 지정 2026-11-24). 이 ㅿ 은 받침 ㄴ 의 표지
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
        'ᆶ' if mark(MARK_EU) => "ɽ", // ㅡ → ḥ 스위치: ㄹㅎ = ɽʱ (숨은 `syllable_oral` 에서 붙인다)
        'ᆶ' => "ʀ", // 종성 ㅀ: 목젖 전동음
        'ᇐ' => "r", // 종성 ㄹㄹ: 전동 r
        'ퟛ' if mark(MARK_EU) => "ɽ", // ㅡ → ḥ 스위치: ㄹㆁ = ɽ (권설 탄음)
        'ퟛ' => "r", // 종성 가벼운 ㄹ (ᅟᆞퟛ): 전동 r
        // 종성 ㄹㅿ: 권설 설측음 ɭ (ळ). 스위치와 상관없는 정음의 일반 규칙이다 — 종성 ㅿ 은 그 음절을 권설음화하고,
        // 여기서는 받침 ㄹ 에 얹힌다 (ㄴㅿ ᇈ = ɳ 과 같이; 사용자 지정 2026-11-27)
        'ᇗ' => "ɭ",
        'ᇶ' => "ʁ", // 종성 ㅎㄹ: 목젖 마찰음, 뒤에 ㄹ 을 받을 음절이 없는 프랑스어 r (merci 메ᇶ씨 — 사용자 지정 2026-11-24)
        'ᇂ' => "h", // 종성 ㅎ: 무성 h. 중성에 ㆍ 가 있으면 인두음 ʕ (`word_syllables`)
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

/// 받침 ㆁ(ᇰ)은 프랑스어 비모음의 표지다 (사용자 지정 2026-11-24). 한글도 正音도 받침 자리의 [ŋ]은 ㅇ(ᆼ)으로
/// 적으므로, 받침 ㆁ 은 프랑스어 비모음을 적을 때만 온다. 그래서 중성 + 받침 ㆁ 은 모음과 받침으로 나누어 읽지
/// 않고 한 묶음으로 비모음 하나에 잇는다:
///   ᅟᅥᇰ / ᅟힺᇰ [ɒ̃] (an en: 파리 발음) · ᅟᅡᇰ / ᅟᅢᇰ [ɛ̃] (in ain) · ᅟᅩᇰ [ɔ̃] (on) · ᅟᆟᇰ [œ̃] (un) · ᅟᅪᇰ [wɛ̃] (oin).
/// 묶음이므로 모음 글자의 다른 규칙(ㅓ ㅗ 의 받침 규칙 등)은 받지 않는다. 이 다섯 밖의 모음 + ㆁ 은 正音에 없는
/// 표기이고, 읽지 않는다 (`syllable_marked`).
fn nasal_vowel(v: &[char]) -> Option<(char, &'static str)> {
    // (자음 규칙을 받지 않을 대리 모음 글자, 비모음의 바탕 모음)
    Some(match v {
        // ㅡ결합 ힺ 도 목젖음 규칙(ᄏힺ = qʰ)을 받지 않게 대리 글자 ㅓ 로 읽는다: quand ᄏힺᇰ = 커ᇰ [kɒ̃]
        ['ᅥ'] | ['ힺ'] => ('ᅥ', "ɒ"),
        ['ᅡ'] => ('ᅡ', "ɛ"),
        ['ᅢ'] => ('ᅢ', "ɛ"),
        ['ᅩ'] => ('ᅩ', "ɔ"),
        ['ᆟ'] => ('ᅩ', "œ"),
        ['ᅪ'] => ('ᅪ', "ɛ"),
        _ => return None,
    })
}

/// ᅼ ᆣ 에서 유성 파열·파찰음이 아닌 자음 뒤에 붙이는 짧은 [h] 의 길이 (s)
const BREATH_AFTER: f64 = 0.05;
/// 구절 끝의 ᅼ ᆣ 음절이 모음 뒤로 내쉬는 숨의 길이 (s): UBC 낱음절 녹음의 70–80 ms
const BREATH_PHRASE_END: f64 = 0.075;

fn syllable(w: &Written) -> Option<Syllable> {
    let mut s = syllable_marked(w)?;
    // 모음사각도 매칭 (사용자 지정 2026-11-23): ㆆ 은 ㅣ 의 긴장·이완을 가르는 표지다 —
    // ᅙᅵ = ᅟᅵᇹ = [ɪ]. 이때 ㆆ 은 [ʔ] 로 나지 않는다. ㅔ 는 어디서나 [e̞] (ᅙᅦ = [ʔe̞], 사용자 지정)
    let glottal_lead = w.lead == Some('ᅙ') && s.onset.first().is_some_and(|o| o.ipa == "ʔ");
    match (w.vowel.as_slice(), glottal_lead, w.tail.as_slice()) {
        (['ᅵ'], true, _) => {
            s.onset.remove(0);
            s.nucleus.ipa = "ɪ".into();
        }
        (['ᅵ'], false, ['ᇹ']) if s.coda.last().is_some_and(|c| c.ipa == "ʔ") => {
            s.coda.pop();
            s.nucleus.ipa = "ɪ".into();
        }
        _ => {}
    }
    // ᅟᆗ (초성 없이): [ʔʊ] (사용자 지정 2026-11-23)
    if w.lead.is_none() && w.vowel == ['ᆗ'] && s.onset.is_empty() {
        s.onset.push(Segment::new("ʔ"));
    }
    Some(s)
}

fn syllable_marked(w: &Written) -> Option<Syllable> {
    // 블록 + 낱글자 ㅡ (베ㅡ 배ㅡ; 데바나가리 भे 의 표기에서 나온 꼴, 사용자 지정 2026-11-27). 한국어의 [pe̞ɯ] 는 '베으' 로
    // 적으므로 이 꼴은 가로지른 ㅡ (ᅼ ᆣ ᆜ) 와 같은 규칙으로 읽는다: `crossed`.
    if let [v @ ('ᅦ' | 'ᅢ' | 'ᅩ'), 'ᅳ'] = w.vowel.as_slice()
        && w.tail.is_empty()
    {
        let mut base = syllable_marked(&Written { text: w.text.clone(), lead: w.lead, vowel: vec![*v], tail: Vec::new() })?;
        // ㅗㅡ = [oʊ] (사용자 지정 2026-11-27): 오ㅡ [oʊ]. 초성과 스위치에 상관없다.
        if *v == 'ᅩ' {
            base.nucleus = Segment { nasalized: base.nucleus.nasalized, ..Segment::new("oʊ") };
            return Some(base);
        }
        // 스위치가 켜진 유성 유기음 뒤에서 ㅔ 는 [e] (데바나가리로 넣은 भे [bʱe] 와 같다)
        let plain = if *v == 'ᅦ' { "e" } else { "ɛ" };
        return crossed(w, base, plain, false);
    }
    let nasal = w.tail.contains(&'ᇰ');
    let french = if nasal { nasal_vowel(&w.vowel) } else { None };
    // 받침 ㆁ 은 프랑스어 비모음 다섯 묶음에만 쓴다 (사용자 지정 2026-11-24). 그 밖의 모음 + ㆁ 은 正音에 없는
    // 표기라 소리를 지어내지 않는다: 읽지 않는다.
    if nasal && french.is_none() {
        return None;
    }
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
    // ㅡ 를 뒤에 품은 결합모음(ᅼ ᆣ: ㅓ ㅏ → ㅡ)은 그 모음 + 기식: 산스크리트 유성 유기음 (사용자 지정).
    // ㄱ ㄷ ㅂ ㅈ (권설 ㄷ+ᇫ 포함) 뒤에서는 유성음이 숨 섞인 개방으로 터지고(엔진의 dʱ) 모음은 보통으로 —
    // ᄀᅼ gʱɐ · ᄌᅼ dʑʱɐ · ᄃᅼ dʱɐ · ᄃᅼᇫ ɖʱɐ · ᄇᅼ bʱɐ. 그 밖의 자음 뒤는 짧은 [h], 홀로는 [ɦ].
    // 구절 끝에서는 모음 뒤에도 숨이 따른다 (`phrase_final_breath`).
    // ᆜ: 기본은 [iɯ] (아래), [ɨ] 는 ᅙᆜ 로 적거나 ㅣ → ɨ 스위치를 켠다.
    let eu = mark(MARK_EU);
    // ㅡ → ḥ 스위치: 초성 ㅊ + 모음 + 종성 ㆆ (처ᇹ, 데바나가리 च 의 표기) 은 기식 없는 [tɕ˭] + 그 음절의 모음이다
    // (사용자 지정 2026-11-27). ㆆ 은 소리 나지 않지만 받침으로는 센다: 모음은 받침 있는 음절의 값이라 (ㅓ = ɤ̞,
    // ㅣ = ɪ), 같은 자음의 쩌 [tɕ˭ʌ] 와 모음으로 갈린다 (처ᇹ [tɕ˭ɤ̞]). 그것이 이 표기의 뜻이다.
    if eu && w.lead == Some('ᄎ') && w.tail == ['ᇹ'] && s.onset.first().is_some_and(|f| f.ipa == "tɕ") {
        s.onset[0] = Segment { dur: s.onset[0].dur, aspirated: Some(false), ..Segment::new("tɕ") };
        s.coda.clear();
    }
    if eu && let Some(first) = s.onset.first_mut() {
        // ㅡ → ḥ 스위치의 초성 (사용자 지정 2026-11-26). 다른 규칙이 이미 바꾼 초성(권설 ɖ dʐ ʂ, 목젖음 …)은 그대로다
        let v = match (w.lead, first.ipa.as_str()) {
            (Some('ᄀ'), "k") => Some("g"),
            (Some('ᄃ'), "t") => Some("d"),
            (Some('ᄇ'), "p") => Some("b"),
            (Some('ᄌ'), "tɕ") => Some("dʒ"),
            (Some('ᄒ'), "h") => Some("ɦ"),
            (Some('ᄉ'), "s" | "ʃ") => Some("ɕ"),
            (Some('ᄊ'), "ɕ") => Some("s"),
            _ => None,
        };
        if let Some(v) = v {
            *first = Segment { dur: first.dur, ..Segment::new(v) };
        }
    }
    // 가로지른 ㅡ (ᅼ ᆣ ᆜ: ㅓ ㅏ ㅣ 를 ㅡ 가 가로지른다): `crossed`
    let plain = match w.vowel.as_slice() {
        ['ᆜ'] => Some("i"),
        ['ᅼ'] => Some("ɐ"),
        ['ᆣ'] => Some("a"),
        _ => None,
    };
    if w.vowel == ['ᆜ'] && w.lead == Some('ᅙ') {
        s.nucleus.ipa = "ɨ".into();
    } else if w.vowel == ['ᆜ'] && mark(MARK_I) {
        // ㅣ → ɨ 스위치: ᆜ = [ɨ]. 초성은 ㅡ 앞에서처럼 (ᄉᆜ = sɨ: 구개음화하지 않는다)
        let mut hard = syllable_oral(&Written { text: w.text.clone(), lead: w.lead, vowel: vec!['ᅳ'], tail: w.tail.clone() })?;
        hard.nucleus = Segment { nasalized: s.nucleus.nasalized, ..Segment::new("ɨ") };
        s = hard;
    } else if let Some(plain) = plain {
        return crossed(w, s, plain, true);
    }
    Some(s)
}

/// 가로지른 ㅡ (ᅼ ᆣ ᆜ) 와 블록 + 낱글자 ㅡ (베ㅡ) 의 읽기 (사용자 지정 2026-11-27). `s` 는 ㅡ 없이 읽은 음절,
/// `plain` 은 그 모음이다. 초성의 갈래로 정한다:
///
///   | 초성                           | 꺼짐 (중립)                    | ㅡ → ḥ 켜짐 (인도·산스크리트)  |
///   |--------------------------------|--------------------------------|--------------------------------|
///   | ㄱ ㄷ ㅂ ㅈ                    | 유성 자음, 모음 + [ɤ]          | 유성 유기음 + 모음 (ᄇᅼ [bʱɐ]) |
///   | ㅸ                             | 모음 + [ɤ]                     | 모음 + [ɤ]                     |
///   | ㅋ ㅌ ㅍ ㅊ ㆄ                 | 모음 + [ɤ̤]                    | 모음 + [ɤ̤]                    |
///   | 정음 sonorant ㄴ ㄹ ㅁ ㅿ ㆁ ᄛ | 모음 + [ɝ]                     | 모음 + 끝의 [ɦ]                |
///   | ㅅ ㅎ                          | 모음 + [ɤ̤] (ᄉᆜ [ɕiːɤ̤])        | 모음 + [h] (ᄉᆜ [ɕih], ᄒᅼ [ɦɐh]) |
///   | 초성 없음                      | ᆜ [iɯ], ᅼ [ɐɤ], ᆣ [aɤ]        | 모음 + [ɤ̤]                    |
///   | ㄲ ㄸ ㅃ ㅉ ㅆ                 | 모음 + [ɤ]                     | 모음 + [ɤ̤]                    |
///
/// 꺼짐은 중립이다: ᅼ ᆣ 는 광둥어·북경어 등의 표기에도 쓰인다. 켜짐의 아래 두 줄은 말이 없어 전과 같이 두었다.
/// `whole` 은 한 글자 (ᅼ ᆣ ᆜ) 일 때 참. 블록 + ㅡ (세ㅡ 에ㅡ) 는 꺼짐에서 ㅅ 도 모음 + [ɤ] 이고, 표에 없는 초성은 읽지 않는다.
fn crossed(w: &Written, mut s: Syllable, plain: &str, whole: bool) -> Option<Syllable> {
    let on = mark(MARK_EU);
    let set = |s: &mut Syllable, v: String| s.nucleus = Segment { nasalized: s.nucleus.nasalized, ..Segment::new(&v) };
    let first = s.onset.first().map(|f| f.ipa.clone()).unwrap_or_default();
    // 풀리기 전의 모음: 한 글자 (ᅼ ᆣ ᆜ) 는 받침과 상관없이 ɐ a i, 블록 + ㅡ 는 그 블록의 모음 (ㅔ e̞, ㅐ ɛ)
    let v0 = if whole { plain.to_string() } else { s.nucleus.ipa.clone() };
    // ㄱ ㄷ ㅂ ㅈ (권설 ㄷ+ᇫ 포함) 의 유성음
    let voiced = match (w.lead, first.as_str()) {
        (Some('ᄀ'), "k" | "g") => Some("g"),
        (Some('ᄃ'), "t" | "d") => Some("d"),
        (Some('ᄃ'), "ɖ") => Some("ɖ"),
        (Some('ᄇ'), "p" | "b") => Some("b"),
        (Some('ᄌ'), "tɕ" | "dʑ") => Some("dʑ"),
        (Some('ᄌ'), "dʒ") => Some("dʒ"),
        _ => None,
    };
    match w.lead {
        Some('ᄀ' | 'ᄃ' | 'ᄇ' | 'ᄌ') if voiced.is_some() => {
            let v = voiced?;
            if on {
                s.onset[0] = Segment { breathy: Some(true), dur: s.onset[0].dur, ..Segment::new(v) };
                set(&mut s, plain.to_string());
            } else {
                s.onset[0] = Segment { dur: s.onset[0].dur, ..Segment::new(v) };
                set(&mut s, format!("{v0}ɤ"));
            }
        }
        // ㅸ 은 켜져도 꺼져도 모음 + [ɤ], ㆄ 과 ㅋ ㅌ ㅍ ㅊ 은 모음 + [ɤ̤]
        Some('ᄫ') => set(&mut s, format!("{v0}ɤ")),
        Some('ᄏ' | 'ᄐ' | 'ᄑ' | 'ᄎ' | 'ᅗ') => set(&mut s, format!("{v0}ɤ̤")),
        Some('ᄂ' | 'ᄅ' | 'ᄆ' | 'ᅀ' | 'ᅌ' | 'ᄛ') => {
            if on {
                set(&mut s, plain.to_string());
                s.coda.insert(0, Segment { dur: Some(BREATH_AFTER), ..Segment::new("ɦ") });
            } else {
                set(&mut s, format!("{v0}ɝ"));
            }
        }
        // ㅅ ㅎ + 가로지른 ㅡ (사용자 지정 2026-11-28). 꺼짐: 모음 + [ɤ̤] (ᄉᆜ = [ɕiːɤ̤]: 긴 [i] 에 짧은 [ɤ̤]. 이중모음의
        // 뒤쪽은 본래 짧게 (모음의 1/5) 나므로 길이를 따로 정하지 않는다). 켜짐: 모음 + [h] (ᄉᆜ = [ɕih], ᄒᅼ = [ɦɐh]).
        // 앞서 ᄉᆜ 의 [i] 뒤 쉼을 [ʔ], [h], 숨의 압력을 내리는 쉼, 비음 [ĩ] 로 내 보았으나 모두 버렸다.
        Some('ᄉ' | 'ᄒ') if whole => {
            if on {
                set(&mut s, plain.to_string());
                s.coda.insert(0, Segment::new("h"));
            } else {
                set(&mut s, format!("{v0}ɤ̤"));
            }
        }
        // 초성 없는 ᆜ = [iɯ] (15세기 음가)
        None | Some('ᄋ') if !on && whole && plain == "i" => set(&mut s, "iɯ".to_string()),
        // 초성 없음, ㅎ, ㄲ ㄸ ㅃ ㅉ ㅆ, 그리고 블록 + ㅡ 의 ㅅ (세ㅡ): 모음 + [ɤ]
        None | Some('ᄋ' | 'ᄒ' | 'ᄁ' | 'ᄄ' | 'ᄈ' | 'ᄍ' | 'ᄊ' | 'ᄉ') if !on => set(&mut s, format!("{v0}ɤ")),
        // 켜짐에서 블록 + ㅡ 도 한 글자와 같이 모음 + [ɤ̤] (말이 없어 표의 값에 맞췄다)
        None | Some('ᄋ' | 'ᄒ' | 'ᄁ' | 'ᄄ' | 'ᄈ' | 'ᄍ' | 'ᄊ' | 'ᄉ') => set(&mut s, format!("{v0}ɤ̤")),
        _ if !whole => return None,
        _ if !on && plain == "i" => set(&mut s, "iɯ".to_string()),
        _ => set(&mut s, format!("{v0}ɤ̤")),
    }
    Some(s)
}

fn syllable_oral(w: &Written) -> Option<Syllable> {
    // 홀로 쓴 자음 낱자 (사용자 지정 2026-11-24). ㅋ ㅌ ㅅ ㅎ 은 낱자로 쓰면 모음 없이 자음 [k t s h] 로만 읽는다:
    // 듸ᇫㅅ = [ðɪs] (this). 모음이 없으니 제 음절을 이루지 않고 이웃 음절에 붙는다 (드ᇫ 트ᇫ 처럼). ㅡ 를 적은
    // 스 는 [sɤ] 그대로다 (버스). ㆄ ㅸ 은 주로 영어의 f v 를 적을 때 나오므로 [ə] 를 붙인다 (ㅸ = [βə]).
    // 그 밖의 자음은 낱자로 쓰이는 일이 없고, 쓰이면 [ə].
    if w.vowel.is_empty() && w.tail.is_empty() && let Some(l) = w.lead {
        let (ipa, a, v) = onset(l)?;
        let nucleus = if matches!(l, 'ᄏ' | 'ᄐ' | 'ᄉ' | 'ᄒ') { "" } else { "ə" };
        return Some(Syllable { text: w.text.clone(), onset: vec![Segment { aspirated: a, vot: v, ..Segment::new(ipa) }], nucleus: Segment::new(nucleus), ..Default::default() });
    }
    // 모음 없이 종성만 있는 음절: 성절 자음 (ᅟᅠᆫ = 嗯 n, ᅟᅠᆼ = ng, ᅟᅠᇢ = 呣 m 을 ㅱ=[ʋ]로)
    if w.vowel.is_empty() && let Some((first, rest)) = w.tail.split_first() {
        let nucleus = coda(*first)?;
        let on: Vec<Segment> = w.lead.and_then(onset).map(|(ipa, a, v)| Segment { aspirated: a, vot: v, ..Segment::new(ipa) }).into_iter().collect();
        let cd = rest.iter().filter_map(|&t| coda(t)).map(Segment::new).collect();
        return Some(Syllable { text: w.text.clone(), onset: on, nucleus: Segment::new(nucleus), coda: cd, ..Default::default() });
    }
    // ᅌᅥ = [əɝ], ᅌᅥᇫ = [ɻ] (사용자 지정 2026-11-27): 초성 ㆁ · 중성 ㅓ 는 중국어 er 의 소리 (ə 에서 r 물든 모음으로
    // 미끄러진다), 받침이 붙어도 그렇다 (ᅌᅥᆫ = əɝn). 종성 ㅿ 하나만 붙은 ᅌᅥᇫ 은 통째로 [ɻ] 이고, 자음만인 음절이라
    // 이웃 음절에 붙는다 (우안ᅌᅥᇫ = 받침 ɻ: 중국어 儿화의 표기). 붙을 음절이 없으면 제 음절 [əɝ] 로 난다 (`word_syllables`).
    if w.lead == Some('ᅌ') && w.vowel == ['ᅥ'] {
        if w.tail == ['ᇫ'] {
            return Some(Syllable { text: w.text.clone(), onset: vec![Segment::new("ɻ")], nucleus: Segment::new(""), ..Default::default() });
        }
        if w.tail.is_empty() {
            return Some(Syllable { text: w.text.clone(), nucleus: Segment::new("əɝ"), ..Default::default() });
        }
        // 받침은 초성 없는 ㅓ 음절의 것 그대로, 모음만 [əɝ]
        let mut s = syllable_oral(&Written { text: w.text.clone(), lead: None, vowel: w.vowel.clone(), tail: w.tail.clone() })?;
        s.nucleus = Segment { nasalized: s.nucleus.nasalized, ..Segment::new("əɝ") };
        return Some(s);
    }
    // 인두 마찰음 표기 (주로 아랍어): ᄒᆞᇹ = ħ, ᅙᆞᇂ = ʕ. 중성 ㆍ는 자리표시가 아니라 이 표기의 구성요소이므로,
    // ᅙᆞ = ə 처럼 초성·ㆍ·종성 세 자를 한 단위로 읽고 따로 음가를 매기지 않는다.
    // 자음만인 음절이라 다음 음절 첫소리(또는 앞 음절 받침)로 붙는다: `word_syllables`
    // 듷 = ðˤ, ᅀᅳᇂ = zˤ (사용자 지정 2026-11-26): 초성·중성 ㅡ·종성 ㅎ 세 자가 꼭 이 조합일 때만. 엔진에 인두음화
    // 기호(ˤ)가 없어 그 자음 뒤에 인두음 [ʕ] 를 이어 낸다. 자음만인 음절이라 이웃 음절에 붙는다.
    if let (Some(l @ ('ᄃ' | 'ᅀ')), ['ᅳ'], ['ᇂ']) = (w.lead, w.vowel.as_slice(), w.tail.as_slice()) {
        let c = if l == 'ᄃ' { "ð" } else { "z" };
        return Some(Syllable { text: w.text.clone(), onset: vec![Segment::new(c), Segment::new("ʕ")], nucleus: Segment::new(""), ..Default::default() });
    }
    // ᄒᆞᇹ = ħ 에 모음이 붙은 꼴 (사용자 지정 2026-11-26): ᄒퟅᇹ = ħ + ퟅ. ㆍ 결합 모음에서 ㆍ 는 표기의 구성요소이고
    // 나머지 모음이 소리 난다; 종성 ㆆ 은 표지라 소리 나지 않는다.
    if w.lead == Some('ᄒ') && w.tail == ['ᇹ'] && matches!(w.vowel.as_slice(), ['ᆟ'] | ['ᆠ'] | ['ᆡ'] | ['ᆢ'] | ['ퟅ'] | ['ퟆ']) {
        let mut s = syllable_oral(&Written { text: w.text.clone(), lead: w.lead, vowel: w.vowel.clone(), tail: Vec::new() })?;
        if let Some(first) = s.onset.first_mut() {
            *first = Segment::new("ħ");
        }
        return Some(s);
    }
    let whole = match (w.lead, w.vowel.as_slice(), w.tail.as_slice()) {
        (Some('ᄒ'), ['ᆞ'], ['ᇹ']) => Some("ħ"),
        (Some('ᅙ'), ['ᆞ'], ['ᇂ']) => Some("ʕ"),
        _ => None,
    };
    if let Some(c) = whole {
        return Some(Syllable { text: w.text.clone(), onset: vec![Segment::new(c)], nucleus: Segment::new(""), ..Default::default() });
    }
    // 떨어진 ㅡ: ㅡ 와 다음 모음을 두 글자로 이어 쓴 것 (ᄀ+ᅳ+ᅡ, 그ㅏ).
    //   사용자 지정 2026-11-24 (11-22 의 배정을 맞바꿈): [ga] 를 쓸 일이 [ɢa] [qa] 보다 훨씬 흔하므로 붙은 결합모음
    //   ᄀힹ 이 [ga] (책의 일본어 탁음 표기 그대로), 떨어진 그ㅏ 가 목젖음 [ɢa], 끄ㅏ 가 [qa] 다 (크ㅏ qʰ, 흐ㅏ χ 도 같이).
    //   ᄃ ᄇ 은 목젖 짝이 없어 떨어져도 붙어도 유성 [d] [b].
    if let Some(l @ ('ᄀ' | 'ᄁ' | 'ᄏ' | 'ᄒ' | 'ᄃ' | 'ᄇ')) = w.lead
        && let ['ᅳ', rest @ ..] = w.vowel.as_slice()
        && !rest.is_empty()
    {
        let (glide, nucleus) = vowel(rest, false, coda_sonorant(&w.tail))?;
        let mut on = vec![match l {
            'ᄀ' => Segment::new("ɢ"),
            'ᄁ' => Segment { aspirated: Some(false), ..Segment::new("q") },
            'ᄏ' => Segment { aspirated: Some(true), vot: Some(0.08), ..Segment::new("q") },
            'ᄒ' => Segment::new("χ"),
            'ᄃ' => Segment::new("d"),
            _ => Segment::new("b"),
        }];
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
    // ㅈ 도 같다 (사용자 지정 2026-11-24): ㅡ 가 유성 표지라 ᄌힺ = [dʑ] + ㅓ (산스크리트 ja), ᄌힹ = [dʑ] + ㅏ (jā).
    // 책과 hun-min.com 의 '저' 는 jha (ᄌᅼ) 를 적을 자리를 남기려고 ᄌힺ 을 줄여 쓴 것이다. 받침에 ㅿ 이 있으면
    // 권설 규칙 (dʐ) 으로 간다.
    if let Some(l @ ('ᄃ' | 'ᄇ' | 'ᄌ')) = w.lead
        && !(l == 'ᄃ' && w.tail.contains(&'ᇫ'))
        && !(l == 'ᄌ' && (w.tail.contains(&'ᇫ') || w.tail.contains(&'ᇈ') || w.tail.contains(&'ᇗ')))
        && let [first, rest @ ..] = w.vowel.as_slice()
        && *first != 'ᅴ'
        && let Some(base) = eu_base(*first)
    {
        let mut v = vec![base];
        v.extend_from_slice(rest);
        let (glide, nucleus) = vowel(&v, false, coda_sonorant(&w.tail))?;
        let mut on = vec![Segment::new(match l {
            'ᄃ' => "d",
            'ᄇ' => "b",
            _ => "dʑ",
        })];
        if !glide.is_empty() {
            on.push(Segment::new(glide));
        }
        let cd = w.tail.iter().filter_map(|&t| coda(t)).map(Segment::new).collect();
        return Some(Syllable { text: w.text.clone(), onset: on, nucleus: Segment::new(nucleus), coda: cd, ..Default::default() });
    }
    // ᆕ(ㅡ+ㅜ = ʉ)는 예외: 그 자체가 모음 [ʉ]라 목젖음 규칙을 받지 않는다 (ᄀᆕ ᄏᆕ ᄒᆕ = 일본어 ぐ く ふ, 사용자 지정)
    if let Some(l @ ('ᄁ' | 'ᄀ' | 'ᄏ' | 'ᄒ')) = w.lead
        // ㅎ + ㅢ 는 목젖 규칙을 받지 않는다 (사용자 지정 2026-11-27): 희 = [hi] (한국어 처리기와 같다; [χi] 도 [hɰi] 도 아니다)
        && !(l == 'ᄒ' && w.vowel == ['ᅴ'])
        && let [first, rest @ ..] = w.vowel.as_slice()
        && *first != 'ᆕ'
        && let Some(base) = eu_base(*first)
    {
        let mut v = vec![base];
        v.extend_from_slice(rest);
        let (glide, nucleus) = vowel(&v, false, coda_sonorant(&w.tail))?;
        let uvular = match l {
            'ᄁ' => Segment { aspirated: Some(false), ..Segment::new("q") },
            // ㄱ + 붙은 ㅡ결합 모음은 유성 [g] (ᄀힹ = ga): ㅡ 가 유성 표지, ㄷ ㅂ ㅈ 과 같다. 목젖음 [ɢ] 은 떨어진 그ㅏ
            'ᄀ' => Segment::new("g"),
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
        // ㄷㅡㅿ · ㅌㅡㅿ 은 ð θ 를 불러내는 세 자리 code 다: 여기의 ㅿ 은 받침이 아니므로 모음의 받침 규칙을 따질 때
        // 뺀다 (권설 표지인 받침 ㅿ 과도 다르다). 이 조합에 붙은 ㅣ 는 [ɪ] (듸ᇫ = 드ᇫ + ㅣ = [ðɪ], this;
        // 사용자 지정 2026-11-24)
        let tail: Vec<char> = w.tail.iter().copied().filter(|&t| t != 'ᇫ').collect();
        let (glide, mut nucleus) = vowel(&v, false, coda_sonorant(&tail))?;
        if v == ['ᅵ'] {
            nucleus = "ɪ";
        }
        let mut on = vec![Segment::new(if l == 'ᄃ' { "ð" } else { "θ" })];
        if !glide.is_empty() {
            on.push(Segment::new(glide));
        }
        let cd = w.tail.iter().filter(|&&t| t != 'ᇫ').filter_map(|&t| coda(t)).map(Segment::new).collect();
        return Some(Syllable { text: w.text.clone(), onset: on, nucleus: Segment::new(nucleus), coda: cd, ..Default::default() });
    }
    let son = sonorant(w.lead);
    let (mut glide, mut nucleus) = vowel(&w.vowel, son, coda_sonorant(&w.tail))?;
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
            // ㅢ after a consonant is [i] (희 → 히, 늬 → 니): 한국어 처리기의 규칙 그대로 (초성이 있으면 [i], 없으면 [ɰi])
            glide = "";
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
        // 받침에 ㅿ 이 들어 있으면 (ᇫ, 또는 ㄴㅿ ᇈ) 그 음절이 권설음이다: 끄셔ᇈ아 = kṣaṇa 의 셔ᇈ 은 [ʂ…ɳ]
        // (hun-min.com/?p=235; 사용자 지정 2026-11-24)
        // ㄹㅿ ᇗ 도 같다 (사용자 지정 2026-11-27): 사ᇗ = [ʂaɭ]
        let retro = w.tail.contains(&'ᇫ') || w.tail.contains(&'ᇈ') || w.tail.contains(&'ᇗ');
        // ㅅ ㅆ 은 ㅣ·j 앞에서 ɕ (시 = 중국어 xi)
        if matches!(l, 'ᄉ' | 'ᄊ') && (glide == "j" || nucleus == "i") && ipa == "s" {
            ipa = "ɕ";
        }
        // 초성 ㄹ + 종성 ㅿ = ɽ (권설 탄음): 종성 ㅿ는 초성의 권설을 표시하고 따로 소리 나지 않는다
        if l == 'ᄅ' && retro {
            ipa = "ɽ";
        }
        // 초성 ㄴ ㅅ ㅈ ㅊ + 종성 ㅿ (사용자 지정 2026-11-24): ㄴ = ɳ, ㅅ = ʂ, ㅈ = dʐ, ㅊ = tʂ.
        // 권설이 된 ㅈ 은 유성 [dʐ] 다: 초성 ㅈ + 종성 ㅿ 은 ㅈ 이 유성음인 폴란드어 (dż 제ᇫ) · 산스크리트 · 힌디어를
        // 적을 때 나온다. 북경어 (zh 즤) 는 종성에 ㅿ 을 쓰지 않으므로 겹치지 않는다. ㅊ 은 그 무성 짝 (폴란드어 cz 체ᇫ).
        if retro {
            match l {
                'ᄂ' => ipa = "ɳ",
                'ᄉ' | 'ᄊ' => ipa = "ʂ",
                'ᄌ' => (ipa, aspirated, vot) = ("dʐ", None, None),
                'ᄎ' => ipa = "tʂ",
                _ => {}
            }
        }
        // 초성 ㅌ ㄸ ㄷ + 종성 ㅿ: 종성 ㅿ는 초성을 바꾸는 표지이고 따로 소리 나지 않는다.
        //   ㅌ+ㅿ = ʈʰ (권설 유기 파열음), ㄸ+ㅿ = ʈ (권설 파열음), ㄷ+ㅿ = ɖ (권설 유성 파열음).
        //   치 마찰음은 모음에 ㅡ 가 있어야 한다: 트ᇫ = θ, 드ᇫ = ð.
        //   자음만 나타내는 표기: 트ᇫ = θ, 드ᇫ = ð (모음 ㅡ), ᄃᆞᇫ = ɖ, ᄄᆞᇫ = ʈ (모음 ㆍ, ㆍ는 표기의 구성요소).
        //   뜨ᇫ 도 ʈ 단독으로 읽는다.
        if matches!(l, 'ᄐ' | 'ᄄ' | 'ᄃ') && retro {
            let eu = w.vowel == ['ᅳ'];
            let arae = w.vowel == ['ᆞ'] && matches!(l, 'ᄃ' | 'ᄄ');
            (ipa, aspirated, vot) = match l {
                // θ 는 ㅌ + ㅡ + ㅿ 세 성분이 다 있을 때만 (트ᇫ; ㅡ결합 모음 ᄐힺᇫ = θ + ㅓ 는 위에서).
                // ㅡ 없는 ㅌ + ㅿ (타ᇫ)는 받침 ㅿ 의 본뜻대로 권설: 기식 있는 ʈ (사용자 지정 2026-11-24)
                'ᄐ' if eu => ("θ", None, None),
                'ᄐ' => ("ʈ", aspirated, vot),
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
        // 된소리 = 자음 + ㆆ (사용자 지정 2026-11-26): ㄲ = kʔ, ㄸ = tʔ, ㅃ = pʔ, ㅉ = tɕʔ. 기식 없는 파열음을 터뜨리고
        // 성문을 닫았다 모음으로 연다. ㄲ ㄸ 이 다른 자음의 표기로 쓰인 자리 (ㄲ+j = c, ㄲ+ㅡ결합 = q, ㄸ+ㅿ = ʈ)는
        // 그 자음 그대로다.
        // 인두 표지 음절 (ᄄᆞᇂ = tˤ) 의 ㄸ 도 그 표기의 구성요소라 그대로다.
        // ㅡ → ḥ 스위치가 켜지면 ㄲ ㄸ ㅃ ㅉ 은 ㆆ 없이 기식 없는 [k˭ t˭ p˭ tɕ˭] 다 (사용자 지정 2026-11-27: 데바나가리
        // क त प 를 꺼 떠 뻐 로 적는다). च 의 표기는 초성 ㅊ + 종성 ㆆ (처ᇹ): `syllable_marked`.
        let plain = mark(MARK_EU) && matches!(l, 'ᄁ' | 'ᄄ' | 'ᄈ' | 'ᄍ');
        if !plain && matches!((l, ipa), ('ᄁ', "k") | ('ᄄ', "t") | ('ᄈ', "p") | ('ᄍ', "tɕ")) && !(w.tail.contains(&'ᇂ') && arae_vowel(w)) {
            on.push(Segment::new("ʔ"));
        }
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
            // 받침 ㅿ 은 그 음절을 권설음화한다 (사용자 지정 2026-11-24; IAST 의 아래점). 혀끝을 말아 올리는 동작이
            // 얹히는 곳은 초성에 달렸다:
            //   · 혀끝 초성 ㄷ ㄸ ㅌ ㄹ ㄴ ㅅ ㅈ ㅊ 은 초성이 권설음이 된다 (위에서).
            //   · ㅓ 는 r 물든 모음 (ᅟᅥᇫ = ɝ, 영어 ɜːr; ɜ 자체는 正音으로 정의하지 않는다 — 사용자 지정 2026-11-23).
            //   · 그 밖의 초성 (ㅍ ㅂ ㄱ ㅋ ㅁ ㅎ …, 초성 없음)은 혀끝을 쓰지 않으니 모음에 얹힌다: 모음 뒤에 권설
            //     접근음 [ɻ] (파ᇫ = [pʰaɻ], 영어 par 의 모음 뒤 r).
            let coronal = matches!(w.lead, Some('ᄂ' | 'ᄃ' | 'ᄄ' | 'ᄅ' | 'ᄉ' | 'ᄊ' | 'ᄌ' | 'ᄍ' | 'ᄎ' | 'ᄐ' | 'ᅀ' | 'ᄙ' | 'ᄛ'));
            if !coronal && matches!(nucleus, "ʌ" | "ɤ̞") {
                nucleus = "ɝ";
            } else if !coronal {
                cd.push(Segment::new("ɻ"));
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
            if t == 'ᆶ' && mark(MARK_EU) {
                // ɽʱ: 엔진의 숨 섞인 개방은 파열음·파찰음에만 있어, 탄음 뒤에 짧은 [ɦ] 로 낸다
                cd.push(Segment { dur: Some(BREATH_AFTER), ..Segment::new("ɦ") });
            }
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

/// 중성에 ㆍ 가 든 음절 (한글은 ㆍ 를 쓰지 않으니 正音 표기다)
fn arae_vowel(w: &Written) -> bool {
    w.vowel.iter().any(|v| matches!(v, 'ᆞ' | 'ᆟ' | 'ᆠ' | 'ᆡ' | 'ᆢ' | 'ퟅ' | 'ퟆ'))
}

/// 받침 ㅎ 이 소리 [h] 가 아니라 인두 표지인 음절: 중성에 ㆍ 가 든 꼴 (ᅟᆞᇂ: ᅙᆞᇂ = ʕ, ᄄᆞᇂ = tˤ, ᄄퟅᇂ = tˤɑ),
/// 그리고 ㅡ 로 적는 ðˤ zˤ (듷 ᅀᅳᇂ ᄃᆕᇂ). 한글의 ㅎ 거센소리 되기를 적용하지 않는다.
fn pharyngeal_mark(w: &Written) -> bool {
    let arae_a = arae_vowel(w);
    let eu = w.vowel.iter().any(|v| matches!(v, 'ᅳ' | 'ᆕ' | 'ᅴ' | 'ힹ' | 'ힺ' | 'ힻ' | 'ힼ'));
    arae_a || (eu && matches!(w.lead, Some('ᄃ') | Some('ᅀ')))
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
    // 받침 ㅎ 뒤에 낱자 ㄹ 을 따로 적은 꼴 = 겹받침 ㅎㄹ (멯ㄹ씨 = 메ᇶ씨): 낱자 ㄹ 은 제 음절을 이루지 않는다
    let mut i = 0;
    while i + 1 < ws.len() {
        let lone_l = ws[i + 1].lead == Some('ᄅ') && ws[i + 1].vowel.is_empty() && ws[i + 1].tail.is_empty();
        if lone_l && ws[i].tail == ['ᇂ'] && !pharyngeal_mark(&ws[i]) {
            let l = ws.remove(i + 1);
            ws[i].tail = vec!['ᇶ'];
            ws[i].text.push_str(&l.text);
        }
        i += 1;
    }
    // 인두 표지 음절의 ㅎㄹ 받침(ᅙퟅᇶ리 ʕalī)은 프랑스어 r 이 아니다: 아랍어 읽기를 넣기 전까지는 전처럼 읽지 않는다
    for w in ws.iter_mut() {
        if w.tail == ['ᇶ'] && pharyngeal_mark(w) {
            w.tail.clear();
        }
    }
    // 한글 발음 규칙 가운데 정음을 mini-IPA 로 쓰는 데 걸리지 않는 둘 (사용자 지정 2026-11-24). 글은 그대로 두고 소리만 바꾼다.
    for i in 0..ws.len().saturating_sub(1) {
        let (a, b) = ws.split_at_mut(i + 1);
        let (a, b) = (&mut a[i], &mut b[0]);
        if b.vowel.is_empty() {
            continue;
        }
        let front = matches!(b.vowel.as_slice(), ['ᅵ'] | ['ᅧ']);
        // ㅎ 거센소리 되기: 받침 ㅎ + ㄱ ㄷ ㅂ ㅈ, 받침 ㄱ ㄷ ㅂ ㅈ + ㅎ = ㅋ ㅌ ㅍ ㅊ (좋다 [조타], 입학 [이팍]).
        // 인두 표지인 받침 ㅎ 은 빼놓는다 (`pharyngeal_mark`), 인두음 ᄒᆞᇹ 의 초성 ㅎ 도.
        let aspirate = |c: char| match c {
            'ᄀ' | 'ᆨ' => Some('ᄏ'),
            'ᄃ' | 'ᆮ' => Some('ᄐ'),
            'ᄇ' | 'ᆸ' => Some('ᄑ'),
            'ᄌ' | 'ᆽ' => Some('ᄎ'),
            _ => None,
        };
        let merged = match (a.tail.as_slice(), b.lead) {
            (['ᇂ'], Some(l)) if !pharyngeal_mark(a) => aspirate(l),
            ([t], Some('ᄒ')) if b.tail != ['ᇹ'] => aspirate(*t),
            _ => None,
        };
        if let Some(m) = merged {
            // 닫히다 [다치다]: 받침 ㄷ 이 ㅎ 과 만나 ㅌ 이 되고, 이·여 앞이라 ㅊ
            let palatal = front && m == 'ᄐ' && a.tail == ['ᆮ'];
            a.tail.clear();
            b.lead = Some(if palatal { 'ᄎ' } else { m });
            continue;
        }
        // 받침 ㅎ + 초성 없는 모음: ㅎ 은 소리 나지 않는다 (좋아 [조아]). 형태주의 맞춤법에서만 나오는 꼴이다.
        // 인두 표지인 받침 ㅎ 은 그대로 둔다 (ᅙퟅᇂ이드: 아래의 모음 사이 ɦ 규칙이 읽는다)
        if a.tail == ['ᇂ'] && b.lead.is_none() && !pharyngeal_mark(a) {
            a.tail.clear();
            continue;
        }
        // 구개음화: 받침 ㄷ ㅌ + 초성 없는 이·여 = 지 치 (굳이 [구지], 같이 [가치], 붙여 [부쳐])
        if b.lead.is_none() && front {
            let moved = match a.tail.as_slice() {
                ['ᆮ'] => Some('ᄌ'),
                ['ᇀ'] => Some('ᄎ'),
                _ => None,
            };
            if let Some(m) = moved {
                a.tail.clear();
                b.lead = Some(m);
                continue;
            }
        }
        // 받침 ㅆ (사용자 지정 2026-11-26): 제 음절에서는 불파음 [t]. 다음 음절에 초성이 없으면 ㅅ 이 그 초성으로
        // 이어지고 (했어 [hɛt.sɤ̞]), 평음 ㄱ ㄷ ㅂ ㅈ 이 오면 된소리가 된다 (했었지 [hɛt.sɤ̞t.tɕʔi]).
        if a.tail == ['ᆻ'] {
            b.lead = match b.lead {
                None => Some('ᄉ'),
                Some('ᄀ') => Some('ᄁ'),
                Some('ᄃ') => Some('ᄄ'),
                Some('ᄇ') => Some('ᄈ'),
                Some('ᄌ') => Some('ᄍ'),
                other => other,
            };
            continue;
        }
        // 연음 (사용자 지정 2026-11-26, 한국어 처리기와 같이): 홑받침 + 초성 없는 모음 → 받침이 다음 음절 첫소리로
        // (먹어 [머거], 물이 [무리]). 받침 ㅇ [ŋ] 은 옮기지 않고, 받침 ㅅ 은 제자리에서 [s] 로 이어진다 (여섯은
        // [jʌ.sɤ̞s.ɤn]). 正音의 표지 받침 (ㆁ ㆆ ㅿ ㅱ ㅎ …) 은 연음하지 않는다.
        if b.lead.is_none() && let [t] = a.tail.as_slice() {
            let moved = match t {
                'ᆨ' => Some('ᄀ'),
                'ᆩ' => Some('ᄁ'),
                'ᆫ' => Some('ᄂ'),
                'ᆮ' => Some('ᄃ'),
                'ᆯ' => Some('ᄅ'),
                'ᆷ' => Some('ᄆ'),
                'ᆸ' => Some('ᄇ'),
                'ᆽ' => Some('ᄌ'),
                'ᆾ' => Some('ᄎ'),
                'ᆿ' => Some('ᄏ'),
                'ᇀ' => Some('ᄐ'),
                'ᇁ' => Some('ᄑ'),
                _ => None,
            };
            if let Some(m) = moved {
                a.tail.clear();
                b.lead = Some(m);
            }
        }
    }
    // 받침 ㅅ (사용자 지정 2026-11-26): 다음 음절이 초성 없는 모음일 때만 [s] 로 이어지고 (여섯은), 낱말 끝과 자음
    // 앞에서는 불파음 [t] (여섯 [jʌ.sɤ̞t]).
    for i in 0..ws.len() {
        let links = ws.get(i + 1).is_some_and(|b| b.lead.is_none() && !b.vowel.is_empty());
        if ws[i].tail == ['ᆺ'] && !links {
            ws[i].tail = vec!['ᇀ']; // [t] 로 나는 받침 (구개음화·거센소리 규칙은 이미 지났다)
        }
    }
    let ws = ws;
    let mut out: Vec<(usize, Syllable)> = ws.iter().enumerate().filter_map(|(i, w)| syllable(w).map(|s| (i, s))).collect();
    // 포르투갈어 비이중모음 (사용자 지정 2026-11-24): ᆟ + ᅌᅮ = [ɐ̃w] 한 묶음, 한 음절 (coração 꼬라ᄉᆟᅌᅮ
    // [koɾɐˈsɐ̃w]). 받침 ㆁ 의 다섯 묶음처럼 모음과 ㆁ 을 나누어 읽지 않는다: 두 블록이 한 음표를 받는다.
    let mut k = 0;
    while k + 1 < out.len() {
        let (i, j) = (out[k].0, out[k + 1].0);
        let (a, b) = (&ws[i], &ws[j]);
        if j == i + 1 && a.vowel == ['ᆟ'] && a.tail.is_empty() && b.lead == Some('ᅌ') && b.vowel == ['ᅮ'] && b.tail.is_empty() {
            let glide = out.remove(k + 1).1;
            let s = &mut out[k].1;
            s.nucleus = Segment { nasalized: Some(true), ..Segment::new("ɐ") };
            s.coda = vec![Segment::new("w")];
            s.text.push_str(&glide.text);
        }
        k += 1;
    }
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
            Some(t) => matches!(t, 'ᆫ' | 'ᆯ' | 'ᆷ' | 'ᆼ' | 'ᇰ' | 'ᇢ' | 'ᇙ' | 'ᇶ' | 'ᆭ' | 'ᆬ' | 'ᆲ' | 'ᆳ' | 'ᆴ' | 'ᆱ'),
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
    // 받침 ㅎ 은 모음 사이에서도 유성 [ɦ] 가 되지 않는다 (사용자 지정 2026-11-26: 이전 규칙 취소). 받침 ㅎ 의 쓰임은
    // 아랍어의 [중성 ㆍ + 종성 ㅎ] = ʕ (아래), 프랑스어의 [종성 ㅎ + 다음 초성 ㄹ] = ʁ 와 겹받침 ㅎㄹ ᇶ · ㄹㅎ ᆶ (위).
    // 중성에 ㆍ 가 든 음절의 받침 ㅎ = 인두음 [ʕ] (사용자 지정 2026-11-26: 알ᄅퟅᇂ아크바르). 모음 사이에서도 ㅎ 탈락·ɦ 가 아니다
    for (i, s) in out.iter_mut() {
        if ws[*i].tail.last() == Some(&'ᇂ') && arae_vowel(&ws[*i])
            && let Some(last) = s.coda.last_mut()
            && matches!(last.ipa.as_str(), "h" | "ɦ")
        {
            *last = Segment::new("ʕ");
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
                // (홀로 쓰인 ᅌᅥᇫ [ɻ] 은 제 음절 [əɝ] 로 난다)
                syls[i].nucleus = cons.first().cloned().map(|c| if c.ipa == "ɻ" { Segment::new("əɝ") } else { c }).unwrap_or_else(|| Segment::new("ə"));
            }
        }
        i += 1;
    }
    // ɪ 는 언제나 [ɪʔ] (사용자 지정 2026-11-27): 짧게 끊어 내기 위해 모음 바로 뒤에 성문 폐쇄가 온다 (입 [ɪʔp])
    for s in &mut syls {
        if s.nucleus.ipa == "ɪ" && s.coda.first().is_none_or(|c| c.ipa != "ʔ") {
            s.coda.insert(0, Segment::new("ʔ"));
        }
    }
    syls
}

impl FrontEnd for Jeongeum {
    fn starts(&self, c: char) -> bool {
        is_block(c) || is_conjoining(c) || compat_lead(c).is_some() || compat_vowel(c).is_some()
    }
    fn continues(&self, c: char) -> bool {
        is_block(c) || is_conjoining(c) || compat_tail(c).is_some() || compat_lead(c).is_some() || compat_vowel(c).is_some()
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

/// 儿 의 正音 표기: ᅌᅥᇫ (사용자 지정 2026-11-27)
const ER: &str = "\u{114C}\u{1165}\u{11EB}";

/// 성조를 노래의 음높이 굽이로 싣는다 (멜로디 음에서 반음 단위로): 1성(경성 포함) 평평 · 2성 아래에서 올라감 ·
/// 3성 구 끝(`last`)이면 온3성 214 (내려갔다 오름), 구 안이면 반3성 21 (내려가기만) · 4성 크게 하강.
/// 3성+3성 → 2성은 사전이 이미 바꿔 온다.
/// 소리(자음·모음)는 손대지 않는다 (사용자 지정 2026-11-27): 한자의 소리는 중국어 처리기의 것 그대로이고 正音 은
/// 화면 표기일 뿐이다. 표기를 바꿔도 소리는 달라지지 않는다. (그 전에는 표기에 ㆆ 이 없으면 처리기의 첫머리 [ʔ] 를
/// 떼고, 구 끝 3성에 [ɨ] 활음을 붙였다.)
fn apply_tone(s: &mut Syllable, tone: u8, last: bool) {
    let bend = |at, len, max, st| Bend { at, len, max, st };
    match tone {
        2 => s.pitch = vec![bend(Edge::Start, 0.75, 0.6, -4.0)],
        3 if !last => s.pitch = vec![bend(Edge::End, 0.8, 0.5, -3.0)],
        3 => s.pitch = vec![bend(Edge::Middle, 0.4, 0.35, -2.0), bend(Edge::End, 0.35, 0.3, 3.0)],
        4 => s.pitch = vec![bend(Edge::End, 0.85, 0.5, -8.0)],
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
            if n == 1 && !spelling.is_empty() {
                s.text = spelling.to_string();
            } else if n == 2 && s.text.ends_with('儿') && !spelling.is_empty() {
                // 儿화 (사용자 지정 2026-11-27): 儿 의 正音 표기는 ᅌᅥᇫ (= [ɻ]). 앞 글자의 표기 뒤에 붙인다 —
                // 玩儿 우ᅌᅡᆫᅌᅥᇫ, 空儿 콩ᅌᅥᇫ. 소리는 중국어 처리기의 것 (앞 음절 끝의 ɻ) 그대로다.
                s.text = format!("{spelling}{ER}");
            }
            if n == 1 && cs.get(i) == Some(&'儿') {
                // 제 음절로 읽는 儿 (儿子, 女儿) 도 같은 글자다; 표의 er2 와도 같다 (er2 ᅌᅥᇫ · er3 ᄋힺᅌᅥᇫ · er4 ᅌᅥᇹ; er1 · er5 는 그렇게 읽는 글자가 없어 표기를 두지 않는다)
                s.text = ER.to_string();
            }
            apply_tone(s, tone, i + n >= cs.len());
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

/// 구절 끝(쉼표·마침표·줄바꿈 앞, 가사의 끝)의 ᅼ ᆣ 음절은 모음 뒤로도 숨을 내쉰다 (사용자 지정 2026-11-24).
/// UBC 산스크리트 녹음: 낱말 안의 gha dha 는 터진 직후 40–50 ms 만 숨이 섞이고 모음은 바로 다음 자음으로
/// 가지만, 낱음절로 끝날 때는 모음이 70–80 ms 의 숨으로 풀린다. 데바나가리로 넣은 가사에도 같다.
pub fn phrase_final_breath(syls: &mut [Syllable]) {
    let n = syls.len();
    for (i, s) in syls.iter_mut().enumerate() {
        let last = s.phrase_end || i + 1 == n;
        if last && s.text.chars().any(|c| matches!(c, 'ᅼ' | 'ᆣ')) && s.coda.is_empty() && matches!(s.nucleus.ipa.as_str(), "ɐ" | "a") {
            s.coda.push(Segment { dur: Some(BREATH_PHRASE_END), ..Segment::new("h") });
        }
    }
}
