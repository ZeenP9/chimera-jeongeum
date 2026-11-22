//! 데바나가리(산스크리트) → 正音 (《正音으로 듣고쓰다》, hun-min.com/?p=235 의 표).
//!
//! 아크샤라(자음 + 모음 기호) 하나가 노래 한 음절이다. 음절 글자는 책의 正音 표기로 보이고, 소리는 산스크리트
//! 음가로 낸다 (일본어 가나 처리기와 같은 방식).
//!
//! 표기 (책의 표, 유성 유기음은 사용자 지정 2026-11-23):
//! - 무성 무기 ㄲ ᄍ ㄸ ㄸ+ᇫ ㅃ, 무성 유기 ㅋ ㅊ ㅌ ㅌ+ᇫ ㅍ, 유성 무기 ㄱ ㅈ ㄷ ㄷ+ᇫ ㅂ,
//!   유성 유기는 ㅡ 를 뒤에 품은 결합모음: घ ᄀᅼ · झ ᄌᅼ · ढ ᄃᅼᇫ · ध ᄃᅼ · भ ᄇᅼ (i 앞 ᆜ, ā 앞 ᆣ, u 앞 ᆖ).
//! - 권설음은 받침 ㅿ (ट 떠ᇫ, ष 셔ᇫ), ś 셔, s 써, h ㅎ, 비음 ङ ㆁ · ञ ㄴ · ण ㄴ+ᇫ · न ㄴ · म ㅁ, y ㅇ+j, r ㄹ, l ㄹ, v ㅸ.
//! - 모음: a ㅓ, ā ퟅ(ㆍ+ㅏ), i ㅣ, ī ퟄ(ㅣ+ㅣ), u ㅜ, ū ᆍ(ㅜ+ㅜ), e ㅔ, o ㅗ, ai ㆎ, au ᆠ, ṛ 리.
//! - anusvāra ं 는 받침 ㆁ/ㄴ/ㅁ (뒤 자음의 자리에 맞춰), visarga ः 는 받침 ㅎ, candrabindu ँ 는 비모음.
//! - 자음 연결(virāma ्): 앞 음절이 있으면 그 받침으로, 아니면 ㅡ 를 붙인 한 음절(끄).
//!
//! 소리: a [ɐ], ā [aː], i, ī [iː], u, ū [uː], e [eː], o [oː], ai [ɐi], au [ɐu]; 무기 파열음은 기식 없음,
//! 유기는 [ʰ], 유성 유기는 숨 섞인 개방 [ʱ] (엔진의 breathy release); 권설 ʈ ʈʰ ɖ ɖʱ ɳ ʂ, 구개 tɕ dʑ ɲ ɕ.

use super::syllables::FrontEnd;
use super::{Segment, Syllable};

pub fn is_deva(c: char) -> bool {
    ('\u{0900}'..='\u{097F}').contains(&c)
}

const VIRAMA: char = '\u{094D}';
const ANUSVARA: char = '\u{0902}';
const VISARGA: char = '\u{0903}';
const CANDRABINDU: char = '\u{0901}';
const NUKTA: char = '\u{093C}';

/// 자음: (正音 초성 자모, 받침 ㅿ 여부(권설), 유성 유기 여부, IPA, 기식 Some(true)/무기 Some(false)/None, 받침 자모).
struct Cons {
    lead: char,
    retro: bool,
    murmur: bool,
    ipa: &'static str,
    asp: Option<bool>,
    tail: Option<char>,
    /// ś ṣ: 모음을 y 계열로 (śa 셔)
    palatal_vowel: bool,
}

