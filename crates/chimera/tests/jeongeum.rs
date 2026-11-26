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
        ("ᅟᆚ", "ø"), ("ᅟᆛ", "y"), ("ㆉ", "œ"), ("ᅟᆟ", "ɔ̞"), ("ᅙᅵ", "ɪ"), ("ᅟᅵᇹ", "ɪ"),
        ("ᅟힺ", "ɘ"), ("에", "e̞"), ("이", "i"), ("애", "ɛ"), ("아", "a"), ("ᅟퟅ", "ɑ"), ("ᅙᆞ", "ə"), ("오", "o̞"), ("ᅟᆞᅢ", "æ"), ("ㆍㅐ", "æ"), ("ᅟᆜᅮ", "ʏ"), ("ᅟᅵᅳᅮ", "ʏ"),
    ] {
        let s = read(typed);
        assert_eq!(s.len(), 1, "{typed}");
        // ㆆ marks the vowel here: no [ʔ] before or after it
        assert!(s[0].1.is_empty(), "{typed}: {:?}", s[0].1);
        assert_eq!(s[0].2, vowel, "{typed}");
    }
}

#[test]
fn the_female_bodys_vowels_keep_the_charts_order() {
    // the female body's vowels are one speaker's (J. Beck's cardinal vowels, Seeing Speech), one
    // value per symbol, so going down each column of the IPA chart its calibrated F1 rises: the bug
    // was o̞ (Beck) under o (Hillenbrand). The male and child bodies are upstream's, for now
    use chimera::phonetics::planner::vowel_table;
    let columns: [&[&str]; 5] = [
        &["i", "e", "e̞", "ɛ", "a"],
        &["y", "ø", "ø̞", "œ", "ɶ"],
        &["ɯ", "ɤ", "ɤ̞", "ʌ", "ɑ"],
        &["u", "o", "o̞", "ɔ", "ɔ̞", "ɒ"],
        &["ɨ", "ɘ", "ə", "ɐ", "ɐ̞"],
    ];
    let tables = vowel_table();
    let table = &tables["female"];
    for col in columns {
        let f1: Vec<f64> = col.iter().map(|v| table.get(*v).unwrap_or_else(|| panic!("{v}")).formants[0]).collect();
        assert!(f1.windows(2).all(|w| w[0] < w[1]), "{col:?}: F1 {f1:?}");
    }
    // one entry per symbol: no ◌͙ or ◌̠ copies of a chart vowel
    assert!(!table.keys().any(|k| k.contains('\u{359}') || ["ɤ̠", "ʌ̠", "ɯ̠", "ɑ̠"].contains(&k.as_str())));
}

#[test]
fn the_jeongeum_vowels_take_their_symbol_from_the_syllable() {
    // 사용자 지정 2026-11-23. ㅔ [e̞] whatever the onset; ㅐ [ɛ]; ㅚ [ø̞]; ㅏ [a] (2026-11-26; was [a]); ퟅ [ɑ]; ᆟ [ɔ̞].
    // ㅣ ㅡ ㅓ ㅗ by the coda: none or a 정음 sonorant (ㄴ ㄹ ㅁ ㅿ ㆁ, ㅇ) → i ɤ ʌ o̞, any other → ɪ ɯ ɤ̞ o
    for (typed, v) in [
        ("에", "e̞"), ("게", "e̞"), ("세", "e̞"), ("네", "e̞"), ("애", "ɛ"), ("외", "ø̞"), ("아", "a"), ("악", "a"), ("ᅟퟅ", "ɑ"), ("ᅟᆟ", "ɔ̞"),
        ("이", "i"), ("인", "i"), ("잉", "i"), ("익", "ɪ"), ("잎", "ɪ"), ("있", "ɪ"),
        ("으", "ɤ"), ("그", "ɤ"), ("은", "ɤ"), ("극", "ɯ"), ("윽", "ɯ"),
        ("어", "ʌ"), ("거", "ʌ"), ("언", "ʌ"), ("업", "ɤ̞"), ("걱", "ɤ̞"),
        ("오", "o̞"), ("고", "o̞"), ("온", "o̞"), ("옥", "o"), ("곡", "o"),
    ] {
        assert_eq!(read(typed)[0].2, v, "{typed}");
    }
    // ㅔ is e̞ everywhere: ᅙᅦ is [ʔe̞], ㆆ sounding as usual
    let s = read("ᅙᅦ");
    assert_eq!((s[0].1.as_slice(), s[0].2.as_str()), (["ʔ".to_string()].as_slice(), "e̞"));
    // ᅟᆗ alone is [ʔʊ]
    let s = read("ᅟᆗ");
    assert_eq!((s[0].1.as_slice(), s[0].2.as_str()), (["ʔ".to_string()].as_slice(), "ʊ"));
    // ㆍ still by the onset: ə, ɑ after a 정음 sonorant (ᄛ ㅿ count)
    for (typed, v) in [("ᄇᆞ", "ə"), ("ᄂᆞ", "ɑ"), ("ᅀᆞ", "ɑ"), ("ᄛᆞ", "ɑ")] {
        assert_eq!(read(typed)[0].2, v, "{typed}");
    }
}

