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

/// 無料時間（ミリ秒）。
const FREE_DURATION_MILLIS: i64 = 300_000; // 5分

/// `Session` の生データを手続き的に処理して料金を算出する。
///
/// # Errors
/// 以下の場合にエラーを返します：
/// - すでに課金済みの場合
/// - 時刻の整合性が取れない場合
/// - エネルギー量や金額が上限を超える場合
pub fn calculate_charge(session: &mut Session) -> Result<u32, String> {
    // 再課金防止
    if session.already_billed {
        return Err("Already billed".to_string());
    }

    // 時刻の整合性チェック
    let started_at = session.started_at.ok_or("Session not started")?;

    // 現在時刻または終了時刻を取得
    let current_time = if let Some(ended) = session.ended_at {
        // 終了時刻が開始時刻より前の場合はエラー
        if ended < started_at {
            return Err("Invalid time range: ended_at is before started_at".to_string());
        }
        ended
    } else {
        // 進行中のセッションの場合、現在時刻を使用
        use time::OffsetDateTime;
        let now = OffsetDateTime::now_utc();
        now.unix_timestamp() * 1000 + i64::from(now.millisecond())
    };

    // エネルギー量の上限チェック
    if session.kwh_milli > MAX_KWH_MILLI {
        return Err(format!(
            "Energy amount exceeds maximum: {} > {}",
            session.kwh_milli, MAX_KWH_MILLI
        ));
    }

    // 経過時間を計算
    let elapsed_millis = current_time - started_at;

    // 負の経過時間は不正
    if elapsed_millis < 0 {
        return Err("Invalid elapsed time: negative duration".to_string());
    }

    // 無料時間内の場合は0円
    if elapsed_millis <= FREE_DURATION_MILLIS {
        session.billed_kwh_milli = 0;
        if session.ended_at.is_some() {
            session.status = "closed".to_string();
            session.already_billed = true;
        }
        return Ok(0);
    }

    // 課金対象エネルギー量を計算
    session.billed_kwh_milli = session.kwh_milli;

    // 料金計算（ミリkWhをkWhに変換して単価を掛ける）
    // オーバーフローを防ぐため、u128で計算
    let amount_u128 = u128::from(session.billed_kwh_milli)
        * u128::from(session.rate_yen_per_kwh)
        / 1000;

    // u32の範囲に収まるかチェック
    if amount_u128 > u128::from(u32::MAX) {
        return Err(format!(
            "Calculated amount exceeds u32::MAX: {}",
            amount_u128
        ));
    }

    let amount = amount_u128 as u32;

    // 最大課金額チェック
    if amount > MAX_AMOUNT_YEN {
        return Err(format!(
            "Calculated amount exceeds maximum: {} > {}",
            amount, MAX_AMOUNT_YEN
        ));
    }

    // セッション終了時は状態を更新
    if session.ended_at.is_some() {
        session.status = "closed".to_string();
        session.already_billed = true;
    }

    Ok(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用のセッションを作成するヘルパー関数
    fn create_test_session(
        started_at: i64,
        ended_at: Option<i64>,
        kwh_milli: u64,
        rate_yen_per_kwh: u32,
    ) -> Session {
        Session {
            started_at: Some(started_at),
            ended_at,
            kwh_milli,
            rate_yen_per_kwh,
            billed_kwh_milli: 0,
            status: "active".to_string(),
            already_billed: false,
        }
    }

    // ========== 日常的な利用パターン ==========

    #[test]
    fn test_normal_charging_session() {
        // 10分間で10kWh（10,000ミリkWh）、単価30円/kWh
        let started = 1_000_000_000_000;
        let ended = started + 600_000; // 10分後
        let mut session = create_test_session(started, Some(ended), 10_000, 30);

        let result = calculate_charge(&mut session);
        assert!(result.is_ok());

        // 10kWh * 30円/kWh = 300円
        assert_eq!(result.unwrap(), 300);
        assert_eq!(session.billed_kwh_milli, 10_000);
        assert_eq!(session.status, "closed");
        assert!(session.already_billed);
    }

    #[test]
    fn test_zero_energy_session() {
        // エネルギーゼロのセッション
        let started = 1_000_000_000_000;
        let ended = started + 600_000; // 10分後
        let mut session = create_test_session(started, Some(ended), 0, 30);

        let result = calculate_charge(&mut session);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(session.billed_kwh_milli, 0);
    }

    #[test]
    fn test_long_session() {
        // 2時間で100kWh（100,000ミリkWh）、単価30円/kWh
        let started = 1_000_000_000_000;
        let ended = started + 7_200_000; // 2時間後
        let mut session = create_test_session(started, Some(ended), 100_000, 30);

        let result = calculate_charge(&mut session);
        assert!(result.is_ok());

        // 100kWh * 30円/kWh = 3,000円
        assert_eq!(result.unwrap(), 3_000);
    }

    #[test]
    fn test_in_progress_session() {
        // 進行中のセッション（ended_atがNone）
        let started = 1_000_000_000_000;
        let mut session = create_test_session(started, None, 10_000, 30);

        let result = calculate_charge(&mut session);
        // 現在時刻が使われるため、無料時間を超えていることを想定
        assert!(result.is_ok());
        // セッションは進行中のままで、already_billedはfalseのまま
        assert_eq!(session.status, "active");
        assert!(!session.already_billed);
    }

    // ========== 境界条件 ==========

    #[test]
    fn test_exactly_free_duration() {
        // ちょうど5分（無料時間）
        let started = 1_000_000_000_000;
        let ended = started + FREE_DURATION_MILLIS;
        let mut session = create_test_session(started, Some(ended), 5_000, 30);

        let result = calculate_charge(&mut session);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(session.billed_kwh_milli, 0);
        assert!(session.already_billed);
    }

    #[test]
    fn test_just_over_free_duration() {
        // 無料時間を1ミリ秒だけ超える
        let started = 1_000_000_000_000;
        let ended = started + FREE_DURATION_MILLIS + 1;
        let mut session = create_test_session(started, Some(ended), 10_000, 30);

        let result = calculate_charge(&mut session);
        assert!(result.is_ok());
        // 課金されるべき
        assert_eq!(result.unwrap(), 300);
        assert_eq!(session.billed_kwh_milli, 10_000);
    }

    #[test]
    fn test_just_under_free_duration() {
        // 無料時間の1ミリ秒前
        let started = 1_000_000_000_000;
        let ended = started + FREE_DURATION_MILLIS - 1;
        let mut session = create_test_session(started, Some(ended), 5_000, 30);

        let result = calculate_charge(&mut session);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(session.billed_kwh_milli, 0);
    }

    // ========== 異常系（不正な操作） ==========

    #[test]
    fn test_double_billing_prevented() {
        // 停止後に再度課金しようとする
        let started = 1_000_000_000_000;
        let ended = started + 600_000;
        let mut session = create_test_session(started, Some(ended), 10_000, 30);

        // 1回目の課金
        let result1 = calculate_charge(&mut session);
        assert!(result1.is_ok());

        // 2回目の課金（エラーになるべき）
        let result2 = calculate_charge(&mut session);
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("Already billed"));
    }

    #[test]
    fn test_time_reversal() {
        // 終了時刻が開始時刻より前
        let started = 1_000_000_000_000;
        let ended = started - 100_000; // 開始より前
        let mut session = create_test_session(started, Some(ended), 10_000, 30);

        let result = calculate_charge(&mut session);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("before started_at"));
    }

    #[test]
    fn test_session_not_started() {
        // started_atがNone
        let mut session = Session {
            started_at: None,
            ended_at: Some(1_000_000_000_000),
            kwh_milli: 10_000,
            rate_yen_per_kwh: 30,
            billed_kwh_milli: 0,
            status: "active".to_string(),
            already_billed: false,
        };

        let result = calculate_charge(&mut session);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not started"));
    }

    #[test]
    fn test_excessive_energy_amount() {
        // 桁外れのエネルギー量（上限を超える）
        let started = 1_000_000_000_000;
        let ended = started + 600_000;
        let mut session = create_test_session(started, Some(ended), MAX_KWH_MILLI + 1, 30);

        let result = calculate_charge(&mut session);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum"));
    }

    #[test]
    fn test_excessive_calculated_amount() {
        // 計算結果が最大課金額を超える
        let started = 1_000_000_000_000;
        let ended = started + 600_000;
        // MAX_AMOUNT_YEN = 1,000,000円を超えるように設定
        // 例: 100,000kWh * 20円/kWh = 2,000,000円
        let mut session = create_test_session(started, Some(ended), 100_000_000, 20);

        let result = calculate_charge(&mut session);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum"));
    }

    #[test]
    fn test_overflow_protection() {
        // u32をオーバーフローさせようとする
        let started = 1_000_000_000_000;
        let ended = started + 600_000;
        // u32::MAX = 4,294,967,295
        // 例: 1,000,000ミリkWh * 5,000円/kWh / 1000 = 5,000,000円
        // これはMAX_AMOUNT_YEN（1,000,000円）を超える
        let mut session = create_test_session(started, Some(ended), 1_000_000, 5_000);

        let result = calculate_charge(&mut session);
        assert!(result.is_err());
        // エラーメッセージを確認（どちらかのチェックに引っかかる）
        let err = result.unwrap_err();
        assert!(err.contains("exceeds maximum") || err.contains("u32::MAX"));
    }

    // ========== 決定性のテスト ==========

    #[test]
    fn test_deterministic_calculation() {
        // 同じ条件で複数回計算しても同じ結果になること
        let started = 1_000_000_000_000;
        let ended = started + 600_000;

        let mut session1 = create_test_session(started, Some(ended), 10_000, 30);
        let mut session2 = create_test_session(started, Some(ended), 10_000, 30);

        let result1 = calculate_charge(&mut session1);
        let result2 = calculate_charge(&mut session2);

        assert_eq!(result1, result2);
        assert_eq!(session1.billed_kwh_milli, session2.billed_kwh_milli);
    }

    // ========== エッジケース ==========

    #[test]
    fn test_minimum_charged_session() {
        // 無料時間を超えた最小のセッション
        let started = 1_000_000_000_000;
        let ended = started + FREE_DURATION_MILLIS + 1;
        let mut session = create_test_session(started, Some(ended), 1, 1);

        let result = calculate_charge(&mut session);
        assert!(result.is_ok());
        // 1ミリkWh * 1円/kWh / 1000 = 0.001円 → 0円（整数除算）
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_maximum_valid_session() {
        // 上限ぎりぎりのセッション
        let started = 1_000_000_000_000;
        let ended = started + 600_000;
        // MAX_AMOUNT_YEN = 1,000,000円になるように設定
        // 1,000,000ミリkWh * 1,000円/kWh / 1000 = 1,000,000円
        let mut session = create_test_session(started, Some(ended), MAX_KWH_MILLI, 1_000);

        let result = calculate_charge(&mut session);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1_000_000);
    }
}
