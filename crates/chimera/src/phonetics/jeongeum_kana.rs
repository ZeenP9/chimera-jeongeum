//! 일본어 가나 → 正音 (《正音으로 듣고쓰다》 243–246쪽, hun-min.com/?p=1335 「雪の華」 표기).
//!
//! 가나 한 박(모라)이 노래 한 음절이다. 음절 글자는 책의 正音 표기로 보이고, 소리는 일본어 음가로 낸다
//! (책의 일본어 표기 ᄀힹ ᄃힹ ᄉᆜ 츻 은 正音 글자를 그대로 읽는 규칙 — ᄀ+ㅡ결합 = 목젖음 등 — 과 다른 뜻으로
//! 쓰였으므로, 正音 처리기에 넘기지 않고 여기서 바로 음가를 매긴다).
//!
//! 표기 규칙 (책):
//! - 청음 k t p ch 는 단어 첫머리 ㅋ ㅌ ㅍ ㅊ, 그 밖 ㄲ ㄸ ㅃ ㅉ (카·까, 타·따; 일본어 귀에는 같은 소리).
//! - 탁음: が행·だ행은 떨어진 ㅡ(유성 표지)로 ᄀ+ᅳ+ᅡ, ᄀ+ᅳ+ᅵ, ᄀ+ᅳ+ᅦ, ᄀ+ᅳ+ᅩ / ᄃ+ᅳ+ᅡ, ᄃ+ᅳ+ᅦ, ᄃ+ᅳ+ᅩ
//!   (책의 ᄀힹ ᄃힹ 처럼 붙은 결합모음은 正音 규칙에서 목젖음이라 두 글자로 떼어 쓴다: 사용자 지정, 2026-11-22).
//!   ぐ 는 ᄀᆕ, ぢ 지, づ ᅀᅴ, ざ행 ᅀᅡ 지 ᅀᅴ ᅀᅦ ᅀᅩ, ば행 바 비 ᄇᆕ 베 보.
//! - う단은 ㅜ 대신 ᅟᆕ [ʉ] (ᄏᆕ ᄉᆕ ᄂᆕ ᄆᆕ ᄅᆕ ᄒᆕ ᄇᆕ ᄑᆕ, う 단독 ᄋᆕ); し ᄉᆜ, つ 츻 (ᄎᆕᇹ 의 줄인 꼴).
//! - 촉음 っ = 받침 ㆆ [ʔ] (나ᅙᅡ떼 처럼 다음 음절 첫소리로 적은 곳도 있지만 받침으로 통일), 발음 ん = 받침 ㄶ [ɴ].
//! - 장음 ー 와 おう 의 う 는 모음을 한 박 더 (호도오).
//! 소리: a i ʉ e̞ o̞, 청음은 첫머리만 가볍게 기식, 탁음은 유성 g d b z dʑ, し ɕ ち tɕ つ ts ひ ç ふ ɸ.
//! 아직: 조사 は·へ 를 wa·e 로 읽는 일(형태소 분석이 필요), 문장 속 한자의 일본어 독음(사전이 필요).

use super::syllables::FrontEnd;
use super::{Segment, Syllable};

pub fn is_kana(c: char) -> bool {
    matches!(c, '\u{3041}'..='\u{3096}' | '\u{30A1}'..='\u{30FA}' | 'ー')
}

/// 가타카나 → 히라가나 (ー 는 그대로).
fn hira(c: char) -> char {
    match c {
        '\u{30A1}'..='\u{30F6}' => char::from_u32(c as u32 - 0x60).unwrap_or(c),
        _ => c,
    }
}

/// 한 박: 자음 행(romaji 자음), 모음(a i u e o), 요음 여부.
struct Mora {
    cons: &'static str,
    vowel: char,
    yoon: bool,
}

/// 히라가나 한 글자 → (자음 행, 모음). 촉음·발음·장음·작은 요음은 따로 다룬다.
fn base(c: char) -> Option<(&'static str, char)> {
    const ROWS: [(&str, &str); 15] = [
        ("", "あいうえお"),
        ("k", "かきくけこ"),
        ("g", "がぎぐげご"),
        ("s", "さしすせそ"),
        ("z", "ざじずぜぞ"),
        ("t", "たちつてと"),
        ("d", "だぢづでど"),
        ("n", "なにぬねの"),
        ("h", "はひふへほ"),
        ("b", "ばびぶべぼ"),
        ("p", "ぱぴぷぺぽ"),
        ("m", "まみむめも"),
        ("y", "や\0ゆ\0よ"),
        ("r", "らりるれろ"),
        ("w", "わ\0\0\0を"),
    ];
    const V: [char; 5] = ['a', 'i', 'u', 'e', 'o'];
    for (cons, row) in ROWS {
        if let Some(k) = row.chars().position(|x| x == c && x != '\0') {
            // を is o
            return Some(if c == 'を' { ("", 'o') } else { (cons, V[k]) });
        }
    }
    // small vowels (ファ フィ ティ …) ride on the consonant before: handled as plain vowels here
    match c {
        'ぁ' => Some(("", 'a')),
        'ぃ' => Some(("", 'i')),
        'ぅ' => Some(("", 'u')),
        'ぇ' => Some(("", 'e')),
        'ぉ' => Some(("", 'o')),
        _ => None,
    }
}

