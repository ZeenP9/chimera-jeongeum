//! 폴란드어 → 正音 (《正音으로 듣고쓰다》 216–218쪽, hun-min.com/?p=384).
//!
//! 가나·데바나가리 처리기와 같은 방식: 음절 글자는 책의 正音 표기로 보이고, 소리는 폴란드어 음가로 낸다.
//! 폴란드어 글자(ą ć ę ł ń ó ś ź ż)가 하나라도 든 가사는 로마자 낱말 전부를 폴란드어로 읽는다 (`whole`).
//!
//! 표기 (217쪽 표, 218쪽 표와 예시):
//! - 모음 a ㅏ · e ㅔ · i ㅣ · o ᆟ · u ó ㅜ · y ᆜ ([ɨ]; 자음 없이 ᅙᆜ) · ą ᆟ+비음 · ę ㅐ+비음.
//!   ą ę 의 비음은 뒤 자음을 따른다: 마찰음 앞 ㆁ (비모음 [ɔ̃ ɛ̃]), b p 앞 ㅁ, c cz d dz dż t 앞 ㄴ, ć dź 앞 ㄴ([ɲ]),
//!   g k 앞 ㅇ([ŋ]); 낱말 끝 ę 는 [ɛ] (imię 이미애), ą 는 [ɔ̃].
//! - 자음 p ㅍ · b ㅂ · t ㅌ · d ㄷ · k ㅋ · g ㄱ · f ㆄ · w ㅸ(무성 ㆄ) · s ㅅ · z ㅿ · c ㅊ · dz ㅈ · m ㅁ · n ㄴ ·
//!   h ch ㆅ · r 초성 ㄹ [ɾ] · l ㄹㄹ (Ale 알레) · ł [w] ㅜ (ㅘ ㅞ ㅝ) · j ㅑ ㅖ ㅛ ㅠ.
//! - 경구개 ć ś ź dź ń (ci si zi dzi ni): ㅊ ㅅ ㅿ ㅈ ㄴ + ᆛ (ᄎᆛ ᄉᆛ ᅀᆛ ᄌᆛ ᄂᆛ); ń + 모음은 녜 냐 뇨 뉴.
//! - 권설 cz sz rz(ż) dż: y 와 만나면 ᄎᆜ ᄉᆜ ᅀᆜ (ᄌᆜ); 그 밖의 모음 앞 cz 체ᇫ · dż 제ᇫ (받침 ㅿ) · sz 슈+모음 ·
//!   rz(ż) ᅀ+ㅘ ㅞ ㅝ (Szczebrzeszyn 슈체ᇫ브ᅀᅰᄉᆜᆫ); 자음 앞 츠ᇫ · 슈 · ᅀᅴ · 즈ᇫ; 낱말 끝 ᄎᆜ · ᄉᆜ (Twarz …ᄉᆜ).
//! - 자음 + i + 모음 (pie kie wie): 자음+ㅣ 블록, 이어 모음 (ᄫᅵ에, 피앤); 모음 없는 자음은 ㅡ 블록 (브 크 스).
//!
//! 소리: a [a] · e [ɛ] · i [i] · o [ɔ] · u [u] · y [ɨ]; 낱말 끝과 무성음 앞의 유성 장애음은 무성음이 되고
//! (역행 동화), w 와 rz 는 앞 무성음을 따른다 (twarz [tf], krzesło [kʂ]); 모음 없는 낱말(w z)은 뒤 낱말에 붙는다.
//! 책과 다를 수 있는 곳: 책의 예시는 같은 철자를 여러 가지로 적는다 (o: ᆟ·ㅗ, nie: 녜·ᄂᆛ에); 여기서는 위 규칙 하나로 적는다.

use super::syllables::FrontEnd;
use super::{Segment, Syllable};

pub fn is_polish_letter(c: char) -> bool {
    matches!(c, 'ą' | 'ć' | 'ę' | 'ł' | 'ń' | 'ó' | 'ś' | 'ź' | 'ż' | 'Ą' | 'Ć' | 'Ę' | 'Ł' | 'Ń' | 'Ó' | 'Ś' | 'Ź' | 'Ż')
}

/// `whole`: 가사 어딘가에 폴란드어 글자가 있어, 로마자 낱말을 모두 폴란드어로 읽는다.
pub struct JeongeumPolish {
    pub whole: bool,
}

