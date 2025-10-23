mod common;

use common::{
  STOP_CASES, StopCase, assert_amount_over_limit_rejected, assert_deterministic, assert_energy_over_limit_rejected,
  assert_invalid_timeline_rejected, assert_negative_energy_rejected, assert_rejects_after_stop, assert_snapshots,
  assert_stop_case,
};
use spec_tests::{BillingResult, adapters::ModelASession};

/// 開始から5分無料・ゼロエネルギー・長時間利用・最大課金額境界など代表的な停止ケースで、
/// 課金結果が期待通りになることを確認する。
#[test]
fn stop_scenarios_match_expected() {
  for case in STOP_CASES {
    assert_stop_case::<ModelASession, _>("model-a", case);
  }
}

/// 進行中に複数回スナップショット課金を行っても、無料5分以降は金額が単調増加し、
/// 途中計算に矛盾が生じないことを確認する。
#[test]
fn scenario5_progressive_billing_is_monotonic() {
  let snapshots = [
    (3, 1_200, BillingResult { billed_energy_milli: 0, amount_yen: 0 }),
    (6, 2_400, BillingResult { billed_energy_milli: 400, amount_yen: 20 }),
    (10, 4_000, BillingResult { billed_energy_milli: 2_000, amount_yen: 100 }),
  ];

  assert_snapshots::<ModelASession, _>("model-a", 50, &snapshots);
}

/// セッションを停止した後に課金しようとすると、「停止済み」として拒否されることを保証する。
#[test]
fn scenario6_rejects_billing_after_stop() {
  assert_rejects_after_stop::<ModelASession, _>("model-a");
}

/// エネルギーに負値を与えた場合、スナップショット・停止ともにエラー扱いとなることを検証する。
#[test]
fn scenario9_negative_energy_is_rejected() {
  assert_negative_energy_rejected::<ModelASession, _>("model-a");
}

/// 停止時刻が開始時刻以前であるような逆転タイムラインを入力すると、
/// 課金処理が不正入力として拒否されることを確認する。
#[test]
fn scenario10_invalid_timeline_is_rejected() {
  assert_invalid_timeline_rejected::<ModelASession, _>("model-a");
}

/// 同じ単価・時間・エネルギーを繰り返し渡しても結果が変わらない決定性を検証する。
#[test]
fn scenario11_same_input_same_result() {
  assert_deterministic::<ModelASession, _>("model-a");
}

/// 端数が発生するケースで常に切り捨て計算が行われ、利用時間が長い方が安くならないことを確認する。
#[test]
fn scenario12_rounding_is_floor_and_monotonic() {
  let short_case = StopCase {
    name:             "short_usage",
    duration_minutes: 7,
    energy_milli:     1_250,
    rate_yen_per_kwh: 33,
    expected:         BillingResult { billed_energy_milli: 357, amount_yen: 11 },
  };
  let long_case = StopCase {
    name:             "long_usage",
    duration_minutes: 12,
    energy_milli:     2_142,
    rate_yen_per_kwh: 33,
    expected:         BillingResult { billed_energy_milli: 1_249, amount_yen: 41 },
  };

  let short = assert_stop_case::<ModelASession, _>("model-a", &short_case);
  let long = assert_stop_case::<ModelASession, _>("model-a", &long_case);
  assert!(long.amount_yen >= short.amount_yen);
}

/// 単価とエネルギーの組み合わせで請求額が100万円を超える場合、
/// 課金と停止のどちらでも拒否されることを検証する。
#[test]
fn scenario14_amount_over_limit_is_rejected() {
  assert_amount_over_limit_rejected::<ModelASession, _>("model-a");
}

/// 総エネルギーが1,000,000ミリkWhの上限を超えると、
/// スナップショットと停止の両方でエラーになることを検証する。
#[test]
fn scenario15_energy_over_limit_is_rejected() {
  assert_energy_over_limit_rejected::<ModelASession, _>("model-a");
}