#[test]
fn english_cup_and_jeongeum_keop_send_their_own_symbols() {
    // English keeps the symbol upstream registered for it ("cup" is ʌ; 사용자 지정 2026-11-24, undoing
    // the fork's ɐ); 正音 컵 is ㅓ before ㅂ = [ɤ̞]
    let g2p = EnglishG2P::new();
    let n = |t: &str| lyrics_to_syllables(t, &g2p)[0].nucleus.ipa.clone();
    assert_eq!(n("cup"), "ʌ");
    assert_eq!(n("컵"), "ɤ̞");
}

#[test]
fn eo_with_a_bansiot_coda_is_the_r_coloured_vowel() {
    // 사용자 지정: ᅟᅥᇫ = [ɝ] (English ɜːr); ɜ itself has no 正音 spelling. Hanzi zi ci si keep the apical ɿ
    for typed in ["ᅟᅥᇫ", "버ᇫ"] {
        assert_eq!(read(typed)[0].2, "ɝ", "{typed}");
    }
}

fn read_pl(text: &str) -> (String, String) {
    let syl = lyrics_to_syllables(text, &EnglishG2P::new());
    let t: Vec<String> = syl.iter().map(|s| s.text.clone()).collect();
    let ipa: Vec<String> = syl
        .iter()
        .map(|s| s.onset.iter().chain(std::iter::once(&s.nucleus)).chain(s.coda.iter()).map(|x| x.ipa.as_str()).collect::<String>())
        .collect();
    (t.concat(), ipa.join(" "))
}

/// 폴란드어 → 正音 (《正音으로 듣고쓰다》 217–218쪽): 책의 표와 예시의 표기, 폴란드어의 소리.
#[test]
fn polish_is_written_in_jeongeum_and_sounds_polish() {
    for (pl, text, ipa) in [
        ("Szczebrzeszyn", "슈체ᇫ브ᅀᅰᄉᆜᆫ", "ʂtʂɛ bʐɛ ʂɨn"),
        ("odpowiedź", "ᄋᆟ트ᄑᆟᄫᅵ에ᄎᆜ", "ɔ tpɔ vjɛtɕ"),
        ("Również", "루ᄫᅳᄂᆛ에ᄉᆜ", "ɾu vɲɛʂ"),
        ("imię", "이미애", "i mjɛ"),
        ("zęby", "ᅀᅢᆷᄇᆜ", "zɛm bɨ"),
        ("pięć", "피앤ᄎᆛ", "pjɛɲtɕ"),
        ("piękny", "피앵크ᄂᆜ", "pjɛŋ knɨ"),
        ("błysk", "부ᅙᆜ스크", "bwɨsk"),
        ("światła", "ᄉᆛᅗᅵ아톼", "ɕfja twa"),
        ("Tajemny król", "타옘ᄂᆜ크룰", "ta jɛm nɨ kɾul"),
        ("Że nie", "ᅀᅰ녜", "ʐɛ ɲɛ"),
    ] {
        let (t, i) = read_pl(pl);
        assert_eq!(jamo(&t), jamo(text), "{pl}");
        assert_eq!(i, ipa, "{pl}");
    }
    // ą ę 앞뒤: 마찰음 앞은 비모음, 낱말 끝과 무성음 앞의 유성음은 무성음
    let syl = lyrics_to_syllables("mąż", &EnglishG2P::new());
    assert_eq!(syl[0].nucleus.nasalized, Some(true));
    assert_eq!(syl[0].coda[0].ipa, "ʂ");
    // 모음 없는 낱말 (w z)은 뒤 낱말의 첫소리에 붙는다
    assert_eq!(read_pl("w każdym").1, "fka ʐdɨm");
    // 폴란드어 글자가 든 가사는 로마자 낱말을 모두 폴란드어로; 없으면 영어 그대로
    assert_eq!(read_pl("Pan się").1.split(' ').next(), Some("pan"));
}

