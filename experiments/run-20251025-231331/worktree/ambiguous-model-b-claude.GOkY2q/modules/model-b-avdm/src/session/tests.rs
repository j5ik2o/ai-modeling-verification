use std::num::NonZeroU32;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::{KwhMilli, RateYenPerKwh, Session, SessionId, SessionValueError};

// =========================================================================
// ヘルパー関数
// =========================================================================

/// テスト用のセッションIDを生成する。
fn test_session_id() -> SessionId {
  // テスト用に固定のUUIDを使用
  SessionId::new(Uuid::from_u128(0x0123456789abcdef0123456789abcdef))
}

/// テスト用の開始時刻を生成する。
fn test_start_time() -> OffsetDateTime {
  OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap()
}

/// テスト用の単価を生成する（30円/kWh）。
fn test_rate() -> RateYenPerKwh {
  RateYenPerKwh::new(NonZeroU32::new(30).unwrap())
}

// =========================================================================
// 日常的な利用パターンのテスト
// =========================================================================

#[test]
fn test_normal_charging_session() {
  // 10分間の充電セッション（5分無料 + 5分有料）、10 kWhのエネルギー
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(10);
  let rate = test_rate();
  let energy = KwhMilli::try_new(10_000).unwrap(); // 10 kWh

  let session = Session::new_active(id, started_at, rate);
  let closed = session.stop(ended_at, energy).unwrap();

  let bill = closed.statement().unwrap();

  // 10分中5分が課金対象なので、5 kWhが課金対象
  // 5 kWh * 30円/kWh = 150円
  assert_eq!(u64::from(bill.billable_energy()), 5_000);
  assert_eq!(u64::from(bill.amount_due()), 150);
}

#[test]
fn test_zero_energy_session() {
  // エネルギーがゼロのケース
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(10);
  let rate = test_rate();
  let energy = KwhMilli::zero();

  let session = Session::new_active(id, started_at, rate);
  let closed = session.stop(ended_at, energy).unwrap();

  let bill = closed.statement().unwrap();

  assert_eq!(u64::from(bill.billable_energy()), 0);
  assert_eq!(u64::from(bill.amount_due()), 0);
}

#[test]
fn test_long_session() {
  // 60分間の充電セッション（5分無料 + 55分有料）、100 kWhのエネルギー
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(60);
  let rate = test_rate();
  let energy = KwhMilli::try_new(100_000).unwrap(); // 100 kWh

  let session = Session::new_active(id, started_at, rate);
  let closed = session.stop(ended_at, energy).unwrap();

  let bill = closed.statement().unwrap();

  // 60分中55分が課金対象なので、約 91.666... kWh が課金対象（床関数で 91666 milli-kWh）
  // 計算: 100000 * 55 / 60 = 91666.666... → 91666 (床)
  // 91.666 kWh * 30円/kWh = 2749.98円 → 2749円（床）
  assert_eq!(u64::from(bill.billable_energy()), 91_666);
  assert_eq!(u64::from(bill.amount_due()), 2_749);
}

// =========================================================================
// 境界条件のテスト
// =========================================================================

#[test]
fn test_exactly_grace_period() {
  // 無料時間ちょうど（5分）のケース
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(5);
  let rate = test_rate();
  let energy = KwhMilli::try_new(10_000).unwrap(); // 10 kWh

  let session = Session::new_active(id, started_at, rate);
  let closed = session.stop(ended_at, energy).unwrap();

  let bill = closed.statement().unwrap();

  // 5分ちょうどなので、課金対象エネルギーと金額はゼロ
  assert_eq!(u64::from(bill.billable_energy()), 0);
  assert_eq!(u64::from(bill.amount_due()), 0);
}

#[test]
fn test_slightly_over_grace_period() {
  // 無料時間を1分超えるケース（6分）
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(6);
  let rate = test_rate();
  let energy = KwhMilli::try_new(12_000).unwrap(); // 12 kWh

  let session = Session::new_active(id, started_at, rate);
  let closed = session.stop(ended_at, energy).unwrap();

  let bill = closed.statement().unwrap();

  // 6分中1分が課金対象なので、2 kWh が課金対象
  // 12 kWh * 1/6 = 2 kWh
  // 2 kWh * 30円/kWh = 60円
  assert_eq!(u64::from(bill.billable_energy()), 2_000);
  assert_eq!(u64::from(bill.amount_due()), 60);
}

#[test]
fn test_within_grace_period() {
  // 無料時間内（3分）のケース
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(3);
  let rate = test_rate();
  let energy = KwhMilli::try_new(5_000).unwrap(); // 5 kWh

  let session = Session::new_active(id, started_at, rate);
  let closed = session.stop(ended_at, energy).unwrap();

  let bill = closed.statement().unwrap();

  // 無料時間内なので、課金対象エネルギーと金額はゼロ
  assert_eq!(u64::from(bill.billable_energy()), 0);
  assert_eq!(u64::from(bill.amount_due()), 0);
}

// =========================================================================
// 異常系テスト（不正操作）
// =========================================================================

#[test]
fn test_billing_after_stop_fails() {
  // 停止後に再度課金しようとする
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(10);
  let rate = test_rate();
  let energy = KwhMilli::try_new(10_000).unwrap();

  let session = Session::new_active(id, started_at, rate);
  let closed = session.stop(ended_at, energy).unwrap();

  // 停止済みセッションに対して bill_after_stop を呼ぶとエラー
  let result = closed.bill_after_stop(ended_at, energy);
  assert!(matches!(result, Err(SessionValueError::AlreadyClosed { .. })));
}

