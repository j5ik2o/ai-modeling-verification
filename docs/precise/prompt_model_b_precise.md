# Model B (AVDM) 実装プロンプト - 正確版

---

## 🚫 重要な制約

**以下のルールを厳守してください：**

- ❌ **gitコマンドは一切操作しないでください**
- ✅ **modules/model-b-avdm/ のみを編集してください**
- ❌ **modules/model-a-non-avdm/ は絶対に見ないでください**（別の実装アプローチです）
- ❌ **spec-tests/ は絶対に見ないでください**（後で実行するテストケース、正解が書いてあります）
- ✅ **spec.md は参照OKです**（仕様書として公開されている情報）
- ✅ **modules/model-b-avdm/ 内のコードは自由に参照してください**

---

## 📋 あなたの役割

あなたは充電セッションの課金ロジックを実装するRustエンジニアです。

既存の`Session` enumに対して課金計算メソッドを実装してください。
以下の詳細仕様に従って、正確に実装してください。

**重要**: 既存のドメインモデル型（`KwhMilli`, `MoneyYen`, `RateYenPerKwh`等）を積極的に活用してください。

---

## 🎯 実装タスク

`modules/model-b-avdm/src/session/base.rs` の `Session` 実装メソッドを完成させてください。
テストも実装してください。最後にテストが通ることを確認してください。
※`modules/model-b-avdm/src/session/tests.rs` は空にできません。

