//! Reference vowel formants (Hz) used to calibrate articulatory vowel targets.
//! English vowels: Hillenbrand, Getty, Clark & Wheeler (1995) means (men / women / children).
//! Other cardinal-ish vowels: typical literature values (men), scaled for women/children.

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
    vref("i", "heed", [342.0, 2322.0, 3000.0], Some([437.0, 2761.0, 3372.0]), Some([452.0, 3081.0, 3702.0]), false, 0.0, 0.0),
    vref("ɪ", "hid", [427.0, 2034.0, 2684.0], Some([483.0, 2365.0, 3053.0]), Some([511.0, 2552.0, 3403.0]), false, 0.2, 0.15),
    vref("e", "hayed", [476.0, 2089.0, 2691.0], Some([536.0, 2530.0, 3047.0]), Some([564.0, 2656.0, 3323.0]), false, 0.3, 0.05),
    vref("ɛ", "head", [580.0, 1799.0, 2605.0], Some([731.0, 2058.0, 2979.0]), Some([749.0, 2267.0, 3310.0]), false, 0.55, 0.1),
    vref("æ", "had", [640.0, 1760.0, 2500.0], Some([669.0, 2349.0, 2972.0]), Some([717.0, 2501.0, 3289.0]), false, 0.8, 0.15),
    vref("a", "front open (Fr. patte)", [780.0, 1450.0, 2550.0], None, None, false, 1.0, 0.35),
    vref("ɑ", "hod", [768.0, 1333.0, 2522.0], Some([936.0, 1551.0, 2815.0]), Some([1002.0, 1688.0, 2950.0]), false, 1.0, 0.85),
    vref("ɒ", "Br. lot", [620.0, 880.0, 2500.0], None, None, true, 0.9, 1.0),
    vref("ɔ", "hawed", [652.0, 997.0, 2538.0], Some([781.0, 1136.0, 2824.0]), Some([803.0, 1210.0, 2982.0]), true, 0.65, 1.0),
    vref("o", "hoed", [497.0, 910.0, 2459.0], Some([555.0, 1035.0, 2828.0]), Some([597.0, 1137.0, 2987.0]), true, 0.35, 1.0),
    vref("ʊ", "hood", [469.0, 1122.0, 2434.0], Some([519.0, 1225.0, 2827.0]), Some([568.0, 1490.0, 3072.0]), true, 0.2, 0.85),
    vref("u", "who'd", [378.0, 997.0, 2343.0], Some([459.0, 1105.0, 2735.0]), Some([494.0, 1345.0, 2988.0]), true, 0.0, 1.0),
    vref("ʌ", "hud", [623.0, 1200.0, 2550.0], Some([753.0, 1426.0, 2933.0]), Some([749.0, 1546.0, 3145.0]), false, 0.6, 0.7),
    vref("ə", "sofa", [500.0, 1450.0, 2450.0], None, None, false, 0.5, 0.5),
    vref("ɐ", "near-open central", [640.0, 1350.0, 2500.0], None, None, false, 0.8, 0.5),
    vref("ɝ", "heard", [474.0, 1379.0, 1710.0], Some([523.0, 1588.0, 1929.0]), Some([586.0, 1719.0, 2143.0]), true, 0.45, 0.5),
    vref("y", "Fr. tu", [300.0, 1800.0, 2150.0], None, None, true, 0.0, 0.0),
    vref("ø", "Fr. peu", [390.0, 1550.0, 2250.0], None, None, true, 0.3, 0.1),
    vref("œ", "Fr. peur", [520.0, 1450.0, 2350.0], None, None, true, 0.55, 0.15),
    vref("ɨ", "close central", [330.0, 1600.0, 2400.0], None, None, false, 0.0, 0.5),
    vref("ɯ", "close back unrounded", [340.0, 1350.0, 2400.0], None, None, false, 0.0, 1.0),
    vref("ɤ", "close-mid back unrounded", [460.0, 1250.0, 2450.0], None, None, false, 0.35, 1.0),
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
    vref("u̠", "Mandarin wu, hu (back u)", [345.0, 661.0, 2783.0], Some([345.0, 759.0, 3309.0]), None, true, 0.0, 1.0),
    // ʊ̠ is the vowel of ong / iong and the off-glide of ao ou iao iou (the table's "u before ŋ": Zee & Lee's
    // men 441 / 730 / 2881, women 533 / 945 / 3314; English /ʊ/ "hood" has F2 1122 / 1225). It achieves
    // (F1 / F2 / F3 Hz): male 422 / 765 / 2345, female 513 / 972 / 2859, child 555 / 1004 / 3360: F1 and F2 within
    // 5 % on every body (F3, weighted least in the fit, -14 to -19 % for the adults).
    vref("ʊ̠", "Mandarin ong (back ʊ)", [441.0, 730.0, 2881.0], Some([533.0, 945.0, 3314.0]), None, true, 0.2, 1.0),
    // 正音's vowels (fork): one speaker, one session, so the chart's order holds. Janet Beck's cardinal
    // vowels on Seeing Speech (seeingspeech.ac.uk, IPA chart 4, the ultrasound session's audio),
    // measured 2026-11-23: female voice, F0 208-216 Hz; LPC orders 11-15 at 11025 Hz on the middle
    // half of the held vowel, median over frames and orders. Female values as measured; male = female
    // × (0.855, 0.833, 0.87), the ratio the earlier rows here used; child by the table's scaling.
    // ɪ ʊ have no ultrasound take: their MRI takes, scaled by the MRI→studio ratio of the cardinal
    // vowels that have both (front F1 0.94 F2 0.91 F3 0.92, back F1 0.94 F2 0.97). o's F3 is the
    // least sure (orders disagree, 2815-3830). F1 is near F0 for the close vowels.
    // Keys: a 正音 vowel whose plain IPA key holds English (Hillenbrand) data takes U+0359 (◌͙),
    // or the earlier U+0320 (ɯ̠ ɤ̠ ʌ̠ ɑ̠); `jeongeum::beck` maps typed 正音 onto these. English is
    // untouched. Checked by tests/jeongeum.rs: F1 rises down each column of the chart.
    vref("i͙", "cardinal 1 (B_cardinal_1)", [200.0, 2130.0, 3195.0], Some([235.0, 2555.0, 3670.0]), None, false, 0.0, 0.0),
    vref("y͙", "cardinal 9 (B_cardinal_9)", [195.0, 1910.0, 2335.0], Some([230.0, 2290.0, 2685.0]), None, true, 0.0, 0.05),
    vref("ɨ͙", "close central unrounded (B_Central_close_unround)", [295.0, 1760.0, 2045.0], Some([345.0, 2110.0, 2350.0]), None, false, 0.0, 0.5),
    vref("u͙", "cardinal 8 (B_cardinal_8)", [255.0, 690.0, 2470.0], Some([300.0, 830.0, 2840.0]), None, true, 0.0, 1.0),
    vref("ɪ͙", "near-close front (MRI_front_close_float_vowel, ×MRI→studio)", [365.0, 1960.0, 2535.0], Some([425.0, 2355.0, 2915.0]), None, false, 0.15, 0.1),
    vref("ʊ͙", "near-close back (MRI_back_close_float_vowel, ×MRI→studio)", [350.0, 865.0, 2280.0], Some([410.0, 1040.0, 2620.0]), None, true, 0.15, 0.9),
    vref("e͙", "cardinal 2 (B_cardinal_2)", [345.0, 2035.0, 2980.0], Some([405.0, 2445.0, 3425.0]), None, false, 0.33, 0.0),
    vref("ø͙", "cardinal 10 (B_cardinal_10)", [340.0, 1885.0, 2410.0], Some([400.0, 2260.0, 2770.0]), None, true, 0.33, 0.05),
    vref("o͙", "cardinal 7 (B_cardinal_7)", [350.0, 705.0, 2635.0], Some([410.0, 845.0, 3030.0]), None, true, 0.33, 1.0),
    vref("ə͙", "schwa (B_schwa)", [510.0, 1360.0, 2345.0], Some([595.0, 1630.0, 2695.0]), None, false, 0.5, 0.5),
    vref("ɛ͙", "cardinal 3 (B_cardinal_3)", [530.0, 1730.0, 2570.0], Some([620.0, 2075.0, 2955.0]), None, false, 0.66, 0.0),
    vref("œ͙", "cardinal 11 (B_cardinal_11)", [475.0, 1525.0, 2485.0], Some([555.0, 1830.0, 2855.0]), None, true, 0.66, 0.05),
    vref("ɔ͙", "cardinal 6 (B_cardinal_6)", [420.0, 640.0, 2560.0], Some([490.0, 770.0, 2940.0]), None, true, 0.66, 1.0),
    vref("a͙", "cardinal 4 (B_cardinal_4)", [830.0, 1390.0, 3205.0], Some([970.0, 1670.0, 3685.0]), None, false, 1.0, 0.0),
    vref("ɒ͙", "cardinal 13 (B_cardinal_13)", [495.0, 710.0, 2590.0], Some([580.0, 855.0, 2975.0]), None, true, 1.0, 1.0),
    // computed, no recording: the midpoint of the two recorded vowels around it on the chart
    vref("æ͙", "ɛ–a midpoint (computed from Beck)", [680.0, 1560.0, 2890.0], Some([795.0, 1870.0, 3320.0]), None, false, 0.83, 0.0),
    vref("ɐ͙", "ə–(a+ɑ)/2 midpoint (computed from Beck)", [600.0, 1235.0, 2615.0], Some([700.0, 1480.0, 3005.0]), None, false, 0.83, 0.5),
    // Still fitted to their earlier targets (the calibrated table keeps these entries; refitting
    // them needs their old entries dropped from vowelTargets.generated.json first). Their new values,
    // from the same Beck measurement as the rows above:
    //   ʉ: male [340.0, 1455.0, 1855.0], female [400.0, 1745.0, 2135.0]
    //   ɯ̠: male [335.0, 1215.0, 2225.0], female [390.0, 1460.0, 2560.0]
    //   ɤ̠: male [370.0, 910.0, 2180.0], female [435.0, 1090.0, 2505.0]
    //   ʌ̠: male [510.0, 880.0, 2345.0], female [595.0, 1055.0, 2695.0]
    //   ɶ: male [545.0, 1285.0, 2395.0], female [635.0, 1545.0, 2755.0]
    //   ɑ̠: male [540.0, 835.0, 2560.0], female [630.0, 1000.0, 2940.0]
    //   e̞: male [435.0, 1885.0, 2775.0], female [510.0, 2260.0, 3190.0]
    //   ø̞: male [410.0, 1705.0, 2445.0], female [480.0, 2045.0, 2810.0]
    //   ɤ̞: male [440.0, 890.0, 2260.0], female [515.0, 1070.0, 2600.0]
    //   o̞: male [385.0, 675.0, 2595.0], female [450.0, 810.0, 2985.0]
    //   ʏ: male [270.0, 1895.0, 2375.0], female [315.0, 2275.0, 2730.0]
    //   ɘ: male [360.0, 1475.0, 2580.0], female [420.0, 1770.0, 2965.0]
    //   ɵ: male [345.0, 1290.0, 2525.0], female [405.0, 1550.0, 2900.0]
    //   ɜ: male [520.0, 1305.0, 2460.0], female [610.0, 1565.0, 2825.0]
    //   ɞ: male [445.0, 1085.0, 2525.0], female [520.0, 1300.0, 2900.0]
    vref("ʉ", "close central rounded (Seeing Speech, J. Beck)", [342.0, 1462.0, 1839.0], Some([400.0, 1755.0, 2115.0]), None, true, 0.0, 0.5),
    vref("ʏ", "near-close near-front rounded (y–ø 보간)", [360.0, 1633.0, 2217.0], None, None, true, 0.2, 0.15),
    vref("ɘ", "close-mid central unrounded (e–ɤ 보간)", [468.0, 1670.0, 2570.0], None, None, false, 0.33, 0.5),
    vref("ɵ", "close-mid central rounded (ø–o 보간)", [444.0, 1230.0, 2355.0], None, None, true, 0.33, 0.55),
    vref("ɜ", "open-mid central unrounded (ɛ–ʌ 보간)", [602.0, 1500.0, 2578.0], None, None, false, 0.58, 0.45),
    vref("ɞ", "open-mid central rounded (œ–ɔ 보간)", [586.0, 1224.0, 2444.0], None, None, true, 0.6, 0.55),
    vref("ɶ", "open front rounded (Seeing Speech, J. Beck)", [585.0, 1288.0, 2404.0], Some([685.0, 1545.0, 2765.0]), None, true, 1.0, 0.2),
    vref("ɤ̠", "cardinal 15 close-mid back unrounded (B_cardinal_15)", [372.0, 896.0, 2187.0], Some([435.0, 1075.0, 2515.0]), None, false, 0.35, 1.0),
    vref("ʌ̠", "cardinal 14 open-mid back unrounded (B_cardinal_14)", [509.0, 879.0, 2365.0], Some([595.0, 1055.0, 2720.0]), None, false, 0.6, 1.0),
    vref("ɤ̞", "mid back unrounded (cardinal 15–14 midpoint)", [440.0, 888.0, 2277.0], Some([515.0, 1065.0, 2618.0]), None, false, 0.48, 1.0),
    vref("ɯ̠", "cardinal 16 close back unrounded (B_cardinal_16)", [325.0, 1217.0, 2226.0], Some([380.0, 1460.0, 2560.0]), None, false, 0.0, 1.0),
    vref("e̞", "mid front unrounded (cardinal 2–3 midpoint)", [421.0, 1918.0, 2715.0], Some([492.0, 2302.0, 3122.0]), None, false, 0.48, 0.0),
    vref("ø̞", "mid front rounded (cardinal 10–11 midpoint)", [402.0, 1702.0, 2439.0], Some([470.0, 2042.0, 2805.0]), None, true, 0.48, 0.1),
    vref("o̞", "mid back rounded (cardinal 7–6 midpoint)", [391.0, 668.0, 2422.0], Some([458.0, 802.0, 2785.0]), None, true, 0.48, 1.0),
    vref("ɑ̠", "cardinal 5 open back unrounded (B_cardinal_5)", [568.0, 829.0, 2565.0], Some([665.0, 995.0, 2950.0]), None, false, 1.0, 1.0),
];
