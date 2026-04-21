// ETNA workload runner for half.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: F16MaxMinNan | F16OrdSignMagnitude | F16SubnormalRoundtrip | All
//
// Each run emits a single JSON line on stdout with fields:
//   status, tests, discards, time, counterexample, error, tool, property.
// Exit status is always 0 on completion; non-zero exit is reserved for
// adapter-level panics that escape the catch_unwind in main().

use crabcheck::quickcheck as crabcheck_qc;
use half::etna::{
    property_f16_max_min_nan, property_f16_ord_sign_magnitude,
    property_f16_subnormal_roundtrip, PropertyResult,
};
use hegel::{generators as hgen, Hegel, Settings as HegelSettings};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestRunner};
use quickcheck::{QuickCheck, ResultStatus, TestResult};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &[
    "F16MaxMinNan",
    "F16OrdSignMagnitude",
    "F16SubnormalRoundtrip",
];

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    let mut final_status: Result<(), String> = Ok(());
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if r.is_err() && final_status.is_ok() {
            final_status = r;
        }
    }
    (final_status, total)
}

// ---- etna (deterministic witness-shaped inputs) ----

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "F16MaxMinNan" => to_err(property_f16_max_min_nan((
            false,
            half::f16::NAN.to_bits(),
            half::f16::from_f32(42.0).to_bits(),
        ))),
        "F16OrdSignMagnitude" => to_err(property_f16_ord_sign_magnitude((
            half::f16::from_f32(-1.0).to_bits(),
            half::f16::from_f32(-2.0).to_bits(),
        ))),
        "F16SubnormalRoundtrip" => to_err(property_f16_subnormal_roundtrip(0x0001)),
        _ => {
            return (
                Err(format!("Unknown property for etna: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    (result, Metrics { inputs: 1, elapsed_us })
}

// ---- proptest ----

fn max_min_strategy() -> BoxedStrategy<(bool, u16, u16)> {
    (any::<bool>(), any::<u16>(), any::<u16>()).boxed()
}

fn ord_strategy() -> BoxedStrategy<(u16, u16)> {
    (any::<u16>(), any::<u16>()).boxed()
}

fn subnormal_strategy() -> BoxedStrategy<u16> {
    // Bias the strategy toward subnormal bit patterns (exp==0, mantissa!=0)
    // so we actually exercise the subnormal path within the configured case
    // budget. Still include arbitrary u16 so the property sees the full
    // domain (most will be discarded) and shrinking hits interesting values.
    prop_oneof![
        (1u16..=0x03FF).prop_map(|m| m), // positive subnormals
        (1u16..=0x03FF).prop_map(|m| 0x8000 | m), // negative subnormals
        any::<u16>(),
    ]
    .boxed()
}

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let mut runner = TestRunner::new(ProptestConfig::default());
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "F16MaxMinNan" => runner
            .run(&max_min_strategy(), move |args| {
                c.fetch_add(1, Ordering::Relaxed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_f16_max_min_nan(args)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({:?})", args)))
                    }
                }
            })
            .map_err(|e| match e {
                proptest::test_runner::TestError::Fail(r, _) => r.to_string(),
                other => other.to_string(),
            }),
        "F16OrdSignMagnitude" => runner
            .run(&ord_strategy(), move |args| {
                c.fetch_add(1, Ordering::Relaxed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_f16_ord_sign_magnitude(args)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({:?})", args)))
                    }
                }
            })
            .map_err(|e| match e {
                proptest::test_runner::TestError::Fail(r, _) => r.to_string(),
                other => other.to_string(),
            }),
        "F16SubnormalRoundtrip" => runner
            .run(&subnormal_strategy(), move |args| {
                c.fetch_add(1, Ordering::Relaxed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_f16_subnormal_roundtrip(args)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("(0x{:04x})", args)))
                    }
                }
            })
            .map_err(|e| match e {
                proptest::test_runner::TestError::Fail(r, _) => r.to_string(),
                other => other.to_string(),
            }),
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ---- quickcheck ----

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_f16_max_min_nan(is_min: bool, a: u16, b: u16) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_f16_max_min_nan((is_min, a, b)) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_f16_ord_sign_magnitude(a: u16, b: u16) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_f16_ord_sign_magnitude((a, b)) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_f16_subnormal_roundtrip(bits: u16) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_f16_subnormal_roundtrip(bits) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let result = match property {
        "F16MaxMinNan" => QuickCheck::new()
            .tests(200)
            .max_tests(2000)
            .max_time(Duration::from_secs(86_400))
            .quicktest(qc_f16_max_min_nan as fn(bool, u16, u16) -> TestResult),
        "F16OrdSignMagnitude" => QuickCheck::new()
            .tests(200)
            .max_tests(2000)
            .max_time(Duration::from_secs(86_400))
            .quicktest(qc_f16_ord_sign_magnitude as fn(u16, u16) -> TestResult),
        "F16SubnormalRoundtrip" => QuickCheck::new()
            .tests(200)
            .max_tests(20_000)
            .max_time(Duration::from_secs(86_400))
            .quicktest(qc_f16_subnormal_roundtrip as fn(u16) -> TestResult),
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        }
        ResultStatus::Aborted { err } => Err(format!("aborted: {err:?}")),
        ResultStatus::TimedOut => Err("timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "gave up: passed={}, discarded={}",
            result.n_tests_passed, result.n_tests_discarded
        )),
    };
    (status, metrics)
}

