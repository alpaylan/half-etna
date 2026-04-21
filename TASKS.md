# half — ETNA Tasks

Total tasks: 12

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task: the command executes the framework-specific adapter against the buggy variant branch and should report a counterexample (or time out).

Run against a variant by first checking out its branch (`git checkout etna/<variant>`) or applying its patch on a clean `base_commit` tree (`git apply patches/<variant>.patch`). `max_min_nan_handling_a61f31b_1` is also reproducible via marauders on `src/binary16.rs` (marker `max_min_nan_handling`).

## Task Index

| Task | Variant | Framework | Property | Witness(es) | Command |
|------|---------|-----------|----------|-------------|---------|
| 001 | `max_min_nan_handling_a61f31b_1` | proptest | `property_f16_max_min_nan` | `witness_f16_max_min_nan_case_self_nan_max`, `witness_f16_max_min_nan_case_self_nan_min` | `cargo run --release --bin etna -- proptest F16MaxMinNan` |
| 002 | `max_min_nan_handling_a61f31b_1` | quickcheck | `property_f16_max_min_nan` | `witness_f16_max_min_nan_case_self_nan_max`, `witness_f16_max_min_nan_case_self_nan_min` | `cargo run --release --bin etna -- quickcheck F16MaxMinNan` |
| 003 | `max_min_nan_handling_a61f31b_1` | crabcheck | `property_f16_max_min_nan` | `witness_f16_max_min_nan_case_self_nan_max`, `witness_f16_max_min_nan_case_self_nan_min` | `cargo run --release --bin etna -- crabcheck F16MaxMinNan` |
| 004 | `max_min_nan_handling_a61f31b_1` | hegel | `property_f16_max_min_nan` | `witness_f16_max_min_nan_case_self_nan_max`, `witness_f16_max_min_nan_case_self_nan_min` | `cargo run --release --bin etna -- hegel F16MaxMinNan` |
| 005 | `partial_cmp_sign_magnitude_22b5bd6_1` | proptest | `property_f16_ord_sign_magnitude` | `witness_f16_ord_sign_magnitude_case_neg_order`, `witness_f16_ord_sign_magnitude_case_neg_zero_eq`, `witness_f16_ord_sign_magnitude_case_mixed_sign` | `cargo run --release --bin etna -- proptest F16OrdSignMagnitude` |
| 006 | `partial_cmp_sign_magnitude_22b5bd6_1` | quickcheck | `property_f16_ord_sign_magnitude` | `witness_f16_ord_sign_magnitude_case_neg_order`, `witness_f16_ord_sign_magnitude_case_neg_zero_eq`, `witness_f16_ord_sign_magnitude_case_mixed_sign` | `cargo run --release --bin etna -- quickcheck F16OrdSignMagnitude` |
| 007 | `partial_cmp_sign_magnitude_22b5bd6_1` | crabcheck | `property_f16_ord_sign_magnitude` | `witness_f16_ord_sign_magnitude_case_neg_order`, `witness_f16_ord_sign_magnitude_case_neg_zero_eq`, `witness_f16_ord_sign_magnitude_case_mixed_sign` | `cargo run --release --bin etna -- crabcheck F16OrdSignMagnitude` |
| 008 | `partial_cmp_sign_magnitude_22b5bd6_1` | hegel | `property_f16_ord_sign_magnitude` | `witness_f16_ord_sign_magnitude_case_neg_order`, `witness_f16_ord_sign_magnitude_case_neg_zero_eq`, `witness_f16_ord_sign_magnitude_case_mixed_sign` | `cargo run --release --bin etna -- hegel F16OrdSignMagnitude` |
| 009 | `subnormal_conversion_18e6467_1` | proptest | `property_f16_subnormal_roundtrip` | `witness_f16_subnormal_roundtrip_case_min_positive_subnormal`, `witness_f16_subnormal_roundtrip_case_mid_subnormal`, `witness_f16_subnormal_roundtrip_case_neg_subnormal` | `cargo run --release --bin etna -- proptest F16SubnormalRoundtrip` |
| 010 | `subnormal_conversion_18e6467_1` | quickcheck | `property_f16_subnormal_roundtrip` | `witness_f16_subnormal_roundtrip_case_min_positive_subnormal`, `witness_f16_subnormal_roundtrip_case_mid_subnormal`, `witness_f16_subnormal_roundtrip_case_neg_subnormal` | `cargo run --release --bin etna -- quickcheck F16SubnormalRoundtrip` |
| 011 | `subnormal_conversion_18e6467_1` | crabcheck | `property_f16_subnormal_roundtrip` | `witness_f16_subnormal_roundtrip_case_min_positive_subnormal`, `witness_f16_subnormal_roundtrip_case_mid_subnormal`, `witness_f16_subnormal_roundtrip_case_neg_subnormal` | `cargo run --release --bin etna -- crabcheck F16SubnormalRoundtrip` |
| 012 | `subnormal_conversion_18e6467_1` | hegel | `property_f16_subnormal_roundtrip` | `witness_f16_subnormal_roundtrip_case_min_positive_subnormal`, `witness_f16_subnormal_roundtrip_case_mid_subnormal`, `witness_f16_subnormal_roundtrip_case_neg_subnormal` | `cargo run --release --bin etna -- hegel F16SubnormalRoundtrip` |

