use crate::{BillingResult, BillingSession, ClosedBillingSession};
use model_a_non_avdm::session::{Session as SessionA, calculate_charge};
use model_b_avdm::session::{KwhMilli, RateYenPerKwh, Session, SessionId, SessionValueError};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// 非AVDMモデルを `BillingSession` として扱うための薄いアダプタ。
#[derive(Debug)]
pub struct ModelASession {
  start_ms: i64,
  rate: u32,
}

/// model-a 固有の失敗をラップするエラー型。
#[derive(Debug, thiserror::Error)]
pub enum ModelAError {
  /// エネルギー値が負だった。
  #[error("エネルギーが負です: {0}")]
  NegativeEnergy(i64),
  /// 手続き的課金ロジックがエラーを返した。
  #[error("計算失敗: {0}")]
  Calculation(String),
}

impl BillingSession for ModelASession {
  type Error = ModelAError;
  type ClosedSession = ClosedModelASession;

  fn start(start_epoch_ms: i64, rate_yen_per_kwh: u32) -> Result<Self, Self::Error> {
    Ok(Self {
      start_ms: start_epoch_ms,
      rate: rate_yen_per_kwh,
    })
  }

  fn bill_snapshot(
    &self,
    end_epoch_ms: i64,
    energy_milli: i64,
  ) -> Result<BillingResult, Self::Error> {
    if energy_milli < 0 {
      return Err(ModelAError::NegativeEnergy(energy_milli));
    }
    let mut session = self.build_session(end_epoch_ms, energy_milli as u64);
    let amount = calculate_charge(&mut session).map_err(ModelAError::Calculation)? as u64;
    Ok(BillingResult {
      billed_energy_milli: session.billed_kwh_milli,
      amount_yen: amount,
    })
  }

  fn stop(
    self,
    end_epoch_ms: i64,
    energy_milli: i64,
  ) -> Result<(BillingResult, Self::ClosedSession), Self::Error> {
    if energy_milli < 0 {
      return Err(ModelAError::NegativeEnergy(energy_milli));
    }
    let result = self.bill_snapshot(end_epoch_ms, energy_milli)?;
    Ok((
      result,
      ClosedModelASession {
        message: "セッションはすでに停止済みです".to_string(),
      },
    ))
  }
}

impl ModelASession {
  fn build_session(&self, end_epoch_ms: i64, energy_milli: u64) -> SessionA {
    SessionA {
      started_at: Some(self.start_ms),
      ended_at: Some(end_epoch_ms),
      kwh_milli: energy_milli,
      rate_yen_per_kwh: self.rate,
      billed_kwh_milli: 0,
      status: "closed".to_string(),
      already_billed: false,
    }
  }
}

/// アダプタが停止後に返すダミーオブジェクト。追加課金は常に拒否する。
#[derive(Debug)]
pub struct ClosedModelASession {
  message: String,
}

impl ClosedBillingSession for ClosedModelASession {
  type Error = ModelAError;

  fn bill_after_stop(
    &self,
    _end_epoch_ms: i64,
    _energy_milli: i64,
  ) -> Result<BillingResult, Self::Error> {
    Err(ModelAError::Calculation(self.message.clone()))
  }
}

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
  type Error = ModelBError;
  type ClosedSession = ClosedModelBSession;

  fn start(start_epoch_ms: i64, rate_yen_per_kwh: u32) -> Result<Self, Self::Error> {
    let started_at = ms_to_offset_datetime(start_epoch_ms)?;
    let rate = RateYenPerKwh::new(rate_yen_per_kwh)?;
    let session = Session::new_active(SessionId::new(Uuid::nil()), started_at, rate);
    Ok(Self { inner: session })
  }

  fn bill_snapshot(
    &self,
    end_epoch_ms: i64,
    energy_milli: i64,
  ) -> Result<BillingResult, Self::Error> {
    if energy_milli < 0 {
      return Err(ModelBError::NegativeEnergy(energy_milli));
    }
    let energy = KwhMilli::try_from_i64(energy_milli)?;
    let ended_at = ms_to_offset_datetime(end_epoch_ms)?;
    let (billed, amount) = self.inner.bill_snapshot(ended_at, energy)?;
    Ok(BillingResult::from_model_b(billed, amount))
  }

  fn stop(
    self,
    end_epoch_ms: i64,
    energy_milli: i64,
  ) -> Result<(BillingResult, Self::ClosedSession), Self::Error> {
    if energy_milli < 0 {
      return Err(ModelBError::NegativeEnergy(energy_milli));
    }
    let energy = KwhMilli::try_from_i64(energy_milli)?;
    let ended_at = ms_to_offset_datetime(end_epoch_ms)?;
    let session = self.inner.stop(ended_at, energy)?;
    let billed = session
      .billed_energy()
      .expect("closed session must have billed energy");
    let amount = session
      .charged_amount()
      .expect("closed session must have charged amount");
    let result = BillingResult::from_model_b(billed, amount);
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
    if inner.billed_energy().is_some() {
      Self { inner }
    } else {
      panic!("expected closed session");
    }
  }
}

impl ClosedBillingSession for ClosedModelBSession {
  type Error = ModelBError;

  fn bill_after_stop(
    &self,
    _end_epoch_ms: i64,
    _energy_milli: i64,
  ) -> Result<BillingResult, Self::Error> {
    Err(ModelBError::Domain(SessionValueError::AlreadyClosed {
      session_id: self.inner.id(),
    }))
  }
}

/// ミリ秒エポックから `OffsetDateTime` を生成する。
///
/// # Errors
/// タイムスタンプが許容範囲外の場合、`ModelBError::TimestampOutOfRange` を返します。
fn ms_to_offset_datetime(epoch_ms: i64) -> Result<OffsetDateTime, ModelBError> {
  let seconds = epoch_ms.div_euclid(1_000);
  let millis = epoch_ms.rem_euclid(1_000);
  let base = OffsetDateTime::from_unix_timestamp(seconds)?;
  Ok(base + Duration::milliseconds(millis))
}
