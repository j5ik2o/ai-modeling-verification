use time::OffsetDateTime;

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
  pub fn new_active(id: SessionId, started_at: OffsetDateTime, rate: RateYenPerKwh) -> Self {
      todo!("new_active");
  }

  /// セッションIDを取得する。
  pub fn id(&self) -> SessionId {
      todo!("id");
  }

  /// セッション開始時刻を取得する。
  pub fn started_at(&self) -> OffsetDateTime {
      todo!("started_at");
  }

  /// セッション終了時刻を取得する（アクティブ時は `None`）。
  pub fn ended_at(&self) -> Option<OffsetDateTime> {
      todo!("ended_at");
  }

  /// セッション単価を取得する。
  pub fn rate(&self) -> RateYenPerKwh {
      todo!("rate");
  }

  /// セッション全体のエネルギー量を取得する（アクティブ時は `None`）。
  pub fn total_energy(&self) -> Option<KwhMilli> {
      todo!("total_enegry");
  }

  /// 課金対象エネルギー量を取得する（アクティブ時は `None`）。
  pub fn billed_energy(&self) -> Option<KwhMilli> {
      todo!("billed_energy");
  }

  /// 請求金額を取得する（アクティブ時は `None`）。
  pub fn charged_amount(&self) -> Option<MoneyYen> {
      todo!("charged_amount");
  }

  /// セッションを停止し、請求を確定させる。
  pub fn stop(
    self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<Self, SessionValueError> {
      todo!("stop");
  }

  /// 指定時点での課金スナップショットを取得する。
  pub fn bill_snapshot(
    &self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
      todo!("bill_snapshot");
  }

  /// 停止後の追加課金要求に応答する。
  pub fn bill_after_stop(
    &self,
    ended_at: OffsetDateTime,
    total_energy: KwhMilli,
  ) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
      todo!("bill_after_stop");
  }

}
