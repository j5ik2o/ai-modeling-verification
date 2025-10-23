use thiserror::Error;
use time::OffsetDateTime;

use super::session_id::SessionId;

/// セッション操作中に発生し得るドメインエラー。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SessionValueError {
  /// 負のエネルギー値が入力された。
  #[error("エネルギーは負にできません (入力: {provided})")]
  NegativeEnergy {
    /// 入力された値。
    provided: i64,
  },
  /// 単価が 0 以下だった。
  #[error("単価は1円/kWh以上である必要があります")]
  NonPositiveRate,
  /// 終了時刻が開始時刻以前だった。
  #[error("終了時刻 {ended_at} は開始時刻 {started_at} より後でなければなりません")]
  InvalidTimeline {
    /// セッション開始時刻。
    started_at: OffsetDateTime,
    /// セッション終了時刻。
    ended_at:   OffsetDateTime,
  },
  /// 金額が `u64` 上限を超えた。
  #[error("料金が表現できる上限を超過しました (入力: {provided})")]
  AmountOverflow {
    /// 入力金額。
    provided: u128,
  },
  /// 既に停止済みセッションに操作しようとした。
  #[error("セッション {session_id:?} は既に停止済みです")]
  AlreadyClosed {
    /// 対象セッションID。
    session_id: SessionId,
  },
  /// エネルギー量が許容範囲を超過した。
  #[error("エネルギー量が上限を超過しています (入力: {provided} / 上限: {max})")]
  EnergyOutOfRange {
    /// 入力値。
    provided: u64,
    /// 許容上限。
    max:      u64,
  },
  /// 金額が許容範囲を超過した。
  #[error("請求額が上限を超過しています (入力: {provided} / 上限: {max})")]
  AmountOutOfRange {
    /// 入力値。
    provided: u64,
    /// 許容上限。
    max:      u64,
  },
  /// 課金窓比率が不正だった。
  #[error("課金比率が不正です (分子: {numerator}, 分母: {denominator})")]
  InvalidChargeRatio {
    /// 比率の分子。
    numerator:   u128,
    /// 比率の分母。
    denominator: u128,
  },
}
