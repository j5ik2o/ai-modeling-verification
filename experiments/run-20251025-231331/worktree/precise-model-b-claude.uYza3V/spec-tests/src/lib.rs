//! spec-tests クレートは、AVDM と 非AVDM
//! の実装を共通受け入れテストで比較するためのハーネスを提供します。
#![deny(missing_docs)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
/// アダプタ実装をまとめたモジュール。
pub mod adapters;

/// 課金結果を両モデルの比較に利用するための共有構造体。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BillingResult {
  /// 課金対象エネルギー量（ミリkWh）。
  pub billed_energy_milli: u64,
  /// 請求金額（円）。
  pub amount_yen:          u64,
}

impl BillingResult {
  /// model-b のドメイン値オブジェクトから比較用の結果を生成する。
  ///
  /// # Errors
  /// 本関数は失敗しません。
  ///
  /// # Returns
  /// `BillingResult` を返します。
  pub fn from_model_b(energy: model_b_avdm::session::KwhMilli, amount: model_b_avdm::session::MoneyYen) -> Self {
    Self { billed_energy_milli: u64::from(energy), amount_yen: u64::from(amount) }
  }
}

/// 受け入れテストで使用する抽象化されたセッション操作。
pub trait BillingSession {
  /// エラー型。
  type Error;
  /// 停止後に利用するセッション型。
  type ClosedSession: ClosedBillingSession<Error = Self::Error>;

  /// セッションを開始する。
  ///
  /// # Errors
  /// 実装側で入力検証に失敗した場合にエラーを返します。
  fn start(start_epoch_ms: i64, rate_yen_per_kwh: u32) -> Result<Self, Self::Error>
  where
    Self: Sized;

  /// 停止前の任意時点で料金計算を行い、スナップショットを取得する。
  ///
  /// # Errors
  /// 実装側で入力が仕様に反した場合にエラーを返します。
  fn bill_snapshot(&self, end_epoch_ms: i64, energy_milli: i64) -> Result<BillingResult, Self::Error>;

  /// セッションを停止し、確定請求と停止済みハンドルを返す。
  ///
  /// # Errors
  /// 実装側で停止処理が認められない場合（例: タイムライン不正など）にエラーを返します。
  fn stop(self, end_epoch_ms: i64, energy_milli: i64) -> Result<(BillingResult, Self::ClosedSession), Self::Error>;
}

/// 停止済みセッションに対する共通操作（再課金拒否など）を表現する。
pub trait ClosedBillingSession {
  /// エラー型。
  type Error;

  /// 停止後の課金を試みた場合の挙動を定義する。
  ///
  /// # Errors
  /// 実装側で停止済み課金が禁止されている場合にエラーを返します。
  fn bill_after_stop(&self, end_epoch_ms: i64, energy_milli: i64) -> Result<BillingResult, Self::Error>;
}
