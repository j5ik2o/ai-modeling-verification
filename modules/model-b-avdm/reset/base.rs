use time::OffsetDateTime;

use super::{
  bill::SessionBill, chargeable_energy::ChargeableEnergy, errors::SessionValueError, grace_period::GracePeriod,
  kwh_milli::KwhMilli, rate::RateYenPerKwh, session_id::SessionId, timeline::SessionTimeline,
};

/// 充電セッションのライフサイクルを表す列挙体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Session {
  /// 課金進行中の状態。
  Active {
    /// セッションID。
    id:         SessionId,
    /// セッション開始時刻。
    started_at: OffsetDateTime,
    /// 単価（円/kWh）。
    rate:       RateYenPerKwh,
  },
  /// 停止済みで請求が確定した状態。
  Closed {
    /// セッションID。
    id:         SessionId,
    /// セッション開始時刻。
    started_at: OffsetDateTime,
    /// 終了時刻。
    ended_at:   OffsetDateTime,
    /// 単価（円/kWh）。
    rate:       RateYenPerKwh,
    /// 確定した請求。
    bill:       SessionBill,
  },
}

impl Session {
  /// アクティブ状態のセッションを生成する。
  pub fn new_active(id: SessionId, started_at: OffsetDateTime, rate: RateYenPerKwh) -> Self {
      todo!("AIに実装させる")
  }

  /// セッションを停止し、請求を確定させる。
  pub fn stop(self, ended_at: OffsetDateTime, total_energy: KwhMilli) -> Result<Self, SessionValueError> {
      todo!("AIに実装させる")
  }

  /// 指定時点での課金スナップショットを取得する。
  pub fn bill_snapshot(
    &self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<SessionBill, SessionValueError> {
    todo!("AIに実装させる")
  }

  /// 停止後の追加課金要求に応答する。
  pub fn bill_after_stop(
    &self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<SessionBill, SessionValueError> {
    todo!("AIに実装させる")
  }

  /// セッションを識別する。
  #[must_use]
  pub fn identity(&self) -> SessionId {
    todo!("AIに実装させる")
  }

  /// 請求書を参照する（停止済みのみ）。
  #[must_use]
  pub fn statement(&self) -> Option<&SessionBill> {
    todo!("AIに実装させる")
  }

  fn grace_period() -> GracePeriod {
    todo!("AIに実装させる")
  }
}
