//! `chimera calibrate …`: ports of `tools/calibrate-vowels.ts` and `tools/calibrate-consonants.ts`.
//! They regenerate the tables the engine embeds (`crates/chimera/src/phonetics/*.generated.json`),
//! written as `JSON.stringify(table, null, 1)` wrote them.

use crate::js::{Json, fixed, pad_end, pad_start};
use crate::repo_path;
use chimera::airway::frequency::{find_peaks, transfer_function};
use chimera::anatomy::human::{
    ArticKey, BODY_CHILD, BODY_FEMALE, BODY_MALE, HumanArtic, HumanBody, NEUTRAL, rest_tip,
};
use chimera::math::{js_max, js_min, js_round, ln, pow};
use chimera::phonetics::inventory::consonants as inventory;
use chimera::phonetics::planner::VowelTable;
use chimera::phonetics::vowel_data::VOWELS;
use chimera::util::optimize::{NelderMeadOpts, nelder_mead};
use chimera::voice::human_airway::HumanAirway;

const VOWEL_TABLE: &str = "crates/chimera/src/phonetics/vowelTargets.generated.json";
const CONSONANT_TABLE: &str = "crates/chimera/src/phonetics/consonantTargets.generated.json";
const BODIES: [(&str, HumanBody); 3] = [
    ("male", BODY_MALE),
    ("female", BODY_FEMALE),
    ("child", BODY_CHILD),
];

/// Split `--out <file>` from the positional arguments.
fn out_option(a: &[String], default: &str) -> Result<(Vec<String>, String), String> {
    let mut rest = Vec::new();
    let mut out = repo_path(default);
    let mut it = a.iter();
    while let Some(x) = it.next() {
        if x == "--out" {
            out = it.next().ok_or("--out needs a file")?.clone();
        } else {
            rest.push(x.clone());
        }
    }
    Ok((rest, out))
}

fn round3(x: f64) -> f64 {
    js_round(x * 1000.0) / 1000.0
}

/// tbX tbY jaw lipAp lipProt lipWidth root larynx curl
// fork: the tongue body may sit as low and as far back as J. Beck's does (ɑ ɒ), within the
// articulators' own range (ARTIC_SPECS)
const LO: [f64; 9] = [1.0, 4.88, 0.05, 0.12, 0.0, 0.0, -0.9, -1.3, 0.0];
const HI: [f64; 9] = [4.6, 8.8, 2.3, 2.0, 1.4, 1.0, 0.9, 1.3, 1.0];

