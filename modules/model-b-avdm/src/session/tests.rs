#![allow(clippy::unwrap_used)]

use time::{Duration, OffsetDateTime};

use super::*;

fn sample_session(rate: u32) -> (Session, OffsetDateTime) {
  let id = SessionId::new(uuid::Uuid::nil());
  let started_at = OffsetDateTime::from_unix_timestamp(0).expect("fixed timestamp");
  let rate = RateYenPerKwh::try_new(rate).expect("positive rate");
  (Session::new_active(id, started_at, rate), started_at)
}

fn energy(value: u64) -> KwhMilli {
  KwhMilli::try_new(value).expect("energy within bounds")
}

fn yen(value: u64) -> MoneyYen {
  MoneyYen::try_new(value).expect("amount within bounds")
}

#[test]
fn calculates_bill_with_free_grace_period() {
  let (session, started_at) = sample_session(40);
  let total_energy = energy(30_000);
  let ended_at = started_at + Duration::minutes(30);

  let (billed_energy, charged_amount) = session.bill_snapshot(ended_at, total_energy).expect("snapshot succeeds");

  assert_eq!(billed_energy, energy(25_000));
  assert_eq!(charged_amount, yen(1_000));

  let closed = session.stop(ended_at, total_energy).expect("stop succeeds");
  match closed {
    | Session::Closed { billed_energy, charged_amount, .. } => {
      assert_eq!(billed_energy, energy(25_000));
      assert_eq!(charged_amount, yen(1_000));
    },
    | _ => panic!("expected closed session"),
  }
}

#[test]
fn zero_energy_session_is_free() {
  let (session, started_at) = sample_session(55);
  let total_energy = energy(0);
  let ended_at = started_at + Duration::minutes(20);

  let (billed_energy, charged_amount) = session.bill_snapshot(ended_at, total_energy).expect("snapshot succeeds");

  assert_eq!(billed_energy, KwhMilli::zero());
  assert_eq!(charged_amount, yen(0));

  let closed = session.stop(ended_at, total_energy).expect("stop succeeds");
  assert_eq!(closed.billed_energy(), Some(KwhMilli::zero()));
  assert_eq!(closed.charged_amount(), Some(yen(0)));
}

#[test]
fn exact_free_window_results_in_no_charge() {
  let (session, started_at) = sample_session(100);
  let total_energy = energy(10_000);
  let ended_at = started_at + Duration::minutes(super::FREE_MINUTES as i64);

  let (billed_energy, charged_amount) = session.bill_snapshot(ended_at, total_energy).expect("snapshot succeeds");

  assert_eq!(billed_energy, KwhMilli::zero());
  assert_eq!(charged_amount, yen(0));
}

#[test]
fn just_beyond_free_window_rounds_down() {
  let (session, started_at) = sample_session(60);
  let total_energy = energy(6_000);
  let ended_at = started_at + Duration::minutes(super::FREE_MINUTES as i64) + Duration::seconds(1);

  let (billed_energy, charged_amount) = session.bill_snapshot(ended_at, total_energy).expect("snapshot succeeds");

  assert_eq!(billed_energy, energy(19));
  assert_eq!(charged_amount, yen(1));
}

#[test]
fn stop_rejects_invalid_timeline() {
  let (session, started_at) = sample_session(50);
  let total_energy = energy(5_000);
  let ended_at = started_at - Duration::minutes(1);

  let err = session.stop(ended_at, total_energy).unwrap_err();
  assert!(matches!(
    err,
    SessionValueError::InvalidTimeline {
      started_at: s,
      ended_at: e,
    } if s == started_at && e == ended_at
  ));
}

#[test]
fn bill_snapshot_rejects_invalid_timeline() {
  let (session, started_at) = sample_session(45);
  let total_energy = energy(4_000);
  let ended_at = started_at - Duration::seconds(1);

  let err = session.bill_snapshot(ended_at, total_energy).unwrap_err();
  assert!(matches!(err, SessionValueError::InvalidTimeline { .. }));
}

#[test]
fn bill_after_stop_is_rejected() {
  let (session, started_at) = sample_session(60);
  let total_energy = energy(8_000);
  let ended_at = started_at + Duration::minutes(15);

  let closed = session.stop(ended_at, total_energy).expect("stop succeeds");
  let err = closed.bill_after_stop(ended_at + Duration::minutes(5), total_energy).unwrap_err();
  assert!(matches!(err, SessionValueError::AlreadyClosed { .. }));
}

#[test]
fn amount_out_of_range_is_rejected() {
  let (session, started_at) = sample_session(1_500);
  let total_energy = energy(super::MAX_KWH_MILLI);
  let ended_at = started_at + Duration::hours(10);

  let err = session.bill_snapshot(ended_at, total_energy).unwrap_err();
  assert!(matches!(err, SessionValueError::AmountOutOfRange { .. }));
}

#[test]
fn negative_energy_input_is_rejected() {
  let err = KwhMilli::try_from_i64(-1).unwrap_err();
  assert!(matches!(err, SessionValueError::NegativeEnergy { provided: -1 }));
}

#[test]
fn energy_above_upper_bound_fails() {
  let err = KwhMilli::try_new(super::MAX_KWH_MILLI + 1).unwrap_err();
  assert!(matches!(err, SessionValueError::EnergyOutOfRange { .. }));
}
