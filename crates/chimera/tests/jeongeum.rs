//! The 正音 front ends (fork): reproductions of the bugs found by ear on the deployed page.
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::syllables::lyrics_to_syllables;

fn read(text: &str) -> Vec<(String, Vec<String>, String)> {
    let g2p = EnglishG2P::new();
    lyrics_to_syllables(text, &g2p)
        .into_iter()
        .map(|s| (s.text, s.onset.into_iter().map(|o| o.ipa).collect(), s.nucleus.ipa))
        .collect()
}

#[test]
fn a_lone_consonant_letter_is_read_with_the_vowel_of_heu_arae_a() {
    // ㅸ typed alone (compatibility jamo, as a keyboard gives it) sang nothing: no vowel, and the
    // router didn't claim the letter. 正音 reads a lone consonant with ᅙᆞ's vowel [ə]
    for (typed, onset) in [("ㅸ", "β"), ("ᄫ", "β"), ("ㄱ", "k"), ("ㆆ", "ʔ")] {
        let s = read(typed);
        assert_eq!(s.len(), 1, "{typed}");
        assert_eq!((s[0].1.as_slice(), s[0].2.as_str()), ([onset.to_string()].as_slice(), "ə"), "{typed}");
    }
    // after a syllable, ㆆ ㅿ ㆁ ㅱ are still its tail (간ㆆ = 간ᇹ)
    assert_eq!(read("간ㆆ").len(), 1);
}

#[test]
fn sanskrit_follows_the_books_spellings() {
    let text: Vec<String> = read("गांधी पारमिता भूमि विभेद घोंघा").into_iter().map(|s| s.0).collect();
    assert_eq!(jamo(&text.concat()), jamo("간ᄃᆜ빠러미ᇹ따부후미ᄫᅵ베더고호ᇰᄀᆣ"));
}

/// precomposed blocks as their jamo, so a spelling compares however its blocks are encoded
fn jamo(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            let u = c as u32;
            if (0xAC00..=0xD7A3).contains(&u) {
                let i = u - 0xAC00;
                let mut v = vec![char::from_u32(0x1100 + i / 588).unwrap(), char::from_u32(0x1161 + (i % 588) / 28).unwrap()];
                if i % 28 != 0 {
                    v.push(char::from_u32(0x11A7 + i % 28).unwrap());
                }
                v
            } else {
                vec![c]
            }
        })
        .collect()
}

#[test]
fn the_vowel_chart_matches_the_users_table() {
    // 사용자 지정 2026-11-23: 모음사각도의 IPA 모음과 正音 모음
    for (typed, vowel) in [
        ("ᅟᆚ", "ø"), ("ᅟᆛ", "y"), ("ㆉ", "œ"), ("ᅟᆟ", "ɔ"), ("ᅟᆗ", "ʊ"), ("ᅙᅵ", "ɪ"), ("ᅟᅵᇹ", "ɪ"),
        ("ᅟힺ", "ɘ"), ("에", "e̞"), ("이", "i"), ("ᅙᅦ", "e"), ("애", "ɛ"), ("아", "ɐ"), ("ᅟퟅ", "ɑ"), ("ᅙᆞ", "ə"), ("오", "o̞"), ("ᅟᆞᅢ", "æ"), ("ㆍㅐ", "æ"),
    ] {
        let s = read(typed);
        assert_eq!(s.len(), 1, "{typed}");
        // ㆆ marks the vowel here: no [ʔ] before or after it
        assert!(s[0].1.is_empty(), "{typed}: {:?}", s[0].1);
        assert_eq!(s[0].2, vowel, "{typed}");
    }
}