/// fork (正音, 사용자 지정 2026-11-23): where J. Beck's tongue body sits for each vowel, in this
/// model's template frame (cm), from her midsagittal MRI on Seeing Speech (seeingspeech.ac.uk, IPA
/// chart 4, "MRI2" takes, 320 px): the postures the vowels are fitted round, in place of upstream's.
/// Per take: the tongue's upper contour (first air pixel above the tongue, x 192-256 px) in 25
/// frames; a circle of the female body's tongue radius (1.98 cm = 26 px) fitted to it; the frames
/// kept whose dorsum peak sits in the most populated 4-px band (the held vowel); the median centre.
/// The head did not move between takes (a skull patch matched within 1 px, ʌ 3 px, corrected).
/// Registration: 13 px/cm (her head's length taken as 18 cm), no rotation, the female body's hard
/// palate laid on hers (model origin at px 195.6, 285), then `un_p` for the female body; her
/// pharyngeal wall then lies within 2 px of the model's. Last, one offset (−0.72, +0.70), which
/// puts her i e ɛ ɪ y ø where the unconstrained fit of her formants had put them: a pixel's worth
/// of registration is 0.09 here, and the model's palate dome is ~0.4 cm taller than hers.
/// Check: with the tongue held at these (weight 2), the female body gives her formants within
/// 7 % for e ɛ ø œ ɤ ʌ ɑ ɔ ɒ ɨ; not the corners (u F1 +36 %, a F1 −18 %, o F2 +21 %, ɯ, i), where
/// the lips or jaw are at their limits. The lowered mids and the vowels without a take sit midway
/// between the two around them, as their formants do (vowel_data.rs). MRI is taken lying down;
/// the offset above absorbs what gravity does to the front vowels, the back ones are as measured.
/// the prior's weight on the tongue body's distance (cm²) from Beck's posture: 0.15 keeps the
/// female fit within 0.1-0.35 cm of it with the formants as close as without (0.03 let the back
/// vowels drift 0.5 cm up; 2 cost o 21 % of F2)
const W_BECK: f64 = 0.15;
/// the prior's weight once the vowel cannot be sounded from Beck's posture
const W_BECK_SOFT: f64 = 0.03;
// ɤ̞ is not here: its entry was the midpoint of ɤ's and ʌ's postures, not a take of hers, and held
// there it gave F1 560 for a target of 515 (root and larynx at their limits); free, 519.
// ɑ ɔ ɒ ɔ̞ o̞ ɶ are not here either: the user set them 110-155 Hz opener than her recordings read (2026-11-25).
// ɨ ə ɯ (and ɐ ɐ̞ ɘ ɵ ɜ ɞ, computed from them) are not here: the user set their formants away from her
// recordings (2026-11-24), so her MRI positions no longer describe them.
/// fork: how far J. Beck's lips part at the midline in her unrounded vowels, as `lipAp` (cm / 0.9).
/// Read by eye from the Seeing Speech MRI2 takes on a 5 px grid, 13 px per cm, +-2 px (0.15 cm):
/// upper lip's lower edge to lower lip's upper edge in a held frame. ɛ æ a ʌ open past the
/// model's range (2.0) and sit at or near it. Lip width cannot be seen in a midsagittal scan.
const BECK_LIP_AP: &[(&str, f64)] = &[
    ("i", 1.55), ("ɪ", 1.8), ("e", 1.8), ("ɛ", 2.0), ("æ", 2.0), ("a", 2.0),
    ("ɨ", 1.6), ("ə", 1.45), ("ɯ", 0.75), ("ɤ", 1.3), ("ɤ̞", 1.6), ("ʌ", 1.9), ("ɑ", 1.55),
];
/// fork: the least her lips part in the rounded vowels that reach their formants as well with them
/// open (user, 2026-11-25): 0.7 of the MRI2 reading (ɔ 1.5 cm, ɒ 1.4 cm), as `lipAp`. Refitted
/// with this floor ɔ stays 593/858 (lips 0.37 -> 1.05 cm high, 2.3 -> 1.0 cm wide, protrusion
/// 0.34 -> 0.55) and ɒ 677/957. o and u cannot: their protrusion, width and jaw are at their
/// limits already, and opening the lips to 0.7 of hers raises F1 (o 433 -> 468, u 357 -> 388).
/// o opens all the same (0.75 cm x 0.7 = 0.52 cm) and its target moved to what that gives,
/// 468/852 (user). u keeps its target of 300/830, which it cannot reach, and opens to 0.27 cm at
/// least like y ʏ ʉ ʊ (user, 2026-11-25; 0.38 cm before, 377/904). It needs a start of its own
/// (STARTS): from the generic guess the lower floor fell into another basin (jaw 0.05), 383/928.
/// o̞ and ɔ̞ are midpoints of these vowels and take the midpoints of their floors (not readings):
/// left free, o̞ fitted with its lips 0.12 cm apart, less than o's.
/// u̠ is u on this body (user, 2026-11-24) and opens with it. y ʏ ʉ ʊ open to 0.27 cm at least
/// (user, 2026-11-25; they had fitted 0.11 to 0.13 cm apart, where hers read 0.4 to 0.8 cm) and
/// keep their formants within 1 %. œ opens to 0.65 cm (user: 0.6 to 0.75): it had fitted at
/// 0.40 cm, less than the closer ø's 0.48 (her œ has no clean frame to read).
const BECK_LIP_AP_ROUND: &[(&str, f64)] = &[("y", 0.3), ("ʏ", 0.3), ("ʉ", 0.3), ("ʊ", 0.3), ("œ", 0.72), ("u", 0.3), ("u̠", 0.3), ("o", 0.58), ("o̞", 0.87), ("ɔ", 1.17), ("ɔ̞", 1.13), ("ɒ", 1.09)];
/// the prior's weight on the lips' distance from it: the tongue's (W_BECK)
const W_BECK_LIP: f64 = 0.15;
/// the share of her opening the model takes (user, 2026-11-25): she lay down and held each cardinal
/// vowel, both of which open the lips further than speech does. At the full reading the female
/// labial stops closed 13.7 ms late (stop_spans, limit 7): the lips had 1 cm further to travel.
/// a ɛ æ take the full reading (user): 0.7 of it shut them below where they were (a: F1 835 -> 813).
const BECK_LIP_SCALE: f64 = 0.7;
/// ʌ keeps nearer its formants instead (user, 2026-11-25): from her tongue posture every bit of
/// lip opening raises F1 (0.5: 603 Hz, 0.7: 626, 0.9: 648, her 1.9 x 0.7: 683, for a target of
/// 595). 0.7 is the least that keeps the nasals on time: at 0.5 its nearly shut lips closed /m/
/// 9-10 ms early and nasal_spans failed (female median -7.4 ms, limit 7)
const LIP_AP_WEDGE: f64 = 0.7;
/// ɤ opens less too (user, 2026-11-25): at 0.7 of her reading it came out 9 % high in F1 and F2
const LIP_AP_RAMS_HORN: f64 = 0.5;
/// ɤ̞ has no take of hers (its entry in BECK_LIP_AP is the ɤ–ʌ midpoint, unused): its lips part
/// 0.45 cm (user, 2026-11-25), between ɤ's and ʌ's. On the unmeasured vowels' floor it had the
/// lips of ə (0.80 x 2.6 cm)
const LIP_AP_RAMS_HORN_LOWERED: f64 = 0.5;
const BECK_TONGUE: &[(&str, f64, f64)] = &[
    ("i", 4.01, 7.96),
    ("e", 3.71, 7.71),
    ("ɛ", 3.61, 6.45),
    ("a", 2.62, 5.57),
    ("o", 1.92, 6.54),
    ("u", 1.62, 7.03),
    ("u̠", 1.62, 7.03),
    ("y", 4.31, 8.05),
    ("ø", 3.71, 7.61),
    ("œ", 3.11, 6.45),
    // not ʌ (user, 2026-11-25): her take puts the tongue lower than any other vowel's (5.28, the
    // floor is 4.88). Held there it fitted 626/1074 for 595/1055 and resonances swept 1-5 kHz on
    // the way in and out of it (안녕: a ping at each edge of 녕). Free, it fits 602/1049 with the
    // tongue at (1.13, 6.60), and the sweep from 4.5 kHz is gone
    ("ɤ", 2.02, 6.84),
    ("ʉ", 3.71, 7.88),
    // ɪ stays held (user, 2026-11-25), short of its target (406 Hz for 385): free, it reached
    // 376/2312 with the tongue 0.46 cm further back and 0.12 higher, near enough the velum that /ŋ/
    // after it closed early and nasal_spans failed (female median -7.65 ms, limit 7); held midway,
    // still -7.40. Raising its F1 target moved only the root. e moved instead, down and forward
    // (430/2400), so that ɪ sits above it and further back.
    ("ɪ", 3.41, 7.42),
    ("ʊ", 2.72, 7.22),
    ("e̞", 3.66, 7.08),
    ("ø̞", 3.41, 7.03),
    ("æ", 3.11, 6.01),
    ("ʏ", 4.01, 7.83),
];

