//! ETNA benchmark harness.
//!
//! This module defines the framework-neutral `PropertyResult` enum plus one
//! `property_*` function per mined bug. Every framework adapter in
//! `src/bin/etna.rs` and every witness test calls into these functions.

#![allow(missing_docs)]

use crate::f16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

/// `f16::max` / `f16::min` must obey the IEEE-754 minNum/maxNum-style spec:
///
/// - If both inputs are NaN, the result is NaN.
/// - If exactly one input is NaN, the result equals the non-NaN input.
/// - Otherwise the result equals the actual larger (for max) or smaller (for
///   min) value and is non-NaN.
///
/// Bug `max_min_nan_handling_a61f31b_1` violates this by returning `self`
/// when `self` is NaN and `other` is non-NaN.
pub fn property_f16_max_min_nan(inputs: (bool, u16, u16)) -> PropertyResult {
    let (is_min, a_bits, b_bits) = inputs;
    let a = f16::from_bits(a_bits);
    let b = f16::from_bits(b_bits);
    let r = if is_min { a.min(b) } else { a.max(b) };
    let a_nan = a.is_nan();
    let b_nan = b.is_nan();
    match (a_nan, b_nan) {
        (true, true) => {
            if r.is_nan() {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!(
                    "both NaN but result 0x{:04x} is finite (is_min={})",
                    r.to_bits(),
                    is_min
                ))
            }
        }
        (true, false) => {
            if r.to_bits() == b.to_bits() {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!(
                    "self NaN, other=0x{:04x}, got 0x{:04x} (is_min={})",
                    b.to_bits(),
                    r.to_bits(),
                    is_min
                ))
            }
        }
        (false, true) => {
            if r.to_bits() == a.to_bits() {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!(
                    "other NaN, self=0x{:04x}, got 0x{:04x} (is_min={})",
                    a.to_bits(),
                    r.to_bits(),
                    is_min
                ))
            }
        }
        (false, false) => {
            if r.is_nan() {
                return PropertyResult::Fail(format!(
                    "both finite (0x{:04x},0x{:04x}) but result is NaN (is_min={})",
                    a_bits, b_bits, is_min
                ));
            }
            let af = a.to_f32();
            let bf = b.to_f32();
            let expected = if is_min {
                if af <= bf { a } else { b }
            } else if af >= bf {
                a
            } else {
                b
            };
            if r.to_bits() == expected.to_bits()
                || r.to_f32() == expected.to_f32()
            {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!(
                    "result 0x{:04x} (={}) != expected 0x{:04x} (={}) for inputs 0x{:04x} 0x{:04x} (is_min={})",
                    r.to_bits(),
                    r.to_f32(),
                    expected.to_bits(),
                    expected.to_f32(),
                    a_bits,
                    b_bits,
                    is_min
                ))
            }
        }
    }
}

/// `f16`'s `PartialOrd`/`PartialEq` must agree with IEEE-754 total ordering on
/// finite (non-NaN) values: the comparison of two f16 values must produce the
/// same result as comparing their f32-promoted counterparts. In particular:
///
/// - `-2.0` is strictly less than `-1.0` (sign-magnitude bit patterns flip).
/// - `+0.0 == -0.0`.
///
/// Bug `partial_cmp_sign_magnitude_22b5bd6_1` implements comparison by raw
/// u16 bit order, which breaks both properties.
pub fn property_f16_ord_sign_magnitude(inputs: (u16, u16)) -> PropertyResult {
    let a = f16::from_bits(inputs.0);
    let b = f16::from_bits(inputs.1);
    if a.is_nan() || b.is_nan() {
        return PropertyResult::Discard;
    }
    let af = a.to_f32();
    let bf = b.to_f32();
    let f32_cmp = af.partial_cmp(&bf);
    let f16_cmp = a.partial_cmp(&b);
    if f16_cmp != f32_cmp {
        return PropertyResult::Fail(format!(
            "partial_cmp(0x{:04x}={}, 0x{:04x}={}) = {:?}, f32 says {:?}",
            inputs.0, af, inputs.1, bf, f16_cmp, f32_cmp
        ));
    }
    if (a == b) != (af == bf) {
        return PropertyResult::Fail(format!(
            "eq(0x{:04x}={}, 0x{:04x}={}) = {}, f32 says {}",
            inputs.0,
            af,
            inputs.1,
            bf,
            a == b,
            af == bf
        ));
    }
    if (a < b) != (af < bf) {
        return PropertyResult::Fail(format!(
            "lt(0x{:04x}={}, 0x{:04x}={}) = {}, f32 says {}",
            inputs.0,
            af,
            inputs.1,
            bf,
            a < b,
            af < bf
        ));
    }
    PropertyResult::Pass
}

/// Converting a subnormal `f16` (exponent bits 0, mantissa != 0) to `f32`
/// must produce the exact real value the f16 represents (every f16 value is
/// exactly representable in f32). Concretely: the result must be
/// `mantissa * 2^-24` with the correct sign, and converting back with
/// `f16::from_f32` must return the original bits.
///
/// Bug `subnormal_conversion_18e6467_1` uses the normalized-mantissa shift
/// for subnormals, producing a value that is too large.
pub fn property_f16_subnormal_roundtrip(bits: u16) -> PropertyResult {
    let exp_bits = (bits >> 10) & 0x1F;
    let mant_bits = bits & 0x3FF;
    // Only subnormals are interesting for this property.
    if exp_bits != 0 || mant_bits == 0 {
        return PropertyResult::Discard;
    }
    let a = f16::from_bits(bits);
    let sign = if bits & 0x8000 != 0 { -1.0_f32 } else { 1.0_f32 };
    let expected = sign * (mant_bits as f32) * f32::powi(2.0, -24);
    // Exercise the software fallback directly so the subnormal path is
    // independent of per-CPU fp16 intrinsics (aarch64 fp16 / x86 f16c).
    let got = a.to_f32_const();
    if got != expected {
        return PropertyResult::Fail(format!(
            "f16(0x{:04x}).to_f32_const() = {} (0x{:08x}), expected {} (0x{:08x})",
            bits,
            got,
            got.to_bits(),
            expected,
            expected.to_bits()
        ));
    }
    let back = f16::from_f32_const(got);
    if back.to_bits() != bits {
        return PropertyResult::Fail(format!(
            "f16(0x{:04x}).to_f32_const().from_f32_const() = 0x{:04x}",
            bits,
            back.to_bits()
        ));
    }
    PropertyResult::Pass
}
