use model_b_avdm::session::{KwhMilli, RateYenPerKwh, Session, SessionId, SessionValueError};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{BillingResult, BillingSession, ClosedBillingSession};

/// AVDM モデルを `BillingSession` として扱うためのアダプタ。
#[derive(Debug)]
pub struct ModelBSession {
  inner: Session,
}

/// model-b のドメインエラーと技術的エラーをまとめた型。
#[derive(Debug, thiserror::Error)]
pub enum ModelBError {
  /// タイムスタンプが許容範囲外だった。
  #[error("タイムスタンプが範囲外です: {0}")]
  TimestampOutOfRange(#[from] time::error::ComponentRange),
  /// エネルギー値が負だった。
  #[error("エネルギーが負です: {0}")]
  NegativeEnergy(i64),
  /// ドメインルールに違反した。
  #[error("ドメインエラー: {0}")]
  Domain(#[from] SessionValueError),
}

impl BillingSession for ModelBSession {
  type ClosedSession = ClosedModelBSession;
  type Error = ModelBError;

  /// セッションを開始する。
  ///
  /// # Errors
  /// - タイムスタンプが範囲外の場合。
  /// - 単価がドメイン制約に反する場合。
  fn start(start_epoch_ms: i64, rate_yen_per_kwh: u32) -> Result<Self, Self::Error> {
    let started_at = ms_to_offset_datetime(start_epoch_ms)?;
    let rate = RateYenPerKwh::try_new(rate_yen_per_kwh)?;
    let session = Session::new_active(SessionId::new(Uuid::nil()), started_at, rate);
    Ok(Self { inner: session })
  }

  fn bill_snapshot(&self, end_epoch_ms: i64, energy_milli: i64) -> Result<BillingResult, Self::Error> {
    if energy_milli < 0 {
      return Err(ModelBError::NegativeEnergy(energy_milli));
    }
    let energy = KwhMilli::try_from_i64(energy_milli)?;
    let ended_at = ms_to_offset_datetime(end_epoch_ms)?;
    let bill = self.inner.bill_snapshot(ended_at, energy)?;
    Ok(BillingResult::from_model_b(bill.billable_energy(), bill.amount_due()))
  }

  fn stop(self, end_epoch_ms: i64, energy_milli: i64) -> Result<(BillingResult, Self::ClosedSession), Self::Error> {
    if energy_milli < 0 {
      return Err(ModelBError::NegativeEnergy(energy_milli));
    }
    let energy = KwhMilli::try_from_i64(energy_milli)?;
    let ended_at = ms_to_offset_datetime(end_epoch_ms)?;
    let session = self.inner.stop(ended_at, energy)?;
    let result = match &session {
      | Session::Closed { bill, .. } => BillingResult::from_model_b(bill.billable_energy(), bill.amount_due()),
      | Session::Active { .. } => panic!("expected closed session"),
    };
    Ok((result, ClosedModelBSession::new(session)))
  }
}

/// 停止済み AVDM セッションを `ClosedBillingSession` として扱うラッパー。
#[derive(Debug)]
pub struct ClosedModelBSession {
  inner: Session,
}

impl ClosedModelBSession {
  fn new(inner: Session) -> Self {
    match inner {
      | Session::Closed { .. } => Self { inner },
      | Session::Active { .. } => panic!("expected closed session"),
    }
  }
}

impl ClosedBillingSession for ClosedModelBSession {
  type Error = ModelBError;

  fn bill_after_stop(&self, _end_epoch_ms: i64, _energy_milli: i64) -> Result<BillingResult, Self::Error> {
    match &self.inner {
      | Session::Closed { id, .. } => Err(ModelBError::Domain(SessionValueError::AlreadyClosed { session_id: *id })),
      | Session::Active { .. } => panic!("closed session wrapper must hold a closed session"),
    }
  }
}

/// ミリ秒エポックから `OffsetDateTime` を生成する。
///
/// # Errors
/// タイムスタンプが許容範囲外の場合、`ModelBError::TimestampOutOfRange` を返します。
///
/// # Returns
/// 変換された `OffsetDateTime`。
fn ms_to_offset_datetime(epoch_ms: i64) -> Result<OffsetDateTime, ModelBError> {
  let seconds = epoch_ms.div_euclid(1_000);
  let millis = epoch_ms.rem_euclid(1_000);
  let base = OffsetDateTime::from_unix_timestamp(seconds)?;
  Ok(base + Duration::milliseconds(millis))
}