#[derive(Clone, Copy, PartialEq)]
struct Ph {
    /// 자음의 IPA, 모음은 철자 (a e i o u y ą ę)
    k: &'static str,
    vowel: bool,
    /// ą ę 의 비음 받침 글자 (모음), 또는 그 비음 자음 자체 (자음: 글자는 모음 블록에 이미 있다)
    nasal: Option<char>,
    /// rz: 무성음 뒤에서 [ʂ] 가 되어도 글자는 ᅀ (krzesło ㅋᅀᅰ, przecież 프ᅀᅰ)
    rz: bool,
    /// 낱말 끝에서 무성음이 된 ź dź: 글자는 ᄉᆜ ᄎᆜ (odpowiedź …ᄎᆜ)
    dev: bool,
}

fn c(k: &'static str) -> Ph {
    Ph { k, vowel: false, nasal: None, rz: false, dev: false }
}
fn v(k: &'static str) -> Ph {
    Ph { k, vowel: true, nasal: None, rz: false, dev: false }
}

fn is_vowel_letter(ch: char) -> bool {
    matches!(ch, 'a' | 'ą' | 'e' | 'ę' | 'o' | 'ó' | 'u' | 'y' | 'i')
}

fn phonemes(word: &str) -> Vec<Ph> {
    let cs: Vec<char> = word.chars().flat_map(|x| x.to_lowercase()).filter(|x| x.is_alphabetic()).collect();
    let mut out = Vec::new();
    let mut i = 0;
    let at = |i: usize, s: &str| s.chars().enumerate().all(|(j, x)| cs.get(i + j) == Some(&x));
    while i < cs.len() {
        // ci si zi ni dzi: 모음 앞에서는 i 가 구개음 표지일 뿐, 자음 앞·끝에서는 모음 i 도 된다
        let soft = [("dzi", "dʑ"), ("ci", "tɕ"), ("si", "ɕ"), ("zi", "ʑ"), ("ni", "ɲ")].into_iter().find(|(s, _)| at(i, s));
        if let Some((s, k)) = soft {
            out.push(c(k));
            let n = s.chars().count();
            let next_vowel = cs.get(i + n).is_some_and(|&x| is_vowel_letter(x) && x != 'i' && x != 'y');
            i += if next_vowel { n } else { n - 1 };
            continue;
        }
        let di = [("ch", "x"), ("cz", "tʂ"), ("dź", "dʑ"), ("dż", "dʐ"), ("dz", "dz"), ("rz", "ʐ"), ("sz", "ʂ")].into_iter().find(|(s, _)| at(i, s));
        if let Some((s, k)) = di {
            out.push(Ph { rz: s == "rz", ..c(k) });
            i += 2;
            continue;
        }
        let ch = cs[i];
        i += 1;
        if ch == 'x' {
            out.push(c("k"));
            out.push(c("s"));
            continue;
        }
        out.push(match ch {
            'a' => v("a"),
            'e' => v("e"),
            'o' => v("o"),
            'u' | 'ó' => v("u"),
            'y' => v("y"),
            'ą' => v("ą"),
            'ę' => v("ę"),
            // 모음 앞의 i 는 [j] (pie [pjɛ])
            'i' if cs.get(i).is_some_and(|&x| is_vowel_letter(x) && x != 'i') => c("j"),
            'i' => v("i"),
            'p' => c("p"),
            'b' => c("b"),
            't' => c("t"),
            'd' => c("d"),
            'k' | 'q' => c("k"),
            'g' => c("g"),
            'f' => c("f"),
            'w' | 'v' => c("v"),
            's' => c("s"),
            'z' => c("z"),
            'c' => c("ts"),
            'ć' => c("tɕ"),
            'ś' => c("ɕ"),
            'ź' => c("ʑ"),
            'ż' => c("ʐ"),
            'ń' => c("ɲ"),
            'm' => c("m"),
            'n' => c("n"),
            'l' => c("l"),
            'r' => c("r"),
            'ł' => c("w"),
            'j' => c("j"),
            'h' => c("x"),
            _ => continue,
        });
    }
    // 겹자음 (ll nn): 모음 앞이 아니면 하나로 (moll)
    let mut i = 1;
    while i < out.len() {
        if !out[i].vowel && out[i] == out[i - 1] && out.get(i + 1).is_none_or(|n| !n.vowel) {
            out.remove(i);
        } else {
            i += 1;
        }
    }
    out
}

