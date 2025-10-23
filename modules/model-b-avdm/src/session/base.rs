use std::convert::TryFrom;

use time::OffsetDateTime;

use super::{
  FREE_MILLISECONDS, chargeable_energy::ChargeableEnergy, errors::SessionValueError, kwh_milli::KwhMilli,
  money_yen::MoneyYen, rate::RateYenPerKwh, session_id::SessionId,
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
    id:             SessionId,
    /// セッション開始時刻。
    started_at:     OffsetDateTime,
    /// 終了時刻。
    ended_at:       OffsetDateTime,
    /// 単価（円/kWh）。
    rate:           RateYenPerKwh,
    /// セッション全体のエネルギー量（ミリkWh）。
    total_energy:   KwhMilli,
    /// 課金対象となったエネルギー量（ミリkWh）。
    billed_energy:  KwhMilli,
    /// 請求金額（円）。
    charged_amount: MoneyYen,
  },
}

impl Session {
  /// アクティブ状態のセッションを生成する。
  pub fn new_active(id: SessionId, started_at: OffsetDateTime, rate: RateYenPerKwh) -> Self {
    Self::Active { id, started_at, rate }
  }

  /// セッションIDを取得する。
  pub fn id(&self) -> SessionId {
    match self {
      | Self::Active { id, .. } | Self::Closed { id, .. } => *id,
    }
  }

  /// セッション開始時刻を取得する。
  pub fn started_at(&self) -> OffsetDateTime {
    match self {
      | Self::Active { started_at, .. } | Self::Closed { started_at, .. } => *started_at,
    }
  }

  /// セッション終了時刻を取得する（アクティブ時は `None`）。
  pub fn ended_at(&self) -> Option<OffsetDateTime> {
    match self {
      | Self::Active { .. } => None,
      | Self::Closed { ended_at, .. } => Some(*ended_at),
    }
  }

  /// セッション単価を取得する。
  pub fn rate(&self) -> RateYenPerKwh {
    match self {
      | Self::Active { rate, .. } | Self::Closed { rate, .. } => *rate,
    }
  }

  /// セッション全体のエネルギー量を取得する（アクティブ時は `None`）。
  pub fn total_energy(&self) -> Option<KwhMilli> {
    match self {
      | Self::Active { .. } => None,
      | Self::Closed { total_energy, .. } => Some(*total_energy),
    }
  }

  /// 課金対象エネルギー量を取得する（アクティブ時は `None`）。
  pub fn billed_energy(&self) -> Option<KwhMilli> {
    match self {
      | Self::Active { .. } => None,
      | Self::Closed { billed_energy, .. } => Some(*billed_energy),
    }
  }

  /// 請求金額を取得する（アクティブ時は `None`）。
  pub fn charged_amount(&self) -> Option<MoneyYen> {
    match self {
      | Self::Active { .. } => None,
      | Self::Closed { charged_amount, .. } => Some(*charged_amount),
    }
  }

  /// セッションを停止し、請求を確定させる。
  pub fn stop(self, ended_at: OffsetDateTime, total_energy: KwhMilli) -> Result<Self, SessionValueError> {
    match self {
      | Self::Active { id, started_at, rate } => {
        let (billed_energy, charged_amount) =
          Self::calculate_bill_components(started_at, ended_at, total_energy, rate)?;
        Ok(Self::Closed { id, started_at, ended_at, rate, total_energy, billed_energy, charged_amount })
      },
      | Self::Closed { id, .. } => Err(SessionValueError::AlreadyClosed { session_id: id }),
    }
  }

  /// 指定時点での課金スナップショットを取得する。
  pub fn bill_snapshot(
    &self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    match self {
      | Self::Active { .. } => {
        let started_at = self.started_at();
        let rate = self.rate();
        Self::calculate_bill_components(started_at, ended_at, total_energy, rate)
      },
      | Self::Closed { id, .. } => Err(SessionValueError::AlreadyClosed { session_id: *id }),
    }
  }

  /// 停止後の追加課金要求に応答する。
  pub fn bill_after_stop(
    &self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    match self {
      | Self::Closed { id, .. } => Err(SessionValueError::AlreadyClosed { session_id: *id }),
      | Self::Active { .. } => {
        let started_at = self.started_at();
        let rate = self.rate();
        Self::calculate_bill_components(started_at, ended_at, total_energy, rate)
      },
    }
  }
}

impl Session {
  fn calculate_bill_components(
    started_at: OffsetDateTime,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
    rate: RateYenPerKwh,
  ) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    let elapsed_millis = Self::elapsed_milliseconds(started_at, ended_at)?;
    if elapsed_millis == 0 {
      let billed_energy = KwhMilli::zero();
      let charged_amount = rate.charge(billed_energy)?;
      return Ok((billed_energy, charged_amount));
    }

    let chargeable_millis = elapsed_millis.saturating_sub(FREE_MILLISECONDS);
    let chargeable_energy = ChargeableEnergy::from_chargeable_window(total_energy, chargeable_millis, elapsed_millis)?;
    let billed_energy = chargeable_energy.billed();
    let charged_amount = rate.charge(billed_energy)?;
    Ok((billed_energy, charged_amount))
  }

  fn elapsed_milliseconds(started_at: OffsetDateTime, ended_at: OffsetDateTime) -> Result<u128, SessionValueError> {
    if ended_at <= started_at {
      return Err(SessionValueError::InvalidTimeline { started_at, ended_at });
    }

    let elapsed = ended_at - started_at;
    let millis = elapsed.whole_milliseconds();
    u128::try_from(millis).map_err(|_| SessionValueError::InvalidTimeline { started_at, ended_at })
  }
}