現在の状態（以下のメソッドが未実装）：
```rust
impl Session {
  pub fn stop(...) -> Result<Self, SessionValueError> {
    todo!("AIに実装させる")
  }

  pub fn bill_snapshot(...) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    todo!("AIに実装させる")
  }

  pub fn bill_after_stop(...) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    todo!("AIに実装させる")
  }
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
   - `modules/model-b-avdm/src/session/mod.rs` に `FREE_MILLISECONDS` 定数があるはずです
   - エネルギーはセッション時間に一様に分布するとみなす（時間比で按分）

3. **課金対象エネルギーの計算**
   ```
   セッション時間（ミリ秒） = 終了時刻 - 開始時刻
   無料時間（ミリ秒） = min(セッション時間, FREE_MILLISECONDS)
   課金対象時間（ミリ秒） = セッション時間 - 無料時間

   // 整数演算で精度を保つ
   課金対象エネルギー（ミリkWh） = (総エネルギー（ミリkWh） × 課金対象時間) / セッション時間
   ```
   - 整数除算の切り捨て結果を採用し、課金対象エネルギーが総エネルギーを超えないようにする。

4. **金額計算**
   - `RateYenPerKwh::charge(KwhMilli)` メソッドを使用
   - このメソッドが内部で切り捨て処理を行う

5. **端数処理**
   - エネルギー按分は整数演算（u128）で精度を保つ
   - 金額は `RateYenPerKwh::charge()` が自動的に切り捨て

6. **停止後課金禁止**
   - `Session::Closed` バリアントに対する `bill_after_stop()` は常にエラー
   - `Session::Active` のみが課金可能

---

## 📝 実装要件

### 1. `duration_millis()` - セッション時間計算

```rust
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
```

**検証:**
- `ended_at > started_at` を確認
- 負または0の場合は `InvalidTimeline` エラー

---

### 2. `billed_energy_for()` - 課金対象エネルギー計算

```rust
fn billed_energy_for(
  started_at: OffsetDateTime,
  ended_at: OffsetDateTime,
  total_energy: KwhMilli,
) -> Result<KwhMilli, SessionValueError> {
  let total_ms = Self::duration_millis(started_at, ended_at)?;
  let free_ms = FREE_MILLISECONDS.min(total_ms);
  let chargeable_ms = total_ms - free_ms;

  if chargeable_ms == 0 {
    return Ok(KwhMilli::zero());
  }

  // 整数演算で精度を保つ
  let total_energy_milli = total_energy.into_u128_milli(); // u128に変換
  let billed_energy_milli = (total_energy_milli * chargeable_ms) / total_ms;

  Ok(KwhMilli::from_milli(billed_energy_milli as u64))
}
```

**ポイント:**
- `FREE_MILLISECONDS` 定数を使用（`use super::FREE_MILLISECONDS;`）
- 無料時間を超えない場合は `KwhMilli::zero()` を返す
- 整数演算（u128）で按分計算し、精度を保つ
- `KwhMilli` 型が自動的に上限検証を行う

---

### 3. `bill_snapshot_for()` - 課金スナップショット計算

```rust
fn bill_snapshot_for(
  started_at: OffsetDateTime,
  rate: RateYenPerKwh,
  ended_at: OffsetDateTime,
  total_energy: KwhMilli,
) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
  let billed_energy = Self::billed_energy_for(started_at, ended_at, total_energy)?;
  let amount = rate.charge(billed_energy)?; // 金額計算（自動切り捨て）
  Ok((billed_energy, amount))
}
```

**ポイント:**
- `RateYenPerKwh::charge()` が自動的に金額上限検証と切り捨てを行う
- エラーは `SessionValueError` に変換される（`?` 演算子）

---

### 4. `bill_snapshot()` - 途中課金計算（公開メソッド）

```rust
pub fn bill_snapshot(
  &self,
  ended_at: OffsetDateTime,
  total_energy: KwhMilli,
) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
  match self {
    Session::Active { started_at, rate, .. } => {
      Self::bill_snapshot_for(*started_at, *rate, ended_at, total_energy)
    }
    Session::Closed {
      id,
      ended_at: stored_end,
      total_energy: stored_energy,
      billed_energy,
      charged_amount,
      ..
    } => {
      // 停止済みの場合、同じ条件なら記録済みの値を返す
      if *stored_end == ended_at && *stored_energy == total_energy {
        Ok((*billed_energy, *charged_amount))
      } else {
        Err(SessionValueError::AlreadyClosed { session_id: *id })
      }
    }
  }
}
```

**ポイント:**
- `Active` の場合: 新規計算
- `Closed` の場合: 同条件なら記録値、異なる条件ならエラー

---

### 5. `stop()` - セッション停止と確定課金

```rust
pub fn stop(
  self,
  ended_at: OffsetDateTime,
  total_energy: KwhMilli,
) -> Result<Self, SessionValueError> {
  match self {
    Session::Active { id, started_at, rate } => {
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
    Session::Closed { id, .. } => {
      Err(SessionValueError::AlreadyClosed { session_id: id })
    }
  }
}
```

**ポイント:**
- `Active` → `Closed` への状態遷移
- `Closed` に対して呼ばれたら `AlreadyClosed` エラー
- 課金計算を行い、結果を `Closed` に保存

---

### 6. `bill_after_stop()` - 停止後課金拒否

```rust
pub fn bill_after_stop(
  &self,
  ended_at: OffsetDateTime,
  total_energy: KwhMilli,
) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
  match self {
    Session::Active { .. } => {
      // アクティブなら通常のスナップショット
      self.bill_snapshot(ended_at, total_energy)
    }
    Session::Closed { id, .. } => {
      // 停止済みなら常にエラー
      Err(SessionValueError::AlreadyClosed { session_id: *id })
    }
  }
}
```

**ポイント:**
- `Active`: 通常のスナップショット計算
- `Closed`: 常にエラー（停止後課金禁止）

---

## ✅ テストケース

以下のテストケースを実装してください：

### 1. 基本ケース

```rust
use time::OffsetDateTime;
use super::*;

fn offset(minutes: i64) -> OffsetDateTime {
  OffsetDateTime::from_unix_timestamp(minutes * 60).unwrap()
}

#[test]
fn scenario1_six_minutes_charges_one_minute() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  let stopped = session.stop(
    offset(6),
    KwhMilli::from_milli(2_400), // 2.4 kWh
  ).unwrap();

  assert_eq!(stopped.billed_energy().unwrap().into_u64_milli(), 400); // 0.4 kWh
  assert_eq!(stopped.charged_amount().unwrap().into_u64_yen(), 20); // 20円
}

#[test]
fn scenario2_four_minutes_is_free() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(80).unwrap(),
  );

  let stopped = session.stop(
    offset(4),
    KwhMilli::from_milli(1_000),
  ).unwrap();

  assert_eq!(stopped.billed_energy().unwrap().into_u64_milli(), 0);
  assert_eq!(stopped.charged_amount().unwrap().into_u64_yen(), 0);
}