/// What an apical vowel (ɿ after z c s, ʅ after zh ch sh r) fixes by construction instead of
/// leaving to the formant fit: the lips (spread and open as for [i], never narrowed or protruded, which
/// is how the fit once hit the formants without the tongue tip) and the tongue tip, held at the
/// place of the consonant before it, opened just enough to be a vowel's constriction. The tongue
/// body, jaw, root and larynx are fitted around that.
struct Apical {
    lip_ap: f64,
    lip_prot: f64,
    lip_width: f64,
    tt_x: f64,
    tt_y: f64,
    curl: f64,
}

/// How far below its sibilant's calibrated constriction an apical vowel's tongue tip sits (the same
/// coordinate as `calibrate consonants` sets, TtY). That constriction is ~0.15 cm² (the fricative's
/// hiss); 0.11 lower it is ~0.3 cm², a vowel's (the fit's own floor is 0.22).
const APICAL_OPENING: f64 = 0.11;

/// The apical vowel `ipa`'s tip, curl and lips for a body: the tip is the place of the sibilant
/// before it (s for ɿ, ʂ for ʅ), from the inventory with its calibrated TtY, and the lips are
/// those of the body's [i] (its already-calibrated entry).
fn apical(ipa: &str, bname: &str, i_lips: Option<[f64; 3]>) -> Option<Apical> {
    let consonant = match ipa {
        "ɿ" => "s",
        "ʅ" => "ʂ",
        _ => return None,
    };
    let c = chimera::phonetics::inventory::phoneme(consonant, bname.into())?.targets;
    let [lip_ap, lip_prot, lip_width] = i_lips?;
    Some(Apical { lip_ap, lip_prot, lip_width, tt_x: c.get(ArticKey::TtX)?, tt_y: c.get(ArticKey::TtY)? - APICAL_OPENING, curl: c.get(ArticKey::TtCurl).unwrap_or(0.0) })
}

/// Where a vowel's Nelder-Mead search starts, when the generic height/back guess is not enough:
/// the (body, vowel) entry whose solved posture is the start. Only the female [u̠] has one, the
/// male's. [u̠] is a corner of the tube (tbX, jaw, lip width at their lower bounds, protrusion at
/// its upper), and from the generic guess the female fit settled in another basin (jaw 1.8,
/// F2 +11 %), as did every start from the English [u] (jaw 1.6, F2 +10 % for male and female).
/// Deterministic: the male body is fitted before the female one, in a full run and, with
/// `--missing`, from the file. The start also centres the cost's priors, as the generic guess does.
fn start_from(ipa: &str, body: &str) -> Option<(&'static str, &'static str)> {
    match (ipa, body) {
        ("u̠", "female") => Some(("male", "u̠")),
        // fork: o̞ is the midpoint of o and ɔ and starts from ɔ, fitted before it. From the generic
        // guess it settled in another basin (tongue 0.9 cm higher, larynx raised 0.94), F2 6 % high
        ("o̞", "female") => Some(("female", "ɔ")),
        _ => None,
    }
}

