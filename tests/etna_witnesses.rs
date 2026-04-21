//! Witness tests for the ETNA workload.
//!
//! Each `witness_*` test calls one of the `property_*` functions in
//! `half::etna` with frozen inputs. Tests pass on the base commit and fail
//! when the corresponding mutation is active.

use half::etna::{
    property_f16_max_min_nan, property_f16_ord_sign_magnitude,
    property_f16_subnormal_roundtrip, PropertyResult,
};
use half::f16;

fn assert_pass(r: PropertyResult) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Fail(m) => panic!("property failed: {m}"),
        PropertyResult::Discard => panic!("property unexpectedly discarded"),
    }
}

// ---- max_min_nan_handling_a61f31b_1 ----

#[test]
fn witness_f16_max_min_nan_case_self_nan_max() {
    let a = f16::NAN.to_bits();
    let b = f16::from_f32(42.0).to_bits();
    assert_pass(property_f16_max_min_nan((false, a, b)));
}

#[test]
fn witness_f16_max_min_nan_case_self_nan_min() {
    let a = f16::NAN.to_bits();
    let b = f16::from_f32(42.0).to_bits();
    assert_pass(property_f16_max_min_nan((true, a, b)));
}

#[test]
fn witness_f16_max_min_nan_case_other_nan_max() {
    let a = f16::from_f32(42.0).to_bits();
    let b = f16::NAN.to_bits();
    assert_pass(property_f16_max_min_nan((false, a, b)));
}

#[test]
fn witness_f16_max_min_nan_case_both_nan_max() {
    let a = f16::NAN.to_bits();
    let b = f16::NAN.to_bits();
    assert_pass(property_f16_max_min_nan((false, a, b)));
}

#[test]
fn witness_f16_max_min_nan_case_non_nan_max() {
    let a = f16::from_f32(1.0).to_bits();
    let b = f16::from_f32(2.0).to_bits();
    assert_pass(property_f16_max_min_nan((false, a, b)));
}

// ---- partial_cmp_sign_magnitude_22b5bd6_1 ----

#[test]
fn witness_f16_ord_sign_magnitude_case_neg_order() {
    // -1.0 > -2.0 under IEEE, but raw u16 bits 0xBC00 < 0xC000.
    let a = f16::from_f32(-1.0).to_bits();
    let b = f16::from_f32(-2.0).to_bits();
    assert_pass(property_f16_ord_sign_magnitude((a, b)));
}

#[test]
fn witness_f16_ord_sign_magnitude_case_neg_zero_eq() {
    // +0 == -0 under IEEE.
    let a = f16::from_f32(0.0).to_bits();
    let b = f16::from_f32(-0.0).to_bits();
    assert_pass(property_f16_ord_sign_magnitude((a, b)));
}

#[test]
fn witness_f16_ord_sign_magnitude_case_mixed_sign() {
    // 1.0 > -0.0 under IEEE (and raw bits agree, so only checks the easy axis).
    let a = f16::from_f32(1.0).to_bits();
    let b = f16::from_f32(-0.0).to_bits();
    assert_pass(property_f16_ord_sign_magnitude((a, b)));
}

// ---- subnormal_conversion_18e6467_1 ----

#[test]
fn witness_f16_subnormal_roundtrip_case_min_positive_subnormal() {
    // 0x0001 is the smallest positive subnormal; true value is 2^-24.
    assert_pass(property_f16_subnormal_roundtrip(0x0001));
}

#[test]
fn witness_f16_subnormal_roundtrip_case_mid_subnormal() {
    // 0x0200 (half the max subnormal) => 2^-14 / 2 = 2^-15.
    assert_pass(property_f16_subnormal_roundtrip(0x0200));
}

#[test]
fn witness_f16_subnormal_roundtrip_case_neg_subnormal() {
    assert_pass(property_f16_subnormal_roundtrip(0x8001));
}
