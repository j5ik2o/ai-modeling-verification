use spec_tests::adapters::{ModelASession, ModelBSession};
use spec_tests::{BillingResult, BillingSession, ClosedBillingSession};

const BASE_START_MS: i64 = 0;
const MINUTE_MS: i64 = 60 * 1_000;

struct StopCase {
    name: &'static str,
    duration_minutes: i64,
    energy_milli: i64,
    rate_yen_per_kwh: u32,
    expected: BillingResult,
}

const STOP_CASES: &[StopCase] = &[
    StopCase {
        name: "scenario1_six_minutes",
        duration_minutes: 6,
        energy_milli: 2_400,
        rate_yen_per_kwh: 50,
        expected: BillingResult {
            billed_energy_milli: 400,
            amount_yen: 20,
        },
    },
    StopCase {
        name: "scenario2_four_minutes",
        duration_minutes: 4,
        energy_milli: 1_000,
        rate_yen_per_kwh: 80,
        expected: BillingResult {
            billed_energy_milli: 0,
            amount_yen: 0,
        },
    },
    StopCase {
        name: "scenario3_five_minutes",
        duration_minutes: 5,
        energy_milli: 3_000,
        rate_yen_per_kwh: 100,
        expected: BillingResult {
            billed_energy_milli: 0,
            amount_yen: 0,
        },
    },
    StopCase {
        name: "scenario4_twenty_minutes",
        duration_minutes: 20,
        energy_milli: 6_000,
        rate_yen_per_kwh: 40,
        expected: BillingResult {
            billed_energy_milli: 4_500,
            amount_yen: 180,
        },
    },
    StopCase {
        name: "scenario7_exactly_five_minutes",
        duration_minutes: 5,
        energy_milli: 2_000,
        rate_yen_per_kwh: 60,
        expected: BillingResult {
            billed_energy_milli: 0,
            amount_yen: 0,
        },
    },
    StopCase {
        name: "scenario8_zero_energy",
        duration_minutes: 12,
        energy_milli: 0,
        rate_yen_per_kwh: 55,
        expected: BillingResult {
            billed_energy_milli: 0,
            amount_yen: 0,
        },
    },
];

#[test]
fn scenarios_1_to_4_and_7_8_match() {
    for case in STOP_CASES {
        assert_stop_case::<ModelASession, _>("model-a", case);
        assert_stop_case::<ModelBSession, _>("model-b", case);
    }
}

#[test]
fn scenario5_progressive_billing_is_monotonic() {
    let snapshots = [
        (
            3,
            1_200,
            BillingResult {
                billed_energy_milli: 0,
                amount_yen: 0,
            },
        ),
        (
            6,
            2_400,
            BillingResult {
                billed_energy_milli: 400,
                amount_yen: 20,
            },
        ),
        (
            10,
            4_000,
            BillingResult {
                billed_energy_milli: 2_000,
                amount_yen: 100,
            },
        ),
    ];

    assert_snapshots::<ModelASession, _>("model-a", 50, &snapshots);
    assert_snapshots::<ModelBSession, _>("model-b", 50, &snapshots);
}

#[test]
fn scenario6_rejects_billing_after_stop() {
    assert_rejects_after_stop::<ModelASession, _>("model-a");
    assert_rejects_after_stop::<ModelBSession, _>("model-b");
}

#[test]
fn scenario9_negative_energy_is_rejected() {
    assert_negative_energy_rejected::<ModelASession, _>("model-a");
    assert_negative_energy_rejected::<ModelBSession, _>("model-b");
}

#[test]
fn scenario10_invalid_timeline_is_rejected() {
    assert_invalid_timeline_rejected::<ModelASession, _>("model-a");
    assert_invalid_timeline_rejected::<ModelBSession, _>("model-b");
}

#[test]
fn scenario11_same_input_same_result() {
    assert_deterministic::<ModelASession, _>("model-a");
    assert_deterministic::<ModelBSession, _>("model-b");
}

#[test]
fn scenario12_rounding_is_floor_and_monotonic() {
    let short_case = StopCase {
        name: "short_usage",
        duration_minutes: 7,
        energy_milli: 1_250,
        rate_yen_per_kwh: 33,
        expected: BillingResult {
            billed_energy_milli: 357,
            amount_yen: 11,
        },
    };
    let long_case = StopCase {
        name: "long_usage",
        duration_minutes: 12,
        energy_milli: 2_142,
        rate_yen_per_kwh: 33,
        expected: BillingResult {
            billed_energy_milli: 1_249,
            amount_yen: 41,
        },
    };

    let short_a = assert_stop_case::<ModelASession, _>("model-a", &short_case);
    let long_a = assert_stop_case::<ModelASession, _>("model-a", &long_case);
    assert!(long_a.amount_yen >= short_a.amount_yen);

    let short_b = assert_stop_case::<ModelBSession, _>("model-b", &short_case);
    let long_b = assert_stop_case::<ModelBSession, _>("model-b", &long_case);
    assert!(long_b.amount_yen >= short_b.amount_yen);
}