fn voiceless(k: &str) -> Option<&'static str> {
    Some(match k {
        "b" => "p",
        "d" => "t",
        "g" => "k",
        "v" => "f",
        "z" => "s",
        "ʐ" => "ʂ",
        "ʑ" => "ɕ",
        "dz" => "ts",
        "dʐ" => "tʂ",
        "dʑ" => "tɕ",
        _ => return None,
    })
}
fn voiced(k: &str) -> Option<&'static str> {
    Some(match k {
        "p" => "b",
        "t" => "d",
        "k" => "g",
        "f" => "v",
        "s" => "z",
        "ʂ" => "ʐ",
        "ɕ" => "ʑ",
        "ts" => "dz",
        "tʂ" => "dʐ",
        "tɕ" => "dʑ",
        _ => return None,
    })
}
fn obstruent(k: &str) -> bool {
    voiceless(k).is_some() || voiced(k).is_some() || k == "x"
}

/// 유성·무성 동화: 장애음 무리는 마지막 장애음을 따르고 (w rz 는 따르기만 한다), 낱말 끝은 무성음.
fn assimilate(ps: &mut [Ph]) {
    let mut i = 0;
    while i < ps.len() {
        if ps[i].vowel || !obstruent(ps[i].k) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < ps.len() && !ps[j].vowel && obstruent(ps[j].k) {
            j += 1;
        }
        let at_end = j == ps.len();
        let run = &mut ps[i..j];
        let single = run.len() == 1;
        let leader = run.iter().rev().find(|p| !matches!(p.k, "v" | "ʐ") || single).map(|p| p.k);
        let make_voiced = if at_end {
            false
        } else {
            match leader {
                Some(k) => voiceless(k).is_some(),
                None => true,
            }
        };
        for p in run.iter_mut() {
            if make_voiced {
                if let Some(x) = voiced(p.k) {
                    p.k = x;
                }
            } else if let Some(x) = voiceless(p.k) {
                p.dev = at_end && matches!(p.k, "ʑ" | "dʑ");
                p.k = x;
            }
        }
        i = j;
    }
}

/// ą ę 의 비음: 뒤 소리에 따라 비음 자음을 세우거나 비모음으로.
fn nasals(ps: Vec<Ph>) -> Vec<Ph> {
    let mut out = Vec::with_capacity(ps.len() + 2);
    for (i, p) in ps.iter().enumerate() {
        if !(p.vowel && matches!(p.k, "ą" | "ę")) {
            out.push(*p);
            continue;
        }
        let (tail, cons) = match ps.get(i + 1).map(|n| n.k) {
            None if p.k == "ę" => (None, None),
            Some("l" | "w") => (None, None),
            Some("p" | "b") => (Some('ᆷ'), Some("m")),
            Some("t" | "d" | "ts" | "dz" | "tʂ" | "dʐ") => (Some('ᆫ'), Some("n")),
            Some("tɕ" | "dʑ") => (Some('ᆫ'), Some("ɲ")),
            Some("k" | "g") => (Some('ᆼ'), Some("ŋ")),
            _ => (Some('ᇰ'), None),
        };
        out.push(Ph { nasal: tail, ..*p });
        if let Some(k) = cons {
            out.push(Ph { k, vowel: false, nasal: tail, rz: false, dev: false });
        }
    }
    out
}

fn lead(k: &str) -> char {
    match k {
        "p" => 'ᄑ',
        "b" => 'ᄇ',
        "t" => 'ᄐ',
        "d" => 'ᄃ',
        "k" => 'ᄏ',
        "g" => 'ᄀ',
        "f" => 'ᅗ',
        "v" => 'ᄫ',
        "s" | "ʂ" | "ɕ" => 'ᄉ',
        "z" | "ʐ" | "ʑ" => 'ᅀ',
        "ts" | "tʂ" | "tɕ" => 'ᄎ',
        "dz" | "dʐ" | "dʑ" => 'ᄌ',
        "m" => 'ᄆ',
        "n" | "ɲ" => 'ᄂ',
        "l" | "r" => 'ᄅ',
        "x" => 'ᅘ',
        _ => 'ᄋ',
    }
}

