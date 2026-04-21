# half — Injected Bugs

Total mutations: 3

Each `etna/<variant>` branch is a pre-applied snapshot containing exactly one buggy commit on top of `base_commit`. Variant 1 is also available as a marauders block (comment-toggled injection) in the base source; variants 2 and 3 are distributed edits and are applied via patch files.

## Bug Index

| # | Name | Variant | File(s) | Injection | Fix Commit |
|---|------|---------|---------|-----------|------------|
| 1 | `f16::max` / `f16::min` ignore self-NaN | `max_min_nan_handling_a61f31b_1` | `src/binary16.rs` | marauders | `a61f31bace6b63c6d3336a3867b5617976b0653e` |
| 2 | `PartialEq` / `PartialOrd` use raw u16 bit comparison | `partial_cmp_sign_magnitude_22b5bd6_1` | `src/binary16.rs` | patch | `22b5bd6c086d4caf8651bf5b866241ba8ce67407` |
| 3 | `f16::to_f32_const` subnormal uses normalized-mantissa shift | `subnormal_conversion_18e6467_1` | `src/binary16/arch.rs` | patch | `18e6467b52f821d21a7179d1da5ac5aded63b3bd` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `max_min_nan_handling_a61f31b_1` | `property_f16_max_min_nan` | `witness_f16_max_min_nan_case_self_nan_max`, `witness_f16_max_min_nan_case_self_nan_min`, `witness_f16_max_min_nan_case_other_nan_max`, `witness_f16_max_min_nan_case_both_nan_max`, `witness_f16_max_min_nan_case_non_nan_max` |
| `partial_cmp_sign_magnitude_22b5bd6_1` | `property_f16_ord_sign_magnitude` | `witness_f16_ord_sign_magnitude_case_neg_order`, `witness_f16_ord_sign_magnitude_case_neg_zero_eq`, `witness_f16_ord_sign_magnitude_case_mixed_sign` |
| `subnormal_conversion_18e6467_1` | `property_f16_subnormal_roundtrip` | `witness_f16_subnormal_roundtrip_case_min_positive_subnormal`, `witness_f16_subnormal_roundtrip_case_mid_subnormal`, `witness_f16_subnormal_roundtrip_case_neg_subnormal` |

## Framework Coverage

| Property | etna | proptest | quickcheck | crabcheck | hegel |
|----------|:----:|:--------:|:----------:|:---------:|:-----:|
| `property_f16_max_min_nan` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_f16_ord_sign_magnitude` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_f16_subnormal_roundtrip` | ✓ | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. `f16::max` / `f16::min` ignore self-NaN

- **Variant**: `max_min_nan_handling_a61f31b_1`
- **Location**: `src/binary16.rs` (`impl f16` `fn max`, `fn min`)
- **Property**: `property_f16_max_min_nan`
- **Witnesses**: `witness_f16_max_min_nan_case_self_nan_max`, `witness_f16_max_min_nan_case_self_nan_min` (both fail under the bug), plus three positive controls that stay green.
- **Fix commit**: `a61f31bace6b63c6d3336a3867b5617976b0653e` — `Fix bug #126: handle NaNs correctly in min() and max()`
- **Invariant violated**: For IEEE 754 max/min, if exactly one input is NaN the non-NaN argument must be returned; in particular `NaN.max(x)` must return `x`.
- **How the mutation triggers**: The buggy body reads `if other > self && !other.is_nan() { other } else { self }`. When `self` is NaN, `other > self` is `false` (any comparison with NaN is unordered), so the code returns `self` (NaN) instead of `other`. The fixed body explicitly checks `self.is_nan()` first.

### 2. `PartialEq` / `PartialOrd` use raw u16 bit comparison

- **Variant**: `partial_cmp_sign_magnitude_22b5bd6_1`
- **Location**: `src/binary16.rs` (`impl PartialEq for f16`, `impl PartialOrd for f16`)
- **Property**: `property_f16_ord_sign_magnitude`
- **Witnesses**: `witness_f16_ord_sign_magnitude_case_neg_order`, `witness_f16_ord_sign_magnitude_case_neg_zero_eq`, `witness_f16_ord_sign_magnitude_case_mixed_sign` (all fail under the bug).
- **Fix commit**: `22b5bd6c086d4caf8651bf5b866241ba8ce67407` — `fix comparisons and add some tests`
- **Invariant violated**: `f16` comparisons must agree with the equivalent `f32` comparison obtained by `f16::from_bits(x).to_f32().partial_cmp(&y.to_f32())`. Concretely: `-0.0 == 0.0`, `-1.0 > -2.0`, `1.0 > -0.0`.
- **How the mutation triggers**: The buggy impls use raw `u16` comparison (e.g. `self.0 < other.0`). In sign-magnitude representation this yields the wrong answer whenever (a) both operands are negative (larger magnitude ⇒ larger u16 ⇒ judged greater, but should be smaller) or (b) one operand is `-0.0` vs `+0.0` (different bits ⇒ judged unequal, but should be equal).

### 3. `f16::to_f32_const` subnormal uses normalized-mantissa shift

- **Variant**: `subnormal_conversion_18e6467_1`
- **Location**: `src/binary16/arch.rs` (`const fn f16_to_f32_fallback`)
- **Property**: `property_f16_subnormal_roundtrip`
- **Witnesses**: `witness_f16_subnormal_roundtrip_case_min_positive_subnormal`, `witness_f16_subnormal_roundtrip_case_mid_subnormal`, `witness_f16_subnormal_roundtrip_case_neg_subnormal` (all fail under the bug).
- **Fix commit**: `18e6467b52f821d21a7179d1da5ac5aded63b3bd` — `hopefully fix subnormals`
- **Invariant violated**: Every f16 value is exactly representable in f32, so for a subnormal f16 (`exp==0, mant!=0`) the fallback conversion `to_f32_const()` must produce exactly `mant * 2^-24` (with sign).
- **How the mutation triggers**: The buggy fallback pre-computes `man = (half_man & 0x03FF) << 13` outside the subnormal branch and reuses that shift for subnormals. The correct subnormal path, restored by the fix, computes `man = (half_man << (14 + e)) & 0x7F_FFFF` where `e` is the leading-zero count; the reused normal shift leaves residual bits that shift the result out of place (off by a factor `(1 + 2^-10)` or similar). Because the property exercises `to_f32_const()` (the software fallback) directly, the bug is detected even on targets with hardware FP16 intrinsics.

## Notes

- `marauders` injection: toggle by running the marauders tool with `--mutate` and the marker tag, or consult the base `src/binary16.rs` for the commented-out buggy block. The `etna/max_min_nan_handling_a61f31b_1` branch is the already-mutated form.
- `patch` injection: `git apply patches/<variant>.patch` against a clean checkout of `base_commit` reproduces the `etna/<variant>` branch HEAD exactly.
