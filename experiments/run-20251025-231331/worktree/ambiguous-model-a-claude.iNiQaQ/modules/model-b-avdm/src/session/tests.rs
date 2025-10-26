use std::num::NonZeroU32;

use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::{KwhMilli, RateYenPerKwh, Session, SessionId, SessionValueError};

fn create_test_session() -> (Session, OffsetDateTime) {
  let session_id = SessionId::new(Uuid::nil());
  let started_at = OffsetDateTime::now_utc();
  let rate = RateYenPerKwh::new(NonZeroU32::new(30).unwrap()); // 30円/kWh
  let session = Session::new_active(session_id, started_at, rate);
  (session, started_at)
}

#[test]
fn test_new_active_creates_session() {
  let session_id = SessionId::new(Uuid::nil());
  let started_at = OffsetDateTime::now_utc();
  let rate = RateYenPerKwh::new(NonZeroU32::new(30).unwrap());

  let session = Session::new_active(session_id, started_at, rate);

  assert_eq!(session.identity(), session_id);
  assert!(session.statement().is_none());
}

// ========================================
// 日常的な利用パターン
// ========================================

#[test]
fn test_normal_session_10_minutes() {
  let (session, started_at) = create_test_session();

  // 10分後、10 kWh消費
  let ended_at = started_at + Duration::minutes(10);
  let total_energy = KwhMilli::try_new(10_000).unwrap(); // 10 kWh = 10,000 milli-kWh

  let closed_session = session.stop(ended_at, total_energy).unwrap();

  let bill = closed_session.statement().unwrap();
  // 無料時間5分を差し引くと、課金対象は5分 = 50%
  // 課金対象エネルギー: 10 kWh * 0.5 = 5 kWh = 5,000 milli-kWh
  assert_eq!(u64::from(bill.billable_energy()), 5_000);
  // 料金: 5 kWh * 30円/kWh = 150円
  assert_eq!(u64::from(bill.amount_due()), 150);
}

#[test]
fn test_zero_energy_session() {
  let (session, started_at) = create_test_session();

  let ended_at = started_at + Duration::minutes(10);
  let total_energy = KwhMilli::zero();

  let closed_session = session.stop(ended_at, total_energy).unwrap();

  let bill = closed_session.statement().unwrap();
  assert_eq!(u64::from(bill.billable_energy()), 0);
  assert_eq!(u64::from(bill.amount_due()), 0);
}

#[test]
fn test_long_session_60_minutes() {
  let (session, started_at) = create_test_session();

  // 60分後、100 kWh消費
  let ended_at = started_at + Duration::minutes(60);
  let total_energy = KwhMilli::try_new(100_000).unwrap(); // 100 kWh

  let closed_session = session.stop(ended_at, total_energy).unwrap();

  let bill = closed_session.statement().unwrap();
  // 無料時間5分を差し引くと、課金対象は55分 ≈ 91.67%
  // 課金対象エネルギー: 100 kWh * (55/60) = 91.666... kWh -> 床で91,666 milli-kWh
  assert_eq!(u64::from(bill.billable_energy()), 91_666);
  // 料金: 91.666 kWh * 30円/kWh = 2,749.98円 -> 床で2,749円
  assert_eq!(u64::from(bill.amount_due()), 2_749);
}

// ========================================
// 境界条件
// ========================================

#[test]
fn test_exactly_free_period_5_minutes() {
  let (session, started_at) = create_test_session();

  // ちょうど5分後
  let ended_at = started_at + Duration::minutes(5);
  let total_energy = KwhMilli::try_new(5_000).unwrap(); // 5 kWh

  let closed_session = session.stop(ended_at, total_energy).unwrap();

  let bill = closed_session.statement().unwrap();
  // 全て無料時間内 -> 課金対象0
  assert_eq!(u64::from(bill.billable_energy()), 0);
  assert_eq!(u64::from(bill.amount_due()), 0);
}

#[test]
fn test_just_after_free_period() {
  let (session, started_at) = create_test_session();

  // 5分1秒後（少しだけ無料時間を超える）
  let ended_at = started_at + Duration::minutes(5) + Duration::seconds(1);
  let total_energy = KwhMilli::try_new(6_000).unwrap(); // 6 kWh

  let closed_session = session.stop(ended_at, total_energy).unwrap();

  let bill = closed_session.statement().unwrap();
  // 課金対象時間: 1秒 / 301秒 ≈ 0.33%
  // 課金対象エネルギー: 6 kWh * (1/301) ≈ 19.9 milli-kWh -> 床で19 milli-kWh
  assert!(u64::from(bill.billable_energy()) > 0);
  assert!(u64::from(bill.billable_energy()) < 100); // 微小な課金
}

// ========================================
// 異常系（不正な操作）
// ========================================

#[test]
fn test_stop_already_stopped_session() {
  let (session, started_at) = create_test_session();

  let ended_at = started_at + Duration::minutes(10);
  let total_energy = KwhMilli::try_new(10_000).unwrap();

  let closed_session = session.stop(ended_at, total_energy).unwrap();

  // 既に停止済みのセッションを再度停止しようとする
  let ended_at_2 = ended_at + Duration::minutes(5);
  let total_energy_2 = KwhMilli::try_new(15_000).unwrap();

  let result = closed_session.stop(ended_at_2, total_energy_2);
  assert!(matches!(result, Err(SessionValueError::AlreadyClosed { .. })));
}