/// 모음 없이 선 자음의 블록 (낱말 끝의 권설음은 ᄎᆜ ᄉᆜ).
fn alone(k: &str, word_final: bool) -> String {
    match k {
        "ʂ" if word_final => "ᄉᆜ".into(),
        "ʂ" => "슈".into(),
        "tʂ" if word_final => "ᄎᆜ".into(),
        "tʂ" => "츠ᇫ".into(),
        "ʐ" => "ᅀᅴ".into(),
        "dʐ" => "즈ᇫ".into(),
        "tɕ" | "dʑ" | "ɕ" | "ʑ" | "ɲ" => format!("{}ᆛ", lead(k)),
        "w" => "우".into(),
        "j" => "이".into(),
        _ => format!("{}ᅳ", lead(k)),
    }
}

fn plain(vk: &str) -> char {
    match vk {
        "a" => 'ᅡ',
        "e" => 'ᅦ',
        "i" => 'ᅵ',
        "o" | "ą" => 'ᆟ',
        "u" => 'ᅮ',
        "ę" => 'ᅢ',
        _ => 'ᆜ',
    }
}
fn with_j(vk: &str) -> char {
    match vk {
        "a" => 'ᅣ',
        "e" => 'ᅨ',
        "o" | "ą" => 'ᅭ',
        "u" => 'ᅲ',
        "ę" => 'ᅤ',
        other => plain(other),
    }
}
fn with_w(vk: &str) -> char {
    match vk {
        "a" => 'ᅪ',
        "e" => 'ᅰ',
        "o" | "ą" => 'ᅯ',
        "i" | "u" => 'ᅱ',
        "ę" => 'ᅫ',
        other => plain(other),
    }
}

fn compose(s: &str) -> String {
    let cs: Vec<char> = s.chars().collect();
    let l = |x: char| ('\u{1100}'..='\u{1112}').contains(&x);
    let vw = |x: char| ('\u{1161}'..='\u{1175}').contains(&x);
    let t = |x: char| ('\u{11A8}'..='\u{11C2}').contains(&x);
    match cs[..] {
        [a, b] if l(a) && vw(b) => char::from_u32(0xAC00 + (a as u32 - 0x1100) * 588 + (b as u32 - 0x1161) * 28).map(String::from).unwrap_or(s.into()),
        [a, b, d] if l(a) && vw(b) && t(d) => char::from_u32(0xAC00 + (a as u32 - 0x1100) * 588 + (b as u32 - 0x1161) * 28 + (d as u32 - 0x11A7)).map(String::from).unwrap_or(s.into()),
        _ => s.to_string(),
    }
}

fn ipa(k: &'static str) -> Segment {
    match k {
        "r" => Segment::new("ɾ"),
        // 폴란드어 무성 파열·파찰음은 기식이 없다
        "p" | "t" | "k" | "ts" | "tʂ" | "tɕ" => Segment { aspirated: Some(false), ..Segment::new(k) },
        _ => Segment::new(k),
    }
}

