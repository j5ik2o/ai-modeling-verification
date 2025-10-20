use super::*;
use crate::session::MAX_YEN;
use time::{Duration, OffsetDateTime};

fn ts(sec: i64) -> OffsetDateTime {
  OffsetDateTime::from_unix_timestamp(sec).expect("timestamp is valid")
}

#[test]
fn kwh_milli_validates_range() {
  assert!(KwhMilli::new(0).is_ok());
  assert!(KwhMilli::new(500_000).is_ok());
  let over = MAX_KWH_MILLI + 1;
  assert!(matches!(
      KwhMilli::new(over),
      Err(SessionValueError::EnergyOutOfRange { provided, max })
          if provided == over && max == MAX_KWH_MILLI
  ));
  assert!(matches!(
    KwhMilli::try_from_i64(-1),
    Err(SessionValueError::NegativeEnergy { provided: -1 })
  ));
  assert!(matches!(
      KwhMilli::try_from(over),
      Err(SessionValueError::EnergyOutOfRange { provided, max })
          if provided == over && max == MAX_KWH_MILLI
  ));
}

#[test]
fn money_yen_validates_range() {
  assert!(MoneyYen::new(0).is_ok());
  assert!(MoneyYen::new(999_999).is_ok());
  let over = MAX_YEN + 1;
  assert!(matches!(
      MoneyYen::new(over),
      Err(SessionValueError::AmountOutOfRange { provided, max })
          if provided == over && max == MAX_YEN
  ));
  assert!(matches!(
      MoneyYen::try_from_u128((MAX_YEN as u128) + 1),
      Err(SessionValueError::AmountOutOfRange { provided, max })
          if provided == over && max == MAX_YEN
  ));
}

#[test]
fn session_stop_transitions_to_closed() {
  let id = SessionId::new(uuid::Uuid::nil());
  let start = ts(0);
  let rate = RateYenPerKwh::new(10).expect("positive rate");
  let session = Session::new_active(id, start, rate);
  let end = ts(60);
  let energy = KwhMilli::new(1_000).expect("non-negative energy");

  let closed = session.clone().stop(end, energy).expect("stop succeeds");
  assert_eq!(closed.ended_at(), Some(end));
  assert_eq!(u64::from(closed.charged_amount().unwrap()), 0);
  assert_eq!(uuid::Uuid::from(closed.id()), uuid::Uuid::nil());

  let invalid = session.stop(start, energy);
  assert!(matches!(
      invalid,
      Err(SessionValueError::InvalidTimeline { started_at, ended_at })
          if started_at == start && ended_at == start
  ));
}

#[test]
fn bill_snapshot_matches_stop() {
  let id = SessionId::new(uuid::Uuid::nil());
  let rate = RateYenPerKwh::new(50).expect("positive rate");
  let start = ts(0);
  let session = Session::new_active(id, start, rate);
  let end = ts(6 * 60);
  let total_energy = KwhMilli::new(2_400).expect("energy");

  let (billed, amount) = session
    .bill_snapshot(end, total_energy)
    .expect("snapshot works");
  assert_eq!(u64::from(billed), 400);
  assert_eq!(u64::from(amount), 20);

  let closed = session.stop(end, total_energy).expect("stop succeeds");
  assert_eq!(u64::from(closed.billed_energy().unwrap()), 400);
  assert_eq!(u64::from(closed.charged_amount().unwrap()), 20);
}

#[test]
fn closed_session_rejects_additional_billing() {
  let id = SessionId::new(uuid::Uuid::nil());
  let rate = RateYenPerKwh::new(60).expect("positive rate");
  let start = ts(0);
  let end = ts(600);
  let energy = KwhMilli::new(2_000).expect("energy");
  let closed = Session::new_active(id, start, rate)
    .stop(end, energy)
    .expect("stop works");
  let err = closed
    .bill_after_stop(end + Duration::minutes(1), energy)
    .expect_err("billing after stop should fail");
  assert!(matches!(err, SessionValueError::AlreadyClosed { .. }));
}

#[test]
fn active_accessors_return_none() {
  let id = SessionId::new(uuid::Uuid::nil());
  let start = ts(0);
  let rate = RateYenPerKwh::new(40).expect("positive rate");
  let session = Session::new_active(id, start, rate);
  assert!(session.ended_at().is_none());
  assert!(session.total_energy().is_none());
  assert!(session.billed_energy().is_none());
  assert!(session.charged_amount().is_none());
}

#[test]
fn rate_rejects_zero_value() {
  assert!(RateYenPerKwh::new(0).is_err());
}

#[test]
fn money_rejects_amount_over_max() {
  assert!(MoneyYen::new(MAX_YEN + 1).is_err());
  assert!(MoneyYen::try_from_u128((MAX_YEN as u128) + 1).is_err());
}

#[test]
fn bill_snapshot_rejects_mismatched_closed_session() {
  let id = SessionId::new(uuid::Uuid::nil());
  let start = ts(0);
  let rate = RateYenPerKwh::new(50).expect("positive rate");
  let end = ts(6 * 60);
  let energy = KwhMilli::new(2_400).expect("energy");
  let closed = Session::new_active(id, start, rate)
    .stop(end, energy)
    .expect("stop works");

  let err = closed
    .bill_snapshot(end + Duration::minutes(1), energy)
    .expect_err("mismatched snapshot should fail");
  assert!(matches!(err, SessionValueError::AlreadyClosed { .. }));
}

#[test]
fn bill_after_stop_active_delegates_to_snapshot() {
  let id = SessionId::new(uuid::Uuid::nil());
  let start = ts(0);
  let rate = RateYenPerKwh::new(50).expect("positive rate");
  let session = Session::new_active(id, start, rate);
  let energy = KwhMilli::new(2_400).expect("energy");
  let end = ts(6 * 60);

  let expected = session.bill_snapshot(end, energy).expect("snapshot works");
  let result = session
    .bill_after_stop(end, energy)
    .expect("active bill_after_stop should succeed");
  assert_eq!(result.0, expected.0);
  assert_eq!(result.1, expected.1);
}