#[test]
fn test_double_stop_fails() {
  // 停止済みセッションをさらに停止しようとする
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(10);
  let rate = test_rate();
  let energy = KwhMilli::try_new(10_000).unwrap();

  let session = Session::new_active(id, started_at, rate);
  let closed = session.stop(ended_at, energy).unwrap();

  // 再度 stop を呼ぶとエラー
  let result = closed.stop(ended_at, energy);
  assert!(matches!(result, Err(SessionValueError::AlreadyClosed { .. })));
}

#[test]
fn test_negative_energy_fails() {
  // 負のエネルギー値
  let result = KwhMilli::try_from_i64(-100);
  assert!(matches!(result, Err(SessionValueError::NegativeEnergy { .. })));
}

#[test]
fn test_time_reversal_fails() {
  // 終了時刻が開始時刻より前
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at - Duration::minutes(10);
  let rate = test_rate();
  let energy = KwhMilli::try_new(10_000).unwrap();

  let session = Session::new_active(id, started_at, rate);
  let result = session.stop(ended_at, energy);

  assert!(matches!(result, Err(SessionValueError::InvalidTimeline { .. })));
}

#[test]
fn test_excessive_energy_fails() {
  // 桁外れのエネルギー量（上限を超える）
  let result = KwhMilli::try_new(2_000_000_000); // 上限 1,000,000 を超える
  assert!(matches!(result, Err(SessionValueError::EnergyOutOfRange { .. })));
}

#[test]
fn test_excessive_amount_calculation() {
  // 桁外れの金額になるケース
  // 上限ギリギリのエネルギー（1,000 kWh = 1,000,000 milli-kWh）と高い単価でテスト
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(10);
  let energy = KwhMilli::try_new(1_000_000).unwrap(); // 上限

  // 非常に高い単価（10,000円/kWh）を設定
  let high_rate = RateYenPerKwh::new(NonZeroU32::new(10_000).unwrap());

  let session = Session::new_active(id, started_at, high_rate);

  // 全エネルギーが課金対象なら 1,000 kWh * 10,000円/kWh = 10,000,000円となり上限超過
  // しかし5分無料なので 500 kWh * 10,000円/kWh = 5,000,000円となりこれも上限超過
  let result = session.stop(ended_at, energy);

  // 上限超過エラーになることを確認
  assert!(matches!(result, Err(SessionValueError::AmountOutOfRange { .. })));
}

// =========================================================================
// bill_snapshot のテスト
// =========================================================================

#[test]
fn test_bill_snapshot_active_session() {
  // アクティブセッションの途中でスナップショットを取得
  let id = test_session_id();
  let started_at = test_start_time();
  let rate = test_rate();
  let session = Session::new_active(id, started_at, rate);

  // 10分経過時点でのスナップショット
  let snapshot_time = started_at + Duration::minutes(10);
  let energy = KwhMilli::try_new(10_000).unwrap();

  let bill = session.bill_snapshot(snapshot_time, energy).unwrap();

  // 10分中5分が課金対象なので、5 kWh * 30円/kWh = 150円
  assert_eq!(u64::from(bill.billable_energy()), 5_000);
  assert_eq!(u64::from(bill.amount_due()), 150);
}

#[test]
fn test_bill_snapshot_closed_session_returns_final_bill() {
  // 停止済みセッションでスナップショットを取得すると確定請求を返す
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(10);
  let rate = test_rate();
  let energy = KwhMilli::try_new(10_000).unwrap();

  let session = Session::new_active(id, started_at, rate);
  let closed = session.stop(ended_at, energy).unwrap();

  // 異なる時刻とエネルギーでスナップショットを取っても、確定請求が返る
  let snapshot_time = started_at + Duration::minutes(20);
  let different_energy = KwhMilli::try_new(20_000).unwrap();

  let bill = closed.bill_snapshot(snapshot_time, different_energy).unwrap();

  // 確定請求と同じ内容が返る
  assert_eq!(u64::from(bill.billable_energy()), 5_000);
  assert_eq!(u64::from(bill.amount_due()), 150);
}

// =========================================================================
// identity と statement のテスト
// =========================================================================

#[test]
fn test_identity_returns_session_id() {
  let id = test_session_id();
  let started_at = test_start_time();
  let rate = test_rate();

  let session = Session::new_active(id, started_at, rate);
  assert_eq!(session.identity(), id);

  let ended_at = started_at + Duration::minutes(10);
  let energy = KwhMilli::try_new(10_000).unwrap();
  let closed = session.stop(ended_at, energy).unwrap();

  assert_eq!(closed.identity(), id);
}

#[test]
fn test_statement_none_for_active() {
  let id = test_session_id();
  let started_at = test_start_time();
  let rate = test_rate();

  let session = Session::new_active(id, started_at, rate);
  assert!(session.statement().is_none());
}

#[test]
fn test_statement_some_for_closed() {
  let id = test_session_id();
  let started_at = test_start_time();
  let ended_at = started_at + Duration::minutes(10);
  let rate = test_rate();
  let energy = KwhMilli::try_new(10_000).unwrap();

  let session = Session::new_active(id, started_at, rate);
  let closed = session.stop(ended_at, energy).unwrap();

  assert!(closed.statement().is_some());
}