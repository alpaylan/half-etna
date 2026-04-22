# half — ETNA Tasks

Total tasks: 12

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `max_min_nan_handling_a61f31b_1` | proptest | `F16MaxMinNan` | `witness_f16_max_min_nan_case_self_nan_max` |
| 002 | `max_min_nan_handling_a61f31b_1` | quickcheck | `F16MaxMinNan` | `witness_f16_max_min_nan_case_self_nan_max` |
| 003 | `max_min_nan_handling_a61f31b_1` | crabcheck | `F16MaxMinNan` | `witness_f16_max_min_nan_case_self_nan_max` |
| 004 | `max_min_nan_handling_a61f31b_1` | hegel | `F16MaxMinNan` | `witness_f16_max_min_nan_case_self_nan_max` |
| 005 | `partial_cmp_sign_magnitude_22b5bd6_1` | proptest | `F16OrdSignMagnitude` | `witness_f16_ord_sign_magnitude_case_neg_order` |
| 006 | `partial_cmp_sign_magnitude_22b5bd6_1` | quickcheck | `F16OrdSignMagnitude` | `witness_f16_ord_sign_magnitude_case_neg_order` |
| 007 | `partial_cmp_sign_magnitude_22b5bd6_1` | crabcheck | `F16OrdSignMagnitude` | `witness_f16_ord_sign_magnitude_case_neg_order` |
| 008 | `partial_cmp_sign_magnitude_22b5bd6_1` | hegel | `F16OrdSignMagnitude` | `witness_f16_ord_sign_magnitude_case_neg_order` |
| 009 | `subnormal_conversion_18e6467_1` | proptest | `F16SubnormalRoundtrip` | `witness_f16_subnormal_roundtrip_case_min_positive_subnormal` |
| 010 | `subnormal_conversion_18e6467_1` | quickcheck | `F16SubnormalRoundtrip` | `witness_f16_subnormal_roundtrip_case_min_positive_subnormal` |
| 011 | `subnormal_conversion_18e6467_1` | crabcheck | `F16SubnormalRoundtrip` | `witness_f16_subnormal_roundtrip_case_min_positive_subnormal` |
| 012 | `subnormal_conversion_18e6467_1` | hegel | `F16SubnormalRoundtrip` | `witness_f16_subnormal_roundtrip_case_min_positive_subnormal` |

## Witness Catalog

- `witness_f16_max_min_nan_case_self_nan_max` — base passes, variant fails
- `witness_f16_max_min_nan_case_self_nan_min` — base passes, variant fails
- `witness_f16_max_min_nan_case_other_nan_max` — base passes, variant fails
- `witness_f16_max_min_nan_case_both_nan_max` — base passes, variant fails
- `witness_f16_max_min_nan_case_non_nan_max` — base passes, variant fails
- `witness_f16_ord_sign_magnitude_case_neg_order` — base passes, variant fails
- `witness_f16_ord_sign_magnitude_case_neg_zero_eq` — base passes, variant fails
- `witness_f16_ord_sign_magnitude_case_mixed_sign` — base passes, variant fails
- `witness_f16_subnormal_roundtrip_case_min_positive_subnormal` — base passes, variant fails
- `witness_f16_subnormal_roundtrip_case_mid_subnormal` — base passes, variant fails
- `witness_f16_subnormal_roundtrip_case_neg_subnormal` — base passes, variant fails