/// ᆜ = j + 숨 섞인 ㅡ (ㅛ = j + ㅗ 처럼), [ɨ] 는 ᅙᆜ (사용자 지정 2026-11-24).
#[test]
fn typed_i_eu_is_j_then_a_breathy_eu() {
    for (w, ipa) in [("ᄉᆜ", "ɕjɤ"), ("ᄃᆜ", "tjɤ"), ("ᄂᆜ", "njɤ"), ("ᅟᆜ", "jɤ"), ("ᄉᆜᆫ", "ɕjɤn"), ("ᄉᆜᆨ", "ɕjɯk"), ("ᅙᆜ", "ʔɨ")] {
        assert_eq!(read_pl(w).1, ipa, "{w}");
        let syl = lyrics_to_syllables(w, &EnglishG2P::new());
        assert_eq!(syl[0].nucleus.breathy, if w == "ᅙᆜ" { None } else { Some(true) }, "{w}");
    }
}

/// ᅼ ᆣ 의 숨은 모음 앞 (터진 직후); 구절 끝에서만 모음 뒤에도 숨이 따른다 (사용자 지정 2026-11-24).
#[test]
fn a_breathy_syllable_breathes_out_after_its_vowel_only_at_a_phrase_end() {
    let syl = lyrics_to_syllables("ᄃᅼ ᄀᅼ, ᄇᅼ 가 ᄃᅼ", &EnglishG2P::new());
    let codas: Vec<String> = syl.iter().map(|s| s.coda.iter().map(|c| c.ipa.as_str()).collect()).collect();
    assert_eq!(codas, ["", "h", "", "", "h"]);
    let breathy: Vec<bool> = syl.iter().map(|s| s.onset[0].breathy == Some(true)).collect();
    assert_eq!(breathy, [true, true, true, false, true], "{:?}", syl.iter().map(|s| s.text.clone()).collect::<Vec<_>>());
}