// ---- crabcheck ----

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_f16_max_min_nan(args: (bool, u16, u16)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_f16_max_min_nan(args) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_f16_ord_sign_magnitude(args: (u16, u16)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_f16_ord_sign_magnitude(args) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_f16_subnormal_roundtrip(bits: u16) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_f16_subnormal_roundtrip(bits) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let cfg = crabcheck_qc::Config { tests: 200 };
    let result = match property {
        "F16MaxMinNan" => crabcheck_qc::quickcheck_with_config(
            cfg,
            cc_f16_max_min_nan as fn((bool, u16, u16)) -> Option<bool>,
        ),
        "F16OrdSignMagnitude" => crabcheck_qc::quickcheck_with_config(
            cfg,
            cc_f16_ord_sign_magnitude as fn((u16, u16)) -> Option<bool>,
        ),
        "F16SubnormalRoundtrip" => crabcheck_qc::quickcheck_with_config(
            crabcheck_qc::Config { tests: 20_000 },
            cc_f16_subnormal_roundtrip as fn(u16) -> Option<bool>,
        ),
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        }
        crabcheck_qc::ResultStatus::TimedOut => Err("timed out".to_string()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => {
            Err(format!("aborted: {error}"))
        }
    };
    (status, metrics)
}

// ---- hegel ----

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new()
        .test_cases(200)
        .suppress_health_check(hegel::HealthCheck::all())
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "F16MaxMinNan" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let is_min = tc.draw(hgen::integers::<u8>()) & 1 == 1;
                let a = tc.draw(hgen::integers::<u16>());
                let b = tc.draw(hgen::integers::<u16>());
                let cex = format!("({:?})", (is_min, a, b));
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_f16_max_min_nan((is_min, a, b))
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{cex}"),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "F16OrdSignMagnitude" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let a = tc.draw(hgen::integers::<u16>());
                let b = tc.draw(hgen::integers::<u16>());
                let cex = format!("({:?})", (a, b));
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_f16_ord_sign_magnitude((a, b))
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{cex}"),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "F16SubnormalRoundtrip" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let bits = tc.draw(hgen::integers::<u16>());
                let cex = format!("(0x{:04x})", bits);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_f16_subnormal_roundtrip(bits)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{cex}"),
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("__unknown_property:{property}"),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "hegel panicked with non-string payload".to_string()
            };
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {rest}")),
                    Metrics::default(),
                );
            }
            Err(msg
                .strip_prefix("Property test failed: ")
                .unwrap_or(&msg)
                .to_string())
        }
    };
    (status, metrics)
}

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (Err(format!("Unknown tool: {tool}")), Metrics::default()),
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!(
            "Properties: F16MaxMinNan | F16OrdSignMagnitude | F16SubnormalRoundtrip | All"
        );
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "panic with non-string payload".to_string()
            };
            emit_json(
                tool,
                property,
                "aborted",
                Metrics::default(),
                None,
                Some(&format!("adapter panic: {msg}")),
            );
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(msg) => emit_json(tool, property, "failed", metrics, Some(&msg), None),
    }
}