/// fork: starts given as postures (the 9 coordinates in `VOWEL_KEYS` order), for a vowel whose
/// good basin no fitted entry reaches. Female u with its lips 0.27 cm apart at least: from the
/// posture it had at 0.38 cm (4a3e784) it fits 351/924; from the generic guess 383/928, from the
/// male u̠ the same, from the female ʊ 347/973 (all for a target of 300/830).
/// u̠ is u on this body (user, 2026-11-24) and starts where u does, overriding its start_from.
const STARTS: &[(&str, &str, [f64; 9])] = &[
    ("u", "female", [2.223, 7.428, 1.642, 0.41, 1.4, 0.0, 0.9, -1.231, 0.0]),
    ("u̠", "female", [2.223, 7.428, 1.642, 0.41, 1.4, 0.0, 0.9, -1.231, 0.0]),
];

/// The 9 fitted coordinates (`VOWEL_KEYS` order) of the entry `ipa` in a body's table.
fn solved_posture(table: &[(String, Json)], ipa: &str) -> Option<Vec<f64>> {
    let Json::Obj(entry) = &table.iter().find(|(k, _)| k == ipa)?.1 else { return None };
    let Json::Obj(artic) = &entry.iter().find(|(k, _)| k == "artic")?.1 else { return None };
    ["tbX", "tbY", "jaw", "lipAp", "lipProt", "lipWidth", "root", "larynx", "ttCurl"]
        .iter()
        .map(|key| match artic.iter().find(|(k, _)| k == key)?.1 {
            Json::Num(x) => Some(x),
            _ => None,
        })
        .collect()
}

fn to_artic(p: &[f64], fixed: Option<&Apical>) -> HumanArtic {
    let (tb_x, tb_y, jaw, mut lip_ap, mut lip_prot, mut lip_width, root, larynx, mut curl) =
        (p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8]);
    if let Some(f) = fixed {
        (lip_ap, lip_prot, lip_width, curl) = (f.lip_ap, f.lip_prot, f.lip_width, f.curl);
    }
    let tip = rest_tip(jaw, tb_x, tb_y);
    // rhotic curl raises the tip toward the post-alveolar region
    let (mut tt_x, mut tt_y) = (tip.x - 0.4 * curl, tip.y + 2.0 * curl);
    if let Some(f) = fixed {
        (tt_x, tt_y) = (f.tt_x, f.tt_y);
    }
    HumanArtic {
        tb_x,
        tb_y,
        jaw,
        lip_ap,
        lip_prot,
        lip_width,
        root,
        larynx,
        tt_x,
        tt_y,
        tt_curl: curl,
        ..NEUTRAL
    }
}

/// Resonance peaks of the airway in posture `a` (frequency-domain model), and its narrowest area.
fn formants(air: &mut HumanAirway, freqs: &[f64], a: &HumanArtic) -> (Vec<f64>, f64) {
    air.apply(a, 0.0);
    let el = air.net.linear_elements();
    let h = transfer_function(&el, air.pharynx.first_node(), freqs, &[air.glottis_port]);
    let pk: Vec<f64> = find_peaks(freqs, &h.mag, 150.0)
        .iter()
        .map(|p| p.freq)
        .collect();
    let mut min_area = f64::INFINITY;
    for &x in air.ph_areas.iter().chain(air.or_areas.iter()) {
        min_area = js_min(min_area, x);
    }
    (pk, min_area)
}