/// 받침 ㆁ 은 프랑스어 비모음의 표지: 중성 + ㆁ 을 한 묶음으로 비모음 다섯에 잇는다 (사용자 지정 2026-11-24).
#[test]
fn a_vowel_with_a_yesieung_coda_is_one_french_nasal_vowel() {
    for (w, onset, v) in [("커ᇰ", "k", "ɒ"), ("ᄏힺᇰ", "k", "ɒ"), ("ᅟힺᇰ", "", "ɒ"), ("ᄫᅡᇰ", "β", "ɛ"), ("패ᇰ", "p", "ɛ"), ("보ᇰ", "p", "ɔ"), ("ᅟᆟᇰ", "", "œ"), ("롸ᇰ", "", "ɛ")] {
        let syl = lyrics_to_syllables(w, &EnglishG2P::new());
        assert_eq!(syl.len(), 1, "{w}");
        assert_eq!(syl[0].nucleus.ipa, v, "{w}");
        assert_eq!(syl[0].nucleus.nasalized, Some(true), "{w}");
        assert!(syl[0].coda.is_empty(), "{w}: {:?}", syl[0].coda);
        if !onset.is_empty() {
            assert_eq!(syl[0].onset[0].ipa, onset, "{w}");
        }
    }
    // 다섯 묶음 밖의 모음 + ㆁ 은 正音에 없는 표기: 소리를 지어내지 않는다
    for w in ["으ᇰ", "우ᇰ", "에ᇰ", "이ᇰ"] {
        assert!(lyrics_to_syllables(w, &EnglishG2P::new()).is_empty(), "{w}");
    }
    // 받침 ㅇ 은 그대로 [ŋ]
    let syl = lyrics_to_syllables("강", &EnglishG2P::new());
    assert_eq!(syl[0].coda[0].ipa, "ŋ");
    assert_eq!(syl[0].nucleus.nasalized, None);
}

/// 포르투갈어 비이중모음: ᆟ + ᅌᅮ = [ɐ̃w] 한 음절 (coração 꼬라ᄉᆟᅌᅮ, 사용자 지정 2026-11-24).
#[test]
fn eo_then_ngu_is_the_portuguese_nasal_diphthong() {
    let syl = lyrics_to_syllables("꼬라ᄉᆟᅌᅮ", &EnglishG2P::new());
    assert_eq!(syl.len(), 3);
    let last = &syl[2];
    assert_eq!(jamo(&last.text), jamo("ᄉᆟᅌᅮ"));
    assert_eq!((last.onset[0].ipa.as_str(), last.nucleus.ipa.as_str(), last.coda[0].ipa.as_str()), ("s", "ɐ", "w"));
    assert_eq!(last.nucleus.nasalized, Some(true));
    // ᆟ 홀로는 그대로 [ɔ̞], ᅌᅮ 홀로도 그대로
    assert_eq!(read_pl("ᄉᆟ 우").1, "sɔ̞ u");
}

/// θ ð 는 초성 ㅌ ㄷ + 모음의 ㅡ + 받침 ㅿ 세 성분이 다 있을 때만; ㅡ 가 없으면 받침 ㅿ 은 권설 (사용자 지정 2026-11-24).
#[test]
fn the_dental_fricatives_need_all_three_parts() {
    for (w, onset, v) in [("ᄐힺᇫ", "θ", "ʌ"), ("ᄐힹᇫ", "θ", "a"), ("타ᇫ", "ʈ", "a"), ("터ᇫ", "ʈ", "ʌ"), ("ᄃힺᇫ", "ð", "ʌ"), ("다ᇫ", "ɖ", "a"), ("따ᇫ", "ʈ", "a")] {
        let syl = lyrics_to_syllables(w, &EnglishG2P::new());
        assert_eq!(syl[0].onset[0].ipa, onset, "{w}");
        assert_eq!(syl[0].nucleus.ipa, v, "{w}");
        assert!(syl[0].coda.is_empty(), "{w}");
    }
    // 세 자리 code 에 붙은 ㅣ 는 [ɪ]; 그 ㅿ 은 받침이 아니어서 다른 모음의 받침 규칙에도 들지 않는다
    for (w, ipa) in [("듸ᇫ", "ðɪ"), ("틔ᇫ", "θɪ"), ("틔ᇫᆼ", "θɪŋ"), ("ᄃힺᇫ", "ðʌ"), ("ᄐힺᇫᆨ", "θɤ̞k")] {
        assert_eq!(read_pl(w).1, ipa, "{w}");
    }
    let syl = lyrics_to_syllables("타ᇫ 따ᇫ", &EnglishG2P::new());
    assert_eq!((syl[0].onset[0].aspirated, syl[1].onset[0].aspirated), (Some(true), Some(false)));
}