#[test]
fn test_bill_snapshot_on_closed_session() {
  let (session, started_at) = create_test_session();

  let ended_at = started_at + Duration::minutes(10);
  let total_energy = KwhMilli::try_new(10_000).unwrap();

  let closed_session = session.stop(ended_at, total_energy).unwrap();

  // 停止済みセッションでスナップショットを取ろうとする
  let ended_at_2 = ended_at + Duration::minutes(5);
  let total_energy_2 = KwhMilli::try_new(15_000).unwrap();

  let result = closed_session.bill_snapshot(ended_at_2, total_energy_2);
  assert!(matches!(result, Err(SessionValueError::AlreadyClosed { .. })));
}

#[test]
fn test_bill_after_stop_on_closed_session() {
  let (session, started_at) = create_test_session();

  let ended_at = started_at + Duration::minutes(10);
  let total_energy = KwhMilli::try_new(10_000).unwrap();

  let closed_session = session.stop(ended_at, total_energy).unwrap();

  // 停止後に追加課金しようとする
  let ended_at_2 = ended_at + Duration::minutes(5);
  let total_energy_2 = KwhMilli::try_new(15_000).unwrap();

  let result = closed_session.bill_after_stop(ended_at_2, total_energy_2);
  assert!(matches!(result, Err(SessionValueError::AlreadyClosed { .. })));
}

#[test]
fn test_negative_energy() {
  // KwhMilli::try_from_i64 で負のエネルギーを拒否
  let result = KwhMilli::try_from_i64(-1000);
  assert!(matches!(result, Err(SessionValueError::NegativeEnergy { .. })));
}

#[test]
fn test_time_reversal() {
  let (session, started_at) = create_test_session();

  // 終了時刻が開始時刻より前
  let ended_at = started_at - Duration::minutes(10);
  let total_energy = KwhMilli::try_new(10_000).unwrap();

  let result = session.stop(ended_at, total_energy);
  assert!(matches!(result, Err(SessionValueError::InvalidTimeline { .. })));
}

#[test]
fn test_excessive_energy() {
  // MAX_KWH_MILLI = 1,000,000 を超えるエネルギー
  let total_energy_result = KwhMilli::try_new(2_000_000);
  assert!(matches!(total_energy_result, Err(SessionValueError::EnergyOutOfRange { .. })));
}

#[test]
fn test_excessive_amount() {
  // 単価を高く設定して、金額上限を超えるケース
  let session_id = SessionId::new(Uuid::nil());
  let started_at = OffsetDateTime::now_utc();
  // 非常に高い単価: 10,000円/kWh
  let rate = RateYenPerKwh::new(NonZeroU32::new(10_000).unwrap());
  let session = Session::new_active(session_id, started_at, rate);

  let ended_at = started_at + Duration::minutes(60);
  // 100 kWh消費
  let total_energy = KwhMilli::try_new(100_000).unwrap();

  // 課金対象: 約91.666 kWh * 10,000円/kWh ≈ 916,660円
  // MAX_YEN = 1,000,000 なので範囲内で通るはず
  let closed_session = session.stop(ended_at, total_energy);
  assert!(closed_session.is_ok());

  // さらに高い単価で上限を超えるケース
  let rate_high = RateYenPerKwh::new(NonZeroU32::new(50_000).unwrap());
  let session_high = Session::new_active(session_id, started_at, rate_high);

  // 100 kWh * 50,000円/kWh * (55/60) ≈ 4,583,300円 -> MAX_YEN超過
  let result_high = session_high.stop(ended_at, total_energy);
  assert!(matches!(result_high, Err(SessionValueError::AmountOutOfRange { .. })));
}

// ========================================
// bill_snapshot のテスト
// ========================================

#[test]
fn test_bill_snapshot_during_active() {
  let (session, started_at) = create_test_session();

  // 途中経過確認: 10分後、10 kWh消費
  let ended_at = started_at + Duration::minutes(10);
  let total_energy = KwhMilli::try_new(10_000).unwrap();

  let bill = session.bill_snapshot(ended_at, total_energy).unwrap();

  // 課金対象は50%
  assert_eq!(u64::from(bill.billable_energy()), 5_000);
  assert_eq!(u64::from(bill.amount_due()), 150); // 5 kWh * 30円/kWh

  // セッションはまだActive
  assert!(session.statement().is_none());
}

#[test]
fn test_bill_snapshot_multiple_times() {
  let (session, started_at) = create_test_session();

  // 1回目: 5分後（無料時間ぴったり）
  let ended_at_1 = started_at + Duration::minutes(5);
  let total_energy_1 = KwhMilli::try_new(5_000).unwrap();
  let bill_1 = session.bill_snapshot(ended_at_1, total_energy_1).unwrap();
  assert_eq!(u64::from(bill_1.amount_due()), 0);

  // 2回目: 10分後
  let ended_at_2 = started_at + Duration::minutes(10);
  let total_energy_2 = KwhMilli::try_new(10_000).unwrap();
  let bill_2 = session.bill_snapshot(ended_at_2, total_energy_2).unwrap();
  assert_eq!(u64::from(bill_2.billable_energy()), 5_000);
  assert_eq!(u64::from(bill_2.amount_due()), 150);

  // セッションはまだActive
  assert!(session.statement().is_none());
}
