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
    // 1. セッション状態検証
    if session.already_billed {
        return Err("session already billed".to_string());
    }
    if session.status != "closed" {
        return Err(format!("status {} is not billable", session.status));
    }

    // 2. 時刻検証
    let started_at = session.started_at.ok_or("missing start timestamp")?;
    let ended_at = session.ended_at.ok_or("missing end timestamp")?;
    if ended_at <= started_at {
        return Err("invalid timeline: end <= start".to_string());
    }

    // 3. エネルギー上限検証
    if session.kwh_milli > MAX_KWH_MILLI {
        return Err("energy exceeds limit".to_string());
    }

    // 4. 課金対象エネルギー計算
    let duration_ms = (ended_at - started_at) as f64;
    let free_ms = 5.0 * 60.0 * 1000.0; // 5分 = 300,000ミリ秒
    let chargeable_ratio = ((duration_ms - free_ms).max(0.0)) / duration_ms;
    let billed_energy_milli = ((session.kwh_milli as f64) * chargeable_ratio).floor() as u64;

    // 5. 金額計算
    let billed_energy_kwh = billed_energy_milli as f64 / 1_000.0;
    let amount = (billed_energy_kwh * session.rate_yen_per_kwh as f64).floor() as u32;

    // 6. 金額上限検証
    if amount > MAX_AMOUNT_YEN {
        return Err("amount exceeds limit".to_string());
    }

    // 7. 状態更新
    session.billed_kwh_milli = billed_energy_milli;
    session.already_billed = true;

    // 8. 戻り値
    Ok(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== 基本ケース =====

    #[test]
    fn scenario1_six_minutes_charges_one_minute() {
        // 6分利用、最初5分無料 → 1分相当のみ課金
        let mut session = Session {
            started_at: Some(0),
            ended_at: Some(6 * 60 * 1000), // 6分
            kwh_milli: 2_400, // 2.4 kWh
            rate_yen_per_kwh: 50,
            billed_kwh_milli: 0,
            status: "closed".to_string(),
            already_billed: false,
        };
        let amount = calculate_charge(&mut session).unwrap();
        assert_eq!(session.billed_kwh_milli, 400); // 0.4 kWh
        assert_eq!(amount, 20); // 20円
    }

    #[test]
    fn scenario2_four_minutes_is_free() {
        // 4分利用 → 全量無料
        let mut session = Session {
            started_at: Some(0),
            ended_at: Some(4 * 60 * 1000),
            kwh_milli: 1_000,
            rate_yen_per_kwh: 80,
            billed_kwh_milli: 0,
            status: "closed".to_string(),
            already_billed: false,
        };
        let amount = calculate_charge(&mut session).unwrap();
        assert_eq!(session.billed_kwh_milli, 0);
        assert_eq!(amount, 0);
    }

    #[test]
    fn scenario3_exactly_five_minutes_is_free() {
        // ぴったり5分 → 無料
        let mut session = Session {
            started_at: Some(0),
            ended_at: Some(5 * 60 * 1000),
            kwh_milli: 3_000,
            rate_yen_per_kwh: 100,
            billed_kwh_milli: 0,
            status: "closed".to_string(),
            already_billed: false,
        };
        let amount = calculate_charge(&mut session).unwrap();
        assert_eq!(session.billed_kwh_milli, 0);
        assert_eq!(amount, 0);
    }

    // ===== 異常系 =====

    #[test]
    fn error_when_already_billed() {
        let mut session = Session {
            started_at: Some(0),
            ended_at: Some(6 * 60 * 1000),
            kwh_milli: 2_400,
            rate_yen_per_kwh: 50,
            billed_kwh_milli: 0,
            status: "closed".to_string(),
            already_billed: true, // すでに課金済み
        };
        assert!(calculate_charge(&mut session).is_err());
    }

    #[test]
    fn error_when_status_not_closed() {
        let mut session = Session {
            started_at: Some(0),
            ended_at: Some(6 * 60 * 1000),
            kwh_milli: 2_400,
            rate_yen_per_kwh: 50,
            billed_kwh_milli: 0,
            status: "active".to_string(), // クローズされていない
            already_billed: false,
        };
        assert!(calculate_charge(&mut session).is_err());
    }

    #[test]
    fn error_when_end_before_start() {
        let mut session = Session {
            started_at: Some(10 * 60 * 1000),
            ended_at: Some(5 * 60 * 1000), // 終了 < 開始
            kwh_milli: 1_000,
            rate_yen_per_kwh: 80,
            billed_kwh_milli: 0,
            status: "closed".to_string(),
            already_billed: false,
        };
        assert!(calculate_charge(&mut session).is_err());
    }

    #[test]
    fn error_when_energy_exceeds_limit() {
        let mut session = Session {
            started_at: Some(0),
            ended_at: Some(10 * 60 * 1000),
            kwh_milli: 1_000_001, // 上限超過
            rate_yen_per_kwh: 50,
            billed_kwh_milli: 0,
            status: "closed".to_string(),
            already_billed: false,
        };
        assert!(calculate_charge(&mut session).is_err());
    }

    #[test]
    fn error_when_amount_exceeds_limit() {
        let mut session = Session {
            started_at: Some(0),
            ended_at: Some(20 * 60 * 1000),
            kwh_milli: 100_000, // 100 kWh
            rate_yen_per_kwh: 20_000, // 高額な単価
            billed_kwh_milli: 0,
            status: "closed".to_string(),
            already_billed: false,
        };
        assert!(calculate_charge(&mut session).is_err());
    }
}
