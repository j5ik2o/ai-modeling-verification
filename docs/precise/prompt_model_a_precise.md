# Model A (Non-AVDM) 実装プロンプト - 正確版

---

## 🚫 重要な制約

**以下のルールを厳守してください：**

- ❌ **gitコマンドは一切操作しないでください**
- ✅ **modules/model-a-non-avdm/ のみを編集してください**
- ❌ **modules/model-b-avdm/ は絶対に見ないでください**（別の実装アプローチです）
- ❌ **spec-tests/ は絶対に見ないでください**（後で実行するテストケース、正解が書いてあります）
- ✅ **spec.md は参照OKです**（仕様書として公開されている情報）
- ✅ **modules/model-a-non-avdm/ 内のコードは自由に参照してください**
- ✅ 最後**`cargo test -p model-a-non-avdm`でテストがパスすることを確認してください**

---

## 📋 あなたの役割

あなたは充電セッションの課金ロジックを実装するRustエンジニアです。

既存の`Session`構造体に対して課金計算メソッドを実装してください。
以下の詳細仕様に従って、正確に実装してください。

ultrathink

---

## 🎯 実装タスク

`modules/model-a-non-avdm/src/session.rs` の `calculate_charge()` 関数を実装してください。
テストも実装してください。最後にテストが通ることを確認してください。

現在の状態：
```rust
pub fn calculate_charge(session: &mut Session) -> Result<u32, String> {
  todo!("AIに実装させる")
}
```

---

## 📐 詳細仕様

### 前提条件

1. **料金計算式**
   ```
   料金（円） = 単価（円/kWh） × 課金対象エネルギー（kWh）
   ```

2. **無料時間**
   - セッション開始から **5分間（300,000ミリ秒）** は無料
   - エネルギーはセッション時間に一様に分布するとみなす（時間比で按分）

3. **課金対象エネルギーの計算**
   ```
   セッション時間（ミリ秒） = 終了時刻 - 開始時刻
   無料時間（ミリ秒） = min(セッション時間, 300,000)
   課金対象時間（ミリ秒） = セッション時間 - 無料時間
   課金対象比率 = 課金対象時間 / セッション時間
   課金対象エネルギー（ミリkWh） = floor(総エネルギー（ミリkWh） × 課金対象比率)
   ```
   - 切り捨てにより課金対象エネルギーが総エネルギーを上回らないことを必ず保証する。

4. **金額計算**
   ```
   課金対象エネルギー（kWh） = 課金対象エネルギー（ミリkWh） / 1000.0
   料金（円） = floor(課金対象エネルギー（kWh） × 単価（円/kWh）)
   ```

5. **端数処理**
   - エネルギー按分は **floor（切り捨て）**
   - 金額は **1円未満を切り捨て**

6. **停止後課金禁止**
   - `already_billed` が `true` の場合はエラー
   - `status` が `"closed"` でない場合はエラー
   - 計算成功後は `already_billed` を `true` に設定

---

### 入力検証

以下の条件をすべて検証してください：

1. **セッション状態**
   - `already_billed == false` （未課金）
   - `status == "closed"` （停止済み）

2. **時刻の整合性**
   - `started_at` が `Some` である
   - `ended_at` が `Some` である
   - `ended_at > started_at` （停止時刻が開始時刻より後）

3. **エネルギーの範囲**
   - `kwh_milli <= 1,000,000` （上限: 1,000,000ミリkWh）
   - （負値は型により防がれている: u64）

4. **金額の範囲**
   - 計算結果が `1,000,000円` を超える場合はエラー

---

## 📝 実装要件

### 必須機能

1. **セッション状態検証**
   ```rust
   if session.already_billed {
     return Err("session already billed".to_string());
   }
   if session.status != "closed" {
     return Err(format!("status {} is not billable", session.status));
   }
   ```

2. **時刻検証**
   ```rust
   let started_at = session.started_at.ok_or("missing start timestamp")?;
   let ended_at = session.ended_at.ok_or("missing end timestamp")?;
   if ended_at <= started_at {
     return Err("invalid timeline: end <= start".to_string());
   }
   ```

3. **エネルギー上限検証**
   ```rust
   const MAX_KWH_MILLI: u64 = 1_000_000;
   if session.kwh_milli > MAX_KWH_MILLI {
     return Err("energy exceeds limit".to_string());
   }
   ```

4. **課金対象エネルギー計算**
   ```rust
   let duration_ms = (ended_at - started_at) as f64;
   let free_ms = 5.0 * 60.0 * 1000.0; // 5分 = 300,000ミリ秒
   let chargeable_ratio = ((duration_ms - free_ms).max(0.0)) / duration_ms;
   let billed_energy_milli = ((session.kwh_milli as f64) * chargeable_ratio).floor() as u64;
   ```

5. **金額計算**
   ```rust
   let billed_energy_kwh = billed_energy_milli as f64 / 1_000.0;
   let amount = (billed_energy_kwh * session.rate_yen_per_kwh as f64).floor() as u32;
   ```

6. **金額上限検証**
   ```rust
   const MAX_AMOUNT_YEN: u32 = 1_000_000;
   if amount > MAX_AMOUNT_YEN {
     return Err("amount exceeds limit".to_string());
   }
   ```

7. **状態更新**
   ```rust
   session.billed_kwh_milli = billed_energy_milli;
   session.already_billed = true;
   ```

8. **戻り値**
   ```rust
   Ok(amount)
   ```

---

## ✅ テストケース

以下のテストケースを実装してください：

### 1. 基本ケース
```rust
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
```

### 2. 異常系
```rust
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
```

---

## 🎬 実装を開始してください

上記の仕様とテストケースに従って、正確に実装してください。
質問があれば遠慮なく聞いてください。

---

## 📊 実験記録（実施後に記入）

### 実装プロセス
- **開始時刻**:
- **完了時刻**:
- **所要時間**:

### AIの挙動観察
- **参照したファイル**:
- **質問の内容**:
- **特記事項**:

### 実装結果
- **編集したファイル**:
  ```bash
  git diff --name-only
  ```

- **テスト実行結果**（ユニットテスト）:
  ```bash
  cargo test -p model-a-non-avdm
  ```

### 制約遵守チェック
- [ ] modules/model-b-avdm/ を参照していない
- [ ] spec-tests/ を参照していない
- [ ] modules/model-a-non-avdm/ のみ編集している