## Witness catalog

Each witness is a deterministic concrete test in `tests/etna_witnesses.rs`. On `base_commit` every witness passes. On each variant branch the witnesses listed for that variant fail; witnesses for the other two variants keep passing, which also serves as a negative control.

### `property_f16_max_min_nan`

- `witness_f16_max_min_nan_case_self_nan_max` — `f16::NAN.max(42.0)` → expected `42.0`. Under the bug, the comparison `other > NaN` is false, so `self` (NaN) is returned.
- `witness_f16_max_min_nan_case_self_nan_min` — mirror of the above for `min`.
- `witness_f16_max_min_nan_case_other_nan_max` — `42.0.max(NaN)` → `42.0`. Positive control: the `!other.is_nan()` guard still works, so this stays green under the bug.
- `witness_f16_max_min_nan_case_both_nan_max` — `NaN.max(NaN)` → result must be NaN. Positive control: whichever branch the code picks, the output is NaN, so this stays green.
- `witness_f16_max_min_nan_case_non_nan_max` — `1.0.max(2.0)` → `2.0`. Positive control: no NaN involved; green on base and variant.

### `property_f16_ord_sign_magnitude`

- `witness_f16_ord_sign_magnitude_case_neg_order` — `partial_cmp(f16(-1.0), f16(-2.0))` must equal `Some(Greater)`. Under the bug (raw u16 compare) `0xBC00 < 0xC000`, so the result is `Some(Less)`.
- `witness_f16_ord_sign_magnitude_case_neg_zero_eq` — `partial_cmp(+0.0, -0.0)` must equal `Some(Equal)`. Under the bug they differ by sign bit, so the result is `Some(Less)`.
- `witness_f16_ord_sign_magnitude_case_mixed_sign` — `partial_cmp(1.0, -0.0)` must equal `Some(Greater)`. Under the bug `0x3C00 < 0x8000`, so the result is `Some(Less)`.

### `property_f16_subnormal_roundtrip`

- `witness_f16_subnormal_roundtrip_case_min_positive_subnormal` — `f16::from_bits(0x0001).to_f32_const()` must equal `2^-24`. Under the bug the residual `(0x0001 << 13) = 0x2000` mantissa bits get OR'd in, producing `2^-24 * (1 + 2^-10)`.
- `witness_f16_subnormal_roundtrip_case_mid_subnormal` — `f16::from_bits(0x0200).to_f32_const()` must equal `2^-15`. Under the bug the shift picks the wrong exponent adjustment, landing on `2^-14`.
- `witness_f16_subnormal_roundtrip_case_neg_subnormal` — `f16::from_bits(0x8001).to_f32_const()` must equal `-2^-24`. Same bug path as the positive case; the sign bit survives but the magnitude is wrong.
