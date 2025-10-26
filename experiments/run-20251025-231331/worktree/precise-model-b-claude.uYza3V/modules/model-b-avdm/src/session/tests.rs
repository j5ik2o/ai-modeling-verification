use time::OffsetDateTime;
use uuid::Uuid;

use super::*;

fn offset(minutes: i64) -> OffsetDateTime {
  OffsetDateTime::from_unix_timestamp(minutes * 60).unwrap()
}

#[test]
fn scenario1_six_minutes_charges_one_minute() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  let stopped = session
    .stop(offset(6), KwhMilli::try_new(2_400).unwrap())
    .unwrap();

  let bill = stopped.statement().unwrap();
  assert_eq!(u64::from(bill.billable_energy()), 400); // 0.4 kWh
  assert_eq!(u64::from(bill.amount_due()), 20); // 20円
}

#[test]
fn scenario2_four_minutes_is_free() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(80).unwrap(),
  );

  let stopped = session
    .stop(offset(4), KwhMilli::try_new(1_000).unwrap())
    .unwrap();

  let bill = stopped.statement().unwrap();
  assert_eq!(u64::from(bill.billable_energy()), 0);
  assert_eq!(u64::from(bill.amount_due()), 0);
}

#[test]
fn scenario3_exactly_five_minutes_is_free() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(100).unwrap(),
  );

  let stopped = session
    .stop(offset(5), KwhMilli::try_new(3_000).unwrap())
    .unwrap();

  let bill = stopped.statement().unwrap();
  assert_eq!(u64::from(bill.billable_energy()), 0);
  assert_eq!(u64::from(bill.amount_due()), 0);
}

#[test]
fn error_when_stop_already_closed() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  let stopped = session
    .stop(offset(6), KwhMilli::try_new(2_400).unwrap())
    .unwrap();

  // 再度stopしようとするとエラー
  let result = stopped.stop(offset(10), KwhMilli::try_new(4_000).unwrap());
  assert!(matches!(result, Err(SessionValueError::AlreadyClosed { .. })));
}

#[test]
fn error_when_bill_after_stop() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  let stopped = session
    .stop(offset(6), KwhMilli::try_new(2_400).unwrap())
    .unwrap();

  // 停止後に課金しようとするとエラー
  let result = stopped.bill_after_stop(offset(10), KwhMilli::try_new(4_000).unwrap());
  assert!(matches!(result, Err(SessionValueError::AlreadyClosed { .. })));
}

#[test]
fn error_when_end_before_start() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(10), // 開始: 10分
    RateYenPerKwh::try_new(50).unwrap(),
  );

  // 終了時刻が開始時刻より前
  let result = session.stop(offset(5), KwhMilli::try_new(1_000).unwrap());
  assert!(matches!(result, Err(SessionValueError::InvalidTimeline { .. })));
}

#[test]
fn test_bill_snapshot_active() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  // 6分経過時点のスナップショット
  let bill = session
    .bill_snapshot(offset(6), KwhMilli::try_new(2_400).unwrap())
    .unwrap();

  assert_eq!(u64::from(bill.billable_energy()), 400); // 0.4 kWh
  assert_eq!(u64::from(bill.amount_due()), 20); // 20円
}

#[test]
fn test_bill_snapshot_closed() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  let stopped = session
    .stop(offset(6), KwhMilli::try_new(2_400).unwrap())
    .unwrap();

  // 停止済みの場合は記録済みの請求書を返す
  let bill = stopped
    .bill_snapshot(offset(6), KwhMilli::try_new(2_400).unwrap())
    .unwrap();

  assert_eq!(u64::from(bill.billable_energy()), 400);
  assert_eq!(u64::from(bill.amount_due()), 20);
}

#[test]
fn test_identity() {
  let id = SessionId::new(Uuid::nil());
  let session = Session::new_active(id, offset(0), RateYenPerKwh::try_new(50).unwrap());

  assert_eq!(session.identity(), id);

  let stopped = session
    .stop(offset(6), KwhMilli::try_new(2_400).unwrap())
    .unwrap();

  assert_eq!(stopped.identity(), id);
}

#[test]
fn test_statement_active_returns_none() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  assert!(session.statement().is_none());
}

#[test]
fn test_statement_closed_returns_some() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  let stopped = session
    .stop(offset(6), KwhMilli::try_new(2_400).unwrap())
    .unwrap();

  assert!(stopped.statement().is_some());
}