fn small_y(c: char) -> Option<char> {
    match c {
        'ゃ' => Some('a'),
        'ゅ' => Some('u'),
        'ょ' => Some('o'),
        _ => None,
    }
}

/// 한 박의 正音 표기 (초성 + 중성 블록). `first`: 단어 첫머리 (청음 ㅋㅌㅍㅊ / 그 밖 ㄲㄸㅃㅉ).
fn spell(m: &Mora, first: bool) -> String {
    let (c, v, y) = (m.cons, m.vowel, m.yoon);
    // 모음 자모: 요음이면 ㅑ ㅠ ㅛ
    let vj = |v: char| match (v, y) {
        ('a', false) => 'ᅡ',
        ('i', _) => 'ᅵ',
        ('u', false) => 'ᆕ',
        ('e', false) => 'ᅦ',
        ('o', false) => 'ᅩ',
        ('a', true) => 'ᅣ',
        ('u', true) => 'ᅲ',
        ('o', true) => 'ᅭ',
        _ => 'ᅦ',
    };
    let lead = |plain: char, medial: char| if first { plain } else { medial };
    let whole = |s: &str| s.to_string();
    let block = |l: char, v: char| format!("{l}{v}");
    let s = match (c, v, y) {
        // 모음 단독: う ᄋᆕ
        ("", v, _) => block('ᄋ', vj(v)),
        ("k", v, _) => block(lead('ᄏ', 'ᄁ'), vj(v)),
        // 탁음 が행: ᄀ + ㅡ결합 (ᄀힹ 긔 ᄀᆕ ᄀힻ ᄀힼ); 요음은 ᄀ + ㅑㅠㅛ
        // 떨어진 ㅡ = 유성 표지 (ᄀ+ᅳ+모음 = g); 붙은 결합모음(ᄀힹ)은 목젖음이라 쓰지 않는다
        ("g", 'a', false) => whole("그ᅡ"),
        ("g", 'i', _) => whole("그ᅵ"),
        ("g", 'u', false) => whole("ᄀᆕ"),
        ("g", 'e', _) => whole("그ᅦ"),
        ("g", 'o', false) => whole("그ᅩ"),
        ("g", v, true) => block('ᄀ', vj(v)),
        ("s", 'i', false) => whole("ᄉᆜ"),
        ("s", v, true) => block('ᄉ', vj(v)),
        ("s", v, _) => block('ᄉ', vj(v)),
        ("z" | "d", 'i', _) if !y => whole("지"),
        ("z" | "d", v, true) => block('ᄌ', vj(v)),
        ("z" | "d", 'u', _) => whole("ᅀᅴ"),
        ("z", v, _) => block('ᅀ', vj(v)),
        ("d", 'a', _) => whole("드ᅡ"),
        ("d", 'e', _) => whole("드ᅦ"),
        ("d", 'o', _) => whole("드ᅩ"),
        ("t", 'i', false) => block(lead('ᄎ', 'ᄍ'), 'ᅵ'),
        ("t", v, true) => block(lead('ᄎ', 'ᄍ'), vj(v)),
        ("t", 'u', _) => whole("츻"),
        ("t", v, _) => block(lead('ᄐ', 'ᄄ'), vj(v)),
        ("n", v, _) => block('ᄂ', vj(v)),
        ("h", v, _) => block('ᄒ', vj(v)),
        ("b", v, _) => block('ᄇ', vj(v)),
        ("p", v, _) => block(lead('ᄑ', 'ᄈ'), vj(v)),
        ("m", v, _) => block('ᄆ', vj(v)),
        ("y", 'a', _) => whole("야"),
        ("y", 'u', _) => whole("유"),
        ("y", _, _) => whole("요"),
        ("r", v, _) => block('ᄅ', vj(v)),
        ("w", _, _) => whole("와"),
        (_, v, _) => block('ᄋ', vj(v)),
    };
    // う단의 ㅠ (요음 ゅ) 는 ᅲ 그대로, 단독 ㅜ 자리는 ᅟᆕ: 위에서 처리
    compose(&s)
}

/// 현대 자모 초성+중성 두 글자는 완성형 한 글자로 (카), 옛 자모가 든 것은 그대로 (ᄀힹ ᄏᆕ).
fn compose(s: &str) -> String {
    let cs: Vec<char> = s.chars().collect();
    if let [l, v] = cs[..]
        && ('\u{1100}'..='\u{1112}').contains(&l)
        && ('\u{1161}'..='\u{1175}').contains(&v)
    {
        let code = 0xAC00 + (l as u32 - 0x1100) * 588 + (v as u32 - 0x1161) * 28;
        return char::from_u32(code).map(String::from).unwrap_or_else(|| s.to_string());
    }
    s.to_string()
}