fn syllables(ps: &[Ph], prefix: usize, nie: bool) -> Vec<Syllable> {
    let vi: Vec<usize> = (0..ps.len()).filter(|&i| ps[i].vowel).collect();
    let mut out: Vec<Syllable> = Vec::new();
    // 앞 음절의 마지막 블록이 받침 없이 열려 있나 (l 의 겹ㄹ 표기용)
    let mut open = false;
    let mut start = 0;
    for (n, &i) in vi.iter().enumerate() {
        // 이 모음 뒤, 다음 모음 앞의 자음: ą ę 가 세운 비음과, 둘 이상일 때 첫 공명음은 이 음절의 받침
        let end = vi.get(n + 1).copied().unwrap_or(ps.len());
        let between = &ps[i + 1..end];
        let coda_n = if n + 1 == vi.len() {
            between.len()
        } else {
            let glued = between.first().is_some_and(|p| p.nasal.is_some()) as usize;
            // 공명음 + 자음은 받침과 초성으로 갈린다 (Ta-jem-ny, gar-dła); 자음 + j·w 는 한 초성 (i-mię)
            let rest = &between[glued..];
            let son = rest.len() >= 2 && matches!(rest[0].k, "m" | "n" | "ɲ" | "l" | "r" | "w" | "j") && !(rest.len() == 2 && matches!(rest[1].k, "j" | "w"));
            glued + son as usize
        };
        let onset = &ps[start..i];
        let coda = &between[..coda_n];
        start = i + 1 + coda_n;
        let vk = ps[i].k;
        let last = n + 1 == vi.len();

        // ---- 글자
        let mut blocks: Vec<String> = Vec::new();
        // 모음 없는 앞 낱말 (w z): 소리는 이 음절의 초성이지만 글자는 제 블록
        let pre = if n == 0 { prefix.min(onset.len()) } else { 0 };
        for p in &onset[..pre] {
            blocks.push(alone(p.k, false));
        }
        let onset_w = &onset[pre..];
        let glide = onset_w.last().filter(|p| matches!(p.k, "j" | "w")).map(|p| p.k);
        let cons = &onset_w[..onset_w.len() - glide.is_some() as usize];
        for p in cons.iter().take(cons.len().saturating_sub(1)) {
            blocks.push(alone(p.k, false));
        }
        let main = cons.last().map(|p| if p.rz { "ʐ" } else { p.k });
        // 받침 글자: ą ę 의 비음, 아니면 첫 받침 자음이 l m n 일 때 그것
        let tail: Option<char> = ps[i].nasal.or(match coda.first().map(|p| p.k) {
            Some("l") => Some('ᆯ'),
            Some("m") => Some('ᆷ'),
            Some("n") => Some('ᆫ'),
            _ => None,
        });
        let tail_from_coda = ps[i].nasal.is_none() && tail.is_some();
        let bare = |v: char| format!("ᄋ{v}");
        // l 은 겹ㄹ: 앞 블록이 받침 없이 열려 있으면 거기에도 ㄹ (Ale 알레, blask 블라스크)
        if main == Some("l") {
            if let Some(b) = blocks.last_mut() {
                *b = regroup(b, 'ᆯ');
            } else if open && let Some(prev) = out.last_mut() {
                prev.text = regroup(&prev.text, 'ᆯ');
            }
        }
        let mut block = match (main, glide) {
            (None, None) if vk == "y" => "ᅙᆜ".to_string(),
            (None, None) => bare(plain(vk)),
            (None, Some("j")) => bare(with_j(vk)),
            (None, _) if matches!(vk, "y" | "u") => {
                blocks.push("우".into());
                if vk == "y" { "ᅙᆜ".to_string() } else { bare(plain(vk)) }
            }
            (None, _) => bare(with_w(vk)),
            // 자음 + ł + y·u: 자음+ㅜ 블록, 이어 모음 (błysk 부ᅙᆜ슼)
            (Some(k), Some("w")) if matches!(vk, "y" | "u") && !matches!(k, "ʂ" | "tʂ" | "ʐ" | "dʐ" | "tɕ" | "dʑ" | "ɕ" | "ʑ" | "ɲ") => {
                blocks.push(format!("{}ᅮ", lead(k)));
                if vk == "y" { "ᅙᆜ".to_string() } else { bare(plain(vk)) }
            }
            // 경구개 ć ś ź dź ń: ㅣ 앞에서는 치 시 ᅀᅵ 지 니, 다른 모음 앞에서는 ᄎᆛ + 모음 (ᄎᆛ에, ᄂᆛ에); 낱말 nie 는 녜
            (Some("ɲ"), None) if nie => "녜".to_string(),
            (Some(k @ ("tɕ" | "dʑ" | "ɕ" | "ʑ" | "ɲ")), g) => {
                if vk == "i" && g.is_none() {
                    format!("{}ᅵ", lead(k))
                } else {
                    blocks.push(alone(k, false));
                    bare(match g {
                        Some("w") => with_w(vk),
                        Some(_) => with_j(vk),
                        None => plain(vk),
                    })
                }
            }
            // 권설 cz sz rz dż: y 와 만나면 ᄎᆜ ᄉᆜ ᅀᆜ ᄌᆜ
            (Some(k @ ("ʂ" | "tʂ" | "ʐ" | "dʐ")), None) if vk == "y" => format!("{}ᆜ", lead(k)),
            (Some("ʂ"), g) => {
                blocks.push("슈".into());
                bare(match g {
                    Some("w") => with_w(vk),
                    Some(_) => with_j(vk),
                    None => plain(vk),
                })
            }
            (Some("ʐ"), None) => format!("ᅀ{}", with_w(vk)),
            // cz dż + 모음: 체ᇫ 제ᇫ (받침 ㅿ 이 권설 표지); 받침 자리가 따로 쓰이면 ᄎᆜ + 모음 (często ᄎᆜ앵)
            (Some(k @ ("tʂ" | "dʐ")), None) if tail.is_none() => format!("{}{}ᇫ", lead(k), plain(vk)),
            (Some(k @ ("tʂ" | "dʐ" | "ʐ")), g) => {
                blocks.push(format!("{}ᆜ", lead(k)));
                bare(match g {
                    Some("w") => with_w(vk),
                    Some(_) => with_j(vk),
                    None => plain(vk),
                })
            }
            // 자음 + i + 모음: 자음+ㅣ 블록, 이어 모음 (키에, ᄫᅵ에, 피앵)
            (Some(k), Some("j")) => {
                blocks.push(format!("{}ᅵ", lead(k)));
                bare(plain(vk))
            }
            (Some(k), Some(_)) => format!("{}{}", lead(k), with_w(vk)),
            (Some(k), None) => format!("{}{}", lead(k), plain(vk)),
        };
        if let Some(t) = tail {
            block.push(t);
        }
        let mut after: Vec<String> = Vec::new();
        for (j, p) in coda.iter().enumerate() {
            if p.nasal.is_some() || (j == 0 && tail_from_coda) {
                continue;
            }
            let word_final = last && j + 1 == coda.len();
            after.push(match p.k {
                "ɕ" if p.dev => "ᄉᆜ".into(),
                "tɕ" if p.dev => "ᄎᆜ".into(),
                k => alone(k, word_final),
            });
        }
        open = tail.is_none() && after.is_empty() && !block.ends_with('ᇫ');
        blocks.push(block);
        blocks.extend(after);
        let text: String = blocks.iter().map(|b| compose(b)).collect();

        // ---- 소리
        let onset_seg: Vec<Segment> = onset.iter().map(|p| ipa(p.k)).collect();
        let nucleus = Segment {
            nasalized: (ps[i].nasal == Some('ᇰ')).then_some(true),
            ..Segment::new(match vk {
                "a" => "a",
                "e" | "ę" => "ɛ",
                "i" => "i",
                "o" | "ą" => "ɔ",
                "u" => "u",
                _ => "ɨ",
            })
        };
        let coda_seg: Vec<Segment> = coda.iter().map(|p| ipa(p.k)).collect();
        out.push(Syllable { text, onset: onset_seg, nucleus, coda: coda_seg, ..Default::default() });
    }
    out
}

