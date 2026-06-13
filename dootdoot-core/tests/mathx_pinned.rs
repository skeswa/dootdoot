//! Owned-math pinned output tests.

use core::f64::consts::{FRAC_PI_2, PI};

use dootdoot_core::{cos, exp, sin, tanh};

#[test]
fn mathx_outputs_match_pinned_bits() {
    for (name, actual, expected_bits) in [
        ("sin(-pi)", sin(-PI), 0x0000_0000_0000_0000),
        ("sin(-pi/2)", sin(-FRAC_PI_2), 0xbfef_feb6_f5b4_8bbf),
        ("sin(-1)", sin(-1.0), 0xbfea_ed4e_d4ed_4ed5),
        ("sin(0)", sin(0.0), 0x0000_0000_0000_0000),
        ("sin(1)", sin(1.0), 0x3fea_ed4e_d4ed_4ed5),
        ("sin(pi/2)", sin(FRAC_PI_2), 0x3fef_feb6_f5b4_8bbf),
        ("sin(pi)", sin(PI), 0x8000_0000_0000_0000),
        ("cos(-pi)", cos(-PI), 0xbff0_0000_0000_0000),
        ("cos(-pi/2)", cos(-FRAC_PI_2), 0xbf4d_4fcd_8311_6800),
        ("cos(-1)", cos(-1.0), 0x3fe1_49f4_9f49_f49f),
        ("cos(0)", cos(0.0), 0x3ff0_0000_0000_0000),
        ("cos(1)", cos(1.0), 0x3fe1_49f4_9f49_f49f),
        ("cos(pi/2)", cos(FRAC_PI_2), 0xbf4d_4fcd_8311_6800),
        ("cos(pi)", cos(PI), 0xbff0_0000_0000_0000),
        ("exp(-1)", exp(-1.0), 0x3fd7_8b56_3631_0268),
        ("exp(0)", exp(0.0), 0x3ff0_0000_0000_0000),
        ("exp(1)", exp(1.0), 0x4005_bf0a_8b10_93e4),
        ("tanh(-1)", tanh(-1.0), 0xbfe8_5efa_b4cc_7bb3),
        ("tanh(0)", tanh(0.0), 0x0000_0000_0000_0000),
        ("tanh(1)", tanh(1.0), 0x3fe8_5efa_b4cc_7bb4),
    ] {
        assert_eq!(
            actual.to_bits(),
            expected_bits,
            "{name} should match its pinned bit pattern",
        );
    }
}