fn cons(c: char) -> Option<Cons> {
    let k = |lead, retro, murmur, ipa, asp, tail| Cons { lead, retro, murmur, ipa, asp, tail, palatal_vowel: false };
    Some(match c {
        'क' => k('ᄁ', false, false, "k", Some(false), Some('ᆨ')),
        'ख' => k('ᄏ', false, false, "k", Some(true), Some('ᆨ')),
        'ग' => k('ᄀ', false, false, "g", None, Some('ᆨ')),
        'घ' => k('ᄀ', false, true, "g", None, Some('ᆨ')),
        'ङ' => k('ᅌ', false, false, "ŋ", None, Some('ᆼ')),
        'च' => k('ᄍ', false, false, "tɕ", Some(false), Some('ᆮ')),
        'छ' => k('ᄎ', false, false, "tɕ", Some(true), Some('ᆮ')),
        'ज' => k('ᄌ', false, false, "dʑ", None, Some('ᆮ')),
        'झ' => k('ᄌ', false, true, "dʑ", None, Some('ᆮ')),
        'ञ' => k('ᄂ', false, false, "ɲ", None, Some('ᆫ')),
        'ट' => k('ᄄ', true, false, "ʈ", Some(false), Some('ᆮ')),
        'ठ' => k('ᄐ', true, false, "ʈ", Some(true), Some('ᆮ')),
        'ड' => k('ᄃ', true, false, "ɖ", None, Some('ᆮ')),
        'ढ' => k('ᄃ', true, true, "ɖ", None, Some('ᆮ')),
        'ण' => k('ᄂ', true, false, "ɳ", None, Some('ᆫ')),
        'त' => k('ᄄ', false, false, "t", Some(false), Some('ᆮ')),
        'थ' => k('ᄐ', false, false, "t", Some(true), Some('ᆮ')),
        'द' => k('ᄃ', false, false, "d", None, Some('ᆮ')),
        'ध' => k('ᄃ', false, true, "d", None, Some('ᆮ')),
        'न' => k('ᄂ', false, false, "n", None, Some('ᆫ')),
        'प' => k('ᄈ', false, false, "p", Some(false), Some('ᆸ')),
        'फ' => k('ᄑ', false, false, "p", Some(true), Some('ᆸ')),
        'ब' => k('ᄇ', false, false, "b", None, Some('ᆸ')),
        'भ' => k('ᄇ', false, true, "b", None, Some('ᆸ')),
        'म' => k('ᄆ', false, false, "m", None, Some('ᆷ')),
        'य' => Cons { palatal_vowel: true, ..k('ᄋ', false, false, "j", None, None) },
        'र' => k('ᄅ', false, false, "ɾ", None, Some('ᆯ')),
        'ल' => k('ᄅ', false, false, "l", None, Some('ᆯ')),
        'व' => k('ᄫ', false, false, "ʋ", None, None),
        'श' => Cons { palatal_vowel: true, ..k('ᄉ', false, false, "ɕ", None, Some('ᆺ')) },
        'ष' => Cons { palatal_vowel: true, ..k('ᄉ', true, false, "ʂ", None, Some('ᆺ')) },
        'स' => k('ᄊ', false, false, "s", None, Some('ᆺ')),
        'ह' => k('ᄒ', false, false, "ɦ", None, None),
        _ => return None,
    })
}

/// 모음: (正音 중성 자모, IPA, 긴 모음 여부). `a` 는 내재 모음.
fn vowel_of(c: Option<char>) -> Option<(char, &'static str, bool)> {
    Some(match c {
        None | Some('अ') => ('ᅥ', "ɐ", false),
        Some('ा' | 'आ') => ('\u{D7C5}', "a", true),
        Some('ि' | 'इ') => ('ᅵ', "i", false),
        Some('ी' | 'ई') => ('\u{D7C4}', "i", true),
        Some('ु' | 'उ') => ('ᅮ', "u", false),
        Some('ू' | 'ऊ') => ('\u{118D}', "u", true),
        Some('े' | 'ए') => ('ᅦ', "e", true),
        Some('ै' | 'ऐ') => ('\u{11A1}', "aɪ", true),
        Some('ो' | 'ओ') => ('ᅩ', "o", true),
        Some('ौ' | 'औ') => ('\u{11A0}', "aʊ", true),
        Some('ृ' | 'ऋ') => ('ᅳ', "ri", false), // ṛ: 자음 + 르이 → `render` 에서 ㅡ 블록 + 리
        _ => return None,
    })
}

fn is_matra(c: char) -> bool {
    matches!(c, '\u{093E}'..='\u{094C}' | '\u{0962}' | '\u{0963}')
}
fn is_independent_vowel(c: char) -> bool {
    matches!(c, '\u{0904}'..='\u{0914}')
}