/// 음절 글자의 마지막 블록에 받침을 더한다 (완성형이면 풀어서).
fn regroup(text: &str, tail: char) -> String {
    let mut cs: Vec<char> = text.chars().collect();
    let Some(&lastc) = cs.last() else { return text.to_string() };
    let u = lastc as u32;
    if (0xAC00..=0xD7A3).contains(&u) && (u - 0xAC00) % 28 == 0 {
        *cs.last_mut().unwrap() = char::from_u32(u + (tail as u32 - 0x11A7)).unwrap_or(lastc);
    } else if ('\u{1161}'..='\u{11A7}').contains(&lastc) || ('\u{D7B0}'..='\u{D7C6}').contains(&lastc) {
        cs.push(tail);
    }
    cs.into_iter().collect()
}

fn read(prefix: Option<&str>, word: &str) -> Vec<Syllable> {
    let mut ps = prefix.map(phonemes).unwrap_or_default();
    let n = ps.len();
    ps.extend(phonemes(word));
    assimilate(&mut ps);
    let ps = nasals(ps);
    syllables(&ps, n, word.eq_ignore_ascii_case("nie"))
}

fn vowelless(word: &str) -> bool {
    !phonemes(word).iter().any(|p| p.vowel)
}

impl FrontEnd for JeongeumPolish {
    fn starts(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic() || is_polish_letter(ch)
    }
    fn claims(&self, word: &str) -> bool {
        // 폴란드어 글자가 없어도 sz cz rz 가 든 낱말은 폴란드어로 본다 (Twarz, Szczebrzeszyn)
        let w = word.to_lowercase();
        self.whole || w.chars().any(is_polish_letter) || ["sz", "cz", "rz"].iter().any(|d| w.contains(d))
    }
    fn word(&self, word: &str) -> Vec<Syllable> {
        read(None, word)
    }
    /// 모음 없는 낱말 (w z): 제 음표가 없고, 뒤 낱말의 첫 음절 앞에 붙는다 (w Szczebrzeszynie [fʂtʂ…]).
    fn word_after(&self, word: &str, prev: Option<&str>) -> Vec<Syllable> {
        if vowelless(word) {
            return Vec::new();
        }
        read(prev.filter(|p| vowelless(p)), word)
    }
}
