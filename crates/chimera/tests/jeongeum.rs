//! The 正音 front ends (fork): reproductions of the bugs found by ear on the deployed page.
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::syllables::lyrics_to_syllables;

fn read(text: &str) -> Vec<(String, Vec<String>, String)> {
    let g2p = EnglishG2P::new();
    lyrics_to_syllables(text, &g2p)
        .into_iter()
        // the vowel without 正音's ◌͙ key mark (`jeongeum::beck`): the tests read IPA
        .map(|s| (s.text, s.onset.into_iter().map(|o| o.ipa).collect(), s.nucleus.ipa.replace('\u{359}', "")))
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
        ("ᅟힺ", "ɘ"), ("에", "e̞"), ("이", "i"), ("ᅙᅦ", "e"), ("애", "ɛ"), ("아", "ɐ"), ("ᅟퟅ", "ɑ̠"), ("ᅙᆞ", "ə"), ("오", "o̞"), ("ᅟᆞᅢ", "æ"), ("ㆍㅐ", "æ"), ("ᅟᆜᅮ", "ʏ"), ("ᅟᅵᅳᅮ", "ʏ"),
    ] {
        let s = read(typed);
        assert_eq!(s.len(), 1, "{typed}");
        // ㆆ marks the vowel here: no [ʔ] before or after it
        assert!(s[0].1.is_empty(), "{typed}: {:?}", s[0].1);
        assert_eq!(s[0].2, vowel, "{typed}");
    }
}

#[test]
fn the_cardinal_vowels_keep_the_charts_order() {
    // 正音's vowels come from one speaker (J. Beck's cardinal vowels, Seeing Speech), so going down
    // each column of the IPA chart, the calibrated F1 rises: the bug was o̞ (Beck) under o (Hillenbrand)
    use chimera::phonetics::planner::vowel_table;
    let columns: [&[&str]; 4] = [
        &["i\u{359}", "e\u{359}", "e̞", "ɛ\u{359}", "a\u{359}"],
        &["y\u{359}", "ø\u{359}", "ø̞", "œ\u{359}", "ɶ"],
        &["ɯ̠", "ɤ̠", "ɤ̞", "ʌ̠", "ɑ̠"],
        &["u\u{359}", "o\u{359}", "o̞", "ɔ\u{359}", "ɒ\u{359}"],
    ];
    for (body, table) in vowel_table() {
        for col in columns {
            let f1: Vec<f64> = col.iter().map(|v| table.get(*v).unwrap_or_else(|| panic!("{body} {v}")).formants[0]).collect();
            assert!(f1.windows(2).all(|w| w[0] < w[1]), "{body} {col:?}: F1 {f1:?}");
        }
    }
    // and typed 正音 reaches them
    let g2p = chimera::phonetics::english::EnglishG2P::new();
    let raw = |t: &str| lyrics_to_syllables(t, &g2p)[0].nucleus.ipa.clone();
    assert_eq!(raw("ᅙᅦ"), "e\u{359}");
    assert_eq!(raw("오"), "o̞");
}
