use spec_tests::{BillingResult, BillingSession, ClosedBillingSession};

pub const BASE_START_MS: i64 = 0;
pub const MINUTE_MS: i64 = 60 * 1_000;
pub const MAX_KWH_MILLI: i64 = 1_000_000;

pub struct StopCase {
  pub name: &'static str,
  pub duration_minutes: i64,
  pub energy_milli: i64,
  pub rate_yen_per_kwh: u32,
  pub expected: BillingResult,
}

pub const STOP_CASES: &[StopCase] = &[
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
  StopCase {
    name: "scenario13_max_amount_boundary",
    duration_minutes: 10,
    energy_milli: MAX_KWH_MILLI,
    rate_yen_per_kwh: 2_000,
    expected: BillingResult {
      billed_energy_milli: 500_000,
      amount_yen: 1_000_000,
    },
  },
];

pub fn assert_amount_over_limit_rejected<S, E>(model_name: &str)
where
  S: BillingSession<Error = E>,
  S::ClosedSession: ClosedBillingSession<Error = E>,
  E: std::fmt::Display,
{
  let session = S::start(BASE_START_MS, 2_001).unwrap_or_else(|err| {
    panic!(
      "{} start failed for over-limit scenario: {}",
      model_name, err
    )
  });
  let snapshot = session.bill_snapshot(end_timestamp(10), 1_000_000);
  assert!(
    snapshot.is_err(),
    "{} should reject snapshot when amount exceeds limit",
    model_name
  );
  let stop_result = session.stop(end_timestamp(10), 1_000_000);
  assert!(
    stop_result.is_err(),
    "{} should reject stop when amount exceeds limit",
    model_name
  );
}

pub fn assert_energy_over_limit_rejected<S, E>(model_name: &str)
where
  S: BillingSession<Error = E>,
  S::ClosedSession: ClosedBillingSession<Error = E>,
  E: std::fmt::Display,
{
  let over_limit = MAX_KWH_MILLI + 1;
  let session = S::start(BASE_START_MS, 60)
    .unwrap_or_else(|err| panic!("{} start failed for energy over-limit: {}", model_name, err));
  let snapshot = session.bill_snapshot(end_timestamp(10), over_limit);
  assert!(
    snapshot.is_err(),
    "{} should reject snapshot when energy exceeds limit",
    model_name
  );
  let stop_result = session.stop(end_timestamp(10), over_limit);
  assert!(
    stop_result.is_err(),
    "{} should reject stop when energy exceeds limit",
    model_name
  );
}

pub fn assert_stop_case<S, E>(model_name: &str, case: &StopCase) -> BillingResult
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

pub fn assert_snapshots<S, E>(model_name: &str, rate: u32, snapshots: &[(i64, i64, BillingResult)])
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

pub fn assert_rejects_after_stop<S, E>(model_name: &str)
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

pub fn assert_negative_energy_rejected<S, E>(model_name: &str)
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

pub fn assert_invalid_timeline_rejected<S, E>(model_name: &str)
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

pub fn assert_deterministic<S, E>(model_name: &str)
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

pub fn end_timestamp(duration_minutes: i64) -> i64 {
  BASE_START_MS + duration_minutes * MINUTE_MS
}