/// 한 박의 소리: (첫소리, 모음).
fn sound(m: &Mora, first: bool) -> (Vec<Segment>, &'static str) {
    let vowel = match m.vowel {
        'a' => "a",
        'i' => "i",
        'u' => "ʉ",
        'e' => "e̞",
        _ => "o̞",
    };
    // 청음 파열·파찰음: 첫머리만 가볍게 기식, 그 밖은 무기
    let stop = |ipa: &str| {
        let (a, v) = if first { (Some(true), Some(0.03)) } else { (Some(false), None) };
        Segment { aspirated: a, vot: v, ..Segment::new(ipa) }
    };
    let i = m.vowel == 'i' && !m.yoon;
    let u = m.vowel == 'u' && !m.yoon;
    let mut on: Vec<Segment> = match m.cons {
        "" => vec![],
        "k" => vec![stop("k")],
        "g" => vec![Segment::new("g")],
        "s" if i || m.yoon => vec![Segment::new("ɕ")],
        "s" => vec![Segment::new("s")],
        "z" | "d" if i || m.yoon => vec![Segment::new("dʑ")],
        "z" | "d" if u => vec![Segment::new("z")],
        "z" => vec![Segment::new("z")],
        "d" => vec![Segment::new("d")],
        "t" if i || m.yoon => vec![stop("tɕ")],
        "t" if u => vec![stop("ts")],
        "t" => vec![stop("t")],
        "n" => vec![Segment::new("n")],
        "h" if i => vec![Segment::new("ç")],
        "h" if u => vec![Segment::new("ɸ")],
        "h" => vec![Segment::new("h")],
        "b" => vec![Segment::new("b")],
        "p" => vec![stop("p")],
        "m" => vec![Segment::new("m")],
        "y" => vec![Segment::new("j")],
        "r" => vec![Segment::new("ɾ")],
        "w" => vec![Segment::new("ɰ")],
        _ => vec![],
    };
    // 요음: 구개음(ɕ tɕ dʑ)은 활음을 품고, 나머지는 j 를 덧붙인다 (きゃ kja)
    if m.yoon && !matches!(m.cons, "s" | "z" | "d" | "t" | "y") {
        on.push(Segment::new("j"));
    }
    (on, vowel)
}

pub struct JeongeumKana;

impl FrontEnd for JeongeumKana {
    fn starts(&self, c: char) -> bool {
        is_kana(c)
    }
    fn word(&self, run: &str) -> Vec<Syllable> {
        let cs: Vec<char> = run.chars().map(hira).collect();
        let mut out: Vec<Syllable> = Vec::new();
        // 장음 ー 와 おう 의 う 가 이어 낼 앞 모음
        let mut last_vowel: Option<char> = None;
        let mut i = 0;
        while i < cs.len() {
            let c = cs[i];
            // 촉음 っ: 앞 음절의 받침 ㆆ [ʔ]
            if c == 'っ' {
                if let Some(prev) = out.last_mut() {
                    prev.text.push('ᇹ');
                    prev.coda.push(Segment::new("ʔ"));
                }
                i += 1;
                continue;
            }
            // 발음 ん: 앞 음절의 받침 ㄶ [ɴ] (홀로면 ᅙᆞᆭ 처럼 한 박)
            if c == 'ん' {
                if let Some(prev) = out.last_mut() {
                    // つん 은 츻 의 받침 ㅎ 에 ㄴ 을 얹어 츠ᇵ (책: tsundere 츠ᇵ데레)
                    if prev.text == "츻" {
                        prev.text = "츠ᇵ".into();
                    } else {
                        prev.text.push('ᆭ');
                    }
                    prev.coda.push(Segment::new("ɴ"));
                } else {
                    out.push(Syllable { text: "ᅙᆞᆭ".into(), nucleus: Segment::new("ɴ"), ..Default::default() });
                }
                i += 1;
                continue;
            }
            let long = c == 'ー' || (c == 'う' && last_vowel == Some('o'));
            let m = if long {
                let Some(v) = last_vowel else {
                    i += 1;
                    continue;
                };
                Mora { cons: "", vowel: v, yoon: false }
            } else if let Some((cons, vowel)) = base(c) {
                // 작은 ゃゅょ 가 뒤따르면 요음 (きゃ)
                match cs.get(i + 1).copied().and_then(small_y) {
                    Some(v) if vowel == 'i' && !cons.is_empty() => {
                        i += 1;
                        Mora { cons, vowel: v, yoon: true }
                    }
                    _ => Mora { cons, vowel, yoon: false },
                }
            } else {
                i += 1;
                continue;
            };
            let first = out.is_empty();
            let (onset, nucleus) = sound(&m, first);
            // し 는 ᄉᆜ: ㅣ + 숨 (책; 일본어 고모음 무성화) — 숨 섞인 i̤
            let breathy = (m.cons == "s" && m.vowel == 'i' && !m.yoon).then_some(true);
            out.push(Syllable { text: spell(&m, first), onset, nucleus: Segment { breathy, ..Segment::new(nucleus) }, ..Default::default() });
            last_vowel = Some(m.vowel);
            i += 1;
        }
        out
    }
}
