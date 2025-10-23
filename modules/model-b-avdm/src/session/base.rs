use time::OffsetDateTime;

use super::{
  FREE_MILLISECONDS, chargeable_energy::ChargeableEnergy, energy::KwhMilli,
  errors::SessionValueError, money::MoneyYen, rate::RateYenPerKwh, session_id::SessionId,
};

/// 充電セッションのライフサイクルを表す列挙体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Session {
  /// 課金進行中の状態。
  Active {
    /// セッションID。
    id: SessionId,
    /// セッション開始時刻。
    started_at: OffsetDateTime,
    /// 単価（円/kWh）。
    rate: RateYenPerKwh,
  },
  /// 停止済みで請求が確定した状態。
  Closed {
    /// セッションID。
    id: SessionId,
    /// セッション開始時刻。
    started_at: OffsetDateTime,
    /// 終了時刻。
    ended_at: OffsetDateTime,
    /// 単価（円/kWh）。
    rate: RateYenPerKwh,
    /// セッション全体のエネルギー量（ミリkWh）。
    total_energy: KwhMilli,
    /// 課金対象となったエネルギー量（ミリkWh）。
    billed_energy: KwhMilli,
    /// 請求金額（円）。
    charged_amount: MoneyYen,
  },
}

impl Session {
  /// アクティブ状態のセッションを生成する。
  ///
  /// # Returns
  /// アクティブ状態の `Session` を返します。
  pub fn new_active(id: SessionId, started_at: OffsetDateTime, rate: RateYenPerKwh) -> Self {
    Session::Active {
      id,
      started_at,
      rate,
    }
  }

  /// セッションIDを取得する。
  ///
  /// # Returns
  /// セッション識別子。
  pub fn id(&self) -> SessionId {
    match self {
      Session::Active { id, .. } | Session::Closed { id, .. } => *id,
    }
  }

  /// セッション開始時刻を取得する。
  ///
  /// # Returns
  /// 開始時刻。
  pub fn started_at(&self) -> OffsetDateTime {
    match self {
      Session::Active { started_at, .. } | Session::Closed { started_at, .. } => *started_at,
    }
  }

  /// セッション終了時刻を取得する（アクティブ時は `None`）。
  ///
  /// # Returns
  /// 終了時刻、またはアクティブ時は `None`。
  pub fn ended_at(&self) -> Option<OffsetDateTime> {
    match self {
      Session::Active { .. } => None,
      Session::Closed { ended_at, .. } => Some(*ended_at),
    }
  }

  /// セッション単価を取得する。
  ///
  /// # Returns
  /// 単価。
  pub fn rate(&self) -> RateYenPerKwh {
    match self {
      Session::Active { rate, .. } | Session::Closed { rate, .. } => *rate,
    }
  }

  /// セッション全体のエネルギー量を取得する（アクティブ時は `None`）。
  ///
  /// # Returns
  /// エネルギー量、またはアクティブ時は `None`。
  pub fn total_energy(&self) -> Option<KwhMilli> {
    match self {
      Session::Active { .. } => None,
      Session::Closed { total_energy, .. } => Some(*total_energy),
    }
  }

  /// 課金対象エネルギー量を取得する（アクティブ時は `None`）。
  ///
  /// # Returns
  /// 課金対象エネルギー量、またはアクティブ時は `None`。
  pub fn billed_energy(&self) -> Option<KwhMilli> {
    match self {
      Session::Active { .. } => None,
      Session::Closed { billed_energy, .. } => Some(*billed_energy),
    }
  }

  /// 請求金額を取得する（アクティブ時は `None`）。
  ///
  /// # Returns
  /// 請求金額、またはアクティブ時は `None`。
  pub fn charged_amount(&self) -> Option<MoneyYen> {
    match self {
      Session::Active { .. } => None,
      Session::Closed { charged_amount, .. } => Some(*charged_amount),
    }
  }

  /// セッションを停止し、請求を確定させる。
  ///
  /// # Returns
  /// 停止後の `Session::Closed` を返します。
  ///
  /// # Errors
  /// - 既に停止済みのセッションに対して呼び出した場合。
  /// - 終了時刻が開始時刻以前の場合。
  pub fn stop(
    self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<Self, SessionValueError> {
    match self {
      Session::Active {
        id,
        started_at,
        rate,
      } => {
        let (billed_energy, charged_amount) =
          Self::bill_snapshot_for(started_at, rate, ended_at, total_energy)?;
        Ok(Session::Closed {
          id,
          started_at,
          ended_at,
          rate,
          total_energy,
          billed_energy,
          charged_amount,
        })
      }
      Session::Closed { id, .. } => Err(SessionValueError::AlreadyClosed { session_id: id }),
    }
  }

  /// 指定時点での課金スナップショットを取得する。
  ///
  /// # Returns
  /// 課金対象エネルギー量と金額のタプル。
  ///
  /// # Errors
  /// - アクティブ状態でタイムラインが不正な場合。
  /// - 停止済みだが異なる条件で計算しようとした場合。
  pub fn bill_snapshot(
    &self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    match self {
      Session::Active {
        started_at, rate, ..
      } => Self::bill_snapshot_for(*started_at, *rate, ended_at, total_energy),
      Session::Closed {
        id,
        ended_at: stored_end,
        total_energy: stored_energy,
        billed_energy,
        charged_amount,
        ..
      } => {
        if *stored_end == ended_at && *stored_energy == total_energy {
          Ok((*billed_energy, *charged_amount))
        } else {
          Err(SessionValueError::AlreadyClosed { session_id: *id })
        }
      }
    }
  }

  /// 停止後の追加課金要求に応答する。
  ///
  /// # Returns
  /// 成功時は課金対象エネルギー量と金額のタプル。
  ///
  /// # Errors
  /// 停止済みセッションに課金しようとした場合は常に `SessionValueError::AlreadyClosed` を返します。
  pub fn bill_after_stop(
    &self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    match self {
      Session::Active { .. } => self.bill_snapshot(ended_at, total_energy),
      Session::Closed { id, .. } => Err(SessionValueError::AlreadyClosed { session_id: *id }),
    }
  }

  #[allow(clippy::missing_errors_doc)]
  fn bill_snapshot_for(
    started_at: OffsetDateTime,
    rate: RateYenPerKwh,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    let billed_energy = Self::billed_energy_for(started_at, ended_at, total_energy)?;
    let amount = rate.charge(billed_energy)?;
    Ok((billed_energy, amount))
  }

  #[allow(clippy::missing_errors_doc)]
  fn billed_energy_for(
    started_at: OffsetDateTime,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<KwhMilli, SessionValueError> {
    let total_ms = Self::duration_millis(started_at, ended_at)?;
    let free_ms = FREE_MILLISECONDS.min(total_ms);
    let chargeable_ms = total_ms - free_ms;

    let chargeable =
      ChargeableEnergy::from_chargeable_window(total_energy, chargeable_ms, total_ms)?;
    Ok(chargeable.billed())
  }

  #[allow(clippy::missing_errors_doc)]
  fn duration_millis(
    started_at: OffsetDateTime,
    ended_at: OffsetDateTime,
  ) -> Result<u128, SessionValueError> {
    let millis = (ended_at - started_at).whole_milliseconds();
    if millis <= 0 {
      Err(SessionValueError::InvalidTimeline {
        started_at,
        ended_at,
      })
    } else {
      Ok(millis as u128)
    }
  }
}