/// 유성 유기음 뒤 모음: ㅡ 를 뒤에 품은 결합모음 (a ᅼ, i ᆜ, ā ᆣ, u ᆖ), 그 밖은 그대로.
fn murmur_vowel(v: char) -> char {
    match v {
        'ᅥ' => 'ᅼ',
        'ᅵ' => 'ᆜ',
        '\u{D7C5}' => 'ᆣ',
        'ᅮ' => 'ᆖ',
        other => other,
    }
}

/// ś ṣ 의 모음: y 계열 (śa 셔, śā 샤, śu 슈, śo 쇼)
fn palatal(v: char) -> char {
    match v {
        'ᅥ' => 'ᅧ',
        '\u{D7C5}' | 'ᅡ' => 'ᅣ',
        'ᅮ' | '\u{118D}' => 'ᅲ',
        'ᅩ' => 'ᅭ',
        other => other,
    }
}

fn compose(s: &str) -> String {
    let cs: Vec<char> = s.chars().collect();
    let l = |c: char| ('\u{1100}'..='\u{1112}').contains(&c);
    let v = |c: char| ('\u{1161}'..='\u{1175}').contains(&c);
    let t = |c: char| ('\u{11A8}'..='\u{11C2}').contains(&c);
    match cs[..] {
        [a, b] if l(a) && v(b) => char::from_u32(0xAC00 + (a as u32 - 0x1100) * 588 + (b as u32 - 0x1161) * 28).map(String::from).unwrap_or(s.into()),
        [a, b, c] if l(a) && v(b) && t(c) => {
            char::from_u32(0xAC00 + (a as u32 - 0x1100) * 588 + (b as u32 - 0x1161) * 28 + (c as u32 - 0x11A7)).map(String::from).unwrap_or(s.into())
        }
        _ => s.to_string(),
    }
}

