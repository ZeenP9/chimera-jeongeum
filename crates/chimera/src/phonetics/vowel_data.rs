//! Reference vowel formants (Hz) used to calibrate articulatory vowel targets.
//!
//! Fork (正音, 사용자 지정 2026-11-23): the female body is J. Beck's; the male and child bodies are
//! left as they were (upstream's values: Hillenbrand, Getty, Clark & Wheeler 1995 for the English
//! vowels, literature values for the rest; the child's derived from the male's), until the female
//! one is heard to work. So each row is (male: as before, female: Beck, child: as before).
//! The female values are Janet Beck's cardinal vowels on Seeing Speech (seeingspeech.ac.uk, IPA
//! chart 4, the ultrasound session's audio), one speaker, so the chart's order and spacing hold:
//! LPC orders 11-15 at 11025 Hz on the middle half of the held vowel, median over frames and
//! orders (her F0 208-216 Hz). ɪ ʊ have only MRI takes, scaled by the MRI→studio ratio of the
//! vowels with both (front F1 0.94 F2 0.91 F3 0.92, back F1 0.94 F2 0.97). Without a recording, the
//! lowered mids (e̞ ø̞ ɤ̞ o̞ ɔ̞ ɐ̞), æ ɐ ʏ ɘ ɵ ɜ ɞ are the midpoint of the two recorded vowels round
//! them (ä, open central, being a–ɑ's). Least sure: o's F3 (orders gave 2815-3830), a's F1 (850 or
//! 970), and the F1 of the close vowels, near her F0. Symbols the male table lacked (ɐ̞ ɔ̞) take
//! her values × the mean men/women ratio of Hillenbrand's vowels. Known and accepted for now: on
//! the male and child bodies a symbol can mean upstream's vowel (ʌ: American "hud"), and their o
//! is opener than their o̞. English keeps upstream's symbols ("cup" is ʌ), so on the female body
//! an English word sounds with Beck's vowel of that symbol (her ʌ is a back vowel).

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VowelRef {
    pub ipa: &'static str,
    /// example word / description
    pub example: &'static str,
    pub male: [f64; 3],
    pub female: [f64; 3],
    pub child: [f64; 3],
    pub round: bool,
    /// 0 close … 1 open
    pub height: f64,
    /// 0 front … 1 back
    pub back: f64,
}

const fn scale_ref(m: [f64; 3], k: [f64; 3]) -> [f64; 3] {
    [m[0] * k[0], m[1] * k[1], m[2] * k[2]]
}
const FEM: [f64; 3] = [1.17, 1.2, 1.15];
const CHI: [f64; 3] = [1.3, 1.33, 1.24];

#[allow(clippy::too_many_arguments)]
const fn vref(ipa: &'static str, example: &'static str, male: [f64; 3], female: Option<[f64; 3]>, child: Option<[f64; 3]>, round: bool, height: f64, back: f64) -> VowelRef {
    let female = match female {
        Some(f) => f,
        None => scale_ref(male, FEM),
    };
    let child = match child {
        Some(c) => c,
        None => scale_ref(male, CHI),
    };
    VowelRef { ipa, example, male, female, child, round, height, back }
}

