/// model-a（非AVDM）で利用する課金セッションの生データ表現。
/// 仕様上の不変条件はほとんど担保せず、呼び出し側で整合性を維持する前提。
pub struct Session {
  /// セッション開始時刻（エポックミリ秒）。
  pub started_at: Option<i64>,
  /// セッション終了時刻（エポックミリ秒）。
  pub ended_at: Option<i64>,
  /// セッション全体のエネルギー量（ミリkWh）。
  pub kwh_milli: u64,
  /// 単価（円/kWh）。
  pub rate_yen_per_kwh: u32,
  /// 計算後に記録される課金対象エネルギー量（ミリkWh）。
  pub billed_kwh_milli: u64,
  /// 状態文字列（例: "active" / "closed"）。
  pub status: String,
  /// 再課金を抑止するためのフラグ。
  pub already_billed: bool,
}

/// 1 セッションあたりの最大課金額（円）。
const MAX_AMOUNT_YEN: u32 = 1_000_000;
/// 1 セッションあたりの最大エネルギー量（ミリkWh）。
const MAX_KWH_MILLI: u64 = 1_000_000;

/// `Session` の生データを手続き的に処理して料金を算出する。
pub fn calculate_charge(session: &mut Session) -> Result<u32, String> {
    todo!("calculate_charge")
}

#[cfg(test)]
mod tests {

}