/// 한 음절의 재료.
struct Syl {
    onset: Vec<Cons>,
    vowel: (char, &'static str, bool),
    tail: Vec<(Option<char>, Segment)>,
    nasal_vowel: bool,
}

pub struct JeongeumDeva;

impl FrontEnd for JeongeumDeva {
    fn starts(&self, c: char) -> bool {
        is_deva(c) && c != '\u{0964}' && c != '\u{0965}'
    }
    fn word(&self, run: &str) -> Vec<Syllable> {
        let cs: Vec<char> = run.chars().collect();
        let mut syls: Vec<Syl> = Vec::new();
        let mut pending: Vec<Cons> = Vec::new(); // virāma 로 이어진 자음
        let mut i = 0;
        while i < cs.len() {
            let c = cs[i];
            // nukta: ड़ ढ़ = 권설 탄음 ɽ (ᅟᅠퟛ), 그 밖은 무시
            let nukta = cs.get(i + 1) == Some(&NUKTA);
            if let Some(mut k) = cons(c) {
                if nukta && matches!(c, 'ड' | 'ढ') {
                    k = Cons { lead: 'ᄅ', retro: true, ipa: "ɽ", tail: Some('ퟛ'), asp: None, ..k };
                }
                i += if nukta { 2 } else { 1 };
                if cs.get(i) == Some(&VIRAMA) {
                    pending.push(k);
                    i += 1;
                    continue;
                }
                let v = if i < cs.len() && is_matra(cs[i]) {
                    let m = cs[i];
                    i += 1;
                    vowel_of(Some(m))
                } else {
                    vowel_of(None)
                };
                let Some(v) = v else { continue };
                // 앞 음절이 있으면 virāma 자음들은 앞 음절 받침으로 가고(धर्म 덜.머, बुद्ध 붇.ᄃᅼ), 이 자음만 초성.
                // 낱말 첫머리의 겹자음(क्ष, प्र)은 초성으로 남는다
                let mut onset = std::mem::take(&mut pending);
                if !syls.is_empty() {
                    let prev = syls.last_mut().unwrap();
                    for p in onset.drain(..) {
                        prev.tail.push((p.tail, Segment { aspirated: p.asp, ..Segment::new(p.ipa) }));
                    }
                }
                onset.push(k);
                syls.push(Syl { onset, vowel: v, tail: Vec::new(), nasal_vowel: false });
            } else if is_independent_vowel(c) {
                i += 1;
                let Some(v) = vowel_of(Some(c)) else { continue };
                syls.push(Syl { onset: std::mem::take(&mut pending), vowel: v, tail: Vec::new(), nasal_vowel: false });
            } else if c == ANUSVARA {
                i += 1;
                // 뒤 자음의 자리에 맞춘 비음 받침 (ङ्क 류), 마찰음·ह·끝 앞은 비모음
                let next = cs.get(i).copied().and_then(cons);
                if let Some(prev) = syls.last_mut() {
                    match next.as_ref().map(|n| n.ipa) {
                        Some("k" | "g" | "ŋ") => prev.tail.push((Some('ᆼ'), Segment::new("ŋ"))),
                        Some("p" | "b" | "m") => prev.tail.push((Some('ᆷ'), Segment::new("m"))),
                        Some("tɕ" | "dʑ" | "ɲ" | "ʈ" | "ɖ" | "ɳ" | "t" | "d" | "n") => prev.tail.push((Some('ᆫ'), Segment::new("n"))),
                        _ => {
                            prev.tail.push((Some('ᇰ'), Segment::new("")));
                            prev.nasal_vowel = true;
                        }
                    }
                }
            } else if c == VISARGA {
                i += 1;
                if let Some(prev) = syls.last_mut() {
                    prev.tail.push((Some('ᇂ'), Segment::new("h")));
                }
            } else if c == CANDRABINDU {
                i += 1;
                if let Some(prev) = syls.last_mut() {
                    prev.nasal_vowel = true;
                }
            } else {
                i += 1;
            }
        }
        // 낱말 끝의 virāma 자음: 마지막 음절의 받침
        for p in pending.drain(..) {
            if let Some(prev) = syls.last_mut() {
                prev.tail.push((p.tail, Segment { aspirated: p.asp, ..Segment::new(p.ipa) }));
            }
        }
        syls.into_iter().map(render).collect()
    }
}

fn render(s: Syl) -> Syllable {
    let (mut vj, vipa, long) = s.vowel;
    let main = s.onset.last();
    if let Some(k) = main {
        if k.murmur {
            vj = murmur_vowel(vj);
        }
        if k.palatal_vowel {
            vj = palatal(vj);
        }
    }
    // 正音 글자: 겹자음 초성은 앞 자음마다 ㅡ 를 붙인 블록 (끄셔ᇫ), 마지막 자음 + 모음 + 받침
    let mut text = String::new();
    for k in s.onset.iter().take(s.onset.len().saturating_sub(1)) {
        text.push_str(&compose(&format!("{}ᅳ", k.lead)));
    }
    let lead = main.map_or('ᄋ', |k| k.lead);
    // ṛ (모음 r̥): 자음은 ㅡ 를 붙여 따로(끄), 이어서 리 — 소리는 자음 + [ɾi]
    let vocalic_r = vipa == "ri";
    if vocalic_r {
        text.push_str(&compose(&format!("{lead}ᅳ")));
    }
    let lead = if vocalic_r { 'ᄅ' } else { lead };
    let vj = if vocalic_r { 'ᅵ' } else { vj };
    let mut block = format!("{lead}{vj}");
    let retro = main.is_some_and(|k| k.retro);
    for (t, _) in &s.tail {
        if let Some(t) = t {
            block.push(*t);
        }
    }
    if retro {
        block.push('ᇫ');
    }
    text.push_str(&compose(&block));
    // 소리
    let onset: Vec<Segment> = s
        .onset
        .iter()
        .map(|k| Segment { aspirated: k.asp, vot: (k.asp == Some(true)).then_some(0.07), breathy: k.murmur.then_some(true), ..Segment::new(k.ipa) })
        .collect();
    let mut onset = onset;
    let vipa = if vocalic_r {
        onset.push(Segment::new("ɾ"));
        "i"
    } else {
        vipa
    };
    let coda: Vec<Segment> = s.tail.into_iter().map(|(_, seg)| seg).filter(|seg| !seg.ipa.is_empty()).collect();
    // 긴 모음은 노래에서 음표가 길이를 정하므로 소리값은 같고, 글자(퍄 ᆍ ퟄ)로만 구분한다
    let _ = long;
    let nucleus = Segment { nasalized: s.nasal_vowel.then_some(true), ..Segment::new(vipa) };
    Syllable { text, onset, nucleus, coda, ..Default::default() }
}