fn assert_stop_case<S, E>(model_name: &str, case: &StopCase) -> BillingResult
where
    S: BillingSession<Error = E>,
    S::ClosedSession: ClosedBillingSession<Error = E>,
    E: std::fmt::Display,
{
    let session = S::start(BASE_START_MS, case.rate_yen_per_kwh)
        .unwrap_or_else(|err| panic!("{} start failed for {}: {}", model_name, case.name, err));
    let (result, _) = session
        .stop(end_timestamp(case.duration_minutes), case.energy_milli)
        .unwrap_or_else(|err| panic!("{} stop failed for {}: {}", model_name, case.name, err));
    assert_eq!(
        result.billed_energy_milli, case.expected.billed_energy_milli,
        "{} billed energy mismatch for {}",
        model_name, case.name
    );
    assert_eq!(
        result.amount_yen, case.expected.amount_yen,
        "{} amount mismatch for {}",
        model_name, case.name
    );
    result
}

fn assert_snapshots<S, E>(model_name: &str, rate: u32, snapshots: &[(i64, i64, BillingResult)])
where
    S: BillingSession<Error = E>,
    S::ClosedSession: ClosedBillingSession<Error = E>,
    E: std::fmt::Display,
{
    let session = S::start(BASE_START_MS, rate)
        .unwrap_or_else(|err| panic!("{} start failed: {}", model_name, err));

    let mut previous_amount = 0;
    for (minute, energy, expected) in snapshots {
        let result = session
            .bill_snapshot(end_timestamp(*minute), *energy)
            .unwrap_or_else(|err| panic!("{} snapshot failed at {}: {}", model_name, minute, err));
        assert_eq!(
            result.billed_energy_milli, expected.billed_energy_milli,
            "{} billed energy mismatch at minute {}",
            model_name, minute
        );
        assert_eq!(
            result.amount_yen, expected.amount_yen,
            "{} amount mismatch at minute {}",
            model_name, minute
        );
        assert!(result.amount_yen >= previous_amount);
        previous_amount = result.amount_yen;
    }
}

fn assert_rejects_after_stop<S, E>(model_name: &str)
where
    S: BillingSession<Error = E>,
    S::ClosedSession: ClosedBillingSession<Error = E>,
    E: std::fmt::Display,
{
    let session = S::start(BASE_START_MS, 50)
        .unwrap_or_else(|err| panic!("{} start failed: {}", model_name, err));
    let (_, closed) = session
        .stop(end_timestamp(6), 2_400)
        .unwrap_or_else(|err| panic!("{} stop failed: {}", model_name, err));

    let result = closed.bill_after_stop(end_timestamp(7), 2_800);
    assert!(
        result.is_err(),
        "{} should reject billing after stop",
        model_name
    );
}

fn assert_negative_energy_rejected<S, E>(model_name: &str)
where
    S: BillingSession<Error = E>,
    S::ClosedSession: ClosedBillingSession<Error = E>,
    E: std::fmt::Display,
{
    let session = S::start(BASE_START_MS, 45)
        .unwrap_or_else(|err| panic!("{} start failed: {}", model_name, err));
    let snapshot = session.bill_snapshot(end_timestamp(4), -500);
    assert!(
        snapshot.is_err(),
        "{} should reject negative energy snapshot",
        model_name
    );

    let stop_result = session.stop(end_timestamp(6), -500);
    assert!(
        stop_result.is_err(),
        "{} should reject negative energy stop",
        model_name
    );
}

fn assert_invalid_timeline_rejected<S, E>(model_name: &str)
where
    S: BillingSession<Error = E>,
    S::ClosedSession: ClosedBillingSession<Error = E>,
    E: std::fmt::Display,
{
    let session = S::start(BASE_START_MS, 60)
        .unwrap_or_else(|err| panic!("{} start failed: {}", model_name, err));
    let snapshot = session.bill_snapshot(BASE_START_MS - MINUTE_MS, 1_000);
    assert!(
        snapshot.is_err(),
        "{} should reject reversed timeline snapshot",
        model_name
    );

    let stop_result = session.stop(BASE_START_MS - MINUTE_MS, 1_000);
    assert!(
        stop_result.is_err(),
        "{} should reject reversed timeline stop",
        model_name
    );
}

fn assert_deterministic<S, E>(model_name: &str)
where
    S: BillingSession<Error = E>,
    S::ClosedSession: ClosedBillingSession<Error = E>,
    E: std::fmt::Display,
{
    let case = StopCase {
        name: "determinism",
        duration_minutes: 9,
        energy_milli: 3_600,
        rate_yen_per_kwh: 70,
        expected: BillingResult {
            billed_energy_milli: 1_600,
            amount_yen: 112,
        },
    };

    let first = assert_stop_case::<S, E>(model_name, &case);
    let second = assert_stop_case::<S, E>(model_name, &case);
    assert_eq!(first, second, "{} must be deterministic", model_name);
}

fn end_timestamp(duration_minutes: i64) -> i64 {
    BASE_START_MS + duration_minutes * MINUTE_MS
}
