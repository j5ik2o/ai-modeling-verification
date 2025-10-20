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

/// `Session` の生データを手続き的に処理して料金を算出する。
/// 無料5分の按分・端数切り捨て・停止後課金禁止など spec.md の要件を
/// 呼び出し側で順守するための薄いユーティリティ。
///
/// # Errors
/// `Err` は以下の理由で発生します。
/// - セッションが既に課金済み、または `status` が `"closed"` ではない場合。
/// - 開始・終了時刻の欠損、あるいは終了時刻が開始時刻以前である場合。
pub fn calculate_charge(session: &mut Session) -> Result<u32, String> {
  if session.already_billed {
    return Err("session already billed".to_string());
  }
  if session.status != "closed" {
    return Err(format!("status {} is not billable", session.status));
  }

  let started_at = session
    .started_at
    .ok_or_else(|| "missing start timestamp".to_string())?;
  let ended_at = session
    .ended_at
    .ok_or_else(|| "missing end timestamp".to_string())?;

  if ended_at <= started_at {
    return Err(format!(
      "invalid timeline: start={} end={}",
      started_at, ended_at
    ));
  }

  let duration_ms = (ended_at - started_at) as f64;
  if duration_ms <= 0.0 {
    return Err("duration must be positive".to_string());
  }

  let free_ms = 5.0 * 60.0 * 1000.0;
  let chargeable_ratio = ((duration_ms - free_ms).max(0.0)) / duration_ms;

  let billed_energy_milli = ((session.kwh_milli as f64) * chargeable_ratio).floor() as u64;
  session.billed_kwh_milli = billed_energy_milli;

  let billed_energy_kwh = billed_energy_milli as f64 / 1_000.0;
  let amount = (billed_energy_kwh * session.rate_yen_per_kwh as f64).floor() as u32;

  session.already_billed = true;

  Ok(amount)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn new_session(start_ms: i64, end_ms: i64, energy_milli: u64, rate: u32) -> Session {
    Session {
      started_at: Some(start_ms),
      ended_at: Some(end_ms),
      kwh_milli: energy_milli,
      rate_yen_per_kwh: rate,
      billed_kwh_milli: 0,
      status: "closed".to_string(),
      already_billed: false,
    }
  }

  #[test]
  fn scenario1_six_minutes_charges_one_minute() {
    let mut session = new_session(0, 6 * 60 * 1000, 2_400, 50);
    let amount = calculate_charge(&mut session).expect("calculation succeeds");

    assert_eq!(session.billed_kwh_milli, 400);
    assert_eq!(amount, 20);
  }

  #[test]
  fn scenario2_four_minutes_is_free() {
    let mut session = new_session(0, 4 * 60 * 1000, 1_000, 80);
    let amount = calculate_charge(&mut session).expect("calculation succeeds");

    assert_eq!(session.billed_kwh_milli, 0);
    assert_eq!(amount, 0);
  }

  #[test]
  fn scenario3_exactly_five_minutes_is_free() {
    let mut session = new_session(0, 5 * 60 * 1000, 3_000, 100);
    let amount = calculate_charge(&mut session).expect("calculation succeeds");

    assert_eq!(session.billed_kwh_milli, 0);
    assert_eq!(amount, 0);
  }
}
