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
    Self::Active { id, started_at, rate }
  }

  /// セッションを停止し、請求を確定させる。
  pub fn stop(self, ended_at: OffsetDateTime, total_energy: KwhMilli) -> Result<Self, SessionValueError> {
    match self {
      | Self::Active { id, started_at, rate } => {
        let timeline = SessionTimeline::between(started_at, ended_at)?;
        let window = timeline.consume_grace_period(Self::grace_period());
        let energy = ChargeableEnergy::allocate(total_energy, window)?;
        let bill = SessionBill::settle(energy, rate)?;
        Ok(Self::Closed { id, started_at, ended_at, rate, bill })
      },
      | Self::Closed { id, .. } => Err(SessionValueError::AlreadyClosed { session_id: id }),
    }
  }

  /// 指定時点での課金スナップショットを取得する。
  pub fn bill_snapshot(
    &self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<SessionBill, SessionValueError> {
    match self {
      | Self::Active { started_at, rate, .. } => {
        let timeline = SessionTimeline::between(*started_at, ended_at)?;
        let window = timeline.consume_grace_period(Self::grace_period());
        let energy = ChargeableEnergy::allocate(total_energy, window)?;
        SessionBill::settle(energy, *rate)
      },
      | Self::Closed { id, .. } => Err(SessionValueError::AlreadyClosed { session_id: *id }),
    }
  }

  /// 停止後の追加課金要求に応答する。
  pub fn bill_after_stop(
    &self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<SessionBill, SessionValueError> {
    match self {
      | Self::Closed { id, .. } => Err(SessionValueError::AlreadyClosed { session_id: *id }),
      | Self::Active { started_at, rate, .. } => {
        let timeline = SessionTimeline::between(*started_at, ended_at)?;
        let window = timeline.consume_grace_period(Self::grace_period());
        let energy = ChargeableEnergy::allocate(total_energy, window)?;
        SessionBill::settle(energy, *rate)
      },
    }
  }

  /// セッションを識別する。
  #[must_use]
  pub fn identity(&self) -> SessionId {
    match self {
      | Self::Active { id, .. } | Self::Closed { id, .. } => *id,
    }
  }

  /// 請求書を参照する（停止済みのみ）。
  #[must_use]
  pub fn statement(&self) -> Option<&SessionBill> {
    match self {
      | Self::Closed { bill, .. } => Some(bill),
      | Self::Active { .. } => None,
    }
  }

  fn grace_period() -> GracePeriod {
    GracePeriod::from_minutes(super::FREE_MINUTES)
  }
}