/// Articulatory vowel targets fitted to reference formants with the frequency-domain airway
/// model. `calibrate vowels [male|female|child|all] [--missing] [--out file]`
///
/// `--missing` fits only the vowels of `VOWELS` that the table lacks, and keeps every entry it
/// already has byte for byte (the fit isn't bit-reproducible, so a full rerun moves old entries
/// by ±0.001–0.002). New entries are appended after the old ones.
pub fn vowels(a: &[String]) -> Result<(), String> {
    let (mut rest, out) = out_option(a, VOWEL_TABLE)?;
    let missing_only = match rest.iter().position(|x| x == "--missing") {
        Some(i) => {
            rest.remove(i);
            true
        }
        None => false,
    };
    let which = rest.first().map(String::as_str).unwrap_or("all");
    let n = 700;
    let freqs: Vec<f64> = (0..n)
        .map(|i| 120.0 * pow(4600.0 / 120.0, i as f64 / (n - 1) as f64))
        .collect();
    let mut results = match std::fs::read_to_string(&out) {
        Ok(text) => Json::parse(&text).map_err(|e| format!("{out}: {e}"))?,
        Err(_) => Json::Obj(Vec::new()),
    };

    for (bname, body) in BODIES {
        if which != "all" && which != bname {
            continue;
        }
        let mut air = HumanAirway::new(96000.0, body);
        let mut air_check = HumanAirway::new(96000.0, body);
        let k = body.scale;
        // `--missing`: start from the entries already in the file and skip those vowels
        let mut table: Vec<(String, Json)> = match (&results, missing_only) {
            (Json::Obj(m), true) => match m.iter().find(|(k, _)| k == bname) {
                Some((_, Json::Obj(old))) => old.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        // [i]'s lips (aperture, protrusion, width) once calibrated, for the apical vowels
        let mut i_lips: Option<[f64; 3]> = None;
        for vw in VOWELS {
            if missing_only && table.iter().any(|(k, _)| k == vw.ipa) {
                // an [i] kept from the file still gives the apical vowels its lips: without them
                // a missing ɿ or ʅ was fitted unpinned, with no word of it
                if vw.ipa == "i" {
                    i_lips = solved_posture(&table, "i").map(|p| [p[3], p[4], p[5]]);
                }
                continue;
            }
            let t = match bname {
                "male" => vw.male,
                "female" => vw.female,
                _ => vw.child,
            };
            // initial guess from vowel-space position
            let mut x0 = vec![
                3.5 - 1.3 * vw.back + 0.2 * vw.height * (1.0 - vw.back),
                8.0 - 2.0 * vw.height - 0.25 * vw.back,
                0.25 + 1.4 * vw.height,
                if vw.round {
                    0.35 + 0.6 * vw.height
                } else {
                    0.6 + 1.0 * vw.height
                },
                if vw.round { 0.9 } else { 0.1 },
                if vw.round { 0.12 } else { 0.85 },
                0.4 * (1.0 - vw.back) - 0.5 * vw.height * vw.back,
                0.4 * (1.0 - vw.height) * (1.0 - vw.back) - if vw.round { 0.4 } else { 0.0 },
                if vw.ipa == "ɝ" { 0.6 } else if vw.ipa == "ʅ" { 0.9 } else { 0.0 },
            ];
            if vw.ipa == "ɯ" {
                x0 = vec![2.6, 7.35, 0.25, 0.5, 0.1, 0.65, 0.4, 0.2, 0.0];
            }
            // fork: the tongue body starts, and its prior centres, where J. Beck's sits (BECK_TONGUE)
            // only the female body, hers; the male and child bodies are fitted as before
            let beck = if bname == "female" { BECK_TONGUE.iter().find(|(v, _, _)| *v == vw.ipa).map(|&(_, x, y)| (x, y)) } else { None };
            if let Some((x, y)) = beck {
                (x0[0], x0[1]) = (x, y);
            }
            // a declared start replaces the generic guess, and the priors below centre on it too
            if let Some(start) = start_from(vw.ipa, bname)
                .and_then(|(b, e)| match &results {
                    // the body being fitted is not in the results yet: its own table so far
                    _ if b == bname => solved_posture(&table, e),
                    Json::Obj(m) => match &m.iter().find(|(k, _)| k == b)?.1 {
                        Json::Obj(t) => solved_posture(t, e),
                        _ => None,
                    },
                    _ => None,
                })
            {
                x0 = start;
            }
            if let Some((_, _, p)) = STARTS.iter().find(|(v, b, _)| *v == vw.ipa && *b == bname) {
                x0 = p.to_vec();
            }
            let jaw_prior = 0.2 + 1.5 * vw.height;
            let pinned = apical(vw.ipa, bname, i_lips);
            // the apical vowels' tongue body and jaw are held a little nearer the vowel-space guess
            // (a close vowel, the tongue hardly leaving the consonant): with the tip and lips
            // pinned the fit otherwise spent the jaw (1.4) and the body on the formants. 0.05 held
            // them best but cost F2 +10% and F3 -10%; 0.01 keeps the formants within 11%.
            let (w_body0, w_jaw) = if pinned.is_some() {
                (0.01, 0.01)
            } else if beck.is_some() {
                // a measured posture: held firmer than the vowel-space guess
                (W_BECK, 0.004)
            } else {
                (0.006, 0.004)
            };
            let w_body = std::cell::Cell::new(w_body0);
            let mut cost = |p: &[f64]| -> f64 {
                let a = to_artic(p, pinned.as_ref());
                // the apical vowels' fixed lips and curl, in the priors below too
                let mut q = p.to_vec();
                if let Some(f) = &pinned {
                    (q[3], q[4], q[5], q[8]) = (f.lip_ap, f.lip_prot, f.lip_width, f.curl);
                }
                let p = &q[..];
                let (f, min_area) = formants(&mut air, &freqs, &a);
                if f.len() < 3 {
                    return 50.0;
                }
                let mut c = 0.0;
                let w = [1.0, 1.0, if matches!(vw.ipa, "ɝ" | "ʅ") { 1.2 } else { 0.45 }];
                for i in 0..3 {
                    c += w[i] * pow(ln(f[i] / t[i]), 2.0);
                }
                // physiology priors (soft)
                let min_ok = 0.22 * k * k;
                if min_area < min_ok {
                    c += 0.4 * pow((min_ok - min_area) / min_ok, 2.0);
                }
                // fork: ɒ uses everything the body has (user, 2026-11-26): no pull toward a jaw, a
                // lip height, a root, a larynx or a tongue place, only its formants and its rounding
                // (lips no wider than they were, 2.5 cm, and protruded). With the pulls it stopped
                // at 689/972 for 740/940; without them it reaches 756/1019 for 780/1000.
                let all_out = bname == "female" && vw.ipa == "ɒ";
                if all_out {
                    return c + 5.0 * pow(js_max(0.0, p[5] - 0.51), 2.0) + 2.0 * pow(js_max(0.0, 0.33 - p[4]), 2.0);
                }
                // fork: ɶ aims where ɐ̞ sounds (791/1476) and differs by its lips alone (user,
                // 2026-11-26): at most 2.3 cm wide, at least 0.75 cm apart, protruded 0.1, and no
                // other pull. With the pulls and lips 1.0 cm apart, protruded 0.2, it stopped at
                // 730/1501; without them it reaches 791/1476 (lips 1.47 x 2.3 cm, jaw 1.84).
                if bname == "female" && vw.ipa == "ɶ" {
                    return c + 5.0 * pow(js_max(0.0, p[5] - 0.45), 2.0) + 2.0 * pow(p[4] - 0.1, 2.0) + 5.0 * pow(js_max(0.0, 0.833 - p[3]), 2.0);
                }
                c += w_jaw * pow(p[2] - jaw_prior, 2.0);
                let open = vw.height > 0.6;
                if vw.round {
                    let (w_max, p_min, ap_max) = (
                        if open { 0.45 } else { 0.3 },
                        if open { 0.35 } else { 0.6 },
                        0.3 + 0.7 * vw.height,
                    );
                    c += 0.05 * pow(js_max(0.0, p[5] - w_max), 2.0)
                        + 0.05 * pow(js_max(0.0, p_min - p[4]), 2.0)
                        + 0.05 * pow(js_max(0.0, p[3] - ap_max), 2.0);
                } else {
                    let ap_prior = 0.35 + 0.75 * p[2];
                    c += 0.05 * pow(js_max(0.0, 0.55 - p[5]), 2.0)
                        + 0.05 * pow(js_max(0.0, p[4] - 0.35), 2.0)
                        + 0.03 * pow(js_max(0.0, (p[3] - ap_prior).abs() - 0.35), 2.0);
                    // fork: a front unrounded vowel fitted to Beck keeps its lips spread. Her [i]'s F1
                    // (235 Hz, near her f0) pulled the lips to the edge of the slack above (protrusion
                    // 0.35, width 0.55): a half-rounded [i], and ɿ ʅ take their lips from it
                    // The central and back unrounded ones (ɨ ɤ ʌ ɜ ɐ) reached her low F2 the same way
                    // (ɤ: aperture 0.12, protrusion 0.42, width 0.09, an [o]'s lips): no protrusion,
                    // and the lips neither narrowed nor closed past the neutral range
                    if bname == "female" && vw.back < 0.5 {
                        c += 2.0 * (p[4] * p[4] + pow(1.0 - p[5], 2.0));
                    } else if bname == "female" {
                        c += 2.0 * (p[4] * p[4] + pow(js_max(0.0, 0.55 - p[5]), 2.0));
                    }
                    // fork: and the lips open as far as hers do (BECK_LIP_AP). Held to the tongue alone,
                    // the fit shut the lips to reach her formants (ə ɯ ɤ ʌ all 0.27 cm high, hers
                    // 0.7 to 1.9 cm). An unrounded vowel she did not record opens at least as far
                    // as the least open of her mid ones (ɤ)
                    if bname == "female" {
                        c += W_BECK_LIP * match BECK_LIP_AP.iter().find(|(v, _)| *v == vw.ipa) {
                            Some(&(v, ap)) => pow(
                                p[3] - match v {
                                    "ʌ" => LIP_AP_WEDGE,
                                    "ɤ" => LIP_AP_RAMS_HORN,
                                    "ɤ̞" => LIP_AP_RAMS_HORN_LOWERED,
                                    // her open vowels' full reading is the model's limit already
                                    "a" | "ɛ" | "æ" => ap,
                                    _ => BECK_LIP_SCALE * ap,
                                },
                                2.0,
                            ),
                            None => pow(js_max(0.0, BECK_LIP_SCALE * 1.3 - p[3]), 2.0),
                        };
                    }
                }
                // fork: the rounded vowels' lips part at least as far as BECK_LIP_AP_ROUND says: the
                // fit reaches the same formants with a taller, narrower, more protruded opening
                if bname == "female" {
                    if let Some(&(_, ap)) = BECK_LIP_AP_ROUND.iter().find(|(v, _)| *v == vw.ipa) {
                        c += 5.0 * pow(js_max(0.0, ap - p[3]), 2.0);
                    }
                }
                c += 0.002 * p[6] * p[6] + 0.006 * p[7] * p[7];
                // the tongue body near where the vowel's height and backness put it: many postures
                // give the same formants, and without this the fit wandered (a schwa with the
                // tongue as low as /ɑ/), making every transition through it needlessly long
                c += w_body.get() * (pow(p[0] - x0[0], 2.0) + pow(p[1] - x0[1], 2.0));
                if !matches!(vw.ipa, "ɝ" | "ʅ") {
                    c += 0.5 * p[8] * p[8];
                }
                c
            };
            let opts = |step: [f64; 9], max_iter: usize| NelderMeadOpts {
                lo: Some(LO.to_vec()),
                hi: Some(HI.to_vec()),
                step: Some(step.to_vec()),
                max_iter: Some(max_iter),
                tol: None,
            };
            let mut best = nelder_mead(
                &mut cost,
                &x0,
                opts([0.4, 0.4, 0.3, 0.25, 0.3, 0.25, 0.3, 0.3, 0.2], 1800),
            );
            // restart from the best point to escape shallow minima
            for _ in 0..2 {
                let nb = nelder_mead(
                    &mut cost,
                    &best.x,
                    opts([0.2, 0.2, 0.15, 0.12, 0.15, 0.12, 0.15, 0.15, 0.1], 1000),
                );
                if nb.fx < best.fx {
                    best = nb;
                }
            }
            // fork: where the body cannot sound the vowel from Beck's posture (F1 or F2 more than 10 %
            // off: the corner vowels, lips or jaw at their limits), the posture gives way: refit with
            // the prior at W_BECK_SOFT, and keep that if its formants are nearer
            if beck.is_some() {
                let miss = |x: &[f64], air: &mut HumanAirway| -> f64 {
                    let (f, _) = formants(air, &freqs, &to_artic(x, pinned.as_ref()));
                    if f.len() < 2 { f64::MAX } else { js_max(ln(f[0] / t[0]).abs(), ln(f[1] / t[1]).abs()) }
                };
                let held = miss(&best.x, &mut air_check);
                if held > 0.095 {
                    w_body.set(W_BECK_SOFT);
                    let mut soft = nelder_mead(&mut cost, &best.x, opts([0.4, 0.4, 0.3, 0.25, 0.3, 0.25, 0.3, 0.3, 0.2], 1800));
                    for _ in 0..2 {
                        let nb = nelder_mead(&mut cost, &soft.x, opts([0.2, 0.2, 0.15, 0.12, 0.15, 0.12, 0.15, 0.15, 0.1], 1000));
                        if nb.fx < soft.fx {
                            soft = nb;
                        }
                    }
                    if miss(&soft.x, &mut air_check) < held {
                        best = soft;
                    }
                }
            }
            let a = to_artic(&best.x, pinned.as_ref());
            if vw.ipa == "i" {
                i_lips = Some([a.lip_ap, a.lip_prot, a.lip_width]);
            }
            let (f, _) = formants(&mut air, &freqs, &a);
            let err: Vec<f64> = (0..3)
                .map(|i| match f.get(i) {
                    Some(&fi) if fi != 0.0 && !fi.is_nan() => (100.0 * (fi - t[i])) / t[i],
                    _ => f64::NAN,
                })
                .collect();
            let nums = |xs: &mut dyn Iterator<Item = f64>| Json::Arr(xs.map(Json::Num).collect());
            let artic = [
                ("tbX", a.tb_x),
                ("tbY", a.tb_y),
                ("jaw", a.jaw),
                ("lipAp", a.lip_ap),
                ("lipProt", a.lip_prot),
                ("lipWidth", a.lip_width),
                ("root", a.root),
                ("larynx", a.larynx),
                ("ttX", a.tt_x),
                ("ttY", a.tt_y),
                ("ttCurl", a.tt_curl),
            ];
            table.push((
                vw.ipa.to_string(),
                Json::Obj(vec![
                    (
                        "artic".into(),
                        Json::Obj(
                            artic
                                .iter()
                                .map(|(k, x)| (k.to_string(), Json::Num(round3(*x))))
                                .collect(),
                        ),
                    ),
                    (
                        "formants".into(),
                        nums(&mut f.iter().take(4).map(|&x| js_round(x))),
                    ),
                    ("target".into(), nums(&mut t.iter().map(|&x| js_round(x)))),
                    (
                        "err".into(),
                        nums(&mut err.iter().map(|&e| js_round(e * 10.0) / 10.0)),
                    ),
                ]),
            ));
            println!(
                "{} {} F={}  T={}  err%={}  tb=({},{}) jaw={} lip={}/{}/{} root={} lx={} curl={}",
                pad_end(bname, 6),
                pad_end(vw.ipa, 2),
                f.iter()
                    .take(3)
                    .map(|x| pad_start(&fixed(*x, 0), 5))
                    .collect::<String>(),
                t.iter()
                    .map(|x| pad_start(&fixed(*x, 0), 5))
                    .collect::<String>(),
                err.iter()
                    .map(|e| pad_start(&fixed(*e, 1), 6))
                    .collect::<String>(),
                fixed(a.tb_x, 2),
                fixed(a.tb_y, 2),
                fixed(a.jaw, 2),
                fixed(a.lip_ap, 2),
                fixed(a.lip_prot, 2),
                fixed(a.lip_width, 2),
                fixed(a.root, 2),
                fixed(a.larynx, 2),
                fixed(a.tt_curl, 2)
            );
        }
        results.set(bname, Json::Obj(table));
    }
    std::fs::write(&out, results.stringify(1)).map_err(|e| format!("{out}: {e}"))?;
    println!("wrote {out}");
    Ok(())
}

/// The main coordinate of each fricative set so the constriction area matches its target
/// (adult-male target × body scale²) in an /a/-like context: bisection on the monotonic contact
/// model. Reads the vowel table (its /ɑ/) from disk. `calibrate consonants [--out file]`
pub fn consonants(a: &[String]) -> Result<(), String> {
    let (_, out) = out_option(a, CONSONANT_TABLE)?;
    let vpath = repo_path(VOWEL_TABLE);
    let vtext = std::fs::read_to_string(&vpath).map_err(|e| format!("{vpath}: {e}"))?;
    let vtable: VowelTable = serde_json::from_str(&vtext).map_err(|e| format!("{vpath}: {e}"))?;
    let mut result: Vec<(String, Json)> = Vec::new();
    for (name, body) in BODIES {
        let mut air = HumanAirway::new(96000.0, body);
        let mut ctx = vtable[name]["ɑ"].artic.apply_to(&NEUTRAL);
        ctx.set(ArticKey::Jaw, 0.55);
        let mut table: Vec<(String, Json)> = Vec::new();
        for p in inventory() {
            let (Some(area), Some(key)) = (p.area, p.cal_key) else {
                continue;
            };
            if area == 0.0 || area.is_nan() {
                continue;
            }
            let target = area * body.scale * body.scale;
            let mut min_area = |v: f64| -> f64 {
                let mut a = p.targets.apply_to(&ctx);
                a.set(key, v);
                air.apply(&a, 0.0);
                let mut m = f64::INFINITY;
                for &x in air.ph_min.iter().chain(air.or_min.iter()) {
                    if x < m {
                        m = x;
                    }
                }
                m
            };
            // search direction: area decreases as the coordinate moves toward closure
            let base = p.targets.get(key).unwrap_or(f64::NAN);
            let (mut lo, mut hi) = (base - 1.2, base + 1.2);
            let decreasing = min_area(hi) < min_area(lo); // area falls as value rises?
            for _ in 0..40 {
                let mid = (lo + hi) / 2.0;
                let area_mid = min_area(mid);
                if (area_mid > target) == decreasing {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let v = (lo + hi) / 2.0;
            let got = min_area(v);
            table.push((
                p.ipa.to_string(),
                Json::Obj(vec![(key.name().to_string(), Json::Num(round3(v)))]),
            ));
            println!(
                "{} {} {}={} area {} (target {})",
                pad_end(name, 6),
                pad_end(p.ipa, 3),
                key.name(),
                fixed(v, 3),
                fixed(got, 3),
                fixed(target, 3)
            );
        }
        result.push((name.to_string(), Json::Obj(table)));
    }
    std::fs::write(&out, Json::Obj(result).stringify(1)).map_err(|e| format!("{out}: {e}"))?;
    Ok(())
}