/// 낱자 ㅋ ㅌ ㅅ ㅎ 은 모음 없이 자음으로만 읽는다; ㅡ 를 적으면 [ɤ] 가 난다 (사용자 지정 2026-11-24).
/// 듸ᇫㅅ = 영어 this [ðɪs], 듸ᇫ스 = 한국인이 외래어로 읽는 [ðɪ sɤ].
#[test]
fn a_lone_consonant_letter_is_a_bare_consonant() {
    for (w, ipa) in [("듸ᇫㅅ", "ðɪs"), ("듸ᇫ스", "ðɪ sɤ"), ("틔ᇫᆼㅋ", "θɪŋk"), ("ㅅ타", "sta"), ("버스", "pʌ sɤ"), ("ㅸ", "βə"), ("ㆄ", "fə")] {
        assert_eq!(read_pl(w).1, ipa, "{w}");
    }
}

/// 받침 ㅿ = 권설음화: 혀끝을 쓰지 않는 초성에서는 모음 뒤의 권설 접근음 [ɻ] (파ᇫ = par; 사용자 지정 2026-11-24).
#[test]
fn a_bansiot_coda_after_a_non_coronal_onset_is_a_retroflex_approximant() {
    for (w, ipa) in [("파ᇫ", "paɻ"), ("가ᇫ", "kaɻ"), ("아ᇫ", "aɻ"), ("모ᇫ", "mo̞ɻ"), ("버ᇫ", "pɝ"), ("어ᇫ", "ɝ"), ("다ᇫ", "ɖa")] {
        assert_eq!(read_pl(w).1, ipa, "{w}");
    }
}

/// 혀끝 초성 + 받침 ㅿ 은 초성이 권설음: ㄴ ɳ · ㅅ ʂ · ㅈ dʐ (유성) · ㅊ tʂ. 받침 ㄴㅿ(ᇈ) 은 권설 비음 [ɳ]
/// (산스크리트 jaṇ = 저ᇈ). 북경어의 즤 츼 는 ㅢ 로 적어 겹치지 않는다 (사용자 지정 2026-11-24).
#[test]
fn a_bansiot_coda_makes_a_coronal_onset_retroflex() {
    for (w, ipa) in [("나ᇫ", "ɳa"), ("사ᇫ", "ʂa"), ("자ᇫ", "dʐa"), ("저ᇫ", "dʐʌ"), ("차ᇫ", "tʂa"), ("저ᇈ", "dʐʌɳ"), ("셔ᇈ", "ʂʌɳ"), ("가ᇈ", "kaɳ"), ("떠ᇫ", "ʈʌ"), ("터ᇫ", "ʈʌ"), ("더ᇫ", "ɖʌ"), ("셔ᇫ", "ʂʌ"), ("즤", "tʂʅ"), ("저", "tɕʌ")] {
        assert_eq!(read_pl(w).1, ipa, "{w}");
    }
}

/// ㅡ 는 유성 표지: ㅈ ㄷ ㅂ + ㅡ결합 모음은 유성음 + 남은 모음 (ᄌힺ = 산스크리트 ja, ᄌힹ = jā; 사용자 지정 2026-11-24).
#[test]
fn a_combined_eu_vowel_voices_its_onset() {
    for (w, ipa) in [("ᄌힺ", "dʑʌ"), ("ᄌힹ", "dʑa"), ("ᄃힹ", "da"), ("ᄇힺ", "bʌ"), ("저", "tɕʌ")] {
        assert_eq!(read_pl(w).1, ipa, "{w}");
    }
}