#[rustfmt::skip]
pub static VOWELS: &[VowelRef] = &[
    vref("i", "cardinal 1: 300/2500 set by the user 2026-11-26 (J. Beck 235/2555, fitted 270/2497)", [342.0, 2322.0, 3000.0], Some([300.0, 2500.0, 3670.0]), Some([452.0, 3081.0, 3702.0]), false, 0.0, 0.0),
    vref("ɪ", "near-close front: F1 F2 set by the user 2026-11-25 (J. Beck's MRI take gave 425/2355, below e's 405; the tongue is held at that take's, so it sounds about 406/2364), F3 Beck", [427.0, 2034.0, 2684.0], Some([385.0, 2300.0, 2915.0]), Some([511.0, 2552.0, 3403.0]), false, 0.15, 0.1),
    vref("e", "cardinal 2: F1 F2 set by the user 2026-11-25 (J. Beck measured 405 / 2445), F3 Beck", [476.0, 2089.0, 2691.0], Some([430.0, 2400.0, 3425.0]), Some([564.0, 2656.0, 3323.0]), false, 0.33, 0.0),
    vref("ɛ", "cardinal 3, J. Beck", [580.0, 1799.0, 2605.0], Some([620.0, 2075.0, 2955.0]), Some([749.0, 2267.0, 3310.0]), false, 0.66, 0.0),
    vref("æ", "user set 710/1870 (2026-11-25; was the ɛ–a midpoint 795/1870, computed from J. Beck)", [640.0, 1760.0, 2500.0], Some([710.0, 1870.0, 3320.0]), Some([717.0, 2501.0, 3289.0]), false, 0.83, 0.0),
    vref("a", "cardinal 4, J. Beck", [780.0, 1450.0, 2550.0], Some([970.0, 1670.0, 3685.0]), None, false, 1.0, 0.0),
    vref("ɑ", "cardinal 5 (user set 750/1200, 2026-11-25; J. Beck 630/1000)", [768.0, 1333.0, 2522.0], Some([750.0, 1200.0, 2940.0]), Some([1002.0, 1688.0, 2950.0]), false, 1.0, 1.0),
    vref("ɒ", "cardinal 13: 755/1020 set by the user 2026-11-26 (700/940 before; 735/840 cannot be sounded: on this body F2 stays 210 Hz or more above F1; J. Beck 580/855)", [620.0, 880.0, 2500.0], Some([755.0, 1020.0, 2975.0]), None, true, 1.0, 1.0),
    vref("ɔ", "cardinal 6: 640/900 set by the user 2026-11-26 (600/840 the day before; J. Beck 490/770)", [652.0, 997.0, 2538.0], Some([640.0, 900.0, 2940.0]), Some([803.0, 1210.0, 2982.0]), true, 0.66, 1.0),
    vref("o", "cardinal 7: 450/920 set by the user 2026-11-26; before it 468/852, which is what the female body gives with the lips as open as J. Beck's (0.52 cm, 0.7 of her reading), set by the user 2026-11-25 (420/840 before, which needed them 0.13 cm apart; Beck 410/845)", [497.0, 910.0, 2459.0], Some([450.0, 920.0, 3030.0]), Some([597.0, 1137.0, 2987.0]), true, 0.33, 1.0),
    vref("ʊ", "near-close back (MRI take, ×MRI→studio), J. Beck", [469.0, 1122.0, 2434.0], Some([410.0, 1040.0, 2620.0]), Some([568.0, 1490.0, 3072.0]), true, 0.15, 0.9),
    vref("u", "cardinal 8, J. Beck", [378.0, 997.0, 2343.0], Some([300.0, 830.0, 2840.0]), Some([494.0, 1345.0, 2988.0]), true, 0.0, 1.0),
    vref("ʌ", "cardinal 14: 680/1180 set by the user 2026-11-26 (680/1100 earlier that day, 635/1090 the day before; J. Beck measured 595/1055, where 녕 was heard near 뇽), F3 Beck", [623.0, 1200.0, 2550.0], Some([680.0, 1180.0, 2695.0]), Some([749.0, 1546.0, 3145.0]), false, 0.66, 1.0),
    vref("ə", "schwa: F1 F2 set to ɝ's by the user 2026-11-25 (was 535/1400, user 2026-11-24; J. Beck measured 595 / 1630), F3 Beck", [500.0, 1450.0, 2450.0], Some([523.0, 1588.0, 2695.0]), None, false, 0.5, 0.5),
    vref("ɐ", "F1 F2 set by the user 2026-11-25 to what it sounded (was the ə–ä midpoint of targets, ä = a–ɑ, 690/1510)", [640.0, 1350.0, 2500.0], Some([690.0, 1541.0, 3005.0]), None, false, 0.83, 0.5),
    vref("ɝ", "heard", [474.0, 1379.0, 1710.0], Some([523.0, 1588.0, 1929.0]), Some([586.0, 1719.0, 2143.0]), true, 0.45, 0.5),
    vref("y", "cardinal 9: 290/2300 set by the user 2026-11-26, lips 0.45 cm apart and 1.5 cm wide at least (calibrate.rs, LIP_FLOORS; J. Beck 230/2290)", [300.0, 1800.0, 2150.0], Some([290.0, 2300.0, 2685.0]), None, true, 0.0, 0.05),
    vref("ø", "cardinal 10, J. Beck", [390.0, 1550.0, 2250.0], Some([400.0, 2260.0, 2770.0]), None, true, 0.33, 0.05),
    vref("œ", "cardinal 11, J. Beck", [520.0, 1450.0, 2350.0], Some([555.0, 1830.0, 2855.0]), None, true, 0.66, 0.05),
    vref("ɨ", "close central unrounded: F1 F2 set by the user 2026-11-24 (J. Beck measured 345 / 2110), F3 Beck", [330.0, 1600.0, 2400.0], Some([340.0, 1985.0, 2350.0]), None, false, 0.0, 0.5),
    vref("ɯ", "cardinal 16: 375/1250 set by the user 2026-11-26, (400/1200 the day before, 380 / 1160 before that; J. Beck measured 390 / 1460), F3 Beck", [340.0, 1350.0, 2400.0], Some([375.0, 1250.0, 2560.0]), None, false, 0.0, 1.0),
    vref("ɤ", "cardinal 15: 480/1260 set by the user 2026-11-26 (460/1120 the day before; J. Beck measured 435 / 1090), F3 Beck", [460.0, 1250.0, 2450.0], Some([480.0, 1260.0, 2505.0]), None, false, 0.33, 1.0),
    // Mandarin's apical vowels: the -i of 四 si (after z c s) and of 是 shi (after zh ch sh r), the
    // consonant's tongue held open; the retroflex one has a low F3 (approximate male means)
    vref("ɿ", "Mandarin si (apical)", [360.0, 1400.0, 2600.0], None, None, false, 0.0, 0.45),
    vref("ʅ", "Mandarin shi (apical, retroflex)", [400.0, 1500.0, 2000.0], None, None, false, 0.05, 0.55),
    // Mandarin's own vowels, measured on Beijing speakers (Zee & Lee 2001, doi 10.21437/Eurospeech.2001-169:
    // 10 men and 10 women, 5 repetitions, LPC at the vowel's midpoint; female values are theirs, the child's
    // are the male's scaled by the file's factor). The English vowels of the same name are not Mandarin's:
    // American /u/ "who'd" has F2 ~1000 Hz, Mandarin's 661. The key is the English symbol plus U+0320
    // (retracted), so these entries cannot collide with, or be reached by typing, the English ones.
    // u̠ achieves (F1 / F2 / F3 Hz, target in brackets): male 383 / 649 / 2482 (345 / 661 / 2783), female 366 / 776 / 2841
    // (345 / 759 / 3309), child 465 / 942 / 3344 (449 / 879 / 3451). F2, what makes it not English /u/, is within
    // 2-3 % for the adults; F1 is +11 % male and +6 % female because the tube cannot go lower: male tbX, jaw and
    // lip width are at their lower bounds (1.2, 0.05, 0) and lip protrusion at its upper (1.4). The female
    // fit starts from the male solution (calibrate.rs `start_from`; from the generic guess it settled at
    // jaw 1.8, F2 +11 %). The child's F2 is +7 %.
    vref("u̠", "Mandarin wu, hu (back u); female: the same as u, J. Beck's cardinal 8 (set by the user 2026-11-24)", [345.0, 661.0, 2783.0], Some([300.0, 830.0, 2840.0]), None, true, 0.0, 1.0),
    // ʊ̠ is the vowel of ong / iong and the off-glide of ao ou iao iou (the table's "u before ŋ": Zee & Lee's
    // men 441 / 730 / 2881, women 533 / 945 / 3314; English /ʊ/ "hood" has F2 1122 / 1225). It achieves
    // (F1 / F2 / F3 Hz): male 422 / 765 / 2345, female 513 / 972 / 2859, child 555 / 1004 / 3360: F1 and F2 within
    // 5 % on every body (F3, weighted least in the fit, -14 to -19 % for the adults).
    vref("ʊ̠", "Mandarin ong (back ʊ)", [441.0, 730.0, 2881.0], Some([533.0, 945.0, 3314.0]), None, true, 0.2, 1.0),
    // fork (正音): symbols upstream lacked, from J. Beck's cardinal vowels (see the module note)
    vref("ʉ", "close central rounded, J. Beck", [342.0, 1462.0, 1839.0], Some([400.0, 1745.0, 2135.0]), None, true, 0.0, 0.5),
    vref("ʏ", "F1 F2 set by the user 2026-11-25 (was the y–ø midpoint computed from J. Beck, 315/2275)", [360.0, 1633.0, 2217.0], Some([350.0, 2150.0, 2730.0]), None, true, 0.15, 0.1),
    vref("ɘ", "ɨ–ə midpoint (computed; the central four stand around ə, user 2026-11-24)", [468.0, 1670.0, 2570.0], Some([430.0, 1785.0, 2520.0]), None, false, 0.33, 0.5),
    vref("ɵ", "ʉ–ə midpoint (computed)", [444.0, 1230.0, 2355.0], Some([465.0, 1670.0, 2415.0]), None, true, 0.33, 0.5),
    vref("ɜ", "ə–ɐ midpoint (computed; of 523/1588 and 690/1541 since 2026-11-25)", [602.0, 1500.0, 2578.0], Some([606.5, 1564.5, 2850.0]), None, false, 0.66, 0.5),
    vref("ɞ", "a quarter of the way from ə to the ɶ–ɒ midpoint, as ɜ is to ä (computed; ɶ 791/1476, ɒ 755/1020 since 2026-11-26)", [586.0, 1224.0, 2444.0], Some([585.5, 1503.0, 2740.0]), None, true, 0.66, 0.5),
    vref("ɶ", "cardinal 12: F1 F2 set by the user 2026-11-26 to where ɐ̞ sounds (791/1476), told from it by its lips (calibrate.rs, the ɶ branch of the cost); was 780/1450", [585.0, 1288.0, 2404.0], Some([791.0, 1476.0, 2755.0]), None, true, 1.0, 0.05),
    vref("ɤ̞", "ɤ–ʌ midpoint (computed; of 480/1260 and 680/1180 since 2026-11-26)", [440.0, 888.0, 2277.0], Some([580.0, 1220.0, 2600.0]), None, false, 0.5, 1.0),
    vref("e̞", "e–ɛ midpoint (computed; of 430/2400 and 620/2075 since 2026-11-25)", [421.0, 1918.0, 2715.0], Some([525.0, 2237.5, 3190.0]), None, false, 0.5, 0.0),
    vref("ø̞", "ø–œ midpoint (computed from J. Beck)", [402.0, 1702.0, 2439.0], Some([480.0, 2045.0, 2810.0]), None, true, 0.5, 0.05),
    vref("o̞", "midpoint of o and ɔ as they are heard, their fitted formants (user 2026-11-25; o 469/939, ɔ 633/911 since 2026-11-26), not of their targets", [391.0, 668.0, 2422.0], Some([551.0, 925.0, 2995.0]), None, true, 0.5, 1.0),
    vref("ɔ̞", "between ɔ and ɒ: 680/910 set by the user 2026-11-26 (was the midpoint of the two as they are heard, 686/970 for ɔ 633/911 and ɒ 739/1029)", [455.0, 705.0, 2620.0], Some([680.0, 910.0, 2960.0]), None, true, 0.83, 1.0),
    vref("ɐ̞", "midpoint of a and ɑ as they are heard, their fitted formants (user 2026-11-25; a 835/1744, ɑ 748/1207). Was the ɐ–ä midpoint of targets, 775/1475, which a's unreached F1 put 70 Hz below the line from a to ɑ", [635.0, 1205.0, 2785.0], Some([791.0, 1476.0, 3160.0]), None, false, 0.92, 0.5),
];
