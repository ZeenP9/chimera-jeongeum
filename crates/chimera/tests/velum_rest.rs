//! A lowered velum rests above the tongue: it does not pass through it.
//!
//! The velum is a flap that turns about the hard palate's edge as its port opens. Turning freely it
//! went through a back vowel's tongue body, so nasalizing the vowel sealed the mouth (child o u ɤ at
//! the planner's port of 0.3 cm²) and, a little wider, the pharynx behind it (female ɤ̃ at 0.65: the
//! folds stop, silence). 正音 nasalizes any vowel (ᅟᅳᇰ ᅟᅮᇰ), and French ɔ̃ ɒ̃ are back vowels.
use chimera::anatomy::human::{BODY_CHILD, BODY_FEMALE, BODY_MALE, HumanBody, HumanFrame, HumanVocalTract, NEUTRAL};
use chimera::phonetics::planner::vowel_table;

const BODIES: [(&str, HumanBody); 3] = [("male", BODY_MALE), ("female", BODY_FEMALE), ("child", BODY_CHILD)];

fn narrowest(tract: &mut HumanVocalTract, frame: &mut HumanFrame, a: &chimera::anatomy::human::HumanArtic) -> (f64, f64) {
    let (mut ph, mut or) = (vec![0.0; 20], vec![0.0; 24]);
    tract.compute(a, frame);
    tract.resample(a, frame, &mut ph, &mut or, None, None);
    (ph.iter().copied().fold(f64::MAX, f64::min), or.iter().copied().fold(f64::MAX, f64::min))
}

#[test]
fn lowering_the_velum_closes_neither_the_mouth_nor_the_pharynx_of_any_vowel() {
    let mut bad = Vec::new();
    for (name, body) in BODIES {
        let mut tract = HumanVocalTract::new(body);
        let mut frame = HumanFrame::default();
        let mut vowels: Vec<&String> = vowel_table()[name].keys().collect();
        vowels.sort();
        for vw in vowels {
            let mut a = vowel_table()[name][vw].artic.apply_to(&NEUTRAL);
            a.velum = 0.0;
            let (ph0, or0) = narrowest(&mut tract, &mut frame, &a);
            for port in [0.3, 0.45, 0.8, 1.2] {
                a.velum = port;
                let (ph, or) = narrowest(&mut tract, &mut frame, &a);
                // as open as with the velum raised, or open enough for a vowel. (The child's open
                // vowels still narrow behind the velum, to 0.25–0.3 cm² from 0.6, at ports of 0.8 and
                // more; nothing lowers it that far, the planner's ports being 0.3 and 0.45.)
                if ph < f64::min(0.2, 0.8 * ph0) || or < f64::min(0.1, 0.8 * or0) {
                    bad.push(format!("{name} {vw} port {port}: pharynx {ph:.3} (raised {ph0:.3}), mouth {or:.3} (raised {or0:.3})"));
                }
            }
        }
    }
    assert!(bad.is_empty(), "{} postures close as the velum lowers:\n{}", bad.len(), bad.join("\n"));
}
