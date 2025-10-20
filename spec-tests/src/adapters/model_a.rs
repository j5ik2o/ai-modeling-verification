use crate::{BillingResult, BillingSession, ClosedBillingSession};
use model_a_non_avdm::session::{Session as SessionA, calculate_charge};

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

  /// セッションを開始する。
  ///
  /// # Errors
  /// 現在の実装ではエラーを返しません。
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