#[test]
fn scenario3_exactly_five_minutes_is_free() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(100).unwrap(),
  );

  let stopped = session.stop(
    offset(5),
    KwhMilli::from_milli(3_000),
  ).unwrap();

  assert_eq!(stopped.billed_energy().unwrap().into_u64_milli(), 0);
  assert_eq!(stopped.charged_amount().unwrap().into_u64_yen(), 0);
}
```

### 2. 異常系

```rust
#[test]
fn error_when_stop_already_closed() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  let stopped = session.stop(offset(6), KwhMilli::from_milli(2_400)).unwrap();

  // 再度stopしようとするとエラー
  let result = stopped.stop(offset(10), KwhMilli::from_milli(4_000));
  assert!(matches!(result, Err(SessionValueError::AlreadyClosed { .. })));
}

#[test]
fn error_when_bill_after_stop() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  let stopped = session.stop(offset(6), KwhMilli::from_milli(2_400)).unwrap();

  // 停止後に課金しようとするとエラー
  let result = stopped.bill_after_stop(offset(10), KwhMilli::from_milli(4_000));
  assert!(matches!(result, Err(SessionValueError::AlreadyClosed { .. })));
}

#[test]
fn error_when_end_before_start() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(10), // 開始: 10分
    RateYenPerKwh::try_new(50).unwrap(),
  );

  // 終了時刻が開始時刻より前
  let result = session.stop(offset(5), KwhMilli::from_milli(1_000));
  assert!(matches!(result, Err(SessionValueError::InvalidTimeline { .. })));
}

#[test]
fn error_when_energy_over_limit() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(50).unwrap(),
  );

  // KwhMilli::from_milli() が上限を超える値を拒否する
  // または try_from を使用
  let result = KwhMilli::try_from_i64(1_000_001);
  // 型レベルで防がれる
}

#[test]
fn error_when_amount_over_limit() {
  let session = Session::new_active(
    SessionId::new(Uuid::nil()),
    offset(0),
    RateYenPerKwh::try_new(20_000).unwrap(), // 高額な単価
  );

  // RateYenPerKwh::charge() が上限超過を検出してエラー
  let result = session.stop(offset(20), KwhMilli::from_milli(100_000));
  // MoneyYenの上限チェックで弾かれる
}
```

---

## 💡 重要なヒント

### 既存のドメインモデル型を活用

1. **`KwhMilli` (modules/model-b-avdm/src/session/energy.rs)**
   - `new(BoundedU64<MAX_KWH_MILLI>)`: 上限付き生成（空気読まずに上限超過できない）
   - `try_new(u64)`: 生の値から検証付き生成
   - `from_milli(u64)`: ミリkWhから生成
   - `try_from_i64(i64)`: 負値チェック付き変換
   - `into_u64_milli()`: u64値取得
   - `into_u128_milli()`: u128値取得（按分計算用）
   - `zero()`: ゼロ値生成
   - 自動的に上限検証（1,000,000ミリkWh）

2. **`MoneyYen` (modules/model-b-avdm/src/session/money.rs)**
   - `new(BoundedU64<MAX_YEN>)`: 上限付き生成
   - `try_new(u64)`: 生の値から検証付き生成
   - `into_u64_yen()`: u64値取得
   - 自動的に上限検証（1,000,000円）

3. **`RateYenPerKwh` (modules/model-b-avdm/src/session/rate.rs)**
   - `new(NonZeroU32)`: 非ゼロ単価を直接生成
   - `try_new(u32)`: 生の値から検証付き生成
   - `charge(KwhMilli) -> Result<MoneyYen, _>`: 金額計算
   - 自動的に切り捨て処理

4. **`SessionValueError` (modules/model-b-avdm/src/session/errors.rs)**
   - `InvalidTimeline { started_at, ended_at }`: 時刻逆転エラー
   - `AlreadyClosed { session_id }`: 停止済みエラー
   - その他のエラーバリアント

5. **`BoundedU64<MAX>` (modules/model-b-avdm/src/session/bounded.rs)**
   - 上限付き `u64` を表すヘルパー
   - `BoundedU64::<MAX>::new(value)` で閾値チェックを強制
   - `KwhMilli` や `MoneyYen` の厳格なコンストラクタで利用

---

## 🎬 実装を開始してください

上記の仕様とテストケースに従って、既存のドメインモデル型を活用しながら正確に実装してください。
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
  cargo test -p model-b-avdm
  ```

### 制約遵守チェック
- [ ] modules/model-a-non-avdm/ を参照していない
- [ ] spec-tests/ を参照していない
- [ ] modules/model-b-avdm/ のみ編集している