/// 한글 발음 규칙 둘만 正音 처리기에 넣는다 (ㅎ 거센소리 되기, 구개음화): 글은 그대로, 소리만.
/// 받침 ㅎ 이 인두 표지인 음절(중성에 ㆍ, 그리고 듷 ᅀᅳᇂ)과 겹받침 ㄶ ㅀ 은 건드리지 않는다.
#[test]
fn h_aspirates_its_neighbour_and_t_palatalises_before_i() {
    for (w, ipa) in [
        ("좋다", "tɕo̞ ta"),
        ("놓고", "no̞ ko̞"),
        ("좋아", "tɕo̞ a"),
        ("놓은", "no̞ ɤn"),
        ("입학", "i pak"),
        ("같이", "ka tɕi"),
        ("굳이", "ku dʑi"),
        ("닫히다", "ta tɕi da"),
        ("붙여", "pu tɕjʌ"),
    ] {
        assert_eq!(read_pl(w).1, ipa, "{w}");
    }
    // 인두 표지 ㅎ 뒤의 ㅂ ㄷ 은 그대로 (ᄄퟅᇂ빕 ṭabīb, 듷다)
    for (w, onset) in [("ᄄퟅᇂ빕", "p"), ("듷다", "t"), ("싫다", "t"), ("않다", "d")] {
        let syl = lyrics_to_syllables(w, &EnglishG2P::new());
        // 듷 = ðˤ 은 자음만인 음절이라 다 의 첫소리 앞에 붙는다 ([ðʕta]): ㄷ 은 첫소리의 마지막
        assert_eq!(syl.last().unwrap().onset.last().unwrap().ipa, onset, "{w}");
        assert_ne!(syl.last().unwrap().onset.last().unwrap().vot, Some(0.08), "{w}");
    }
}

/// 받침 ㅅ ㅆ, 연음, 된소리, 인두 표지 (사용자 지정 2026-11-26).
#[test]
fn codas_link_to_a_vowel_and_s_is_unreleased_at_a_word_end() {
    for (w, ipa) in [
        ("여섯", "jʌ sɤ̞t"),
        ("여섯은", "jʌ sɤ̞s ɤn"),
        ("했었지", "hɛt sɤ̞t tɕʔi"),
        ("먹어", "mʌ gʌ"),
        ("물이", "mu ɾi"),
        ("강이", "kaŋ i"),
        ("아까", "a kʔa"),
        ("아빠", "a pʔa"),
        ("오또", "o̞ tʔo̞"),
        ("가짜", "ka tɕʔa"),
        ("알ᄅퟅᇂ아", "al ɾɑʕ a"),
        ("듷아", "ðʕa"),
        ("ᅀᅳᇂ아", "zʕa"),
        ("ᄒퟅᇹ", "ħɑ"),
    ] {
        assert_eq!(read_pl(w).1, ipa, "{w}");
    }
}

/// 프랑스어 r: 받침 ㅎ + 다음 음절 ㄹ (Paris 빻리), 받을 음절이 없으면 겹받침 ㅎㄹ (merci 메ᇶ씨).
#[test]
fn the_french_r_is_h_then_l_or_the_hl_coda() {
    for (w, ipa) in [("빻리", "pʔa ʁi"), ("메ᇶ씨", "me̞ʁ ɕi"), ("멯ㄹ씨", "me̞ʁ ɕi")] {
        assert_eq!(read_pl(w).1, ipa, "{w}");
    }
}

/// Mood 페이지 네 조각 스위치의 ㅣ → ɨ (사용자 지정 2026-11-26): 꺼진 것이 기본이고, 켜면 ᆜ 가 [ɨ].
/// 나머지 세 조각은 자리만 있고 읽기를 바꾸지 않는다.
#[test]
fn the_i_switch_reads_the_yi_vowel_as_barred_i() {
    use chimera::phonetics::jeongeum::{MARK_I, set_marks};
    for (bits, w, ipa) in [(0, "ᄉᆜ", "ɕjɤ"), (MARK_I, "ᄉᆜ", "sɨ"), (MARK_I, "ᄇᆜ", "pɨ"), (MARK_I, "ᅙᆜ", "ʔɨ"), (0, "ᅙᆜ", "ʔɨ"), (MARK_I, "시", "ɕi")] {
        set_marks(bits);
        assert_eq!(read_pl(w).1, ipa, "{w} with switches {bits}");
    }
    set_marks(0);
}
